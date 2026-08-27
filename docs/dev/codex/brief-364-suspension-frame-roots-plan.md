# brief-364 — gensym roots hidden inside suspension frames (design only)

## Goal

The gensym mark-and-sweep does not see `Expr` values held by suspension
frames, so a gensym reachable only from a suspended continuation can be
reclaimed while it is still needed. The root walk covers the suspension's
`active_stack` and stops there; the frames themselves are
`Box<dyn SuspensionFrame>` wrapping arbitrary Rust closures, and a closure's
captured `Expr` cannot be enumerated by any traversal that exists today.

The repair has already been decided by the director (see **The decided shape**
below): make frame state a named type instead of a closure capture, derive the
`Expr` traversal from the type's fields, and delete the closure-taking API so
the derived path is the only path. Your job is to produce the step-by-step
plan for that, plus drafted follow-up implementation briefs — **and, first, to
re-probe the defect and establish how wide it actually is.**

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes
with the release `rum-cli` plus throwaway `#[test]` scratch files under `/tmp`
are in scope; leave no probe files in the repository, and feed every
`rum-cli` probe `stdin < /dev/null`.

## The decided shape (do not re-litigate; do critique the details)

Three parts, and the director's position is that all three are required —
any one alone degrades into "an obligation people have to remember":

1. **Delete `ClosureFrame<F>` and the closure-taking `suspend_with_frame*` /
   `drive_*_value` API** (`crates/rum-runtime/src/runtime/suspension/core.rs:24`
   and the wrappers around it). While a closure-taking entry point exists,
   nothing stops a call site from using it.
2. **Every frame becomes a named `struct` implementing `SuspensionFrame`**,
   with its captured state as fields.
3. **The `Expr` traversal is derived from the fields**, so adding a field to a
   frame covers it automatically and adding a frame without the derive fails
   to compile.

The property being bought is a reversal of the default: today forgetting is
silent and correct behavior costs vigilance; afterwards the compiler or the
derive covers the common case and only a named exception can be wrong.

**A standing ruling you must apply while planning (user, 2026-08-27):** this
is a research project with no downstream users and no human-labour budget.
**Pick the most correct design; the size of the change is not an argument
against it.** Do not propose a smaller variant on the grounds that the full
one touches too many call sites, and do not propose a stopgap now with the
real fix later — a temporary patch is itself debt here. Arguments that do
count: correctness, semantic loss, whether a design is structurally
unavoidable, whether it leaves an open-ended obligation, enforceability, and
measurable cost on paths that are not suspended. A pin/defer-the-sweep variant
was considered and rejected for leaving the root cause in place; you may
mention it only as a comparison point.

## Established evidence (director, tree at `bea70a5d`)

The root walk, and where it stops —
`crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:187`:

```rust
if let Some(suspension) = &self.pending_suspension {
    for entry in &suspension.active_stack {
        visit(&entry.expr);
    }
}
```

The structure it is walking — `suspension/core.rs:5`:

```rust
pub(in crate::runtime) struct SuspendedEvaluation {
    pub(in crate::runtime) active_stack: Vec<EvaluationStackEntry>,
    pub(in crate::runtime) frames: VecDeque<Box<dyn SuspensionFrame + Send>>,
}
```

A representative capture — `crates/rum-stdlib/src/associations/mutation.rs:183`
moves three `Expr` values into the closure, none of them reachable from the
root walk:

```rust
let expr = session.suspend_with_frame(target_result, {
    let call_args = args.to_vec();
    let target_expr = target_expr.clone();
    let spec = spec.clone();
    move |session, expr| finish_associate_to_value(session, call_args, target_expr, expr, spec)
})?;
```

A capture that is **not** a bare `Expr` — 
`crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:21`
captures `CompiledOptionalPattern`, `Vec<MatcherArg>`, `Vec<Expr>` and
`SymbolExprMap` (`= HashMap<Symbol, Expr>`, `model/cache.rs:13`). `MatcherArg`
is a recursive enum over `MatcherNode` / `CompiledSequencePattern` /
`CompiledOptionalPattern` (`session/session_state.rs:277-308`).

Scale, from the director's grep — **verify this count yourself and report
what you get**:

```
grep -rn "suspend_with_frame\|suspend_with_result_frame\|drive_evaluated_value\|drive_internal_value\|push_suspension_frame" \
  --include='*.rs' ./crates | grep -v "/tests/\|suspension/core.rs" | wc -l
```

gave 398 call sites across 123 files (rum-runtime 80, rum-stdlib 43).

The sweep's only non-test trigger is `scoping/lexical/module.rs:240` — a
`Module` scope exiting with `candidates >= watermark`. Default threshold
200 000 (`gensym_gc.rs:5`), watermark = candidates + threshold
(`gensym_gc.rs:209`).

Two reachable paths, both recorded in `TODO.md` §"Opened by batch 7":

