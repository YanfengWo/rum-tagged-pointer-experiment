# Brief 272 — `JoinAcross`: relational joins over lists of associations

Implementation brief. Opens the row-workload pure line (TODO.md §ACTIVE
PLAN, user ruling 2026-07-20): pure-kernel preparation for future
downstream database applications. `JoinAcross` is the one missing
relational primitive on the "table = List of Associations" surface.

## Goal

`JoinAcross` is a registered, documented, tested stdlib builtin in the
associations family implementing hash-based Inner/Left/Right/Outer joins
over two lists of associations, with single-key, multi-key, and
rule-renamed key specifications, per the exact semantics in this brief.
`cargo fmt --check` and `cargo test --workspace` are green; the generated
builtin reference includes the new entry (638 → 639 documented builtins).

## Current behavior (evidence, probed 2026-07-20 on release rum-cli @ a00f74e)

```
JoinAcross[{<|"id"->1|>}, {<|"id"->1,"v"->2|>}, "id"]
  → JoinAcross[List[Association[Rule["id", 1]]], List[Association[Rule["id", 1], Rule["v", 2]]], "id"]   (inert; no builtin)

Join[<|"a"->1|>, <|"b"->2,"a"->9|>]
  → Association[Rule["a", 9], Rule["b", 2]]   (association Join: later argument wins on key collision)
```

The second probe pins the collision policy this brief inherits for joined
rows (right row wins), so `JoinAcross` stays consistent with `Join`.

## Specification (rum's own documented behavior — folk scope)

Argument forms:

- `JoinAcross[left, right, keyspec]` — inner join (default).
- `JoinAcross[left, right, keyspec, jtype]` — `jtype` is one of the
  strings `"Inner"`, `"Left"`, `"Right"`, `"Outer"`.

`left` and `right` must each be a `List` whose every element is an
`Association`; otherwise emit the `lists` message (with the offending
argument position) and return the whole call inert.

Key specification `keyspec` (anything else → `keys` message + inert):

- a string `"k"` — sugar for `Key["k"]`, same key name on both sides;
- `Key[expr]` — join on key `expr` (any expression key), both sides;
- `specL -> specR` — join left rows on `specL`, right rows on `specR`
  (each side a string or `Key[expr]`); used when the join columns are
  named differently;
- a `List` of the above (multi-key join; each element independently may
  be a plain key or a rule). An empty list is invalid (`keys` message).

Join semantics:

- The join value of a row for one key spec = the association's value at
  that key. A row that LACKS one of its join keys participates in no
  match (it is "unmatched"; it still appears in Left/Right/Outer output
  per below). Match = the left row's and right row's join-value tuples
  are pairwise `SameQ`-identical (use the same key/value identity the
  associations family already uses for key lookup — see `eval_merge` /
  `eval_position_index` for the existing map idiom; do NOT invent a new
  comparison).
- A matched output row = the association-`Join` of the left row then the
  right row (right's values win on any colliding key, exactly like
  `Join[leftRow, rightRow]` — see evidence above). With rule-renamed
  keys the two join columns keep their own names; both appear in the
  output row. No column is dropped, reordered, or renamed.
- Multiple matches produce one output row per matching (left, right)
  pair — the full cross product for duplicated join values.
- Row order is deterministic:
  - `"Inner"`: for each left row in input order, one output row per
    matching right row in right-input order.
  - `"Left"`: as `"Inner"`, plus each unmatched left row appears (bare,
    only its own columns) at its own position in the left-major walk.
  - `"Right"`: for each RIGHT row in input order, one output row per
    matching left row in left-input order (output rows still built as
    `Join[leftRow, rightRow]`); unmatched right rows appear bare at
    their own position.
  - `"Outer"`: the `"Left"` output, followed by every unmatched right
    row (bare) in right-input order.
- Unmatched rows are NOT padded with `Missing[...]` — they carry only
  their own columns. (Consumers use `Lookup[row, key, default]`.)
- Empty `left` or `right` lists are valid (e.g. inner join with an empty
  side → `{}`; `"Left"` with empty right → all left rows bare).
- Implementation must be a hash join (index one side by join-value
  tuple, walk the other), not a nested O(n·m) scan, reusing the existing
  associations-family map machinery referenced above.

Messages (catalog `with_messages`, family template style):

- `argb` — "JoinAcross called with `1` arguments; 3 or 4 arguments are expected."
- `lists` — "The argument at position `1` in `2` is not a valid list of Associations."
- `keys` — "The argument `1` is not a valid key specification: a key, a rule of keys, or a non-empty list of these."
- `jtype` — "The argument `1` is not a valid join type; \"Inner\", \"Left\", \"Right\", or \"Outer\" is expected."

Every failure path emits exactly one message and returns the original
call inert (family convention). Attributes: `Protected` only.
Capability: `Pure` (the catalog default for `BuiltinSpec::ordinary`).

## Context pointers

- Read `AGENTS.md` (Testing Policy hard lines) first.
- `crates/rum-stdlib/src/associations/` — family implementation dir;
  `grouping.rs` (`eval_group_by`, `eval_position_index`) and
  `transforms.rs` (`eval_merge`) hold the key-indexing idiom to reuse.
- `crates/rum-stdlib/src/associations/mod.rs` — `mod` + re-export
  wiring; the catalog calls `associations::eval_join_across`.
