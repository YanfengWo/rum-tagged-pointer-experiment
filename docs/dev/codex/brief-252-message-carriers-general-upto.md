# Brief 252 — Register the message carriers: General + UpTo (user ruling 2026-07-17)

## Goal

Close the registry's standing carrier exclusions. `General` becomes a
registered inert builtin carrying its 12 message pairs (4 direct + the 8
EvalError resource-family tags that already emit as `General::…`); `UpTo`
becomes a registered inert builtin carrying its 2 pairs (`argx`, `innf`).
After this brief `Messages[General]` and `Messages[UpTo]` answer, the
generated reference gains both entries (613 → 615), and the registry's
excluded set shrinks to the open families + the one proven-unreachable
ledger row. Wire-visible by design — the ruling approved it.

## Context pointers

- Read `AGENTS.md` first. Evidence:
  `docs/dev/message-registry-inventory.md` (General direct rows; Table C
  EvalError mapping; UpTo rows), plus these verified facts:
  - EvalError-family messages already emit with
    `symbol: "General"` (`session_helpers.rs:206-210`).
  - General direct sites: `session_helpers.rs:359` (`noio`), `:363`
    (`noentropy`), `eval_dispatch/mod.rs:146`, and
    `builtins_context_loader.rs:73` (read each site for its tag/text).
  - UpTo emission: `kernel_data/structure_data/selectors.rs:291-295`
    (with a `has_message` dedup guard) and its `parse.rs`/
    `diagnostics.rs` neighbors.
  - `expr_runtime_helpers.rs:411` lists `"UpTo"` in a match — read it
    and confirm registration does not conflict (report what that match
    is for).
- Pattern: brief-246's kernel catalog entries; WL posture: both symbols
  are `Protected` inert symbols in WL.

## Scope

- Files you MAY modify:
  - NEW FILE `crates/rum-runtime/src/runtime/builtin_catalog/inert_heads.rs`
    — two `BuiltinSpec` entries (no handler, no dispatch,
    `Capability::Pure`, `.protected()`, `.with_messages(...)`):
    `General` (12 pairs) and `UpTo` (2 pairs). Do NOT grow namespace.rs
    or rules.rs (both are in size-warning ranges).
  - `crates/rum-runtime/src/runtime/builtin_catalog.rs` — MECHANICAL:
    wire the new module + array into `core_builtin_specs`.
  - `docs/reference/builtins.md` — REGENERATED ONLY (run the generator).
  - Tests whose pinned counts/lists legitimately move (runtime
    `messages.rs` global sample+count assertions — kernel closed pairs go
    304 → 318; any Information/host pins), plus NEW probe tests below.
    Every updated expectation verified empirically, old→new stated.
- MUST NOT touch: emission sites (the EvalError path and the General/UpTo
  push sites stay byte-identical — registration aligns the registry to
  them, not the reverse), mechanism code, stdlib crates, docgen.rs,
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, other `docs/**` except appending
  to your own report, `.git`, `Cargo.lock`.

## Transcription rules

1. The 4 direct General pairs: transcribe tag + template from the sites
   (placeholder convention as in briefs 246-248).
2. The 8 EvalError-family pairs (`cancel`, `iterlim`, `matchreclim`,
   `memlim`, `passlim`, `reclim`, `reslim`, `timelim`): their runtime
   text is `error.to_string()` (fully dynamic) — use concise static
   descriptions marked `DESC`, one per tag, describing what each
   termination means (source: the `EvalError` variants' Display impls).
3. UpTo's `argx`/`innf`: transcribe from the selector sites.

## Deliverable

- The new catalog module + wiring; regenerated reference (615 entries).
- New tests pinning: `Messages[General]` lists exactly the 12 seeded
  tags; `Messages[UpTo]` lists `argx`/`innf`; `General = 5` and
  `UpTo = 5` now fail with the protection message (WL-conformant);
  `UpTo[3]` stays inert AND `Take[{1,2,3}, UpTo[5]]` stays byte-identical
  to today's result — which is the INERT form
  `Take[List[1, 2, 3], UpTo[5]]` (VERIFIED 2026-07-17: stdlib
  `take_drop.rs` has no UpTo case; that WL-conformance gap is a recorded
  follow-up candidate, explicitly OUT of this brief's scope — do not
  implement it). Additionally: locate the builtin that consumes the
  kernel selector path emitting `UpTo::argx`/`innf`
  (`structure_data/selectors.rs:291-295` call sites), fire ONE live probe
  through it, and pin that its message emission is byte-identical after
  registration.
- Green `cargo fmt --all --check`; `cargo test --workspace` minus the 9
  known sandbox-blocked ZeroMQ loopback tests (run the suite, list the 9
  as the only failures, plus a `--exclude rum-jupyter` full-green run —
  the director's out-of-sandbox gate covers them).
- Live probes pasted: `Messages[General]`, `Messages[UpTo]`,
  `Take[{1,2,3}, UpTo[5]]` (expect the unchanged inert form
  `Take[List[1, 2, 3], UpTo[5]]`), `General = 5` (expect the protection
  message), one `UpTo`-message probe through its real consumer, and one
  resource message end-to-end (e.g. a small recursion overflow if
  cheaply reachable — otherwise state why skipped).

## rum / Rust pitfalls (read twice)

1. Registration must NOT change evaluation semantics: no handler, no
   dispatch — `General` and `UpTo[…]` stay inert. The ONLY behavior
   change is `Protected` (assignment now rejected) and non-empty
   listings. If any existing test fails for another reason, STOP and
   report — that is a finding, not ripple.
2. The `expr_runtime_helpers.rs:411` match and the selectors'
   `has_message` dedup are read-and-confirm items — do not modify them.
3. Reference regeneration only via the generator; double-generation
   `cmp` proof as in brief-249.
4. No timing gate (catalog data + inert symbols).
5. Stop-loss: any conflict between registration and existing resolution
   behavior → STOP with the evidence.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --all --check
cargo test --workspace                      # 9 known loopback failures only
cargo test --workspace --exclude rum-jupyter
cargo run -p rum-stdlib --bin generate-builtin-reference   # twice + cmp
```

Plus the live probes, all pasted verbatim.

## Hard rules

- No `git commit`/branch/push/config. Leave changes in the working tree.
- Nothing outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, the
  release `rum-cli`, `python3`.
- Testing Policy hard line (`AGENTS.md`). No new `unsafe`. Minimal diff.
- Verbatim outputs only; the 9 sandbox-blocked tests are reported as
  exactly that, never hidden.

## Report format (your final message)

1. What changed — the new module (both entries in full), wiring, and
   regeneration stats.
2. Verification — verbatim totals (with the 9 loopback failures called
   out), cmp proof, probes.
3. The 14 seeded pairs with per-pair evidence (site or Display impl).
4. Updated test expectations (old→new, verified).
5. Stop-loss / open questions (incl. what expr_runtime_helpers:411 is).
