# brief-352 — rhs `Condition` guards, step 5: string rules

## Goal

String rule consumers live in `rum-stdlib` with their own rule parser, so none
of the previous four steps reached them. Measured today (release CLI at
`b94a5a4a`):

```text
StringReplace["abc", "a" -> ("X" /; True)]   ->  "Condition[\"X\", True]bc"   must be "Xbc"
StringReplace["abc", "a" -> ("X" /; False)]  ->  "Condition[\"X\", False]bc"  must be "abc"
StringReplace["abc", "a" :> ("X" /; True)]   ->  "Condition[\"X\", True]bc"
StringReplace["abc", {"a" -> ("X" /; False), "a" -> "Y"}]
                                             ->  "Condition[\"X\", False]bc"  must be "Ybc"
StringReplace["a1b", x_ ~~ NumberString -> ("N" /; True)]
                                             ->  "Condition[\"N\", True]b"
StringCases["a1b2", DigitCharacter -> ("d" /; True)]
                                             ->  {"d" /; True, "d" /; True}   must be {"d", "d"}
```

This lane is the ugliest of the five: the `Condition` is **stringified** into
the result rather than merely left in place.

`StringSplit["a-b", "-" -> ("s" /; True)]` returns unevaluated — determine
whether `StringSplit` accepts rules at all and report either way; do not
assume my probe was well-formed.

**Reuse the substrate.** `definitions/rhs_guards.rs` has been byte-identical
through four steps. This step needs it from another crate, which is the one
new design problem here — see below.

**Only string rules.** `OwnValues` is the last step; leave it alone.

## The cross-crate constraint

`rhs_guards.rs` is `pub(in crate::runtime)`. `rum-stdlib` needs guard
splitting and guard evaluation, but **the guard mechanism must not become
public API**. Export the narrowest possible surface that lets stdlib do the
job — ideally a single function per operation, taking and returning ordinary
`Expr`/slices, with the substrate's types staying private. State in your
report exactly what you exported and why each item was unavoidable.

If the narrow surface would require exposing internals, STOP and report
rather than widening it.

## Semantics (director's rulings)

1. A declining guard means **that rule does not apply at that position** —
   indistinguishable from the pattern not matching there.
2. String replacement is **first-match-wins**, scanning left to right, so at a
   given position the rules are tried **in list order** and the first whose
   pattern matches *and* whose guards accept is used. A decline advances to
   the next rule at the same position; if every rule declines, no replacement
   happens at that position and the scan continues from the next character —
   exactly as if no rule had matched there.
3. Guards evaluate with the match bindings substituted, accepting only exact
   `True`, in the guarded body's option scope.
4. `StringCases` omits an occurrence whose guard declined; it must not collect
   it.

## Context pointers

- `5ab6afc5`, `efd6dc80`, `5418c060`, `b94a5a4a` — the four previous steps.
  Read `b94a5a4a` first; its lane-selection-once-per-call shape is the model.
- `crates/rum-stdlib/src/strings/patterns.rs` and
  `strings/patterns/search/rules.rs` are the starting points for the rule
  parser and the consumers — **find the full consumer set yourself**;
  `StringReplace`, `StringCases`, and whatever else takes rules. A consumer
  missed here is a defect shipped.
- `crates/rum-stdlib/src/strings/patterns/matcher/split_trim.rs:165` and
  `matcher/literal.rs` hold the literal fast paths that must stay untouched.
- Note for reading guard-effect counters: a `Rule` evaluates its rhs at
  construction, so a guard written with `->` has already run once before the
  consumer sees it and arrives with its condition reduced. Use `RuleDelayed`
  to observe guard evaluation alone.

## Deliverable

- String rules split their guards at parse time, for both rule heads, across
  literal patterns, string-expression patterns, and pattern-bound forms.
- The four rulings above, implemented and tested.
- **Wholly unguarded rule sets keep their current fast paths byte-identical**,
  including the single-literal path. The guarded lane is selected **once per
  rule set**, never per character position or per occurrence.
- Suspension inside a guard resumes at the same rule, the same character
  position, and the same accumulated output, without replaying earlier guards
  or their effects.
- Delayed bodies (`:>`) stay unevaluated until after the guard accepts.

### Tests

Exact values, never just absence of error:

- `StringReplace` with accepting and declining guards, both rule heads,
  literal and pattern rules.
- Rule lists: a declining rule handing off to a later one at the same
  position, and the reverse order.
- Every rule declining at a position: the character is left alone and the scan
  continues, including a case where a later position does replace.
- `StringCases` omitting a declined occurrence.
- Overlapping and adjacent matches with guards, so position accounting is
  pinned.
- Unicode: a guard declining on a multi-byte character must not split it or
  shift subsequent positions.
- Empty-pattern and zero-length-match edge cases already covered by the
  existing suites, now with a guard.
- Non-boolean guard results decline.
- A guard that suspends resumes correctly — assert a counter.
- Guard evaluation count: exactly once per (rule, position) candidate tried,
  with a `RuleDelayed` counter.
- Whatever you determine about `StringSplit`, pinned by a test.
- Wholly unguarded behavior byte-identical — existing suites are the
  assertion.

### A/B

`--scale 5`, same-binary noise floor reported **first**, then base versus
candidate:

```
string-repeat-small       character-range-small
replaceall-simple         cases-wide-no-match
matcher-single-tiny       symbol-ownvalue-read
```

Add stable labels for: an unguarded single-literal `StringReplace`, an
unguarded pattern `StringReplace`, a guarded hit, and a guarded
decline-with-fallback. The first two are the ones that must not move.

Name the untouched controls; never the suite median. In the four previous
steps the same-binary spread reached 3.8%, 8.1%, 7.2% and 4.2%, and two
flagged movements evaporated on re-measurement — a movement smaller than your
measured floor is not a regression, and if it is within a third of a
percentage point of the floor, re-measure the pair before concluding.

## Scope

- Files you MAY modify: the string pattern/rule modules under
  `crates/rum-stdlib/src/strings/`; the narrow runtime export
  (`definitions/rhs_guards.rs` and whatever module re-exports it);
  stdlib and runtime tests; `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported**, each file justified.
  If a site needs a real decision rather than a mechanical fix, STOP and
  report.
- **The general accept/stop criterion**: *any behavior change that occurs only
  when a guarded rule exists is legitimate; any behavior change observable
  with no guarded rule present is a stop.*
- MUST NOT touch: `OwnValues`, the definition tables, generic rewrite rules,
  `ReplacePart`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`, `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **String scanning is per character.** A guard check paid per position when
   no guard exists is a rejection; select the lane once per rule set.
2. **Do not turn general pattern matching into a literal-only contract**, and
   do not allocate a new pattern structure per replacement step.
3. **Operate on character boundaries**, never byte indices, when a guard
   changes control flow.
4. **Guards run user code** — preserve the suspension continuation shape and
   the accumulated output across a suspension.
5. **A decline is not an error** and must not emit a message.
6. **Reuse the substrate**; a second guard evaluator in stdlib is worse than
   the bug.
7. **Counters before timings.**
8. **Stop-loss is real.** If a step fails twice for the same reason, STOP and
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
2. **The exported surface**: exactly what became visible outside
   `crate::runtime`, and why each item was unavoidable.
3. The string-rule consumer inventory you found, and what you determined about
   `StringSplit`.
4. Verification — each command with verbatim output.
5. The exact-value test matrix, including Unicode and position accounting.
6. The A/B — counters first, then rows, with the same-binary noise floor and
   the controls named.
7. Stop-loss / open questions.
