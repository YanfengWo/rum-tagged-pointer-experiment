# Formatter Style Notes

This document records the current formatter behavior in `rum-syntax`.

It describes the formatter as it exists today; it is not a promise that every
rule is final.

## Status

The formatter is no longer a token-stream prototype. The current implementation
is a trivia-aware document-IR pretty-printer built around `Doc`, `Break`,
`SoftLine`, and `Group`.

The current implemented surface includes:

- canonical spacing across the currently supported syntax subset
- preserved outer trivia and interior comment trivia
- width-aware layout for calls, lists, binary expressions, and binary chains
- normalized hanging layouts for multiline binary chains
- current-column-aware fitting of nested groups
- regression coverage for nested forms, comments, width-driven layout, and
  idempotence

## API

`FormatterOptions` currently exposes:

- `trailing_newline`
- `line_width`

The default `line_width` is `40`.

The CLI exposes width override through `--format ... --line-width N`.
The REPL exposes width override through `:format [--line-width N] EXPR`.

Formatter-option parsing is now shared across CLI and REPL so width handling
and error messages stay aligned.

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
- regression coverage for long inline comments and escaped string literals in
  width-driven layouts
- an idempotence regression corpus that asserts
  `format(format(source)) == format(source)` for representative inputs
- current-column-aware fit coverage for nested groups

## Known Limitations

- The formatter only covers the currently implemented syntax subset. As parser
  and evaluator surface area grows, formatter coverage will need to expand with
  it.
- Width and grouping policy are intentionally simple and local. The formatter
  is good at the current call/list/binary subset, but it is not yet trying to
  solve broader global layout optimization across every future construct.
- Formatter internals are still larger and flatter than the strengthened code-
  organization rules ideally allow, especially inside `formatter.rs`.

## Next Formatter Milestone

The current formatter milestone is continued consolidation rather than another
architectural rewrite.

The remaining work is to:

- keep splitting formatter internals into smaller planner, builder, render, and
  test modules
- continue expanding formatter coverage as more syntax/operators are added
- keep extending the regression corpus as the supported syntax surface grows
