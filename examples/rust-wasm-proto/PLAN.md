# Rust Wasm Prototype Plan

This directory is a placeholder for a future Rust-hosted example project.
Do not treat it as active implementation work until the core `rum` runtime and
host-adapter APIs are stable enough.

## Goal

Build a Rust-hosted, `rum`-specified WebAssembly validation and semantics
prototype.

The point is not to build a production Wasm runtime. The point is to show that
Rust plus `rum` can rapidly prototype a rigorous, nontrivial language
semantics:

- Rust owns system-facing infrastructure.
- `rum` owns symbolic validation, step rules, diagnostics, and explanation.

This should be the Rust-host counterpart to:

- `mma-hol`: pure symbolic language pressure test
- `python-ai-policy`: Python adapter / AI-infra policy middleware
- `rust-wasm-proto`: Rust host / compiler-runtime semantics prototype

## Why Wasm

WebAssembly is a good demonstration target because it has:

- a rigorous public core specification
- a stack-machine execution model
- explicit validation rules
- structured control flow
- traps and well-defined runtime errors
- enough complexity to be convincing
- a natural fit for a Rust host

The surprising effect should be that a meaningful Wasm Core subset can be
expressed quickly as symbolic rules instead of as a conventional hand-written
VM from the start.

## Division Of Responsibility

Rust should own:

- project structure and test harness
- fixtures and CLI entry points
- host adapter API calls into `rum`
- IR storage and conversion
- byte-oriented memory representation
- optional parser integration
- result mapping into Rust enums / structs

`rum` should own:

- validation rules
- stack typing
- control-stack typing
- small-step execution rules
- trap conditions
- structured `Failure[...]` diagnostics
- explanation traces
- optional IR canonicalization / rewrite rules

## Non-Goals

Do not start with:

- complete Wasm binary parsing
- complete WAT parsing
- full official `.wast` runner
- JS/Web API embedding
- JIT or AOT compilation
- high-performance interpreter dispatch
- SIMD
- GC proposal support
- complete floating-point edge semantics
- production security claims

This project should be named and described as a prototype, not as a Wasm
runtime.

## Milestones

### Milestone 0: Project Contract

Define the host/rule-core boundary.

Deliverables:

- canonical Rust IR for the first Wasm subset
- canonical `rum` representation of the same IR
- adapter contract for calling `ValidateFunction` and `RunFunction`
- expected shape of value, trap, invalid, and explanation results

### Milestone 1: Tiny Executable Core

Support the smallest useful executable subset.

Possible instruction subset:

- `i32.const`
- `i32.add`
- `i32.sub`
- `i32.mul`
- simple integer comparisons

Required behavior:

- value stack
- single function body
- one return value
- basic `ValidateFunction`
- basic `RunFunction`

Example demos:

- `add(1, 2)`
- constant expression evaluation
- invalid stack-shape diagnostics

### Milestone 2: Locals And Direct Calls

Add basic function structure.

Possible instruction subset:

- function signatures
- parameters and locals
- `local.get`
- `local.set`
- `local.tee`
- direct `call`

Required behavior:

- type-stack checking
- local index checking
- direct call signature checking
- multiple functions in a module-like fixture

Example demos:

- simple function composition
- non-recursive factorial helper
- invalid local index / call signature failures

### Milestone 3: Structured Control

This is the first milestone where `rum` should visibly pay off.

Possible instruction subset:

- `block`
- `loop`
- `if`
- `br`
- `return`
- `unreachable`

Required behavior:

- control stack
- label typing
- branch target validation
- unreachable / polymorphic-stack subset
- small-step execution for structured control

Example demos:

- factorial
- fibonacci
- loop sum
- invalid branch target diagnostics

### Milestone 4: Linear Memory Subset

Add a small memory model while preserving the Rust/`rum` split.

Possible instruction subset:

- one linear memory
- `i32.load`
- `i32.store`

Required behavior:

- Rust stores bytes
- `rum` models address calculation and type rules
- bounds checks produce traps
- alignment can be accepted but not optimized

Example demos:

- array sum
- simple memory copy
- out-of-bounds trap

### Milestone 5: Spec-Test Style Harness

Add a lightweight conformance-style test layer without implementing a full
`.wast` runner.

Fixture forms:

- `assert_return`
- `assert_trap`
- `assert_invalid`

Required behavior:

- deterministic before/after traces
- structured validation failures
- structured trap reports
- clear mapping from fixture outcome to Rust test result

## Later Extensions

These extensions are good because they continue to exercise `rum`'s symbolic
strength instead of turning the project into parser or performance work.

- Deeper validation: multi-value, full unreachable typing, block type edges
- Table/reference semantics: `funcref`, tables, `call_indirect`
- Diagnostics: richer `Failure[...]` objects with stack/control-stack snapshots
- Symbolic execution: path conditions for small functions and possible-trap
  analysis
- IR rewrite: constant folding, dead branch simplification, stack-effect
  simplification
- Feature gates: rule switches for selected Wasm proposal subsets
- Cranelift backend experiment: keep Rust plus `rum` responsible for parsing,
  canonical IR, validation, semantic normalization, diagnostics, and
  explanation, then lower the validated subset from Rust into Cranelift IR for
  native code generation
- Partial official test import: only import cases matching the implemented
  subset

If this project ever grows from semantics prototype toward compiler prototype,
the preferred route is:

```text
Wasm binary / WAT / fixture
  -> Rust parser / canonical IR
  -> rum validation + semantic normalization + diagnostics
  -> Rust lowering to Cranelift IR
  -> Cranelift native code
```

This preserves the core demonstration: `rum` accelerates the semantic frontend,
while Rust and Cranelift handle systems integration and backend codegen.

## Expected Size

Keep the first convincing version small:

- Rust host and tests: roughly 2k-4k lines
- `rum` rules: roughly 1k-2k lines
- fixtures: enough to demonstrate each milestone, not a full conformance suite

If the prototype grows beyond this before Milestone 3, cut parser and harness
scope first.
