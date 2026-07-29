# Brief 291 — DESIGN-ONLY: plan the Association / AppendTo incremental-update fix

**This is a design-only brief. `Scope: NONE` for source. You write a plan and
brief drafts; you write no Rust.**

## Goal

Incremental container updates in rum are quadratic with a large constant.
Building an Association key-by-key (`a[k] = v` in a loop) and accumulating a
list (`AppendTo[lst, x]` in a loop) are both O(n^2), and the Association case
additionally pays a large per-update constant. Produce a **reviewable,
step-by-step implementation plan** that lowers this cost without changing
observable semantics, plus **drafts of the follow-up implementation briefs**.

Claude has already done a first diagnosis (below). **Treat it as a hypothesis
to verify, not as a specification.** Reproduce each piece of evidence
yourself. If you can falsify any of it, say so plainly and explain what the
real cause is — a well-argued refutation is a better outcome than a plan built
on a wrong premise. Two independent readings of the problem is the point of
this brief.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines) and
  `docs/dev/codex/TEMPLATE.md` (brief format — your drafts must follow it).
- The repo is post-publication and in maintenance. Prefer the smallest change
  that gets the win; a large representation change needs a strong argument.

### Claude's diagnosis so far (verify all of it)

**Finding A — the unique-ownership fast path exists but never fires.**
`SharedArgs::into_vec` (`crates/rum-core/src/expr.rs:579`) already tries
`Arc::try_unwrap` and takes the `Vec` with zero copies when it is the sole
owner, falling back to a clone otherwise; it records which branch ran via
`record_into_vec` (`crates/rum-core/src/profiling.rs:41`). Measured on a
500-iteration `AppendTo` loop with `RUM_PROFILE_REPR=1`:

```
into_vec_total=500 into_vec_clone=500 shared_args_alloc=1023
```

i.e. the fast path fired **0 times out of 500**.

**Finding B — the suspected cause is a `clone` of the current value.** Two
sites pull the current value out of the definition table by cloning it, which
should be enough to keep the `Arc` refcount above 1 for the rest of the
update:

- `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:302`
  (`current_symbol_held_update_value`) — serves `AppendTo`, `PrependTo` and
  the other compound assignments.
- `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:415`
  (`association_lvalue_current`) — serves `a[k] = v`.

Hypothesis: taking ownership out of the definition (and writing the updated
value back) instead of cloning would let `into_vec` hit its fast path, turning
`AppendTo` accumulation from O(n^2) into O(n).

**Finding C — Association pays a second, separate cost.**
`set_association_key_path` (`association_data.rs:527`) converts the whole
Association `Expr` into a `Vec<AssociationEntryForm>`, does a linear
`position()` scan, then re-renders the entries back into an `Expr` — roughly
3n clones per single-key update, independent of Finding B. A discriminating
probe (equal lookup volume in both arms, but 2x the round-trip volume in one)
came out at 2.88x and 2.96x for n=400 and n=600, i.e. **the round-trip
dominates and the linear key scan does not**. So a hash index for key lookup
is likely NOT the fix.

**Finding D — a tree/HAMT representation looks blocked by the builtin ABI.**
`SharedArgs` must hand out a contiguous `&[Expr]` (`impl AsRef<[Expr]>`,
`expr.rs:620`) because `BuiltinFn = fn(&mut Session, &Expr, &[Expr])`
(`crates/rum-runtime/src/runtime/metadata.rs:288`) is the ABI of every
builtin. A trie-backed vector cannot produce a contiguous slice. Claude reads
this as ruling out structural sharing; check whether that is really true.

**Finding E — nesting makes the outer level dominate.** With an inner list
appended 200 times inside an outer list of size m, min-of-3, monotonic clock:

| shape | time | outer overhead |
|---|---|---|
| flat, no outer | 1.281 ms | — |
| nested, m=10 | 1.583 ms | +0.30 ms |
| nested, m=100 | 2.056 ms | +0.78 ms |
| nested, m=1000 | 6.460 ms | +5.18 ms |

At m=1000 the outer rebuild costs 4x the inner accumulation. Whether the
Finding-B fix extends through the nested/`Part` path
(`current_part_held_update_value`, `definition_updates.rs:317`) is **open** —
Claude did not trace it.

**Known hazard.** Of the compound assignments, `ApplyTo` runs *user code*
inside its update window (recorded during brief-208). If the target symbol's
value is taken out of the definition table while user code runs, that code can
observe the symbol as unset. Any ownership-transfer design must state per
operator whether it is safe, and confirm independently which operators run
user code in the window.

## Scope

- Files you MAY create: **`docs/dev/codex/plan-291-assoc-append-ownership.md`**
  (your plan document — this one file, new).
- Files you MUST NOT modify: **every `.rs` file in the repo**, plus
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `CLAUDE.md`, `Cargo.*`, `.git`,
  anything under `scripts/`, and every other file under `docs/`. This brief
  changes no source and no test.
- You MAY run read-only commands (`cargo build`/`cargo test` for baseline
  facts, the release `rum-cli`, `python3`, the microbench scripts) and you MAY
  write scratch probe files under `/tmp`.

## Deliverable

