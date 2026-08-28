# brief-382 — one canonical identity for the association mutation fallback

Step 2 of plan-378 (repair family B). Landed so far: 1a the primitives
(`5698e8e5`), 1b preparation with `Set`/`SetDelayed` (`aa441ff0`), 1c `Unset`
and value tables (`45c5aa33`). Five of the family's seven defects are closed
and the ordinary assignment path is done. This step closes B1.

## Goal

`AssociateTo` and `KeyDropFrom` read their target correctly, compute the update
correctly, and then install the result against the **unevaluated** target
expression, leaving the real definition untouched. Route the fallback through
`PreparedDefinitionLhs` so the store uses the same identity the read did.

**Director's re-probe on the current tree (`45c5aa33`)** — unchanged by 1b and
1c, each case carrying a liveness assertion:

```
ii = 2; af[2] = <|"a" -> 1|>;
AssociateTo[af[ii], "k" -> 7]
  returns <|"a" -> 1, "k" -> 7|>          (correct)
  af[2] is still <|"a" -> 1|>             (defect)
  DownValues[af] = {HoldPattern[af[2]]  :> <|"a" -> 1|>,
                    HoldPattern[af[ii]] :> <|"a" -> 1, "k" -> 7|>}
```

`KeyDropFrom` behaves the same way, as does the SubValue form
(`sf[pp][qq]`). plan-378 widened the fallback further: it legitimately carries
DownValues, SubValues, evaluated and delayed head aliases, delayed argument
values, and curried operator forms. **The fallback itself is correct and must
survive — only the store identity is wrong.**

The defective store is `mutation.rs:412`
(`store_definition_assignment_target_immediate`), reached from `AssociateTo`
at `mutation.rs:251` and `KeyDropFrom` at `mutation.rs:377`. The other two
call sites construct exact symbol targets and are already correct.

## The hard part: read and store must share one preparation

This consumer differs from every one migrated so far. `Set` prepares an lhs and
stores. `Unset` prepares and removes. **B1 must prepare, then evaluate the
prepared call to read the current association, then store back to the same
prepared identity.**

The naive shape is wrong: preparing the lhs and then calling
`evaluate(canonical_lhs)` prepares active nested arguments a **second** time.
What is needed, per plan-378, is an operation that resumes the existing
dispatch plan *after* `PrepareArguments`, applies the outer definition once,
and retains any `Unevaluated` reconstruction plan for the no-match result.

**Exactly-once is a hard requirement and it currently holds** — verified by the
director on the current tree, so this is a property to preserve, not to
achieve:

```
sideIdx := (sideCount = sideCount + 1; 2);   AssociateTo[afSide[sideIdx], …]
  → sideCount is 1
afRead[2] := (readCount = readCount + 1; <|"a" -> 1|>);
  AssociateTo[afRead[kk], …]  with kk = 2
  → readCount is 1
```

A repair that fixes the identity but reads the target twice is a regression,
not a fix. Prove both counters stay at 1.

## What must not change

- **The symbol and `Part` fast paths.** `ag = <|…|>; AssociateTo[ag, …]` and
  `pk = 1; pa = {<|…|>}; AssociateTo[pa[[pk]], …]` both work today (the latter
  since family A) and must pay no new branch or allocation.
- The head-specific protection check keeps its current relative position,
  now against the canonical owner.
- Return conventions, message tags, and the `invak`/`wrsym` diagnostics.
- Everything 1a-1c fixed. B3, B4, held `Flat`/`Orderless`, `Unset`, and value
  tables must stay byte-identical in the differential gate — a regression
  there is a stop, not a diff line to justify.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/associations/mutation.rs`
- `crates/rum-stdlib/src/associations/mutation/resume_frames.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs`
  and its resume-frame module
- `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
- the runtime and stdlib test modules covering association mutation
- `docs/guide/evaluation.md`

**The MAY list is a floor, not a ceiling.** A purely mechanical ripple of an
in-scope change — an exhaustive `match` arm forced by a new variant, an
exhaustive literal forced by a changed field set, a call site forced by a
changed signature, a `FrameRootTrace` derive forced by a type entering frame
state — you MAY edit unlisted; list each under "mechanical ripple". Anything
changing behavior, contract, or a test expectation in an unlisted file: STOP
and report. When unsure, it is the second kind.

