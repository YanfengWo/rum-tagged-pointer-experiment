# Brief 193 — Attribute hygiene: Protected + NumericFunction on GCD/LCM/Max/Min/Abs (unit-5 fix 5)

Implementation brief. Executes the unit-5 triage "attribute hygiene"
batch (tracker `docs/dev/usability-pass.md`, unit-5 section; evidence
E1/E2 in `docs/dev/usability-pass/math-findings.md`), in its
CONSERVATIVE, zero-eval-cost form. Read the tracker's unit-5 triage
first.

`GCD`/`LCM` are `{Listable}`, `Max`/`Min` are `{}`, and `Abs` is
`{Listable}` — all missing the `Protected` (constant-like operators
should not be silently redefinable) and, for the numeric-valued ones,
`NumericFunction` attributes that WL and the sibling elementwise
functions (`Sign`/`Floor`/`Ceiling`/… already carry `{Listable,
NumericFunction, Protected}`). Add those two attribute classes only.

## Deliberate scope limit — NO Orderless / Flat / OneIdentity

Do NOT add `Orderless`, `Flat`, or `OneIdentity` to these builtins,
even though WL has them. rum honors `Orderless`/`Flat` by NORMALIZING
arguments in `eval_call_prepare` (sort / flatten) on every call — adding
them to these folk numeric reducers imposes per-call cost on the common
numeric case for only a marginal symbolic-canonicalization benefit, and
would change symbolic output ordering (there is a committed test pinning
`FullForm[Max[x, 3]] == "Max[x, 3]"`). This is a Claude-level decision
(no-default-path-cost line); this brief is `Protected` + `NumericFunction`
ONLY, which touch neither argument normalization nor ordering.

## Change — `crates/rum-stdlib/src/catalog/arithmetic.rs`

Extend the `AttributeSet` on these five `BuiltinSpec` entries (mirror the
existing `.with(Attribute::...)` builder already used for `Sign`/`Floor`;
`Max`/`Min` are likely `ordinary` today — convert them to
`ordinary_with_attributes`):

- `Abs`: `{Listable}` → `{Listable, NumericFunction, Protected}` (match
  the `Sign`/`Floor` sibling set).
- `GCD`: `{Listable}` → `{Listable, Protected}` (WL `GCD` has NO
  `NumericFunction`).
- `LCM`: `{Listable}` → `{Listable, Protected}`.
- `Max`: `{}` → `{NumericFunction, Protected}` (`catalog/arithmetic.rs`
  Max entry).
- `Min`: `{}` → `{NumericFunction, Protected}`.

No `eval_*` function changes — the reductions are unchanged; only the
declared attributes change.

## Regenerate the reference

The committed `docs/reference/builtins.md` lists Attributes per builtin
and a drift test
(`crates/rum-stdlib/src/tests/builtin_reference_docgen.rs`) enforces it.
After the catalog change, regenerate it:

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

so the drift test stays green (the only reference diff should be the
Attributes lines for these five builtins).

## Tests

The behavior tests are the existing suite plus the regenerated reference.
Add a small attribute-assertion test (co-locate with the math attribute
tests, e.g. `crates/rum-stdlib/src/tests/incidental_arithmetic.rs`):
`Attributes[GCD]` → `{Listable, Protected}`, `Attributes[Max]` →
`{NumericFunction, Protected}`, `Attributes[Abs]` → `{Listable,
NumericFunction, Protected}`. Confirm no reordering regression: keep the
existing `FullForm[Max[x, 3]] == "Max[x, 3]"` test passing (unchanged —
we did NOT add Orderless).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Attributes[GCD], Attributes[LCM], Attributes[Max], Attributes[Min], Attributes[Abs], FullForm[Max[x, 3]], Max[3, 5, 1], GCD[12, 18]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — reference
regenerated, drift test passes, new attribute test added);
`Attributes[GCD]` → `{Listable, Protected}`, `Attributes[Max]` →
`{NumericFunction, Protected}`, `Attributes[Abs]` → `{Listable,
NumericFunction, Protected}`; `FullForm[Max[x, 3]]` STILL `Max[x, 3]`
(no reordering); `Max[3, 5, 1]` → `5`, `GCD[12, 18]` → `6` unchanged.

## Scope

- MAY modify: `crates/rum-stdlib/src/catalog/arithmetic.rs` (the five
  attribute sets), `docs/reference/builtins.md` (regenerated — Attributes
  lines only), and the attribute-assertion test file under
  `crates/rum-stdlib/src/tests/**`.
- MUST NOT touch: the `eval_*` reduction functions; any builtin other
  than GCD/LCM/Max/Min/Abs; do NOT add Orderless/Flat/OneIdentity to
  anything; `TODO.md`, the tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Add ONLY `Protected` and `NumericFunction` (the latter only on
  Max/Min/Abs). NO `Orderless`/`Flat`/`OneIdentity` — they change
  argument ordering / add normalization cost and are out of scope.
- No A/B gate needed: `Protected`/`NumericFunction` are not consulted in
  `eval_call_prepare` argument normalization, so there is no per-call
  eval cost and no ordering change. (If `cargo test` reveals any
  reordering or redefinition regression, STOP and report — that would
  mean an unexpected interaction.)
- Regenerate `builtins.md` with the generator bin — do NOT hand-edit it.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the five attribute sets + the reference regen, line
   ranges.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
