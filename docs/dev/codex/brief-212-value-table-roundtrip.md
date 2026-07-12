# Brief 212 — Round-trippable value-table assignment (unit-8 fix batch 4)

Implementation brief with a MANDATORY microbench A/B gate (it touches the
assignment lvalue-classification path). Read `AGENTS.md` (Testing Policy),
the unit-8 findings (`docs/dev/usability-pass/kernel-a-findings.md`, the
value-table rows / E7), and the compat rule in `docs/dev/usability-pass.md`
before starting.

## Goal

Make the value-table inspectors round-trippable, the WL-conformant way:
`OwnValues[f] = rules`, `DownValues[f] = rules`, `UpValues[f] = rules`,
`SubValues[f] = rules` install `rules` as `f`'s corresponding value table
(atomic replace), instead of the current behavior (blocked with a misleading
`Set::wrsym: Symbol DownValues is Protected` because the assignment routes
through the ordinary path onto the now-protected inspector head).

The load-bearing acceptance test is the IDENTITY round-trip:

```
f[1] := 111; f[x_] := x + 1;
saved = DownValues[f];
ClearAll[f];
DownValues[f] = saved;
{f[1], f[5]}                 (* must be {111, 6} *)
```

Today that leaves `f` undefined. After this brief it restores `f`'s
definitions.

## Current state (verify first — the safety half is already done)

After brief-210 every inspector head is `Protected`, so
`DownValues[g] = X` now fails closed with `Set::wrsym` and installs nothing —
the old SILENT false round-trip is already gone. This brief is the
convenience/feature half: real round-trippable assignment. Do NOT reintroduce
any silent-write behavior; unsupported cases stay fail-closed with a CLEAR
message.

## The mechanism (already diagnosed — reuse, do not reinvent)

Read side (the exact shape you must parse as the inverse):
`definition_rule_expr`
(`crates/rum-runtime/src/runtime/definitions/definition_catalog/entries.rs:98`)
emits each entry as `RuleDelayed[HoldPattern[lhs], rhs]`, and
`own_values_expr_for_symbol` / `down_values_expr_for_symbol` /
`up_values_expr_for_symbol` / `sub_values_expr_for_symbol` (same file) wrap
them in a `List`. So a value table reads back as
`List[RuleDelayed[HoldPattern[lhs], rhs], …]`.

Install primitive (reuse this — it compiles the matcher and all derived
fields): `store_definition_target(target, lhs, rhs, delayed)`
(`crates/rum-runtime/src/runtime/definitions/definition_tags.rs:178`) takes a
`TaggedDefinitionTarget` (`OwnValue` / `DownValue` / `SubValue` / `UpValue`,
each carrying the target symbol) plus `lhs`/`rhs`/`delayed` and does the full
install exactly as `TagSet`/`UpSet` do. Per-kind validation semantics already
exist in `classify_tagged_definition` (same file) — use them (or mirror them)
so the head-matching rules are identical to `f /: … := …`.

Assignment hook: `finish_definition_assignment`
(`crates/rum-runtime/src/runtime/definitions/definitions.rs:198`) already has
the template — the `normalize_options_target_expr(&lhs)` branch at `:267`
handles `Options[f] = …`. Add a sibling branch for value tables.

## The change

1. **Detect the lvalue shape (cheap).** Add a classifier
   `normalize_value_table_target_expr(&lhs) -> Option<(ValueTableKind, Symbol)>`
   that matches `lhs = Call{ head: <one of the four System inspector symbols>,
   args: [sym] }` and returns the kind + target symbol. This must be a cheap
   head-name check (one comparison against the four names) — it runs on the
   assignment path for EVERY `Set`/`SetDelayed`, so keep it to a small match,
   no allocation on the common miss. Place the branch near the `Options` branch
   in `finish_definition_assignment` (immediate `Set` only; a DELAYED
   `DownValues[f] := …` is out of scope — fall through to the ordinary path,
   which already fails closed on the protected head).

2. **Guard the TARGET (not the inspector).** Call
   `ensure_symbol_not_protected_for_write(symbol, &target)` on the target
   symbol `f`. If `f` is protected, fail closed (return `rhs`) — same as any
   other definition write to a protected symbol.

3. **Parse + validate the whole RHS before mutating anything (atomic /
   fail-closed).** The RHS must be a `List` of rules; accept each rule as
   `RuleDelayed[HoldPattern[l], r]` or `RuleDelayed[l, r]` (delayed) and
   `Rule[HoldPattern[l], r]` or `Rule[l, r]` (immediate); strip `HoldPattern`.
   For each rule, validate the extracted `l` is well-formed for the kind and
   references the TARGET symbol the same way `classify_tagged_definition`
   requires (OwnValues: `l` is `f` itself; DownValues: head of `l` is `f`;
   SubValues: `l` is `f[…][…]`; UpValues: `f` occurs in `l`). If the RHS is not
   a list of rules, or ANY rule is malformed or head-mismatched, fail closed:
   push a clear message (e.g. `SetDelayed::write`-style or a dedicated
   `<Inspector>::setval` tag — pick one tag, document it) describing the
   offending rule, and change NOTHING.

4. **Atomic replace.** Only after every rule validates: clear the target's
   existing table for that kind (reuse the clear path in
   `definition_clear.rs`, or clear the relevant `definition.<table>` vector /
   own-value field directly), then install each parsed rule via
   `store_definition_target(TaggedDefinitionTarget::<Kind>(target), l, r,
   delayed)` in order. Return `rhs` (the assigned list), matching WL's
   assignment return.

