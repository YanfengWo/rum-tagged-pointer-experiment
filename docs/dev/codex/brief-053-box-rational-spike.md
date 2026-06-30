# brief-053: DESIGN SPIKE — box the `Atom::Rational` variant, measure the memory/time tradeoff (do NOT commit)

## Goal

A measured design spike for Phase 3 (representation constant factor). Box the
`Rational` variant of `Atom` (`Atom::Rational(Box<Rational>)`) so `size_of::<Expr>()`
drops from **64 → 32** (today the enum is sized by `Rational` = two inline 32-byte
`BigInt`s; every `Vec<Expr>` slot pays 64). Then MEASURE the three-axis tradeoff and
REPORT it — **do NOT commit; leave the change in the working tree for Claude's gate.**
The decision (keep / revert) is Claude's, made from your numbers. Deliverable =
evidence, not a merge.

Pre-diagnosed (do not re-derive): measured `size_of::<Expr>()=64`,
`size_of::<Atom>()=64`, `size_of::<Rational>()=64` (two `BigInt`s of 32),
`size_of::<BigInt>()=32`, `size_of::<Symbol>()=32`. `ByteCount[Range[1000]]` ≈ 66
B/elem (rum) vs ~8.3 (WK) — the 64-byte inline `Expr` slot dominates. Boxing Rational
→ Expr 32 → dense-int-list `ByteCount` ~66→~34 B/elem (~1.9x; 8.3x→~4.1x vs WK).
The COST: rum is rationals-only, so boxing adds one heap alloc + one indirection per
`Rational`; the COUNTER-benefit: every `Expr` clone/move/`Vec` halves (32 vs 64). Net
time is workload-dependent — that is exactly what this spike measures.

## What to change

1. `crates/rum-core/src/expr.rs`: `Atom::Rational(Rational)` → `Atom::Rational(Box<Rational>)`.
   - The derives (`Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`) all
     pass through `Box<Rational>` unchanged — no manual impls needed.
   - The constructor `Expr::rational(value: Rational)` (~:617) wraps in `Box::new`.
   - `Display` for `Atom::Rational(value)` auto-forwards (`value: &Box<Rational>`).
   - Accessors `Rational::numerator()`/`denominator()` are UNCHANGED; match sites
     binding `Atom::Rational(r)` then calling `r.numerator()` auto-deref through the
     `Box` — most need NO change. Only the ~33 `Atom::Rational(...)` sites
     (1 rum-core, 1 rum-difftest, 21 rum-runtime files) that CONSTRUCT the variant
     need `Box::new(...)`; matches that move the inner value out need a `*` deref.
2. `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`
   `expr_memory_proxy_bytes` Rational arm (~:261): the boxed layout's bytes are now
   `size_of::<Expr>()` (the smaller node) + `size_of::<Rational>()` (the heap box) +
   the two `BigInt` buffers. Update it to ADD `std::mem::size_of::<Rational>()` so
   `ByteCount` of a Rational stays accurate (a lone Rational gets slightly bigger;
   the WIN is on the common Integer/Symbol Exprs whose slots shrink — the calibration
   workload `Range[1000]` is integers, so its `ByteCount` must drop ~1.9x).

Keep the change minimal and mechanical. Do not refactor `Rational` itself, do not
touch `Atom::Integer`/`Symbol` (those are separate later levers), do not change any
arithmetic semantics — rationals-only exact arithmetic must remain byte-identical.

## Scope

- MAY modify: `crates/rum-core/src/expr.rs`, the `Atom::Rational` construction sites
  across `crates/rum-core`, `crates/rum-runtime`, `crates/rum-difftest`, and the one
  `expr_memory_proxy_bytes` Rational arm in `memory.rs`. (A throwaway `size_of` test
  is OK if you delete it before finishing.)
