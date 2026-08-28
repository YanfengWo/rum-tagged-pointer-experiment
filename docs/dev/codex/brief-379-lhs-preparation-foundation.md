# brief-379 — call-preparation primitives and a prepared-identity view (no behavior change)

Step 1a of plan-378 (repair family B). The plan is in `/tmp/cdx_brief_378.log`.

Plan-378 proposed one very large first step — preparation primitives, a
`CallDispatchState` rework, `Set`/`SetDelayed`, `Unset`, value tables, list
destructuring, package scanning, and a sealed store type — all at once. The
director has split it. This brief is the foundation only, and its acceptance
criterion is the strongest one available: **no observable behavior changes at
all.**

The reason for the split is not size, it is gateability. A step this central
must be independently verifiable and independently revertible; if a
seven-consumer step fails its gate, nothing localizes the fault. The
preparation module itself is deliberately **not** in this brief — it moves to
1b with `Set`, its first consumer, so that no unverified, uncalled new code
lands here.

## Goal

Two refactors inside the evaluator's call path, both behavior-preserving:

1. **Extract the reusable call-preparation primitives** currently embedded in
   `crates/rum-runtime/src/runtime/eval_call_prepare.rs` (31.7k, 905 lines):
   call metadata, the argument plan, `Sequence` splicing, and identity
   finalization. They are needed by the definition-lhs preparation that 1b
   introduces, and today they exist only inline in the ordinary-call path.
2. **Give lookup a prepared-identity view.** Turn `CallDispatchState`
   (`session/session_state.rs:175`) into a state abstraction that distinguishes
   raw from prepared identity and hands a *borrowed* prepared-identity view to
   up-, down-, and sub-value lookup, instead of exposing its fields.

Extraction means **moving** code, not copying it: the ordinary call path must
call the extracted primitives afterwards. If the old inline code survives
alongside a new copy, the behavior-preservation gate proves nothing, because
the new code is not the code being exercised.

## What must NOT change

Everything observable. Specifically:

- No stored definition identity changes. `fD[iD] = 8` under `iD := 1 + 1` still
  stores `HoldPattern[fD[1 + 1]]`, and `fH[xH] = 9` under a `HoldFirst` head
  still stores `HoldPattern[fH[5]]`. **Those are the defects family B exists to
  fix, and they stay broken in this step.** Fixing one here would destroy this
  brief's acceptance criterion.
- No lookup result changes, no message changes, no rendering changes, no
  evaluation-order changes.
- No new module for definition-lhs preparation, and no consumer migrated:
  `Set`, `SetDelayed`, `Unset`, value-table assignment, `TagSet`, `UpSet`,
  `TagUnset`, `AssociateTo`, and `KeyDropFrom` are all untouched in this step.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_call_prepare.rs`
- `crates/rum-runtime/src/runtime/session/session_state.rs`
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/src/runtime/eval_core/call.rs`
- new module(s) under `crates/rum-runtime/src/runtime/` holding the extracted
  primitives
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
- the runtime test modules needed to cover the above

**The MAY list is a floor, not a ceiling.** A purely mechanical ripple of an
in-scope change — an exhaustive `match` arm forced by a new variant, an
exhaustive struct literal forced by a changed field set, a call site forced by
a changed signature, a re-export forced by a moved item — you MAY edit even if
unlisted; list each one in your report under "mechanical ripple". Anything that
changes behavior, contract, or a test expectation in an unlisted file: STOP and
report instead. If you cannot tell which kind it is, it is the second kind.

Out of scope: `Expr`/`Symbol` representation, any public serde shape,
`Cargo.lock`, `.git`, `TODO.md`, `PROGRESS.md`, `AGENTS.md`, `docs/**`.

## Suspension safety

`CallDispatchState` is already inside the frame-root system — it is asserted at
`session/session_state.rs:632` and appears as a field on two suspension frames
(`eval_core/call.rs:387` and `:425`). So:

- Every type you introduce or reshape here derives `FrameRootTrace` and appears
  in `runtime/tests/frame_root_trace/derived_inventory.rs`.
