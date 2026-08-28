# brief-380 — definition-lhs preparation, and `Set`/`SetDelayed` as its first consumer

Step 1b of plan-378 (repair family B). 1a landed in `5698e8e5`: the
call-preparation primitives now live in `runtime/call_preparation.rs`, the
ordinary path calls them, and lookup consumes a borrowed
`PreparedDefinitionLookup<'_>`. This step adds the definition-lhs preparation
that reuses those primitives, and migrates its first consumer.

**Note on the split**: plan-378's step 7 makes `Flat`/`Orderless` identity
normalization part of the preparation pipeline rather than a separate
transform, so the held-ground-`Flat` defect is fixed here, with `Set`, not in a
later step.

## Goal

Introduce `PreparedDefinitionLhs` and route `Set` and `SetDelayed` through it,
closing three of family B's seven defects:

- **B3** — `iD := 1 + 1; fD[iD] = 8` stores `HoldPattern[fD[1 + 1]]`, so
  `fD[2]` cannot reach it. Active positions must complete evaluation.
- **B4** — with `SetAttributes[fH, HoldFirst]` and `xH = 5`, `fH[xH] = 9`
  stores `HoldPattern[fH[5]]`, so the natural `fH[xH]` lookup misses. Held
  positions must keep their raw symbols. **This is the opposite direction from
  B3**, and a change that fixes one while leaving the other is not this brief.
- **Held ground nested `Flat`** — `SetAttributes[hFlat, Flat]`,
  `SetAttributes[fHF, HoldAll]`, then `fHF[hFlat[1, hFlat[2, 3]]] = 802`
  stores an identity that **neither** `fHF[hFlat[1, 2, 3]]` nor the source form
  reaches, because candidate matching normalizes `Flat`/`Orderless`
  structurally while the ground index does not.

All three are the director's re-probes at `dfedb317`, each with a liveness
assertion and a working control.

Out of scope for this step and each with its own brief: `Unset` and
value-table assignment (1c), the `AssociateTo`/`KeyDropFrom` fallback (2),
`TagSet`/`UpSet`/`TagUnset` and the final sealing (3). Do not migrate them
here even though the preparation will obviously serve them.

## The preparation contract

From plan-378's specification, which the director has reviewed and adopted.
Follow it; critique it in your report if the code contradicts it.

1. Recognize reserved assignment forms (`MessageName`, `Default`, metadata
   assignments, pattern wrappers) before treating the lhs as an ordinary
   executable call.
2. **Preserve the existing pattern-region policy** (`definitions.rs:512`):
   `Pattern[name, pattern]` keeps the binder name raw and prepares the pattern;
   blank-family head constraints are prepared; `PatternTest` and `Condition`
   prepare only the pattern operand and leave the predicate deferred;
   `HoldPattern`, `MessageName`, `Default` stay opaque to user evaluation;
   `Alternatives`, `Optional`, `Except`, `Repeated`, `RepeatedNull` keep their
   recursive treatment. Preparation must never apply a rule to pattern syntax.
3. Evaluate a non-pattern call head exactly as ordinary evaluation does
   (`eval_core/call.rs:21`). A head like `headFactory[]` may apply definitions
   and may suspend.
4. Snapshot call metadata immediately after the head is ready, using the same
   builder as `eval_dispatch/mod.rs:5`, including `SubValuesHoldAll` for nested
   callable heads. One plan snapshot per preparation — do not re-read
   attributes after an earlier argument mutates them.
5. Prepare arguments left to right with the planner from
   `eval_call_prepare.rs:152`: active positions evaluate once (delayed
   own-values and nested definitions included); held positions keep their raw
   subtree; `Evaluate` escapes ordinary holds; `HoldAllComplete` blocks that
   escape; `Unevaluated` strips its wrapper and suppresses evaluation once;
   `Sequence` splices unless `SequenceHold` or `HoldAllComplete` blocks it.
