# brief-375 — clear the cutover's tidy-up debt

## Goal

Four items left by brief-374's cutover (`7dcc1cce`). The defect it closed
stays closed; this brief changes no suspension semantics, adds no frame, and
removes none.

They are ordered by how much damage they do, not by size.

## 1. Source positions are burned into generated frame type names

The macro names generated frames after their source location:
`ResumeKeySortByFromIndexSuspensionFrameL150C25`,
`EvalUpdateAttributesSuspensionFrameL85C13`,
`BuiltinEvaluateSuspensionFrameL935C62`.

**483 of the 893 entries in the derived inventory carry one**
(`crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`).

So inserting a single line above `attributes.rs:85` renames that frame and
turns the inventory test red, with nothing about the capture graph having
changed. That test exists to catch a type that lost its traversal; a gate that
also fires on every unrelated edit is one people learn to regenerate
reflexively, and then it stops being a gate at all. **This is the most
damaging item here even though it produces no warning.**

Give frames position-independent names. A name derived from the enclosing
function plus a disambiguating index within that function is the obvious
shape; an explicit name argument to the macro is another. Pick one, say why,
and state what happens when two frames in one function would collide.

**Acceptance is mechanical**: insert a blank line at the top of a file that
contains a generated frame, rebuild, and show the inventory test still passes.
Paste that. Then remove the blank line.

## 2. 47 new build warnings against a zero-warning baseline

`4c2adfea` builds with **zero** warnings; `7dcc1cce` builds with 47. Three
kinds:

- **22 × "type `X` is more private than the item ...`::state`"** — a generated
  frame field is more visible than the state type it holds, e.g.
  `SuspendedKeySortByState` at `associations/resumable/key_sort.rs:340`.
- **12 × "variable `state__index` should have a snake case name"** — the
  double underscore in generated bindings.
- **12 × "structure field ... should have a snake case name"** — the same in
  generated fields.

Fix these in the macro's generation, **not with `#[allow]` attributes**. An
allow silences the report without changing what is generated, and the naming
is generated code we control. If a specific case genuinely needs an allow, it
must be narrowly scoped and justified in the report.

Item 1's renaming may resolve some of these; do 1 first and then see what
remains.

## 3. Three production files crossed the 35k line

brief-374 asked for responsibility-based `resume_frames.rs` siblings and got
sixty of them, but three files still ended over:

| Bytes | File |
|---:|---|
| 37,850 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs` |
| 37,611 | `crates/rum-runtime/src/runtime/pattern_matching/core.rs` |
| 36,593 | `crates/rum-stdlib/src/strings/patterns/search/rules.rs` |

Split them the same way the other sixty were split — by responsibility, into a
sibling beside the code that owns it. Not by cutting at a line count.

Out of scope, and deliberately: `derived_inventory.rs` (101,044) is generated
inventory data rather than logic, `tests/context_and_loader.rs` (37,489) and
`tests/scoping_gensym_gc.rs` (35,452) are test files and the latter was
already over at baseline. Leave all three alone.

## 4. The matcher regression cannot see its own defect

`matcher_guard_binding_survives_a_sweep_while_suspended`
(`crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/suspension_frames.rs`)
passes the candidate directly as `gcMThunk[gcM$1]`, which leaves the call
expression on the active stack. That is a second root, so removing
`trace_frame_roots` from the sweep leaves this test green while the other four
turn red. It cannot see the defect it was written for.

plan-364's probe got this right by clearing the external root from inside the
thunk, so the binding table in the suspended frame was the only path left:

```text
Module[{gcM}, gcM = 703; gcMRoot = Hold[gcM]; Hold[gcM]]
gcMFn[z_] := First[z] /; SameQ[InputString["M: "], "yes"]
gcMThunk := With[{v = gcMRoot}, Clear[gcMRoot, gcMThunk]; v]
gcMFn[gcMThunk]
```

**A warning from the director's own failed attempt**: porting that shape
directly into the existing test made resumption fail with a second
`InputRequired` — the guard suspended again. The probe drives the session with
`eval_request` + `resume_eval_request_with_input`, while the regression mixes
`eval_request` with `resume_suspended_evaluation`. Work out which pairing the
regression needs rather than assuming the substitution is mechanical.

**Acceptance is the negative calibration, and it is the whole point**: with
the test strengthened, temporarily remove `trace_frame_roots` from
`collect_and_sweep_dead_gensyms` and show **five** regressions red instead of
four, then restore it and show all six green. Paste both runs.

## Scope

MAY modify: `crates/rum-derive/**` (items 1 and 2); the generated-name
consumers that follow from it, including `derived_inventory.rs`; the three
files in item 3 and their new siblings; and the one test in item 4.

MUST NOT touch:

- suspension semantics — no frame added or removed, no change to rooting, the
  finalizer, the queue, `gensym_gc.rs`, or `memory.rs` beyond what a rename
  mechanically forces;
- `crates/rum-core/src/expr.rs` representation and public serde shapes;
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.toml`,
  `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **No `#[allow]` as the fix for item 2** unless a case is argued
   individually.
2. **A rename must not weaken the inventory gate.** It must still fail when a
   type loses its traversal — prove that too, by removing one derive and
   showing the test red.
3. **Splitting is by responsibility**, not by byte count. A sibling that is
   just "the second half of the file" is worse than the file.
4. **No behavior change.** Workspace results must be identical.
5. **Stop-loss is real.** If the renaming scheme collides irreducibly, STOP
   and report the shape.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo build --workspace --offline
cargo test --workspace --offline
```

Baseline is `7dcc1cce`: **2163 passed, 0 failed**, and 47 warnings against
`4c2adfea`'s zero. Report the new total with `removed_or_changed=0`, and
**the warning count, which must be zero**.

Paste, verbatim:

- the blank-line insertion test from item 1, before and after;
- the removed-derive test from pitfall 2, showing the gate still bites;
- the five-red / six-green negative calibration from item 4;
- the final sizes of the three split files and their siblings.

No microbench gate: this brief changes names, visibility, module boundaries
and one test. Say that explicitly rather than running one, and if you believe
you touched an evaluation path, STOP and say so instead.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo except `/tmp`; no network; `cargo --offline`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`**; no new dependency.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; name
  the prior same-shape duration each timeout derives from.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. The naming scheme chosen, why, and the collision rule.
2. The blank-line proof, and the removed-derive proof.
3. Warning count before and after, by category, and any allow you kept.
4. The three splits, by responsibility, with final sizes.
5. The matcher regression — what changed, which session pairing it needed, and
   the five-red / six-green calibration.
6. Correctness — commands, totals, `removed_or_changed=`.
7. Stop-loss / open questions.
