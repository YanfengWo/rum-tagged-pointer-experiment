# Brief 167 — Strings usability fixes 4b: url/argv/encoding ergonomics

Implementation brief. Unit-1 fix batch 4b of the Usability Pass
(tracker `docs/dev/usability-pass.md`, triage item 4; evidence E6/E7
in `docs/dev/usability-pass/strings-findings.md`). All opt-in: every
currently-accepted call stays byte-identical; only rejected/inert
forms gain meaning, and new options default to today's behavior.

Design note: the audit proposed asymmetric option names for the URL
pair (`"SpaceEncoding"` on encode, `"PlusAsSpace"` on decode); triage
overrode that with ONE symmetric option name on both —
`"FormEncoding"` — because the concept is one wire format
(application/x-www-form-urlencoded), not two features.

## Changes

1. **`UrlEncode`/`UrlDecode` gain `"FormEncoding" -> False`**
   (rum-native). With `True`: `UrlEncode` emits spaces as `+` (plus
   signs still `%2B`); `UrlDecode` decodes `+` to space. Defaults
   (RFC 3986 percent-only, E7: `UrlEncode["a b+c"]` → `"a%20b%2Bc"`,
   `UrlDecode["a+b%20c"]` → `"a+b c"`) unchanged.
2. **`UrlParse` gains two options** (rum-native):
   - `"DefaultScheme" -> None`: when a string (e.g. `"https"`),
     schemeless input like `"example.com/a?x=1"` parses as if
     prefixed with that scheme; default `None` keeps today's strict
     `scheme://…`-only behavior.
   - `"DecodeQuery" -> False`: with `True`, query keys/values are
     percent-decoded and `+` becomes space (query strings are
     form-encoded by convention — document this in the catalog
     description). Default keeps raw text as today.
   Pin the existing output shape for a full URL FIRST (a test on the
   current association keys), then add the options.
3. **`ParseArgs` gains `"ShortOptions" -> False`** (rum-native). With
   `True`, single-dash tokens become clustered single-letter boolean
   flags: `"-abc"` → flags `a`, `b`, `c` (into the existing `"Flags"`
   field, same shape long flags use). No attached-value syntax for
   short options (folk scope; document it). Default: today's behavior
   (single-dash tokens are positional, E7).
4. **`ToCharacterCode` encoding argument** (WL-documented form):
   `ToCharacterCode[s, "UTF8"]` returns the UTF-8 BYTE list
   (`ToCharacterCode["é", "UTF8"]` → `{195, 169}`). Accept `"UTF-8"`
   as an alias. Other encoding names stay rejected-inert (pin one,
   e.g. `"Latin1"`). One-argument behavior (code points) unchanged.
5. **`StringToByteArray` accepts `"UTF-8"`** as an alias for the
   existing `"UTF8"` (E6 shows the hyphenated spelling is inert
   today). Unknown encodings keep today's inert behavior.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{UrlEncode["a b+c"], UrlEncode["a b+c", "FormEncoding" -> True], UrlDecode["a+b%20c"], UrlDecode["a+b%20c", "FormEncoding" -> True], ToCharacterCode["é"], ToCharacterCode["é", "UTF8"], ToCharacterCode["é", "UTF-8"], StringToByteArray["é", "UTF-8"], ToCharacterCode["é", "Latin1"]}'
printf '' | cargo run -q -p rum-cli -- --eval '{UrlParse["https://h/p?q=a%20b"], UrlParse["https://h/p?q=a%20b", "DecodeQuery" -> True], UrlParse["example.com/a?x=1", "DefaultScheme" -> "https"], UrlParse["example.com/a"], ParseArgs[{"-abc", "--out=x", "pos"}], ParseArgs[{"-abc", "--out=x", "pos"}, "ShortOptions" -> True]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1:
defaults byte-identical (`"a%20b%2Bc"`, `"a+b c"`, `{233}`), form
mode `"a+b%2Bc"` / `"a b c"`, byte list `{195, 169}` for both
spellings, `StringToByteArray` hyphen alias works, `"Latin1"` inert;
probe 2: full-URL default output unchanged (paste it), decoded query
shows `"a b"`, schemeless+`"DefaultScheme"` parses with scheme
`"https"`, schemeless WITHOUT the option keeps today's behavior,
`ParseArgs` default unchanged (E7 shape), short-options mode shows
flags `a`/`b`/`c` with `"pos"` positional and `out -> x` intact.

## Scope

- MAY modify: `crates/rum-stdlib/src/strings/scripting/**` (url/args
  files), the encoding implementation file(s) under
  `crates/rum-stdlib/src/strings/**` that own `ToCharacterCode`/
  `StringToByteArray`, `crates/rum-stdlib/src/strings.rs`,
  `crates/rum-stdlib/src/catalog/strings.rs` (descriptions),
  `crates/rum-runtime/src/runtime/options/defaults.rs` (register the
  new option defaults), `crates/rum-stdlib/src/tests/**`,
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/builtins.md`
  (regenerated if descriptions changed).
- MUST NOT touch: other families, the guide, `TODO.md`, the tracker,
  the findings file, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Every currently-accepted call byte-identical; changes only give
  meaning to currently-rejected forms plus options defaulting to
  today's behavior.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; the pinned pre-change `UrlParse` shape;
   any place the audit's expectation didn't match the code you found.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
