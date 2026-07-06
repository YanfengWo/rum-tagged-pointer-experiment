# Brief 166 — Strings usability fixes 4a: text-shaping options and forms

Implementation brief. Unit-1 fix batch 4a of the Usability Pass
(tracker `docs/dev/usability-pass.md`, triage item 4; evidence E8/E10
and code citations in `docs/dev/usability-pass/strings-findings.md`).
Five items, all opt-in: every existing accepted call stays
byte-identical; only currently-rejected/inert forms gain meaning, plus
new options with defaults equal to today's behavior.

## Changes

1. **`StringWrap` gains `"PreserveNewlines" -> False`** (rum-native).
   With `True`, existing line breaks are kept and wrapping applies per
   paragraph/line instead of collapsing them (E8:
   `StringWrap["alpha\n\nbeta", 20]` → `"alpha beta"` today — that
   default stays). Define and document the `True` semantics precisely
   in the catalog description: wrap each line independently, preserve
   blank lines verbatim.
2. **`StringIndent` gains `"BlankLines" -> False`** (rum-native). With
   `True`, blank and whitespace-only lines are indented too (today
   they are skipped, E8 — that default stays).
3. **`AlignColumns` gains `"Separator" -> "  "`** (rum-native; the two
   spaces are the current hard-coded literal in
   `strings/scripting/text_shape.rs`). Any string value is accepted.
4. **`StringRiffle` flat-list brace form** (WL-documented):
   `StringRiffle[{"a", "b", "c"}, {"(", ", ", ")"}]` → `"(a, b, c)"`.
   Today a third flat-list argument form is rejected. Implement the WL
   `{left, sep, right}` form for flat lists; if the nested-list
   two-separator form (`StringRiffle[rows, outer, inner]`) is missing
   too, add it in the same style — check what exists first and report;
   existing accepted forms byte-identical.
5. **`StringPadLeft`/`StringPadRight` list padding** (WL-documented):
   accept a LIST of strings as the padding spec, cycled to fill, e.g.
   `StringPadLeft["x", 4, {"0", "1"}]` → `"010x"` (cycle starts at the
   pad's left edge; document the phase rule you implement in the
   catalog description and pin it with tests — including a pad string
   of width >1 in the list and a truncation case). Existing
   single-string padding byte-identical.

Naming note: rum-native options use string names (`"PreserveNewlines"`,
`"BlankLines"`, `"Separator"`) per the existing rum-native convention;
the WL names gain no new options, only their documented argument forms.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{StringWrap["alpha\n\nbeta", 20], StringWrap["alpha\n\nbeta", 20, "PreserveNewlines" -> True], StringIndent["alpha\n\nbeta", "> "], StringIndent["alpha\n\nbeta", "> ", "BlankLines" -> True], AlignColumns[{{"a","b"},{"long","c"}}], AlignColumns[{{"a","b"},{"long","c"}}, "Separator" -> " | "], StringRiffle[{"a","b","c"}, {"(", ", ", ")"}], StringPadLeft["x", 4, {"0","1"}], StringPadRight["x", 4, {"ab"}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); defaults
identical to today (`"alpha beta"`, `"> alpha\n\n> beta"`, two-space
alignment); the new forms produce the documented outputs (paste them;
the `"PreserveNewlines"`/`"BlankLines"` outputs must show the
preserved/indented blank line; `"(a, b, c)"`; `"010x"`; `"xaba"` or
your documented truncation — state the rule).

## Scope

- MAY modify: `crates/rum-stdlib/src/strings/scripting/**`,
  `crates/rum-stdlib/src/strings/text/**`,
  `crates/rum-stdlib/src/strings.rs`,
  `crates/rum-stdlib/src/catalog/strings.rs` (descriptions),
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  the options-defaults table in
  `crates/rum-runtime/src/runtime/options/defaults.rs` (register the
  three new option defaults), `docs/reference/builtins.md`
  (regenerated if descriptions changed).
- MUST NOT touch: other families, the guide, `TODO.md`, the tracker,
  the findings file, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Every currently-accepted call byte-identical; the changes only give
  meaning to currently-rejected forms and add options whose defaults
  reproduce today's behavior exactly.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; the documented semantics you chose for
   `"PreserveNewlines"` and the padding cycle/truncation phase rule;
   what `StringRiffle` forms existed before.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
