# Brief 213 — Name-pattern targets + `Information` inspection (unit-8 fix batch 5, closing)

Implementation brief. The last two unit-8 fix batches, packaged. Read
`AGENTS.md` (Testing Policy) and the unit-8 findings
(`docs/dev/usability-pass/kernel-a-findings.md`, the `Clear`/`Remove`/
`Information` rows, E6 and E8) first. Both gaps are re-verified as still live
on the current tree (probes below) — batches 1-4 did not change them.

## Fix 1 — Name-pattern targets (one shared fix point, five heads)

Today a wildcard string is treated as a LITERAL symbol name, so the call is a
silent no-op:

```
$ printf '' | cargo run -q -p rum-cli -- --eval 'zz1 = 5; zz2 = 6; Clear["Global`*"]; Print[{zz1, zz2}]'
{5, 6}            # nothing cleared, no message
```

`Remove["Global`*"]` is the same, and `Protect["Foo`*"]` protects a symbol
literally named `*`. All five heads share ONE normalizer:
`normalize_clear_targets`
(`crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:117`)
— used by `Clear`/`ClearAll`/`Remove` (`definition_clear.rs:35`, `:68`) and,
via `normalize_multi_symbol_targets` (`attributes.rs:200`), by
`Protect`/`Unprotect`. Fix the normalizer once and all five gain pattern
support.

The matcher already exists and works — do NOT write a new one. `Names` uses
it: `Names["Global`*"]` → `{"Global`$ScriptInput", "Global`aa1", "Global`aa2"}`,
`Names["aa*"]` → `{"aa1", "aa2"}`. Reuse the wildcard matching in
`crates/rum-runtime/src/runtime/namespace_tools/names.rs`
(`wildcard_match_ends` / `wildcard_chars_match`) and the same name-enumeration
`Names` walks.

Contract:
- A string arg CONTAINING a wildcard expands against existing symbol names to
  a target list. A string arg with NO wildcard keeps resolving to exactly that
  one symbol (today's behavior — do not regress `Clear["ww"]`).
- **Precompute the whole target set before any mutation**, then apply the
  existing per-target protection check for each (a protected match emits its
  `wrsym`-class message and is skipped; unprotected matches are cleared). Do
  not abort the whole call on one protected match — that is the existing
  per-target behavior; keep it.
- A pattern matching nothing is a silent no-op (WL-like). It must NOT create
  or resolve to a literal symbol named with the wildcard.
- Symbol args and list args keep working unchanged.

## Fix 2 — `Information`: hold its target, real usage, message tags

Three sub-gaps, all in `Information`:

**(a) It evaluates its argument, so the direct-symbol form is useless:**

```
$ printf '' | cargo run -q -p rum-cli -- --eval 'xq = 5; Information[xq]'
Information[5]          # inert; the string form Information["xq"] works
```

`Definition` already gets this right — it carries `{HoldFirst, Protected}` and
`Definition[xq]` correctly returns `HoldComplete[Set[xq, 5]]`, while
`Information` carries only `{Protected}`. Give `Information` `HoldFirst`,
matching its sibling. This is a justified attribute addition (it fixes a real
bug and matches an existing in-tree precedent), not a WL-table transcription.

Because it is an attribute change: sync the runtime attribute mirror
(`builtin_catalog/stdlib_attributes.rs` if applicable) and REGENERATE the
reference (`cargo run -p rum-stdlib --bin generate-builtin-reference`) — the
drift test enforces it (the brief-210 lesson).

**(b) The `Usage` field for a builtin is just the qualified name:**

```
$ printf '' | cargo run -q -p rum-cli -- --eval 'Information["Set"]'
… Rule["Usage", "System`Set"] …          # not the builtin's summary
```

Every builtin carries a `summary` in its `BuiltinSpec` (the same text the
reference generator prints). Use it as the `Usage` fallback for a builtin with
no user-defined usage message. The field is built in
`usage_message_expr_for_symbol`
(`crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:248`).

**(c) `InformationData` has no message field.** Add one (e.g.
`Rule["Messages", …]`) listing the symbol's STORED message tags. This works
immediately for user-defined messages (`f::tag = "…"`); for builtins it will
be empty — see the deferral below, and do not fake it.

## Explicitly DEFERRED (do not build; state it in your report)

**A builtin message-tag registry.** `Messages[Set]` / `MessageTags[Set]` are
empty even right after `Set::argx` fires, because builtin diagnostics are
emitted ad hoc from ~108 `push_message` call sites with no static catalog of a
builtin's possible tags/templates. Building that registry is its own project
(a design + a mechanical sweep of every call site) and it pairs naturally with
docgen — the builtin reference should list each builtin's message tags from
the same source. It is OUT OF SCOPE here. Fix 2(c) adds the field so a future
registry lights it up for free; builtins showing an empty message list today
is the honest state.

## Tests

- `Clear["Global`*"]` clears matching user symbols; a `Protected` match emits
  its `wrsym` message and survives while unprotected matches are still
  cleared; `Clear["ww"]` (no wildcard) still clears exactly `ww`; a pattern
  matching nothing is a silent no-op and creates no symbol.
- The same pattern form for `Remove` and for `Protect`/`Unprotect` (assert a
  wildcard `Protect` protects the matched symbols, not a symbol named `*`).
- `Information[xq]` with `xq = 5` inspects the SYMBOL (not `5`);
  `Information["xq"]` still works.
- `Information["Set"]` (or any builtin) shows the catalog summary as `Usage`.
- The `Messages` field lists a user-defined `f::tag`; it is empty for a
  builtin (pin the current honest behavior).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo run -p rum-stdlib --bin generate-builtin-reference   # after the attribute change
cargo fmt --check
cargo test --workspace        # run to COMPLETION (it stops at the first failing crate)
```

