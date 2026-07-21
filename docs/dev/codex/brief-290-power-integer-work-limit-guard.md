# Brief 290 — extend the `integer_work_limit` guard from `Factorial` to `Power`

## Goal

`Factorial` already fails closed when its result would be astronomically large: a
pre-check compares the input against `resource_limits.integer_work_limit` and returns
`EvalError::IntegerWorkLimitExceeded { limit }`
(`crates/rum-runtime/src/runtime/arithmetic/factorial.rs:74-87`). `Power` has NO such
guard — `eval_power` computes `base.pow(&exponent)` in one atomic num-bigint call
(`crates/rum-runtime/src/runtime/arithmetic/canonical.rs:32-33`), so `2^(10^9)` builds a
~10^9-bit integer with only the OPT-IN memory/time limits (off by default) as a backstop.
Make `Power` respect the same always-on `integer_work_limit` knob that `Factorial` already
respects, by estimating the result size BEFORE computing it and failing closed with the same
error. This is a consistency fix between two big-integer producers that share one budget knob.

When done: `2^(10^9)` returns `TerminatedEvaluation["IntegerWorkLimit"]` (like an oversized
`Factorial`), normal powers are byte-for-byte unchanged, and the arithmetic hot path shows no
microbench regression.

## Context pointers

- Read `AGENTS.md` (operational notes + Testing Policy hard lines) first.
- `crates/rum-runtime/src/runtime/arithmetic/canonical.rs:18-36` — `eval_power`. The relevant
  branch: `base` is an exact `Rational` (line 23), `exponent` is a signed integer `BigInt`
  (line 24), and the result is `base.pow(&exponent)` (line 32-33). This is the only branch that
  produces a large integer/rational; the other branches restructure symbolic powers and do not
  compute a big value.
- `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:74-87` — the model:
  `enforce_integer_work_limit_for_factorial` returns `Err(EvalError::IntegerWorkLimitExceeded {
  limit })` when `work_units > integer_work_limit`. Mirror its structure.
- Reuse the EXISTING error — `EvalError::IntegerWorkLimitExceeded { limit }`
  (`session_state.rs:328`). Its rendering, termination classification, and request mapping are
  already wired (`model/requests.rs:37,190`, `session/session_helpers.rs:292-293`,
  `session/session_requests.rs:434-435`). Reusing it means ZERO changes at those sites and
  byte-identical output to an oversized `Factorial`:
  `General::reslim: evaluation exceeded integer work limit (100000)` then
  `TerminatedEvaluation["IntegerWorkLimit"]` (verified live).
- `resource_limits.integer_work_limit` default is `100_000`
  (`crates/rum-runtime/src/runtime/metadata.rs:9`); the setter is
  `Session::set_integer_work_limit` (`session/session_api.rs:81-82`).
- Acceptance is mechanical: the named tests + the microbench A/B rows.

## The guard to add (precise design — implement exactly this rule)

Add a method mirroring the Factorial guard, e.g. in `canonical.rs` next to `eval_power`:

```
fn enforce_integer_work_limit_for_power(&self, base: &Rational, exponent: &BigInt)
    -> Result<(), EvalError>
```

Rule:
1. **Trivial-base skip.** If the base magnitude is 0 or 1 — i.e. `denominator == 1` AND
   `|numerator| <= 1` (base is `0`, `1`, or `-1`) — the result is bounded regardless of the
   exponent, so return `Ok(())`. (Do NOT skip on a small denominator: `(1/2)^(10^9)` has an
   astronomical denominator and MUST be guarded.)
2. **Estimate the result bit-size** without computing the power. With
   `num_bits = numerator.bits()` and `den_bits = denominator.bits()` (num-bigint `.bits()` is
   cheap — it reads the limb count, it does not allocate a new big value), the result
   `(numer/denom)^exponent` has numerator and denominator bit-lengths bounded by
   `|exponent| * num_bits` and `|exponent| * den_bits`. Use the total:
   `est_bits = |exponent| * (num_bits + den_bits)`, computed in `BigInt` (the exponent can be
   ~10^9, so do NOT narrow it to `usize` before multiplying — multiply as `BigInt`).
3. **Budget.** `budget_bits = integer_work_limit.saturating_mul(64)` (treat the limit as a
   machine-word/limb budget: 64 bits per unit). At the default this is 6,400,000 bits
   (~1.9M decimal digits / ~800 KB), the same order of magnitude as `Factorial`'s effective
   output cap, and it decisively blocks `2^(10^9)`.
4. If `est_bits > budget_bits` (compare as `BigInt`), return
   `Err(EvalError::IntegerWorkLimitExceeded { limit })` where `limit = integer_work_limit`.
   Otherwise `Ok(())`.

Call it in `eval_power` INSIDE the exact-rational-base + integer-exponent branch, AFTER both
`base` and `exponent` are in hand and after the `0^0` early return, and BEFORE
`base.pow(&exponent)` at line 32-33 — i.e. `self.enforce_integer_work_limit_for_power(&base,
&exponent)?;` guarding the compute. The recursive `eval_power` calls in the `Times` branch pass
back through this same branch, so nested integer powers are covered automatically.

