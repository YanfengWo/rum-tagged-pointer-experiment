# Brief 270 — Sealed packages, batch 2: hidden-by-default introspection

## Goal

Implement the batch-2 visibility rules of
`docs/dev/sealed-scoping-design.md` §4 (2026-07-19 version): sealed
private symbols (identity in the bit-62 region) are absent from every
in-language enumeration and inspection surface unless the session's
capability policy allows `Capability::Diagnostic`. Batch 1 made sealed
bindings unreachable by name; this batch makes them unenumerable and
their bodies undisclosed. Plus one batch-1 hardening item: the
registration plan's `replacement_index` must be re-located by package
context at finish/fail time instead of trusted from pre-install capture.
When you are done: with a policy that lacks `Diagnostic`, no `Names`/
`Contexts`/`Definition`/`Information`/completion/message-catalog/`Save`/
`DumpSave` result mentions a sealed symbol or discloses its definitions;
with `Diagnostic` allowed, all of it is visible (the owner/debug view).
The host textual snapshot path stays byte-identical (its sealed-aware
redesign is batch 3, not this brief).

## Context pointers

- Read `AGENTS.md`, `docs/dev/sealed-scoping-design.md` §4 + §7, and
  `docs/dev/sealed-scoping/grounding-findings.md` **Q5** — the Q5 table
  is the authoritative site map for this brief (file:line anchors for
  every filter point). Batch-1 code: `runtime/scoping/packages.rs`
  (`SEALED_IDENTITY_MARKER`, `SourceUnit`, the registry).
- Central helpers to add in `packages.rs` and use EVERYWHERE (no local
  reimplementations):
  - `fn symbol_is_sealed_private(symbol: &Symbol) -> bool` — identity
    present AND `identity & SEALED_IDENTITY_MARKER != 0` (the same
    predicate `mark_symbol_removed` uses; refactor that site onto the
    helper).
  - `Session::sealed_symbols_visible(&self) -> bool` —
    `self.capability_policy.allows(Capability::Diagnostic)`.
- Key filter sites (from grounding Q5; verify each anchor before
  editing):
  - `namespace_tools/names.rs` — `names_matching_patterns` (~186): skip
    sealed symbols at the loop start (~195), BEFORE the
    `explicit_context` branch — this one point covers ordinary `Names`,
    explicit-context `Names`, and `resolve_names_matching_expr` (the
    wildcard feeder for `Clear`/`Remove`/`Protect`/`Unprotect`).
  - `builtins_context_loader.rs` — `eval_contexts` (~102-168): skip the
    context insertion for sealed symbols in every symbol-derived
    extraction (definitions, attributes, options, syntax_information,
    declared_stubs, removed_symbols, message_definitions).
  - `messages/builtins_messages_inspect.rs` — `eval_definition` (~4)
    and `eval_information` (~16): after target normalization, a sealed
    target without visibility renders exactly as an undefined symbol
    does (mirror the existing no-definition presentation; do not invent
    a redaction marker).
  - `messages/message_catalog.rs` — the merged catalog collection
    (`merged_message_catalog_sources` per Q5, ~245-390): skip
    sealed-keyed message entries; covers `Messages[]`/`MessageTags[]`/
    `MessageDefinitions[]`.
  - `session/session_api.rs` — completion candidates (~374-453,
    `insert_completion_candidate`): skip sealed symbols and do not
    contribute their contexts as completion candidates.
  - `package_tools.rs` — `symbols_in_context` (~271-359) feeds both
    `Save`/`DumpSave` AND (per grounding Q6) the host snapshot
    renderer. **Apply the sealed filter at the in-language
    `Save`/`DumpSave` call sites (or via a filtered wrapper), NOT
    inside the shared helper** — the host snapshot's sealed handling is
    batch 3, and its current output must stay byte-identical (pinned by
    the existing host snapshot round-trip tests; if none covers a
    sealed-containing session, do NOT add one — snapshot semantics for
    sealed state are explicitly out of scope).
- Hardening item (batch-1 edge): in `packages.rs`,
  `finish_package_install` and `fail_package_install` currently index
  `sealed_units` with the pre-install `replacement_index`; a body form
  that itself registers/unregisters OTHER packages can stale it.
  Re-locate the unit's position by `package_context` at finish/fail
  time (drop the stored index or keep it only as a hint); add the
  nested-registration test below.
- Explicit NON-goals (ruled by the director; do not implement):
  - `SymbolName`/`Context` on a held sealed symbol stay unchanged
    (display already prints the readable qualified name; the boundary
    is reach/enumeration, not the name of a value you already hold).
  - `NameQ` stays unchanged — resolution already cannot see sealed
    symbols, so it truthfully returns False; pin with a test, change
    nothing.
  - Host-facing summary/context/inspection APIs (`session_api.rs`
    definition summaries, `rum-host` session accessors) are a TRUSTED
    embedder surface — no filtering; the in-language builtins are the
    governed surface.
  - No display/`Definition`-rendering polish (batch 3), no snapshot
    changes (batch 3), no new capability, no new messages.
