# brief-377 — evaluated `Part` selectors across all thirteen mutation heads

Step 1 of plan-376 (repair family A). The plan is in
`/tmp/cdx_brief_376.log`; this brief is the director's rewrite of its draft A
and supersedes it where they differ.

## Goal

Every assignment-style `Part` mutation must consume an opaque, exactly-once
evaluated selector sequence. Today the pure selector parsers receive *raw*
expressions on the mutation paths, so `kk = 1; lst[[kk]] += 5` reports
`Part::pkspec1` and changes nothing, while `lst[[kk]] = 99` and `lst[[1]] += 5`
both work. Fix all eleven held-update heads plus the `Part` branches of
`AssociateTo` and `KeyDropFrom` — thirteen in total — without disturbing plain
`Set`, association `Set`, bulk selection, return conventions, diagnostics,
suspension behavior, or `AppendTo`'s literal incremental fast path.

The seam must be typed, not a convention: assignment-target parsers accept
only an `EvaluatedPartSelectors`, and the sole ways to obtain one are
exactly-once evaluation or a constructor that proves the selectors are stable
literals. A future mutation head must then be unable to pass raw selectors
without the compiler stopping it. **A repair the compiler enforces beats one
that relies on remembering to call something** — that is the standing
preference on this project, and it is why this step is typed rather than a
matter of fixing three call sites.

## Established evidence

Director's probes, release CLI at `65c03870`, each with a liveness assertion
and a working control. Reproduced independently by plan-376 on all eleven
runtime heads:

```
kk = 1; lst = {10, 20, 30}; lst[[kk]] += 5;
  → lst unchanged, and two messages:
    Part::pkspec1: The expression kk cannot be used as a part specification.
    Set::pkspec1:  The expression kk cannot be used as a part specification.

lst2[[kk]] = 99   → {99, 20, 30}   (plain Set, symbol selector — works)
lst3[[1]] += 5    → {15, 20, 30}   (compound, literal selector — works)
```

The eleven runtime heads, all confirmed failing with a symbol selector and
working with a literal one — `AddTo`, `SubtractFrom`, `TimesBy`, `DivideBy`,
`ApplyTo`, `AppendTo`, `PrependTo`, `Increment`, `Decrement`, `PreIncrement`,
`PreDecrement` — all converging on `current_held_update_value`
(`definition_updates.rs:319`). Plus two association heads whose own `Part`
branches fail the same way (`mutation.rs:513`), which is what makes the
surface thirteen rather than eleven:

```
AssociateTo::wrsym: Symbol Part is Protected.
KeyDropFrom::invak: The argument Part[kda, kdpk] is not a valid Association.
```

Where the raw selectors enter, and the working model to copy:

- Plain `Set` evaluates selectors first (`part_assignment.rs:119`) and passes
  evaluated values (`part_assignment.rs:227`). **This is the reference.**
- Held updates pass raw selectors (`definition_updates.rs:388` and `:403`).
- Association mutation passes raw selectors (`mutation.rs:513`).
- The parsers themselves are correctly pure (`structure_data/mod.rs:35`) and
  must stay that way.
- `AppendTo`'s incremental fast path passes raw selectors
  (`incremental_append.rs:115`) but only after proving them stable positive
  literals. That proof must survive as a zero-callback constructor.
- Association `Set` already has an independent, correct exactly-once driver
  (`association_data.rs:762`).

**Design input (WL family semantics, prose only — never a test oracle):** the
compound form updates the selected position, exactly as the plain assignment
does. rum's own contract is the one being restored here; nothing about the
WL comparison enters the repository, per the Testing Policy.

## The evaluation-order contract this step commits to

State it in `docs/guide/evaluation.md` and enforce it in tests:

1. For the seven binary heads the second argument is already evaluated once
   during ordinary `HoldFirst` argument preparation
   (`eval_call_prepare.rs:444`) — **never evaluate it again**.
