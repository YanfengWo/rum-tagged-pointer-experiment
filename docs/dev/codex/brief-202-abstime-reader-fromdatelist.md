# Brief 202 — AbsoluteTime = reader-only; add FromDateList; remove Now

Design-refinement brief (user rulings 2026-07-08), the finish of the
Now/AbsoluteTime rework. Corrects the brief-200 naming: `Now` is RESERVED
for downstream rum-os (the DateObject / time-zone-aware layer) and must NOT
exist in rum today. The single numeric wall-clock reader is `AbsoluteTime`
(matching WL's numeric AbsoluteTime sense), and the datelist→seconds
conversion becomes its own pure `FromDateList`.

End state (three cleanly-separated symbols):
- `AbsoluteTime[]` — the ONE numeric wall-clock reader, `Capability::Clock`,
  KERNEL (stays in the debug/timing catalog with Timing/AbsoluteTiming).
  Reader-ONLY: no argument forms.
- `FromDateList[datelist]` — NEW pure converter, `Capability::Pure`, STDLIB
  datetime, mirrors `FromDateString` (datelist → exact Unix timestamp).
- `Now` — REMOVED from rum (was added in brief-200; reserved for rum-os).

`DateList`/`DateString` stay the pure conversions brief-200 made them.
Current time = `DateList[AbsoluteTime[]]` / `DateString[AbsoluteTime[]]`.

## Change 1 — kernel AbsoluteTime → reader-only (`debug_tools/timing.rs` + `builtin_catalog/debug.rs`)

`eval_absolute_time` (`timing.rs`) is currently dual-form (`[]` now +
`[expr]` datelist/DateObject/integer conversion). Make it reader-ONLY:

```rust
pub(in crate::runtime) fn eval_absolute_time(
    &mut self,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !args.is_empty() {
        return Ok(self.invalid_call_with_exact_arg_count("AbsoluteTime", args, 0));
    }
    Ok(unix_time_expr(SystemTime::now()))
}
```

Then DELETE the now-dead datelist helpers in `timing.rs`:
`absolute_time_from_date_list_expr`, `absolute_time_integer_field_or_default`,
`absolute_time_second_field_or_default`, `absolute_time_days_in_month`,
`absolute_time_days_from_civil`, and any other fn that becomes unused (let
the compiler's dead-code warnings guide you; remove ALL of them — this
de-duplicates logic that now lives only in stdlib `calendar`). Keep
`unix_time_expr` (used by the `[]` reader) and the `Timing`/`AbsoluteTiming`
handlers untouched.

`debug.rs`: keep `AbsoluteTime` `.with_capability(Capability::Clock)`;
update its summary to drop the conversion clause, e.g. "Returns the current
exact rational Unix timestamp (seconds since 1970-01-01T00:00:00Z);
requires Capability::Clock." Leave `Timing`/`AbsoluteTiming`/`RepeatedTiming`
alone.

## Change 2 — remove `Now`, add `FromDateList` (`catalog/datetime.rs` + `datetime.rs`)

`catalog/datetime.rs`:
- DELETE the `Now` spec (brief-200's addition).
- ADD `FromDateList` (Protected, like the rest of the datetime family;
  Capability::Pure — no `.with_capability`):

```rust
BuiltinSpec::ordinary_with_attributes(
    "FromDateList",
    "Converts a UTC date list {y, mo, d, h, mi, s} to an exact Unix timestamp; the inverse of DateList.",
    AttributeSet::empty().with(Attribute::Protected),
    datetime::eval_from_date_list,
),
```

`datetime.rs`:
- DELETE `eval_now`. Then the clock helpers `current_unix_time` /
  `unix_time_rational` and the `std::time::{SystemTime, UNIX_EPOCH}` import
  become dead (nothing in stdlib reads the clock any more — `AbsoluteTime`
  is kernel) — DELETE them too. If `Capability` is now unused in
  `datetime.rs`, drop it from the import.
- ADD `eval_from_date_list`, reusing the existing stdlib validated converter
  `calendar::timestamp_from_date_list_expr` (the same one `DateList`/
  `DateString` use — it validates month/day and rejects e.g. `{1900,2,29}`):

```rust
pub(crate) fn eval_from_date_list(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [date_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("FromDateList", args, 1));
    };
    match calendar::timestamp_from_date_list_expr(date_list) {
        Ok(timestamp) => Ok(Expr::rational(timestamp)),
        Err(error) => Ok(fail_datetime(session, "FromDateList", error)),
    }
}
```

(If `timestamp_from_date_list_expr` is `pub(super)`, `datetime.rs` — the
parent of `mod calendar` — can call it directly.)

## Change 3 — runtime mirror (`stdlib_attributes.rs`)

Remove `Spec::new("Now", ATTR_PROTECTED)`; add
`Spec::new("FromDateList", ATTR_PROTECTED)` (alphabetical). `AbsoluteTime`
is kernel (not in the stdlib mirror) — do not touch it here.

## Change 4 — docgen + reference regen (`docgen.rs`)

- REMOVE the `Now` LIMITS note.
- UPDATE the `AbsoluteTime` LIMITS note: it currently mentions "date-list
  conversion uses UTC…"; drop the conversion clause (AbsoluteTime no longer
  converts) — keep the timestamp-contract sentence, e.g. "the timestamp
  contract is exact rational seconds since the Unix epoch
  1970-01-01T00:00:00Z." (name-keyed EDIT, do NOT overwrite unrelated
  content).
