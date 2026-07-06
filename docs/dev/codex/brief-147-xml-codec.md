# Brief 147 — XML codec (`FromXml` / `ToXml`) on a symbolic element tree

Implementation brief. Third step of `docs/dev/codec-family-plan.md`. XML's
data model is a tree, not maps — the natural rum representation is a
symbolic head: `XmlElement[tag, attrs, children]`. Pure, zero
dependencies, fail-closed; deliberately NO DTD processing (the XXE lesson:
external/parameter entities are a security hazard — rejecting `<!DOCTYPE`
outright is a feature; document it as such).

## Value model

`XmlElement[tag_String, attrs_Association, children_List]` — attrs keyed
by String attribute names (insertion order), children a mix of
`XmlElement[…]` and `String` text nodes. `XmlElement` itself is an inert
constructor head: register it as a documented handler-less symbol if the
registry supports that (the brief-137 pattern-atom precedent); otherwise
guide-only — report the choice.

## Decode contract (`FromXml[str]` → `XmlElement[…]`)

- Supported: optional XML declaration (validated, not represented);
  elements; attributes (single- or double-quoted); self-closing elements;
  the five predefined entities + numeric `&#N;`/`&#xH;`; CDATA sections
  (become text); comments DROPPED (documented — codecs are
  value-oriented; the span-oriented counterpart is `TokenizeWith`).
- Whitespace policy (folk, documented): text nodes that are entirely
  whitespace between elements are dropped; any text node containing
  non-whitespace is kept VERBATIM (no trimming). Adjacent text/CDATA runs
  merge into one String.
- Namespaces are NOT resolved: prefixed names (`ns:tag`) stay verbatim in
  tag/attribute Strings — document.
- Fail-closed with byte offsets: `<!DOCTYPE` (explicitly, with a message
  saying DTD is deliberately unsupported), mismatched/unclosed tags,
  duplicate attributes on one element, unquoted attribute values, unknown
  named entities, bare `&`/`<` in text, multiple root elements, non-
  whitespace after the root, processing instructions other than the XML
  declaration.

## Encode contract (`ToXml[element]` → String)

- Input must be a well-formed `XmlElement` tree (String children allowed
  anywhere). Text and attribute values are entity-escaped; empty-children
  elements self-close; attributes in Association order.
- Tag/attribute names validated against a folk XML-Name subset (start:
  ASCII letter, `_`, or `:`; then letters, digits, `_`, `-`, `.`, `:`) —
  anything else fails closed.
- Compact by default; `ToXml[element, "Indent" -> n]` pretty-prints
  (indentation only around element-only content; mixed content —
  element+text siblings — never reformats, or the round-trip would break;
  document).
- Anything that is not `XmlElement`/String in the tree → fail-closed.

## Round-trip contract

`FromXml[ToXml[x]] === x` for every tree ToXml accepts (compact mode; the
whitespace-drop policy makes this exact). Pin with tests, incl. a
mixed-content case.

## Scope

- MAY create: `crates/rum-stdlib/src/xml.rs` (+ submodules),
  `crates/rum-stdlib/src/catalog/xml.rs`, tests
  `crates/rum-stdlib/src/tests/codecs_xml.rs`.
- MAY modify: catalog mod/install path, `stdlib_attributes.rs` vocabulary,
  docgen if needed, `docs/reference/` via generator,
  `docs/guide/structured-data.md` (XML section: value model, whitespace
  policy, the no-DTD security stance, transcripts).
- MUST NOT touch: other codecs, kernel crates beyond vocabulary/inert-
  symbol registration, `Cargo.*`, evaluator/dispatch.

## Tests

- Realistic document end-to-end (nested elements, attributes both quote
  styles, self-closing, entities, CDATA, comment dropped, whitespace-only
  text dropped, mixed content kept verbatim) asserting the full tree.
- Unicode text/attrs; numeric entity pins (`&#x4E16;` → 世).
- Fail-closed negatives incl. the explicit `<!DOCTYPE` rejection message,
  mismatched tags with offset, duplicate attribute, unknown entity.
- Encode: escaping pins, self-closing, name validation negatives, Indent
  mode + the mixed-content no-reformat rule; round-trip pins compact and
  a FromXml∘ToXml∘FromXml idempotence check on the realistic doc.
- Cross-codec: extract values from a parsed config-ish XML and `ToJson`
  them.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib tests::builtin_reference_docgen::builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect 592 (+1 if XmlElement gets its own entry — report)
printf '' | cargo run -q -p rum-cli -- --eval 'FromXml["<cfg env=\"dev\"><db host=\"h\" port=\"5432\"/><note>a &amp; b</note><![CDATA[<raw>]]></cfg>"]'
printf '' | cargo run -q -p rum-cli -- --eval 'ToXml[XmlElement["a", <|"x" -> "1<2"|>, {XmlElement["b", <||>, {}], "t&t"}]]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromXml[ToXml[XmlElement["r", <||>, {"mixed ", XmlElement["i", <||>, {"x"}], " tail"}]]]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromXml["<!DOCTYPE x><x/>"]'      # fail-closed, DTD message
printf '' | cargo run -q -p rum-cli -- --eval 'FromXml["<a><b></a>"]'            # fail-closed, mismatch offset
printf '' | cargo run -q -p rum-cli -- --eval 'ToJson[<|"port" -> FromXml["<c><p>8080</p></c>"][[3, 1, 3, 1]]|>]'
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- NO DTD, no entity definition processing of any kind — reject, never
  expand. Namespaces verbatim.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The contract as landed + the XmlElement registration choice.
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