2. Selectors evaluate exactly once, left to right.
3. Only after the last selector may the owner's removal / protection / value
   state be read. Re-fetch it there; do not reuse a pre-selector snapshot.
   Plain `Set` already states this invariant at `part_assignment.rs:193`.
4. Paths and the current selection come from that post-selector snapshot.
5. The update evaluates once.
6. Protection is rechecked at store time.
7. Each head keeps its current return convention — `Increment` and `Decrement`
   return the previous selection, the other nine return the updated value.

Diagnostics for an invalid selector must name the *evaluated* offender, not
the raw expression.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/definitions/definition_updates.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs`
- `crates/rum-runtime/src/runtime/definitions/part_assignment.rs`
- `crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs`
- `crates/rum-runtime/src/runtime/definitions/incremental_append.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs`
- `crates/rum-runtime/src/runtime/kernel_data/mod.rs`
- `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`
- `crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/**`
- new selector-state module(s) under `crates/rum-runtime/src/runtime/kernel_data/`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- `crates/rum-runtime/src/runtime/tests/part_assignment.rs`
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
- `crates/rum-stdlib/src/associations/mutation.rs`
- `crates/rum-stdlib/src/associations/mutation/resume_frames.rs`
- `crates/rum-stdlib/src/tests/incidental_definitions_updates.rs`
- `crates/rum-stdlib/src/tests/associations/associations_mutation_surface.rs`
- `crates/rum-stdlib/src/tests/session_requests/transforms_resume_associations.rs`
- `docs/guide/evaluation.md`
- `scripts/core-language-microbench.sh`

**The MAY list is not a promise of completeness.** This project has lost five
dispatches to a list that missed a mechanical consequence, so the rule is a
criterion, not a longer list:

- A **purely mechanical** ripple of an in-scope change — an exhaustive `match`
  arm forced by a new enum variant, an exhaustive struct literal forced by a
  new field, a call site forced by a changed signature, a re-export forced by
  a moved item — you MAY edit even if its file is not listed. List every such
  file in your report under "mechanical ripple", one line each.
- Anything that changes **behavior, contract, or test expectations** in a file
  not listed above: STOP and report instead. If you cannot tell which side a
  change falls on, it is the second kind.

Explicitly OUT of scope for this step, even though it is the same family:

- **Do not touch the `Set`/`SetDelayed` lhs normalization**, and do not change
  what identity any definition is stored under. `fDelayed[iDel] = 8` storing
  `HoldPattern[fD[1 + 1]]`, and `fHeld[xHeld] = 9` storing `HoldPattern[fH[5]]`
  when the head is `HoldFirst`, are both real defects — they are repair family
  B and have their own briefs. Selector evaluation only.
- **Do not change the `AssociateTo`/`KeyDropFrom` fallback store identity**
  (`mutation.rs:225` / `:364`). That is D1, also family B. This step fixes only
  those two heads' `Part` *selector* handling.
- `Expr`/`Symbol` representation, any public serde shape, `Cargo.lock`, `.git`,
  `TODO.md`, `PROGRESS.md`, `AGENTS.md`, other `docs/**`.

## Deliverable

- An opaque `EvaluatedPartSelectors` that assignment-target parsers require.
- An exactly-once, resumable selector driver.
- All thirteen mutation heads routed through it.
- A stable-literal constructor that keeps `AppendTo`'s incremental fast path
  zero-callback.
- The order contract documented in `docs/guide/evaluation.md`.
- The full test matrix below, and the microbench A/B rows.

## Suspension safety

These are `eval_*` paths with resume frames, and briefs 364-375 made frame
roots a compile-time obligation:

- Selector sequence state holds raw selectors, a cursor, and the evaluated
  prefix; every new state or frame type **derives `FrameRootTrace`** and
  appears in `runtime/tests/frame_root_trace/derived_inventory.rs`.
- Insert continuation frames only through `resume_or_suspend!`. The frame
  queue is private and stays that way — do not add an insertion route.
- A resumed selector is appended before the cursor advances, so neither it nor
  any earlier selector replays. Prove this with the two-suspension test below.
- Do not deep-clone the evaluated prefix on suspension.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

Workspace, not `-p rum-runtime`: this change crosses into `rum-stdlib` through
the kernel bridge, so a crate-local green is not a green tree.

### Test matrix (all of it is the gate)

- Table-driven across **all eleven** runtime heads: rebound symbol selector,
  with a literal-selector control in the same table.
- Both association heads with symbol selectors on `Part` targets.
- Multi-selector left-to-right order.
- RHS-before-selector order for all seven binary heads.
- Return conventions preserved: `Increment`/`Decrement` return the previous
  selection, the other nine the updated value.
- Invalid evaluated selectors: the diagnostic names the **evaluated** offender.
- One- and two-suspension tests (via `InputString`) proving no replay and a
  post-resume re-fetch.
- Existing plain `Set`, association `Set`, bulk selector, alias/COW, and
  `AppendTo` incremental tests stay green.

**Assert after the trigger, not only before it.** Selector evaluation can run
user code, so the test matrix must include selectors whose evaluation
reassigns the owner, clears it, removes it, protects it, and unprotects it —
with the assertions taken *after* that side effect. This line lost six green
gates to tests that asserted only before the invalidating action; a gate of
that shape does not count here.

Also confirm the changed code is actually on the path an ordinary call takes.
Brief-348 wrote correct rebuild logic that a fast path returned before
reaching. Name the entry point per head, and if a fast path can short-circuit
the new seam, test the fast path too.

### Performance gate

`Part` mutation is a hot path, so a rum-vs-rum A/B is required, one variable,
profiling env OFF:

- Existing rows that must not regress: `appendto-part-outer1000-small/large`,
  `appendto-list-incremental-small/large`, all four `set-part-*` rows, and
  `association-set-key-incremental-small/large`.
- You MAY add permanent symbol-selector rows for held updates and association
  `Part` mutation. **Every added label carries a liveness assertion** — a probe
  that silently measures a failing path is this project's most expensive
  recorded mistake, and this brief's own defect makes the mutation a no-op, so
  an unasserted new label would measure exactly nothing.
- Run the **same script** against saved base and candidate binaries; never a
  base-script/candidate-script comparison.
- Best-of-2 per build, four suite executions total; state the cap.
- Judge against named untouched control labels, never the suite median.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree.
- **The pure selector parsers must not acquire evaluator callbacks.** If you
  find a path where evaluation genuinely cannot complete before parsing —
  for instance a selector whose evaluation must suspend at a point the parser
  has already been entered — that means the seam is in the wrong place: STOP
  and report it, do not thread a callback into the parser to get past it.
- No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
- No unconditional cost on the ordinary, non-assignment evaluation path. Any
  instrumentation goes behind the env-gated cold-load `AtomicU8` pattern.
- No owned-tree deep clones on hot paths; `Expr`/`SharedExpr`/`SharedArgs` are
  `Arc`-shared.
- **Testing Policy is a hard line** (`AGENTS.md`): no differential testing
  against another vendor's engine, no oracle harness, no third-party output
  committed.
- Timeout discipline: use `scripts/run-bounded.py --timeout SECONDS` and size
  the bound from a measured run of the same shape.
- Minimal diff — no drive-by reformatting or renames.
- **Stop-loss**: if the same step fails twice for the same reason, or if this
  brief turns out to be unsatisfiable as written (a prohibition blocking a
  legitimate necessary case, an instruction contradicting what you find in the
  code), STOP and report the specific contradiction. Do not work around it
  silently. Recent stop-losses on this line were correct calls on the
  director's omissions.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Mechanical ripple — every file edited under the mechanical-ripple criterion.
3. Verification — each command with verbatim output (test counts, per-label
   rows).
4. The A/B — before/after rows for the target labels AND the named controls,
   with the execution cap you used.
5. Stop-loss / open questions.
