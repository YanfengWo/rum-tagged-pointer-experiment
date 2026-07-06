# Brief 165 — Strings usability fixes 3: Overlaps parity + Nearest option registration

Implementation brief. Unit-1 fix batch 3 of the Usability Pass (tracker
`docs/dev/usability-pass.md`, triage item 3; evidence E15 and E2 in
`docs/dev/usability-pass/strings-findings.md`).

## Changes

1. **`StringCases` accepts `Overlaps -> True`** (overlapping
   extraction). Today the option value `True` is rejected (the call
   stays inert, E15); `StringCount` ALREADY accepts `Overlaps -> True`
   (`StringCount["aaaa", "aa", Overlaps -> True]` → 3) — mirror its
   overlap-enumeration semantics so
   `StringCases["aaaa", "aa", Overlaps -> True]` → `{"aa", "aa", "aa"}`
   (same starting positions `StringPosition` reports: 1, 2, 3). The
   default (`Overlaps -> False`, non-overlapping scan) is unchanged.
2. **`StringPosition` accepts `Overlaps -> False`** (non-overlapping
   scan). Today the default output INCLUDES overlaps and `False` is
   rejected (E15); after the change
   `StringPosition["aaaa", "aa", Overlaps -> False]` →
   `{{1, 2}, {3, 4}}`. The default (`Overlaps -> True`) is unchanged.
   Register/document the option default accordingly in
   `Options[StringPosition]` (it already lists
   `IgnoreCase -> False, Overlaps -> False` per E2 — fix that stale
   default to `Overlaps -> True` so the registered default matches the
   actual behavior; if the registration is load-bearing elsewhere,
   report what you found before changing it).
3. **Register `Nearest`'s accepted option**: `Options[Nearest]` is
   empty today although `DistanceFunction -> HammingDistance` is
   accepted and honored (E2). Register `DistanceFunction -> Automatic`
   so the surface is discoverable. Accepted behavior unchanged.
4. **Tests**: overlap extraction/position cases pinned (including the
   two probes above verbatim); `Options[StringCases]`,
   `Options[StringPosition]`, `Options[Nearest]` assertions; defaults
   byte-identical (existing no-option and `IgnoreCase` calls).
5. Regenerate `docs/reference/builtins.md` only if catalog descriptions
   or registered options appear there (check; skip if no drift).

## Watch out

- The `Overlaps -> All` value (WL has it for StringCases) is OUT of
  scope — it must keep today's rejected-inert behavior; add a test
  pinning that so the boundary is explicit.
- `StringCount`'s existing `Overlaps` handling is the semantic
  reference — do not fork a second overlap-enumeration; reuse the same
  helper if the code allows.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{StringCases["aaaa", "aa", Overlaps -> True], StringCases["aaaa", "aa"], StringPosition["aaaa", "aa", Overlaps -> False], StringPosition["aaaa", "aa"], StringCount["aaaa", "aa", Overlaps -> True], Options[Nearest], Options[StringPosition], StringCases["aaaa", "aa", Overlaps -> All]}'
git status --short
```

Expected: fmt clean; workspace green (report totals);
`{{"aa","aa","aa"}, {"aa","aa"}, {{1,2},{3,4}},
{{1,2},{2,3},{3,4}}, 3, {DistanceFunction -> Automatic},
{IgnoreCase -> False, Overlaps -> True},
StringCases["aaaa","aa",Overlaps -> All]}` (the last stays inert).

## Scope

- MAY modify: `crates/rum-stdlib/src/strings/**`,
  `crates/rum-stdlib/src/strings.rs`,
  `crates/rum-stdlib/src/catalog/strings.rs`, the option-registration
  site for `Nearest`/`StringPosition` (wherever `Options[...]`
  defaults live — stdlib or the rum-runtime options table; report the
  file), `crates/rum-stdlib/src/tests/**`,
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/builtins.md`
  (regenerated only, per Changes #5).
- MUST NOT touch: `StringCount`'s behavior, other families, the guide,
  `TODO.md`, the tracker, the findings file, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Behavior changes are EXACTLY: (a) `StringCases` honors
  `Overlaps -> True`, (b) `StringPosition` honors `Overlaps -> False`,
  (c) `Options[...]` registrations per Changes #2-3. Every other
  result byte-identical.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; where the options tables live; what the
   stale `Options[StringPosition]` default turned out to be.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
