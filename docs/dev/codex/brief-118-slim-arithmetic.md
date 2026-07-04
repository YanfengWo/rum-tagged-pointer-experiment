# Brief 118 — file-slimming batch 5: arithmetic.rs (42KB, HOT — the Plus/Times bodies)

## Goal

Fifth slimming batch. Split `crates/rum-runtime/src/runtime/arithmetic.rs`
(41,921 chars — the kernel-resident bodies of the engine's hottest handlers:
Plus/Times/Power canonicalization plus the rest of the arithmetic family
behind `ArithmeticDataBuiltin`, and the `combine_*` methods stdlib families
call) into an `arithmetic/` directory per brief-114's SPLITTING PRINCIPLES
(split map first). Zero behavior change; workspace green (1301/0); every new
file ≤20k; **microbench A/B required, 115/116-style** (you run it; rebuild
release rum-cli before the post-edit run; the gate reproduces independently
if residue is on-edge).

## Context pointers

- Read `AGENTS.md`, `docs/code-organization-guidelines.md`, brief-114
  principles, briefs 115/116 (hot-split precedents).
- Content (verify before mapping): 27 handler bodies + private helpers.
  Suggested axis BY NUMERIC DOMAIN (verify cohesion; adjust with
  justification): additive/multiplicative canonicalization (Plus/Times/
  combine_* — the hottest; consider keeping them together if they share
  helpers), Power/PowerMod, rounding/parts (Floor/Ceiling/Round/
  IntegerPart/FractionalPart/Abs/Sign/Minus/Subtract/Divide), integer ops
  (Mod/Quotient/GCD/LCM), factorial family, extremum (Max/Min/MinMax/
  exact_extremum), Total/Numerator/Denominator, shared exact-rational
  helpers (fast_integer_plus etc. — keep adjacent to their hot callers if
  cohesion says so).
- HOT RED LINES: bodies VERBATIM — exact-rational arithmetic, overflow
  paths, canonical ordering, `fast_integer_plus`-style fast-outs, message
  text/order all byte-stable. `ArithmeticDataBuiltin` +
  `eval_arithmetic_data_builtin` (the wrapper facade) keep working
  unchanged. The kernel attribute vocabulary and the drift test are
  untouched.
- A/B judged labels: table-iterator-plus (the 113-regression sentinel!),
  do-small, downvalue-*, set-ownvalue, literal-integer, replaceall-simple.
  Machine noise ±10-15% on repeated cycles — consistent direction only; one
  rerun allowed.

## Scope

- MAY modify: `arithmetic.rs` → `arithmetic/` (directory); `use` lines
  inside; parent mod declaration only if syntax demands (likely zero).
- MUST NOT touch: any caller outside the directory (incl. the stdlib
  wrappers and kernel_api), any behavior, `Cargo.toml`/`Cargo.lock`, any
  `mma-hol` clone, `docs/**` except your report, `.git`.

## Verification (MANDATORY)

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_118_micro_base   # BEFORE any edit
# ... split ...
cargo fmt --check
cargo test --workspace
cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_118_micro_new
ls crates/rum-runtime/src/runtime/arithmetic.rs 2>&1                                                # No such file
find crates/rum-runtime/src/runtime/arithmetic -name "*.rs" | while read f; do wc -c "$f"; done | sort -rn
git status --short -- crates/                                                                        # exactly: 1 deletion + new directory
cargo run -p rum-cli --release -- --eval "Plus[1, 2, 3] * Power[2, 3] - Mod[17, 5] + Max[1, 9] + Fibonacci[10]" 2>/dev/null | tail -1   # expect 110
```

Plus the A/B table with judgment (table-iterator-plus gets explicit
commentary — it is the regression sentinel from brief-113).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, the
  microbench script, `grep`, `python3`. Never edit any `mma-hol` file.
  **No new `unsafe`.**
- Probe budget: two A/B runs + one rerun.
- Stop-loss: table-iterator-plus or any judged label consistently
  regressing, a boundary that can't keep callers unchanged, same compile
  error twice → STOP and report.

## Report format

1. SPLIT MAP (module → responsibility → chars; DAG; where the hottest
   bodies live).
2. What changed — inventory; untouched-callers evidence.
3. Verification — verbatim outputs incl. size + A/B tables with the
   sentinel commentary.
4. Verdict — red-line assertion; advice for batch 6 (the final two test
   files: stdlib incidental_runtime 67k, syntax parser/tests 38k).
5. Stop-loss / open questions.
