# brief-346 — `ValueQ` answers by lookup, never by evaluation (design only)

## Goal

Plan the replacement of `ValueQ`'s trial-evaluate-and-roll-back implementation
with one that **never evaluates its argument**. Produce a diagnosis of how far
rum's existing machinery can decide the question, a step-by-step repair plan,
and drafted follow-up implementation briefs.

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes
with the release `rum-cli` are in scope; leave no probe files in the
repository, and feed every probe `stdin < /dev/null`.

## The ruling this brief implements (director, 2026-08-26)

`ValueQ[expr]` is `True` exactly when evaluating `expr` would produce
something different from `expr` — and the answer must be reached **without
evaluating it**. The precision of today's answers is kept:

```text
f[1] = 3
ValueQ[f[1]]   -> True    (a definition applies)
ValueQ[f[2]]   -> False   (no definition can apply)
```

A coarser rule — "the head has some definition, therefore True" — was
considered and **rejected**: it makes `ValueQ[f[2]]` `True` merely because
`f[1]` was defined, which is surprising and less useful. Do not propose it.

This is not primarily a performance change. Not evaluating means there is
nothing to roll back, so it closes three contract breaches that brief-345
reproduced (the deterministic PRNG advances; `Print` reaches an installed host
sink; `FileWrite` commits), and it removes the whole-session snapshot as a
side effect rather than as a goal. Treat correctness as the deliverable and
the speed as a consequence.

## Context pointers

- Read `AGENTS.md` and `docs/testing-strategy.md` before touching anything.
- `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:37-56`
  — today's `eval_value_q`: snapshot, `evaluate`, restore, compare.
- `crates/rum-runtime/src/runtime/eval_core/stability.rs` — **the most likely
  starting point, and the thing to be most careful about.** The evaluator
  already decides "would evaluating this change it" in order to skip stable
  expressions (`resolution.rs:321-329` consumes it; `stability.rs:206-210`
  handles plain symbols). It is deliberately **conservative**: when unsure it
  reports "not stable", and that direction becomes a wrong `True` here. Its
  precision is the central question of this brief — establish it with probes,
  do not assume it.
- `crates/rum-runtime/src/runtime/definitions/` — the value tables the lookup
  must consult.
- `crates/rum-runtime/src/runtime/tests/definitions_and_rules.rs:355-425` —
  `valueq_uses_rollback_trial_evaluation_without_side_effects`, which encodes
  the current design intent and will need rewriting. Note its `Print`
  assertion checks the **buffered** `prints` vector, which is why the sink
  leak went unnoticed; the replacement tests must cover the sink path.

## What the plan must contain

1. **The decision procedure, specified.** State precisely what makes
   `ValueQ[expr]` true, as a rule over expression shape, and check it against
   every case below. Report the shape's current answer and the answer your
   rule gives, marking every divergence:
   - a bare symbol with an `OwnValue`, with a delayed `OwnValue`, with
     neither;
   - a bare symbol carrying only `DownValues` (today `False` — keep it);
   - `h[args]` where a `DownValue` matches, where none matches, where `h` has
     no definitions at all;
   - `SubValues` (`f[1][2]`), and `UpValues` attached to an argument rather
     than the head — an `UpValue` makes the expression change without the head
     owning any rule;
   - **attribute-driven change with no rule at all**: `Orderless` reordering,
     `Flat` flattening, `OneIdentity`, `Listable` threading, `Sequence`
     splicing. `ValueQ[f[b, a]]` with `f` `Orderless` evaluates to `f[a, b]`,
     which is a change;
   - built-in heads: `1 + 1` is `True` today, `{x}` is `False`; give the rule
     that separates them and name where in the catalog it can be read;
   - `NValues`, `DefaultValues`, and any other value type rum stores;
   - **nesting**: `{f[1]}` — the head `List` is inert but an inner element
     changes, so the whole expression changes. Say whether the rule recurses
     and what that costs;
   - association and `Part` forms; opaque lazy heads (`HoldAllComplete`);
   - atoms (integer, rational, string) and self-evaluating symbols.

2. **How far the existing stability check gets, measured.** For each shape
   above, does `cached_expr_is_evaluation_stable` decide it correctly, decide
   it conservatively wrong, or not reach it? Probe it — a claim about this
   machinery needs a probe that reaches it. Where it is conservative, say what
   the conservative answer would make `ValueQ` return and whether that is
   acceptable.

