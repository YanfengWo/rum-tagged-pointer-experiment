# brief-367 — eliminate the opaque continuation captures

## Goal

Step 3 of plan-364. Replace the generic-closure captures that a suspension
frame can hold with named, derivable data, so that step 4 has nothing left it
cannot express as a struct. Evaluation semantics and suspend/resume behavior
are preserved exactly.

**This brief does not touch the suspension API itself.** `SuspendedEvaluation`,
the frame queue, `ClosureFrame`, `suspend_with_frame*`, `drive_*_value`, the
GC root walk, and the memory proxy are all step 4. Here you only change what
those APIs are handed.

This is the first step in the line that touches hot paths, so it carries a
measured A/B gate.

## Context pointers

- Read `AGENTS.md` first.
- `docs/dev/codex/plan-364-frame-roots.md:579` — the escape-hatch table with
  the intended redesign per group. Its line numbers predate steps 1 and 2;
  the anchors below are current as of `a02064b8`.
- Steps 1 and 2 are landed: `FrameRootTrace` exists, and 155 types derive it.
  Anything you introduce here should be derivable the same way.

## What counts as an escape hatch (read this before grepping)

An escape hatch is a **generic closure that a suspension frame can end up
holding**. The reliable signal is a `Fn`/`FnOnce` bound carrying
`+ Send + 'static` on a path that reaches `suspend_with_frame*` or
`drive_*_value`.

The workspace has many other generic closure parameters that are **not** in
scope and must not be touched — `lowering.rs` and `lowering/binary*.rs`
(`F: FnMut(&str) -> Option<Symbol>`, ten-plus sites),
`predicates.rs`/`predicates/numeric.rs`/`predicates/collections.rs`,
`definitions/definition_tags.rs`, `substitution/binding_substitution.rs`,
`profiling/timing.rs`, `definitions/application/mod.rs:28`, and
`kernel_data/control_data/branching_loops/iteration/mod.rs:16`. None of them
is stored across a suspension; they are ordinary monomorphized helpers.

**Attributing this change too widely is the specific failure mode to avoid
here.** If a site does not reach a suspension frame, leave it alone, and say
so if you are unsure rather than converting it.

The eight `F:` bounds in `suspension/core.rs` are the API itself — step 4.

## The five groups, with current anchors

1. **Count-like finish** — `tree_walkers/count_like.rs:207`,
   `F: FnOnce(usize) -> Expr + Send + 'static`. Plan's redesign: a
   `CountLikeFinish` enum with Count / MemberQ / FreeQ arms.
2. **Rule-catalog continuations** —
   `definitions/definition_catalog/rewrite_rules.rs:54` and `:67`, both
   `FnOnce(&mut Session, ...) -> Result<U, EvalError> + Send + 'static`.
   Named continuation-state enums for the six rule-expression callers and the
   two compiled-rule callers. Enumerate the callers rather than recalling
   them.
3. **The rule applier** — this is the source of the family:
   `definitions/application/replace_all.rs:61`,
   `shared_rewrite_rule_applier`, returns
   `impl Clone + Fn(&mut Session, Expr, RuleApplicationSource) -> Result<Option<Expr>, EvalError> + Send + 'static`.
   It already builds a `RewriteRulePlan` internally, so the redesign is to
   carry that plan (and the source) explicitly and call a named
   rule-application method.
4. **Its consumers** — `rewrite_data/replace/once.rs:161`, `:209`, `:327`,
   `:422`, and `structure_data/replace/replace_at.rs:105`, all taking the
   same `F: Clone + Fn(...) + Send + 'static`.
5. **Merge completion** — `rum-stdlib/structure_ops/resumable_merge_sort.rs:88`
   and `:111`, `C: FnOnce(Vec<T>) -> Expr + Send + 'static`. Named
   `MergeCompletion<T>` implementations, **no blanket closure
   implementation** — a blanket `impl` for all `FnOnce` would recreate the
   hole this step closes.

Plan-364 counted 13 boundaries. Report the number you actually find, the same
way step 2 did with its type count; a different number is a finding, not a
discrepancy to pad toward.

## Scope

Files you MAY modify:

- the five groups' files above, and the caller files the conversion forces;
- their focused tests.

Files/areas you MUST NOT touch:

- `crates/rum-runtime/src/runtime/suspension/**` — the API and the traversal
  are both finished for now;
- `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs`,
  `debug_tools/resources/memory.rs`;