- **Path A — a second request during a suspension.** `eval_request` /
  `eval_expr_request` (`session/session_requests.rs:108,126`) do not consult
  `pending_suspension`.
- **Path B — a resumption that itself crosses the watermark.**
  `resume_suspended_evaluation` (`suspension/core.rs:222`) does
  `self.pending_suspension.take()` into a **local variable** and then runs
  frames from it, so during the whole resume the remaining frames are not
  reachable from `Session` at all — the root walk cannot see them even in
  principle. Nested suspension re-establishes one and appends the remainder
  (`core.rs:250`).

The existing test covers only the half that works —
`crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs:479`,
`suspended_computation_keeps_candidate_live_until_resume_finishes`, whose
probe `List[{candidate}, List[InputString["gc: "]]]` keeps the candidate on
`active_stack`. It is green and its greenness says nothing about frames.

**The same blind spot has a second victim.** `for_each_root_expr` carries this
comment (`gensym_gc.rs:113`):

> Keep in sync with `current_memory_proxy_bytes`; both must cover every Expr-holding Session field.

and the memory proxy charges a flat 16 bytes per frame
(`debug_tools/resources/memory.rs:581`):

```rust
.saturating_add(suspended.frames.len().saturating_mul(16))
```

so the `Expr` values a frame holds are not counted toward the request memory
proxy at all. Same root cause, so it belongs in the same plan.

**Dependency situation.** The workspace has nine members and no proc-macro
crate (`Cargo.toml:3`). Its direct dependency list is deliberately small (ten
entries, `Cargo.toml:25-35`), but `serde` is already taken with the `derive`
feature (`Cargo.toml:34`), so `syn`/`quote`/`proc-macro2` are already in the
dependency tree — confirm this from `Cargo.lock` and report it, because it
changes what "a new dependency" costs here.

## What the plan must contain

1. **Re-probe first — findings age, and probes lie.** Reproduce both paths
   with a probe that carries a **liveness assertion**: show the gensym's
   definition present, then gone, by an exact printed value or an exact
   `Definition`/`OwnValues` listing. "No error" is not evidence. Two of the
   previous batch's items turned out to be already fixed, and one survey
   finding was an artifact of an invalid probe. If a path does **not**
   reproduce, say so plainly and show what you ran — but read the code path
   and settle the mechanism before calling it absent, because a weak probe's
   NOT-REPRODUCED is a false negative (this project has been bitten by exactly
   that).

2. **How wide is the exposure, really.** In particular: a matcher frame
   captures `SymbolExprMap`, the pattern-binding table, and rhs `Condition`
   guards now run user code during candidate selection, so a guard containing
   `InputString` should be able to suspend with gensyms live in the bindings.
   **Confirm or refute this specific shape with a probe.** Report any other
   frame family whose captures can hold a gensym.

3. **Frame shape inventory.** All ~398 call sites grouped by the shape of
   what they capture. How many distinct named frame types does the repair
   actually need? Which families are mechanical (`Vec<Expr>` + a plain
   continuation) and which are not (the matcher group). This number decides
   how the work is split into steps, so ground it in the grep, not in an
   estimate.

4. **Capture type closure.** The transitive set of types that must gain an
   `Expr` traversal — at minimum `MatcherArg`, `MatcherNode`,
   `CompiledSequencePattern`, `CompiledOptionalPattern`, `SymbolExprMap`,
   whatever `AssociationEntryForm` and friends turn out to be. For each: can a
   field-recursive derive cover it, or does it need a hand-written impl? A
   hand-written impl is a residual obligation, so enumerate them explicitly
   and say how each is enforced.

5. **Escape hatches — the whole residual risk lives here.** Any frame whose
   state cannot be expressed as named fields (a `Box<dyn Any>`, a function
   pointer, an interior-mutable handle, a type from another crate you cannot
   annotate). List every one you find. If the list is empty, say so and show
   how you established it. If it is not, propose how each is enforced
   mechanically — this project's precedents for that are the message-registry
   declared-or-exempt rule and the compile-time sortedness assertion
   (`8dcc80e`), not a comment asking people to be careful.

6. **The enforcement argument, made explicitly.** After the closure API is
   deleted, is there any remaining way to get a frame into
   `SuspendedEvaluation::frames` that skips the traversal? Walk the visibility
   of every constructor and field. The whole value of this design is that the
   answer is "no, and here is why the compiler says so" — if the honest answer
   is "no, provided nobody adds one", say that instead.

7. **The derive host, both options argued.** (a) a new workspace member
   (`rum-derive` or similar) with `syn`/`quote`; (b) no proc-macro at all — a
   declarative `macro_rules!` or hand-written impls plus a mechanical
   enforcement test. Give both a real hearing and recommend one. State what
   (a) does to build time, MSRV (1.85), and the published crate surface, and
   what (b) leaves as a residual obligation. Do not add any dependency in this
   brief — this is the argument, not the change.

