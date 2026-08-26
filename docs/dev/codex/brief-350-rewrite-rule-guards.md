# brief-350 — rhs `Condition` guards, step 3: generic rewrite rules

## Goal

Extend the guard substrate to `Rule` and `RuleDelayed` as consumed by the
replacement system. Measured today (release CLI at `efd6dc80`):

```text
g = x_Integer :> "pos" /; x > 0

{1, -2} /. g            ->  {"pos" /; True, "pos" /; False}     must be {"pos", -2}
Replace[1, g]           ->  "pos" /; True                       must be "pos"
Replace[-2, g]          ->  "pos" /; False                      must be -2
{1, -2} //. g           ->  {"pos" /; True, "pos" /; False}     must be {"pos", -2}
ReplaceList[1, g]       ->  {"pos" /; True}                     must be {"pos"}
Cases[{1,-2,3}, g]      ->  {"pos" /; True, "pos" /; False, "pos" /; True}
                                                                must be {"pos", "pos"}
{1,-2} /. (x_Integer -> ("pos" /; x > 0))    same defect through the Rule head
{1,-2} /. {guarded, unguarded}               same defect in a rule list
```

`Cases` is the sharpest case: it currently *collects* an element whose guard
declined. A declining guard must mean the rule did not apply, so that element
is simply not a case.

**Reuse the substrate.** `definitions/rhs_guards.rs` is unchanged through both
previous steps and must stay the only guard mechanism. If it needs a genuine
extension, say so and explain why.

**Only generic rewrite rules and the handler consumers.** `ReplacePart`,
string rules, and `OwnValues` are later briefs — leave their current wrong
behavior alone.

## Context pointers

- `5ab6afc5` (DownValues) and `efd6dc80` (UpValues/SubValues/Defaults) are the
  models — read both diffs first. This step is the same shape applied to
  `RewriteRule`.
- `crates/rum-runtime/src/runtime/model/definitions.rs:245-252` —
  `RewriteRule`, which has no guard field. Put the new field **last**.
- `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:595-610`
  — `normalize_single_rewrite_rule`, where the whole rhs is taken as the body.
