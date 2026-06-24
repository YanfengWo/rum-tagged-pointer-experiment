# brief-044: remove the symbolic Plus/Times regression brief-043 introduced (perf-only, behavior frozen)

## Goal

brief-043 added like-term/like-base combination to `combine_additive_args`
(Plus) and `combine_multiplicative_args` (Times), but it regressed SYMBOLIC
`Plus`/`Times` evaluation by ~2-3x on larger term counts (measured, see below).
This brief makes the combination fast WITHOUT changing any result: every
`cargo test`, every brief-043 `===` probe, every out-of-scope guard, and the
real-tower 1122 behavior must stay byte-identical. Pure performance work on the
symbolic path; the numeric fast paths are already fine and must stay fine.

**This brief edits the brief-043 changes that are CURRENTLY IN THE WORKING TREE
(uncommitted).** Read the current `arithmetic.rs` (it already has 043's (A)/(B)/(C)
+ the `split_*`/`build_power_from_base_exp` helpers); optimize them in place.

## The measurement (verified this session, machine idle, `Do[expr,{20000}]`, min of 3)

| probe | 043-before (target) | 043-after (now, regressed) |
| --- | --- | --- |
| symbolic Plus `a + b + c + a + b + c` | 0.233s | 0.660s (**2.86x**) |
| symbolic Times `x y z x y z` | 0.233s | 0.496s (**2.13x**) |
| numeric Plus `2 + 3 + 4 + 5` | 0.048s | 0.049s (ok) |
| numeric Times `2*3*4*5` | 0.077s | 0.079s (ok) |

Localization (043-after): `Plus` no-merge 3-term `0.077s`, no-merge 6-term
`0.308s` (4x for 2x the terms → the O(n^2) grouping dominates), merge 6→3
`0.660s` (the extra `0.35s` over no-merge 6-term is the 3 rebuild calls). So the
two costs are (1) the grouping/split/rebuild runs on EVERY symbolic call even
when nothing merges, and (2) the rebuild of a merged term calls back into the
full `combine_multiplicative_args`.

## Optimizations (apply in order; each is behavior-neutral)

### Opt 1 — stop rebuilding merged Plus terms via `combine_multiplicative_args`
In `combine_additive_args` (~:419), the merged-term rebuild currently does:
```rust
combined.push(self.combine_multiplicative_args(
    &system_expr("Times"),
    &[Expr::rational(coefficient), kernel],
)?);
```
That re-runs the entire Times canonicalizer (numeric fold + (B) grouping +
normalize) for every merged term. Replace with a DIRECT construction of
`Times[coeff, <kernel factors>]`:
- if `kernel` is a `Times[args..]` call → `Expr::call(Times, [coeff, args...])`
  (splice; coeff first);
- else → `Expr::call(Times, [coeff, kernel])`.
This matches the canonical shape (numeric coefficient first, kernel factors
already Orderless-sorted from the input term) — the outer `Plus`
`normalize_call_args_by_attributes` does not reorder inner Times. Verify the
`===` probes still pass (they assert structural/`SameQ` equality, so any shape
drift would fail them).

### Opt 2 — no-merge fast path in BOTH `combine_additive_args` and `combine_multiplicative_args`
While grouping, track whether any term/factor actually merged (a `find` hit).
If nothing merged, LEAVE `combined` exactly as it was — skip the rebuild loop and
the `grouped → combined` reconstruction entirely. Sketch:
```rust
let mut merged_any = false;
for term in &combined {
    let (coefficient, kernel) = split_coeff_kernel(term);
    if let Some((_, existing)) = grouped.iter_mut().find(|(k, _)| k == &kernel) {
        *existing = existing.add(&coefficient);
        merged_any = true;
    } else {
        grouped.push((kernel, coefficient));
    }
}
if merged_any {
    // existing rebuild (with Opt 1 for the additive case)
}
// else: combined is untouched — the inputs were already distinct & canonical
```
This makes the common no-merge case (`a + b + c`, `x y z`) pay only split + the
O(n^2) compare, not the Vec reconstruction. (Same shape for the Times/(B) side,
using `split_base_exp` and the exponent accumulation.)

