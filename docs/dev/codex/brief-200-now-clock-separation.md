# Brief 200 — Separate stateful clock read into `Now`; make DateList/DateString purely pure

Design-refinement brief (user ruling 2026-07-08), a follow-up to unit-6 /
the FS-purity Clock design. NOT part of a sweep unit. Read
`docs/dev/usability-pass.md` unit-6 close-out (datetime layering) for
context.

## The problem and the ruling

`DateList[]` / `DateString[]` today are DUAL-form: `DateList[ts]` is pure,
but the zero-argument `DateList[]` reads the wall clock (gated by a RUNTIME
`clock_capability_allowed` check, with the builtin declared `Pure`). That
mixes a stateful read and a pure conversion in ONE symbol — a foot-gun: a
caller who treats `DateList` as pure gets bitten by `DateList[]` being
stateful, and the `Pure` label is dishonest for the now-path.

User ruling: **strictly separate the stateful read from the pure
conversions.**
- Add ONE stateful clock reader, `Now`, declared `Capability::Clock` (the
  honest label — the dispatcher denies it in `sandboxed()`).
- REMOVE the zero-argument now-forms from `DateList` / `DateString`; they
  become purely pure conversion functions (require an argument).
- Current date/time = composition: `DateList[Now[]]`, `DateString[Now[]]`,
  `DateString[Now[], "ISODate"]`.
- `Now[]` returns the raw exact Unix timestamp; formatting is the pure
  functions' job (NO format option on `Now` — user decision A, maximal
  separation).

This DELIBERATELY diverges from WL (whose `DateList[]` is stateful) — the
safety separation overrides WL-conformance here (sandbox-safety
differentiation direction).

## How capability gating works (already in place — use it)

`eval_dispatch/mod.rs:125` denies a builtin at call time when
`!capability_policy.allows(spec.capability)`, returning
`sandbox_denied_builtin` = a `General::sandbox` message + `$Failed`
(exactly what `assert_sandbox_denied` checks). `RandomBytes` /
`RandomPassword` are the precedent: declared
`.with_capability(Capability::Random)`, with NO manual check in their eval.
So `Now` needs ONLY the spec-level `.with_capability(Capability::Clock)` —
NO `clock_capability_allowed` call in its handler.

## Change 1 — add `Now` (`catalog/datetime.rs` + `datetime.rs`)

`catalog/datetime.rs`: add a spec (Protected, per unit-6 hygiene, AND
Capability::Clock):

```rust
BuiltinSpec::ordinary_with_attributes(
    "Now",
    "Reads the current time as an exact Unix timestamp (seconds since the epoch). Requires Capability::Clock; compose with DateList/DateString for the current date.",
    AttributeSet::empty().with(Attribute::Protected),
    datetime::eval_now,
)
.with_capability(Capability::Clock),
```

`datetime.rs`: add `eval_now`:

```rust
pub(crate) fn eval_now(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !args.is_empty() {
        return Ok(session.invalid_call_with_exact_arg_count("Now", args, 0));
    }
    Ok(Expr::rational(current_unix_time()))
}
```

No capability check in the handler — the dispatcher enforces
`Capability::Clock` via the spec. `current_unix_time()` /
`unix_time_rational()` / the `SystemTime` import stay (now used by `Now`).

## Change 2 — remove the now-forms from DateList/DateString (`datetime.rs`)

`eval_date_list`: drop the `[]` arm; require exactly 1 arg:

```rust
    let [timestamp] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DateList", args, 1));
    };
    match calendar::timestamp_from_timestamp_or_date_list_expr(timestamp) { ... }
```

`eval_date_string`: drop the `[]` arm; keep the 1-arg and 2-arg (fmt) forms
from brief-198; `[]` → arity error. The arity range becomes 1..2
(`invalid_call_with_arg_range("DateString", args, 1, 2)`).

Then DELETE `clock_capability_allowed` (no longer referenced) and drop the
now-unused `Capability` import from `datetime.rs` (it moves to
`catalog/datetime.rs`). `DateList`/`DateString` stay `Capability::Pure` —
now genuinely pure (no clock path).

Update the `DateList`/`DateString` catalog SUMMARIES to drop the "or reads
the current …" clause (they are pure conversions now).

## Change 3 — runtime mirror (`stdlib_attributes.rs`)

Add `Spec::new("Now", ATTR_PROTECTED)` (alphabetical). `Now` is Protected;
the drift test enforces catalog↔mirror agreement. (Capability is not in the
attribute mirror — no change there.)

## Change 4 — docgen notes + reference regen (`docgen.rs`)

The `DateList` and `DateString` LIMITS notes each mention "the
zero-argument current-time form requires Capability::Clock" — now WRONG.
EDIT them to REMOVE that clause (preserve the rest — do NOT overwrite the
whole note; this is the docgen name-keyed merge lesson):
- `DateList`: "uses UTC, the proleptic Gregorian calendar, and no leap
  seconds." (drop the "; the zero-argument …" clause).