- The consumers: `definitions/application/replace_all.rs` and its `plan`
  module, `replace/all.rs`, `replace/once.rs`, `replace/repeated.rs`,
  `replace/list*.rs`, `cases/collect.rs`, and the shared rewrite-plan cache.
  **Find them all yourself** — the list above is a starting point, not an
  inventory, and a consumer missed here is a defect shipped.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:147-155`
  — `CatchExceptions` parses its handlers as a list of rules
  (`parse_exception_handlers`). **Determine whether handler selection has the
  same defect** and report the probe either way; structured exceptions are a
  separate channel from `Throw`/`Catch`, so use the right one to reach it.
- Rule-shaped **data** must not change: association entries
  (`<|1 -> ("v" /; True)|>` stays as it is today), option rules, and schema
  pairs are data, not candidate selection. Confirm this holds after your
  change.
- Acceptance is mechanical: the named tests and A/B rows decide.

## Deliverable

- `RewriteRule` carries an ordered guard sequence, split during normalization,
  for both `Rule` and `RuleDelayed`.
- On a match, guards evaluate left to right with the bindings substituted;
  only exact `True` accepts; a decline means **the rule did not apply** — the
  next rule in the list is tried, and if none applies the expression is left
  unchanged.
- Per consumer: `Replace`/`ReplaceAll` leave the subexpression untouched;
  `ReplaceRepeated` treats a decline as no rewrite for the fixed-point test;
  `ReplaceList` omits that binding without losing the enumeration of the
  others; `Cases` omits the element entirely.
- Rule lists preserve order: a declining guarded rule hands off to the next
  rule exactly as a pattern mismatch does.
- `NormalizeRules` and any public rule-inspection surface rebuild the
  `Condition` form.
- Whatever you find for `CatchExceptions`: if handlers share the defect, a
  declining handler guard resumes the existing handler/ancestor scan cursor
  rather than aborting it.

### Tests

Exact values, never just absence of error:

- Each consumer above, with a guard that accepts and one that declines.
- Rule lists where the guarded rule declines and a later unguarded rule
  applies, and the reverse order.
- Both rule heads (`->` and `:>`).
- Nested guards; a two-guard rule whose second guard declines.
- Non-boolean guard results decline.
- A guard that suspends (`InputString`) resumes at the same rule and the same
  position without replay — assert a counter.
- A guard with a side effect runs exactly once per candidate tested.
- `ReplaceRepeated` terminates when the only applicable rule's guard declines.
- Rule-shaped data unchanged: association entries, options.
- The excluded lanes still behave as they do today (`ReplacePart` and
  `StringReplace` keep leaking `Condition`) — assert that deliberately, so the
  next step has a baseline.

### A/B

Base versus candidate at `--scale 5`, with the same-binary noise floor
reported first:

```
replaceall-simple            cases-wide-no-match        count-wide-no-match
position-wide-no-match       deletecases-wide-no-match  replace-wide-no-match
deletecases-wide-association-no-match
downvalue-exact-hit          downvalue-pattern-hit      matcher-single-tiny
set-ownvalue                 symbol-ownvalue-read
```

Add stable labels for a guarded rewrite hit and a guarded decline-with-fallback.

Before calling anything a regression, run the **same** binary twice and report
that spread. In the previous two steps the same-binary spread reached 3.8% and
8.1% respectively, and one flagged "regression" reversed sign on
re-measurement. Name the untouched controls; never the suite median.

## Scope

- Files you MAY modify: `model/definitions.rs`;
  `definitions/definition_analysis_helpers.rs`;
  `definitions/rhs_guards.rs` only if a genuine extension is needed;
  the rewrite-rule application and plan modules under
  `definitions/application/` and `kernel_data/rewrite_data/`;
  `kernel_data/control_data/exceptions.rs` if handlers share the defect;
  `definition_catalog` entries needed for reconstruction; runtime tests;
  `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported**, the same set the
  previous two steps touched. Justify each file. If a site needs a real
  decision rather than a mechanical fix, STOP and report.
- **The general accept/stop criterion** — use this, not the file list: *any
  behavior change that occurs only when a guarded rule exists is legitimate;
  any behavior change observable with no guarded rule present is a stop.*
- MUST NOT touch: `ReplacePart`, string rules, `OwnValues`, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`, `Expr`/`Symbol`
  representation.

## rum / Rust pitfalls (read twice)

1. **The unguarded path must cost nothing** — no new per-candidate branch,
   allocation, or clone. Unguarded rules keep their exact-match index and
   their current matcher shape; only guarded rules take the slower lane.
2. **The shared rewrite plan is cached.** Guards must not defeat plan reuse
   for unguarded rules, and a cached plan must not outlive a guard's meaning.
3. **Guards run user code** — preserve `suspend_with_frame*` continuation
   shape, and make a suspension resume at the same rule and position.
4. **A decline is not an error.** It must be indistinguishable from a pattern
   mismatch to every consumer, including the fixed-point test in
   `ReplaceRepeated` and the enumeration in `ReplaceList`.
5. **Reuse the substrate** — a divergent accept rule is worse than the bug.
6. **Counters before timings.**
7. **Stop-loss is real.** If a step fails twice for the same reason, STOP and
   report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B as described, pasting per-label rows and the same-binary spread.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no third-party engine comparison anywhere.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What changed — per file, mechanically-forced files listed separately.
2. The consumer inventory you found, and how you found it.
3. What you determined about `CatchExceptions` handlers, with the probe.
4. Verification — each command with verbatim output.
5. The exact-value test matrix, per consumer.
6. The A/B — counters first, then rows, with the same-binary noise floor and
   the controls named.
7. Stop-loss / open questions.
