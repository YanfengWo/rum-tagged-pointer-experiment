# Brief 243 — Builtin diagnostic-message inventory (audit; no source change)

## Goal

Produce the complete factual inventory of how builtin diagnostic messages
are emitted and catalogued today, as the evidence base for designing a
static per-builtin message-tag registry (the docs/reference should later
list a builtin's message tags from the same source that `Messages[f]`
reads). This is a **diagnostic brief**: the deliverable is one findings
document with all-rows tables and verbatim probe output. Zero source
changes.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines) first.
- Key files:
  - `crates/rum-runtime/src/runtime/session/session_helpers.rs:165` —
    `push_message(symbol, tag, text)`, the single push API (text arrives
    pre-formatted; there is no template/parameter separation);
    `:187` `runtime_message_for_non_loader_eval_error` — the second
    emission path (EvalError variant → tag mapping).
  - `crates/rum-runtime/src/runtime/model/messages.rs:44` —
    `RuntimeMessage` itself.
  - `crates/rum-runtime/src/runtime/messages/` — the existing message
    machinery: `message_catalog.rs` (user-defined `f::tag = "…"` store +
    `message_tags_expr_for_symbol`), `message_format.rs`,
    `builtins_messages.rs` (`Messages` builtin),
    `builtins_messages_inspect.rs` (Information's `Messages` field),
    `message_selector_parsing.rs` (On/Off selectors).
  - `crates/rum-runtime/src/runtime/metadata.rs:287` — `BuiltinSpec`
    (name/summary/attributes/dispatch/handler/capability; const
    builders like `.protected()` are the declaration idiom).
  - `crates/rum-stdlib/src/docgen.rs` + `src/bin/generate-builtin-reference.rs`
    + `src/tests/builtin_reference_docgen.rs` — reference generator and
    its drift test.
- Acceptance is mechanical: the named tables with exact counts derived
  from commands you show verbatim, plus the named live probes.

## Scope

- Files you MAY modify: **exactly one, created new:**
  `docs/dev/message-registry-inventory.md`. Nothing else — this is a
  read-only audit of the source tree.
- Files/areas you MUST NOT touch: everything else, including all crate
  sources, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, other `docs/**`, and
  anything under `docs/dev/codex/` except appending to your own report.
- No source change means no test gate; do NOT "fix" anything you find.

## Deliverable

Write `docs/dev/message-registry-inventory.md` **incrementally as you
work** (header and table skeletons first, rows appended as you classify
each site — the file must be useful even if your run is cut short).
Sections:

1. **Table A — every direct `push_message` call site** (expect on the
   order of 114 non-test sites; report the exact count and the grep
   command that produced it). Columns:
   `# | crate | file:line | symbol arg (literal "X" / variable / fn
   parameter) | tag arg (literal "x" / computed e.g. error.tag()) |
   text shape (STATIC literal / FORMATTED with N runtime values /
   PASSTHROUGH variable) | in-shared-helper? (helper fn name or —)`.
   Classify by OPENING each site, not by guessing from the grep line.
2. **Table B — helper fan-in.** For every Table-A site that lives inside
   a shared helper (a function whose callers determine the real emitting
   builtin — e.g. `invalid_call_with_arg_form`), list the helper's
   direct callers: `helper | caller file:line | builtin symbol passed`.
   One level deep; if a caller is itself a helper, mark HELPER-NESTED
   and expand one more level, then stop and give a count.
3. **Table C — emission paths that do not go through `push_message`.**
   (a) `runtime_message_for_non_loader_eval_error`: every EvalError
   variant it maps, with the tag and message text; note which symbol the
   resulting message is attributed to. (b) Any other construction of
   `RuntimeMessage {` outside `push_message` (there are known sites in
   `debug_tools/test_report/evaluation.rs`; find them all with a shown
   grep). (c) If a loader-side sibling of the "non_loader" function
   exists, document it the same way.
4. **Section D — tag vocabulary.** Distinct tags overall; distinct
   (symbol, tag) pairs (state how you counted given dynamic symbol
   args); the generic/shared tags (`argx`, `argform`, …) with per-tag
   usage counts; for each of the top shared tags, whether its text is
   uniform across sites or varies.
5. **Section E — what the existing catalog machinery reads today.**
   For each of: `Messages[f]`, `f::tag` (MessageName read),
   `On`/`Off` selectors, `Information`'s `Messages` field, and the
   session's `inspect_message_catalog` (the `HostMessageTemplate` seam) —
   state what store it consults and whether builtin `push_message`
   emissions ever enter that store. Then confirm the headline gap with a
   LIVE probe on the release CLI (build it once): evaluate a `Set` call
   that fires `Set::argx`, then evaluate `Messages[Set]`, and paste both
   outputs verbatim. Also probe `Messages[General]` and one user-defined
   template round-trip (`f::tag = "…"; Messages[f]`) so the report shows
   what DOES work.
6. **Section F — registry attachment facts (no design).** `BuiltinSpec`'s
   current fields and const-builder set; how catalog entries are declared
   (one example each from the stdlib catalog and the kernel
   builtin_catalog); where docgen pulls per-builtin data from and the
   drift test's name/location; the mirror-lockstep mechanism brief-210
   used (so a later design knows every place a new per-builtin static
   field must be threaded).
7. **Stop-loss / open questions** — anything ambiguous, with file:line.

## rum / Rust pitfalls (read twice)

1. This is a read-only audit: no timing numbers, no source edits, no new
   tests. The evidence IS the deliverable.
2. Counts must come from commands shown verbatim in the report AND
   cross-checked against at least three hand-opened sites each (a regex
   that under-captures is a known failure mode in this repo).
3. Probes run against the release CLI: `cargo build --release -p rum-cli`
   once, then `target/release/rum-cli --eval '…'` (feed stdin from
   /dev/null). Paste probe output verbatim; if a probe surprises you,
   record the surprise — do not re-shape the probe until it agrees.
4. Write the findings file incrementally (skeleton first, rows as you
   go). If you must stop, the partial file plus a stop-loss note is a
   valid outcome.

## Verification (MANDATORY)

No test gate (no source change). Verification =
- the findings file exists with all sections;
- the Table-A row count equals the grep total you show (state both);
- the Section-E probes' verbatim CLI transcripts are embedded in the file.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the new
  file in the working tree; Claude runs the authoritative gate and
  commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.
- Full machine access, but touch NOTHING outside this repo, no network.
  The only commands you run are file reads/edits within Scope, `grep`/
  `rg`, `cargo build --release -p rum-cli`, the release `rum-cli` for
  Section-E probes, and `python3` for counting/parsing.
- Testing Policy is a hard line (`AGENTS.md`): no comparisons against
  other vendors' engines anywhere, including in prose.
- No new `unsafe` (trivially satisfied: no source change).
- Minimal footprint: one new file, nothing else.

## Report format (your final message)

1. "Diagnostic only, no source change" + the one file written.
2. Verification — the exact grep/count lines and the probe transcripts,
   verbatim.
3. The finding — the shape of the gap in two paragraphs: how many sites,
   how many distinct (symbol, tag) pairs, what fraction of texts are
   static vs formatted, and what the catalog machinery would need to
   know about (facts, not a design).
4. Stop-loss / open questions.
