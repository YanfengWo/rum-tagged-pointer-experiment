# Code Organization Guidelines

This document defines the code organization standard for `rum`.

## Introduction

`rum` should be organized in a way that stays pleasant for humans to read,
straightforward to maintain, friendly to incremental compilation, and easy to
verify with focused unit tests.

Those goals imply a few practical rules of thumb:

- prefer coherent module boundaries over giant mixed-responsibility files
- avoid both oversized files and forests of tiny files
- store major branches of the codebase in feature-oriented subfolders and
  modules rather than endlessly growing flat roots
- split code in ways that improve both reviewability and incremental
  compilation, not just raw file size metrics
- design functions so important policy and normalization logic can be tested in
  isolation
- treat large-file exceptions as explicit, documented debt rather than silent
  drift

The rest of this document expands those principles into concrete rules.

## Environment Note

Routine development in the current repository should continue to use the parent
portable toolchain workspace rather than assuming global tools. The project
entry point for that setup is described in `../README.md`.

This is a local development rule, not part of the long-term product design.

## Primary Goals

### Human-readable and review-friendly code

Source files and modules should be structured so a reviewer can understand a
coherent responsibility in one pass.

### Long-term maintainability

The codebase should stay easy to refactor and audit six months later, not just
work for the current patch.

### Incremental-compilation friendliness

Module boundaries should help Rust reuse compiled artifacts and avoid
unnecessary recompilation of unrelated code.

### Unit-test friendliness

Important policy, normalization, and planning logic should be reachable through
small, focused unit tests instead of only broad end-to-end scenarios.

## File Size And Splitting Policy

The preferred target is still that a source file remain under roughly `10k`
characters. This remains a maintenance target rather than a mechanical law, but
it is now a stronger constraint than before.

Use the following thresholds:

- under `10k`: preferred range
- `10k` to `20k`: warning range; the file should be reviewed for possible
  splitting
- `20k` to `35k`: split-plan range; active work should not continue without an
  explicit TODO item stating that the file should be split
- over `35k`: default architecture debt; the file should usually be split
  immediately unless there is a written exception

Additional rules:

- if a file is still under active development and exceeds `10k`, add an
  explicit TODO item noting that the file should be split
- if a file is mostly feature-complete and exceeds the threshold, treat
  splitting as immediate cleanup rather than indefinite future debt
- if a large file is kept temporarily, document why it remains large and how it
  is expected to be split later

## Test File Guidance

Test files may be somewhat larger than implementation files because they often
benefit from keeping related scenarios together, but they are not exempt from
the same maintenance concerns.

In practice:

- prefer grouping tests by behavior family rather than piling all scenarios into
  one `tests.rs`
- once a test file becomes difficult to review in one pass, split it by feature
  or behavior area
- very large test files should be treated as structure debt in the same way as
  large implementation files, even if the threshold is reached slightly later

## Preferred Split Style

Split code along real responsibility boundaries, not arbitrary size cuts.

Good split boundaries usually look like:

- syntax vs lowering
- evaluation vs matching
- scoping vs messages
- transport vs wire codec vs backend
- planning/normalization vs rendering/mutation

Preferred patterns:

- use normal Rust `mod` boundaries rather than `include!` trees
- create feature-oriented subdirectories when a subsystem becomes large
- separate code that changes frequently from code that changes rarely when that
  boundary is meaningful

Avoid:

- giant catch-all implementation files
- arbitrary size-based splitting with no conceptual boundary
- tiny micro-modules whose only benefit is reducing a character count

## Function Design For Testability

Function boundaries should support isolated unit tests.

In practice, this means:

- separate parsing, normalization, planning, state mutation, and rendering when
  doing so improves clarity and testability
- prefer pure helpers for policy decisions and normalization steps
- avoid routines that mix validation, session mutation, and output formatting
  unless the combination is genuinely inseparable
- factor branch-heavy code so important branches can be exercised directly
  without booting the entire subsystem

Not every helper needs a dedicated test, but important policy and normalization
steps should usually be reachable without full-stack setup.

## Large-File Workflow Rules

Large files should not be treated as a passive fact of life.

Whenever a change touches a file that is already in the warning or debt range:

1. Re-evaluate whether the current file still represents one clear
   responsibility.
2. Decide whether the current patch should perform or begin a split.
3. If not splitting immediately, leave a concrete TODO note that the file
   should be split, or document the temporary exception.

This rule exists to prevent the codebase from repeatedly adding new behavior to
files that everyone already knows are too large.

## Exception Policy

Exceptions are allowed, but they must be explicit.

Acceptable temporary reasons may include:

- the subsystem is in rapid flux and a stable split boundary has not yet
  emerged
- a protocol or wire layer is mid-migration and splitting during the migration
  would create extra churn
- the current patch is intentionally limited in scope and the split is already
  recorded as near-term cleanup

When an exception is taken, the project should still record:

- why the file remains large
- what boundary is expected to be used for the future split
- whether the file is still under active development or is mostly complete

## Decision Checklist

When deciding whether to split or merge code, use this order of judgment:

1. Does the current file still represent a clear, coherent responsibility?
2. Is the file still reasonably easy to review and audit in one pass?
3. Would splitting it create a real module boundary that helps incremental
   compilation?
4. Would the split make important logic easier to test in isolation?
5. Would the new boundary remain understandable six months later?
6. If not splitting now, is the reason explicit and documented?

Only split when the boundary is conceptually real. Only defer the split when
the reason is concrete and visible.

## Preferred Outcomes

Good outcomes:

- one file per coherent subsystem
- functions whose important branches can be unit-tested without full-stack
  setup
- a module tree that mirrors runtime responsibilities
- boundaries that reduce unrelated recompilation
- files that are easy to inspect during review and debugging

Avoid:

- giant catch-all files that keep accumulating unrelated work
- repeated ad hoc parsing or normalization logic scattered across builtins
- `include!` used as the default organization strategy
- arbitrary splits that improve file size metrics but harm readability
- excessive micro-modules with weak conceptual separation
