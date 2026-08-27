# brief-368 — the atomic named-frame cutover

## Goal

Step 4 of plan-364, and the one that actually closes the defect. Convert every
closure-based suspension boundary into a named derived frame, keep both the
pending and the currently-resuming frames rooted in `Session`, feed one
traversal into both the gensym root walk and the memory proxy, and delete
every closure-taking suspension API.

When this lands, a gensym reachable only from a suspension frame is no longer
reclaimable, and a future frame that forgets its traversal fails to compile.

**This step is deliberately atomic.** Splitting the runtime and stdlib halves
into separately landed changes would require keeping a legacy opaque-frame
path alive between them — a stopgap, which the 2026-08-27 standard rules out.
It is not complete until runtime, stdlib, host, workspace, and the measured
gate all pass together.

## Where the line stands

Steps 1-3 are landed and are not to be redone:

- `d13aa27f` — `rum-derive` and the `FrameRootTrace` contract
  (`crates/rum-runtime/src/runtime/suspension/trace.rs`), with `Expr` handed
  to the visitor whole and bare `Symbol` a root in its own right.
- `a02064b8` — 155 types derive it; the closure was closed by the compiler.
  `kernel_api` re-exports the trait, the visitor, and a doc-hidden derive.
- `8e97ba41` — the 11 opaque continuation boundaries are gone; nothing that
  reaches a frame is a generic closure any more.

Current counts, measured at `8e97ba41` — **verify them yourself and report
what you get**:

- 399 call sites of `suspend_with_frame` / `suspend_with_result_frame` /
  `drive_evaluated_value` / `drive_internal_value` / `push_suspension_frame`,
  across 80 runtime and 43 stdlib files.
- Eight `F:` closure bounds remain, all in
  `crates/rum-runtime/src/runtime/suspension/core.rs` — the API itself, which
  this brief deletes.

plan-364 estimated 367 named shapes from 398 sites. **That estimate is not an
acceptance criterion** — step 2 came in 32 under its own estimate and said so.
Report the number you reach.

## The four defects this must close

All four reproduce today; probes and exact liveness evidence are in
`docs/dev/codex/plan-364-frame-roots.md` under "Re-probe verdict".

1. **Path A** — a second request during a suspension sweeps a frame-only
   gensym. `gcA$1 -> 701`, then `gcA$1 -> gcA$1`.
2. **Path B** — a resumption crosses the watermark itself. The remaining
   frames are not in `Session` at all during resume, because
   `resume_suspended_evaluation` moves the whole `SuspendedEvaluation` into a
   local.
3. **Matcher bindings** — a frame holding `SymbolExprMap` loses its bindings.
4. **Bare `Symbol` in a `Block` frame** — candidate tracking is lost and a
   restored definition leaks on the *next* sweep. Note this one fails in the
   opposite direction from the others.

## What to build

### 1. Named frames

Every boundary becomes a named `struct` implementing `SuspensionFrame`, with
its captured state as `#[derive(FrameRootTrace)]` fields. Share one type
between two sites only when the captured type signature and the normalized
resume body are identical — do not centralize unrelated stateless behavior to
reduce the type count.

Frame definitions live beside the resume behavior that owns them. **Files
crossing the 35k architecture-debt line get a responsibility-based sibling
`resume_frames.rs`; there is no central frame file.** The current large files
with the most boundaries are `pattern_matching/core.rs` (22k, 12),
`rewrite_data/replace/once.rs` (20.7k, 12), `pattern_matching/sequence.rs`
(21.8k, 10), `rewrite_data/cases/delete.rs` (21.9k, 9),
`stdlib structure_ops/ordering_sets.rs` (28.7k, 5), and
`stdlib associations/basic.rs` (24.4k, 5). Report the final size of every file
you grew, and confirm none exceeds 35k.

### 2. Session-rooted frames, including during resume

```rust
pending_suspension: Option<SuspendedEvaluation>,
resuming_suspension: Option<SuspendedEvaluation>,

frames: VecDeque<Arc<dyn SuspensionFrame + Send + Sync>>,
```

`SuspensionFrame::resume` takes `self: Arc<Self>` or `&self` — it must not
consume the rooted object. The resume sequence:

