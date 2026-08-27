# brief-371 — runtime continuation ownership, outside matcher and rewrite

## Goal

Step 2 of plan-369. In the runtime partition **outside** matcher and rewrite,
convert continuation state so a future rooted frame can hold it without a deep
copy: read-only sequences become `SharedArgs`/`Arc` slices plus a cursor, and
genuinely mutable work becomes purpose-specific immutable or persistent state.

**The suspension mechanism does not change here.** The closure queue stays the
only one, `suspend_with_result_frame_lazy` stays the construction path, and no
frame type, `FrameAction`, or Session slot is introduced. Those are step 374.
This brief only changes what the existing lazy factories are handed.

## The partition, drawn so 371 and 372 are exhaustive and disjoint

Measured at `55290016`: 79 runtime files hold suspension sites.

**Out of scope here — step 372 owns these 29 files:**
`pattern_matching/**`, `kernel_data/rewrite_data/**`,
`definitions/application/**`, `definitions/rhs_guards.rs`, and
`kernel_data/structure_data/replace/**`.

**In scope — the remaining 50 files**, whose largest groups are
`kernel_data/control_data` (8), `debug_tools` (5+2), `definitions/*.rs` outside
`application` and `rhs_guards` (4), `definitions/definition_catalog` (3),
`scoping` and `scoping/lexical` (5), `tree_walkers` (2), `options` (2),
`messages` (2), `eval_core` (2), the branching-loop iteration files (2), plus
single-file entries including `eval_call_prepare.rs`, `package_tools.rs`,
`hold_thread.rs`, `attributes.rs`, `resolution.rs`, `builtins_input.rs`,
`builtin_adapters.rs`, and one each under `session`, `eval_dispatch`,
`kernel_data`, and `debug_tools/resources`.

Derive the exact list yourself and report it. **If a file is genuinely
ambiguous between the two partitions, say so rather than guessing** — a site
missed by both briefs is the failure this split has to avoid.

## What "ownership conversion" means concretely

plan-369 established that borrowing alone cannot serve every downstream
function, so the target is state a frame can hold immutably and hand onward
cheaply:

- **Read-only sequences** — `SharedArgs` or `Arc<[T]>` plus a scalar cursor,
  instead of a consumed `Vec` or a consuming iterator.
- **Append-only accumulators** — persistent chunk chains, so appending is
  O(1) and the single linear traversal happens once at the end.
- **Maps and similar** — an immutable base plus delta or tombstone overlay.
- **Compound cases** — a family-specific immutable state struct, defined
  beside its owner.

Ordinary (non-suspending) code keeps its owned or borrowed fast form. **The
freeze into shared/persistent form happens only inside the lazy
`InputRequired` factory** — the same discipline brief-370 established, where
the ordinary path stopped paying 63.8 KiB for a capture it never used.

## Scope

MAY modify: the in-scope runtime files above, family-local state modules they
need, doc-hidden internal helpers, and focused runtime tests.

MUST NOT touch:

- the 29 step-372 files listed above;
- `crates/rum-stdlib/**` — step 373;
- `crates/rum-runtime/src/runtime/suspension/**` — frame storage, the queue,
  and the trait are step 374's;
- `model/session.rs` pending/resuming fields, `scoping/gensym_gc.rs`,
  `debug_tools/resources/memory.rs`;
- `crates/rum-core/src/expr.rs` representation and public serde shapes;
- `crates/rum-derive/**`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`,
  `.git`, `Cargo.toml`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **No `Owned(Vec<_>)` escape variant** in any state type you introduce. A
   state enum with an owned-bulk arm is the deep copy this line exists to
   remove, wearing a named-type disguise.
2. **Do not introduce `FrameAction` here, even partially.** plan-369 puts the
   action protocol in step 374; a half-adopted version is precisely the second
   suspension mechanism the whole line forbids.
3. **Rejected ownership answers stay rejected**: `Arc::make_mut` on a shared
   handle, `Mutex`, `RefCell::take`, deferred sweep, pinning, snapshot roots.
   Each was assessed in plan-369 §3 with a concrete reason it fails.
4. **No central carrier module.** Keep family-specific state beside its owner;
   a shared module that grows past the 35k line would trade one debt for
   another. Report the size of any file you grow.
5. **The ordinary path must not pay.** Freezing, `Arc` bumps, cursor
   allocation — all only in the `InputRequired` arm. Prove it with counters,
   not with a reading of the code.
6. **This changes no language behavior.** If a workspace test's result changes,
   investigate rather than adjusting the test.
7. **Stop-loss is real.** If a family resists conversion twice, STOP and report
   the shape and the error, leaving the tree as it stands.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime --offline
cargo test --workspace --offline
```

Baseline is `55290016`: **2150 passed, 0 failed**. Report the new total and
prove no existing test changed result with a sorted per-test-name diff and
`removed_or_changed=0`.

**Counters first, timings second.**

- For each converted family, a **numeric suspend/resume liveness probe**: the
  exact resumed value, proving the converted path actually ran. Not an absence
  of error.
- **Width-slope probes in the shape of brief-370's**: for at least the
  control-flow and tree-walker families, show that ordinary-path
  `bytes_total` no longer carries a per-element slope from a suspension
  capture. Paste narrow/wide pairs and the differential, with the liveness
  value for each.
- Repr counters (`shared_args_alloc`, `shared_expr_alloc`, `into_vec_clone`)
  for those probes. `into_vec_clone` is now the detector for accidental COW or
  unwrap fallback — any nonzero value needs an explanation.
- The in-scope file list you derived, and a scan showing no unapproved raw
  owned-bulk capture remains in this partition.

Then the microbench A/B, profiling env OFF, three runs per arm:

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief371-base
bash scripts/core-language-microbench.sh --output-dir /tmp/brief371-new
```

Affected labels at minimum: `do-count-null`, `count-wide-no-match`, and the
control-flow and iteration labels your conversions touch — name them.
Named controls: `sort-default-wide`, `sortby-default-wide`,
`keysort-default-wide`, `sort-symbol-default-width4000`,
`sort-symbol-default-width16000`.

Report the per-label spread across the three runs and judge against it. Never
normalize against the suite median; if the controls move as much as the
affected labels, that is the noise floor.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo except `/tmp`; no network; `cargo --offline`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own behavior only.
- **No new `unsafe`**; `rum-core` and `rum-runtime` are zero-`unsafe`.
- No new dependency.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; name
  the prior same-shape duration each timeout is derived from. Persist raw
  microbench output before parsing.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. The in-scope file list you derived, and any file you found ambiguous
   between partitions.
2. What changed — per family, with the state types introduced.
3. Liveness probes — exact resumed values per family.
4. Width-slope probes — narrow/wide/differential, plus repr counters.
5. Correctness — commands, totals, `removed_or_changed=`.
6. The A/B — affected rows, controls, three-run spread, verdict.
7. File sizes for anything you grew.
8. Stop-loss / open questions (empty if none).
