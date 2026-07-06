# Brief 168 — Strings usability fixes 5 (unit-1 final): fail-closed strict modes

Implementation brief. Unit-1 fix batch 5 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, triage item 5; evidence E3 in
`docs/dev/usability-pass/strings-findings.md`). Four builtins whose
malformed/edge inputs are silently inert today get an explicit failure
path. Three take an opt-in `"Strict"` option (defaults unchanged); one
gets the WL-parity message.

## Changes

1. **`TemplateApply` gains `"Strict" -> False`** (string-named rum
   extension on a WL name). With `True`: any unfilled slot (named slot
   absent from the association, or positional slot out of range) emits
   a `TemplateApply::slot`-style message (pick the tag consistent with
   existing message conventions; report it) and returns `$Failed`.
   Default: today's behavior byte-identical (E3:
   `TemplateApply["`name`", <||>]` stays inert).
2. **`Uncompress` gains `"Strict" -> False`**. With `True`: a
   malformed/corrupt envelope emits a message and returns `$Failed`.
   Default: inert, unchanged (E3: `Uncompress["not-an-envelope"]`).
3. **`HtmlUnescape` gains `"Strict" -> False`**. With `True`: a
   malformed entity (e.g. `&#xZZ;`, `&#999999999;`, unterminated
   `&amp`) emits a message and returns `$Failed`. Default: preserve
   the malformed text verbatim, unchanged (E3).
4. **`HammingDistance` on unequal-length inputs emits a message**
   (WL-parity: WL messages this case) and stays INERT as today — no
   option; the message is the fix. Equal-length behavior untouched.

Register the three `"Strict"` defaults in the options table
(`crates/rum-runtime/src/runtime/options/defaults.rs`). Update catalog
descriptions and the `docgen.rs` LIMITS notes where they state the old
contract (the `Uncompress`/`TemplateApply`/`HtmlUnescape` notes may
claim silent-inert is the only behavior — keep them true); regenerate
the reference.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# strict paths: messages + $Failed
printf '' | cargo run -q -p rum-cli -- --eval '{TemplateApply["`name`", <||>, "Strict" -> True], Uncompress["not-an-envelope", "Strict" -> True], HtmlUnescape["&#xZZ;", "Strict" -> True], HammingDistance["ab", "a"]}'
# defaults byte-identical
printf '' | cargo run -q -p rum-cli -- --eval '{TemplateApply["`name`", <||>], Uncompress["not-an-envelope"], HtmlUnescape["&#xZZ;"], HammingDistance["ab", "ab"], TemplateApply["`x`", <|"x" -> 1|>, "Strict" -> True], HtmlUnescape["&amp;", "Strict" -> True]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 shows
three messages + three `$Failed` and the new `HammingDistance` message
with the call staying inert; probe 2 shows today's outputs unchanged
(inert, inert, `"&#xZZ;"`, `0`) and the two strict-mode calls on VALID
input returning the normal results (`"1"`, `"&"`).

## Scope

- MAY modify: the implementation files under
  `crates/rum-stdlib/src/strings/**` owning the four builtins,
  `crates/rum-stdlib/src/strings.rs`,
  `crates/rum-stdlib/src/catalog/strings.rs`,
  `crates/rum-stdlib/src/docgen.rs` (LIMITS notes),
  `crates/rum-runtime/src/runtime/options/defaults.rs`,
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  `docs/reference/builtins.md` (regenerated).
- MUST NOT touch: `Compress`, other families, the guide, `TODO.md`,
  the tracker, the findings file, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Behavior changes are EXACTLY: (a) the three `"Strict" -> True` paths
  (message + `$Failed` on invalid input; normal results on valid
  input), (b) the `HammingDistance` unequal-length message (return
  value still the inert call). All defaults byte-identical.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; the message tags chosen.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