- ADD a `FromDateList` note if useful (e.g. "UTC, proleptic Gregorian, no
  leap seconds; validates the date list and is the inverse of DateList.").
- Regenerate: `cargo run -q -p rum-stdlib --bin generate-builtin-reference`.
  `builtins.md` diff = Now entry gone, FromDateList entry added, AbsoluteTime
  summary/Limits edited. Net builtin count unchanged (−Now +FromDateList).

## Change 5 — tests

- `crates/rum-stdlib/src/tests/datetime.rs`: MIGRATE every
  `AbsoluteTime[{…}]` datelist-conversion assertion to `FromDateList[{…}]`
  (lines ~49, 61, 105, 113, 130, 133, 134, 137-138 — including the
  `AbsoluteTime[DateList[t]] == t` round-trip → `FromDateList[DateList[t]] ==
  t`, and the invalid-datelist fail-closed asserts now on the `FromDateList`
  symbol). REMOVE the brief-200 `Now[]` tests (~74-94). ADD: `AbsoluteTime[]`
  returns an exact numeric timestamp (unrestricted policy), `AbsoluteTime[0]`
  → arity error, `DateList[AbsoluteTime[]]` length 6.
- `crates/rum-stdlib/src/tests/catalog_capabilities.rs`: REMOVE the `Now`
  capability assertion; ADD `FromDateList` → `Capability::Pure`;
  `AbsoluteTime` stays `Clock` (unchanged, in the kernel capability test at
  `rum-runtime .../capabilities.rs:235` — do not touch that).
- `crates/rum-host/src/tests.rs`: brief-200 added `Now[]` sandbox/clock/
  composition tests (~377, 401, 405, 409). The reader is now `AbsoluteTime[]`
  — which is ALREADY tested for sandbox-denied/clock-allowed at ~250/351.
  Replace the `Now[]` references with `AbsoluteTime[]` and DEDUPLICATE
  against the existing AbsoluteTime[] tests; keep the `DateList[Now[]]` /
  `DateString[Now[]]` composition checks as `DateList[AbsoluteTime[]]` /
  `DateString[AbsoluteTime[]]`; keep `DateList[0]`/`DateString[0]` allowed in
  sandbox.

## Change 6 — guides

- `docs/guide/resource-limits-and-sandbox.md`: the brief-200 line "`Now[]`
  is the current-time reader and requires `Clock`" → "`AbsoluteTime[]` is
  the current-time reader and requires `Clock`". (The line listing
  `AbsoluteTime`, `Timing`, … as wall-clock builtins already fits — leave
  it.)
