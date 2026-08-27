# brief-374 — the atomic cutover, and the end of the line

## Goal

The last step of plan-364 and plan-369, and the one that actually closes the
defect. Convert every suspension boundary into a named derived frame, keep
both the pending and the currently-resuming frames rooted in `Session`, feed
one traversal into the gensym root walk and the memory proxy, land the pure
finalizer protocol, convert the deferred sort family, and delete every
closure-taking suspension API.

When this lands: a gensym reachable only from a suspension frame is no longer
reclaimable, and a future frame that forgets its traversal fails to compile.

## Read this first — it has already cost two stop-losses

plan-369's pure finalizer makes `Arc::try_unwrap` succeed by running **after**
the rooted frame and its dispatch `Arc` are dropped. **That frame lifetime is
built here, in this brief, and nowhere earlier.** Steps 371-373 could only use
the freeze-and-read shape, and two dispatches stopped by reaching for this
mechanism before it existed:

- **brief-368** satisfied rooting with `self.as_ref().clone()`, which failed
  for non-`Clone` state and deep-copied roughly 230 `Vec<Expr>` captures.
- **brief-373**'s first attempt built carriers whose completion assumed
  uniqueness while a rooted frame still held an alias.

**Why brief-368's failure cannot repeat now**: steps 370-373 converted the
captured state itself. Bulk state is `SharedArgs`, `Arc` slices, frozen
snapshots and persistent carriers, so a rooted frame lending `&self` no longer
implies a deep copy — the clone that broke 368 is now a refcount bump, and the
completion paths that need ownership are exactly what the finalizer serves.

## Where the line stands

Landed and not to be redone: `d13aa27f` (derive substrate), `a02064b8` (155
types derive it), `8e97ba41` (11 opaque continuations removed), `0425f8a3`
(`BuiltinFn` borrows `&SharedArgs`), `c0510048` (runtime non-matcher
ownership), `55edea50` (matcher/rewrite ownership), `751d50de` (stdlib
ownership, sort deferred).

**Current boundaries — verify and report your own counts:**

| API | Calls |
|---|---:|
| `push_suspension_frame` | 166 |
| `drive_internal_value` | 46 |
| `suspend_with_result_frame_lazy` | 32 |
| `drive_evaluated_value` | 6 |
| `suspend_with_frame` | 0 |
| `suspend_with_result_frame` | 0 |

250 real call sites, down from 399, because steps 371-373 collapsed many into
named state plus a single push. The two zero-call APIs can simply go.

plan-364's estimate of 367 named shapes predates all of this. **It is not an
acceptance criterion** — report the number you reach, as steps 2 and 372 did.

## What to build

### 1. Named frames, and the only way to insert one

Every boundary becomes a named `struct` implementing `SuspensionFrame`, state
in `#[derive(FrameRootTrace)]` fields. Share a type between two sites only
when the captured signature and the normalized resume body are identical.

Frames live beside the resume behavior that owns them. **Files crossing 35k
get a responsibility-based sibling `resume_frames.rs`; there is no central
frame file.** The tightest files today are
`pattern_matching/sequence.rs`, `pattern_matching/core.rs`,
`rewrite_data/replace/once.rs`, and `structure_ops/ordering_sets.rs` — plan
their siblings rather than discovering the overflow. Report every grown file's
final size.

### 2. Session-rooted frames, including during resume

```rust
pending_suspension: Option<SuspendedEvaluation>,
resuming_suspension: Option<SuspendedEvaluation>,

frames: VecDeque<Arc<dyn SuspensionFrame + Send + Sync>>,
```

`SuspensionFrame::resume` takes `self: Arc<Self>` or `&self` and must not
consume the rooted object. Sequence:

1. Move `pending_suspension` into `resuming_suspension`.
2. Move only `active_stack` into `evaluation_stack`; frames stay in `Session`.
3. Clone the front `Arc` and invoke it while the original stays at the front.
4. Pop the original only after the frame returns.
5. On nested input, pop the completed frame, take the new
   `pending_suspension`, and append the still-rooted suffix from
   `resuming_suspension`, preserving today's ordering
   (`suspension/core.rs:250` is the current append).
6. On success or terminal error, clear `resuming_suspension` explicitly.

**`clear_suspension_state` must be split.** Queued input and best-effort
monitor callbacks may clear a fresh pending suspension while an outer
resumption is still active; they must not erase `resuming_suspension`.

