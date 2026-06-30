# Brief 058 — Matcher backtracking depth guard (Orderless + sequence: crash → graceful TerminatedEvaluation)

## Goal

A pathological `Orderless` + `BlankNullSequence`/`BlankSequence` pattern currently
**crashes the host process with a native stack overflow (SIGABRT)** from a tiny
input — e.g. `SetAttributes[f, Orderless]; MatchQ[f @@ Range[12], f[a___, b___, c___] /; False]`
crashes; `Range[14]` with 2 sequence vars crashes too. When you are done, that
same input must **degrade gracefully**: return a `TerminatedEvaluation[...]` value
(a controlled resource-limit trip, like rum already does for the recursion /
iteration-work limits) instead of overflowing the stack — AND every currently
passing matcher test plus the real-workload corpus must still pass unchanged, with
**no hot-path regression** on `core-language-microbench.sh`.

This is the host-resilience fix, NOT an attempt to make the matcher faster or
smarter. The problem (`Orderless` + variable-length matching = AC-matching) is
NP-hard; WK itself does naive `k^n` enumeration with no pruning (measured) — it
just degrades gracefully because its matcher is iterative / bounded-stack. rum's
is deep native recursion, so it crashes. We make rum bound that recursion.

## Context pointers

- Read `AGENTS.md` (operational notes, the unmodified-`mma-hol` hard line) and the
  `## Performance And Representation` TODO entry "Pattern-matching as TWO
  under-evaluated axes" — this brief implements the resilience half of axis (b).
- **Diagnosis (already done — this is WHY, do not re-derive):** the `Orderless`
  backtracking in `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs`
  tries the next candidate via a **recursive call with `index + 1`** (lines ~30,
  ~53 and the flat-head twin ~95, ~120), driven by `drive_internal_value`. In the
  normal (non-suspended) path `drive_internal_value` calls its continuation
  **synchronously on the same native stack** (`suspension/core.rs:91-92`,
  `Ok(value) => cont(self, value)`). Rust does NOT guarantee TCO, so each
  `index + 1` re-entry and each `rest` recursion adds a stack frame that is not
  reclaimed until the whole chain bottoms out. The candidate count for one
  sequence-variable position is combinatorial (~`2^n`), and each frame is large
  (the functions `.clone()` whole `Vec<MatcherArg>` / `SymbolExprMap` per frame,
  lines 17-24). So the native stack depth grows ~ with the WORK, and ~`2^n` large
  frames overflow the 8 MB main-thread stack at n≈12 (3-seq) / n≈14 (2-seq). The
  binding resource is **native stack depth**, not a work count — `iteration_work_limit`
  (100_000) is far above the few-thousand frames that fit in 8 MB, so it never
  fires before the crash. **Therefore a DEPTH guard is the correct fix, and it
  subsumes a work counter here: because depth≈work, the first backtracking path
  that exceeds the depth limit returns an `Err` that propagates (via the existing
  `?` / `drive_internal_value` Err propagation) and aborts the whole match fast —
  it does NOT grind through `k^n`.** Do not add a separate work counter.
- Mechanism to MIRROR — the existing evaluation-depth guard:
  `crates/rum-runtime/src/runtime/eval_core.rs:20-34` (`self.evaluation_depth`
  field, checked against `self.resource_limits.recursion_limit`, raising
  `EvalError::RecursionLimitExceeded`; `+= 1` before the recursive descent, `-= 1`
  after). The parser used the lower-blast-radius variant — a private const
  `MAX_PARSE_DEPTH = 128` in `crates/rum-syntax/src/parser/front.rs:9,36` (brief-055).
- Key files / call sites:
  - `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs`
    — the `index + 1` backtracking chain (both `match_compiled_orderless_choices_owned`
    and `match_compiled_orderless_flat_choices_owned`). The PRIMARY guard site.
  - `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/*.rs`,
    `pattern_matching/sequence/entry.rs`, `pattern_matching/core.rs` — the
    surrounding recursive matcher methods (all `&mut self` on `Session`); find the
    minimal chokepoint(s) the backtracking actually passes through.
  - `crates/rum-runtime/src/runtime/model.rs` — `ResourceLimits` (line 8), the
    `Session` struct (line 316; `evaluation_depth` is line 353), the `EvalError`
    enum + its `Display` (around line 255-296 and the variant defs ~630-650).
  - `crates/rum-runtime/src/runtime/session_state.rs` — the `EvalError` variant
    list (`RecursionLimitExceeded` is line 298).
  - `crates/rum-runtime/src/runtime/session_helpers.rs:168-205` — the TWO functions
    a new resource-limit variant must be wired into: `runtime_message_for_non_loader_eval_error`
    (the `General::<tag>` message) and `terminated_evaluation_reason_for_error`
    (the `TerminatedEvaluation[<reason>]` tag). Per the 63efadd philosophy, give the
    new limit its OWN specific tag so a user knows which budget tripped.
  - `crates/rum-runtime/src/runtime/session_requests.rs:384` and
    `crates/rum-host/src/config.rs` / `crates/rum-jupyter/src/adapter/render.rs:161`
    — the `EvalError` → `SessionRequestError` → `HostFailureCode` mapping chain a
    new variant ripples through (this is WHY the gate is `--workspace`, not
    `-p rum-runtime`).