6. **Reuse the original `SharedArgs` when nothing changes.** Allocate only
   after the first real change, as the ordinary path does
   (`eval_call_prepare.rs:514`).
7. Apply the root `Flat`/`Orderless` normalization (`eval_call_prepare.rs:745`),
   and when the outer head holds, also the recursive held-match normalization
   (`resolution.rs:58`). This is what closes the held-ground defect: the exact
   index and the compiled matcher must agree.
8. **Stop before outer `Listable`, upvalue, downvalue, subvalue, builtin, or
   function dispatch.** An active nested argument uses the full evaluator; the
   outer lhs does not apply itself.
9. Classify ownership only after canonicalization.
10. The stored identity is a **state snapshot**: later own-value or attribute
    changes affect future preparation and lookup but never rewrite existing
    definitions.

**Preparation is not evaluation.** `self.evaluate(lhs)` is wrong: it continues
past identity preparation and can apply the very definition being installed.
If you find yourself reaching for it, the seam is misplaced — stop and report.

## Ordering, and what must not change

`Set` evaluates its RHS first, then prepares the lhs, then checks protection
and stores. `SetDelayed` prepares the lhs without evaluating the RHS. Keep
that order; the new seam replaces the current normalization point
(`normalize_definition_lhs_call_heads`), it does not move it.

Already-correct behavior that must survive: `iImm = 2; fImmediate[iImm] = 7`
still stores `fImmediate[2]`; pattern binder names and typed blanks stay raw;
`HoldAllComplete`'s upvalue suppression is documented behavior, not a defect.

## Scope

Files you MAY modify:

- new `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs`
  and its resume-frame module
- `crates/rum-runtime/src/runtime/definitions/definitions.rs`
- `crates/rum-runtime/src/runtime/definitions/mod.rs`
- `crates/rum-runtime/src/runtime/call_preparation.rs`
- `crates/rum-runtime/src/runtime/resolution.rs`
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
- the runtime and stdlib test modules covering `Set`/`SetDelayed` identity
- `docs/guide/evaluation.md`

**The MAY list is a floor, not a ceiling.** A purely mechanical ripple of an
in-scope change — an exhaustive `match` arm forced by a new variant, an
exhaustive literal forced by a changed field set, a call site forced by a
changed signature, a re-export forced by a moved item — you MAY edit unlisted;
list each in your report under "mechanical ripple". Anything changing
behavior, contract, or a test expectation in an unlisted file: STOP and report.
When unsure, it is the second kind.

Out of scope: `Expr`/`Symbol` representation, public serde shapes,
`Cargo.lock`, `.git`, `TODO.md`, `PROGRESS.md`, `AGENTS.md`, other `docs/**`.

## Suspension safety

Preparation runs user code, so it suspends. Per plan-378: a persistent
zipper/stack holding the current expression and pattern context, the evaluated
head with its captured metadata, the original `SharedArgs`, the argument
cursor, the persistent prepared prefix, `Unevaluated` reconstruction metadata,
parent call contexts, and the consumer continuation.

- The cursor advances exactly once when a resumed value is merged; earlier
  arguments live in an `Arc`-backed prefix and are **never replayed**.
- Every state, node and frame derives `FrameRootTrace` and is listed in
  `runtime/tests/frame_root_trace/derived_inventory.rs`.
- Frames are inserted only through `resume_or_suspend!`; the queue stays
  private.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The bounded differential gate

1a's gate was an empty diff. This step changes behavior, so the gate becomes:
**the diff must be non-empty in exactly the expected places and empty
everywhere else.**

- Reuse 1a's probe, `/tmp/rum-brief-379-*/behavior-preservation.rum`, extended
  with the held-ground-`Flat` case and any case you add.
- **Calibrate it first**, as 1a did: run it against a pre-family-A binary and
  show it reports a diff. A scanner that reports clean must be shown capable of
  finding something. Paste that calibration.
