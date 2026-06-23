# brief-028: decode `\[Name]` named-character escapes (Greek + common math/logic) in strings AND code

## Goal

rum must decode WolframLanguage named-character escapes `\[Name]` to their Unicode
characters, in BOTH string literals and code, for a broad curated set (Greek
lower/upper case + common logic / relation / set / operator / arrow symbols).
Today rum decodes none: `"\[Exists]"` lexes as the literal 8-char string
`[Exists]` (should be the 1-char `∃`), and `\[Alpha]` in code is not recognized.
This is the current correctness-mainline blocker: `auto/Arith.wl`'s Cooper QE
builds the existential constant via `mkConst["\[Exists]", ...]`, so rum creates a
constant named `[Exists]` instead of `∃` (which mma-hol registers and uses as the
raw Unicode char elsewhere) → `constType: unknown constant [Exists]` halts the
load at `stdlib/Finite.wl`. The user also wants this generally (Greek-letter
input). When done: in a string, `"\[Exists]"` is `∃` (length 1); in code,
`\[Alpha]` lexes as the symbol `α` and `\[And]` as the `∧` operator (the same
chars mma-hol already uses directly); the curated table below is covered.

## Context pointers

- Read `AGENTS.md` and the checkpoint-chain PROGRESS entries. Root-caused from
  the v3 rebuild: `constType: unknown constant [Exists]` ←
  `mkConst["\[Exists]", ...]` ← rum does not decode `\[Exists]`. Confirmed:
  `StringLength["\[Exists]"]` is `8` (should be `1`); the raw `"∃"` is length `1`.