### 3. The pure finalizer, under its four constraints

Where a completion needs ownership, the frame returns a **named** finalizer.
The driver removes and drops the completed frame and its dispatch `Arc`
**first**, then invokes it. The finalizer:

1. is a named, closed internal type;
2. takes no `Session` argument;
3. cannot evaluate or suspend;
4. runs only after the rooted frame is dropped.

Those four are what make it not a second suspension mechanism. **Enforce them
in the type system where possible and in the repository invariant test
otherwise** — not in prose. With them, `Arc::try_unwrap` on uniquely held
persistent state succeeds and there is one final linear traversal, never an
O(N) clone per continuation step.

### 4. The deferred sort family

`Sort`, `SortBy`, `ReverseSort`, `KeySort`, `KeySortBy` were deferred from
step 373 because a suspension-only merge representation changed observable
comparator progression and output ordering. `MergeSortState<T, C>` already
derives `FrameRootTrace` and `C: MergeCompletion<T>` is already a named trait
(brief-367); what remains is that `resume_merge_sort` takes `mut state` by
value.

**The requirement is a single representation**: the ordinary and the resumed
path must run the same merge state machine, so a user comparator cannot detect
which one it is in. That is what a suspension-only representation could not
give. Two merge representations have already failed — if a third fails, STOP.

### 5. One traversal, two consumers, and the deletion

- **GC** — `scoping/gensym_gc.rs:187` walks only `active_stack` today. Walk
  both `pending_suspension` and `resuming_suspension`; `visit_expr` seeds via
  `seed_live_gensyms_from_expr`, `visit_symbol` seeds the candidate directly.
- **Memory** — `debug_tools/resources/memory.rs:572` charges a flat 16 bytes
  per frame. Use the same traversal, plus
  `size_of::<SuspendedEvaluation>()`, active-stack entries as today,
  `frames.capacity() * size_of::<Arc<...>>()`, each Arc header and
  `size_of_val(frame)`. Count both slots.

`for_each_root_expr` (`gensym_gc.rs:112`) carries a comment requiring these
two to stay in sync; after this brief that holds by construction.

Then delete `ClosureFrame`, `suspend_with_result_frame`,
`suspend_with_result_frame_lazy`, `drive_evaluated_value`,
`drive_internal_value`, and closure-taking `push_suspension_frame`, and make
the deletion stick:

1. `SuspendedEvaluation::frames` becomes private to `suspension::core`.
2. `new`, queue mutation, pop, and append stay private.
3. `SuspensionFrame: FrameRootTrace + Send + Sync + 'static` — the supertrait
   makes a missing traversal a compile error.
4. One doc-hidden generic `Session` insertion method taking
   `F: SuspensionFrame`, which creates the `Arc`.
5. A repository invariant test bans the old API names, bans direct frame-queue
   mutation outside `suspension/core.rs`, bans handwritten `FrameRootTrace`
   impls outside the sorted allowlist, and enforces the finalizer constraints.

The allowlist holds `RandomState` and `SyntaxError`. It grows only under
brief-366's two-part test, argued per entry.

## The four defects, and how tests must prove them closed

All four reproduce at the line's start; probes and exact values are in
`docs/dev/codex/plan-364-frame-roots.md` under "Re-probe verdict":

1. **Path A** — a second request during a suspension: `gcA$1 -> 701`, then
   `gcA$1 -> gcA$1`.
2. **Path B** — a resumption crossing the watermark itself.
3. **Matcher bindings** — a frame holding `SymbolExprMap` loses them.
4. **Bare `Symbol` in a `Block` frame** — candidate tracking lost, and a
   restored definition leaks on the *next* sweep. Note this one fails in the
   opposite direction.

**Every liveness regression must force a real sweep between the suspension and
the assertion.** In the guard line, six green gates and a whole-line
regression test all missed a real defect because every one asserted *before*
the invalidating action. The shape:

1. Assert the candidate's value before suspension.
2. Suspend.
3. Cross the watermark through a real `Module` exit.
4. Assert `gensym_gc_sweep_count` increased — prove the sweep ran.
5. Resume.
6. Assert the **exact** resumed value, never an absence of error.

Use the subprocess helper in
`crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs:890` with
`RUM_GENSYM_GC_THRESHOLD=1`.

Also required:

