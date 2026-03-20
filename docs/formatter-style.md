# Formatter Style Notes

This document records the current formatter behavior in `rum-syntax`.

It describes the formatter as it exists today; it is not a promise that every
rule is final.

## Status

The formatter now has a stable prototype surface for:

- canonical spacing across the currently supported syntax subset
- preserved outer trivia and interior comment trivia
- width-aware multiline layout for calls, lists, binary expressions, and binary
  chains
- normalized hanging layouts for multiline binary chains
- regression coverage for mixed inline/block comment placement in nested forms
- a document-IR pretty-printer built around `Doc`, `Break`, `SoftLine`, and
  `Group`

The current implementation is no longer a token-stream layout-hint formatter.
It now builds a document IR directly from parsed syntax and trivia, uses a
structured group planner for containers and binary chains, and renders groups
with current-column-aware fit checks.

## API

`FormatterOptions` currently exposes:

- `trailing_newline`
- `line_width`

The default `line_width` is `40`.

The CLI exposes width override through `--format ... --line-width N`.
The REPL exposes width override through `:format [--line-width N] EXPR`.

## Current Layout Rules

### Calls and Lists

- Calls and lists stay inline when they fit comfortably.
- They actively expand when they already contain top-level multiline structure.
- They actively expand when they have at least four top-level items.
- They also expand when they have at least two top-level items and their inline
  form exceeds the configured `line_width`.
- Nested multiline children do not automatically force the parent container to
  expand unless the parent's own top-level layout triggers one of the rules
  above.
- Nested groups are fitted relative to the column where they appear, not only
  as if they started at column 0.

### Binary Expressions

- Single binary expressions expand when their inline form exceeds
  `line_width`.
- Binary chains expand as a unit: once a chain becomes multiline, later chain
  items are normalized into a hanging layout.
- Right-associative chains such as assignment-style forms follow the same
  width-aware hanging strategy.

### Comments

- Outer comments are preserved.
- Leading comments on multiline items render as their own indented lines.
- Inline comments attached to later items in a chain or multiline container are
  promoted to standalone comment lines when that item moves onto its own line.
- Inline comments that remain attached to already-emitted left-hand material
  stay inline.
- Closing delimiters dedent after multiline comment blocks.

## Stability Expectations

The formatter now has:

- targeted regression tests for nested forms, width-driven layout, and comment
  placement
- an idempotence regression corpus that asserts
  `format(format(source)) == format(source)` for representative inputs
- regression coverage for current-column-aware fitting of nested groups

## Known Limitations

- Width fitting still uses `flat_width` estimates derived from syntax display,
  not the exact flat rendering of the document tree.
- As a result, very long inline comments or string literals that need escaping
  can remain inline longer than the configured `line_width` would suggest.
- CLI and REPL both expose `line_width`, but they currently parse that option
  with separate helper implementations rather than one shared formatter-UX path.

## Next Formatter Milestone

The current formatter milestone is now architectural consolidation rather than
another formatter rewrite.

The remaining work is to keep refining the pretty-printer implementation:

- replace estimated `flat_width` planning with true flat-doc measurement
- add width-regression coverage for long inline comments and escaped strings
- fix syntax-display string escaping so formatter planning can rely on
  round-trippable surface rendering
- unify formatter option parsing across CLI and REPL
- continue splitting formatter internals into smaller planner, builder, render,
  and test modules
- keep expanding the regression corpus as new syntax support lands
