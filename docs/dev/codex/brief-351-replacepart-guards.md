# brief-351 — rhs `Condition` guards, step 4: `ReplacePart`

## Goal

`ReplacePart` has its own rule parser and target planner, so it did not
benefit from step 3. Every rule form leaks the guard today (release CLI at
`5418c060`):

```text
ReplacePart[{1,2,3}, 1 -> ("a" /; True)]        -> {"a" /; True, 2, 3}   must be {"a", 2, 3}
ReplacePart[{1,2,3}, 1 -> ("a" /; False)]       -> {"a" /; False, 2, 3}  must be {1, 2, 3}
ReplacePart[{1,2,3}, {1 -> ("a" /; False), 2 -> "b"}]
                                                -> {"a" /; False, "b", 3} must be {1, "b", 3}
ReplacePart[{1,2,3}, 1 :> ("a" /; True)]        -> {"a" /; True, 2, 3}
ReplacePart[{{1,2},3}, {1,2} -> ("a" /; True)]  -> {{1, "a" /; True}, 3}
ReplacePart[{1,2,3}, {2 ;; 3} -> ("s" /; True)] -> {1, "s" /; True, "s" /; True}
```

Unguarded behavior is correct and must stay byte-identical, including
later-rule-wins: `ReplacePart[{1,2,3}, {1 -> "x", 1 -> "y"}]` is `{"y", 2, 3}`.

**Reuse the substrate.** `definitions/rhs_guards.rs` has been byte-identical
through three steps and must stay the only guard mechanism.

**Only `ReplacePart`.** String rules and `OwnValues` are later briefs; leave
their current wrong behavior alone.

## Director's rulings on the semantics

These are decided; implement them, do not re-open them.

1. **A declining guard means the rule does not apply to that position** — it
   is indistinguishable from a rule whose position spec did not select that
   position. If no rule applies to a position, the original element stays.
2. **A declining rule does not compete for later-rule-wins.** With
   `{1 -> "x", 1 -> ("y" /; False)}` the result is `{"x", 2, 3}`: the guarded
   rule dropped out, so the earlier rule wins the position. It is *not*
   `{1, 2, 3}`.
3. **Guard evaluation order for a contested position is last rule to first.**
   The last rule whose guard accepts wins, and the guards of earlier rules for
   that position are then **not evaluated at all**. This keeps the number of
   guard evaluations minimal and matches the later-wins direction. It is
   observable when guards have effects, so make it a documented, tested
   contract rather than an accident of implementation.
4. **Each (rule, position) candidate evaluates its guard at most once** per
   `ReplacePart` call.

## Context pointers

- `5ab6afc5`, `efd6dc80`, `5418c060` are the three previous steps — read the
  last one's diff first; this is the same shape against a different planner.
- `crates/rum-runtime/src/runtime/definitions/rhs_guards.rs` — the substrate.
- `ReplacePart`'s own parsing and planning live under
  `kernel_data/structure_data/` (`replace.rs`, `parsing.rs`, `replace_part.rs`,
  `plan.rs`). Find the full set yourself.
- The merged exact-position planner is the performance-critical path; the
  `replacepart-wide-rule-list` microbench label exercises it.
- **Association positions**: `ReplacePart[<|"k" -> 1|>, "k" -> ("v" /; True)]`
  returns the association unchanged today, which looks like the position spec
  is simply not accepted in that form rather than a guard defect. Determine
  the accepted association position forms, cover them, and report what you
  found — do not assume my probe was well-formed.

## Deliverable

- `ReplacePart` rules split their guards at parse time, for both `->` and
  `:>`, across every position form: exact index, negative index, nested
  position list, `Span`, `All`, pattern-driven and key-driven forms, and
  whatever else the parser accepts.
- Wholly unguarded rule sets keep the existing merged exact planner untouched.
  Only a rule set containing at least one guard takes the rule-major lane, and
  that selection happens **once per call**, not once per position.
- The four rulings above, implemented and tested.
- Guards evaluate with bindings substituted for pattern-driven rules, in the
  guarded body's option scope, accepting only exact `True`.
- Suspension inside a guard resumes at the same rule and the same position,
  without replaying earlier guards or their effects, and without losing the
  already-planned targets.
- Raw delayed bodies (`:>`) are still inserted unevaluated after the guard
  accepts.

### Tests

Exact values, never just absence of error:

- Each position form above, with an accepting and a declining guard.
- Ruling 2: `{1 -> "x", 1 -> ("y" /; False)}` is `{"x", 2, 3}`.
- Ruling 3: a counter in each guard proves the earlier rule's guard was never
  evaluated when a later one accepted, and that the order is last-to-first.
- Ruling 4: a counter proves one evaluation per (rule, position).
- A declining guard on a `Span` leaves **every** position in that span
  unchanged.
- Nested positions, negative indices, `All`.
- Non-boolean guard results decline.
- A guard that suspends resumes correctly — assert a counter.
- Wholly unguarded rule sets: existing behavior byte-identical (the existing
  suites are the assertion; add one explicit later-rule-wins case).
- The excluded lanes still leak as they do today (`StringReplace`) — keep that
  baseline assertion.

### A/B

`--scale 5`, same-binary noise floor reported **first**, then base versus
candidate:

```
replacepart-wide-rule-list   set-part-fill-small     set-part-fill-large
set-part-fixed100-len100     set-part-fixed100-len800
appendto-part-outer1000-small  appendto-part-outer1000-large
replaceall-simple            matcher-single-tiny     symbol-ownvalue-read
```

Add stable labels for a guarded `ReplacePart` hit and a guarded
decline-with-fallback. Name the untouched controls; never the suite median.
In the three previous steps the same-binary spread reached 3.8%, 8.1% and
7.2%, and one flagged regression reversed sign on re-measurement — so a
movement smaller than your measured floor is not a regression.

## Scope

- Files you MAY modify: the `ReplacePart` parsing/planning/application modules
  under `kernel_data/structure_data/`; `definitions/rhs_guards.rs` only if a
  genuine extension is needed; runtime tests;
  `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported**, with each file
  justified. If a site needs a real decision rather than a mechanical fix,
  STOP and report.
- **The general accept/stop criterion**: *any behavior change that occurs only
  when a guarded rule exists is legitimate; any behavior change observable
  with no guarded rule present is a stop.*
- MUST NOT touch: string rules, `OwnValues`, generic rewrite rules, the
  definition tables, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`, `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **The unguarded path must cost nothing.** The merged exact planner is the
   hot path; a per-position guard check when no guard exists is a rejection.
   Select the lane once per call.
2. **Do not allocate a second selector or target vector** for the guarded
   lane; reuse what the planner already owns.
3. **Guards run user code** during planning — preserve `suspend_with_frame*`
   continuation shape and the already-planned target state across suspension.
4. **A decline is not an error** and must not emit a message.
5. **Reuse the substrate**; a divergent accept rule is worse than the bug.
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

Then the A/B as described.

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
2. The position forms the parser accepts, and what you found about
   association positions.
3. Verification — each command with verbatim output.
4. The exact-value test matrix, including the counter evidence for rulings 3
   and 4.
5. The A/B — counters first, then rows, with the same-binary noise floor and
   the controls named.
6. Stop-loss / open questions.