A single plan document at `docs/dev/codex/plan-291-assoc-append-ownership.md`
containing, in this order:

1. **Verification table.** One row per finding A–E: `CONFIRMED` /
   `REFUTED` / `PARTIAL`, the command or `path:line` you used, and the actual
   output. State plainly where Claude was wrong.
2. **The critical unknown, answered.** After changing the clone to an
   ownership transfer, does the `Arc` refcount actually reach 1 at the
   `into_vec` call — or does some other holder (the evaluation cache, a
   suspension frame, the trace buffer, an argument slice still alive up the
   stack, the definitions map itself) keep it above 1? **Enumerate every
   live holder of the target value's `SharedArgs` along the `AppendTo` path,
   with `path:line`.** If the answer is "the refcount would still exceed 1",
   the whole plan changes and that is the single most valuable thing you can
   report. Do not guess: read the path.
3. **Per-operator safety table** for the ownership transfer: every compound
   assignment that reaches `current_symbol_held_update_value`, whether its
   update window can run user code, and therefore whether it may take
   ownership. Cite the dispatch site for each.
4. **The failure/restore story.** If the value is taken out and the update
   then fails (message, `EvalError`, resource limit, interrupt, panic in a
   nested evaluation), how does the symbol's value get restored? Show where
   in the code this must be enforced.
5. **A step-by-step plan.** Each step: what changes, which files/functions,
   what invariant it preserves, its gate (tests + which microbench labels),
   and expected win. Order the steps so each is independently gateable and
   revertible. Explicitly state which step delivers which of the two
   workloads (Association vs list accumulation).
6. **Microbench label proposals.** The current suite
   (`scripts/core-language-microbench.sh`) has **no label** for incremental
   container update, so today there is no A/B baseline for any of this. Give
   the exact label names, iteration counts, and the `rumMicroBench`/
   `rumMicroOnce` lines to add, chosen so the labels stay in the suite's
   existing runtime budget. Note: the suite now times with `AbsoluteTiming`
   (monotonic); do not propose `AbsoluteTime[]` differencing.
7. **Drafts of the follow-up implementation briefs**, one per step, each a
   complete brief in `TEMPLATE.md` shape — Goal, Context pointers, Scope
   (explicit MAY-modify list; enumerate the mechanical ripple, e.g. every call
   site of a changed signature), Deliverable, Verification, Hard rules, Report
   format. These drafts are raw material for Claude to rewrite, so make the
   Scope lists as complete as you can: a missing file in a MAY list is the
   single most common defect in this project's briefs.
8. **Risks and open questions** you could not settle, each with what evidence
   would settle it.

## Verification (MANDATORY — you run it; this is a plan, so the evidence is measurements and code reading, not tests)

Reproduce at minimum:

```
# Finding A — the fast-path hit rate. Build release first.
cargo build --release -p rum-cli
printf 'lst = {};\nDo[AppendTo[lst, i], {i, 1, 500}];\nPrint[RumProfileSnapshot[]];\n' > /tmp/p291a.rum
RUM_PROFILE_REPR=1 ./target/release/rum-cli --file /tmp/p291a.rum < /dev/null
```

```
# Findings C/E — cost shape. Use AbsoluteTiming (monotonic), min-of-3, and
# report raw rationals plus the decimal conversion. Vary n and m; show that
# time scales with n as claimed (a number that does NOT scale with n means the
# probe is measuring something else — this project has been burned by exactly
# that).
```

Persist raw probe output to files under `/tmp` before parsing it. Profiling
env vars must be OFF for any timing number (`RUM_PROFILE_REPR` included — it
is only for the hit-rate probe, never during timing).

Do NOT run the full `cargo test` suite as a gate for this brief; you change no
code. Run it only if you need a baseline fact, and say so.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- **Write no Rust.** Not even a "small illustrative patch" applied to the
  tree. Code excerpts inside the plan document are fine and encouraged.
- Do NOT modify any file outside the single plan document named in Scope. If
  you believe you must, STOP and write that in the report instead.
- Touch nothing outside this repo except `/tmp` scratch files. No network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no comparisons against other engines, no oracle harnesses.
- **No new `unsafe`** in any code you propose; `rum-core`/`rum-runtime` are
  zero-`unsafe`. If a proposal seems to need it, say so and propose the safe
  alternative alongside.
- Any proposal must not add unconditional cost to the default evaluation path;
  instrumentation goes behind the existing env-gated `RUM_PROFILE_*` pattern.
- Semantics must not change. rum's value semantics require that after
  `a = <|1 -> 2|>; b = a; b[3] = 4`, `a` is unchanged. Any ownership-transfer
  proposal must preserve this observably; explain why yours does.

## Report format (your final message)

Keep it short and point at the plan document:

1. One-line verdict per finding A–E (CONFIRMED / REFUTED / PARTIAL).
2. The answer to Deliverable item 2 (the refcount question) in 3–5 sentences —
   this is the highest-value output of the brief; put it in the report itself,
   not only in the document.
3. The step list, one line each, with expected win.
4. Anything you refused to plan and why.
5. Confirm `git status` shows exactly one new file (the plan document) and no
   modified files.