Then the microbench A/B (the `Information` attribute change touches
attribute-driven call planning):

```
bash scripts/core-language-microbench.sh --output-dir /tmp/mb_base_213   # BASE
bash scripts/core-language-microbench.sh --output-dir /tmp/mb_new_213    # candidate
```

Expectation, stated so you do not misread noise: attributes are a PER-SYMBOL
lookup and the panel never calls `Information`, so this should be FLAT. Paste
the per-label rows anyway; if a label appears to move, check whether the
UNAFFECTED labels moved as much (that is the noise floor) before claiming a
regression. Do not "fix" noise.

Behavior probes (paste verbatim):

```
printf '' | cargo run -q -p rum-cli -- --eval 'zz1 = 5; zz2 = 6; Clear["Global`*"]; Print[{zz1, zz2}]'
# expect both cleared (inert symbols)
printf '' | cargo run -q -p rum-cli -- --eval 'ww = 7; Clear["ww"]; Print[ww]'
# expect ww inert (no regression on the exact form)
printf '' | cargo run -q -p rum-cli -- --eval 'xq = 5; Information[xq]'
# expect the InformationData for the SYMBOL xq, not Information[5]
printf '' | cargo run -q -p rum-cli -- --eval 'Information["Set"]'
# expect Usage to be Set's catalog summary, not "System`Set"
```

## Scope

- MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs`
    (the shared normalizer + pattern expansion).
  - `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs`
    (`Information` usage + the messages field).
  - The catalog entry for `Information` (`HoldFirst`) + the attribute mirror.
  - `docs/reference/builtins.md` — REGENERATED only (never hand-edited).
  - Test files near the existing clear/protect/information tests.
- MUST NOT touch: the `Names` matcher's own semantics, the `push_message` call
  sites (the deferred registry), `TODO.md`, the tracker, the findings file,
  `Cargo.*`, git.

## Hard rules

- No `git commit`, branch, push, or git config.
- No new `unsafe`; `rum-core`/`rum-runtime` stay zero-`unsafe`.
- Reuse the existing wildcard matcher — do NOT write a second one.
- Precompute the target set before mutating; keep the existing per-target
  protection checks (a protected match is skipped with its message, not a
  whole-call abort).
- A no-match pattern is a silent no-op — it must never create a literal
  wildcard-named symbol.
- Do NOT build the builtin message registry; do not fabricate builtin message
  tags to fill the new field.
- Attribute change ⇒ mirror sync + reference regeneration (the drift test will
  catch a miss).
- Minimal diff; no drive-by reformatting. Paste real verbatim output; if the
  suite is not green, say so.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — workspace totals, fmt, reference regenerated, the A/B rows
   (with your noise-floor reasoning), and the four behavior probes, verbatim.
3. Confirmation that the exact-name forms (`Clear["ww"]`, symbol args) are
   unregressed, and that a no-match pattern creates no symbol.
4. Stop-loss / open questions (empty if none).
