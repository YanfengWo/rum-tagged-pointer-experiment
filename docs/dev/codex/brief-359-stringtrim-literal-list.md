# brief-359 — `StringTrim` with a literal list falls off its fast path

## Goal

`StringTrim[s, "1"]` has a fast path. `StringTrim[s, {"1", "2"}]` does not, and
pays 13-16x more for the same shape of work. Measured on `e09e5671`:

| length | `{"1", "2"}` | `"1"` | whitespace | ratio |
|---:|---:|---:|---:|---:|
| 4 000 | 200.8 us | 13.4 us | 10.1 us | 15.0x |
| 8 000 | 262.2 us | 20.4 us | 18.5 us | 12.9x |
| 16 000 | 521.7 us | 36.1 us | 35.4 us | 14.5x |
| 32 000 | **2.17 ms** | 134.5 us | 59.5 us | 16.1x |

Both forms are slightly superlinear (8x length costs about 10x), so the defect
is the constant factor, not the complexity class — an earlier survey called
this a "quadratic suffix helper" and that reading is wrong.

Give the exact-literal-list case a fast path, and leave the general pattern
path exactly as it is.

## Context pointers

- The dispatcher, the existing single-literal fast path, and the general
  matcher live under `crates/rum-stdlib/src/strings/patterns/`
  (`search.rs`, `matcher/split_trim.rs`, `matcher/literal.rs`,
  `matcher/engine.rs`). Line numbers from earlier surveys are stale — locate
  them yourself.
- `docs/reference/builtins.md` documents `StringTrim`; the generated text is
  normative. **Establish the current semantics first** and write them down in
  your report before changing anything:
  - how much is trimmed from each end (once, or repeatedly);
  - which alternative wins when several match at the same position — longest,
    first in list order, or something else;
  - what happens with overlapping literals, equal-length alternatives, an
    empty-string alternative, and a literal longer than the input;
  - whether the one-literal form and the one-element-list form agree today.

  Those answers are the contract to preserve. If the one-literal and
  one-element-list forms **disagree** today, that is a finding — report it and
  stop rather than picking a winner.

## Deliverable

- A fast path taken only when every supplied trim pattern is an exact string
  literal, doing work proportional to what is actually trimmed rather than to
  the whole input.
- The general pattern path unchanged, including `Whitespace`, character
  classes, alternatives containing non-literals, and mixed lists.
- The literal fast path selected **once per call**, not per position.

### Tests

Exact values, never absence of error:

- The semantics you established, each pinned: repeated versus single trim,
  the alternative-selection rule, overlapping literals, equal-length ties,
  empty-string alternative, literal longer than input, no match at either end,
  match at one end only.
- One-literal and one-element-list parity.
- Unicode: multi-byte literals at both ends, and a literal that is a prefix of
  a multi-byte sequence must not split a character.
- Threading over a list of strings.
- Mixed list (literal plus pattern) still takes the general path and returns
  the same answers as today.
- A wide input (100 000 characters) trimmed by a literal list completes
  quickly — the constant factor is the defect, so a wide case is the test that
  demonstrates the repair.

### A/B

`--scale 5`, same-binary noise floor **first**, then base versus candidate.
Add stable labels for the exact-literal-list form at two lengths and keep the
single-literal and whitespace forms as same-tier neighbors — those two must
not move.

Same-tier neighbors:

```
string-repeat-small        character-range-small
replaceall-simple          cases-wide-no-match
matcher-single-tiny        symbol-ownvalue-read
```

Name the untouched controls; never the suite median. A movement smaller than
your measured floor is not a regression; if a required row lands within a
third of a percentage point of the floor, re-measure the pair.

## Scope

- Files you MAY modify: the string pattern modules under
  `crates/rum-stdlib/src/strings/patterns/`; stdlib tests;
  `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported**, each file justified.
- **The general accept/stop criterion**: *any answer change is a stop.* This
  brief changes cost, not answers.
- MUST NOT touch: the guarded string-rule lanes landed in `79afc6d8`, the
  runtime crate, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Do not turn general pattern matching into a literal-only contract.** The
   fast path is an addition; the general path keeps every behavior it has.
2. **Operate on character boundaries**, never byte indices.
3. **Do not allocate a new pattern structure per trim step.**
4. **Match the current tie-breaking exactly** — longest, order, whatever you
   established. A "more sensible" rule is an answer change and therefore a
   stop.
5. **Counters before timings**: an allocation counter showing the per-call
   rescans disappear is stronger than any timing.
6. **Stop-loss is real.** If the same step fails twice for the same reason, or
   if the exact-list label does not improve materially, STOP and report.

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
  behavior only.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. The current semantics you established, stated as the contract, and where
   you established each answer.
2. Whether the one-literal and one-element-list forms agree today.
3. What changed — per file.
4. Verification — each command with verbatim output.
5. The exact-value test matrix.
6. The A/B — counters first, then rows at two lengths, with the noise floor
   and the controls named.
7. Stop-loss / open questions.
