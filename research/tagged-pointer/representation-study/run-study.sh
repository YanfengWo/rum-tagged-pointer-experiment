#!/usr/bin/env bash
set -euo pipefail
study="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$study/../../.." && pwd)"
probe="$study/../compact-expr"
out="${1:-$study/target/study-evidence}"
mkdir -p "$out"
out="$(cd -- "$out" && pwd)"
printf 'RUNNING\n' > "$out/status.txt"
trap 'task_exit=$?; if [ "$task_exit" -ne 0 ]; then printf "FAIL exit %s; inspect logs\n" "$task_exit" > "$out/status.txt"; fi' EXIT
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
cd "$probe"
bounded 180 cargo +1.85.0 fmt --check > "$out/probe-format.log" 2>&1
bounded 600 cargo +1.85.0 test --locked > "$out/native.log" 2>&1
if [[ "${RUN_MIRI:-0}" == 1 ]]; then
    MIRIFLAGS="-Zmiri-strict-provenance" bounded 900 cargo +nightly-2026-10-05 miri test --locked --lib > "$out/miri-stacked.log" 2>&1
    MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-tree-borrows" bounded 900 cargo +nightly-2026-10-05 miri test --locked --lib > "$out/miri-tree.log" 2>&1
fi
cd "$study"
bounded 180 cargo +1.85.0 fmt --check > "$out/study-format.log" 2>&1
bounded 600 cargo +1.85.0 test --locked --test compatibility > "$out/compatibility-native.log" 2>&1
if [[ "${RUN_MIRI:-0}" == 1 ]]; then
    MIRIFLAGS="-Zmiri-strict-provenance" bounded 900 cargo +nightly-2026-10-05 miri test --locked --test compatibility > "$out/compatibility-miri-stacked.log" 2>&1
    MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-tree-borrows" bounded 900 cargo +nightly-2026-10-05 miri test --locked --test compatibility > "$out/compatibility-miri-tree.log" 2>&1
fi
bounded 600 cargo +1.85.0 build --release --locked > "$out/build.log" 2>&1
binary="${CARGO_TARGET_DIR:-$study/target}/release/rum-representation-study"
cpu="$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')"
printf 'CPU affinity: %s; rounds: 1 warmup + 15 measured; runs: 3; five representations interleaved\n' "$cpu" > "$out/environment.txt"
for run in 1 2 3; do
    bounded 300 taskset -c "$cpu" "$binary" > "$out/run-$run.tsv" 2> "$out/run-$run.stderr"
done
printf 'PASS\n' > "$out/status.txt"
