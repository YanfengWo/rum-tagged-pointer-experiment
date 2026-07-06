# rum Language Guide

This guide is the hand-written introduction to `rum`.

It teaches the language and the Rust embedding surface from zero, using the
current implementation as the contract.

The generated builtin reference is the companion contract:

- [Builtin reference](../reference/builtins.md)
- [Disclaimer and project boundary](../../DISCLAIMER.md)

Read this guide when you want to understand how to write source code, evaluate
expressions, define rewrite rules, control resources, or embed a session in a
Rust host.

Read the reference when you need a per-builtin entry with attributes,
capability class, summary text, and any implementation limit note.

## Status

The examples in these chapters are current-build transcripts.

They are not compatibility claims for another language, system, protocol, file
format, or front end.

`rum` is its own symbolic term-rewriting language and runtime. Its syntax uses
familiar symbolic notation, but the documented behavior is the behavior in this
repository.

If the guide and reference disagree, treat that as documentation drift. The
generated reference is derived from the standard-library catalog, while this
guide explains how the pieces fit together.

## Chapters

1. [Getting Started](./getting-started.md)
2. [Syntax](./syntax.md)
3. [String Patterns](./string-patterns.md)
4. [Structured Data](./structured-data.md)
5. [Evaluation](./evaluation.md)
6. [Patterns And Rules](./patterns-and-rules.md)
7. [Scoping And Contexts](./scoping-and-contexts.md)
8. [Resource Limits And Sandbox](./resource-limits-and-sandbox.md)
9. [Embedding](./embedding.md)
10. [Design Boundaries](./design-boundaries.md)

The order is intentional.

Start with command-line evaluation, then syntax, then string patterns, then
structured data, then evaluation. Patterns and rules depend on that evaluation
model. Scoping and contexts explain how names are resolved and persisted.
Resource limits and sandboxing explain how callers bound work. The embedding
chapter then shows the Rust host facade.

## Who This Is For

Use this guide if you are:

- writing `rum` source by hand
- writing rewrite rules
- writing packages that are loaded with `Get` or `Needs`
- embedding `rum` through `rum-host`
- deciding which resource and capability policy a host should install
- checking whether a behavior is inside the current documented surface

The guide is also useful for contributors, but it is written as user-facing
language documentation, not as an implementation walkthrough.

## How To Read Examples

Examples are shown as shell transcripts.

The input line is the command that was run from the repository root. The output
that follows is the exact output printed by the current build.

For example:

```text
$ cargo run -q -p rum-cli -- --eval 'Total[{1, 2, 1/2}]'
7/2
```

The command above evaluates one expression with the CLI.

The displayed result is the rendered final expression. Runtime messages, when
present, are shown on following lines.

Some examples use `/tmp/rum-guide-*` paths. They are ordinary temporary files
created by the transcript command and are not part of the repository.

## Guide Scope

The guide covers:

- atoms and expression syntax
- sugar and canonical runtime heads
- definitions and fixed-point evaluation
- attributes and held arguments
- messages and controlled failure
- patterns, rules, and replacement
- lexical and dynamic scoping
- contexts, packages, and session persistence
- resource limits, cancellation, timeout, and capability policy
- the public `rum-host` facade

The guide does not try to catalog every builtin. Use the generated reference
for that.

## Contract Sources

The guide is grounded in these repository documents:

- [Lowering canonical forms](../lowering-canonical-forms.md)
- [Runtime evaluation audit](../runtime-evaluation-audit.md)
- [Host adapter plan](../host-adapter-plan.md)
- [Host embedding and hardening](../host-embedding-and-hardening.md)
- [Testing strategy](../testing-strategy.md)
- [v0.1 acceptance boundary](../v0.1-acceptance-boundary.md)

The guide intentionally stays at the language and host-contract level. Kernel
module layout, internal matcher IR, and implementation-specific helper APIs are
not part of this surface.

## Naming

Names with a trailing backtick are context names, such as `` Global` `` or
`` System` ``.

Names with an embedded backtick are fully qualified symbols, such as
`` Demo`value ``.

Unqualified names are resolved against `$Context` and `$ContextPath`.

The context chapter explains the details.

## Reference Links

Builtin links point into [the generated reference](../reference/builtins.md).

Common entries:

- [`Set`](../reference/builtins.md#set)
- [`SetDelayed`](../reference/builtins.md#setdelayed)
- [`Rule`](../reference/builtins.md#rule)
- [`RuleDelayed`](../reference/builtins.md#ruledelayed)
- [`MatchQ`](../reference/builtins.md#matchq)
- [`ReplaceAll`](../reference/builtins.md#replaceall)
- [`Module`](../reference/builtins.md#module)
- [`Block`](../reference/builtins.md#block)
- [`With`](../reference/builtins.md#with)
- [`Get`](../reference/builtins.md#get)
- [`Needs`](../reference/builtins.md#needs)
- [`DumpSave`](../reference/builtins.md#dumpsave)
- [`TerminatedEvaluation`](../reference/builtins.md#terminatedevaluation)

## Reporting Drift

If you find an example that no longer produces the shown output, rerun the
example first.

If the output really changed, update the guide and, if needed, the reference or
tests in the same change.

Do not weaken a test or a documented behavior just to make a failing example
pass. Surface the mismatch as a contract question.
