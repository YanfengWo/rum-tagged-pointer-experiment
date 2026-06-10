# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

## 2026-06-10 Scope Reset

The project is moving from broad core-language expansion toward a `v0.1`
kernel-first stabilization pass.

Current direction:

- stop treating Wolfram same-name surface growth as the default next action
- use concrete downstream pressure, especially `mma-hol`, to promote missing
  semantics back into active work
- keep `rum-host` as the main embedding boundary
- freeze project-owned Python adapter work until a real downstream need returns
- keep Jupyter as an optional capability-validation adapter, not a roadmap
  driver

Detailed future showcase plans for a Python AI policy router and a Rust Wasm
prototype were removed. Those remain possible application directions, but they
are not active project-owned examples.

## 2026-06-10 Correctness Fixes

- Definition insertion now applies the same specificity ordering to
  `UpValues` and `SubValues` that `DownValues` already used, so later typed or
  literal rules can outrank earlier generic patterns while equivalent pattern
  definitions still replace in place.
- Added runtime regressions plus curated Lane A `definitions_precedence`
  coverage for generic-before-typed `UpValue` and `SubValue` definitions.

## Completed Architecture Work

- The workspace was split into responsibility-oriented crates:
  `rum-syntax`, `rum-core`, `rum-runtime`, `rum-stdlib`, `rum-terminal`,
  `rum-repl`, `rum-cli`, `rum-host`, `rum-jupyter`, and `rum-difftest`.
- Major `include!`-style and monolithic runtime, syntax, Jupyter, and
  difftest files were replaced with normal Rust module trees.
- The 2026 source-size cleanup pass removed Rust files over the `35k`
  architecture-debt threshold; the remaining work is mostly warning-range
  opportunistic cleanup.
- `rum-host` now provides the first-class Rust embedding facade over runtime
  sessions, requests, structured outcomes, failures, inspection, completion,
  completeness checks, and input suspension/resume.
- `rum-jupyter` was kept optional and layered as an adapter instead of becoming
  evaluator infrastructure.

## Completed Core-Language Mainlines

The `CL-01` through `CL-09` mainlines are considered closed for the current
kernel-first phase. Remaining gaps are demand-driven parity tails, not default
implementation work.

Broad completed areas include:

- expression model, lowering, parser diagnostics, formatter, exact integers
- definitions, attributes, values, scoping, contexts, package loading
- matcher IR, rules, patterns, replacement, traversal, and tree walkers
- functional/list combinators across the current first-pass supported subset
- associations, keyed data, missing values, keyed mutation, and grouping
- procedural control, nonlocal flow, cleanup, status objects, host input
- string/text surface, string patterns, formatting, templates, byte arrays
- package/options/protection/name-inspection tooling
- tracing, messages, resource limits, timing, memory proxies, test reports

The useful lesson from this phase is that every new Wolfram same-name builtin
is expensive: it tends to touch evaluator semantics, messages, resource
budgets, suspension/resume, host surfaces, docs, and conformance fixtures.
Future additions should therefore be promoted by a concrete blocker rather
than by inventory coverage alone.

## Archived Roadmap Detail

The old `TODO.md` carried long completed checklists for individual CL waves,
external-review fixes, difftest lane construction, string-language expansion,
and math-surface footholds. Those details were compressed on 2026-06-10 so the
roadmap can show active decisions instead of completed work.

The archived completed waves include:

- `CL-08` package, namespace, options, protection, persistence, and name
  tooling
- `CL-09` tracing, timing, resource-limit proxies, monitors, messages,
  evaluation data, test reports, and dialog support
- post-mainline closure passes for `CL-01` through `CL-09`
- first practical math footholds for exact arithmetic, polynomial expansion,
  coefficient extraction, factorization, `Together`, and symbolic `D`
- the first large string-language surface, including string construction,
  exact search, string patterns, templates, conversions, byte arrays,
  hashing/compression/encryption, nearest-neighbor helpers, and pluralization
- resolved external-review P0/P1 items for definition replacement, Jupyter
  signing/codec/frame hardening, conformance fixtures, and package/context
  coverage

Canonical current status should be read from the support matrix and tests, not
from the archived checklist wording.

## Verification And Harness Work

The `rum-difftest` workstream now has:

- Lane A for stateless Wolfram-oracle comparisons
- Lane B for stateful session comparisons
- Lane C for checked known-difference ledger validation
- Lane D for local performance smoke/baseline runs
- non-oracle inventory and aggregate `check` gates
- artifact-hygiene checks to keep generated oracle material out of the repo
- summary JSON output with stable top-level schema versions

The practical default gate remains:

```bash
cargo fmt --check
cargo test --workspace
cargo run -p rum-difftest -- check
```

Wolfram oracle runs are intentionally environment-gated and should remain
optional for normal development.

## Operational Lessons

- Do not use the full Wolfram builtin inventory as a commitment list.
- Prefer local WolframKernel spot checks for same-name semantics, but do not
  commit generated oracle transcripts.
- Keep resource limits cooperative and explicit. Generated-output functions
  need budget checks before allocation.
- Keep support-matrix statuses conservative. `partial` means only the
  documented subset is compatibility-targeted.
- Prefer one real downstream driver over multiple speculative examples.
- Move completed roadmap detail here once it no longer changes decisions.

## Frozen Or Archived Directions

### Python Adapter

Python remains a plausible future host, especially for AI/data/tooling
workflows, but project-owned Python binding work is frozen for now.

If revived later, the rules remain:

- build on `rum-host`, not broad `rum-runtime`
- keep `pyo3` and packaging concerns outside kernel crates
- start with eval/resume/inspect/complete/is-complete only
- add a real downstream use case before committing to public Python API shape

### Standalone Showcase Plans

The detailed Python AI policy and Rust Wasm prototype plans were removed to
avoid turning speculative examples into roadmap obligations.

The underlying application directions remain valid as short examples of where a
symbolic rule-engine kernel could be useful:

- policy / validation / rewrite layers in AI or data tooling
- semantics and validation prototypes for compilers, runtimes, or DSLs