1. Move `pending_suspension` into `resuming_suspension`.
2. Move only `active_stack` into `evaluation_stack`; frames stay in `Session`.
3. Clone the front `Arc` and invoke it while the original stays at the front.
4. Pop the original only after the frame returns.
5. On nested input, pop the completed frame, take the new
   `pending_suspension`, and append the still-rooted suffix from
   `resuming_suspension`, preserving today's ordering exactly
   (`suspension/core.rs:250` is the current append).
6. On success or terminal error, clear `resuming_suspension` explicitly.

**`clear_suspension_state` must be split.** Queued input and best-effort
monitor callbacks may clear a fresh pending suspension while an outer
resumption is still active; they must not erase `resuming_suspension`.

### 3. One traversal, two consumers

- **GC** (`scoping/gensym_gc.rs:187` today walks only `active_stack`):
  `visit_expr` seeds via `seed_live_gensyms_from_expr`; `visit_symbol` seeds
  the candidate directly. Walk **both** `pending_suspension` and
  `resuming_suspension`.
- **Memory** (`debug_tools/resources/memory.rs:572` currently charges a flat
  16 bytes per frame): the same traversal feeds `expr_memory_proxy_bytes`,
  plus `size_of::<SuspendedEvaluation>()`, active-stack entries as today,
  `frames.capacity() * size_of::<Arc<...>>()`, each Arc header and
  `size_of_val(frame)`. Count both slots. Conservative double-counting of
  shared allocations is consistent with the existing proxy and is fine.

`for_each_root_expr` (`gensym_gc.rs:112`) carries a comment requiring these
two to stay in sync. After this brief that requirement is satisfied by
construction rather than by the comment.

### 4. Delete the closure APIs, and make the deletion stick

Delete `ClosureFrame`, `suspend_with_frame`, `suspend_with_result_frame`,
`drive_evaluated_value`, `drive_internal_value`, and the closure-taking
`push_suspension_frame`. Then:

1. `SuspendedEvaluation::frames` becomes private to `suspension::core` — not
   `pub(in crate::runtime)`.
2. `SuspendedEvaluation::new`, queue mutation, pop, and append stay private.
3. `SuspensionFrame: FrameRootTrace + Send + Sync + 'static` — the supertrait
   is what makes a missing traversal a compile error.
4. The only insertion route is one doc-hidden generic `Session` method taking
   `F: SuspensionFrame`, which creates the `Arc`.
5. A repository invariant test bans the old API names, bans direct frame-queue
   mutation outside `suspension/core.rs`, and bans handwritten
   `FrameRootTrace` implementations outside the sorted allowlist.

The allowlist currently holds `RandomState` and `SyntaxError` as its only
external leaves. It may grow only under the same two-part test as brief-366 —
a derive structurally impossible, and the type structurally incapable of
holding a root — with both halves argued per entry in your report.

## Testing (this is where the last line failed)

**Every liveness regression must force a real sweep between the suspension and
the assertion.** A test that suspends and immediately asserts proves nothing:
in the guard line, six green gates and a whole-line regression test all missed
a real defect because every one of them asserted *before* the invalidating
action and never after one.

The shape, for each of the four defects:

1. Assert the candidate's value before suspension.
2. Suspend.
3. Cross the watermark through a real `Module` exit.
4. Assert `gensym_gc_sweep_count` increased — prove the sweep actually ran.
5. Resume.
6. Assert the **exact** resumed value, never the absence of an error.

Use the existing subprocess helper in
`crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs` with
`RUM_GENSYM_GC_THRESHOLD=1` so the `OnceLock` threshold initializes in a fresh
process.

Required regressions:

- Path A, Path B, matcher bindings, and the bare-`Symbol` `Block` case, each
  asserting the exact value from plan-364's re-probe table.
- **Current-frame test**: the executing frame and a later queued frame each
  hold a different candidate; a sweep during the current frame's resume must
  preserve both. This is the one that proves point 2.3 above.
- **Nested suspension ordering**: A -> nested B -> old suffix C, asserting
  exact side-effect order and values.
- **Memory**: a large frame state with a limit between the old flat-16
  estimate and the corrected proxy; assert classified `MemoryLimit`
  termination and that the session recovers afterwards.
- Keep the existing `active_stack` test, renamed so its narrower scope is
  explicit.

## Scope

Files you MAY modify: the 123 files holding the 399 boundaries; the suspension
module; `model/session.rs`, its constructor, helpers, and reset paths;
`scoping/gensym_gc.rs`; `debug_tools/resources/memory.rs`; `kernel_api.rs`;
new `resume_frames.rs` siblings; and focused runtime, stdlib, and host tests.

