# Brief 164 — Strings usability fixes 2: WL-documented operator forms

Implementation brief. Unit-1 fix batch 2 of the Usability Pass (tracker
`docs/dev/usability-pass.md`, triage item 2; evidence E4 in
`docs/dev/usability-pass/strings-findings.md`).

## Objective

Give these EIGHT string builtins their WL-documented unary operator
forms:

`StringContainsQ`, `StringStartsQ`, `StringEndsQ`, `StringMatchQ`,
`StringFreeQ`, `StringReplace`, `StringCases`, `StringDelete`.

Semantics (the standard WL contract):

- `Foo[patt]` (one argument) evaluates to ITSELF, inert, with NO
  message — it is now a valid operator form. (Today it emits an arity
  message; that message disappearing for the 1-arg call is an intended
  behavior change.)
- Applying it rewrites to the ordinary call:
  `Foo[patt][s]` ≡ `Foo[s, patt]`.
- Options travel: `Foo[patt, opts___][s]` ≡ `Foo[s, patt, opts]`
  (e.g. `StringCases["a", IgnoreCase -> True]["A"]` → `{"A"}`).
- Validation of `patt` happens at APPLICATION, exactly as the ordinary
  call would (an invalid pattern gives the ordinary call's behavior).
- List handling comes free via the rewrite (the ordinary calls already
  accept string lists as the first argument).

## Mechanism

rum already has curried application working for `Select[f][…]`,
`Map[f][…]`, `SortBy[f][…]` (kernel side; see
`crates/rum-runtime/src/runtime/kernel_data/functional_data.rs` and
the other `kernel_data` operator-form sites). EXTEND that existing
mechanism to these stdlib string builtins — do NOT invent a second
dispatch path. If the existing mechanism turns out to be hard-wired to
kernel-registered builtins only and cannot reach stdlib-registered
ones without a new seam, STOP and report the design constraint instead
of building the seam ad hoc.

`StringSplit` and `StringTrim` are EXCLUDED on purpose (their unary
calls are already meaningful data forms). `StringTake`/`Drop`/`Count`/
`Position` are excluded (no WL operator form; deferred).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# the eight operator forms, incl. an options case and a list case:
printf '' | cargo run -q -p rum-cli -- --eval '{StringContainsQ["a"]["cat"], StringStartsQ["a"]["abc"], StringEndsQ["c"]["abc"], StringMatchQ[LetterCharacter..]["abc"], StringFreeQ["z"]["abc"], StringReplace["a" -> "b"]["cat"], StringCases["a", IgnoreCase -> True]["A cat"], StringDelete["a"]["banana"], StringContainsQ["a"][{"cat", "dog"}]}'
# 1-arg calls are inert operator forms with NO message:
printf '' | cargo run -q -p rum-cli -- --eval '{StringReplace["a" -> "b"], StringCases["a"]}'
# ordinary 2-arg calls byte-identical:
printf '' | cargo run -q -p rum-cli -- --eval '{StringContainsQ["cat", "a"], StringReplace["cat", "a" -> "b"], StringDelete["banana", "a"]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 gives
`{True, True, True, True, True, "cbt", {"A", "a"}, "bnn",
{True, False}}`; probe 2 shows the two inert forms and NO messages;
probe 3 unchanged from today (`{True, "cbt", "bnn"}`).

## Scope

- MAY modify: the operator-form/curried-application site(s) in
  `crates/rum-runtime/src/runtime/` that already serve
  `Select`/`Map`/`SortBy`, `crates/rum-stdlib/src/strings/**` and
  `crates/rum-stdlib/src/strings.rs`,
  `crates/rum-stdlib/src/catalog/strings.rs` (descriptions/arity
  metadata only), `crates/rum-stdlib/src/tests/**`,
  `crates/rum-stdlib/src/tests.rs`, kernel-side tests under
  `crates/rum-runtime/src/runtime/tests/**` if the mechanism's own
  tests live there, `docs/reference/builtins.md` (regenerated only, if
  descriptions changed).
- MUST NOT touch: other builtin families' behavior, the guide,
  `TODO.md`, the tracker, the findings file, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Behavior changes are EXACTLY: (a) the eight 1-arg calls become inert
  message-free operator forms, (b) applying those forms rewrites to
  the ordinary call. Every other result byte-identical — in
  particular, all ordinary N≥2-arg calls of the eight.
- Hot-path rule: the curried dispatch must not add work to ordinary
  symbol-head calls. If your change touches the general evaluation
  path (not just the compound-head branch), run the microbench A/B
  (`scripts/core-language-microbench.sh`) and report it; if it is
  confined to the compound-head branch, say so explicitly instead.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. How the existing mechanism accommodates stdlib builtins — what you
   extended, per file.
2. Verification — all commands verbatim (incl. the microbench A/B or
   the confinement statement).
3. Stop-loss / open questions (empty if none).
