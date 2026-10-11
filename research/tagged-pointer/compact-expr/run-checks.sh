#!/usr/bin/env bash
set -euo pipefail
probe="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$probe/../../.." && pwd)"
out="${1:-$probe/target/evidence}"
mkdir -p "$out"
out="$(cd -- "$out" && pwd)"
printf 'RUNNING\n' > "$out/status.txt"
trap 'task_exit=$?; if [ "$task_exit" -ne 0 ]; then printf "FAIL exit %s; inspect logs\n" "$task_exit" > "$out/status.txt"; fi' EXIT
cd "$probe"
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
bounded 180 cargo +1.85.0 fmt --check > "$out/format.log" 2>&1
bounded 600 cargo +1.85.0 test --locked > "$out/native.log" 2>&1
# Install nightly-2026-10-05 with miri and rust-src before enabling this option.
if [[ "${RUN_MIRI:-0}" == 1 ]]; then
    MIRIFLAGS="-Zmiri-strict-provenance" bounded 900 cargo +nightly-2026-10-05 miri test --locked --lib > "$out/miri-stacked.log" 2>&1
    MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-tree-borrows" bounded 900 cargo +nightly-2026-10-05 miri test --locked --lib > "$out/miri-tree.log" 2>&1
fi
bounded 600 cargo +1.85.0 run --release --locked --bin representation-bench > "$out/representation.tsv" 2> "$out/bench-build.log"
printf 'PASS\n' > "$out/status.txt"
