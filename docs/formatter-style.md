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

The current implementation is still a token-stream formatter with
syntax-guided layout hints. The approved next architecture step is to replace
that layout-hint engine with a true document-IR pretty-printer while keeping
the current trivia-aware output behavior.

## API

`FormatterOptions` currently exposes:

- `trailing_newline`
- `line_width`

The default `line_width` is `40`.

The CLI and REPL `:format` command still use the default formatter options.

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

## Next Formatter Milestone

The next formatter milestone is no longer "add another handful of layout
heuristics".

It is to promote the formatter from the current syntax-guided token-stream
prototype to a true document-IR pretty-printer, while keeping the existing
comment-preservation behavior and regression corpus intact.