- `docs/guide/design-boundaries.md`: the datelist↔timestamp round-trip prose
  ("round-trip through `DateList` and `AbsoluteTime`", "`AbsoluteTime[]`
  returns rum's timestamp value…") — the CONVERSION is now `FromDateList`
  (AbsoluteTime[] is the reader). Update: `AbsoluteTime[]` reads the clock;
  the datelist↔timestamp round-trip is `DateList` ↔ `FromDateList`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Head[AbsoluteTime[]], FromDateList[{1970,1,1,0,0,0}], FromDateList[{1970,1,2,0,0,0}], FromDateList[DateList[86400]], DateList[FromDateList[{2026,7,7,1,2,3}]], DateList[AbsoluteTime[]], Attributes[FromDateList], Attributes[AbsoluteTime]}'
printf '' | cargo run -q -p rum-cli -- --eval '{Now[], AbsoluteTime[5], FromDateList[{1900,2,29}], FromDateList[]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — reference
regenerated, net count unchanged, kernel dead code removed). First probe:
`AbsoluteTime[]` head `Rational`/`Integer` (a timestamp); `FromDateList`
converts (`0`, `86400`, round-trips `DateList[86400]`→`86400`,
`DateList[FromDateList[…]]` returns the date list); `DateList[AbsoluteTime[]]`
length 6; `Attributes[FromDateList]` → `{Protected}`; `Attributes[AbsoluteTime]`
→ `{}` (kernel, not swept). Second probe: `Now[]` is now an UNKNOWN symbol
(stays inert `Now[]`, no clock — Now is gone); `AbsoluteTime[5]` → `argx`
(reader-only); `FromDateList[{1900,2,29}]` → `$Failed` (invalid date);
`FromDateList[]` → `argx`.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/debug_tools/timing.rs`,
  `crates/rum-runtime/src/runtime/builtin_catalog/debug.rs`,
  `crates/rum-stdlib/src/catalog/datetime.rs`,
  `crates/rum-stdlib/src/datetime.rs`,
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`,
  `crates/rum-stdlib/src/docgen.rs`, `docs/reference/builtins.md`
  (regenerated), `crates/rum-stdlib/src/tests/datetime.rs`,
  `crates/rum-stdlib/src/tests/catalog_capabilities.rs`,
  `crates/rum-host/src/tests.rs`,
  `docs/guide/resource-limits-and-sandbox.md`,
  `docs/guide/design-boundaries.md`.
- MUST NOT touch: `Timing`/`AbsoluteTiming`/`RepeatedTiming` or any other
  timing/kernel builtin; the `capabilities.rs` AbsoluteTime=Clock test (stays
  true); the other datetime functions' bodies (DateList/DateString/
  FromDateString/DatePlus/DateDifference/DayOfWeek/LeapYearQ/DaysInMonth);
  random; `TODO.md`, the tracker, findings files, other briefs, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- `AbsoluteTime` is reader-ONLY (0-arg), stays kernel + `Capability::Clock`;
  remove ALL its now-dead datelist helper fns (de-dup — the validated
  converter lives only in stdlib `calendar`).
- `FromDateList` is a pure (`Capability::Pure`) stdlib datetime builtin,
  `Protected`, reusing `calendar::timestamp_from_date_list_expr` (same
  validation as DateList/DateString).
- `Now` is fully REMOVED (no spec, handler, mirror entry, note, or test).
- docgen LIMITS is name-keyed: EDIT the AbsoluteTime note (drop the
  conversion clause); do NOT overwrite unrelated content. Regenerate
  `builtins.md` with the generator — do NOT hand-edit it.
- The MAY list spans EVERY file referencing `Now`/`AbsoluteTime` (grepped) —
  update them all so the tree stays green; do not leave a dangling reference.
- Minimal diff beyond the necessary migration. Stop-loss: same failure twice
  → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — kernel AbsoluteTime reader-only + dead-code deletion, the
   Now removal, the FromDateList add, the mirror, docgen+regen, the test
   migration, the guides; line ranges.
2. Verification — all commands verbatim (test totals + both probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
