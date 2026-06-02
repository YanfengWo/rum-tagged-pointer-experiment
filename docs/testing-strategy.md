# Testing Strategy

This document defines how `rum` tests core-language behavior after the
`CL-01` through `CL-09` mainlines have closed. The goal is not to claim full
Wolfram Language coverage. The goal is to make each supported subset explicit,
test it at the right layer, and use a local Wolfram oracle where same-name
compatibility is part of the contract.

## Compatibility Contract

For a Wolfram Language same-name function or symbol, compatibility is scoped by
[core-language-support-matrix.md](./core-language-support-matrix.md):

- `implemented`: the supported behavior should be Wolfram-compatible except
  for ordinary follow-up polish or explicitly documented divergence.
- `partial`: only the documented subset is compatibility-targeted.
  Unsupported forms are not bugs unless promoted by `TODO.md` or by a concrete
  downstream project.
- `planned`: inside the scoped kernel target but not implemented.
- `excluded`: deliberately outside the current project phase.
- `extension`: project-owned surface, tested against `rum`'s own contract
  rather than Wolfram behavior.

Do not treat accidental behavior as supported just because one test passes. If
a behavior matters, put it in the matrix or in an explicit deferred note before
depending on it.

## Test Layers

Use the lowest layer that can prove the contract being tested.

1. Crate-local unit tests
   - Own small helper invariants, planners, parser/formatter pieces, matcher
     branches, normalization functions, codec helpers, and host facades.
   - Prefer direct helper tests when they make a high-risk policy branch
     reachable without large evaluator setup.
2. Runtime regression tests
   - Own evaluator/session semantics, state mutation, messages, held
     evaluation, suspension/resume, and `rum`-native request behavior.
   - These tests may use Wolfram-shaped expectations when the behavior is
     already known, but they should not replace oracle-backed conformance for
     high-risk same-name semantics.
3. Differential conformance tests
   - Own Wolfram-oracle-comparable same-name behavior for the documented
     subset.
   - Keep oracle outputs transient. Fixtures should store inputs, metadata, and
     deliberate known-difference notes, not generated Wolfram transcripts.
4. Adapter and host-contract tests
   - Own CLI, REPL, `rum-host`, future Python host, and Jupyter protocol
     behavior. These are not Wolfram oracle lanes unless the behavior is a
     same-name language surface.
5. Real-project smoke tests
   - Own package/script-level confidence such as future `mma-hol` smoke lanes.
   - These prove integration maturity, not full language equivalence.

## Coverage Axes For Same-Name Behavior

Do not restrict conformance suites to happy paths. For each same-name function
that is `implemented` or substantially `partial`, choose cases across the axes
that matter for that function:

- canonical documented examples
- atom, empty, singleton, nested, and large-enough structural boundaries
- symbolic, held, unevaluated, and delayed inputs
- option defaults, explicit option overrides, invalid option names, and invalid
  option values
- invalid arity, invalid specs, message tags, and inert residual expression
  shape
- stateful behavior: definitions, attributes, contexts, messages, options,
  removed-symbol identity, package loading, and session-local caches
- interactions with core mechanisms such as `Part`, `Heads`, pattern matching,
  replacement, `Quiet`, `Check`, `Failure`, and suspension/resume
- deliberate divergence or unsupported forms, recorded as such rather than
  hidden in generic failures

A small, well-tagged set that spans these axes is more useful than a large
copy of similar happy-path examples.

## Wolfram Documentation Examples

The Wolfram documentation is an input corpus, not an exhaustive test target.
It includes front-end behavior, broad CAS behavior, platform state, historical
quirks, and examples outside `rum`'s current kernel-first scope.

Use documentation examples in this order:

1. Canonical examples that define ordinary user expectations.
2. Boundary and negative examples that expose evaluation, option, message, or
   symbolic-shape semantics.
3. Interaction examples that cross feature boundaries already claimed in the
   matrix.
4. Out-of-scope examples only as explicit `planned`, `excluded`, or divergence
   notes.

## Differential Lanes

`rum-difftest` should stay split into explicit lanes instead of becoming a
single broad diff bucket:

- Lane A: single-expression, stateless Wolfram comparisons. This is the first
  place to put compact canonical and boundary cases.
- Lane B: stateful session scripts, including definitions, options, messages,
  contexts, package loading, and repeated evaluations.
- Lane C: checked known-difference ledger, tying divergences to matrix entries
  or TODO notes.
- Lane D: performance baseline inputs and runners. These are not semantic
  conformance tests.
- Lane E: host-contract tests stay in host/native crate suites.
- Lane F: adapter/protocol tests stay in adapter-native suites.

Messages and diagnostics start in runtime regression tests unless the
differential runner can capture and compare them reliably. Lane B now provides
ordered-session comparisons against one local Wolfram kernel process per
fixture, including result values, per-step messages, and per-step print output.
That makes focused message and print behavior eligible for Lane B when the
behavior is Wolfram-oracle-comparable.

Lane C is the synchronization check for `!!! id=...` fixture entries. Keep
known differences in `rum-difftest/divergences/known-differences.txt` and use
`rum-difftest lane-c` to catch missing or unreferenced ledger rows.

## Oracle Policy

Oracle runs use a user-provided local Wolfram kernel such as `WolframKernel` or
`MathKernel`. Connector-hosted or MCP-wrapped sessions are acceptable for tiny
manual spot checks but are not the baseline oracle.

Rules:

- Do not require Wolfram tooling for default `cargo test --workspace`.
- Do not commit generated oracle output.
- Record the oracle command and version in run output when practical.
- Treat version differences as compatibility data, not as a reason to pin the
  project to one hidden oracle environment.
- Record deliberate divergences next to fixtures, with a matrix or TODO
  reference.

## Fixture Admission Checklist

Before adding a conformance fixture, identify:

- Matrix group and function family.
- Support status: `implemented`, `partial`, deliberate divergence, or deferred
  tail.
- Test layer: unit, runtime regression, difftest lane, adapter, or real-project
  smoke.
- Coverage axis: canonical, boundary, negative, stateful, interaction, or
  divergence.
- Expected comparison mode: value-only, held shape, messages, side effects,
  request/suspension, or performance.
- Whether the fixture should block ordinary development or require an explicit
  oracle-enabled command.

## Practical Priorities

Near-term conformance work should focus on the semantics most likely to break
real packages:

- rules, patterns, replacement, and traversal
- definitions, attributes, messages, and option resolution
- held evaluation, cleanup, failure/status objects, and host input
- part specifications, structural mutation, and association/keyed data
- syntax/lowering round trips for package-level source files

String, arithmetic, and existing CL-04/CL-07 smoke fixtures already have first
footholds. Extend them when real bugs or migration needs expose missing axes,
not just to increase fixture count.