- `crates/rum-stdlib/src/catalog/associations_transform.rs` — catalog
  entries for `Merge`/`GroupBy`/`PositionIndex`; add the `JoinAcross`
  `BuiltinSpec::ordinary(...).with_messages(&[...]).protected()` entry
  here, following the existing shape exactly.
- `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
  — one `Spec::new("JoinAcross", ATTR_PROTECTED)` line, inserted in
  SORTED position (the table is binary-searched; an unsorted insert is a
  test failure).
- `crates/rum-stdlib/src/tests/associations/` — per-topic test files
  wired in the local `mod.rs`; add a new topic file for JoinAcross.
- Reference regen: `cargo run -p rum-stdlib --bin generate-builtin-reference`
  rewrites `docs/reference/builtins.md`; the drift test
  `crates/rum-stdlib/src/tests/builtin_reference_docgen.rs` compares the
  committed file byte-for-byte — regen and commit-in-tree together.
- Acceptance is mechanical: the named tests and gates decide, not prose.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/join_across.rs` (NEW — the
    implementation);
  - `crates/rum-stdlib/src/associations/mod.rs` (MECHANICAL ONLY: `mod`
    line + re-export of `eval_join_across`);
  - `crates/rum-stdlib/src/catalog/associations_transform.rs` (one new
    catalog entry);
  - `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
    (MECHANICAL ONLY: one sorted `Spec::new` line);
  - `crates/rum-stdlib/src/tests/associations/join_across.rs` (NEW) and
    `crates/rum-stdlib/src/tests/associations/mod.rs` (MECHANICAL ONLY:
    wire the new test module);
  - `docs/reference/builtins.md` (REGENERATED ONLY — never hand-edited).
- Files/areas you MUST NOT touch:
  - `crates/rum-core/**`, `crates/rum-runtime/**` outside the single
    `stdlib_attributes.rs` line above — this is a stdlib-only builtin;
    if you believe a kernel change is needed, STOP and report;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (other than the
    regenerated reference), anything under `docs/dev/codex/`;
  - `.git`, git config, `Cargo.lock`.
- Do NOT add unconditional cost to the default evaluation path.

## Deliverable

- The `JoinAcross` implementation + registration chain (impl file,
  mod wiring, catalog entry with the four messages, sorted attribute
  mirror line).
- Tests covering AT MINIMUM: inner join basic; left/right/outer
  including bare-unmatched rows and output order; multi-key join;
  rule-renamed key (both columns present in output); duplicate join
  values (cross product, deterministic order); collision (right row
  wins, `Join`-consistent); a row missing a join key (unmatched, not an
  error); empty side for each jtype; each of the four messages fires
  once and the call stays inert (bad list arg at positions 1 and 2, bad
  keyspec incl. empty list, bad jtype).
- Regenerated `docs/reference/builtins.md` containing the JoinAcross
  entry (638 → 639).
- The report per Report format below.

## rum / Rust pitfalls (read twice)

1. **Suspension safety**: the join itself must not call back into the
   evaluator (keys are data lookups, not function application), so
   implement it as a plain data transform — no `suspend_with_frame`
   machinery needed. If you find yourself needing evaluator callbacks,
   STOP and report; the spec does not require them.
2. **Hot paths avoid owned-tree clones.** Rows are `Arc`-shared; build
   output rows with the same shared-value idiom `eval_merge` uses; do
   not deep-clone associations you only read.
3. **The attribute mirror is a SORTED const** — binary-searched; insert
   `"JoinAcross"` in exact sorted position or the vocabulary test fails.
4. **The reference is generated** — never hand-edit
   `docs/reference/builtins.md`; run the generator bin and let the drift
   test verify.
5. **Message templates use the family's backtick-placeholder style**
   (see the `Merge`/`Counts` entries) and every message you emit must be
   one of the four declared tags — the registry enforcement is
   debug-assertion-backed (declared-or-exempt).
6. **Stop-loss is real.** If the same step fails twice for the same
   reason, STOP and report — do not keep tuning.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test --workspace
cargo run -p rum-stdlib --bin generate-builtin-reference && git diff --stat docs/reference/builtins.md
```

Then paste the verbatim output of these five release-CLI probes
(`cargo build --release -p rum-cli` first if needed):

```
target/release/rum-cli --eval 'JoinAcross[{<|"id"->1,"a"->"x"|>,<|"id"->2,"a"->"y"|>}, {<|"id"->1,"b"->10|>,<|"id"->3,"b"->30|>}, "id"]'
target/release/rum-cli --eval 'JoinAcross[{<|"id"->1,"a"->"x"|>,<|"id"->2,"a"->"y"|>}, {<|"id"->1,"b"->10|>,<|"id"->3,"b"->30|>}, "id", "Outer"]'
target/release/rum-cli --eval 'JoinAcross[{<|"u"->1,"a"->"x"|>}, {<|"v"->1,"b"->10|>}, "u"->"v"]'
target/release/rum-cli --eval 'JoinAcross[{<|"k"->1,"c"->"L"|>}, {<|"k"->1,"c"->"R"|>}, "k"]'
target/release/rum-cli --eval 'JoinAcross[{<|"k"->1|>}, {<|"k"->1|>}, "k", "Sideways"]'
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
   (test totals, the reference diffstat, the five probe outputs).
3. The finding / verdict — semantics implemented vs the spec, any
   deliberate deviation (should be NONE without a stop-loss).
4. Stop-loss / open questions — where you stopped and why (empty if
   none).
