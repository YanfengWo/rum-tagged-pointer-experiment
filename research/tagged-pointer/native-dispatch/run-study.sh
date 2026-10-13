#!/usr/bin/env bash
# Linux reproduction. Install the two pinned Rust toolchains and rustfmt/Miri.
set -euo pipefail
study="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$study/../../.." && pwd)"
[[ "$#" = 2 ]] || { printf 'Usage: bash run-study.sh NEW_SOURCE_PARENT NEW_EVIDENCE_DIRECTORY\n'; exit 2; }
sources="$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).resolve())' "$1")"
out="$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).resolve())' "$2")"
python3 - "$sources" "$out" <<'PY'
from pathlib import Path
import sys
a,b=map(Path,sys.argv[1:]);assert a!=b and a not in b.parents and b not in a.parents
for p in [a,b]:assert not p.exists() or (p.is_dir() and not any(p.iterdir()))
a.mkdir(parents=True,exist_ok=True);b.mkdir(parents=True,exist_ok=True)
PY
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}" CARGO_PROFILE_DEV_DEBUG=0
unset RUM_PROFILE_REPR RUM_PROFILE_ALLOC RUM_PROFILE_LEXICAL_SUBSTITUTION RUM_PROFILE_BUILTIN_TIMING
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
modes=(base api raw24 safe24 packed16 tagged16 sort-api sort-24 sort-safe16 sort-tagged16)
for mode in "${modes[@]}"; do
    overlay="$sources/$mode"
    bounded 180 python3 "$study/prepare-overlay.py" --output "$overlay" --metadata "$out/$mode-overlay.json" --mode "$mode" > "$out/$mode-prepare.log" 2>&1
    export CARGO_TARGET_DIR="$sources/target-$mode"
    cd "$overlay"
    features=()
    if [[ "$mode" != base && "$mode" != api && "$mode" != sort-api ]]; then
        if [[ "$mode" = sort-* ]]; then features=(--features "research-$mode"); else features=(--features "research-dispatch-$mode"); fi
    fi
    bounded 900 cargo +1.85.0 build --release -p rum-cli "${features[@]}" > "$out/$mode-build.log" 2>&1
    cp Cargo.lock "$out/$mode-Cargo.lock"
    sha256sum "$CARGO_TARGET_DIR/release/rum-cli" > "$out/$mode-binary.sha256"
    bounded 180 cargo +1.85.0 fmt --check > "$out/$mode-format.log" 2>&1
    if [[ "$mode" != base ]]; then bounded 1800 cargo +1.85.0 test --workspace --locked "${features[@]}" -- --test-threads=1 > "$out/$mode-workspace.log" 2>&1; fi
    if [[ "$mode" != base && "$mode" != api && "$mode" != sort-api ]]; then
        package=native-dispatch;n=3;native_features=()
        if [[ "$mode" = sort-* ]]; then
            package=native-sort;n=2
            if [[ "$mode" != sort-24 ]]; then native_features=(--features "${mode#sort-}"); fi
        elif [[ "$mode" != raw24 ]]; then native_features=(--features "$mode"); fi
        manifest="$overlay/research/tagged-pointer/$package/Cargo.toml"
        bounded 600 cargo +1.85.0 test --manifest-path "$manifest" "${native_features[@]}" > "$out/$mode-native.log" 2>&1
        cp "$overlay/research/tagged-pointer/$package/Cargo.lock" "$out/$mode-study-Cargo.lock"
        for model in stacked tree; do
            flags=-Zmiri-strict-provenance
            if [[ "$model" = tree ]]; then flags="$flags -Zmiri-tree-borrows"; fi
            MIRIFLAGS="$flags" bounded 1200 cargo +nightly-2026-10-05 miri test --manifest-path "$manifest" --locked "${native_features[@]}" > "$out/$mode-miri-$model.log" 2>&1
        done
    fi
done
printf 'CHECKS_PASS\n' > "$out/status.txt"
printf 'SORT_CHECKS_PASS\n' > "$out/sort-status.txt"
cpu="$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')"
cd "$sources/base"
bounded 600 bash scripts/core-language-microbench.sh --rum-cli "$sources/target-base/release/rum-cli" --output-dir "$out/profile-base" --reps 3 --timeout 300 > "$out/profile-base.log" 2>&1
bounded 600 env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 RUM_PROFILE_BUILTIN_TIMING=1 "$sources/target-base/release/rum-cli" --file "$out/profile-base/core-language-microbench.rum" > "$out/profile-enabled.stdout" 2> "$out/profile-enabled.stderr"
printf 'CPU %s; 3 rotating counter-off process runs; unchanged suite scale5/reps9\n' "$cpu" > "$out/timing-plan.txt"
for run in 1 2 3; do
    if [[ "$run" = 1 ]]; then order=(base api raw24 safe24 packed16 tagged16 sort-api sort-24 sort-safe16 sort-tagged16); fi
    if [[ "$run" = 2 ]]; then order=(sort-tagged16 sort-safe16 sort-24 sort-api tagged16 packed16 safe24 raw24 api base); fi
    if [[ "$run" = 3 ]]; then order=(packed16 tagged16 sort-api sort-24 sort-safe16 sort-tagged16 base api raw24 safe24); fi
    for mode in "${order[@]}"; do
        cd "$sources/$mode"
        if [[ "$run" = 1 ]]; then
            bounded 300 "$sources/target-$mode/release/rum-cli" --file "$study/../native-sort/acceptance.rum" > "$out/$mode-acceptance.stdout" 2> "$out/$mode-acceptance.stderr"
            grep -q NATIVE_SORT_ACCEPTANCE_PASS "$out/$mode-acceptance.stdout"
        fi
        bounded 900 taskset -c "$cpu" bash scripts/core-language-microbench.sh --rum-cli "$sources/target-$mode/release/rum-cli" --output-dir "$out/timing-$run-$mode" --reps 9 --scale 5 --timeout 600 > "$out/timing-$run-$mode.log" 2>&1
    done
done
printf 'TIMINGS_PASS\n' > "$out/timing-status.txt"
bounded 180 python3 "$study/analyze.py" "$out"
