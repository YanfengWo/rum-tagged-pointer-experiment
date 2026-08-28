# brief-381 — `Unset` and value-table assignment on the shared preparation

Step 1c of plan-378 (repair family B). 1a landed the primitives (`5698e8e5`);
1b landed `PreparedDefinitionLhs` with `Set`/`SetDelayed` as its first consumer
(`aa441ff0`), closing B3, B4 and the held-ground `Flat` defect.

This step migrates the two remaining ordinary-assignment consumers, so that
removal and bulk installation construct identity the same way installation
does.

## Goal

Route `Unset` and value-table assignment through `PreparedDefinitionLhs`,
closing family B's remaining two ordinary-path defects.

**Director's probes on the current tree (`aa441ff0`), each with a liveness
assertion:**

```
iU = 2; fU[iU] = 701;
  Set stores  {HoldPattern[fU[2]] :> 701}      (correct since before this line)
  Unset[fU[iU]] leaves it in place             (defect — removal compares a raw lhs)

iV = 4; DownValues[fV] = {HoldPattern[fV[iV]] :> 703};
  stores {HoldPattern[fV[iV]] :> 703}, and fV[4] does not reach it
```

**One thing 1b changed that you must not misread.** `Unset` on a *held*
position now works — `SetAttributes[fUH, HoldFirst]; xUH = 5; fUH[xUH] = 702;
Unset[fUH[xUH]]` does remove the definition. That is a coincidence, not
correctness: 1b made `Set` store the raw held form, and `Unset` also compares a
raw form, so two unprepared expressions happen to be equal. Once `Unset`
prepares its lhs the held position must still work, because preparation keeps
held positions raw — but it will then work *by construction*. Keep a test on
it either way.

Already covered, do not redo: list destructuring (`{fL[iL], gL} = {10, 20}`
already stores `fL[2]` after 1b).

Out of scope, each with its own step: the `AssociateTo`/`KeyDropFrom` fallback
(step 2), and `TagSet`/`UpSet`/`TagUnset` plus the final sealing (step 3). Do
not migrate them here — `TagUnset` in particular looks like it belongs with
`Unset`, and it does not: it needs the tagged classification that step 3
introduces.

## The contract

Preparation is defined by 1b and is unchanged. Two consequences settle what
these consumers must do — plan-378's contract point 10, which the director has
adopted:

**A stored identity is a state snapshot.** Later own-value or attribute changes
affect future preparation and lookup; they never rewrite existing definitions.
Therefore:

- `i = 2; f[i] = 1; Unset[f[i]]` **removes** it — both sides prepare to `f[2]`.
- `i = 2; f[i] = 1; i = 3; Unset[f[i]]` **does not** remove it: the removal
  addresses `f[3]`, a different identity. This is correct, not a gap.
- `Clear[f]` remains the operation that removes every identity owned by `f`,
  and is unchanged by this step.