Out of scope: `TagSet`/`UpSet`/`TagUnset` and the final sealing (step 3),
`Expr`/`Symbol` representation, public serde shapes, `Cargo.lock`, `.git`,
`TODO.md`, `PROGRESS.md`, `AGENTS.md`, other `docs/**`.

## Suspension safety

Preparation runs user code, and so does the outer-call evaluation, so this
consumer can suspend **twice** in one operation.

- Reuse 1b's zipper; do not fork a second preparation path.
- A resume must not re-prepare arguments already prepared, and must not re-read
  the target. The exactly-once counters above are the test.
- Every new state/frame derives `FrameRootTrace` and appears in
  `runtime/tests/frame_root_trace/derived_inventory.rs`; frames are inserted
  only through `resume_or_suspend!`; the queue stays private.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The bounded differential gate

Same shape as 1b and 1c: **non-empty in exactly the expected places, empty
everywhere else, every differing line justified.**

- Reuse and extend the probe from `/tmp/rum-brief-381-*/behavior-preservation.rum`.
- **Calibrate first**: run it against a pre-family-A binary and show it reports
  a diff. A scanner that reports clean must be shown capable of finding
  something.
- Then diff `HEAD` against your candidate.

### Test matrix

- Both heads across every fallback shape plan-378 found: DownValue, SubValue,
  evaluated head alias, delayed head, delayed index, curried operator forms.
- Held targets, which already behave correctly and must stay raw.
- Symbol and `Part` fast paths, exercised explicitly so they are not silently
  bypassed.
- Exactly-once counters for the index argument **and** the outer definition
  read, in the plain case and across a suspension.
- Protection: a canonical alias to a protected owner reports `wrsym` and stores
  nothing.
- Suspension during target preparation and during the outer read, with no
  replay after resume.

**Assert after the invalidating action**, not only before:

- after `SetAttributes` changes the target head's attributes;
- after the index own-value is rebound and then cleared — the mutation
  addresses the newly prepared identity while previously stored ones stay put;
- after `Clear[owner]` and reinstall.

Say per test which invalidation it acts after, and name the entry point per
form: "I added the logic" is not "the logic runs" (brief-348), and this file
has symbol and `Part` fast paths that return before the fallback.

### Performance gate

- rum-vs-rum A/B, one variable, profiling OFF, same script both binaries,
  **best-of-3 per binary** — 1c's worker left two labels open as possible
  regressions on best-of-2 spreads of 2.01% and 2.96%; at best-of-3 those same
  labels' spreads are 10.6% and 11.3% and both movements fell inside. Two runs
  are a single random sample, not a noise floor. Report each label's three-run
  range.
- **Controls apply for this step**: the change sits on the association
  mutation path, not the common call path, so the pure-computation labels are
  a genuine control group. Report the control median beside the affected
  median. (This distinction decided 1b's gate; do not inherit 1a's no-controls
  rule, which applied only because a common-path change leaves nothing
  untouched.)
- Targets: `association-set-key-incremental-small/large`,
  `appendto-list-incremental-small/large`, `appendto-part-outer1000-*`, and
  the `downvalue-*` rows.
- Only fallback targets may pay preparation; symbol and `Part` paths pay
  nothing new.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
- No unconditional cost on the ordinary evaluation path.
- No owned-tree deep clones; `Expr`/`SharedExpr`/`SharedArgs` are `Arc`-shared.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only; no other
  vendor's engine, no oracle harness, no third-party output committed.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized from a
  measured run of the same shape.
- **Stop-loss** — report rather than improvise if: the prepared outer call
  cannot be evaluated without re-preparing arguments; an exactly-once counter
  moves to 2; a fast path cannot be preserved without duplicating the
  preparation; a 1a-1c differential line regresses; or a differing line appears
  you cannot justify.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Mechanical ripple — every file edited under the criterion above.
3. Verification — `fmt`, workspace counts, the calibration, the bounded diff
   with every differing line justified, and the exactly-once counters.
4. The A/B — control median, affected median, per-label three-run ranges, and
   what the symbol and `Part` fast paths pay.
5. Stop-loss / open questions.
