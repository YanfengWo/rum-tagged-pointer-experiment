#!/usr/bin/env bash
# Linux reproduction; Git, Python3, taskset, Rust1.85 + nightly2026-10-05 Miri.
set -euo pipefail
study="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$study/../../.." && pwd)"
[[ "$#" = 2 ]] || { printf 'Usage: bash run-study.sh NEW_SOURCE_DIRECTORY NEW_EVIDENCE_DIRECTORY\n'; exit 2; }
overlay="$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).resolve())' "$1")"
out="$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).resolve())' "$2")"
python3 - "$overlay" "$out" <<'PY'
from pathlib import Path
import sys
a,b=map(Path,sys.argv[1:])
assert a!=b and a not in b.parents and b not in a.parents
for p in [a,b]:assert not p.exists() or (p.is_dir() and not any(p.iterdir()))
b.mkdir(parents=True,exist_ok=True)
PY
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
printf 'RUNNING\n' > "$out/status.txt"
trap 'task_exit=$?; if [ "$task_exit" -ne 0 ]; then printf "FAIL exit %s\n" "$task_exit" > "$out/status.txt"; fi' EXIT
bounded 180 python3 "$study/prepare-overlay.py" --output "$overlay" --metadata "$out/overlay.json" > "$out/prepare.log" 2>&1
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$overlay/target-symbol-ops}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}" CARGO_PROFILE_DEV_DEBUG=0
unset RUM_PROFILE_REPR
cd "$overlay/research/tagged-pointer/symbol-operations"
bounded 900 cargo +1.85.0 test --locked > "$out/native.log" 2>&1
bounded 1800 cargo +1.85.0 test --manifest-path "$overlay/Cargo.toml" --workspace --locked -- --test-threads=1 > "$out/workspace.log" 2>&1
for model in stacked tree; do
    flags=-Zmiri-strict-provenance
    if [ "$model" = tree ]; then flags="$flags -Zmiri-tree-borrows"; fi
    MIRIFLAGS="$flags" bounded 1200 cargo +nightly-2026-10-05 miri test --locked > "$out/miri-$model.log" 2>&1
done
bounded 900 cargo +1.85.0 build --release --locked > "$out/build.log" 2>&1
binary="$CARGO_TARGET_DIR/release/rum-symbol-operations"
sha256sum "$binary" > "$out/binary.sha256"
bounded 300 "$binary" validate ../expression-census/corpus > "$out/semantic.log" 2>&1
cpu="$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')"
{
    printf 'CPU %s; 3 rotating runs; round0 excluded; run2 reversed; common RandomState per process; counters absent\n' "$cpu"
    uname -srvm
    rustup run 1.85.0 rustc --version --verbose
} > "$out/environment.txt"
for run in 1 2 3; do
    bounded 900 taskset -c "$cpu" "$binary" timing ../expression-census/corpus "$run" > "$out/run-$run.tsv" 2> "$out/run-$run.stderr"
done
bounded 180 cargo +1.85.0 fmt --check > "$out/format.log" 2>&1
printf 'PASS\n' > "$out/status.txt"
# Analyze from unchanged repository to preserve pinned baseline controls.
bounded 180 python3 "$study/analyze.py" "$out"
