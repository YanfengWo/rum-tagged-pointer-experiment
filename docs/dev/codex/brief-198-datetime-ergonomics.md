# Brief 198 — Datetime ergonomics: DateString format + DayOfWeek selector (unit-6 fix 4)

Implementation brief. Executes the unit-6 triage batch 4 (tracker
`docs/dev/usability-pass.md`, "Unit 6 (codecs+bytes+datetime) triage";
evidence E8 in `docs/dev/usability-pass/codecs-bytes-datetime-findings.md`).
Two cheap, folk-compatible flexibility additions. Read the tracker's
unit-6 triage first.

## What and why

Two datetime heads only expose their single most-verbose form:
- `DateString[date]` always emits the full `YYYY-MM-DDTHH:MM:SSZ`; there is
  no cheap date-only or time-only selector (`DateString[0, "ISODate"]` is
  rejected today, E8).
- `DayOfWeek[date]` only returns the weekday SYMBOL (`Thursday`); scripts
  cannot ask for the ISO integer weekday (`DayOfWeek[0, "Integer"]` is
  rejected today, E8).

Add a positional second argument to each (WL-idiomatic — `DateString[date,
spec]` is positional). Keep UTC-only, exact seconds, no locale / time-zone
database, and the existing forms byte-identical. Fail CLOSED on an
unknown format/selector name (message + `$Failed`).

## Change 1 — `DateString[date, fmt]` (`crates/rum-stdlib/src/datetime.rs`, `datetime/format.rs`)

`eval_date_string` (`datetime.rs:30`): extend the arity to accept an
optional format string as a THIRD match position (the existing `[]` now-form
and `[value]` default-format form stay EXACTLY as-is — the `[]` form keeps
its `Capability::Clock` gate). Add a `[value, fmt]` arm: resolve the
timestamp as today, then dispatch on `fmt`:

- `"ISODateTime"` → the current full output (default; identical to the
  no-format form).
- `"ISODate"` → `YYYY-MM-DD` (date only).
- `"ISOTime"` → `HH:MM:SSZ` (time only, UTC).
- `fmt` not a string, or any other string → fail closed via
  `fail_datetime(session, "DateString", DateTimeError::date("unknown
  DateString format; use \"ISODateTime\", \"ISODate\", or \"ISOTime\""))`.
- Update the arity fallback to `invalid_call_with_arg_range("DateString",
  args, 0, 2)`.

In `datetime/format.rs`, add two builders beside
`date_string_from_timestamp` (reuse `parts_from_timestamp`, `format_year`,
`format_second`):
- `iso_date_from_timestamp` → `format!("{}-{:02}-{:02}", format_year(&parts.year)?, parts.month, parts.day)`.
- `iso_time_from_timestamp` → `format!("{:02}:{:02}:{}Z", parts.hour, parts.minute, format_second(&parts.second)?)`.

(`date_string_from_timestamp` stays the `"ISODateTime"` form.) The
non-terminating-fractional-second and year-range fail-closed paths inside
`format_second`/`format_year` are inherited unchanged.

## Change 2 — `DayOfWeek[date, sel]` (`datetime.rs`, `datetime/calendar.rs`)

`calendar::day_of_week` (`calendar.rs:121`) computes an index 0=Monday..
6=Sunday then indexes a names array. Refactor minimally so the index is
reusable: extract the names to a `pub(super) const DAY_NAMES: [&str; 7]`
and add `pub(super) fn day_of_week_index(date: &Expr) -> Result<usize,
DateTimeError>` returning the 0..6 index; keep `day_of_week` returning
`DAY_NAMES[day_of_week_index(date)?]` (existing 1-arg behavior unchanged).

`eval_day_of_week` (`datetime.rs:107`): accept an optional selector:
- `[date]` → the weekday symbol (unchanged default; same as `"Name"`).
- `[date, sel]`:
  - `"Name"` → the weekday symbol (as today).
  - `"ISOInteger"` → `Expr::small_integer((index + 1) as i64)` (Monday=1 ..
    Sunday=7).
  - `sel` not a string, or any other string → fail closed via
    `fail_datetime(session, "DayOfWeek", DateTimeError::date("unknown
    DayOfWeek selector; use \"Name\" or \"ISOInteger\""))`.
