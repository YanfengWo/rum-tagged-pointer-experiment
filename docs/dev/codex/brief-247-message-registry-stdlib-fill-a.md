# Brief 247 — Message registry fill, batch 2: stdlib catalog A

## Goal

Fill the message-tag registry for the stdlib builtins whose specs live in
the five largest catalog files — `strings.rs`, `structure.rs`,
`predicates.rs`, `functional.rs`, `associations.rs` — transcribing each
symbol's closed-baseline tag set from the brief-243 inventory into
`.with_messages(&[...])`, exactly like the landed kernel batch
(brief-246, commit e09983f). Batch B (every other stdlib catalog file) is
the NEXT brief — do not start it.

## Context pointers

- Read `AGENTS.md` first. Evidence base:
  `docs/dev/message-registry-inventory.md` (Tables A/B, Section D).
- Landed pattern to copy: commit e09983f — per-entry `.with_messages`
  slices + a shared-template consts module for verified-uniform generic
  tags + reshaped global count assertions.
- Key files: the five stdlib catalog files above under
  `crates/rum-stdlib/src/catalog/`;
  `crates/rum-runtime/src/runtime/kernel_api.rs:54` (the stdlib-facing
  re-export line — `BuiltinMessageSpec` is not yet exported there).

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/catalog/{strings,structure,predicates,functional,associations}.rs`
    — ONLY adding `.with_messages(&[...])` to existing entries (+ needed
    `use` lines).
  - NEW FILE `crates/rum-stdlib/src/catalog/message_templates.rs` —
    stdlib's own `pub(crate) const BuiltinMessageSpec` items for
    verified-uniform generic tags (the kernel's module is `pub(super)`
    and not reachable; duplicate the needed consts here, texts verified
    from the inventory). Wire it in `crates/rum-stdlib/src/catalog/mod.rs`.
  - `crates/rum-runtime/src/runtime/kernel_api.rs` — MECHANICAL ONLY: add
    `BuiltinMessageSpec` to the line-54 metadata re-export; nothing else.
  - Tests whose pinned GLOBAL COUNTS legitimately move because stdlib
    pairs now register (the brief-246 reshaped sample+count assertions in
    `crates/rum-runtime/src/runtime/tests/messages.rs`, and any stdlib or
    rum-host count pins): update the counts empirically and state old→new
    per test. Symbol-scoped exact tests must NOT need changes — if one
    fails, treat it as a defect in your transcription, not an expectation
    to edit.
  - `crates/rum-stdlib/src/tests.rs` (or its test modules) MAY gain one
    new sampled assertion for a batch-A symbol.
- Files/areas you MUST NOT touch: kernel catalogs and mechanism code, all
  emission sites, the OTHER stdlib catalog files (batch B), docgen + the
  generated reference (drift test stays green untouched — the reference
  renders no message data), `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  `docs/**` except appending to your own report, `.git`, `Cargo.lock`.
- Same EXCLUSIONS as brief-246: `General`, EvalError-family resource
  tags, open families. Symbols outside the five files go to the batch-B
  remainder count, not into edits.

## Transcription rules (pinned — identical to brief-246)

1. Per symbol: closed-baseline pairs from inventory Table A direct rows +
   Table B wrapper fan-in rows.
2. Templates: real site text with `` `1` ``/`` `2` `` placeholders in
   argument order; PASSTHROUGH sites get a concise static description,
   marked `DESC` in the report.
3. Alphabetical tags per slice; shared consts only for tags whose text is
   verified uniform across THIS batch's sites.
4. Sampled live verification: 12 entries spanning all five files, generic
   and specific tags, fired on the release CLI, transcripts pasted.
5. Reconciliation in the report: batch-A registered pairs + batch-B
   remainder (listed per file, counted from the inventory) must sum to
   650, with any per-symbol ambiguity in an exclusion table (SKIP, do not
   guess).

## Deliverable

- `.with_messages` on every batch-A entry with closed pairs; the stdlib
  shared-template module; the kernel_api one-line re-export.
- Green `cargo fmt --all --check` and `cargo test --workspace`.
- Report: per-file entry/pair counts, reconciliation math, 12 probe
  transcripts, every count-pin update (old→new, each verified), exclusion
  table.

## rum / Rust pitfalls (read twice)

1. Transcription, not invention — one evidence citation per symbol.
2. `&'static [BuiltinMessageSpec]` of literals/consts only; `Copy` stays.
3. The drift test must pass WITHOUT regenerating the reference; a failure
   means out-of-scope change — STOP.
4. Write report tables incrementally per file.
5. No timing gate (static registration data); evaluation-path edits are
   out of scope — STOP if tempted.
6. Stop-loss: ambiguous attribution → exclusion table with the ambiguity
   noted.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --all --check
cargo test --workspace
```

Plus the 12 sampled probes and the reconciliation count, pasted verbatim.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you
  must.
- Full machine access, but touch NOTHING outside this repo, no network.
  Only file edits within Scope, `cargo`, the release `rum-cli`, and
  `python3` for parsing/counting.
- Testing Policy hard line (`AGENTS.md`). No new `unsafe`. Minimal diff.
- Paste real verbatim verification output; never present a failing state
  as green.

## Report format (your final message)

1. What changed — per catalog file: entries touched, pairs added.
2. Verification — verbatim fmt/test totals, 12 probe transcripts,
   reconciliation math (batch A + batch B remainder = 650).
3. Exclusion table (symbol, tag, reason).
4. Count-pin updates (test, old→new, empirical check).
5. Stop-loss / open questions.