- `DateString`: keep the ISO-only / terminating-second sentence and the
  two-argument-form sentence; drop the "and the zero-argument current-time
  form requires Capability::Clock" clause.

Optionally add a `Now` note (e.g. "reads the host wall clock; requires
Capability::Clock and is denied under sandboxed()."). Then regenerate:

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

`builtins.md` diff = the new `Now` entry + the two edited Limits lines +
the DateList/DateString summary edits.

## Change 5 — tests

- `crates/rum-stdlib/src/tests/catalog_capabilities.rs`: add `Now` →
  `Capability::Clock`; keep DateList/DateString → `Pure`.
- `crates/rum-stdlib/src/tests/datetime.rs`: `Now[]` returns an exact
  numeric timestamp (in the default unrestricted policy); `DateList[Now[]]`
  is a 6-element list; `DateString[Now[]]` contains `"T"`; `DateList[]` and
  `DateString[]` now emit `argx` (arity) — mirror the file's helpers.
- `crates/rum-host/src/tests.rs`: update the two datetime capability tests
  (`sandboxed_host_config_blocks_datetime_current_readers_but_allows_pure_forms`
  and `clock_policy_allows_datetime_current_readers`): the denied/allowed
  current-time reader is now `Now[]` (not `DateList[]`/`DateString[]`);
  `DateList[0]`/`DateString[0]` still succeed in sandbox; under the Clock
  policy, `Now[]` and `DateList[Now[]]` succeed. Rename the test if its name
  now misleads.
- `crates/rum-host/src/tests.rs` may also gain a check that `DateList[]`
  (zero-arg) is now a plain arity error, not a sandbox denial.

## Change 6 — guide (`docs/guide/resource-limits-and-sandbox.md`)

Line ~179 documents `DateList[]`/`DateString[]` as current-time readers
requiring `Clock`. Update: the current-time reader is now `Now[]` (requires
`Clock`); `DateList`/`DateString` are pure conversions.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Head[Now[]], Length[DateList[Now[]]], StringContainsQ[DateString[Now[]], "T"], DateString[Now[], "ISODate"], DateList[0], DateString[0, "ISOTime"], Attributes[Now]}'
printf '' | cargo run -q -p rum-cli -- --eval '{DateList[], DateString[], Now[1]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — reference
regenerated, +1 builtin, capability + host tests updated). First probe:
`Now[]` a numeric timestamp (Integer/Rational head); `DateList[Now[]]`
length 6; `DateString[Now[]]` contains `T`; `DateString[Now[], "ISODate"]`
a date string; `DateList[0]`/`DateString[0, "ISOTime"]` unchanged;
`Attributes[Now]` → `{Protected}`. Second probe: `DateList[]`,
`DateString[]`, `Now[1]` all `argx` errors (arity). The reference gains a
`Now` entry and the DateList/DateString Clock clauses are gone.

## Scope

- MAY modify: `crates/rum-stdlib/src/catalog/datetime.rs`,
  `crates/rum-stdlib/src/datetime.rs`,
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`,
  `crates/rum-stdlib/src/docgen.rs`, `docs/reference/builtins.md`
  (regenerated), `crates/rum-stdlib/src/tests/datetime.rs`,
  `crates/rum-stdlib/src/tests/catalog_capabilities.rs`,
  `crates/rum-host/src/tests.rs`,
  `docs/guide/resource-limits-and-sandbox.md`.
- MUST NOT touch: any other builtin or datetime function body
  (FromDateString/DatePlus/DateDifference/DayOfWeek/LeapYearQ/DaysInMonth
  unchanged); the operator-form machinery; `TODO.md`, the tracker
  (`usability-pass.md`), findings files, other briefs, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- `Now` is gated by the SPEC-level `Capability::Clock` ONLY (dispatcher
  enforces it) — do NOT add a manual capability check in `eval_now`, and do
  NOT keep `clock_capability_allowed` (delete it once unused).
- `DateList`/`DateString` become genuinely pure: NO clock path, NO
  `Capability` reference in their handlers; `DateList[]`/`DateString[]`
  become arity errors, NOT sandbox denials.
- No format option on `Now` (user decision A). `Now[]` returns the raw
  exact Unix timestamp; formatting is DateList/DateString's job.
- docgen LIMITS notes are name-keyed: EDIT the DateList/DateString notes to
  drop the Clock clause; do NOT overwrite their other content.
- Regenerate `builtins.md` with the generator bin — do NOT hand-edit it.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the `Now` spec+handler, the DateList/DateString now-form
   removal + clock_capability_allowed deletion, the mirror, docgen+regen,
   the tests, the guide; line ranges.
2. Verification — all commands verbatim (test totals + both probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
