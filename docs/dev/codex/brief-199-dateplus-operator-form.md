# Brief 199 — DatePlus[spec] operator form (unit-6 fix 5)

Implementation brief. Executes the unit-6 triage batch 5 (tracker
`docs/dev/usability-pass.md`, "Unit 6 (codecs+bytes+datetime) triage";
evidence E11 in `docs/dev/usability-pass/codecs-bytes-datetime-findings.md`),
NARROWED to `DatePlus` only. This touches call planning → the microbench
A/B hard gate applies (the 175/182 protocol). Read the tracker's unit-6
triage first.

## Scope decision — DatePlus only (ConformSchema/SchemaErrors deferred)

The triage listed three operator forms. This brief ships ONLY
`DatePlus[spec]`. The `ConformSchema[schema]` / `SchemaErrors[schema]`
operator forms are DEFERRED (a Claude-level decision recorded at unit
close): those two heads carry `HoldRest`, so in the direct form
`ConformSchema[value, schema]` the schema is HELD (position 2), but in the
operator capture `ConformSchema[schema]` the schema sits at position 1 and
is EVALUATED — a potential divergence for non-fixed-point schemas
(e.g. an arithmetic or side-effecting schema value). That hold-semantics
question needs its own design pass and must not be shipped on a guess.
`DatePlus` has no hold and its spec (`{n, "unit"}`) is a self-evaluating
list, so its operator form is unambiguous and clean. Do NOT touch
ConformSchema/SchemaErrors in this brief.

## The pattern (mirror GatherBy exactly)

`GatherBy` is the worked precedent: a stdlib builtin whose 1-arg form is a
silent-inert operator head (`collection_stats.rs:266`) plus a name entry in
the runtime `structure_operator_symbol` registry (`functional_data.rs:710`).
`GatherBy[f]` is inert-no-message; `GatherBy[f][list] -> GatherBy[list, f]`.
Do the same two-part change for `DatePlus`.

`DatePlus[spec][date]` must become `DatePlus[date, spec]` (data first, spec
second — matching the direct `DatePlus[date, spec]`). The 1-arg operator
arity cannot be a valid data form (`DatePlus` needs 2 args), so it
qualifies under the disambiguation rule.

## Change 1 — stdlib silent-inert operator arity (`crates/rum-stdlib/src/datetime.rs`)

`eval_date_plus` (`datetime.rs:77`) currently errors on any non-2 arity.
Make the 1-arg case silent-inert (mirror `eval_gather_by`), and use the
`head` parameter (rename `_head` → `head`):

```rust
pub(crate) fn eval_date_plus(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [date, spec] = args else {
        if args.len() == 1 {
            // DatePlus[spec] operator form: stay inert (no message) so it can
            // be applied as DatePlus[spec][date].
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count("DatePlus", args, 2));
    };
    // ... unchanged
```

This changes `DatePlus[x]` (1-arg) from an `argx` message to silent inert —
the established operator-form convention (same as `GatherBy[f]`,
`Cases[patt]`). All other arities and the 2-arg behavior are byte-identical.

## Change 2 — runtime operator registry (`crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`)

Add one arm to `structure_operator_symbol` (`functional_data.rs:700`,
alongside GatherBy/KeyGroupBy/MergeNested — this match is the general
stdlib single/double-arg operator-form registry):

```rust
        1 if is_system_symbol_name(head, "DatePlus") => Some("DatePlus"),
```

Nothing else: `parse_structure_operator_wrapper` /
`apply_structure_operator_wrapper_to_args` /
`invalid_structure_operator_wrapper_argx_symbol` already handle arity-1
operators generically (data-first ordering, and an `argx` "1 argument is
expected" when the operator is applied to != 1 args). `is_callable_operator_
wrapper` picks it up so `DatePlus[spec][date]` routes to the Function
dispatch stage.

## Tests — `crates/rum-stdlib/src/tests/datetime.rs`

Co-locate with the existing DatePlus tests:
- `DatePlus[{1, "Days"}]` alone → inert `DatePlus[{1, "Days"}]`, NO message.
- `DatePlus[{1, "Days"}][0] == DatePlus[0, {1, "Days"}]` (both `86400`);
  `DatePlus[{36, "Hours"}][0] == 129600`.
- `DatePlus[{1, "Days"}][0, 0]` (operator applied to 2 args) → `argx`
  message (mirror `assert_failed_with_message` on the `DatePlus` symbol) —
  or assert the inert `DatePlus[...][0, 0]` with the argx message, matching
  how the other operator families report a wrong operator arity.
- Confirm the pre-existing forms unchanged: `DatePlus[0, {1, "Days"}] ==
  86400`, `DatePlus[0, {1, "Months"}]` still fails closed (folk boundary).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{DatePlus[{1,"Days"}], DatePlus[{1,"Days"}][0], DatePlus[0,{1,"Days"}], DatePlus[{36,"Hours"}][0], Map[DatePlus[{1,"Days"}], {0, 86400}], DatePlus[{1,"Days"}][0,0], DatePlus[0,{1,"Months"}]}'
git status --short
```

Then the microbench A/B (this change touches the operator-wrapper dispatch
in call planning — the hard gate):

```
# baseline binary = current main (pre-change), changed binary = this tree.
# Build both to distinct paths, then run the suite against each and diff
# timings.tsv by label (best-of-2, same-tier neighbors checked):
scripts/core-language-microbench.sh --rum-cli <baseline-bin>   # -> timings.tsv (save as baseline)
scripts/core-language-microbench.sh --rum-cli <changed-bin>    # -> timings.tsv (save as changed)
```
Report the per-label baseline-vs-changed deltas. Expected: within noise (one
added `is_system_symbol_name` arm on the arity-1 operator-recognition path).
If any label regresses beyond noise, STOP and report — do NOT hand-wave.

Expected functional probe: `DatePlus[{1,"Days"}]` inert; `[...][0]` and
`DatePlus[0,{1,"Days"}]` both `86400`; `[{36,"Hours"}][0]` → `129600`;
`Map[DatePlus[{1,"Days"}], {0, 86400}]` → `{86400, 172800}`;
`[...][0,0]` → argx; `DatePlus[0,{1,"Months"}]` still `$Failed`.

## Scope

- MAY modify: `crates/rum-stdlib/src/datetime.rs` (eval_date_plus 1-arg
  inert), `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
  (one `structure_operator_symbol` arm), `crates/rum-stdlib/src/tests/
  datetime.rs` (the operator-form tests).
- MUST NOT touch: ConformSchema / SchemaErrors (deferred); any other
  builtin or operator family; the `_head`→`head` rename is the only
  signature change; `docs/reference/builtins.md` (operator forms are not in
  the reference — no regen); `TODO.md`, the tracker, findings files,
  `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- DatePlus ONLY. The 1-arg form becomes silent inert (operator convention);
  every other DatePlus behavior is byte-identical.
- Microbench A/B is REQUIRED (call-planning touch): saved-binaries,
  per-label timings.tsv, report the deltas. Profiling env OFF.
- No reference regen (operator forms are not documented in builtins.md).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the eval_date_plus 1-arg arm, the structure_operator_symbol
   arm, the tests; line ranges.
2. Verification — all commands verbatim (test totals + the functional probe
   + the microbench A/B per-label deltas + `git status --short`).
3. Stop-loss / open questions (empty if none).
