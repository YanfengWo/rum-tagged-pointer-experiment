# Brief 254 — `UpTo[n]` sequence specs for Take/Drop/TakeDrop (WL-conformance gap, brief-252 follow-up)

## Goal

`Take`, `Drop`, and `TakeDrop` accept `UpTo[n]` (n a non-negative machine
integer) and `UpTo[Infinity]` as a sequence spec, at any spec position
(multi-level specs included), with clamp-to-length semantics:
`Take[{1,2,3}, UpTo[5]]` → `{1,2,3}`, `Take[{1,2,3}, UpTo[2]]` → `{1,2}`,
`Drop[{1,2,3}, UpTo[5]]` → `{}`, `TakeDrop[{1,2,3}, UpTo[2]]` →
`{{1,2},{3}}`. A structurally well-formed `UpTo[…]` whose content is
invalid (negative, non-integer, or wrong arity) emits the same
`UpTo::innf` / `UpTo::argx` diagnostics the kernel Part-span path already
emits, and the call stays inert. Every other Take/Drop/TakeDrop behavior
is byte-identical to today.

## Context pointers

- Read `AGENTS.md` (Testing Policy hard lines) before touching anything.
- Key files:
  - `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs` — the whole
    implementation: `StructuralSequenceSpec` enum, `parse_structural_sequence_spec`,
    `resolve_take_components`, `take_drop_message_range`, message helpers.
  - `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/parse.rs:175-205`
    — the kernel span-endpoint parser whose `UpTo` acceptance rule you mirror
    (unary `UpTo`, content = non-negative machine integer OR `Infinity`).
    READ ONLY — do not modify.
  - `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:290-330`
    — the kernel `UpTo::innf`/`UpTo::argx` message texts and the
    push-once dedup. READ ONLY — mirror the message TEXT shapes exactly.
  - `crates/rum-stdlib/src/tests/structure_ops/sequence.rs:237-242` — the
    pinned inert-behavior test (`Take[{1,2,3}, UpTo[5]]` stays inert); this
    brief REPLACES that pin with behavior tests.
  - `crates/rum-stdlib/src/catalog/structure.rs:136-176` — the Take/Drop/
    TakeDrop catalog entries (summaries say "current standard sequence-spec
    subset"; extend the wording to mention `UpTo`). Do NOT add message
    declarations here: the malformed-content diagnostics are pushed under the
    `UpTo` carrier, whose `innf`/`argx` pairs are already registered
    kernel-side (brief-252); the registry merge is global, so the push_message
    debug assertion passes as-is.
  - `docs/reference/builtins.md` — GENERATED. If catalog text changes,
    regenerate via `cargo run -p rum-stdlib --bin generate-builtin-reference`
    (the drift test `builtin_reference_docgen.rs` enforces agreement).
- Acceptance is mechanical: the named tests and probes below decide, not prose.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs`
  - `crates/rum-stdlib/src/tests/structure_ops/sequence.rs`
  - `crates/rum-stdlib/src/catalog/structure.rs` (summary wording of the
    three entries ONLY; no new message declarations, no attribute changes)
  - `docs/reference/builtins.md` (regeneration output ONLY)
- Files/areas you MUST NOT touch: everything else, including the kernel
  selector/span code, `namespace.rs`, `Cargo.lock`, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, any other `docs/**`.
- Do NOT add unconditional cost to the default evaluation path. This change
  is confined to the Take/Drop/TakeDrop builtin bodies (spec parsing), which
  are not on the evaluator hot path; no A/B gate is required, and none of
  eval/dispatch/matcher may be touched.

## Design decisions (already made — implement, do not re-litigate)

1. New variant `UpTo(Option<isize>)` on `StructuralSequenceSpec`
   (`Some(n)` for `UpTo[n]`, `None` for `UpTo[Infinity]`), parsed in
   `parse_structural_sequence_spec` BEFORE the generic `Expr::Call`
   fallthrough. Parse acceptance mirrors the kernel endpoint rule.
2. Clamp semantics in `resolve_take_components`:
   `UpTo(Some(n))` → components `1..=min(n, len)`; `UpTo(None)` → all
   components (same as `All`). Both `Take` and `Drop` consume the same
   components (Drop drops them) — that is what makes `Drop[{1,2,3}, UpTo[5]]`
   → `{}` and `TakeDrop` consistent. Never a failure for n > len — that is
   the point of `UpTo`.
3. `take_drop_message_range` (atom-argument error message):
   `UpTo(Some(n))` → `(1, n)`, `UpTo(None)` → `(1, -1)` (like `All`).
   `Take[7, UpTo[2]]` therefore messages `Take::take` "Cannot take positions
   1 through 2 in 7." exactly like `Take[7, 2]` does today.
4. Malformed content in a unary `UpTo[bad]` (negative integer, non-integer,
   or `UpTo` with ≠1 args): push the kernel-shaped message under the `UpTo`
   symbol (`innf` for bad content, `argx` for wrong arity — copy the exact
   template wording from the kernel sites), then return the inert call
   (parse verdict None → the existing inert path). If the stdlib side has no
   clean dedup seam, the known fixed-point message repetition is acceptable
   (existing precedent); do NOT build new dedup machinery for this.
5. A spec that is `UpTo` applied to something structurally alien (e.g.
   `UpTo[1,2]`'s arity case aside, a non-call `UpTo` symbol alone) is NOT an
   UpTo spec — it falls through to today's behavior unchanged.
6. Multi-level specs need no extra work: each spec argument parses
   independently and `apply_take_drop_rest_to_expr` recurses; add a test
   proving `Take[{{1,2,3},{4,5}}, UpTo[3], UpTo[2]]` → `{{1,2},{4,5}}`
   (per-row clamping on the ragged row).
7. Other `UpTo` consumers (Partition, string functions, …) are OUT of this
   brief. Take/Drop/TakeDrop only.

## Deliverable

- The implementation in `take_drop.rs` per the decisions above.
- Tests in `tests/structure_ops/sequence.rs`: the replaced pin becomes a
  behavior suite covering at least — Take clamp (n > len, n < len, n = 0),
  Drop clamp (n > len → `{}`), TakeDrop clamp, `UpTo[Infinity]`,
  multi-level/ragged, atom-argument message parity (decision 3), malformed
  `UpTo[-2]` → `UpTo::innf` + inert, `UpTo[1,2]` → `UpTo::argx` + inert,
  and one no-regression pin (`Take[{1,2,3}, 2]` and `Take[{1,2,3}, {2,3}]`
  byte-identical to today).
- Catalog summary wording updated; reference regenerated; drift test green.

## rum / Rust pitfalls (read twice)

1. Follow the file's existing style: the spec enum is `Copy`; keep the new
   variant `Copy`-compatible (`Option<isize>` is fine).
2. `expr_as_signed_isize` is the existing integer extractor — reuse it; the
   Infinity check is `is_system_symbol_name(limit, heads::structure::INFINITY)`
   exactly as the kernel parser does.
3. Message templates must match the kernel wording byte-for-byte (`Non-negative
   integer or Infinity expected at position 1 in {expr}.` shape) — the
   registry templates are frozen; a diverging text is a defect.
4. Do not "improve" neighboring specs (negative counts, ranges, step) — one
   feature, minimal diff.
5. Stop-loss is real: if the same step fails twice for the same reason, STOP
   and report — do not keep tuning.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Then paste VERBATIM the live probes through the release CLI:

```
cargo run -q -p rum-cli -- --eval 'Take[{1,2,3}, UpTo[5]]'
cargo run -q -p rum-cli -- --eval 'Take[{1,2,3}, UpTo[2]]'
cargo run -q -p rum-cli -- --eval 'Drop[{1,2,3}, UpTo[5]]'
cargo run -q -p rum-cli -- --eval 'TakeDrop[{1,2,3}, UpTo[2]]'
cargo run -q -p rum-cli -- --eval 'Take[{1,2,3}, UpTo[Infinity]]'
cargo run -q -p rum-cli -- --eval 'Take[{{1,2,3},{4,5}}, UpTo[3], UpTo[2]]'
cargo run -q -p rum-cli -- --eval 'Take[{1,2,3}, UpTo[-2]]'
cargo run -q -p rum-cli -- --eval 'Take[{1,2,3}, 2]'
cargo run -q -p rum-cli -- --eval 'Take[7, UpTo[2]]'
```

No microbench A/B: no eval/dispatch/matcher code is touched and the panel
has no Take/Drop label; the "no default-path cost" rule still applies.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- Touch NOTHING outside this repo, no network. Commands: file edits within
  Scope, `cargo`, the release `rum-cli`.
- Testing Policy is a hard line (`AGENTS.md`): tests verify rum's own
  documented behavior; no external-engine comparisons of any kind.
- No new `unsafe`.
- Minimal diff: no drive-by reformatting, renames, or comment changes.
- You MUST paste the real verbatim verification output. If a run did not
  reach green, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command run, with the exact verbatim output lines.
3. Verdict — the before/after behavior table for the probe list.
4. Stop-loss / open questions (empty if none).
