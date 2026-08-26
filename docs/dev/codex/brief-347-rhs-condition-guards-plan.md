# brief-347 — rule right-hand-side `Condition` guards are not implemented (design only)

## Goal

`f[x_] := rhs /; test` is one of the most common ways to write a conditional
definition, and rum does not implement it. The guard is never evaluated, a
false guard never rejects the rule, and the `Condition` expression leaks into
the result. The same hole exists in the replacement system. Diagnose it and
produce a step-by-step repair plan with drafted follow-up implementation
briefs.

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes
with the release `rum-cli` are in scope; leave no probe files in the
repository, and feed every probe `stdin < /dev/null`.

This is a correctness defect of the same family as the two found earlier in
this sweep (`Do[f[i] = i, …]` installing one raw definition, `ValueQ`
committing side effects): a standard idiom that silently produces something
other than what it says.

## Measured evidence (director, release CLI at `8fb36c58`)

Definitions:

```text
b[x_] := x^2 /; x > 0

DownValues[b]  ->  {HoldPattern[b[x_]] :> (x ^ 2 /; Greater[x, 0])}
b[3]           ->  9 /; True          (must be 9)
b[-3]          ->  9 /; False         (must be b[-3]: the rule does not apply)
```

Rule ordering and dedup:

```text
f[x_] := "positive" /; x > 0
f[x_] := "other"

f[3]   ->  "other"      (must be "positive")
f[-3]  ->  "other"
```

The guarded rule is gone entirely: dedup compares left-hand sides only
(`definitions/definition_tags.rs:420`, `lhs_patterns_equivalent`), so a rule
that differs solely by its rhs guard is treated as a redefinition of the
unguarded one.

Replacement system, both rule heads:

```text
{1, -2, 3} /. (x_Integer :> "pos" /; x > 0)  ->  {"pos" /; True, "pos" /; False, "pos" /; True}
{1, -2}    /. (x_Integer -> "pos" /; x > 0)  ->  {"pos" /; True, "pos" /; False}
                                          (must be {pos, -2, pos} and {pos, -2})
```

Immediate assignment stores the guard unprocessed too:

```text
g[1] = 5 /; True
DownValues[g]  ->  {HoldPattern[g[1]] :> (5 /; True)}
g[1]           ->  5 /; True
```

**What already works, and is the model to follow:** the guard in *pattern*
position is correct, as is `PatternTest`:

```text
c[x_ /; x > 0] := x^2      c[3] -> 9      c[-3] -> c[-3]
d[x_?NumericQ] := x^2      d[sym] -> d[sym]
```

A bare `Condition` expression evaluates to itself (`5 /; True` -> `5 /; True`).
I believe that is correct — outside a rule there is nothing to guard — but
confirm it rather than assuming it.

## Context pointers

- Read `AGENTS.md` and `docs/guide/patterns-and-rules.md` before touching
  anything.
- `crates/rum-core/src/heads.rs:39` — `Condition` is a registered pattern
  head; `:141` lists it among the pattern heads.
- `crates/rum-runtime/src/runtime/pattern_matching/core.rs:498` —
  `MatcherNode::Condition`, the **working** lhs-side implementation, and
  `:125-128` where the guard is substituted and evaluated. `:43-50` is the
  `PatternTest` analogue.
- `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:420` —
  `lhs_patterns_equivalent`, the dedup predicate that loses the guarded rule.
- `crates/rum-runtime/src/runtime/definitions/application/` — where a matched
  definition's rhs is produced; `fast_match.rs:194-198` already declines
  `MatcherNode::Condition`, so the slow path is the one that matters.
- The replacement system consumes rules through its own paths — find them;
  `ReplaceAll` is only one consumer.
- Acceptance is mechanical: the named tests and gates decide.

## What the plan must contain

1. **The mechanism, proved.** Where a matched rule's rhs is produced, and why
   a `Condition` there is passed through instead of being unwrapped and
   tested. State plainly whether this is an omission or a deliberate
   simplification, with `path:line` evidence.

2. **The full surface, enumerated — not sampled.** For every construct that
   stores or consumes a rule, report the current behavior and the required
   behavior:
   - definition heads: `Set`, `SetDelayed`, `TagSet`, `TagSetDelayed`,
     `UpSet`, `UpSetDelayed`, and whatever else installs a rule;
   - value types: `OwnValues`, `DownValues`, `SubValues`, `UpValues`,
     `NValues`, `DefaultValues`;
   - rule heads in the replacement system: `Rule` and `RuleDelayed`, through
     **every** consumer you can find (`ReplaceAll`, `Replace`,
     `ReplaceRepeated`, `ReplacePart`, `Cases`, `DeleteCases`, `Position`,
     `Count`, association and `Part` rule forms, `Dispatch`-like paths if any);
   - nested guards (`rhs /; a /; b`) and a guard on a lhs that already has one;
   - a guard whose test is not `True`/`False` (neither, or a symbol);
   - a guard that errors, that returns unevaluated, or that suspends.