Do NOT touch `Factorial`, `PowerMod` (already modular/bounded), the error type, or any rendering
site. Do NOT change the `integer_work_limit` default. This brief is **Power only** — other
single-op big-integer producers (e.g. bit shifts) are a separate future probe, explicitly out of
scope (avoid over-generalizing the guard).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/arithmetic/canonical.rs` — the guard call in `eval_power`
    plus the new `enforce_integer_work_limit_for_power` method.
  - `crates/rum-runtime/src/runtime/tests/arithmetic.rs` — the regression tests below.
- MUST NOT touch: `factorial.rs`, the `EvalError` definition or any of its rendering/
  classification/request-mapping sites (you REUSE the existing variant), the catalog/attribute
  mirror (this adds no builtin and no attribute), `Cargo.lock`, git, any doc.
- Do NOT add unconditional cost to the default path beyond the guard itself; the guard runs only
  in the integer-power branch and must stay a few cheap ops for small powers (bits + one BigInt
  multiply + one compare).

## Deliverable

- The `enforce_integer_work_limit_for_power` method + its call site in `eval_power`.
- Regression tests in `tests/arithmetic.rs` (name them clearly), covering:
  - `2^(10^9)` → terminates with the integer-work-limit reason (was unbounded). Assert the
    termination/`TerminatedEvaluation["IntegerWorkLimit"]` outcome the harness exposes, matching
    how the existing `Factorial` over-limit test asserts it (find and mirror that test's shape).
  - `2^100000` → still computes to a normal integer (do NOT over-block a fast, ~30k-digit power).
  - `1^(10^9)`, `0^(10^9)`, `(-1)^(10^9)` → compute trivially (`1`, `0`, `1`) — the trivial-base
    skip works and does not falsely trip.
  - `(1/2)^(10^9)` → terminates (astronomical denominator is guarded, not skipped).
  - `2^(-10^9)` → terminates (negative exponent, huge denominator).
  - knob: after `set_integer_work_limit` to a large value, a power that was blocked now computes
    (the guard reads the live limit).
- The microbench A/B rows (below).

## rum / Rust pitfalls (read twice)

1. **Estimate in `BigInt`, never narrow the exponent to `usize` first** — `|exponent|` can be
   ~10^9 and the product can exceed `usize` on 32-bit; keep the multiply and the comparison in
   `BigInt`.
2. **`.bits()` is the cheap size read — do not compute the power to measure it.** The whole point
   is to fail closed BEFORE the expensive `.pow()`.
3. **Reuse the error variant; do not invent a new one.** A new `EvalError` variant would force
   edits at every exhaustive match (rendering, classification, request mapping) and change output.
   The existing `IntegerWorkLimitExceeded` already renders identically to the `Factorial` case.
4. **One variable per measurement (A/B).** The guard touches `Power`, an arithmetic hot path.
   Measure; do not assert "it's cheap."
5. **Stop-loss is real.** If a test or the A/B fails twice for the same reason, STOP and report.

## Verification (MANDATORY — run exactly these, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then the performance gate (Power is a hot path):

```
bash scripts/core-language-microbench.sh --output-dir /tmp/mb_base_290    # on BASE (stash the change)
bash scripts/core-language-microbench.sh --output-dir /tmp/mb_new_290     # on the change
# paste the per-label rum_seconds rows for every arithmetic/power label AND its same-tier
# neighbors; a regression on any hot label is a defect to fix or report.
```

Also paste the live before/after for the headline case, via the release CLI under
`scripts/run-bounded.py --timeout 15`:

```
target/release/rum-cli --eval 'IntegerLength[2^(10^9)]'   # expect: TerminatedEvaluation["IntegerWorkLimit"], fast
target/release/rum-cli --eval '2^100000'                  # expect: a normal integer, unchanged
```

Loop edit → test → A/B until green with no regression; never claim a number you did not paste.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the working tree;
  Claude runs the authoritative gate.
- Do NOT modify files outside Scope. If you believe you must (e.g. a rendering site), STOP and
  write that in the report — reusing the existing error variant is designed to make that
  unnecessary.
- Touch nothing outside this repo; no network. Commands: file edits within Scope, `cargo`, the
  release `rum-cli`, the microbench scripts, `scripts/run-bounded.py`, and `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior only; no
  external-engine comparison, no oracle harness.
- **No new `unsafe`.** `rum-core`/`rum-runtime` are ZERO-`unsafe`. If you think it's unavoidable,
  STOP and report.
- **Hot-path change ⇒ the microbench A/B gate is mandatory**, not just "it looks cheap."
- Minimal diff: no drive-by reformatting or renames outside the task.
- Paste the real verbatim verification output; if a run did not reach green / no-regression, say
  so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — `cargo fmt --check` + `cargo test --workspace` summary lines; the microbench
   per-label rows (target + same-tier neighbors); the two live before/after CLI lines.
3. The result — confirm `2^(10^9)` now fails closed identically to an oversized `Factorial`, and
   that `2^100000` and the trivial bases are unchanged; state the worst microbench delta and the
   neighbor check.
4. Stop-loss / open questions (empty if none).
