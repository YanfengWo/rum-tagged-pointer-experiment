# Brief 015 — Thread the dispatch AttributeSet into the value stages (cut redundant lookups)

## Goal

A per-call WORK reduction on the down/sub-value dispatch path (the Pair
conversion-combinator path). `build_call_dispatch_plan` already computes the call
head's `AttributeSet` (`cached_attribute_set_for_head(head)`) and stores it on the
plan, and `run_call_dispatch_stage` already has it in scope as `attributes`. But
the value stages re-look-it-up:

- `try_apply_down_values` (definition_application.rs ~line 37) computes
  `self.attribute_set_for_head(head).hold_attribute()` again, AND then calls
  `normalize_definition_candidate_for_matching` (~line 75) which AGAIN does
  `self.cached_attribute_set_for_head(head).hold_attribute()` (~line 820).
- `try_apply_sub_values` (~line 612) and `try_apply_up_values` (~line 414) each
  call `normalize_definition_candidate_for_matching` → another lookup.

So a single down-value dispatch does the same head-attribute lookup ~3 times
(plan + line 37 + normalize). Thread the already-computed `AttributeSet`
(`Copy`) into the three value stages and into
`normalize_definition_candidate_for_matching`, eliminating the re-lookups.

## Expected, measurable result (the gate)

This is a lookup elimination — there is no `RUM_PROFILE_REPR` counter for it. Gate
on behavior + timing:
- `cargo test -p rum-runtime` green (no behavior change — the threaded
  `AttributeSet` is the same value the stages re-looked-up).
- A clean same-session back-to-back A/B (the change is uncommitted, so `git stash`
  it for the baseline half) shows the down/sub-value labels improve or hold:
  `downvalue-exact-hit`, `downvalue-pattern-hit`, `downvalue-pattern-miss`; the
  no-value controls (`if-true`, `sameq-integer`) unchanged.
- A bounded Pair re-measure (`basic-chain-orelsec-split`, rum) improves or holds
  vs the post-A4 baseline (manual-loop ~`1.25s`, full ~`2.2s` on an idle
  machine).

Expected magnitude is modest (a few hashmap lookups per dispatch). If the
down/sub-value labels do not move at all in the back-to-back A/B, report that
plainly — a null result is a valid outcome that tells us the lookup was already
cheap.

## Context pointers

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`: `run_call_dispatch_stage`
  has `attributes: AttributeSet` (= `plan.attributes`) in scope; the
  UpValues/DownValues/SubValues arms (~lines 99-108) call the three stages
  WITHOUT passing it.
- `crates/rum-runtime/src/runtime/definition_application.rs`:
  - `try_apply_down_values(head, args)` line ~4; the redundant lookup is line
    ~37 (`self.attribute_set_for_head(head).hold_attribute()` for
    `head_allows_raw_miss_cache`).
  - `try_apply_up_values(head, args)` line ~383; `try_apply_sub_values(head,
    args)` line ~577.
  - `normalize_definition_candidate_for_matching(head, candidate)` line ~814 —
    its body (line ~819-820) does `self.cached_attribute_set_for_head(head)
    .hold_attribute()`. Called only from the three value stages (lines ~75, ~414,
    ~612).
- `cached_attribute_set_for_head` / `attribute_set_for_head` return the same
  `AttributeSet` (cached is memoized); `plan.attributes` is
  `cached_attribute_set_for_head(head)` from `build_call_dispatch_plan`, so it is
  the identical value the stages re-compute. `AttributeSet` is `Copy`.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` (pass `attributes` to
    the three value stages),
  - `crates/rum-runtime/src/runtime/definition_application.rs` (the three stage
    signatures + the `normalize_definition_candidate_for_matching` signature and
    its 3 call sites + the line-37 lookup).
