# Brief 240 — typed value bridge, out: widen HostTyped (Bool/Association/Rule) + jupyter application/json consumer

Implementation brief — embedding API line, item a3 (see
`docs/dev/embedding-api-line.md`, finding F2: `HostTyped` shipped with
zero consumers and collapses structured values to `Other`). Two parts:
widen the typed surface with three variants, and give it its first real
consumer — jupyter's execute result gains a conservative
`application/json` MIME entry (user ruling 2026-07-16: wire-visible
addition approved; the three existing MIME entries stay untouched).

## Pre-diagnosed facts (verified this session)

- `host_typed_from_expr_at_depth`
  (`rum-host/src/model/outcome.rs:442-464`): atoms map via
  `host_typed_from_atom`; the ONLY structured head recognized is `List`
  (`is_list_head`, `:482-484`, using
  `symbol.is_system_name("List")`); everything else — including
  `Association` and `Rule` — collapses to `Other`; depth ≥
  `MAX_TYPED_DEPTH` (64) collapses to `Other`. `True`/`False` currently
  surface as `Symbol("True")`/`Symbol("False")` (they are symbol atoms,
  not `Other`).
- Association entries: `rum_runtime::kernel_api` exports
  `association_entries_from_expr(&Expr) -> Option<Vec<(Expr, Expr)>>`
  and `association_entry_forms_from_expr` (whose entry form carries
  `delayed: bool`) — use these; do not reimplement association shape
  detection.
- jupyter execute-result bundle: `expr_mime_bundle`
  (`rum-jupyter/src/adapter/render.rs:178-183`) = text/plain +
  text/markdown + application/vnd.rum.expr. `KernelMimeBundle.items`
  is a `BTreeMap<String, String>`. JSON *encoding* helpers live in
  `rum-jupyter/src/transport/json/encode.rs` — reuse its string-escape
  facility if it is reusable; otherwise a small local escaper is fine
  (no new dependencies).
- `HostTyped::` reference sites (the widening ripple):
  `rum-host/src/model/outcome.rs` (7), `rum-host/src/tests.rs` (12),
  `rum-host/src/lifecycle_contract_tests.rs` (3),
  `rum-host/tests/lifecycle_contract.rs` (3),
  `rum-terminal/src/lib.rs` (1). `HostTyped` has no consumers outside
  these (workspace-verified), so no exhaustive-match breakage beyond
  them is possible.

## Required change

### Part A — widen `HostTyped` (`rum-host/src/model/outcome.rs`)

Add three documented variants (crate has `#![warn(missing_docs)]`):

1. **`Bool(bool)`** — produced for the system symbols `True`/`False`
   (same `is_system_name` discipline as `is_list_head`). Rustdoc MUST
   state that `True`/`False` no longer surface as `Symbol`
   (deliberate convenience mapping; no in-tree consumer depended on
   the old shape).
2. **`Association(Vec<(HostTyped, HostTyped)>)`** — ordered key/value
   pairs via `association_entries_from_expr`, keys and values
   converted at `depth + 1`. If the association contains ANY delayed
   entry (`association_entry_forms_from_expr` reports `delayed`), the
   WHOLE association stays `Other` — a data view must not present an
   unevaluated right-hand side as a value. Pin this with a test.
3. **`Rule { lhs: Box<HostTyped>, rhs: Box<HostTyped> }`** — for the
   system head `Rule` with exactly two arguments, converted at
   `depth + 1`. `RuleDelayed` stays `Other` (same honesty rule) — pin
   it.

Depth discipline: all three respect `MAX_TYPED_DEPTH` exactly like
`List` (children at `depth + 1`; at the cap the node collapses to
`Other`). Update the `HostTyped` enum rustdoc summary accordingly.

Mechanical ripple: fix every `HostTyped::` reference site listed above
that stops compiling or asserts the OLD shape for
`True`/`False`/associations/rules — each such assertion update must be
listed in the report with old → new. (`rum-terminal`'s single site and
the lifecycle tests are expected to be untouched or trivially
mechanical.)

### Part B — jupyter consumer (`rum-jupyter/src/adapter/render.rs`)