- Then diff `HEAD` against your candidate and **enumerate every differing
  line, with a one-line justification each**. A diff line you cannot justify
  is a defect, not an improvement.

### Test matrix

- B3: immediate and delayed own-values in active positions, `SetDelayed` too.
- B4: `HoldFirst`, `HoldRest`, `HoldAll`, `HoldAllComplete`, and partial holds,
  each asserting both the stored identity and that the **source-form lookup**
  reaches it.
- `Evaluate` escaping `HoldAll` but not `HoldAllComplete`; `Unevaluated`
  stripping; `Sequence` splicing with and without `SequenceHold`.
- Nested calls: active nested applies definitions, held nested does not;
  callable heads (`headFactory[]`) evaluate before attributes are read.
- Held ground `Flat` and `Orderless`: the stored identity is reachable from
  both the source form and the normalized form.
- Pattern regions: binder names, typed blanks, `PatternTest`, `Condition`.
- Suspension in the head and in multiple arguments, with counters proving
  left-to-right and exactly-once, and no replay after resume.

**Assert after the invalidating action.** Preparation reads session state, so
the gate must act after that state changes, not only before:

- after `SetAttributes` adds a hold attribute to a head that **already has**
  definitions — old and newly prepared identities must coexist correctly;
- after an lhs own-value is reassigned and then cleared — future identities
  change while stored ones stay put (contract point 10);
- after `Clear[head]` and reinstall — no stale prepared identity or matcher.

This line lost six green gates to tests that asserted only before the trigger.
Say per test which invalidation it acts after.

Also name the entry point per form and show the changed code is on the path an
ordinary call takes. "I added the logic" is not "the logic runs" — brief-348
wrote correct rebuild code a fast path returned before reaching, and
`definitions.rs` has several such fast paths.

### Performance gate

`Set` runs on every assignment. Required:

- The already-canonical case must not pay for preparation: immediate symbol,
  Options, SyntaxInformation, association-lvalue and Part-lvalue fast paths
  stay ahead of it, and an lhs needing no change must reuse its `SharedArgs`
  with no allocation. State what that case pays; if it cannot be made free,
  STOP and report with the measurement rather than shipping it.
- rum-vs-rum A/B, one variable, profiling OFF, same script against both saved
  binaries, best-of-2 per binary, four executions; state the cap.
- Targets: `set-ownvalue`, the `downvalue-*` and `subvalue-*` rows, and
  `symbol-ownvalue-read`.
- **Do not name "untouched" control labels the way brief-379 did.** That was a
  mistake: a change on the common call path leaves nothing untouched, and the
  four labels named there were one tier of fast labels that moved as a group
  (median -7.16% against a suite median of -1.83%), which would have read as a
  4.5% target regression that did not exist. Judge each label against **its own
  same-build spread**, and report the suite distribution for context. Report
  both runs per binary so the spread is visible.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
- No unconditional cost on the ordinary evaluation path; instrumentation goes
  behind the env-gated cold-load `AtomicU8` pattern.
- No owned-tree deep clones on the assignment path; `Expr`/`SharedExpr`/
  `SharedArgs` are `Arc`-shared.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only; no other
  vendor's engine, no oracle harness, no third-party output committed. WL
  semantics may inform design and be described in prose; they never enter the
  repo as a test oracle.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized from a
  measured run of the same shape.
- **Stop-loss** — report rather than improvise if: preparation cannot avoid
  applying the outer definition; the already-canonical case cannot be made
  allocation-free; fixing B3 and B4 together requires a representation change
  beyond this scope; or a differential line appears that you cannot justify.
  A stop here is a correct outcome.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Mechanical ripple — every file edited under the criterion above.
3. Verification — `fmt`, workspace counts, the calibration, and the bounded
   diff with every differing line justified.
4. The A/B — target rows with same-build spreads, suite distribution, cap, and
   what the already-canonical `Set` pays.
5. Stop-loss / open questions.