Files/areas you MUST NOT touch:

- `crates/rum-core/src/expr.rs` `Expr`/`Symbol` representation, and any public
  serde shape;
- `crates/rum-derive/**` — the macro is finished; if it cannot express a
  frame's shape, STOP and report;
- `TODO.md`, `PROGRESS.md`, `CHANGELOG.md`, `AGENTS.md`, `docs/**` — recording
  the closed defect is the director's step, not yours;
- `.git`, `Cargo.toml`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **No deferred sweep, no pinning, no parallel root snapshot.** A variant
   that keeps candidates alive while frames exist was considered and rejected
   for leaving the root cause in place. The frames get rooted; the sweep keeps
   running normally.
2. **Frame construction only in the `InputRequired` arm.** The non-suspending
   arm calls the continuation directly today (`suspension/core.rs:67`) and
   must keep doing so — no frame, no `Arc`, no allocation when nothing
   suspends. This is a zero-by-construction requirement, not something to
   measure away afterwards.
3. **No runtime closure factories.** Replacing a deleted API with a function
   that builds a boxed closure at runtime reintroduces exactly what this line
   removes.
4. **No deep clones.** Bulk captured state is `SharedArgs`, `Arc<[T]>`, or
   another Arc-backed named type, so resuming does not deep-copy an expression
   tree or trigger an accidental O(N) container clone.
5. **`Expr` is still visited whole.** The recursive symbol descent stays in
   `gensym_gc.rs:236`; do not duplicate it in a frame traversal.
6. **Preserve suspend/resume semantics exactly** — ordering, one-shot resume,
   and the per-lane contracts in `docs/guide/patterns-and-rules.md`.
7. **No new `unsafe`.** If you believe rooting the executing frame requires
   it, STOP and report — the `Arc` design exists precisely to avoid that.
8. **Stop-loss is real.** This is a large change; if a mechanism defeats you
   twice, STOP and report with the working tree left as is. A clean stop
   costs one round, and three of this line's four dispatches so far stopped
   on my own brief errors rather than on Codex's work.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime --offline
cargo test -p rum-stdlib --offline
cargo test -p rum-host --offline
cargo test --workspace --offline
```

Baseline is `8e97ba41`: **2150 passed, 0 failed**. Report the new total, and
prove no existing test changed result with a sorted per-test-name diff and
`removed_or_changed=0`.

Then, for each of the four defects, paste the **before/after pair**: the
probe's value under the old behavior as recorded in plan-364, and your
regression test's assertion now. The defect is closed only when the exact
value is right, not when the test is merely green.

Then the measured A/B, profiling env OFF, three runs per arm:

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief368-base
bash scripts/core-language-microbench.sh --output-dir /tmp/brief368-new
```

Affected labels: `matcher-single-tiny`, `matcher-single-wide-small`,
`matcher-single-wide-large`, `replaceall-simple`, `replace-wide-no-match`,
`replacepart-wide-rule-list`, `replacepart-guarded-hit`,
`replacepart-guarded-decline-fallback`, `count-wide-no-match`,
`do-count-null`, and the callback sort family.

Named controls: `sort-default-wide`, `sortby-default-wide`,
`reversesort-default-wide`, `keysort-default-wide`, `keysortby-default-wide`,
`sort-symbol-default-width4000`, `sort-symbol-default-width16000`.

**Report the per-label spread across the three runs and judge against it.**
Never normalize against the suite median. If the controls move as much as the
affected labels, that is the noise floor — report no regression rather than
chasing it.

Finally: the file-size table for every file you grew, and the enforcement
test's output.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo except `/tmp`; no network; `cargo --offline`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own behavior only.
- **No new `unsafe`**; `rum-core` and `rum-runtime` are zero-`unsafe`.
- No new dependency.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; state
  the bound and where you derived it. Persist raw microbench output before
  parsing.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per area, with the frame count you reached versus 367.
2. The rooting design as built — the two slots, the Arc lifetime through a
   resume, the nested append, and the `clear_suspension_state` split.
3. Enforcement — what is private, what the supertrait forces, what the
   invariant test bans, and any allowlist entry with both arguments.
4. The four defects — before/after value pairs.
5. Correctness — commands, totals, `removed_or_changed=`.
6. The A/B — affected rows, control rows, three-run spread, verdict.
7. File sizes for everything you grew.
8. Stop-loss / open questions (empty if none).
