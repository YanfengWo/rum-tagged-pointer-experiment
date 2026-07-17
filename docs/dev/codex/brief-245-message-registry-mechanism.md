# Brief 245 — Builtin message-tag registry: mechanism + Set seed

## Goal

Give `BuiltinSpec` a static per-builtin message-tag declaration and merge it
into every catalog/listing surface that today reads only the user-defined
template store, so that a builtin's possible diagnostic tags are queryable
(`Messages[Set]` stops returning an empty list). This brief lands the
MECHANISM plus one seeded builtin (`Set`) to prove the pipe end-to-end; the
per-catalog fill sweeps, docgen pairing, and any emission-side enforcement
are LATER briefs — do not start them.

## Context pointers

- Read `AGENTS.md` first. Evidence base:
  `docs/dev/message-registry-inventory.md` (brief-243) — especially Table A
  rows for `Set`, Table B wrapper fan-in, and Sections E/F.
- Key files:
  - `crates/rum-runtime/src/runtime/metadata.rs:287` — `BuiltinSpec` (six
    fields, all const constructors initialize every field; const modifier
    builders like `.protected()` copy-preserve the rest).
  - `crates/rum-runtime/src/runtime/messages/message_catalog.rs` — the four
    read surfaces backed by `message_definitions` only:
    `message_names_expr_for_symbol` (~:239), `message_tags_expr_for_symbol`
    (~:326), `message_catalog_entries_for_symbol` (~:305),
    `message_catalog_entries_for_key` (~:316).
  - `crates/rum-runtime/src/runtime/messages/builtins_messages.rs` /
    `builtins_messages_inspect.rs` — the `Messages` builtin branches and
    `Information`'s `"Messages"` field, both funnel into the functions
    above.
  - `crates/rum-runtime/src/runtime/session/session_api.rs:149-181` —
    `register_builtin` stores full specs; `registered_builtin_specs`
    exposes them. This stored map is the registry lookup source at query
    time; no new global.
  - `crates/rum-runtime/src/runtime/builtin_catalog/rules.rs:25` — the
    `Set` catalog entry (the seed target).
  - `crates/rum-runtime/src/runtime/tests/messages.rs` — existing message
    tests (`inspect_message_catalog_exposes_templates_and_delayed_flags`
    at ~:299). `crates/rum-host/src/tests.rs` — host-side catalog tests.
