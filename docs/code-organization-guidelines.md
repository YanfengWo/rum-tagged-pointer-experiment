# Code Organization Guidelines

This document defines the code organization standard for `rum`.

The goal is to keep the codebase maintainable, auditable, and friendly to
incremental compilation without turning the repository into an overly fragmented
set of tiny files.

## Development environment note

At the moment, `rum` is developed inside a parent portable toolchain workspace.
Because of that, routine local development commands should prefer the portable
entrypoints exposed by the parent workspace rather than assuming global tools.

In practice, this means that for the current development setup we should prefer
commands such as:

- `cmd /c "call ..\scripts\activate.cmd && cargo test --workspace"`
- `cmd /c "call ..\scripts\activate.cmd && cargo fmt --all"`
- `cmd /c "call ..\scripts\activate.cmd && git status"`

This is an environment-adaptation rule for the current repository location, not
part of the long-term product design. During release hardening or packaging, we
should explicitly review and remove development-only wrappers, parent-relative
entrypoints, and any assumptions that `rum` must be run from the surrounding
portable workspace.

## Core principles

### 1. Split source files for maintainability first

Source files should be split when a file becomes difficult to read, review, or
audit as a single unit.

As a working rule:

- each source file should preferably stay under roughly `10k` characters
- small, cohesive files are preferred over large mixed-responsibility files
- related logic that must be read together may remain in one file even if it is
  slightly smaller than the threshold

The `10k` guideline is a maintenance target, not a hard law. It should be used
as a prompt to reconsider structure, not as a reason to create awkward splits.

### 2. Prefer splits that align with module and compilation boundaries

When splitting code, prefer real Rust module boundaries over textual
concatenation.

In practice, this means:

- prefer `mod`-based decomposition to `include!`-based decomposition for normal
  source organization
- group code by stable responsibilities such as syntax, lowering, evaluation,
  matching, scoping, messages, or inspection
- try to isolate code that changes frequently from code that changes rarely, so
  incremental builds can reuse more compiled artifacts

The best split is usually one where the module boundary is also a meaningful
concept boundary.

### 3. Do not split too aggressively

Excessively fine-grained splitting is discouraged.

Over-splitting causes its own problems:

- navigation becomes harder
- related logic becomes scattered across too many files
- local reasoning during maintenance becomes slower
- the compile-time savings may be negligible compared with the added complexity

If a split creates many tiny files with weak conceptual separation, it is
usually not worth it.

## Decision rules

When deciding whether to split or merge code, use the following order of
judgment:

1. Does the current file still represent a clear, coherent responsibility?
2. Is the file still reasonably easy to review and audit in one pass?
3. Would splitting it create a real module boundary that helps incremental
   compilation?
4. Would the new boundary remain understandable six months later?
5. Is the expected compile-time win meaningful enough to justify the added
   maintenance surface?

Only split when the answer is favorable across most of these questions.

## Preferred outcomes

Good outcomes:

- one file per coherent subsystem
- a module tree that mirrors runtime responsibilities
- boundaries that reduce unrelated recompilation
- files that are easy to inspect during review and debugging

Avoid:

- giant catch-all implementation files
- `include!` trees used as the default structuring tool
- arbitrary size-based splitting with no responsibility boundary
- excessive micro-modules whose only benefit is theoretical compile-time gain

## Audit note

For maintenance and audit purposes, any future structural refactor should be
able to explain both of the following:

- why the split improves readability or reviewability
- why the split is neutral to positive for incremental compilation

If neither explanation is convincing, the refactor should usually not proceed.