- MUST NOT change: matcher/normalization behavior, dispatch ordering, the
  candidate build (A3), the RHS substitution, the arg-head attribute lookups
  inside `fast_match_compiled`/`fast_match_definition_pattern` (lines ~960/~1046
  — those are the VALUE arg head, a different head, NOT redundant; leave them),
  `Expr`/`AttributeSet` public types, serde, any `mma-hol` clone, `.git`.

## Design (implement exactly; idiomatic Rust is yours)

1. `try_apply_down_values(&mut self, head: &Expr, args: &SharedArgs, attributes:
   AttributeSet)`: replace the line-37 `self.attribute_set_for_head(head)` with
   the passed `attributes`. Pass `attributes` to
   `normalize_definition_candidate_for_matching`.
2. `try_apply_up_values` / `try_apply_sub_values`: add the `attributes:
   AttributeSet` param; pass it to `normalize_definition_candidate_for_matching`.
   (They have no other head-attribute lookup to replace.)
3. `normalize_definition_candidate_for_matching(&mut self, head: &Expr, candidate:
   &Expr, attributes: AttributeSet)`: use `attributes.hold_attribute()` instead
   of `self.cached_attribute_set_for_head(head)`. (Keep `head` if still needed by
   `normalize_match_expr`; if `head` becomes unused, drop it.)
4. `run_call_dispatch_stage` (eval_dispatch/mod.rs): pass `attributes` to the
   UpValues/DownValues/SubValues arms.

**Why this is correct (state it in your report):** `plan.attributes` is exactly
`cached_attribute_set_for_head(head)` for the same call head; the stages were
re-deriving that identical value. Substituting the threaded copy changes no
behavior — `attribute_set_for_head` and `cached_attribute_set_for_head` are the
same function memoized.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
```

All green. Then the clean back-to-back A/B (stash for the baseline half, since the
change is uncommitted):

```
# A4 baseline: git stash push crates/...; cargo build --release -p rum-cli; microbench -> /tmp/cdx_b015_base
# this change:  git stash pop;          cargo build --release -p rum-cli; microbench -> /tmp/cdx_b015_new
bash scripts/core-language-microbench.sh --engine both --output-dir <dir>
```

Paste the `downvalue-exact-hit`, `downvalue-pattern-hit`, `downvalue-pattern-miss`,
`if-true`, `sameq-integer` rows for BOTH halves (back-to-back cancels machine
drift). Then the bounded Pair (rum, idle machine):

```
RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance bash \
  scripts/pair-simp-hotspot-probes.sh --mma-hol /Users/fft/Developments/mma-hol-acceptance \
  --only basic-chain-orelsec-split --engine rum --timeout 180 --output-dir /tmp/cdx_b015_pair
```

Report `basic-chain-manual-rule-loop-tryConv` and
`basic-chain-full-TRYCONV-REPEATC-depth19` rum seconds; state whether they moved.

## rum / Rust pitfalls (read twice)

1. `AttributeSet` is `Copy` — pass by value, no clone.
2. Do NOT touch the arg-head attribute lookups in the fast-match functions
   (lines ~960/~1046); they are a different head.
3. Measure the RELEASE binary; for the A/B, build each half and run back-to-back
   with nothing else competing for CPU.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree (and make sure you
  `git stash pop` so the change is restored after the A/B baseline half).
- Only the listed files; nothing outside repo + `/tmp`; no network. Commands:
  `cargo`, release `rum-cli`, the microbench + pair-simp scripts, `python3`.
  Remove any `.rum_history` you create. mma-hol clone is read-only.
- Stop-loss: same `cargo test` failure twice → STOP and report.
- Minimal diff. Paste real test summary, the back-to-back A/B rows, and the Pair
  seconds; never fake a result. A null timing result is fine — report it
  honestly.

## Report format

1. Per-file changes, line ranges + one-line why.
2. The correctness argument (threaded AttributeSet == the re-looked-up value).
3. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-runtime`
   summary, the back-to-back A/B rows (both halves), and the Pair seconds.
4. Stop-loss / open questions (empty if none).
