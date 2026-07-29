# Brief 298 — implement `lst[[i]] = v` (a missing language feature, not an optimization)

Step 1 of `docs/dev/codex/plan-297-part-assignment.md`. **This is a
correctness/feature brief. Do not optimize anything.**

## Goal

Positional assignment into a List does not work in rum today:

```
ee = {1, 2, 3}; ee[[2]] = 99   ->  ee is still {1,2,3}
                                   ! Set::wrsym: Symbol Part is Protected.
```

`Set` reaches `eval_set` (`definitions/definitions.rs:14`), evaluates the rhs,
finds no matching lvalue handler (only Association has one), falls into
definition-lhs normalization, rebuilds the whole lhs, classifies it as a
DownValue for the protected `System\`Part`, and rejects it. The user sees a
false "Part is Protected" diagnostic and a silently unchanged list.

What makes this a real gap rather than a design choice: **the compound form
already works.** `dd[[2]] += 10` yields `{1, 12, 3}`, and
`AppendTo[cc[[1]], 9]` works and was even optimized in brief-295. Association
Part assignment works (`A[[Key[1]]] = 99`). Only the most basic form is
missing.

Make `lst[[i]] = v` work, with correct semantics. Performance is explicitly
NOT a goal here: use the existing generic rebuild, accept O(m) per assignment,
and leave the fast path to a later brief. An honest working baseline is the
deliverable.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines).
- `docs/dev/codex/plan-297-part-assignment.md` §2 (execution path), §5
  (observability boundary and proposed contract), §6 Step 1. Follow **this**
  brief where they differ.
- Reuse `exact_position_paths_from_part_target_selector_exprs`
  (`kernel_data/structure_data/mod.rs:35`) and `replace_exact_path`. Do not
  change either.

## The semantic contract

Follow the ordering already tested for Association Set
(`crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs:147-175`):

1. evaluate the rhs first;
2. evaluate positional selectors left-to-right, suspension-safely;
3. **re-fetch the owner's current value after all callbacks**;
4. **re-check owner protection and the exact List path** after all callbacks;
5. perform the structural replacement with **no evaluator call in the mutation
   window**;
6. bump the owner epoch and return the evaluated rhs.

**The single most important rule:** the rhs and the selectors can run
arbitrary user code before the mutation, including reassigning, clearing,
protecting, or removing the target. Therefore **no pointer, path, protection
verdict, or borrowed definition slot obtained before selector evaluation may
be reused afterward.** Everything must be re-derived after the last callback.