1. New helper `json_from_typed(&HostTyped) -> Option<String>` —
   returns a JSON document iff the typed tree is FULLY representable
   under the conservative mapping:
   - `Integer(i64)` → JSON number
   - `Str` → JSON string (escaped)
   - `Bool` → `true`/`false`
   - `List` → array
   - `Association` → object, in entry order, iff every key is `Str`
     and keys are pairwise distinct (duplicate or non-string key →
     `None`)
   - anything else (`BigInteger`, `Rational`, `Symbol`, `Rule`,
     `Other`) → `None` for the whole tree.
   No pretty-printing requirements; a single-line document is fine.
2. The execute-result bundle: when `json_from_typed(&value.typed)`
   is `Some(json)`, `bundle.insert("application/json", json)`. The
   three existing entries are byte-unchanged in all cases.
   (The success path in `render.rs` has access to the full
   `HostEvalSuccess` — thread `&HostTyped` to the bundle builder as
   needed; keep `expr_mime_bundle` behavior for every other call
   site, e.g. suspension/failure rendering, unchanged.)

## Tests

- `rum-host` (`src/tests.rs`):
  a. `True` evals to `HostTyped::Bool(true)`; `False` → `Bool(false)`.
  b. an association literal (e.g. `<|"a" -> 1, "b" -> {True}|>`)
     yields `Association` with ordered entries and nested conversion;
  c. an association with a delayed entry (`:>`) stays `Other`;
  d. `a -> b` yields `Rule` with `Symbol` sides; `a :> b` stays
     `Other`;
  e. the existing deep-nesting cap test still passes (extend it to
     cover an association nested at the cap if cheap).
- `rum-jupyter` (existing adapter test file):
  f. executing a JSON-representable value (e.g.
     `<|"a" -> 1, "b" -> {True, "x"}|>`) yields an execute result
     whose bundle contains `application/json` with the exact expected
     document AND the unchanged text/plain entry;
  g. a non-representable value (e.g. `1/2` or `x -> 1`) yields NO
     `application/json` key; the three existing MIME entries are
     present as before.

## Scope

MAY modify:
- `crates/rum-host/src/model/outcome.rs`
- `crates/rum-host/src/lib.rs` (docs/re-exports only, if needed)
- `crates/rum-host/src/tests.rs`,
  `crates/rum-host/src/lifecycle_contract_tests.rs`,
  `crates/rum-host/tests/lifecycle_contract.rs` (MECHANICAL/assertion
  updates, each listed)
- `crates/rum-terminal/src/lib.rs` (single reference site, MECHANICAL
  only)
- `crates/rum-jupyter/src/adapter/render.rs`
- `crates/rum-jupyter/src/adapter/mod.rs` (plumbing only, if needed)
- `rum-jupyter` test files for the new/changed expectations (list each)
- `docs/guide/embedding.md` (typed-surface note: the three new
  variants, the Bool mapping, the delayed-entry honesty rule)

MUST NOT touch: anything in `rum-runtime`/`rum-core`/`rum-syntax`/
`rum-stdlib`, `rum-cli`/`rum-repl`, `HostFailureCode`,
`HostEvalRequest`, the three existing MIME entries' content, JSON
*parsing* (`json_support.rs`), the wire codec
(`transport/json/encode.rs` may be READ for escape reuse; modify it
only if exposing an existing escaper requires a visibility change —
say so).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

- **No microbench A/B**: the typed conversion runs once per top-level
  request in the outcome mapping (not per evaluation step), and the
  microbench label timings are in-process loops that never reach it;
  state this in the report.

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- Compatible enhancement only: existing MIME entries and all
  non-typed-related test expectations stay byte-identical. Allowed
  expectation changes are exactly those caused by the three new
  variants (each listed old → new in the report).
- Stop-loss: if the association helpers cannot express something this
  brief assumes (e.g. entry order not preserved), STOP and report
  instead of hand-rolling shape detection.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Assertion updates from the widening (each old → new).
3. The JSON mapping edge cases you pinned (duplicate keys, non-Str
   keys, nested non-representable subtree, depth-cap interaction).
4. Test survey — each new test/assertion, named.
5. Verification — commands with verbatim output (test totals).
6. The no-A/B justification (one sentence).
7. Deviations / open questions (empty if none).
