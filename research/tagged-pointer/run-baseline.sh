#!/usr/bin/env bash
# Portable copy is published under research/tagged-pointer/ in the rum checkout.
set -euo pipefail
research_root="$(cd -- "$(dirname -- "$0")" && pwd)"
repo_root="$(cd -- "$research_root/../.." && pwd)"
output_root="${1:-$repo_root/target/research-evidence}"
mkdir -p "$output_root"
cd "$repo_root"
export CARGO_TARGET_DIR="$repo_root/target/research-base"
export CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0
unset RUM_PROFILE_REPR RUM_PROFILE_ALLOC
git rev-parse HEAD > "$output_root/checkout-commit.txt"
rustc +1.85.0 -Vv > "$output_root/rustc-version.txt"
python3 scripts/run-bounded.py --timeout 180 -- cargo +1.85.0 fmt --check > "$output_root/fmt.txt" 2>&1
python3 scripts/run-bounded.py --timeout 1200 -- cargo +1.85.0 test --workspace --locked > "$output_root/tests.txt" 2>&1
python3 scripts/run-bounded.py --timeout 900 -- cargo +1.85.0 build --release --locked -p rum-cli > "$output_root/release-build.txt" 2>&1
python3 scripts/run-bounded.py --timeout 300 -- cargo +1.85.0 run --release --locked --manifest-path "$research_root/layout-probe/Cargo.toml" > "$output_root/layout.json" 2> "$output_root/layout-build.txt"
python3 scripts/run-bounded.py --timeout 600 -- bash scripts/core-language-microbench.sh --rum-cli "$CARGO_TARGET_DIR/release/rum-cli" --output-dir "$output_root/timing-1" --timeout 300 > "$output_root/timing-1.txt" 2>&1
python3 scripts/run-bounded.py --timeout 600 -- bash scripts/core-language-microbench.sh --rum-cli "$CARGO_TARGET_DIR/release/rum-cli" --output-dir "$output_root/timing-2" --timeout 300 > "$output_root/timing-2.txt" 2>&1
python3 scripts/run-bounded.py --timeout 600 -- bash scripts/core-memory-microbench.sh --rum-cli "$CARGO_TARGET_DIR/release/rum-cli" --output-dir "$output_root/memory" --no-rss > "$output_root/memory.txt" 2>&1
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_TARGET_DIR="$repo_root/target/research-alloc" cargo +1.85.0 build --release --locked -p rum-cli --features alloc-count > "$output_root/alloc-build.txt" 2>&1
python3 scripts/run-bounded.py --timeout 600 -- env RUM_PROFILE_ALLOC=1 RUM_PROFILE_REPR=1 bash scripts/core-memory-microbench.sh --rum-cli "$repo_root/target/research-alloc/release/rum-cli" --output-dir "$output_root/alloc" --no-rss > "$output_root/alloc.txt" 2>&1
printf 'Checks finished for the recorded checkout; inspect logs before comparison.\n'
