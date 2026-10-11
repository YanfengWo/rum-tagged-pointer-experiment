#!/usr/bin/env bash
# Linux reproduction using Git, Python 3, taskset and the user's rustup environment.
# Install Rust 1.85.0 + rustfmt, and nightly-2026-10-05 + miri/rust-src first.
set -euo pipefail
study="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$study/../../.." && pwd)"
overlay=""
out=""
native="1.85.0"
miri="nightly-2026-10-05"
cpu=""
usage() {
    printf 'Usage: bash run-study.sh --overlay NEW_SOURCE_DIR --evidence NEW_EVIDENCE_DIR [--cpu CPU] [--native-toolchain NAME] [--miri-toolchain NAME]\n'
}
while [[ "$#" -gt 0 ]]; do
    case "$1" in
        --overlay|--evidence|--cpu|--native-toolchain|--miri-toolchain)
            [[ "$#" -ge 2 ]] || { usage; exit 2; }
            case "$1" in
                --overlay) overlay="$2";;
                --evidence) out="$2";;
                --cpu) cpu="$2";;
                --native-toolchain) native="$2";;
                --miri-toolchain) miri="$2";;
            esac
            shift 2;;
        --help|-h) usage; exit 0;;
        *) usage; exit 2;;
    esac
done
[[ -n "$overlay" && -n "$out" ]] || { usage; exit 2; }
overlay="$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).resolve())' "$overlay")"
out="$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).resolve())' "$out")"
python3 - "$overlay" "$out" <<'PY'
from pathlib import Path
import sys
overlay, evidence = map(Path, sys.argv[1:])
if overlay == evidence or overlay in evidence.parents or evidence in overlay.parents:
    raise SystemExit("source and evidence directories must be separate")
for path in (overlay, evidence):
    if path.exists() and (not path.is_dir() or any(path.iterdir())):
        raise SystemExit("source and evidence directories must be absent or empty")
evidence.mkdir(parents=True, exist_ok=True)
PY
printf 'RUNNING\n' > "$out/status.txt"
trap 'task_exit=$?; if [ "$task_exit" -ne 0 ]; then printf "FAIL exit %s; inspect logs\n" "$task_exit" > "$out/status.txt"; fi' EXIT
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
bounded 300 python3 "$study/prepare-overlay.py" --output "$overlay" --metadata "$out/overlay.json" > "$out/prepare.log" 2>&1
source_study="$overlay/research/tagged-pointer/layout-design"
corpus="$overlay/research/tagged-pointer/expression-census/corpus"
export CARGO_TARGET_DIR="$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).resolve())' "${CARGO_TARGET_DIR:-$overlay/target-layout}")"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}"
unset RUM_PROFILE_REPR
cd "$source_study"
bounded 180 cargo +"$native" fmt --check > "$out/format.log" 2>&1
bounded 900 cargo +"$native" test --locked > "$out/native.log" 2>&1
bounded 900 cargo +"$native" test --all-features --locked > "$out/native-all-features.log" 2>&1
cd "$overlay"
bounded 900 cargo +"$native" test -p rum-core --locked > "$out/core-api.log" 2>&1
bounded 1800 cargo +"$native" test --workspace --locked -- --test-threads=1 > "$out/workspace.log" 2>&1
cd "$source_study"
for model in stacked tree; do
    flags="-Zmiri-strict-provenance"
    if [[ "$model" == tree ]]; then flags="$flags -Zmiri-tree-borrows"; fi
    MIRIFLAGS="$flags" bounded 1200 cargo +"$miri" miri test --locked --lib > "$out/miri-$model.log" 2>&1
done
bounded 1200 cargo +"$native" build --release --locked --features corpus > "$out/timing-build.log" 2>&1
binary="$CARGO_TARGET_DIR/release/layout-study"
bounded 600 "$binary" validate "$corpus" > "$out/semantic.log" 2>&1
cpu="$(python3 -c 'import os,sys; allowed=os.sched_getaffinity(0); cpu=int(sys.argv[1]) if sys.argv[1] else min(allowed); assert cpu in allowed; print(cpu)' "$cpu")"
{
    printf 'Pinned upstream: 77029938f999b14dbe5b0755bebdf98e51517ae6\n'
    printf 'CPU affinity: %s; no host isolation claimed\n' "$cpu"
    printf 'Three runs; one warmup and nine measured rounds; allocation counters absent\n'
    printf 'RUM_PROFILE_REPR unset; Original wrappers retain disabled profiling branches, research wrappers omit them\n'
    printf 'Dataset, setup allocation and operation order rotate between runs/rounds\n'
    uname -srvm
    python3 --version
    rustup run "$native" rustc --version --verbose
    rustup run "$miri" rustc --version --verbose
} > "$out/environment.txt" 2>&1
for run in 1 2 3; do
    bounded 600 taskset -c "$cpu" "$binary" timing "$corpus" "$run" > "$out/run-$run.tsv" 2> "$out/run-$run.stderr"
done
bounded 1200 cargo +"$native" build --release --locked --features corpus,allocation-count > "$out/allocation-build.log" 2>&1
bounded 600 "$binary" measure "$corpus" "$out/allocations.json" > "$out/allocations.log" 2>&1
printf 'PASS\n' > "$out/status.txt"
if [[ -f "$study/timings.py" ]]; then
    bounded 180 python3 "$study/timings.py" --evidence "$out" > "$out/timing-analysis.log" 2>&1
fi
if [[ -f "$study/analyze.py" ]]; then
    # Analyze from the unchanged repository: its pinned core and historical
    # evidence are the controls; the isolated core intentionally has an extension.
    # Do not redirect its output into evidence: the analyzer hashes all raw
    # evidence files, and a log still being written would invalidate that hash.
    bounded 180 python3 "$study/analyze.py" "$out"
fi
printf 'PASS: isolated layout study completed; inspect the requested evidence directory\n'