Note a real difference from `AppendTo`: positional replacement does not
construct and evaluate a new call head, so no List downvalue/upvalue runs
after the store. `AppendTo`'s "no interfering List definitions" condition is
therefore not semantically required here — but keep the accepted shape narrow
anyway (see Scope).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/definitions.rs` — **routing
    call only**; it is 28,979 chars, already in split-plan range, so no policy
    goes here
  - `crates/rum-runtime/src/runtime/definitions/mod.rs` — module declaration
  - `crates/rum-runtime/src/runtime/tests/mod.rs` — test module declaration
  - `crates/rum-runtime/src/runtime/builtin_catalog/rules.rs` — `Set`'s
    summary, if you change what `Set` documents
  - `docs/reference/builtins.md` — **regenerate only**, never hand-edit
  - `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs` and
    `selectors.rs` — only to expose an existing diagnostic helper, no logic
    change
- Files you MAY create:
  - `crates/rum-runtime/src/runtime/definitions/part_assignment.rs`
  - `crates/rum-runtime/src/runtime/tests/part_assignment.rs`
- Files you MUST NOT touch: everything else. Specifically **do not** touch
  `incremental_append.rs`, `association_data.rs`, `association_lvalue.rs`,
  `evaluation_cache/**`, `replace_exact_path`, `scripts/**`, `Cargo.*`.

Accepted shape for this brief: a plain symbol owner, one exact path of one or
more **positive machine-sized integer** selectors, through a plain `List` at
every level. Everything else — bulk/multi/duplicate selectors, negative or
zero indices, `Key[...]`, non-`List` containers, non-symbol owners — either
keeps its current behavior or produces a proper diagnostic (see below), but
**must never again fall through to defining protected `System\`Part`**.

## Deliverable

1. `lst[[i]] = v` works, including nested paths (`m[[1, 2]] = v`).
2. **The false diagnostic is gone.** A recognized-but-invalid positional
   target must produce a sensible Part/Set message, not
   `Set::wrsym: Symbol Part is Protected`.
3. If you change what `Set` documents, update `rules.rs` and **regenerate**
   `docs/reference/builtins.md` with
   `cargo run -p rum-stdlib --bin generate-builtin-reference`. The drift test
   (`rum-stdlib/src/tests/builtin_reference_docgen.rs`) enforces consistency —
   never hand-edit the generated file.
4. No performance work. The generic rebuild stays; O(m) per assignment is the
   expected, honest baseline for a later brief.

## Verification (MANDATORY — you run it; iterate to green)

### Correctness is the gate here, not timing

Add a focused `runtime/tests/part_assignment.rs` covering at minimum:

**Basic behavior**
- `ee = {1,2,3}; ee[[2]] = 99` gives `{1,99,3}` and returns `99`;
- nested: `ff = {{1,2},{3,4}}; ff[[1,2]] = 9` gives `{{1,9},{3,4}}`;
- **value semantics**: `x = {1,2,3}; y = x; y[[1]] = 9` leaves `x` unchanged.

**The callback hazard — this is where it breaks**
- an rhs that **reassigns** the target: `g = {1,2,3}; g[[2]] = (g = {7,8}; 5)`
  — state the resulting behavior and assert it deliberately;
- a **selector** that reassigns, clears, or `Protect`s the target mid-flight;
- a selector that suspends (request/resume), then resumes — the write must
  still re-derive everything after resume;
- assert explicitly that protection is re-checked **after** callbacks: if the
  target becomes `Protected` during rhs/selector evaluation, the write must
  refuse.

**Diagnostics** — each must give a sensible message, and none may mention
`Part is Protected`:
- index `0`, negative index, out-of-range index, non-integer selector;
- a non-`List` container at some level;
- `Part` applied to a non-symbol owner.

**Nothing else may change**
- Association Key lvalues (`A[[Key[1]]] = 99`, `B[1] = 99`) unchanged;
- compound Part updates (`dd[[2]] += 10`, `AppendTo[cc[[1]], 9]`) unchanged;
- **list destructuring** (`{a, b} = {1, 2}`) unchanged;
- ordinary definition lhs (`f[x_] := ...`, `f[1] = 2`) unchanged — the new
  route must not swallow anything the normalization path used to handle.

### Standard gates

- `cargo fmt --check`.
- `cargo test --workspace` — report totals before/after and tests added. The
  nine `rum-jupyter` ZeroMQ tests fail under the dispatch sandbox (loopback
  binding forbidden); report as the known limitation.
- The generated-reference drift test must pass.
- **Record a counter baseline** for the now-working fill:
  `qL = Table[0,{200}]; Do[qL[[i]] = i, {i,1,200}]` under `RUM_PROFILE_REPR=1`,
  plus its wall time. This is the honest starting point for the later fast
  path — note that all previously recorded numbers for this workload measured
  a *failing* path and are void.
- Microbench A/B is a **regression check only** (nothing should get faster).
  Judge by within-run quotients or against specific untouched labels; **never
  normalize against the suite median** (see `cc86cdd`, `a904935`).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.**
- **Do not optimize.** No `make_mut`, no cache-proof reuse, no fast path. If
  you find yourself editing `evaluation_cache/` or `incremental_append.rs`,
  you have left this brief.
- Never hand-edit `docs/reference/builtins.md`; regenerate it.
- If the accepted shape turns out to need widening to pass a semantic test,
  STOP and report rather than widening on your own initiative.
- Touch nothing outside this repo except `/tmp`. No network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.

## Report format (your final message)

1. `ee[[2]] = 99` before/after, and confirmation the false
   `Part is Protected` diagnostic is gone.
2. The routing point and where the new policy lives.
3. The callback-hazard tests: what each asserts and why that is the right
   contract. Call out the re-check-after-callbacks tests explicitly.
4. Diagnostics table for each invalid shape.
5. Confirmation nothing else changed (Association, compound Part,
   destructuring, definition lhs).
6. The counter + wall-time baseline for the working n=200 fill.
7. Whether `Set`'s documented summary changed, and that the reference was
   regenerated (not hand-edited) with the drift test passing.
8. Test totals, tests added, `cargo fmt --check`.
9. Confirmation that `git status` shows only Scope files.
