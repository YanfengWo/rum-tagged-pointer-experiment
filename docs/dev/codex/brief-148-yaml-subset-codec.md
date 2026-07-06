# Brief 148 — YAML sane-subset codec (`FromYaml` / `ToYaml`)

Implementation brief. Last codec of `docs/dev/codec-family-plan.md`, and
the one where fail-closed matters most: full YAML is a PERMANENT non-goal;
the subset below is the whole contract, and everything outside it rejects
loudly. Pure, zero dependencies.

## Decode contract (`FromYaml[str]`)

Supported (the sane block subset):
- block mappings (`key: value`), block sequences (`- item`), arbitrary
  nesting by indentation — SPACES only (a tab anywhere in indentation
  fails closed with its offset);
- plain (unquoted) scalars, single-quoted (with `''` escape),
  double-quoted (JSON-style escapes incl. `\u`);
- literal block scalars `|` (clip) and `|-` (strip); NO folded `>` (fail
  closed);
- comments (`#`, full-line or after a scalar with preceding space); blank
  lines;
- EXACTLY two flow forms, needed for round-trip: `{}` (empty mapping) and
  `[]` (empty sequence) as scalar-position values. All other flow syntax
  (`[a, b]`, `{k: v}`) fails closed.

Scalar typing — pinned, Norway-safe:
- unquoted `true`/`false` → `True`/`False`; `null`, `~`, or empty value →
  `Null`; decimal integers → exact Integer; decimal/exponent numbers →
  exact base-10 Rational (family philosophy); EVERYTHING else unquoted →
  String — including `no`, `yes`, `on`, `off`, `No`, `NORWAY`-adjacent
  words (document: rum does not implement the YAML 1.1 boolean zoo);
- any quoted scalar → String, always.

Fail-closed with offsets: anchors `&`, aliases `*`, tags `!`, directives
`%`, document markers `---`/`...` (single-document only), merge keys
`<<:`, duplicate keys in a mapping, inconsistent indentation, folded `>`,
non-empty flow collections, a sequence item and mapping key at the same
indentation ambiguity — reject, never guess.

## Encode contract (`ToYaml[value]` → String)

- Associations (String keys) → block mappings; Lists → block sequences;
  2-space indentation; insertion order preserved; nested empties emit the
  two flow forms `{}` / `[]`.
- Scalars: Integer and terminating Rational as numbers (non-terminating →
  fail closed, as the family does); `True`/`False`/`Null` → `true`/
  `false`/`null`; Strings emit PLAIN only when they match the safe-plain
  rule — nonempty ASCII, starts with a letter, contains only letters,
  digits, spaces, `_ . / -`, no leading/trailing space, and is not (case-
  insensitively) one of `true/false/null/yes/no/on/off/~` and does not
  parse as a number — otherwise double-quoted with escapes. Multiline
  Strings emit as `|-`/`|` blocks (choose by trailing newline; document).
- Mapping KEYS: same safe-plain rule, else double-quoted.
- Anything else (symbols, calls, non-String keys) → fail-closed.

## Round-trip contract

`FromYaml[ToYaml[x]] === x` for every value ToYaml accepts (the typing
rules and safe-plain quoting are designed to make this exact — the tests
must prove it, including strings like `"no"`, `"3.14"`, `"null"`, and
multiline strings with and without trailing newline).

## Scope

- MAY create: `crates/rum-stdlib/src/yaml.rs` (+ submodules),
  `crates/rum-stdlib/src/catalog/yaml.rs`, tests
  `crates/rum-stdlib/src/tests/codecs_yaml.rs`.
- MAY modify: catalog mod/install path, `stdlib_attributes.rs` vocabulary,
  docgen if needed, `docs/reference/` via generator,
  `docs/guide/structured-data.md` (YAML section: the subset table, the
  typing rules with the Norway example, the permanent-non-goal statement,
  transcripts).
- MUST NOT touch: other codecs, kernel crates beyond vocabulary,
  `Cargo.*`, evaluator/dispatch.

## Tests

- A k8s-flavored config end-to-end (nested mappings, sequence of mappings,
  literal block, quoted and plain scalars, comments) asserting the full
  value.
- Norway pins: `country: no` → `"no"` String; `no: 1` key stays `"no"`;
  `on/off/yes` likewise.
- Number exactness (`0.1` → `1/10`; big integers); `~`/empty → Null.
- Literal block `|` vs `|-` chomping pins; indentation edge cases.
- Every fail-closed construct above with message assertions (anchor,
  alias, tag, `---`, merge key, flow `[a,b]`, folded `>`, tab indent,
  duplicate key).
- Round-trip pins incl. the adversarial strings and empties.
- Cross-codec: `FromYaml` → `ToJson` on the k8s-ish doc; `FromJson` →
  `ToYaml` inverse.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib tests::builtin_reference_docgen::builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect 595
printf '' | cargo run -q -p rum-cli -- --eval 'FromYaml["svc:\n  name: web\n  replicas: 2\n  hosts:\n    - a\n    - b\n  note: |-\n    line1\n    line2\n"]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromYaml["country: no\npi: 3.14\nempty:\n"]'
printf '' | cargo run -q -p rum-cli -- --eval 'ToYaml[<|"a" -> {1, "no", "3.14"}, "b" -> <||>|>]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromYaml[ToYaml[<|"k" -> {"null", "x y", 1/4}|>]]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromYaml["a: &x 1"]'          # fail-closed (anchor)
printf '' | cargo run -q -p rum-cli -- --eval 'FromYaml["a: [1, 2]"]'        # fail-closed (flow)
printf '' | cargo run -q -p rum-cli -- --eval 'ToJson[FromYaml["p:\n  q: 8080\n"]]'
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- The subset IS the contract: when a real-world YAML feature is missing,
  the answer is a clear rejection message, not a guess. Full YAML stays a
  permanent non-goal.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The contract as landed + decisions beyond this brief.
2. What changed — per file.
3. Verification — all commands verbatim (Norway pins included).
4. Stop-loss / open questions (empty if none).
