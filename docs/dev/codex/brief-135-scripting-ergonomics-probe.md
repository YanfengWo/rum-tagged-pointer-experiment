# Brief 135 — Scripting-ergonomics probe + string-pattern surface audit

DIAGNOSTIC brief — **no source changes**. Opens the stdlib-expansion mainline
(`TODO.md` §Stdlib Expansion For Scripting). The deliverable is evidence: a
friction list from real scripting tasks and a support map of the
string-pattern language's undocumented surface.

## Goal

Two evidence artifacts, verbatim in the report:

1. **Six embedded-scripting tasks written in current rum**, each either
   working end-to-end (script + real output) or blocked with the exact
   missing piece named. Friction notes per task.
2. **A string-pattern support table**: which pattern constructs the string
   functions actually accept today, versus what the reference documents.

## Part 1 — the six tasks

Write each as a self-contained `.wl` script under `/tmp/rum-135/`, run with
`cargo run -q -p rum-cli -- --file ...` (use `scripts/run-bounded.py` for
anything possibly slow). Choose idiomatic rum; when something is missing,
implement a workaround IN rum if reasonable (that is evidence of friction,
not a failure) and note the cost.

1. **Log filter/summary**: given a multi-line log string (define it inline),
   extract lines matching a level ("ERROR"), parse out a timestamp field and
   a message, produce counts per hour bucket and the top-3 most common
   messages.
2. **Text transform**: turn an inline markdown-ish bullet list (nested `-`
   items) into (a) a numbered flat list and (b) an indented plain-text tree.
3. **Tabular munging without Import**: an inline CSV-like string (header +
   rows, quoted fields with commas inside) → list of associations → filter,
   group by a column, aggregate (sum/mean as exact rationals), render an
   aligned text table.
4. **Config validation**: a nested association (inline) validated against
   rules (required keys, type predicates, value ranges, defaults filled in),
   producing either the normalized config or a list of readable errors.
5. **Report generation**: combine task 3's aggregates with `StringTemplate`
   /`TemplateApply`-style substitution into a formatted multi-line report.
6. **Word frequency**: given a paragraph, tokenize, normalize case, strip
   punctuation, count, and print the top 10 with aligned columns.

Per task record: lines of code; missing builtins (what you reached for that
does not exist); awkward idioms (exists but clumsy — quote the clumsy
snippet); any surprising behavior (paste input/output); wall time if
noticeable.

## Part 2 — string-pattern support map

Systematically probe `StringCases`, `StringReplace`, `StringMatchQ`,
`StringSplit`, `StringPosition`, `StringContainsQ` with each construct:

- literals; `~~`-style concatenation if it parses; alternatives `|`
- `_`, `__`, `Repeated`/`..`, `RepeatedNull`/`...`
- character classes: `DigitCharacter`, `LetterCharacter`,
  `WhitespaceCharacter`, `Whitespace`, `WordCharacter`, `PunctuationCharacter`,
  `StartOfString`/`EndOfString`/`StartOfLine`/`EndOfLine`, `WordBoundary`
- `Except[...]`, `Shortest`/`Longest`, named pattern binding (`x_` /
  `Pattern`) inside string patterns with rule RHS use
- `NumberString`; case-insensitivity options if any

For each cell: WORKS (paste one probe + output) / FAILS-inert / FAILS-error /
NOT-PARSED. Then compare against `docs/reference/builtins.md` and the guide:
list constructs that work but are uncataloged (the dark surface — e.g.
`DigitCharacter` is known to work while absent from the reference), and
constructs documented but limited.

## Part 3 — ranked gap list (your verdict)

Merge Parts 1-2 into a ranked list: for each gap — name, evidence (which
task/probe), proposed shape (new builtin with a rum-native name / depth fix /
documentation-only), and a rough size (S/M/L). Rank by scripting value, not
by ease. Candidate frames to confirm or refute with the evidence: in-language
JSON codec, string-pattern contract + depth, date/time folk subset,
collection odds-and-ends (`Tally`, `MaxBy`, `Riffle`, `GatherBy`).

## Scope

- Files you MAY modify: NONE (scripts live under `/tmp/rum-135/`).
- No source, doc, or test changes of any kind.

## Verification

Diagnostic: the evidence IS the deliverable. Every claim in the tables must
carry a verbatim probe (input + output). Paste each task's full script and
its real output.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Everything runs with the current build; no fabricated outputs.
- Probe budget: single runs are fine (nothing here is a timing gate); keep
  the total under ~150 CLI invocations by batching probes into scripts.
- Stop-loss: if a task cannot be finished after two honest attempts, record
  it as BLOCKED with the missing piece — that is a valid result.

## Report format (your final message)

1. Per-task: script (verbatim), output (verbatim), friction notes.
2. The string-pattern support table + dark-surface list.
3. The ranked gap list with evidence links and sizes.
4. Anything that looked like a bug (exact input/output).