- Update the arity fallback to `invalid_call_with_arg_range("DayOfWeek",
  args, 1, 2)`.

## Change 3 — document + regenerate the reference (`crates/rum-stdlib/src/docgen.rs`)

Add two LIMITS notes to the docgen table (`(name, note)`, in table order),
so the new forms are discoverable:
- `DateString`: "the two-argument form DateString[date, fmt] accepts fmt
  \"ISODateTime\" (default, full UTC timestamp), \"ISODate\" (date only), or
  \"ISOTime\" (time only); all output is UTC."
- `DayOfWeek`: "the two-argument form DayOfWeek[date, sel] accepts sel
  \"Name\" (default weekday symbol) or \"ISOInteger\" (Monday=1 through
  Sunday=7)."

Regenerate; do NOT hand-edit `builtins.md`:

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

The only `builtins.md` diff should be the two new Limits lines.

## Tests — `crates/rum-stdlib/src/tests/datetime.rs`

Add assertions (co-locate with the existing DateString/DayOfWeek tests):
- `DateString[0, "ISODateTime"] == DateString[0]` (`"1970-01-01T00:00:00Z"`),
  `DateString[0, "ISODate"] == "1970-01-01"`, `DateString[0, "ISOTime"] ==
  "00:00:00Z"`, `DateString[{2026,7,7,12,34,56}, "ISOTime"] == "12:34:56Z"`.
- `DayOfWeek[0] == Thursday`, `DayOfWeek[0, "Name"] == Thursday`,
  `DayOfWeek[0, "ISOInteger"] == 4` (1970-01-01 is a Thursday).
- Fail-closed: `DateString[0, "Nope"]` → `$Failed` + a `DateString`
  message; `DayOfWeek[0, "Nope"]` → `$Failed` + a `DayOfWeek` message
  (mirror the file's `assert_failed_with_message` helper).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{DateString[0], DateString[0, "ISODateTime"], DateString[0, "ISODate"], DateString[0, "ISOTime"], DateString[{2026,7,7,12,34,56}, "ISODate"], DayOfWeek[0], DayOfWeek[0, "Name"], DayOfWeek[0, "ISOInteger"], DateString[0, "Nope"], DayOfWeek[0, "Nope"]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — datetime tests +,
reference regenerated). Probe: `DateString[0]` and `DateString[0,
"ISODateTime"]` both `"1970-01-01T00:00:00Z"`; `"ISODate"` →
`"1970-01-01"`; `"ISOTime"` → `"00:00:00Z"`; the date-list ISODate →
`"2026-07-07"`; `DayOfWeek[0]`/`[0,"Name"]` → `Thursday`; `[0,"ISOInteger"]`
→ `4`; the two `"Nope"` calls → `$Failed` (with messages).

## Scope

- MAY modify: `crates/rum-stdlib/src/datetime.rs`,
  `crates/rum-stdlib/src/datetime/format.rs`,
  `crates/rum-stdlib/src/datetime/calendar.rs`,
  `crates/rum-stdlib/src/docgen.rs` (two Limits notes),
  `docs/reference/builtins.md` (regenerated — two Limits lines),
  `crates/rum-stdlib/src/tests/datetime.rs`.
- MUST NOT touch: the `Capability::Clock` gate or the now-forms; any other
  datetime head (DateList/FromDateString/DatePlus/DateDifference/LeapYearQ/
  DaysInMonth); options registration; any other builtin; `TODO.md`, the
  tracker, findings files, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- Additive only: the existing `DateString[]`/`DateString[date]` and
  `DayOfWeek[date]` forms stay byte-identical (verify in the probe). UTC
  only, exact seconds, NO locale / time-zone / calendar-database creep.
- Unknown format/selector → fail closed (message + `$Failed`), never a
  silent inert or wrong value.
- Regenerate `builtins.md` with the generator bin — do NOT hand-edit it.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the two eval forms, the format.rs/calendar.rs helpers,
   the two docgen notes + regen, the tests; line ranges.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
