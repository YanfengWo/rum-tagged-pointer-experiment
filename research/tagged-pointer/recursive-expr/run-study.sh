#!/usr/bin/env bash
set -euo pipefail
study="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$study/../../.." && pwd)"
out="${1:-$study/target/study-evidence}"
mkdir -p "$out"
out="$(cd -- "$out" && pwd)"
printf 'RUNNING\n' > "$out/status.txt"
trap 'task_exit=$?; if [ "$task_exit" -ne 0 ]; then printf "FAIL exit %s; inspect logs\n" "$task_exit" > "$out/status.txt"; fi' EXIT
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
cd "$study"
bounded 180 cargo +1.85.0 fmt --check > "$out/format.log" 2>&1
bounded 600 cargo +1.85.0 test --locked > "$out/native.log" 2>&1
bounded 600 cargo +1.85.0 test --all-features --locked > "$out/native-all-features.log" 2>&1
if [[ "${RUN_MIRI:-0}" == 1 ]]; then
    for model in stacked tree; do
        flags="-Zmiri-strict-provenance"
        if [[ "$model" == tree ]]; then flags="$flags -Zmiri-tree-borrows"; fi
        MIRIFLAGS="$flags" bounded 900 cargo +nightly-2026-10-05 miri test --locked --lib > "$out/miri-$model.log" 2>&1
        MIRIFLAGS="$flags" bounded 900 cargo +nightly-2026-10-05 miri test --locked --bin recursive-study > "$out/fixture-miri-$model.log" 2>&1
        MIRIFLAGS="$flags" bounded 900 cargo +nightly-2026-10-05 miri run --locked --features allocation-count --bin recursive-study > "$out/allocator-miri-$model.tsv" 2> "$out/allocator-miri-$model.log"
    done
fi
bounded 600 cargo +1.85.0 build --release --locked --features allocation-count > "$out/allocation-build.log" 2>&1
binary="${CARGO_TARGET_DIR:-$study/target}/release/recursive-study"
bounded 300 "$binary" > "$out/allocations.tsv" 2> "$out/allocations.stderr"
# Rebuild without instrumentation before measuring any timings.
bounded 600 cargo +1.85.0 build --release --locked > "$out/timing-build.log" 2>&1
cpu="$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')"
printf 'CPU affinity: %s; 1 warmup + 9 measured rounds; 3 runs; representations rotate; counters absent from timing build\n' "$cpu" > "$out/environment.txt"
for run in 1 2 3; do
    bounded 300 taskset -c "$cpu" "$binary" > "$out/run-$run.tsv" 2> "$out/run-$run.stderr"
done
printf 'PASS\n' > "$out/status.txt"
extra=()
if [[ "${RUN_MIRI:-0}" == 1 ]]; then extra+=(--require-miri); fi
python3 "$study/analyze.py" "$out" "${extra[@]}"