8. **Path B's representation problem.** Naming the frames does not by itself
   make them reachable during a resume, because `resume_suspended_evaluation`
   moves the whole `SuspendedEvaluation` out of `Session`. Design that
   explicitly: what holds the remaining frames during a resume such that the
   root walk reaches them, what it costs per popped frame, and how it composes
   with the nested-suspension append at `core.rs:250`. **A plan that fixes
   only the captures and leaves this window open does not close the defect.**

9. **Memory proxy, same pass.** How the derived traversal also feeds
   `current_memory_proxy_bytes` so the two stay in sync as the comment at
   `gensym_gc.rs:113` already requires, and what the corrected numbers do to
   the existing memory-limit tests.

10. **Cost on the paths that do not suspend.** Note that the non-suspending
    arm calls the continuation directly and builds no frame
    (`core.rs:67`); the repair must keep that true. Say per step what it
    costs when nothing suspends, which steps touch the matcher (a hot path),
    and which therefore need the microbench A/B gate. Zero-by-construction is
    the target; "measured as noise afterwards" is second best.

11. **Test design, informed by how the last line failed.** The whole-line
    lesson from the guard work is that every gate asserted **before** the
    invalidating action and never after one, so six green gates missed a real
    defect. Here the analogue is obvious: a test that suspends and immediately
    asserts proves nothing. **Every test in this plan must force a sweep
    between the suspension and the assertion**, and must assert on the
    resumed value, not on the absence of an error. Say which existing tests
    encode the current shape and what replaces them.

12. **A step-by-step plan**, smallest safe steps first, each naming
    files/functions, the invariant it preserves, its gate tests, and its
    ripple inventory. Mechanical mass conversion of call sites should be its
    own step (or steps) separate from the substrate.

13. **Drafted follow-up implementation briefs** in `TEMPLATE.md` shape.

14. **Open questions for the director**, each with your recommendation.

## Scope

- Files you MAY modify: **NONE**. Design-only. Put probes in `/tmp`.
- Files/areas you MUST NOT touch: everything, including `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `CLAUDE.md`, `docs/**`, `.git`, `Cargo.toml`,
  `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Every probe needs a liveness assertion.** A probe that shows no error may
   be a probe that never reached the code. State, for each probe, the observed
   value that proves the sweep ran and that the gensym was or was not
   reclaimed. `RUM_GENSYM_GC_THRESHOLD` exists (`gensym_gc.rs:4`) so you do not
   have to allocate 200 000 candidates to force a sweep — use it, and say so.
2. **Do not design a second GC mechanism.** One root walk, one traversal. A
   parallel "frame roots" registry maintained alongside the existing one would
   be worse than the bug.
3. **Do not narrow the design to cut work.** See the standing ruling above.
   If you believe the full shape is wrong on *correctness* grounds, say so
   with evidence — that is welcome. "Too many call sites" is not a reason.
4. **Do not over-generalize either.** Scope is set by the problem. This
   project's recurring review finding is a correct root-cause observation
   attributed too widely; if a frame family genuinely cannot hold a gensym,
   say so with evidence rather than sweeping it in for symmetry.
5. **Suspension safety is the thing you are editing.** Preserve the
   suspend/resume continuation shape exactly; never collapse a suspending call
   into an eager one to simplify a frame.
6. **`Expr`/`SharedExpr`/`SharedArgs` are `Arc`-shared.** A traversal must
   borrow, never deep-clone, and must not change frame construction from
   `Arc` clone to deep clone.
7. **Stop-loss is real.** If the shape inventory or the enforcement argument
   does not resolve after two serious attempts, report what you established
   and where the trail ends.

## Verification (MANDATORY)

No source change, so no test gate. The evidence IS the deliverable:

- Verbatim probe output for path A, path B, and the matcher-bindings shape,
  each with its liveness assertion shown.
- Your own count for the call-site grep, and the per-family breakdown.
- Verbatim `path:line` source for every mechanism, capture type, and escape
  hatch you name.
- The `Cargo.lock` evidence for whether `syn`/`quote` are already present.
- The existing-test survey, quoted.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config, and do NOT modify
  any file in the repository.
- Touch nothing outside this repo except `/tmp`; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): argue required semantics
  from rum's own documented behavior and internal consistency — never from
  another engine's observed output, and put no such output in your report.
- No new `unsafe` in any drafted plan for `rum-core`/`rum-runtime`.
- Add no dependency and edit no manifest in this brief; the derive-host
  question is an argument to be settled by the director.
- Bounded probes: `scripts/run-bounded.py --timeout SECONDS -- <command>`;
  state the bound and where you derived it.

## Report format (your final message)

1. "Design only, no source change."
2. Verification — each probe and grep, with verbatim output.
3. The re-probe verdict — does each path still reproduce, and how wide is the
   exposure.
4. The plan — items 3 through 12 above.
5. The drafted follow-up briefs.
6. Open questions with your recommendations, and any stop-loss.
