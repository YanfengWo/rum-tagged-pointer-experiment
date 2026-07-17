# Brief 249 — Message registry ↔ docgen pairing

## Goal

The generated builtin reference lists each builtin's message tags and
templates FROM THE REGISTRY (the same `BuiltinSpec.messages` source that
`Messages[f]` reads), completing the registry↔docs pairing. Docgen gains
a compact per-entry Messages rendering; the committed reference is
regenerated; the drift test stays the guard. No registry content
changes, no mechanism changes, no emission changes.

## Context pointers

- Read `AGENTS.md` first.
- `crates/rum-stdlib/src/docgen.rs` — `push_entry` (~:340-365) currently
  renders name/summary (with `SUMMARY_OVERRIDES`), attributes,
  capability, plus the name-keyed `LIMITS` notes;
  `installed_builtin_specs` (~:301) already carries the full spec —
  `messages` is on it since brief-245.
- `crates/rum-stdlib/src/bin/generate-builtin-reference.rs` — writes
  `docs/reference/builtins.md`.
- Drift test: `builtin_reference_is_in_sync`
  (`crates/rum-stdlib/src/tests/builtin_reference_docgen.rs:6`).

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/docgen.rs` — the Messages rendering.
  - `docs/reference/builtins.md` — REGENERATED ONLY (run the generator
    bin; never hand-edit).
  - `crates/rum-stdlib/src/tests/builtin_reference_docgen.rs` — only if
    the drift assertion itself needs a mechanical touch (it should not;
    regeneration keeps it green).
- MUST NOT touch: catalogs, mechanism code, emission sites, kernel
  crates, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, other `docs/**`
  (including `docs/dev/**`) except appending to your own report, `.git`,
  `Cargo.lock`.

## Rendering (pinned)

1. Per builtin entry, after the existing fields: a single `Messages:`
   block listing `` symbol::tag — template `` one per line, tags in the
   registry's slice order (already alphabetical). Entries with an empty
   `messages` slice render NOTHING new (no empty header).
2. Template text is rendered verbatim (backquote placeholders included)
   — escape only as needed for the file's existing markdown conventions
   (match how summaries/limits are escaped today; do not invent a new
   convention).
3. The rendering must be deterministic and total: every registered pair
   appears exactly once; a Python cross-count in your report compares
   the reference's rendered pair count against the registry total (951).

## Deliverable

- The docgen change + regenerated reference.
- Green `cargo fmt --all --check` and `cargo test --workspace` (drift
  test green via regeneration).
- A diff sanity proof in the report: the reference diff consists ONLY of
  added Messages blocks (a `git diff --stat docs/reference/builtins.md`
  plus a grep showing no existing line was modified or removed — e.g.
  `git diff docs/reference/builtins.md | grep '^-' | grep -v '^---'`
  must be empty).
- The 951 cross-count, pasted verbatim.

## rum / Rust pitfalls (read twice)

1. Never hand-edit the generated file; regeneration is the only write.
2. Determinism: same input specs → byte-identical output across two
   consecutive generator runs (prove it with two runs + `cmp`).
3. The reference is consumed as docs — keep the block compact; no
   trailing whitespace (fmt/lint conventions of the existing generator).
4. No timing gate (docgen is offline tooling). Evaluation-path or
   catalog edits are out of scope — STOP.
5. Stop-loss: if rendering hits a template that breaks the file's
   markdown (unlikely), report the exact string rather than ad-hoc
   escaping beyond the existing convention.

## Verification (MANDATORY)

```
cargo fmt --all --check
cargo test --workspace
cargo run -p rum-stdlib --bin generate-builtin-reference   # (or the documented invocation)
```

Two consecutive generations byte-identical (`cmp`), the empty
removed-lines grep, and the 951 cross-count — all pasted verbatim.

## Hard rules

- No `git commit`/branch/push/config. Leave changes in the working tree.
- Nothing outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`,
  `python3`, `git diff`/`cmp` for the proofs.
- Testing Policy hard line. No new `unsafe`. Minimal diff.
- Verbatim outputs only.

## Report format (your final message)

1. What changed — docgen.rs line ranges + reference regeneration stats.
2. Verification — verbatim fmt/test totals, double-generation cmp, the
   empty removed-lines grep, the 951 cross-count.
3. One rendered sample entry (e.g. Set) pasted from the new reference.
4. Stop-loss / open questions.