- Key file: `crates/rum-syntax/src/lexer.rs`.
  - `lex_string` (line ~258): the escape match (line ~276) handles `"`, `\`, and
    `\:`-style hex via `try_lex_unicode_escape` (line ~319) — add a `[` case.
  - `next_token` / the main `match ch` (line ~53): add handling for `\` followed
    by `[` in CODE so `\[Alpha]` decodes to `α` and flows into the existing
    symbol lexing (`is_symbol_start` is `is_alphabetic()`, line ~461, so Greek
    letters already lex as symbols once produced; Unicode operators like `∧`/`⇒`
    are already handled — mma-hol uses them directly).
- Cleanest integration (your call): a `pub fn named_character(name: &str) ->
  Option<char>` lookup over the table below, invoked from both `lex_string` (push
  the decoded char into the string value) and the code path (decode then let the
  decoded char re-enter normal char dispatch / symbol continuation). Span for a
  decoded char must cover the whole `\[Name]` source range. An unknown
  `\[NotInTable]` should be a clean syntax error (do not silently drop it).

## The table (authoritative — generated from the local WolframKernel; use verbatim)

```rust
// (name, char) — from ToCharacterCode[ToExpression["\"\\[<name>]\""]]
("Alpha", '\u{3b1}'), ("Beta", '\u{3b2}'), ("Gamma", '\u{3b3}'), ("Delta", '\u{3b4}'),
("Epsilon", '\u{3f5}'), ("Zeta", '\u{3b6}'), ("Eta", '\u{3b7}'), ("Theta", '\u{3b8}'),
("Iota", '\u{3b9}'), ("Kappa", '\u{3ba}'), ("Lambda", '\u{3bb}'), ("Mu", '\u{3bc}'),
("Nu", '\u{3bd}'), ("Xi", '\u{3be}'), ("Omicron", '\u{3bf}'), ("Pi", '\u{3c0}'),
("Rho", '\u{3c1}'), ("Sigma", '\u{3c3}'), ("Tau", '\u{3c4}'), ("Upsilon", '\u{3c5}'),
("Phi", '\u{3d5}'), ("Chi", '\u{3c7}'), ("Psi", '\u{3c8}'), ("Omega", '\u{3c9}'),
("CurlyEpsilon", '\u{3b5}'), ("CurlyTheta", '\u{3d1}'), ("CurlyKappa", '\u{3f0}'),
("CurlyPi", '\u{3d6}'), ("CurlyRho", '\u{3f1}'), ("CurlyPhi", '\u{3c6}'), ("FinalSigma", '\u{3c2}'),
("CapitalAlpha", '\u{391}'), ("CapitalBeta", '\u{392}'), ("CapitalGamma", '\u{393}'),
("CapitalDelta", '\u{394}'), ("CapitalEpsilon", '\u{395}'), ("CapitalZeta", '\u{396}'),
("CapitalEta", '\u{397}'), ("CapitalTheta", '\u{398}'), ("CapitalIota", '\u{399}'),
("CapitalKappa", '\u{39a}'), ("CapitalLambda", '\u{39b}'), ("CapitalMu", '\u{39c}'),
("CapitalNu", '\u{39d}'), ("CapitalXi", '\u{39e}'), ("CapitalOmicron", '\u{39f}'),
("CapitalPi", '\u{3a0}'), ("CapitalRho", '\u{3a1}'), ("CapitalSigma", '\u{3a3}'),
("CapitalTau", '\u{3a4}'), ("CapitalUpsilon", '\u{3a5}'), ("CapitalPhi", '\u{3a6}'),
("CapitalChi", '\u{3a7}'), ("CapitalPsi", '\u{3a8}'), ("CapitalOmega", '\u{3a9}'),
("And", '\u{2227}'), ("Or", '\u{2228}'), ("Not", '\u{ac}'), ("Xor", '\u{22bb}'),
("Nand", '\u{22bc}'), ("Nor", '\u{22bd}'), ("ForAll", '\u{2200}'), ("Exists", '\u{2203}'),
("NotExists", '\u{2204}'), ("Implies", '\u{f523}'), ("Equivalent", '\u{29e6}'),
("RightTee", '\u{22a2}'), ("LeftTee", '\u{22a3}'), ("UpTee", '\u{22a5}'), ("DownTee", '\u{22a4}'),
("DoubleRightTee", '\u{22a8}'), ("Wedge", '\u{22c0}'), ("Vee", '\u{22c1}'),
("Equal", '\u{f431}'), ("NotEqual", '\u{2260}'), ("LessEqual", '\u{2264}'),
("GreaterEqual", '\u{2265}'), ("LongEqual", '\u{f7d9}'), ("Tilde", '\u{223c}'),
("TildeTilde", '\u{2248}'), ("TildeEqual", '\u{2243}'), ("TildeFullEqual", '\u{2245}'),
("Congruent", '\u{2261}'), ("Proportional", '\u{221d}'), ("Element", '\u{2208}'),
("NotElement", '\u{2209}'), ("ReverseElement", '\u{220b}'), ("Subset", '\u{2282}'),
("Superset", '\u{2283}'), ("SubsetEqual", '\u{2286}'), ("SupersetEqual", '\u{2287}'),
("Union", '\u{22c3}'), ("Intersection", '\u{22c2}'), ("EmptySet", '\u{2205}'),
("Infinity", '\u{221e}'), ("PartialD", '\u{2202}'), ("Del", '\u{2207}'),
("Integral", '\u{222b}'), ("ContourIntegral", '\u{222e}'), ("Sum", '\u{2211}'),
("Product", '\u{220f}'), ("CirclePlus", '\u{2295}'), ("CircleTimes", '\u{2297}'),
("CircleDot", '\u{2299}'), ("Cross", '\u{f4a0}'), ("Times", '\u{d7}'), ("Divide", '\u{f7}'),
("PlusMinus", '\u{b1}'), ("MinusPlus", '\u{2213}'), ("CenterDot", '\u{b7}'),
("Star", '\u{22c6}'), ("Bullet", '\u{2022}'), ("Diamond", '\u{22c4}'),
("SmallCircle", '\u{2218}'), ("Degree", '\u{b0}'), ("Angle", '\u{2220}'),
("Perpendicular", '\u{27c2}'), ("RightArrow", '\u{2192}'), ("LeftArrow", '\u{2190}'),
("LeftRightArrow", '\u{2194}'), ("UpArrow", '\u{2191}'), ("DownArrow", '\u{2193}'),
("DoubleRightArrow", '\u{21d2}'), ("DoubleLeftArrow", '\u{21d0}'),
("DoubleLeftRightArrow", '\u{21d4}'), ("Rule", '\u{f522}'), ("RuleDelayed", '\u{f51f}'),
("Function", '\u{f4a1}'),
```

Store as a `&[(&str, char)]` with a linear or `match`/`HashMap` lookup
(`named_character`). 128 entries; do not invent others — if a name is not in the
table, it is an unknown escape (syntax error).

## Scope

- Files you MAY modify:
  - `crates/rum-syntax/src/lexer.rs` (the table + the two decode sites).
  - the lexer test file (`crates/rum-syntax/src/lexer.rs` `mod tests`, or a sibling)
    for regression cases.
- MUST NOT touch: any `mma-hol` clone (read-only `Get` for verification);
  `crates/rum-core`; the parser beyond what the decoded chars naturally feed;
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except your report; `.git`,
  git config, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Both contexts.** String `"\[Exists]"` → `∃` (length 1); code `\[Alpha]` →
   symbol `α`. A decoded letter must be able to BEGIN and CONTINUE a symbol so
   `\[Alpha]` is the whole symbol `α` and `x\[Alpha]` / `\[Alpha]1` behave like
   `xα` / `α1`.
2. **Operators flow through existing handling.** `\[And]` → `∧`; rum already
   lexes `∧` (mma-hol uses it directly). Do not special-case operators — decode
   to the char and let the normal dispatch handle it.
3. **Span coverage** — the decoded char's span must span the whole `\[Name]`.
4. **Unknown names are errors**, not silent drops.
5. **Minimal diff**; keep the table tidy. **Stop-loss:** if the code-context
   integration fights the lexer architecture, implement the STRING context fully
   (the mma-hol blocker) + as much code-context as is clean, and REPORT what is
   left — do not hack the lexer.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# strings:
$BIN --eval 'StringLength["\[Exists]"]'                 # 1
$BIN --eval '"\[Exists]" == "∃"'                        # True
$BIN --eval '"\[ForAll]\[And]\[Or]\[Not]\[DoubleRightArrow]" == "∀∧∨¬⇒"'  # True
$BIN --eval 'StringLength["\[Alpha]\[Beta]\[CapitalOmega]"]'  # 3

# code (Greek as a symbol, math as operator):
$BIN --eval 'FullForm[Hold[\[Alpha]]]'                  # Hold[α]  (a Symbol named α)
$BIN --eval '\[Alpha] = 5; \[Alpha] + 1'                # 6
$BIN --eval 'FullForm[Hold[\[Alpha] \[And] \[Beta]]]'   # the same as Hold[α ∧ β] (And[α, β])

# unknown name errors:
$BIN --eval '"\[NotARealName]"'                         # a syntax error (not silently "[NotARealName]")

# the mma-hol-shaped use (string -> the ∃ char, matching the raw char):
$BIN --eval 'SameQ["\[Exists]", FromCharacterCode[8707]]'  # True (8707 = U+2203 = ∃)
```

Paste verbatim. (Full Finite.wl / number-tower re-verification via the
checkpoint chain is Claude's follow-up — you do NOT run it.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`, `python3`.
- Minimal diff. You MUST paste real verbatim output; if not green, say so.

## Report format (final message)

1. What changed — the table + the decode sites (string and code), line ranges.
2. Verification — each command, verbatim output.
3. The verdict — strings and code both decode; Greek symbols + the 6 mma-hol
   logic chars work; unknown names error; tests green. Note any code-context
   limitation you hit (per stop-loss).
4. Stop-loss / open questions.