- The frame queue stays private; frames are inserted only through
  `resume_or_suspend!`.
- A borrowed prepared-identity view must not extend a borrow across a suspend
  point. If the borrow cannot be made to work without cloning the identity on
  the ordinary path, **STOP and report** — do not pay a per-call clone to make
  the shape fit. That tradeoff is a design decision, not an implementation
  detail, and it belongs to the director.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The behavior-preservation gate

Tests alone cannot carry this brief: they cover what someone thought to
assert. Build a differential probe and run it on both binaries.

- Write one `.rum` probe exercising every path that goes through call
  preparation and dispatch: down-value exact and pattern lookup, up-values,
  sub-values, `Flat`/`Orderless`/`OneIdentity` heads, all five hold attributes,
  `Sequence` splicing with and without `SequenceHold`, `Evaluate` and
  `Unevaluated`, nested and callable heads, immediate and delayed assignment,
  `Unset`, value-table assignment, and each defect B1-B4 in its current broken
  state.
- Print the result **and** the resulting `DownValues`/`UpValues`/`SubValues`
  for every case, so a changed stored identity shows up as a diff.
- Run it against a base binary built from `HEAD` and a candidate binary built
  from your tree, then `diff` the two outputs. **The diff must be empty.**
- Paste the `diff` invocation and its (empty) output, plus the two binaries'
  SHA-256.

This is rum-vs-rum, which the Testing Policy permits; no other engine is
involved.

### Performance gate

This is the hottest path in the evaluator, so a rum-vs-rum A/B is required.
One variable, profiling env OFF, same script against both saved binaries.

- Target labels: the down/up/sub-value lookup rows, plus `literal-integer`,
  `symbol-ownvalue-read`, `set-ownvalue`, and `part-list`.
- Named controls: `literal-string`, `list-literal-small`, `sameq-integer`,
  `string-repeat-small`.
- Best-of-2 per binary, four suite executions total; state the cap.
- **Judge against the named controls, never the suite median.** Family A's run
  carried a +1.81% whole-run offset measured on 102 untouched labels while the
  ten affected ones moved +1.02%; the raw split was 86-of-112 slower and reads
  as a regression until the controls are consulted. Report the control
  movement alongside the target movement, and report the same-build spread
  between your two runs of each binary so the noise floor is visible.
- A pure refactor that costs measurable time on the ordinary call path is a
  defect here, not a tradeoff. If you cannot get it to parity, STOP and report
  with the numbers rather than shipping a slower evaluator.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
- No unconditional cost on the ordinary evaluation path; instrumentation, if
  any, goes behind the env-gated cold-load `AtomicU8` pattern.
- No owned-tree deep clones introduced on the call path; `Expr`/`SharedExpr`/
  `SharedArgs` are `Arc`-shared.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only; no other
  vendor's engine, no oracle harness, no third-party output committed.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized from a
  measured run of the same shape.
- Minimal diff. This is a refactor: no renames or reformatting beyond what the
  extraction itself requires.
- **Stop-loss.** Three specific cases, all of which mean STOP and report rather
  than improvise:
  1. An extraction cannot preserve behavior — the inline code depends on
     surrounding state in a way the primitive cannot express.
  2. The borrowed view forces a per-call clone (see Suspension safety).
  3. The differential diff is non-empty and you cannot make it empty without
     changing what a consumer stores or looks up.

  A stop here is a correct outcome, not a failure. Recent stop-losses on this
  line were correct calls on the director's own omissions.
- Paste real verbatim output. If a run did not reach green, say so explicitly.

## Report format

1. What changed — per file, line ranges, one-line why. State explicitly, per
   primitive, that the ordinary path now calls it and the inline original is
   gone.
2. Mechanical ripple — every file edited under the criterion above.
3. Verification — `fmt`, workspace tests with counts, and the differential
   probe with its empty `diff` and both SHA-256 values.
4. The A/B — target rows, control rows, same-build spread, execution cap.
5. Stop-loss / open questions.