- Acceptance is mechanical: the named tests below decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/metadata.rs` — add
    `pub struct BuiltinMessageSpec { pub tag: &'static str, pub template:
    &'static str }`; add `pub messages: &'static [BuiltinMessageSpec]` to
    `BuiltinSpec`; MECHANICAL: initialize `messages: &[]` in EVERY
    constructor, and make every modifier builder (incl.
    `with_fallback_dispatch`) preserve it; add a const `.with_messages(...)`
    builder.
  - `crates/rum-runtime/src/runtime/messages/message_catalog.rs` — merge
    registered-spec entries into the four read surfaces (details below).
  - `crates/rum-runtime/src/runtime/messages/builtins_messages.rs` and
    `builtins_messages_inspect.rs` — ONLY if a merge cannot live in the
    four functions and must sit at their call sites.
  - `crates/rum-runtime/src/runtime/builtin_catalog/rules.rs` — the `Set`
    entry gains `.with_messages(&[...])` (seed derivation below).
  - Tests: `crates/rum-runtime/src/runtime/tests/messages.rs` (new
    assertions + necessary expectation updates),
    `crates/rum-host/src/tests.rs` (one host catalog assertion).
  - Plus any EXISTING test whose pinned expectation legitimately changes
    because `Set`/`Information[Set]`/`Messages` output is no longer empty —
    update ONLY rows you have verified empirically against the new build,
    and list every such edit in your report (the brief-185/-210 "necessary
    ripple" rule).
- Files/areas you MUST NOT touch: everything else — `docgen.rs` and the
  generated reference (they must stay byte-identical; the drift test is
  your canary), `push_message` and all emission sites, stdlib catalogs,
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your
  own report, `.git`, `Cargo.lock`.
- No default-path cost: the registry is consulted ONLY inside the four
  query surfaces (cold inspection paths). `register_builtin` merely stores
  one more `Copy` field. `push_message` is untouched.

## Design (pinned — implement exactly)

1. **Merge semantics, all four surfaces:** result = user
   `message_definitions` entries ∪ builtin registry entries for the queried
   scope, deduplicated by `(symbol, tag)` with the USER entry winning
   (a user who defines `Set::argx = "…"` sees their template in catalog
   entries; the key appears once in listings). Output SHAPES are frozen:
   each surface returns exactly the same expression/struct shape it
   returns today, with more rows. Merged ordering must be deterministic —
   match the existing BTreeMap-derived ordering (sort merged rows by the
   same key order).
2. **Builtin catalog entries** map to `MessageCatalogEntry` with
   `template = Expr::string(spec_template)` and `delayed = false`.
3. **Registry lookup**: from the session's stored registered specs (the
   `register_builtin` map). Symbol-scoped queries look up that symbol's
   spec; `None`/all-scope queries iterate all registered specs.
4. **Seed = `Set` only.** Derive the candidate tag list for `Set` from the
   inventory doc (its direct sites AND wrapper-attributed tags such as
   `argx`; include the protected-write tag if the inventory attributes it
   to `Set`). VERIFY every candidate by firing it live on the release CLI
   (e.g. `Set[]` for `argx`; a write to a `Protected` symbol for the
   protection tag) and seed EXACTLY the verified set — paste each probe
   transcript in your report. Do NOT seed `General` (not a registered
   builtin; carrier decision belongs to the fill design — note it as an
   open question if you hit it).
5. **Template strings** use WL-style backquote placeholders `` `1` ``,
   `` `2` `` … for interpolated values in argument order (e.g. `Set::argx`
   → ``"expected exactly `1` argument(s), got `2`"`` if that matches the
   real site's interpolation); where a site's text is fully dynamic, use a
   concise static description of what the message reports. The template is
   documentation-grade truth for LISTING surfaces only — rendering of
   emitted messages is untouched.

## Deliverable

- The mechanism + Set seed, with new tests pinning:
  - `Messages[Set]` returns the seeded tag list in MessageName form
    (non-empty; exact expected value from your build, pasted);
  - a user override `Set::argx = "custom"` dedups to one row with the
    user template winning in catalog entries;
  - `Information`'s `"Messages"` field for `Set` is non-empty;
  - `Session::inspect_message_catalog(Some("Set"))` includes builtin
    entries with `delayed = false` (and the rum-host mapping surfaces
    them as `HostMessageTemplate`);
  - the no-argument `Messages[]` shape is unchanged in kind (strings,
    deduplicated) and now includes seeded tags;
  - a user-only symbol (`f::tag = "…"`) behaves exactly as before.
- Green `cargo fmt --all --check` and `cargo test --workspace` (the
  reference drift test green = docgen byte-stability proof).
- Report lists every pre-existing test expectation you had to update, with
  the empirical verification for each.

## rum / Rust pitfalls (read twice)

1. `BuiltinSpec` is `Clone, Copy, PartialEq, Eq` — the new field must keep
   that (a `&'static [BuiltinMessageSpec]` does; derive the same traits on
   `BuiltinMessageSpec`).
2. Every `BuiltinSpec` constructor initializes ALL fields — adding the
   field without updating one constructor is a compile error; updating all
   of them is the point (grep `-> Self` blocks in metadata.rs; also
   `with_fallback_dispatch` must carry `messages` through).
3. Do not change what `push_message` or any emission site does; this brief
   is read-surface only.
4. Do not "helpfully" seed more builtins or touch docgen — the fill sweep
   and docgen pairing are separate briefs with their own gates.
5. Shapes are contracts: `Messages[f]` stays a `List` of MessageName
   expressions; `Information`'s field stays a `List` of tag strings; the
   host entry struct is unchanged. Pin with tests before relying on them.
6. No timing gate: no eval-hot-path code changes here (inspection surfaces
   only). If you find yourself editing anything on the evaluation path,
   STOP — that is out of scope.
7. Stop-loss: if the merge cannot be expressed inside the four functions
   without changing a public shape, STOP and report the conflict instead
   of redesigning.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --all --check
cargo test --workspace
```

Plus the live probes for the Set seed derivation (release CLI, stdin from
/dev/null), pasted verbatim, and a before/after transcript of:

```
target/release/rum-cli --eval 'Messages[Set]' </dev/null
target/release/rum-cli --eval 'Information[Set]' </dev/null
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in
  the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope; STOP and report if you believe you
  must.
- Full machine access, but touch NOTHING outside this repo, no network.
  Only file edits within Scope, `cargo`, the release `rum-cli`, and
  `python3` for parsing.
- Testing Policy hard line (`AGENTS.md`): tests verify rum's own documented
  behavior; no external-engine comparisons.
- No new `unsafe`.
- Minimal diff: no drive-by reformatting, renames, or comment edits.
- Paste real verbatim verification output; if something did not reach
  green, say so explicitly.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim fmt/test totals, the Set-seed probe
   transcripts, and the before/after `Messages[Set]`/`Information[Set]`
   outputs.
3. The seeded Set tag set with each tag's evidence line from the inventory
   doc + its live probe.
4. Every pre-existing expectation updated, each with its empirical check.
5. Stop-loss / open questions (e.g. the `General` carrier note).