### Opt 3 — only if Opt 1+2 leave symbolic still >~1.4x of 043-before
Reduce cloning in `split_coeff_kernel`/`split_base_exp`: do the grouping
comparison against a BORROWED kernel/base and clone only when actually inserting
a new group or merging. `split_coeff_kernel` currently clones every non-numeric
factor and rebuilds a `Times` even when there is a single factor / no merge —
avoid that allocation on the compare path. Keep it simple; do not over-engineer.

## Scope
- File you MAY modify: `crates/rum-runtime/src/runtime/arithmetic.rs` ONLY
  (the two `combine_*` fns + the `split_*` helpers). The brief-043 test files
  (`tests/arithmetic.rs`, `tests/structure_ops/surface.rs`) stay as they are —
  do NOT change a single expected value; they are the behavior-freeze guard.
- MUST NOT touch: anything else; the result of ANY evaluation; the numeric fast
  paths (`fast_integer_plus`, the numeric fold) beyond leaving them intact; any
  `mma-hol`/snapshot; `crates/rum-core`; docs except this report; git.
- Behavior is FROZEN. If any `cargo test` case or any `===` probe changes value,
  you broke correctness — STOP and report.

## rum / Rust pitfalls
1. **Behavior-neutral means byte-identical.** The `===` probes use `SameQ`, so a
   reordered `Times[x, 2]` vs `Times[2, x]` WILL flip a probe to `False`. Match
   the canonical shape (numeric first, sorted factors) exactly.
2. **Don't regress the numeric path.** `fast_integer_plus` and the empty-`combined`
   case must keep their current near-zero cost; the new fast path only adds a
   boolean + the existing loop.
3. **No new behavior.** This is not the place to also fix the out-of-scope items
   (Power-over-Times, symbolic exponents). Frozen.
4. **Minimal diff**, only inside the named fns/helpers.
5. **Stop-loss:** if Opt 1+2+3 cannot get symbolic Plus/Times under ~1.5x of the
   043-before numbers without changing a result, STOP and report what you tried.

## Verification (MANDATORY — run, paste verbatim)
```
cargo fmt --check
cargo test -p rum-runtime           # 690/0, unchanged
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# behavior freeze — every brief-043 === probe must still print True:
$BIN --eval 'x^2 x^3 === x^5'
$BIN --eval 'x - x === 0'
$BIN --eval '2 a 3 a === 6 a^2'
$BIN --eval 'a + b + a === 2 a + b'
$BIN --eval 'x^3/x^2 === x'
# guards unchanged:
$BIN --eval '(2 x)^2'                 # Power[Times[2, x], 2]
$BIN --eval '1 (x^2)^(1/2)'           # Power[Power[x, 2], 1/2]

# perf A/B — paste the float for each (rum prints rationals; convert with python3):
for p in 'a + b + c + a + b + c' 'x y z x y z' 'a + b + c' 'x y z' '2 + 3 + 4 + 5' '2*3*4*5'; do
  r=$($BIN --eval "AbsoluteTiming[Do[$p, {20000}]][[1]]")
  python3 -c "print('$p', float($r))"
done
```
Target after optimization (min of 3 runs, idle machine): symbolic
`a + b + c + a + b + c` and `x y z x y z` back near the **043-before** values
(0.233s / 0.233s); at worst ≤ ~1.4x of them. Numeric probes unchanged. Report
the before(=043-after)/after(=optimized) floats and the ratio vs 043-before.

## Hard rules
- Do NOT git commit/branch/push. Leave changes in the working tree.
- NEVER edit any `mma-hol` clone or snapshot.
- Do NOT change any test expectation or any evaluation result — perf only.
- Full machine access, nothing outside this repo, no network. Commands: edits in
  scope, `cargo`, release `rum-cli`, `python3` for the float conversion.
- Minimal diff; paste real verbatim output; if a result changed, say so — do not
  hide it.

## Report format
1. What changed — Opt 1/2/(3) per fn, line ranges, one-line why each.
2. Verification — `fmt`, test count, every `===`/guard verbatim, the perf floats
   (043-after vs optimized) and the ratio vs 043-before.
3. Verdict — symbolic Plus/Times regression removed (give the ratios), behavior
   byte-identical, numeric path unchanged.
4. Stop-loss / open questions.
