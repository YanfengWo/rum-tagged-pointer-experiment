# Brief 170 — Unit-2 fixes 1: Heads options registration for the map/apply family

Implementation brief. Unit-2 fix batch 1 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, unit-2 triage item 1; evidence
E2 in `docs/dev/usability-pass/structure-functional-findings.md`).

## Background (mechanism fact — read this before coding)

The options-defaults table
(`crates/rum-runtime/src/runtime/options/defaults.rs`) is a DISCOVERY
surface: builtin implementations never consult it at call time;
registered defaults feed `Options[...]`, `SetOptions` bookkeeping, and
`OptionValue` resolution only. `Cases`/`Depth`/`FreeQ`/`Position`
already register `Heads` there while their implementations parse
explicit `Heads -> …` from the argument tail and hard-code the
default. This brief extends that EXACT pattern — do not invent
call-time wiring.

## Changes

1. In `defaults.rs`, register `Heads -> False` for the eight symbols
   that already ACCEPT the option in their argument tails but return
   empty `Options[...]` (E2): `Map`, `MapIndexed`, `MapAll`, `Scan`,
   `Apply`, `MapApply`, `Comap`, `ComapApply`.
2. NO implementation changes: the hard-coded behavior defaults and the
   explicit-option parsing in
   `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs` stay
   byte-identical.
3. Tests: `Options /@ {...}` assertions for the eight; a behavior pin
   that explicit `Heads -> True` still works
   (e.g. `Map[f, p[q][x], 1, Heads -> True]` → `f[p[q]][f[x]]`, from
   E2) and that the no-option defaults are unchanged.
4. Regenerate `docs/reference/builtins.md` ONLY if registered options
   render there (they did not for brief-165's registrations — check
   and skip if no drift).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Options /@ {Map, MapIndexed, MapAll, Scan, Apply, MapApply, Comap, ComapApply}, Map[f, p[q][x], 1, Heads -> True], Map[f, {a, b}], Apply[f, p[q][x], 1, Heads -> True]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); all eight
`Options` lists show `{Heads -> False}`; the three behavior probes
byte-identical to today (`f[p[q]][f[x]]`, `{f[a], f[b]}`, `f[q][x]`).

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/options/defaults.rs`,
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  kernel-side options tests under
  `crates/rum-runtime/src/runtime/tests/**` if that is where
  options-registration tests live, `docs/reference/builtins.md`
  (regenerated only if drift).
- MUST NOT touch: `normalize.rs` or any implementation file, other
  catalogs, the guide, `TODO.md`, the tracker, findings files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- ZERO behavior change outside `Options[...]`/`OptionValue` surfaces.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