3. **Rule identity and ordering.** Two rules with the same lhs and different
   rhs guards must coexist and be tried in order. Say exactly what identity
   rule replaces today's lhs-only comparison, what "in order" means for rum's
   definition storage (which sorts by specificity), and how a guarded rule
   that declines hands off to the next candidate. **This is the part most
   likely to ripple**: quote every site that compares, sorts, or deduplicates
   definitions.

4. **Guard evaluation runs user code.** That means side effects, messages,
   errors, resource limits, and suspension inside what used to be pure rule
   selection. Specify what happens on each, and confirm the design preserves
   the `suspend_with_frame*` continuation shape. Note that the lhs-side guard
   already faces this and check what it does today — consistency with it is
   the default answer.

5. **Cost analysis.** Rule selection is the hottest path in the evaluator.
   Say what each proposed step costs when **no** guard is present — that must
   be zero or near-zero — and which steps need the microbench A/B gate.

6. **Existing-test survey.** Which tests encode the current (wrong) behavior,
   which would catch the repair, what new tests are needed. Quote the
   assertions. Check the generated reference too: if `docs/reference/`
   describes `Condition`, quote what it promises.

7. **A step-by-step plan**, smallest safe steps first, each naming
   files/functions, the invariant it preserves, its gate tests, and its ripple
   inventory.

8. **Drafted follow-up implementation briefs** in `TEMPLATE.md` shape.

9. **Open questions for the director**, each with your recommendation. Include
   at minimum: what an immediate `Set` should do with a guarded rhs, and what
   a non-boolean guard result means.

## Scope

- Files you MAY modify: **NONE**. Design-only. Put probes in `/tmp`.
- Files/areas you MUST NOT touch: everything, including `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Every claim needs a probe that reaches the code you are claiming about**,
   and every probe needs a liveness assertion — an exact `DownValues` listing
   or a printed result, never a bare absence of error.
2. **Enumerate the matrix.** A repair that fixes `SetDelayed` and leaves
   `UpSetDelayed` or `Cases` broken is not acceptable, and this project has
   been bitten specifically by attributing a correct root cause too narrowly
   or too widely.
3. **The lhs-side implementation is a working reference.** Read it before
   designing anything new; a second, divergent guard mechanism would be worse
   than the bug.
4. **Do not change what a bare `Condition` expression evaluates to** without
   saying so explicitly and justifying it.
5. **Rule selection is a hot path.** A per-candidate branch paid when no guard
   exists is a regression to be avoided by construction, not measured away
   afterwards.
6. **Suspension safety** — guard evaluation can suspend; preserve continuation
   shape.
7. **Stop-loss is real.** If the mechanism does not resolve after two serious
   attempts, report what you established and where the trail ends.

## Verification (MANDATORY)

No source change, so no test gate. The evidence IS the deliverable:

- The full construct matrix, with verbatim `rum-cli` output for every cell.
- Verbatim `path:line` source for the mechanism and for every dedup/ordering
  site you name.
- The existing-test survey, quoted.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config, and do NOT modify
  any file in the repository.
- Touch nothing outside this repo except `/tmp`; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): argue the required
  semantics from rum's own docs, its existing lhs-side guard implementation,
  and internal consistency — never from another engine's observed output, and
  put no such output in your report.
- No new `unsafe` in any drafted plan for `rum-core`/`rum-runtime`; no new
  dependencies.
- Bounded probes: `scripts/run-bounded.py --timeout SECONDS -- <command>`;
  state the bound and where you derived it.

## Report format (your final message)

1. Diagnostic only, no source change — state it.
2. The mechanism, with `path:line` evidence and the probe that reaches it.
3. The full construct matrix with verbatim backing.
4. Rule identity and ordering: the new predicate and every rippled site.
5. Guard-evaluation semantics: effects, errors, non-boolean, suspension.
6. Cost analysis for the no-guard path.
7. The existing-test survey and the reference's promises.
8. The step plan with per-step gates and ripple.
9. The drafted follow-up implementation briefs.
10. Open questions for the director, each with your recommendation.
11. Stop-loss — where you stopped and why (empty if none).