**Value-table assignment is a definition store**, so every imported lhs is
prepared. It must also stay **transactional**: parse, prepare and validate every
entry into a persistent accumulator, recheck protection, and only then clear
and store. A preparation that fails — or suspends and then fails — on entry
five must not leave the table already cleared.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/definitions/definition_clear.rs`
- `crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs`
  and its resume-frame module
- `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs`
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
- the runtime and stdlib test modules covering `Unset` and value tables
- `docs/guide/evaluation.md`

**The MAY list is a floor, not a ceiling.** A purely mechanical ripple of an
in-scope change — an exhaustive `match` arm forced by a new variant, an
exhaustive literal forced by a changed field set, a call site forced by a
changed signature, a `FrameRootTrace` derive forced by a type entering frame
state — you MAY edit unlisted; list each in your report under "mechanical
ripple". Anything changing behavior, contract, or a test expectation in an
unlisted file: STOP and report. When unsure, it is the second kind.

Out of scope: `Expr`/`Symbol` representation, public serde shapes,
`Cargo.lock`, `.git`, `TODO.md`, `PROGRESS.md`, `AGENTS.md`, other `docs/**`.

## Required first step — inventory the surface

Before changing anything, enumerate and report:

- Every removal form that must prepare: `Unset[f[x]]`, the `f[x] =.` syntax,
  and any operator or held variants. State which reach the same entry point.
- Every value-table surface: `DownValues`, `UpValues`, `SubValues`,
  `OwnValues`, `NValues`, `DefaultValues`, `Messages`, `Options`, and anything
  else assignable as a table. Say which are definition stores needing
  preparation and which are not, with evidence.
- Which of these already behave correctly, so the diff stays bounded.

If a surface turns out already correct, say so and leave it alone.

## Suspension safety

Preparation runs user code, so both consumers can suspend — and value-table
assignment can suspend **once per entry**.

- Reuse 1b's zipper state; do not fork a second preparation path.
- The accumulator holding prepared entries is persistent and `Arc`-backed; a
  resumed entry is appended before the cursor advances, so no entry is
  prepared twice. Prove it with a counter.
- Every new state/frame derives `FrameRootTrace` and is listed in
  `runtime/tests/frame_root_trace/derived_inventory.rs`; frames are inserted
  only through `resume_or_suspend!`; the queue stays private.
- **Transactionality across suspension is the interesting case**: a table
  assignment that suspends midway and then fails must leave the old table
  intact. Test it.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The bounded differential gate

Same shape as 1b: **non-empty in exactly the expected places, empty everywhere
else, every differing line justified.**

- Reuse and extend `/tmp/rum-brief-380-*/behavior-preservation.rum`.
- **Calibrate first**: run the probe against a pre-family-A binary and show it
  reports a diff. A scanner that reports clean must be shown capable of
  finding something.
- Then diff `HEAD` against your candidate. The B3/B4/`Flat`/`Orderless` lines
  1b fixed must **stay fixed** — a regression there is a stop, not a diff line
  to justify.

### Test matrix

- `Unset` on active positions (the defect), on held positions (the coincidence
  above, now by construction), and on pattern lhs.
- The snapshot semantics, both directions: removal after no change removes;
  removal after rebinding the own-value does not, and the original definition
  survives intact.
- `Clear[f]` still removes everything regardless of identity.
- Value tables: active and held heads, patterns, and every surface your
  inventory found to be a definition store.
- Transactionality: an invalid entry late in the list leaves the previous table
  untouched; the same across a suspension.
- Suspension in `Unset`'s lhs and in multiple table entries, with counters
  proving exactly-once and no replay.

**Assert after the invalidating action**, not only before it:

- after `SetAttributes` adds a hold attribute to a head that already has
  definitions — removal must address the newly prepared identity while the
  stored one stays as it was;
- after an lhs own-value is reassigned and then cleared;
- after `Clear[head]` and reinstall.

This line lost six green gates to tests asserting only before the trigger. Say
per test which invalidation it acts after. And name the entry point per form:
"I added the logic" is not "the logic runs" (brief-348), and `definition_clear.rs`
has early exits.

### Performance gate

Neither `Unset` nor value-table assignment is on the ordinary evaluation path,
so the expectation is no movement at all outside them.

- rum-vs-rum A/B, one variable, profiling OFF, same script both binaries,
  **best-of-3 per binary** — two runs do not estimate a noise floor.
- **Controls apply for this step.** Unlike 1a, this change does not sit on the
  common call path, so the pure-computation labels are a genuine control
  group. That distinction decided 1b's gate: its 28 pure-computation labels
  moved +3.82% at the median while the definition-path labels moved +2.10%, so
  the affected labels moved *less* than the controls and three apparent
  regressions dissolved. Report the control-group median alongside the
  definition-path median, and each label's own three-run spread.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
- No unconditional cost on the ordinary evaluation path.
- No owned-tree deep clones; `Expr`/`SharedExpr`/`SharedArgs` are `Arc`-shared.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only; no other
  vendor's engine, no oracle harness, no third-party output committed.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized from a
  measured run of the same shape.
- **Stop-loss** — report rather than improvise if: a removal form cannot use
  the shared preparation without a second code path; transactionality cannot
  be preserved across suspension without buffering the whole old table; a
  1b-fixed differential line regresses; or a differing line appears you cannot
  justify.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. The surface inventory — removal forms and value-table surfaces, each marked
   needs-preparation or already-correct, with evidence.
2. What changed — per file, line ranges, one-line why.
3. Mechanical ripple — every file edited under the criterion above.
4. Verification — `fmt`, workspace counts, the calibration, and the bounded
   diff with every differing line justified.
5. The A/B — control-group median, definition-path median, per-label three-run
   spreads.
6. Stop-loss / open questions.
