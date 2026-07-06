# Brief 141 — Date/time folk subset (exact timestamps, UTC only)

Implementation brief. Pure calendar arithmetic on exact rational timestamps;
current-time readers stay `Clock`. Folk scope with documented boundaries:
UTC + fixed offsets only (no tz database), proleptic Gregorian, no leap
seconds.

## Value model (the contract)

- THE timestamp is what `AbsoluteTime[]` already returns: **exact rational
  seconds since the Unix epoch (1970-01-01T00:00:00Z)**. Document this epoch
  explicitly in the reference and guide — it is rum's own contract.
- A date list is `{y, mo, d, h, mi, s}` with integer fields except `s`,
  which may be a rational (sub-second exactness preserved end to end).
  Shorter lists fill with defaults (`{y}` → Jan 1 00:00:00, etc.).
- No new opaque date type. `DateObjectQ`/`TimeObjectQ` stay as they are.

## Builtins (new stdlib family `datetime`; all Pure unless noted)

- `DateList[ts]` → UTC date list; `DateList[]` → current (Clock, via the
  in-handler dual-form check — the SeedRandom precedent from brief-140).
- `AbsoluteTime[{y, mo, d, h, mi, s}]` → timestamp (extend the existing
  kernel builtin's argument surface; the no-arg form stays Clock).
  Validation: out-of-range fields fail closed (message + `$Failed`), except
  seconds/minutes/hours/days may NOT overflow-normalize — reject, don't
  guess.
- `DateString[ts | datelist]` → ISO 8601 UTC (`"2026-07-07T12:34:56Z"`;
  sub-second seconds rendered only when non-integer, as the exact decimal
  when terminating, else fail closed with a convert-explicitly message).
  `DateString[]` → current (Clock, dual-form).
- `FromDateString[str]` → timestamp. ISO 8601 subset, fail-closed:
  `YYYY-MM-DD`, `YYYY-MM-DDTHH:MM:SS`, optional fractional seconds,
  optional `Z` or `±HH:MM` fixed offset (folded into the timestamp).
  Anything else rejects with an offset-bearing message (the codec-family
  error style).
- `DatePlus[ts | datelist, {n, "Seconds"|"Minutes"|"Hours"|"Days"|"Weeks"}]`
  → timestamp shifted by exact arithmetic (`n` exact integer/rational).
  Calendar units ("Months"/"Years") are OUT of the folk scope — document.
- `DateDifference[ts1, ts2, unit]` → exact rational count of the unit.
- `DayOfWeek[ts | datelist]` → symbol `Monday`..`Sunday` (ISO weekday).
- `LeapYearQ[y]`, `DaysInMonth[y, mo]` — calendar helpers.

Civil-calendar conversion uses the standard days-from-civil /
civil-from-days integer algorithms (public domain, ~30 lines safe Rust,
exact for the proleptic Gregorian calendar, negative years included).

## Scope

- MAY create: `crates/rum-stdlib/src/datetime.rs` (+ submodules),
  `crates/rum-stdlib/src/catalog/datetime.rs`, tests
  `crates/rum-stdlib/src/tests/datetime.rs`.
- MAY modify: the kernel `AbsoluteTime` builtin (date-list argument path
  only; its Clock classification and no-arg behavior unchanged),
  `crates/rum-stdlib/src/catalog/mod.rs` + install path,
  `stdlib_attributes.rs` vocabulary (brief-136 lesson), docgen if needed,
  `docs/reference/` via generator, `docs/guide/design-boundaries.md`
  (UTC-only / no-tzdb / no-leap-seconds / epoch statement) and
  `docs/guide/resource-limits-and-sandbox.md` (Clock dual-form list).
- MUST NOT touch: evaluator/dispatch, the eval-loop clock machinery, other
  docs. No new dependencies.

## Tests

- Epoch pins: `AbsoluteTime[{1970,1,1,0,0,0}] == 0`;
  `DateList[0] == {1970,1,1,1,0,0}`-shaped check (correct field order and
  values); a known modern date pin and a pre-1970 (negative ts) pin.
- Round-trips: ts → DateList → AbsoluteTime identity on a spread of values
  incl. rational seconds (`1/3` s preserved); leap-year edges (1900 not
  leap, 2000 leap, Feb 29 valid/invalid), month lengths.
- `FromDateString`: each accepted shape + offset folding
  (`"1970-01-01T01:00:00+01:00"` → 0); fail-closed negatives (bad month,
  trailing garbage, missing pieces, `24:00`).
- `DatePlus`/`DateDifference` exactness; `DayOfWeek` known dates
  (1970-01-01 = Thursday; 2000-02-29 = Tuesday).
- Dual-form Clock: `{Pure}` policy denies `DateList[]`/`DateString[]` but
  allows the argumentful forms; `{Pure, Clock}` allows both (rum-host
  policy test, mirroring the brief-140 shape).
- Vocabulary/catalog registrations + reference drift.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
cargo run -q -p rum-cli -- --eval '{AbsoluteTime[{1970,1,1,0,0,0}], DateList[86400], DayOfWeek[{1970,1,1}]}'
cargo run -q -p rum-cli -- --eval 'DateString[FromDateString["2026-07-07T12:34:56+08:00"]]'
cargo run -q -p rum-cli -- --eval '{DatePlus[0, {36, "Hours"}], DateDifference[0, 86400*7, "Weeks"], LeapYearQ[2000], DaysInMonth[2026, 2]}'
cargo run -q -p rum-cli -- --eval 'FromDateString["2026-13-01"]'      # fail-closed
cargo run -q -p rum-cli -- --eval 'DateList[AbsoluteTime[{2026,7,7,1,2,1/3}]]'   # rational seconds round-trip
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Zero new dependencies; exact arithmetic throughout (no floats anywhere).
- No new `unsafe`. If dispatch/eval code needs touching beyond the listed
  AbsoluteTime path, STOP and report.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The contract as landed (epoch/date-list/boundaries + any decisions
   beyond this brief).
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