5. **DefaultValues — DEFERRED, but honest.** `DefaultValues` is not one of the
   four `TaggedDefinitionTarget` kinds and has an awkward `Default[…]` shape.
   Do NOT implement its round-trip here. `DefaultValues[f] = …` should keep
   failing closed, but if it is cheap to give it the same clear "value-table
   assignment for DefaultValues is not supported" message instead of the
   misleading "DefaultValues is Protected", do so; otherwise leave it and note
   it. State which you did.

## Fidelity notes (fold into the code/tests, don't fight them)

- The read side always emits `RuleDelayed` even for an immediate own value, so
  an own-value round-trip comes back delayed; since the rhs is already the
  evaluated value, this is folk-acceptable. Note it in a comment.
- A cross-symbol copy `DownValues[g] = DownValues[f]` has rules whose head is
  `f`, not `g`; per step 3 that FAILS with the clear head-mismatch message
  (the honest outcome — it would never fire for `g`). The working cross-symbol
  form is `DownValues[g] = (DownValues[f] /. f -> g)`, which validates and
  installs. Add a test for BOTH (the mismatch message, and the substituted
  form working).

## Tests (add; each asserting the behavior + fail-closed where relevant)

- Identity round-trip for DownValues (the Goal example) → `{111, 6}`.
- Identity round-trip for OwnValues (`a = 10; s = OwnValues[a]; ClearAll[a];
  OwnValues[a] = s; a` → `10`).
- UpValues and SubValues identity round-trips.
- Cross-symbol head-mismatch fails closed with the message + target unchanged.
- Cross-symbol with `/. f -> g` substitution works.
- Malformed RHS (not a list of rules; a rule with a non-matching head) fails
  closed, message emitted, target table unchanged (atomic — a later bad rule
  does not leave earlier rules installed).
- Protected target `f` → fail closed (`ensure_symbol_not_protected_for_write`).
- Put them in the definitions/attributes test area near the existing
  value-table tests.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace        # full tree green; run to completion
```

Then the microbench A/B (this touches the assignment lvalue path):

```
bash scripts/core-language-microbench.sh --output-dir /tmp/mb_base_212   # on BASE (stash your change)
bash scripts/core-language-microbench.sh --output-dir /tmp/mb_new_212    # with the change
# paste the per-label rum_seconds rows for the assignment/Set labels AND
# their same-tier neighbors; the new value-table branch must not regress the
# ordinary Set / SetDelayed / compound-assignment labels.
```

Best-of-2 for the A/B; profiling env OFF. If a Set-tier label regresses
beyond noise, the cheap-detection requirement (step 1) was not met — fix the
branch placement/cost or report it. Also paste the behavior probes:

```
printf '' | cargo run -q -p rum-cli -- --eval 'f[1] := 111; f[x_] := x + 1; saved = DownValues[f]; ClearAll[f]; DownValues[f] = saved; Print[{f[1], f[5]}]'
# expect {111, 6}
printf '' | cargo run -q -p rum-cli -- --eval 'f[x_] := x; DownValues[g] = DownValues[f]; Print[{g[3], DownValues[g]}]'
# expect the head-mismatch message + g[3] inert + DownValues[g] == {}
printf '' | cargo run -q -p rum-cli -- --eval 'f[x_] := x + 1; DownValues[g] = (DownValues[f] /. f -> g); Print[g[3]]'
# expect 4
```

## Scope

- MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/definitions.rs` (the hook +
    classifier).
  - `crates/rum-runtime/src/runtime/definitions/definition_tags.rs` or a new
    small sibling module for the parse/validate/install-all logic (reusing
    `store_definition_target` / `classify_tagged_definition`).
  - Test files near the existing value-table / definitions tests.
- MUST NOT touch: the read side (`entries.rs`) semantics, the inspector heads'
  `Protected` attribute, the catalog/mirror/reference, `TODO.md`, the tracker,
  the findings file, `Cargo.*`, git.

## Hard rules

- No `git commit`, branch, push, or git config.
- No new `unsafe`; `rum-core`/`rum-runtime` stay zero-`unsafe`.
- Reuse `store_definition_target` and the existing matcher-compiling install;
  do NOT hand-roll a second matcher-construction path (the `DownValue` /
  `UpValue` / `SubValue` structs carry compiled matchers + derived fields — a
  partial hand-rolled install will silently break matching).
- Atomic + fail-closed: validate the WHOLE rule list before any mutation; on
  any malformed/mismatched rule, change nothing and emit one clear message.
  Never install a partial table.
- **Hot-path discipline:** the new detection must add negligible cost to the
  common `x = 1` / `f[x_] := y` assignment. The A/B gate decides; a Set-tier
  regression beyond noise is a defect to fix or report.
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output (tests, A/B rows, probes). If the suite is not
  green or a label regresses, say so.

## Report format (final message)

1. What changed — per file, line ranges, one-line why; and whether
   DefaultValues got the clearer message or was left.
2. Verification — workspace totals, fmt, the A/B per-label rows (Set-tier +
   neighbors, before/after), and the three behavior probes, all verbatim.
3. Confirmation that the ordinary assignment path is unregressed (A/B) and
   that partial installs are impossible (the atomic-validation structure).
4. Stop-loss / open questions (empty if none).
