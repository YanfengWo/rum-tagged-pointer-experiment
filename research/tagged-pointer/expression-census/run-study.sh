#!/usr/bin/env bash
set -euo pipefail
study="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$study/../../.." && pwd)"
out="${1:-$study/target/census-evidence}"
mkdir -p "$out"
out="$(cd -- "$out" && pwd)"
printf 'RUNNING\n' > "$out/status.txt"
trap 'task_exit=$?; if [ "$task_exit" -ne 0 ]; then printf "FAIL exit %s; inspect logs\n" "$task_exit" > "$out/status.txt"; fi' EXIT
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
cd "$study"
bounded 180 cargo +1.85.0 fmt --check > "$out/format.log" 2>&1
bounded 600 cargo +1.85.0 test --locked > "$out/native.log" 2>&1
bounded 600 cargo +1.85.0 test --locked --all-features > "$out/native-all-features.log" 2>&1
bounded 900 cargo +1.85.0 build --release --locked > "$out/build.log" 2>&1
binary="${CARGO_TARGET_DIR:-$study/target}/release/rum-expression-census"
bounded 300 "$binary" census "$study/corpus" "$out/census.json" > "$out/census.log" 2>&1
bounded 900 cargo +1.85.0 build --release --locked --features allocation-count > "$out/allocation-build.log" 2>&1
bounded 300 "$binary" measure "$study/corpus" "$out/allocations.json" > "$out/allocations.log" 2>&1
printf 'PASS\n' > "$out/status.txt"
python3 analyze.py "$out"