- `crates/rum-derive/**`;
- the not-in-scope generic helpers listed above;
- `crates/rum-core/src/expr.rs` representation and public serde shapes;
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`.

## Deliverable

1. Each group converted to named state, derivable with `FrameRootTrace`.
2. **A capture scan showing zero remaining `F`/`impl Fn`/`FnOnce` captures
   that can reach a suspension frame**, with the command and its output. The
   out-of-scope helpers above will still match a naive grep — your scan has to
   distinguish them, and you should say how it does.
3. Identical results on both the ordinary and the suspended path for every
   converted group. A test that exercises the suspended path of at least the
   rule applier and the merge completion, since those are the two that carry
   real state.
4. The measured A/B below.

## rum / Rust pitfalls (read twice)

1. **Preserve continuation order and one-shot behavior.** A frame resumes at
   most once; the ordering contracts of the rewrite lanes are documented in
   `docs/guide/patterns-and-rules.md` and were established by briefs 347-354.
   `ReplacePart` in particular runs back-to-front and does not evaluate a
   winner's earlier rivals.
2. **Do not turn a suspending call eager** to simplify a conversion. If a
   conversion seems to require that, STOP and report.
3. **Static dispatch where the loop is tight.** An enum branch inside a
   per-element loop is a regression risk; prefer resolving the branch outside
   the loop. The A/B is what settles it.
4. **No deep expression clones.** `RewriteRulePlan` and friends are shared;
   carry `Arc`/borrowed forms, never a deep copy per element.
5. **Do not widen the change.** See "What counts as an escape hatch".
6. **Stop-loss is real.** If a group resists conversion twice, STOP and report
   the shape and the error.

## Verification (MANDATORY — you run it; iterate to green)

Correctness first:

```
cargo fmt --check
cargo test --workspace --offline
```

Baseline is `a02064b8`: **2149 passed, 0 failed**. Report the new total and
prove no existing test changed result with a sorted per-test-name diff and
`removed_or_changed=0`, as in steps 1 and 2.

Then the measured A/B, profiling env OFF, base and candidate in one session:

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief367-base   # on a02064b8
bash scripts/core-language-microbench.sh --output-dir /tmp/brief367-new    # on the change
```

**Affected labels** — paste every row:

- `count-wide-no-match`, `do-count-null`
- `replaceall-simple`, `replace-wide-no-match`,
  `replacepart-wide-rule-list`, `replacepart-guarded-hit`,
  `replacepart-guarded-decline-fallback`
- `stringreplace-guarded-hit`, `stringreplace-guarded-decline-fallback`,
  `stringreplace-pattern-unguarded`
- `sort-callback-small`, `sort-callback-large`, `sortby-callback-small`,
  `sortby-callback-large`, `reversesort-callback-small`,
  `reversesort-callback-large`, `keysort-callback-small`,
  `keysort-callback-large`, `keysortby-callback-small`,
  `keysortby-callback-large`
- `matcher-single-tiny`, `matcher-single-wide-small`,
  `matcher-single-wide-large`

**Named controls** — paste these too. They exercise the default-comparator
paths that this change does not touch, so they are the noise reference:

- `sort-default-wide`, `sortby-default-wide`, `reversesort-default-wide`,
  `keysort-default-wide`, `keysortby-default-wide`,
  `sort-symbol-default-width4000`, `sort-symbol-default-width16000`

**Noise floor: three runs minimum, and report the spread.** Two runs do not
estimate a noise floor — their difference is a single sample, and this project
has already had one false stop-loss from exactly that. State the observed
per-label spread across runs, and judge any movement against it. **Never
normalize against the suite median**; use the named controls above. If the
controls move as much as the affected labels, that is the noise floor —
report no regression rather than "fixing" noise.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo except `/tmp`; no network; `cargo --offline`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own behavior only.
- **No new `unsafe`**; `rum-core` and `rum-runtime` are zero-`unsafe`.
- No new dependency.
- Minimal diff; no drive-by reformatting or reorganization.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; state
  the bound and where you derived it. Persist raw microbench output to files
  before parsing it.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per group, with the named types introduced.
2. The boundary count you found, versus plan-364's 13.
3. The capture scan — command, output, and how it excludes the out-of-scope
   helpers.
4. Correctness — commands, totals, and `removed_or_changed=`.
5. The A/B — affected rows, control rows, the three-run spread, and your
   verdict against that spread.
6. Stop-loss / open questions (empty if none).
