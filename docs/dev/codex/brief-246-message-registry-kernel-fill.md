# Brief 246 — Message registry fill, batch 1: kernel catalog

## Goal

Fill the message-tag registry for every KERNEL-registered builtin (the
`builtin_catalog/*.rs` specs), transcribing each symbol's closed-baseline
tag set from the brief-243 inventory into `.with_messages(&[...])`
declarations, so `Messages[f]` / `Information` / the host catalog answer
for the whole kernel surface. Brief-245 landed the mechanism and seeded
`Set`; this batch is transcription at scale — no mechanism changes, no
stdlib, no docgen, no emission-site edits.

## Context pointers

- Read `AGENTS.md` first. Evidence base:
  `docs/dev/message-registry-inventory.md` (you wrote it in brief-243) —
  Table A direct sites, Table B wrapper fan-in, Table C loader/EvalError
  paths, Section D per-tag pair counts.
- Brief-245's landed shape (read the diff of commit e5a1135): the
  `BuiltinMessageSpec` struct, the `.with_messages` builder, and the
  `Set` seed in `builtin_catalog/rules.rs` — your entries must look
  exactly like that seed.
- Key files: `crates/rum-runtime/src/runtime/builtin_catalog/*.rs` (all
  kernel spec arrays); `crates/rum-runtime/src/runtime/metadata.rs`
  (types only — read, do not modify).

## Scope

- Files you MAY modify:
  - Every `crates/rum-runtime/src/runtime/builtin_catalog/*.rs` catalog
    file — ONLY by adding `.with_messages(&[...])` to existing entries
    (and a `use`/`mod` line if the shared-template module below needs
    wiring).
  - NEW FILE `crates/rum-runtime/src/runtime/builtin_catalog/
    message_templates.rs` — shared `pub(super) const BuiltinMessageSpec`
    items for the UNIFORM generic tags (Section D: `argx`, `argform`,
    `argtu`, `argt`, and any other tag whose template text is identical
    across sites — verify uniformity from your inventory before sharing;
    a tag with per-site text variants gets per-entry literals, not a
    shared const). Wire it in the builtin_catalog module file.
  - Tests: `crates/rum-runtime/src/runtime/tests/messages.rs` and any
    EXISTING test whose pinned expectation legitimately changes because
    kernel symbols now list tags. The full-equality assertions over
    all-scope listings (e.g. the `Messages[]` /
    all-tags expectations from brief-245) MAY be reshaped into robust
    assertions (containment of named samples + a total-count check)
    instead of ever-growing literal lists — this reshape is
    pre-approved; keep symbol-scoped full-equality tests (like
    `Messages[Set]`) exact.
- Files/areas you MUST NOT touch: `metadata.rs`, `message_catalog.rs`
  and all mechanism code, every emission site, stdlib catalogs
  (`crates/rum-stdlib/**`), docgen + the generated reference (drift test
  must stay green untouched), `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  `docs/**` except appending to your own report, `.git`, `Cargo.lock`.
- EXCLUSIONS from this batch (do not register): `General` (not a
  registered builtin — carrier decision is a later design brief);
  the open families (arbitrary `MessageName` emit targets and the
  held-call inspection heads' user-symbol attributions); EvalError→tag
  resource messages (`reclim`/`passlim`/… — they attribute to the failing
  head dynamically, not to one spec; later design). If the inventory
  attributes a closed tag to a kernel symbol via these paths ONLY, list
  it in the report's exclusion table instead of registering it.

## Transcription rules (pinned)

1. Per kernel symbol, the entry set = its closed-baseline pairs from the
   inventory (Table A direct rows + Table B wrapper fan-in rows + the
   path-sensitive loader rows for `FindFile`/`Get` from Table C).
2. Template text: transcribe the site's real text with WL-style backquote
   placeholders `` `1` ``, `` `2` `` … replacing interpolated values in
   argument order (the 245 Set seed is the pattern). PASSTHROUGH sites
   (pre-built dynamic strings) get a concise static description of what
   the message reports — mark those rows `DESC` in your report table.
3. Tags sort alphabetically within each `.with_messages` slice; one
   slice per entry; shared consts for verified-uniform generic tags.
4. Sampled live verification, budgeted: pick 12 entries spanning at
   least 6 different catalog files and both generic and specific tags;
   fire each sampled message on the release CLI and confirm symbol::tag
   matches your registration. Do NOT attempt to fire all pairs.
5. Completeness cross-check (mechanical, in the report): a Python count
   of registered kernel pairs (parse your own diff or the catalog files)
   vs the inventory-derived kernel closed-pair count, with the exclusion
   table explaining every difference. The two numbers plus exclusions
   must reconcile exactly.

## Deliverable

- `.with_messages` on every kernel catalog entry that has closed-baseline
  pairs; shared-template module; reconciled counts.
- Green `cargo fmt --all --check` and `cargo test --workspace`.
- Report: per-file entry/pair counts, the reconciliation math, the
  12 sampled probe transcripts, every reshaped/updated test with its
  empirical check, and the exclusion table.

## rum / Rust pitfalls (read twice)

1. This is transcription, not invention: every (tag, template) must trace
   to an inventory row or a code read of the emitting site — cite one
   evidence line per SYMBOL in the report (per-tag citations only for
   non-obvious attributions).
2. Const slices must stay `Copy`-friendly (`&'static [BuiltinMessageSpec]`
   of literals/consts — no allocation, no lazy_static).
3. The drift test (`builtin_reference_is_in_sync`) must pass WITHOUT
   regenerating the reference — if it fails, you changed something out of
   scope; STOP and report.
4. Write your report's findings tables incrementally as you fill each
   file, so a cut-off run still leaves an auditable trail.
5. No timing gate: registration is session-construction static data; if
   you find yourself touching evaluation-path code, STOP.
6. Stop-loss: if a symbol's inventory attribution is ambiguous (you cannot
   tell which spec owns a tag), SKIP it into the exclusion table with the
   ambiguity noted — do not guess.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --all --check
cargo test --workspace
```

Plus the 12 sampled probes and the reconciliation count, all pasted
verbatim.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you
  must.
- Full machine access, but touch NOTHING outside this repo, no network.
  Only file edits within Scope, `cargo`, the release `rum-cli`, and
  `python3` for parsing/counting.
- Testing Policy hard line (`AGENTS.md`).
- No new `unsafe`.
- Minimal diff: `.with_messages` additions and the named test edits only.
- Paste real verbatim verification output; never fake or summarize a
  failing state as green.

## Report format (your final message)

1. What changed — per catalog file: entries touched, pairs added.
2. Verification — verbatim fmt/test totals, 12 probe transcripts,
   reconciliation math (registered vs inventory-derived vs exclusions).
3. The exclusion table (symbol, tag, reason).
4. Reshaped/updated tests, each with empirical verification.
5. Stop-loss / open questions.
