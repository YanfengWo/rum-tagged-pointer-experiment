# Design Boundaries

This chapter states deliberate limits.

Some are product boundaries.

Some are current implementation limits.

All are part of writing correct `rum` code today.

## Exact Numbers

`rum` has exact integers and exact rationals in the current language boundary.

There is no floating-point atom and no host typed floating-point value.

Decimal literals lower to exact rationals.

```text
$ cargo run -q -p rum-cli -- --eval '{1.25, MachineNumberQ[1.25]}'
List[5/4, False]
```

The decimal input becomes `5/4`.

`MachineNumberQ` returns `False`.

This is intentional. It keeps arithmetic deterministic inside the current
documented core.

## Arithmetic Scope

Arithmetic builtins operate on the exact-number subset they implement.

Several number-theory and combinatorics builtins use integer-work budgets.

Large exact-integer work can return
`TerminatedEvaluation["IntegerWorkLimit"]`.

The reference includes limit notes for selected builtins such as
[`Prime`](../reference/builtins.md#prime),
[`PrimePi`](../reference/builtins.md#primepi), and
[`Factor`](../reference/builtins.md#factor).

## Polynomial Helpers

Polynomial helpers are useful but bounded by their current implementation.

`Factor` currently handles rational-coefficient polynomial expressions and
extracts monomial factors and recursively discovered linear factors.

```text
$ cargo run -q -p rum-cli -- --eval 'Factor[x^3 - x]'
Times[x, Plus[-1, x], Plus[1, x]]
```

Check the generated reference before treating a symbolic algebra helper as a
complete algebra system.

## String And Pattern Scope

String builtins cover a useful first-pass string subset.

For example, `StringSplit` supports whitespace and exact delimiters in the
current reference entry.

Pattern matching is useful for rule authoring, replacement, and traversal, but
it is still a documented subset.

Known pattern limits include the match-depth guard and the `Flat` plus
`Orderless` plus `Optional` corners listed in the patterns chapter and TODO.

## Replacement Scope

`Replace`, `ReplaceAll`, and `ReplaceRepeated` are the primary replacement
tools.

`ReplaceList` is currently described by the reference as first-pass root
replacement results across a rule list, with optional max-result limiting.

If you need broad replacement-list behavior, write tests against the exact
current behavior before relying on it as a public contract.

## DumpSave Is Textual

`DumpSave` writes a textual source image.

It is not a binary session image.

That is why the file can be inspected with ordinary text tools and restored
with `Get`.

The host snapshot APIs also use a UTF-8 source image.

A binary snapshot format is deferred work.

## Filesystem Is A Capability

The kernel ships no general file utilities.

The remaining filesystem builtins are not removed from the binary for
sandboxing.

Instead, each builtin has a capability class and the host chooses a
`CapabilityPolicy`.

The filesystem surface is the module system (`Get`, `Needs`, `FindFile`) and
session persistence (`Save`, `DumpSave`) only; a broader filesystem surface is
future downstream work.

`HostConfig::sandboxed()` allows only `Pure`.

That makes sandboxing a dispatch policy, not a source-text scanner.

## Determinism

`rum` makes two sandbox claims: no ambient authority by construction, and
deterministic by default.

The default sandbox policy excludes filesystem, clock, random, and diagnostic
capabilities. Evaluation results therefore depend only on session state and
source unless the host grants more capability.

Randomness follows that same boundary. `RandomInteger`, `RandomChoice`,
`RandomSample`, `BlockRandom`, `RandomUuid`, and `UuidQ` use a deterministic
session PRNG and are `Pure`. The algorithm is xoshiro256** with splitmix64 seed
expansion. A fresh session starts from default seed `0`, and `reset()` restores
that same default seed. The same seed yields the same sequence on every
platform and rum version; changing this algorithm or seed expansion would be a
documented breaking change.

`SeedRandom[n]` accepts an exact integer. The integer seed is folded by reading
little-endian magnitude bytes through splitmix64; negative seeds add a sign tag
before xoshiro256** state expansion. `SeedRandom[0]` matches the fresh-session
default stream. `BlockRandom[body]` evaluates `body` and restores the prior
PRNG state afterward.

Host snapshots are textual definition images and do not capture PRNG state.
Restoring a snapshot starts from the session configuration's current PRNG
state after reset, not from the source session's random stream position.

The session PRNG is not cryptographic. It is for reproducible symbolic scripts,
sampling, and session-local identifiers. `RandomUuid[]` returns a version-4
format UUID string from that deterministic stream; use an entropy reseed first
when global uniqueness matters. Password-grade bytes come from `RandomBytes`,
which uses the host entropy provider directly and requires `Capability::Random`.
`SeedRandom[]` also requires `Capability::Random` because it reseeds the session
PRNG from host entropy.

There is no `RandomReal` in the current exact-number boundary. A script that
needs a rational value in `[0, 1)` should draw
`RandomInteger[{0, d - 1}]/d` explicitly.

This is not a claim of functional-programming referential transparency. The
language has session state by design.

## Diagnostics Are A Capability

Diagnostic builtins are also capability-gated.

`RumProfileReset`, `RumProfileSnapshot`, `MemoryInUse`, and `MaxMemoryUsed` are
classified as `Diagnostic`.

A restricted host can hide that surface without feature-gating the binary.

## Deep Expr Trees Through Rust API

Source evaluation is protected by parser depth, evaluator recursion, and matcher
depth guards.

There is a separate host-side limitation: a Rust host must not directly build
pathologically deep `Expr` trees through the low-level Rust expression API.

Dropping a deeply nested `Expr` chain built directly by Rust can overflow the
native stack.

That route is outside normal source evaluation and outside the `rum-host`
facade.

If a host needs to manufacture very deep trees, it should use a depth-limited
builder or a dedicated explicit cleanup strategy rather than depending on
ordinary recursive drop.

## Protocols And File Formats

The current product boundary is the embeddable symbolic runtime and rule
engine.

It does not implement external notebook protocols as part of the language.

It does not implement other vendors' private binary session formats.

Adapters may exist in separate crates, but the language contract remains the
runtime, builtins, host facade, and documented source formats.

## Front-End Scope

`rum` is not a notebook UI.

It is not a plotting or visualization environment.

The core language and host facade are designed to be embedded by callers that
provide their own UI, transport, persistence policy, and trust boundary.

## Compatibility Scope

The normative behavior is the guide, the generated reference, host rustdoc, and
the tests that protect those contracts.

Do not infer behavior by comparing against another engine.

If you need behavior not documented here or in the reference, treat it as a
feature request or a contract question.

## Practical Rule

When in doubt:

- check the generated reference entry
- check whether the builtin has a limit note
- check whether the operation needs file or diagnostic capability
- run a small current-build example
- add a focused test near the crate that owns the behavior

That workflow keeps code and documentation tied to `rum`'s own behavior.