3. **What an undecidable shape should answer.** If some shape cannot be
   decided without evaluating, the options are: answer `True` conservatively,
   answer `False`, or fall back to today's trial evaluation for that shape
   alone. **If you propose the fallback, enumerate exactly which shapes reach
   it** — those are the shapes where the three side-effect breaches would
   survive, and the director needs that list to judge the design. Give your
   recommendation.

4. **Suspension and requests.** Today `ValueQ[InputString["…"]]` returns
   `True` through the `InputRequired` arm and leaves no pending request. A
   lookup never suspends. Confirm the new rule reaches the same answer for
   that shape and every other suspension-capable one, and that no continuation
   machinery is left expecting a trial evaluation.

5. **The dependency inventory.** Everything that relies on `ValueQ`'s current
   behavior or on the rollback machinery it shares: in-tree callers, host API
   surface, `Monitor` refresh (recorded in `TODO.md` as sharing the
   "best-effort" caveat), and the snapshot helpers themselves — `eval_file`
   is the other consumer and must be left byte-identical. Quote what you find.

6. **The existing-test survey**, quoting the assertions: which encode the
   trial-evaluation design and must be rewritten, which stay, and what new
   tests are needed — including a `Print`-with-sink test, a PRNG test, and a
   `FileWrite` test proving the effects no longer happen.

7. **The documentation change.** The generated reference's "by trial
   evaluation without committing side effects" description comes from the
   catalog source (`builtin_catalog/namespace/symbols.rs`). State the new
   wording; it must describe what the implementation does, with no promise it
   cannot keep.

8. **A step-by-step plan** — smallest safe steps first, each naming
   files/functions, the invariant it preserves, its gate tests, its ripple
   inventory, and whether it needs the microbench A/B.

9. **Drafted follow-up implementation briefs** in `TEMPLATE.md` shape — real
   drafts, not step summaries.

10. **Open questions for the director**, each with your recommendation.

## Scope

- Files you MAY modify: **NONE**. Design-only. Put probes in `/tmp`.
- Files/areas you MUST NOT touch: everything, including `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Every claim needs a probe that reaches the code you are claiming about.**
   This applies especially to the stability check, which is reached from the
   evaluator's hot path and may not be exercised by the route you assume.
2. **Every probe carries a liveness assertion** — a printed value, an exact
   `DownValues` listing, a counter delta. Never a bare timing or a bare
   absence of error.
3. **Enumerate the matrix; do not sample it.** Attribute-driven change with no
   rule present is the shape most likely to be missed, and it is exactly the
   kind of omission that has cost this project before.
4. **Do not weaken the answer to make the implementation easy.** Returning
   `True` whenever the head has definitions is the rejected rule; arriving at
   it by another route is the same rejection.
5. **No default-path cost.** Whatever the lookup consults must not add work to
   ordinary evaluation. If a proposal requires new bookkeeping on assignment
   or evaluation paths, say so plainly — that is a rejection criterion.
6. **`eval_file` shares the snapshot helpers.** A `ValueQ` change must leave
   loader rollback exactly as it is.
7. **Stop-loss is real.** If the decision procedure does not resolve after two
   serious attempts, report the part you established, the shapes you could not
   decide, and stop.

## Verification (MANDATORY)

No source change, so no test gate. The evidence IS the deliverable:

- The full shape table, with verbatim `rum-cli` output for every current
  answer and your rule's answer beside it.
- Verbatim `path:line` source for the stability check's behavior on each shape
  you classify.
- The dependency inventory, quoted.
- The existing-test survey, quoted.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config, and do NOT modify
  any file in the repository.
- Touch nothing outside this repo except `/tmp`; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. Argue the intended semantics from rum's own docs, its
  evaluator's rules, and internal consistency — never from another engine's
  observed output, and put no such output in your report.
- No new `unsafe` in any drafted plan for `rum-core`/`rum-runtime`; no new
  dependencies.
- Bounded probes: `scripts/run-bounded.py --timeout SECONDS -- <command>`;
  state the bound and where you derived it.

## Report format (your final message)

1. Diagnostic only, no source change — state it.
2. The decision procedure, specified as a rule over expression shape.
3. The full shape table: current answer, your rule's answer, divergences
   marked.
4. How far the existing stability check gets, per shape, with probes.
5. Undecidable shapes and what they should answer, with your recommendation
   and — if you propose a fallback — the exact list of shapes that reach it.
6. Suspension behavior.
7. The dependency inventory and the existing-test survey.
8. The new reference wording.
9. The step plan with per-step gates, ripple, and cost.
10. The drafted follow-up implementation briefs.
11. Open questions for the director, each with your recommendation.
12. Stop-loss — where you stopped and why (empty if none).