- MUST NOT touch: any `mma-hol` clone; `Atom::Integer`/`Symbol` representation;
  `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your own report; `.git`,
  `Cargo.lock`.
- This is a SPIKE: **do NOT `git commit`** — leave everything in the working tree.

## Verification (MANDATORY — run all, paste verbatim; this IS the deliverable)

**Axis 0 — correctness.**
```
cargo fmt --check
cargo test --workspace
```
Must be green (rationals-only exact arithmetic unchanged). Paste the result lines.

**Axis 0b — confirm the size win.** Add a throwaway test (then delete it) printing
`std::mem::size_of::<Expr>()` / `::<Atom>()`; paste the lines — expect Expr 64→32.

**Axis 1 — memory (the win).**
```
bash scripts/core-memory-microbench.sh --engine both --output-dir /tmp/cdx_brief_053_mem
```
Paste the `comparison.tsv` `retain-range-1000/10000/100000` rows. Expect
`rum_retained` ≈ 34000 / 340000 / 3.4M (down from 66538 / 660538 / 6.67M) and the
`retained_ratio` ~8.3 → ~4.x. (Read only `retained` — per the script's INTERPRETATION
note, peak/transient/RSS WK columns are confounded.)

**Axis 2 — general per-call time (regression check).** Build a clean BEFORE binary
from the parent state and an AFTER binary, then interleave (per the project A/B
discipline):
```
# AFTER = current working tree (the boxed change)
cargo build --release -p rum-cli && cp target/release/rum-cli /tmp/rum_new
# BEFORE = HEAD without the change
git stash push -- crates/    # stash the boxed change
cargo build --release -p rum-cli && cp target/release/rum-cli /tmp/rum_base
git stash pop                 # restore the change
# interleaved core microbench, base vs new
bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum_base --output-dir /tmp/cdx_053_t_base
bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum_new  --output-dir /tmp/cdx_053_t_new
```
Paste the per-label rum timing for base vs new on the hot labels (`downvalue-exact-hit`,
`pattern-hit`, `if-true`, `block-local` or whatever the script emits). Expect neutral
or a slight IMPROVEMENT (smaller Expr).

**Axis 3 — rational-arithmetic time (the cost).** With the SAME `/tmp/rum_base` and
`/tmp/rum_new`, measure a rational-heavy probe via internal `AbsoluteTiming` (NOT wall
time), 5 runs each, interleaved, report medians:
```
PROBE='AbsoluteTiming[Do[s = 0; Do[s = s + 1/k, {k, 1, 400}], {200}]][[1]]'
/tmp/rum_base --eval "Print[$PROBE]"   # x5
/tmp/rum_new  --eval "Print[$PROBE]"   # x5
```
Paste all 10 timings + the base/new medians + the ratio. This is the boxing cost on
the rationals-only hot path; expect SOME regression — quantify it.

## Hard rules

- **Do NOT `git commit`/branch/push.** This is a spike; leave it uncommitted.
- Do NOT modify files outside Scope. If a site genuinely needs a non-mechanical
  change, note it in the report.
- Full machine access, repo only, no network. Commands: the edits, `cargo`, the two
  microbench scripts, the release `rum-cli`.
- Never edit any `mma-hol` file.
- Stop-loss: if `cargo test --workspace` is red after two focused fix passes for the
  same failure, STOP and report the failing test + output (do NOT loosen a test).
- Measure honestly: internal `AbsoluteTiming` only (never wall), profiling env OFF,
  one variable per run; if a timing axis is too noisy to call, say so rather than
  forcing a verdict.
- Paste REAL verbatim numbers; never fabricate.

## Report format (your final message)

1. What changed — per file + line ranges, and the construction-site count actually
   touched.
2. Verification — each command + verbatim output for all four axes (test result
   lines; the `size_of` lines; the `retain-range` memory rows; the core-microbench
   base/new timings; the 10 rational-probe timings + medians + ratio).
3. The tradeoff verdict — memory win (retained ratio before/after) vs time cost
   (rational-probe base/new ratio) vs general-time (core-microbench neutral?).
4. Stop-loss / open questions (empty if none). Remember: DECISION IS CLAUDE'S — you
   report, you do not keep or revert based on your own judgment.