- Acceptance is mechanical: the named tests + `cargo test --workspace`
  + the microbench runs decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/scoping/packages.rs` (helpers +
    the replacement_index hardening)
  - `crates/rum-runtime/src/runtime/namespace_tools/names.rs`
  - `crates/rum-runtime/src/runtime/builtins_context_loader.rs`
  - `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs`
  - `crates/rum-runtime/src/runtime/messages/message_catalog.rs`
  - `crates/rum-runtime/src/runtime/messages/builtins_messages.rs`
    (only if the catalog filter's seam is there)
  - `crates/rum-runtime/src/runtime/session/session_api.rs`
    (completion only)
  - `crates/rum-runtime/src/runtime/package_tools.rs` (Save/DumpSave
    call-site filter only)
  - `crates/rum-runtime/src/runtime/definitions/definition_clear.rs`
    (MECHANICAL: refactor the existing guard onto the shared predicate)
  - `crates/rum-runtime/src/runtime/tests/sealed_packages.rs` (extend)
    or a new `crates/rum-runtime/src/runtime/tests/sealed_visibility.rs`
    + `crates/rum-runtime/src/runtime/tests/mod.rs`
  - CONDITIONAL, MECHANICAL ONLY (report each use): existing tests that
    pin `Names`/`Contexts`/completion/catalog outputs, if a count
    changes as a mechanical consequence.
- MUST NOT touch: `crates/rum-core/**`, `resolution.rs`,
  `builtin_adapters.rs`, `builtin_catalog/**`, session field files
  (no new state beyond what batch 1 added), `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, `docs/**`, `docs/dev/codex/**` except your report, `.git`,
  `Cargo.lock`.
- No unconditional default-path cost: every filter runs on
  introspection paths only (cold); the predicate is one identity-bit
  test per enumerated symbol. Nothing on the evaluation hot path
  changes.

## Deliverable

- The visibility filters + helpers + hardening above.
- Tests (each in both policy variants where marked; construct a
  restricted policy via the existing `CapabilityPolicy` API — determine
  empirically what the default test session allows and state it in the
  report):
  1. `names_hides_sealed` / `names_reveals_with_diagnostic`: after
     registering the probe package, ``Names["Pkg`Private`*"]`` and a
     bare `Names["*helper*"]` contain nothing sealed without
     `Diagnostic`, and contain the readable sealed name with it.
  2. `contexts_hides_private_context` / reveal twin: `Contexts[]`
     omits/includes `` "Pkg`Private`" ``.
  3. `definition_hides_sealed_body` / reveal twin: `Definition` called
     (Rust-constructed call) on the registry's sealed symbol renders as
     an undefined symbol without `Diagnostic`, full body with it. Also
     pin: `Definition["Pkg`Private`helper"]` (string target) shows
     nothing sealed regardless (it normalizes to the decoy).
  4. `information_hides_sealed` / reveal twin: same shape as 3.
  5. `completion_hides_sealed` / reveal twin: the completion API's
     candidates exclude/include the sealed name and the private
     context.
  6. `message_catalog_hides_sealed` / reveal twin: a package body
     defining a message on a private symbol; `Messages[]` and
     `MessageTags[]` exclude/include it.
  7. `save_excludes_sealed`: in-language `Save`/`DumpSave` over the
     package context emits no sealed definitions (assert on the emitted
     text); AND the host snapshot bytes for a sealed-containing session
     are IDENTICAL before/after this brief's change (capture via
     `snapshot_definitions_source` in a Rust test — this pins that you
     did not touch the shared path).
  8. `nameq_stays_false`: ``NameQ["Pkg`Private`helper"]`` is False (no
     code change; pin).
  9. `wildcard_ops_skip_sealed`: ``Clear["Pkg`Private`*"]`` and
     wildcard `Unprotect` leave the sealed definitions intact (they no
     longer even enumerate).
  10. `nested_registration_keeps_registry_consistent` (hardening): a
      package body whose form re-registers ANOTHER earlier-registered
      package (successfully) while the outer registration later fails —
      after the dust settles, `sealed_units` contains exactly the
      expected units keyed by the right contexts (no wrong-slot
      overwrite, no panic). Plus the plain re-registration test from
      batch 1 still green.
  11. `decoy_visibility_is_callers_own`: after assigning the spelled
      decoy, `Names` shows the decoy's name even without `Diagnostic`
      (it is the caller's ordinary symbol) — pins the discrimination.

## rum / Rust pitfalls (read twice)

1. **One predicate, one gate.** Every site uses
   `symbol_is_sealed_private` + `sealed_symbols_visible`; if you find
   yourself writing `1 << 62` at a call site, stop and use the helper.
2. **Filter placement in `names_matching_patterns` matters**: before
   the `explicit_context` branch, or the explicit-context path leaks.
3. **Do not touch the shared save/snapshot helper's behavior** — the
   host snapshot byte-parity test (deliverable 7) is the tripwire.
4. **The `Definition`/`Information` gate sits after normalization,
   before rendering** — string targets already land on decoys; the gate
   exists for Rust-side/held sealed exprs, and the hidden rendering
   mirrors the undefined-symbol presentation exactly.
5. **No new session state, no new messages, no new capability.**
6. **Stop-loss is real.** Same failure twice → STOP, report, leave the
   tree clean.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Note: the 9 rum-jupyter native ZeroMQ loopback tests fail under the
dispatch sandbox (deterministic loopback-bind denial, known since
brief-250); if they are your ONLY failures, state so explicitly — the
director's out-of-sandbox gate covers them. Everything else must be
green.

Then the microbench, twice, on your candidate (noise documentation; the
true base-vs-candidate A/B is the director's gate step):

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief270_run1
bash scripts/core-language-microbench.sh --output-dir /tmp/brief270_run2
```

Paste the per-label rows for the scoping/dispatch tier and the list
tier (the director is re-observing batch 1's layout-lottery labels:
list-literal-small, sameq-list, part-list, length-list, head-list).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report instead.
- Nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts, `python3`.
- Testing Policy hard line (`AGENTS.md`); no new `unsafe`; minimal
  diff; verbatim outputs only, never faked.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim: fmt, workspace totals (with the jupyter
   caveat if hit), the two microbench row sets.
3. Verdict — per-surface hide/reveal behavior summary; the default
   test-session policy you observed; anything deviating from this
   brief's pinned spec.
4. Stop-loss / open questions (empty if none); CONDITIONAL-file uses
   with forcing errors.
