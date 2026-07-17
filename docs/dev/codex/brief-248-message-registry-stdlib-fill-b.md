# Brief 248 — Message registry fill, batch 3 (final): stdlib catalog B

## Goal

Complete the message-tag registry fill: transcribe the closed-baseline
tag sets for the REMAINING stdlib catalog files (everything under
`crates/rum-stdlib/src/catalog/` except the five batch-A files and the
shared modules), 215 pairs per the brief-247 reconciliation. After this
batch the entire 966-pair closed baseline is either registered or in the
standing exclusion set (General 4, EvalError family 8, unowned UpTo 2,
open families). Same rules as brief-247 (commit pending in tree; pattern
identical to e09983f).

## Context pointers

- Read `AGENTS.md` first. Evidence:
  `docs/dev/message-registry-inventory.md`; brief-247's reconciliation
  lists the expected per-file pair counts for this batch:
  arithmetic 21, bytes 26, calculus 3, combinatorics 10, control 45,
  csv 2, datetime 19, ini 2, integer_digits 10, json 3, logic 6,
  number_theory 10, polynomials 5, random 18, rewrite 27, schema 2,
  toml 2, xml 2, yaml 2 (character_classes 0). Treat these as the
  target ledger: your final per-file counts must reconcile against it,
  with any delta explained from the inventory, not waved off.
- The stdlib shared-template module `catalog/message_templates.rs` and
  the `kernel_api` re-export already exist (batch A) — extend, don't
  recreate.

## Scope

- Files you MAY modify:
  - Every `crates/rum-stdlib/src/catalog/*.rs` EXCEPT the batch-A five
    (`strings`, `structure`, `predicates`, `functional`, `associations`)
    — ONLY adding `.with_messages(&[...])` (+ needed `use` lines).
  - `crates/rum-stdlib/src/catalog/message_templates.rs` — add shared
    consts needed by this batch (verified-uniform texts only).
  - Tests whose pinned global counts legitimately move (empirical
    old→new per test); one new sampled assertion allowed. Symbol-scoped
    exact tests must not need edits — a failure there is your defect.
- MUST NOT touch: batch-A catalog files, kernel code, emission sites,
  docgen + generated reference (drift stays green untouched),
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to
  your own report, `.git`, `Cargo.lock`.
- Standing EXCLUSIONS unchanged: General, EvalError family, unowned
  `UpTo`, open families — report rows, not edits.

## Transcription rules (pinned — identical to briefs 246/247)

1. Inventory Table A direct rows + Table B wrapper fan-in per symbol.
2. Real site text with `` `1` ``/`` `2` `` placeholders; PASSTHROUGH →
   concise static description marked `DESC`.
3. Alphabetical tags per slice; shared consts for verified-uniform texts.
4. Sampled live verification: 12 entries spanning at least 8 of this
   batch's files; transcripts pasted.
5. Final whole-registry reconciliation in the report:
   kernel 304 + batch A 433 + batch B (this brief) + standing exclusions
   (4+8+2) + open families = the full inventory baseline; every number
   shown with its source.

## Deliverable

- `.with_messages` on every batch-B entry with closed pairs.
- Green `cargo fmt --all --check` and `cargo test --workspace`.
- Report: per-file counts vs the target ledger, final whole-registry
  reconciliation, 12 probe transcripts, count-pin updates (old→new),
  exclusion table.

## rum / Rust pitfalls (read twice)

1. Transcription, not invention — one evidence citation per symbol.
2. `&'static [BuiltinMessageSpec]` literals/consts only.
3. Drift test green WITHOUT regenerating the reference, or STOP.
4. Report tables written incrementally per file.
5. No timing gate; evaluation-path edits out of scope — STOP.
6. Stop-loss: ambiguous attribution → exclusion table, do not guess.

## Verification (MANDATORY)

```
cargo fmt --all --check
cargo test --workspace
```

Plus the 12 sampled probes and the reconciliation, pasted verbatim.

## Hard rules

- No `git commit`/branch/push/config. Leave changes in the working tree.
- Nothing outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, the
  release `rum-cli`, `python3`.
- Testing Policy hard line. No new `unsafe`. Minimal diff.
- Verbatim outputs only; never present a failing state as green.

## Report format (your final message)

1. Per-file: entries touched, pairs added, vs the target ledger.
2. Verification — verbatim totals, probes, final reconciliation.
3. Exclusion table.
4. Count-pin updates (old→new, verified).
5. Stop-loss / open questions.