- `metadata.rs:4` `DEFAULT_RECURSION_LIMIT = 1024`, `metadata.rs:8`
  `DEFAULT_ITERATION_WORK_LIMIT = 100_000` — sibling defaults for reference.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs`
    and, if the minimal chokepoint requires, the sibling recursive matcher methods
    under `crates/rum-runtime/src/runtime/pattern_matching/**` (keep the diff
    minimal — guard at the fewest chokepoints that bound the backtracking).
  - `crates/rum-runtime/src/runtime/model.rs` (add the `EvalError` variant + its
    `Display`; add the `match_depth` field to `Session` and initialize it).
  - `crates/rum-runtime/src/runtime/session_state.rs` (the `EvalError` variant
    list, if the enum is declared there).
  - `crates/rum-runtime/src/runtime/session_helpers.rs` (wire the new variant into
    the two tag functions).
  - `crates/rum-runtime/src/runtime/session_requests.rs` and the minimal
    host/jupyter mapping (`crates/rum-host/src/config.rs`,
    `crates/rum-jupyter/src/adapter/render.rs`) ONLY as needed to make the new
    `EvalError` variant compile through the existing mapping chain — mirror how
    `RecursionLimitExceeded` is handled, nothing more.
  - wherever `Session` is constructed/reset, initialize `match_depth: 0` (mirror
    `evaluation_depth`).
  - a new or existing rum-runtime test module for the regression tests below.
- Files/areas you MUST NOT touch:
  - anything under any `mma-hol` clone or the `mma-hol` standard (NEVER edit the
    acceptance standard; diagnostic edits only in throwaway scratch clones, never
    left behind);
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol` representation and any public
    type/serde shape;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your own
    report; `.git`, git config, `Cargo.lock`.
- **No new `unsafe`** (rum-core/rum-runtime are zero-`unsafe`; keep it that way).
- Do NOT add a configurable `ResourceLimits` field unless you find the const
  approach genuinely insufficient — prefer a private const `MAX_MATCH_DEPTH`
  (mirroring `MAX_PARSE_DEPTH`) over expanding the public `ResourceLimits` /
  host-config surface. If you believe it must be configurable, STOP and report the
  reason rather than expanding the API.

## Design (do this; deviate only with a reported reason)

1. Add a `match_depth: usize` counter on `Session` (mirror `evaluation_depth`),
   initialized to 0 everywhere `Session` is built/reset.
2. Add a private `const MAX_MATCH_DEPTH: usize` in the matcher module. Pick the
   value EMPIRICALLY (step 4): comfortably **above** the depth real workloads and
   tractable matches reach, and **below** the depth that overflows the 8 MB stack.
3. At the entry of the recursive backtracking chokepoint(s) (start with the two
   functions in `match_choices.rs`), do the `evaluation_depth` dance: if
   `self.match_depth >= MAX_MATCH_DEPTH` return `Err(EvalError::MatchRecursionLimitExceeded { limit })`;
   else `self.match_depth += 1`, run the body, `self.match_depth -= 1` on the way
   out (ensure the decrement happens on BOTH the Ok and Err return paths — a small
   scope guard struct with a `Drop` that decrements is the clean safe way, OR
   careful manual dec mirroring `eval_core.rs`; no `unsafe`). The returned `Err`
   will propagate through `drive_internal_value` and abort the whole match.
4. Wire `MatchRecursionLimitExceeded` through: the enum + `Display`
   ("evaluation exceeded pattern-match recursion limit (N)" or similar), the two
   `session_helpers.rs` tag functions (a NEW specific tag, e.g.
   `terminated_evaluation_reason_for_error` → `Some("PatternMatchRecursionLimit")`,
   `runtime_message_for_non_loader_eval_error` → a `"reclim"`-style General tag),
   and the `SessionRequestError`/host/jupyter mapping chain so the workspace
   compiles. Mirror `RecursionLimitExceeded` at each site.
5. EMPIRICAL limit-setting: temporarily instrument max `match_depth` reached (a
   local `eprintln!` or a scratch counter you remove before finishing) and run the
   adversarial probe at the n values that currently SUCCEED (n=8, 10, 11 for 3-seq)
   vs the crash boundary, to confirm `MAX_MATCH_DEPTH` sits above the
   currently-working depths and below the overflow. Report the measured depths.
   Remove the temporary instrumentation. (If even n=10/11 — which take 0.7s+ and
   are themselves semi-pathological — must be refused to stay safely below the
   stack limit, that is ACCEPTABLE as long as the real corpus in step Verification
   passes; report it.)

## Deliverable

- The `match_depth` counter + `MAX_MATCH_DEPTH` guard at the matcher backtracking
  chokepoint(s); the `MatchRecursionLimitExceeded` `EvalError` variant fully wired
  with its own specific `TerminatedEvaluation` tag.
- A rum-runtime regression test that feeds the adversarial probe
  (`SetAttributes[f, Orderless]; MatchQ[f @@ Range[14], f[a___, b___, c___] /; False]`)
  and asserts it returns a graceful `TerminatedEvaluation[...]` value (NOT a panic /
  not a crash) — this is safe to run in-process precisely BECAUSE the guard returns
  `Err` before the stack overflows.
- A positive regression test that a NORMAL `Orderless` + sequence match still
  works, e.g. `SetAttributes[f, Orderless]; MatchQ[f[3, 1, 2], f[a_, b__]]` → `True`
  and a couple of realistic small Orderless matches → unchanged results.
- The measured max-depth numbers from step 5 and the chosen `MAX_MATCH_DEPTH`.
- The microbench A/B ratios (below) showing no hot-path regression.

## rum / Rust pitfalls (read twice)

1. Measure internal `AbsoluteTiming`, never wall time; profiling env OFF for any
   timing number.
2. **This IS a matcher hot-path change** → the standing rule applies: run
   `scripts/core-language-microbench.sh` base-vs-new and paste per-label ratios.
   The guard adds a `usize` compare + inc/dec per matcher recursion; it must not
   measurably regress any label. (It mirrors the cost `evaluation_depth` already
   pays per eval step, so expect ~flat — but PROVE it.)
3. Default-path cost: the guard runs unconditionally, so it must be exactly an
   integer compare + inc/dec — no allocation, no `Mutex`, no heap touch. A scope
   guard struct is fine (zero-cost). Do not log/trace on the hot path.
4. Evaluator/matcher flows are suspension-safe: do NOT collapse any
   `drive_internal_value` / `suspend_with_frame*` into an eager call. The
   inc/dec must be balanced even when a continuation suspends — the scope-guard
   `Drop` approach handles this; if you do manual inc/dec, account for the
   `Err(InputRequired)` early-return path too.
5. Hot paths avoid owned-tree clones — do not ADD clones; (a noted opportunity:
   the existing per-frame `.clone()`s are what make frames large, but reducing
   them is OUT of scope for this brief — depth guard only).
6. One variable per measurement.
7. Stop-loss: if the guard can't be placed without regressing the microbench, or
   the adversarial probe still crashes after guarding, STOP and report.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test --workspace        # NOT -p rum-runtime: a new EvalError variant ripples
                              # to rum-host / rum-jupyter mapping; a narrow gate
                              # would miss those (the brief-046 lesson).
```

Then the crash-is-gone check via the release CLI (build first):

```
cargo build --release -p rum-cli
# MUST print a TerminatedEvaluation value (or the General::<tag> message), NOT crash:
./target/release/rum-cli --eval 'SetAttributes[f, Orderless]; MatchQ[f @@ Range[12], f[a___, b___, c___] /; False]'
./target/release/rum-cli --eval 'SetAttributes[f, Orderless]; MatchQ[f @@ Range[14], f[a___, b___, c___] /; False]'
./target/release/rum-cli --eval 'SetAttributes[h, Orderless]; MatchQ[h @@ Range[16], h[a___, b___] /; False]'
# MUST still work normally (no false resource trip on real matches):
./target/release/rum-cli --eval 'SetAttributes[f, Orderless]; MatchQ[f[3, 1, 2], f[a_, b__]]'   (* True *)
./target/release/rum-cli --eval 'MatchQ[g[1, 2, 3], g[a_, b__]]'                                  (* True *)
```

Then the hot-path gate:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_058_micro
# paste the ratios.tsv rows for the matcher-bearing label(s) AND their same-tier
# neighbors; confirm no regression vs a base (pre-change) run you also capture.
```

Paste ALL of: the `cargo test --workspace` pass/fail summary line, the five CLI
outputs verbatim, and the microbench ratio rows (base vs new).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config — leave changes in the
  working tree for Claude's gate.
- Do NOT modify files outside Scope; if you believe you must, STOP and report.
- No new `unsafe` anywhere in rum-core/rum-runtime. Use a safe scope guard
  (`struct Guard<'a>(&'a mut usize); impl Drop`) or balanced manual inc/dec.
- Never edit any `mma-hol` file as the fix or the acceptance path.
- Hot-path change → microbench A/B is mandatory (see Verification); a regression
  on any hot label is a defect to fix or report, not to ship.
- Stop-loss: same failure twice, or the probe still crashes, or the microbench
  regresses and you can't recover it → STOP and report with the verbatim output.
- Minimal diff: no drive-by reformatting/renames/comment churn outside the task.
- Paste REAL verbatim verification output; if a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — each command with verbatim output (the `cargo test --workspace`
   summary, the 5 CLI lines, the microbench base-vs-new ratio rows).
3. The finding — the measured max match-depth at the succeeding n values, the
   chosen `MAX_MATCH_DEPTH`, and confirmation the adversarial probe now returns
   `TerminatedEvaluation` (not a crash) while normal matches are unchanged.
4. Stop-loss / open questions — anything you were unsure of (e.g. whether any real
   corpus match came close to the limit), empty if none.
