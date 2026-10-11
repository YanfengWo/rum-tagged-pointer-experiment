#!/usr/bin/env bash
set -euo pipefail
study="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$study/../../.." && pwd)"
out="${1:?Usage: bash run-study.sh ABSOLUTE_OUTPUT_DIRECTORY}"
mkdir -p "$out"
out="$(cd -- "$out" && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rum-selective-sharing-build}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}" CARGO_PROFILE_DEV_DEBUG=0
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
cd "$study"
printf 'RUNNING\n' > "$out/status.txt"
trap 'task_exit=$?; if [ "$task_exit" -ne 0 ]; then printf "FAIL exit %s\n" "$task_exit" > "$out/status.txt"; fi' EXIT
bounded 600 cargo +1.85.0 test --locked --offline --lib > "$out/native.log" 2>&1
bounded 600 cargo +1.85.0 test --locked --offline --all-features > "$out/native-all-features.log" 2>&1
for model in stacked tree; do
    flags="-Zmiri-strict-provenance"
    if [[ "$model" == tree ]]; then flags="$flags -Zmiri-tree-borrows"; fi
    MIRIFLAGS="$flags" bounded 900 cargo +nightly-2026-10-05 miri test --locked --offline --lib > "$out/miri-$model.log" 2>&1
done
bounded 900 cargo +1.85.0 build --release --locked --offline --features corpus > "$out/timing-build.log" 2>&1
binary="$CARGO_TARGET_DIR/release/selective-study"
bounded 300 "$binary" validate ../expression-census/corpus > "$out/semantic.log" 2>&1
cpu="$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')"
printf 'CPU affinity: %s; 1 warmup + 9 measured rounds; 3 runs; run 2 reverses dataset/case order; counters absent; probe included; root >=64, budget 256, repeats >=8, cache 128\n' "$cpu" > "$out/environment.txt"
for run in 1 2 3; do
    mode=timing
    if [[ "$run" == 2 ]]; then mode=timing-reverse; fi
    bounded 300 taskset -c "$cpu" "$binary" "$mode" ../expression-census/corpus > "$out/run-$run.tsv" 2> "$out/run-$run.stderr"
done
bounded 900 cargo +1.85.0 build --release --locked --offline --features corpus,allocation-count > "$out/allocation-build.log" 2>&1
bounded 300 "$binary" measure ../expression-census/corpus "$out/allocations.json" > "$out/allocations.log" 2>&1
bounded 180 cargo +1.85.0 fmt --check > "$out/format.log" 2>&1
printf 'PASS\n' > "$out/status.txt"
python3 analyze.py "$out"