- **Current-frame test** — the executing frame and a later queued frame each
  hold a different candidate; a sweep during the current frame's resume must
  preserve both. This is what proves section 2 step 3.
- **Nested suspension ordering** — A → nested B → old suffix C, asserting
  exact side-effect order and values.
- **Memory** — a large frame state with a limit between the old flat-16
  estimate and the corrected proxy; assert classified `MemoryLimit`
  termination and session recovery afterwards.
- **Comparator progression** — the sort family's call sequence is identical
  ordinary versus resumed.

## Scope

MAY modify: the files holding the 250 boundaries; the suspension module;
`model/session.rs`, its constructor, helpers and reset paths;
`scoping/gensym_gc.rs`; `debug_tools/resources/memory.rs`; `kernel_api.rs`;
new `resume_frames.rs` siblings; the sort family; and focused runtime, stdlib
and host tests.

MUST NOT touch: `crates/rum-core/src/expr.rs` representation and public serde
shapes; `crates/rum-derive/**` (if the macro cannot express a frame, STOP and
report); `TODO.md`, `PROGRESS.md`, `CHANGELOG.md`, `AGENTS.md`, `docs/**` —
recording the closed defect is the director's step; `.git`, `Cargo.toml`,
`Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **No deferred sweep, no pinning, no snapshot roots.** Rejected for leaving
   the root cause in place. Frames get rooted; the sweep runs normally.
2. **Frame construction only in the suspending arm.** The non-suspending arm
   calls the continuation directly and must keep doing so — no frame, no
   `Arc`, no allocation when nothing suspends. Zero by construction, not
   measured away afterwards.
3. **No runtime closure factories.** Replacing a deleted API with something
   that builds a boxed closure reintroduces what this line removes.
4. **No deep clones**, and no `Arc::make_mut` on a handle a frame also holds.
5. **`Expr` is visited whole**; the recursive symbol descent stays at
   `gensym_gc.rs:236`.
6. **Preserve suspend/resume semantics exactly** — ordering, one-shot resume,
   and the per-lane contracts in `docs/guide/patterns-and-rules.md`.
7. **No new `unsafe`.** If rooting the executing frame seems to need it, STOP
   — the `Arc` design exists to avoid that.
8. **Stop-loss is real, and this brief has three of them**: a third merge
   representation, a mechanism failing twice, or a frame shape the derive
   cannot express. Stop and report with the tree as it stands.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime --offline
cargo test -p rum-stdlib --offline
cargo test -p rum-host --offline
cargo test --workspace --offline
```

Baseline is `4c2adfea`: **2155 passed, 0 failed**. Report the new total and
prove no existing test changed result with a sorted per-test-name diff and
`removed_or_changed=0`. The derived-inventory test must stay green.

For each of the four defects, paste the **before/after value pair**: the old
behavior from plan-364, and your regression's assertion now. Closed means the
exact value is right, not that a test is green.

**Counters first, timings second**, including an ordinary-path allocation
comparison against `4c2adfea` in the shape that settled steps 372 and 373.

Then the A/B, profiling env OFF, three runs per arm:

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief374-base
bash scripts/core-language-microbench.sh --output-dir /tmp/brief374-new
```

Affected: the matcher, replace, cases, position, guarded and callback-sort
families. Controls: pick labels this change cannot reach and name why.
Report per-label spread across the three runs and judge against it. Never
normalize against the suite median. **If the controls move as much as the
affected labels, the verdict is "no regression", not a speedup** — that is how
372 and 373 read, in opposite directions.

Finally: the file-size table, and the enforcement test's output.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo except `/tmp`; no network; `cargo --offline`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`**; `rum-core` and `rum-runtime` are zero-`unsafe`.
- No new dependency.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; name
  the prior same-shape duration each timeout derives from. Persist raw
  microbench output before parsing.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — frame count reached versus plan-364's 367, and per area.
2. The rooting design as built — two slots, Arc lifetime through a resume, the
   nested append, the `clear_suspension_state` split.
3. The finalizer — how each of its four constraints is enforced.
4. The sort family — the single representation, and the comparator-progression
   evidence.
5. Enforcement — what is private, what the supertrait forces, what the
   invariant test bans, any allowlist entry with both arguments.
6. The four defects — before/after value pairs.
7. Correctness — commands, totals, `removed_or_changed=`.
8. Counters, then the A/B — affected rows, controls, three-run spread, verdict.
9. File sizes.
10. Stop-loss / open questions.
