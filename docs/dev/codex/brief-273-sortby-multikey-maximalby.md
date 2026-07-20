# Brief 273 — `SortBy` multi-key tie-break form + `MaximalBy`/`MinimalBy`

Implementation brief. Second package of the row-workload pure line
(TODO.md §ACTIVE PLAN): fix the silent multi-key degradation (audit F3)
and fill the extremal-elements gap (audit F4) in the ordering family.

## Goal

`SortBy` supports the WL multi-key tie-break specification (a literal
`List` of key functions) in its 2- and 3-argument forms, lexicographic
and stable, suspension-safe; `MaximalBy` and `MinimalBy` are registered,
documented, tested builtins (2-argument + operator forms). `cargo fmt
--check` and `cargo test --workspace` are green; the generated reference
gains the two new entries (639 → 641).

## Current behavior (evidence, probed 2026-07-20 on release rum-cli)

```
SortBy[{<|"i"->3,"d"->"x"|>, <|"i"->1,"d"->"y"|>, <|"i"->2,"d"->"x"|>}, {Key["d"], Key["i"]}]
  → List[Association[Rule["i", 1], Rule["d", "y"]], Association[Rule["i", 2], Rule["d", "x"]], Association[Rule["i", 3], Rule["d", "x"]]]
```

WRONG for a multi-key sort (expected d-then-i order: the two `"x"` rows
`i`=2,3 first, then the `"y"` row). What actually happens today: the
`List` spec is applied as a function, producing inert
`List[Key["d"], Key["i"]][row]` keys, whose canonical comparison
reduces to comparing the embedded rows — silent degradation that often
LOOKS right when the row's own key order coincides with the intended
sort priority. This brief deliberately changes behavior for literal-List
specs (today's result is garbage ordering, never a documented contract).

```
SortBy[{{2,1},{1,9},{1,5}}, First]
  → List[List[1, 9], List[1, 5], List[2, 1]]      (stable: ties keep input order — the contract to preserve)

MinimalBy[{{2,1},{1,9}}, First]
  → MinimalBy[List[List[2, 1], List[1, 9]], First]  (inert; no builtin)
```

## Specification (rum's own documented behavior — folk scope)

### A. `SortBy` multi-key form

- In `SortBy[expr, spec]` and `SortBy[expr, spec, ordering]`: when
  `spec` is a literal `List` call `{f1, ..., fk}` (any k ≥ 0), the sort
  key of element `e` is the evaluated tuple `{f1[e], ..., fk[e]}`
  (a `List` expression).
- Comparison of tuple keys: 2-arg form = canonical structural order of
  the tuple lists (equal-length `List`s compare elementwise, so this IS
  lexicographic per key function); ties beyond the last key keep input
  order (the existing stable contract). 3-arg form = the `ordering`
  binary predicate receives the two TUPLES (as `List` expressions);
  predicate ties keep input order, unchanged.
- `{}` (empty spec list): all keys are the empty tuple — the input comes
  back unchanged (stability). Not an error.
- A non-`List` `spec` behaves exactly as today (single key function —
  including the operator form `SortBy[spec]`, whose semantics follow
  from the 2-arg form when applied). No change outside literal-List
  specs.
- Suspension safety: key tuples are evaluated through the existing
  per-element `resume_sort_by_*` chain. Suggested mechanism (verify it
  compiles cleanly, do not treat as gospel): construct the key
  expression as `Expr::call(List-head, vec![f1[e], ..., fk[e]])` and
  evaluate THAT via the existing `session.evaluate` +
  `suspend_with_frame` frame — one key evaluation per element, same
  frame shape, `compare_structural_order` then does the lexicographic
  work unmodified. If you instead evaluate per-component, each component
  evaluation must carry its own suspension frame.
- Update the `SortBy` catalog description to mention the multi-key form
  (it feeds the generated reference).

### B. `MaximalBy` / `MinimalBy`

- `MaximalBy[expr, f]` — evaluates `f[e]` for each top-level element of
  `expr` (suspension-safe, same per-element pattern as `SortBy`), and
  returns `expr`'s head wrapping ALL elements whose key is maximal under
  canonical structural order, in input order. `MinimalBy` = minimal.
- `MaximalBy[f]` / `MinimalBy[f]` — operator forms, following the
  existing `sort_by_operator_form` convention in the same file.
- Empty input (`{}`) → empty output (no message).
- Non-atomic first argument required: same `normal`-style message and
  inert-call convention as `SortBy`/`Sort` in this family
  (`invalid_ordering_normal_call` or the family's equivalent).
- Wrong arity → the family's arg-range message (1 to 2 arguments).
- Attributes: `Protected` only. Capability: `Pure`.

## Context pointers

- Read `AGENTS.md` (Testing Policy hard lines) first.
- `crates/rum-stdlib/src/structure_ops/ordering_sets.rs` — `eval_sort_by`
  (three arg forms + operator form), `resume_sort_by_from_index` /
  `resume_sort_by_with_key` / `resume_sort_by_ordering_from_index` (the
  suspension chain, `SortByEntry {item, key, index}`, final
  `compare_structural_order(...).then(index)` stable sort). Part A lives
  here; part B's implementation belongs in this family too (this file or
  `collection_stats.rs`, whichever fits the existing split —
  `collection_stats.rs` holds `eval_take_largest_by`/`eval_take_by`).
- `crates/rum-stdlib/src/structure_ops/mod.rs` — wiring/re-exports for
  the new eval fns (MECHANICAL; do not forget — a missed re-export
  stop-lossed an earlier brief).
- `crates/rum-stdlib/src/catalog/structure_ordering.rs` — `SortBy` entry
  (description update) + two new `BuiltinSpec::ordinary` entries with
  messages, following the family shape.
- `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
  — two `Spec::new("...", ATTR_PROTECTED)` lines in SORTED position
  (binary-searched const).
- `crates/rum-stdlib/src/tests/structure_ops/` — `ordering_sets.rs` /
  `collection_stats.rs` test files; add coverage where each part lives.
- Reference regen: `cargo run -p rum-stdlib --bin generate-builtin-reference`;
  drift test compares the committed file byte-for-byte.
- Acceptance is mechanical: the named tests and gates decide, not prose.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/structure_ops/ordering_sets.rs`;
  - `crates/rum-stdlib/src/structure_ops/collection_stats.rs` (only if
    part B lands there);
  - `crates/rum-stdlib/src/structure_ops/mod.rs` (MECHANICAL ONLY:
    wiring/re-exports);
  - `crates/rum-stdlib/src/catalog/structure_ordering.rs`;
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
    (MECHANICAL ONLY: two sorted lines);
  - `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs`,
    `crates/rum-stdlib/src/tests/structure_ops/collection_stats.rs`, and
    that dir's `mod.rs` if a new test file needs wiring;
  - `docs/reference/builtins.md` (REGENERATED ONLY — never hand-edited).
- Files/areas you MUST NOT touch:
  - `crates/rum-core/**`, `crates/rum-runtime/**` outside the two
    `stdlib_attributes.rs` lines — if you believe a kernel change is
    needed, STOP and report;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (other than the
    regenerated reference), anything under `docs/dev/codex/`;
  - `.git`, git config, `Cargo.lock`.
- Do NOT add unconditional cost to the default evaluation path.

## Deliverable

- Part A: the multi-key `SortBy` implementation + updated catalog
  description.
- Part B: `MaximalBy`/`MinimalBy` + registration chain (impl, wiring,
  catalog entries with messages, two sorted mirror lines).
- Tests covering AT MINIMUM:
  - multi-key sort where input order ≠ canonical element order ≠
    multi-key order (use the F3 witness above — it MUST come out
    d-then-i: the two `"x"` rows with `i`=2,3 first, then `"y"`);
  - multi-key stability (equal full tuples keep input order);
  - `{}` spec returns input unchanged;
  - 3-arg form with a List spec (predicate sees tuples);
  - single-function `SortBy` unchanged (regression pin: the stable-ties
    probe above stays byte-identical);
  - `MaximalBy`/`MinimalBy`: unique extremum; multiple tied extrema in
    input order; operator forms; empty list; `normal` message on an
    atomic first argument; arity message;
  - a suspension-safety test if the family has an existing pattern for
    one (a key function that is a user-defined downvalue is enough to
    exercise the evaluator round-trip).
- Regenerated `docs/reference/builtins.md` (639 → 641).
- The report per Report format below.

## rum / Rust pitfalls (read twice)

1. **Evaluator flows are suspension-safe.** Every key evaluation goes
   through `session.evaluate` + `suspend_with_frame` with a resume
   continuation, exactly like `resume_sort_by_from_index`; do not
   collapse the chain into eager calls.
2. **Preserve stability.** The existing
   `compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))`
   idiom is the contract; multi-key must keep the `.then(index)` tail.
3. **Hot paths avoid owned-tree clones.** Reuse the `SortByEntry`
   decorated-vector idiom; clone `Expr`s (Arc-backed), not deep trees.
4. **The attribute mirror is a SORTED const** — two inserts, exact
   sorted position each.
5. **The reference is generated** — regen via the bin; the drift test
   verifies byte identity.
6. **Message tags are declared-or-exempt** — every emitted message must
   be a declared catalog tag; follow the family's existing tags/
   templates for arity and `normal` diagnostics.
7. **Stop-loss is real.** Same step fails twice for the same reason →
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test --workspace        # known caveat: loopback-bind ZeroMQ tests may fail under the sandbox; if so, run and paste the two supplemental splits (--workspace --exclude rum-jupyter; -p rum-jupyter --lib -- --skip zeromq::tests::native) and say so
cargo run -p rum-stdlib --bin generate-builtin-reference && git diff --stat docs/reference/builtins.md
```

Then paste the verbatim output of these probes
(`cargo build --release -p rum-cli` first):

```
target/release/rum-cli --eval 'SortBy[{<|"i"->3,"d"->"x"|>, <|"i"->1,"d"->"y"|>, <|"i"->2,"d"->"x"|>}, {Key["d"], Key["i"]}]'
target/release/rum-cli --eval 'SortBy[{{2,1},{1,9},{1,5}}, First]'
target/release/rum-cli --eval 'SortBy[{{2,1},{1,9},{1,5}}, {First, Last}]'
target/release/rum-cli --eval 'SortBy[{{2,1},{1,9},{1,5}}, {}]'
target/release/rum-cli --eval 'MaximalBy[{<|"k"->1|>,<|"k"->3|>,<|"k"->3|>}, Key["k"]]'
target/release/rum-cli --eval 'MinimalBy[First][{{2,1},{1,9},{1,5}}]'
target/release/rum-cli --eval 'MaximalBy[7, First]'
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all
  changes in the working tree; Claude runs the authoritative gate and
  commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.
- Touch NOTHING outside this repo, no network. The only commands you run
  are file edits within Scope, `cargo`, and the release `rum-cli`.
- **Testing Policy is a hard line** (`AGENTS.md`): tests verify rum's
  own documented behavior; no black-box comparisons against other
  vendors' engines, no oracle harnesses, no third-party engine outputs.
- **No new `unsafe`.**
- Minimal diff: no drive-by reformatting, renames, or comment changes
  outside the task.
- You MUST paste the real verbatim verification output. If a run did not
  reach green, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command run, with the exact verbatim output lines
   (test totals, the reference diffstat, the probe outputs).
3. The finding / verdict — semantics implemented vs the spec, any
   deliberate deviation (should be NONE without a stop-loss).
4. Stop-loss / open questions — where you stopped and why (empty if
   none).
