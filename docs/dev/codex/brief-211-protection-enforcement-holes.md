# Brief 211 — Close the three protection-enforcement holes (unit-8 fix batch 3)

Implementation brief. Three mutation paths write to an explicitly `Protected`
symbol with no `wrsym`-class diagnostic. All three are gate-reproduced (probes
below). Each fix reuses the existing guard `ensure_symbol_not_protected_for_write`
and adds it where it is missing. Read `AGENTS.md` (Testing Policy) and the
unit-8 findings (`docs/dev/usability-pass/kernel-a-findings.md`, the E5 and E14
sections + the enforcement rows) before starting.

Context: after brief-210 every builtin except four session vars is `Protected`.
`Protected` is a mistake-guard and it is REVERSIBLE — `Unprotect[f]` (and
`ClearAttributes[f, Protected]`) must keep working. The fixes below must NOT
break those escape hatches; that is the one place a naive fix over-blocks.

## Hole 1 — `$ContextPath` / `$Packages` / `$Path` setters bypass protection

`finish_definition_assignment`
(`crates/rum-runtime/src/runtime/definitions/definitions.rs:205-243`) has three
dedicated branches that mutate `self.context_path` / `self.loaded_packages` /
`self.package_search_paths` and `return Ok(rhs)` BEFORE the ordinary guard in
`try_set_symbol_own_value_lvalue` is reached. These three vars are
deliberately open by DEFAULT (they are among the four sweep exclusions), but an
explicit `Protect` must be honored.

Reproduced:

```
$ printf '' | cargo run -q -p rum-cli -- --eval 'Protect[$ContextPath]; $ContextPath = {"System`", "Leak`"}; Print[{$ContextPath, Attributes[$ContextPath]}]'
{{System`, Leak`}, {Protected}}       # wrote through a Protected symbol, no message
```

Fix: at the TOP of each of the three branches (before the validated mutation),
call `self.ensure_symbol_not_protected_for_write(symbol, &sym)` where `sym` is
the corresponding system symbol (`Symbol::system("$ContextPath")` etc.) and
`symbol` is the operator name already passed into
`finish_definition_assignment`. If it returns `false`, `return Ok(rhs)` without
mutating (the guard already emitted the `wrsym` message). Keep the existing
arg-form validation after the guard. Default behavior is unchanged because the
three vars are unprotected by default — the guard only fires after an explicit
`Protect`.

## Hole 2 — `SetAttributes` mutates a protected symbol; preserve the ClearAttributes escape

`finish_update_attributes_after_resolved_attributes`
(`crates/rum-runtime/src/runtime/attributes.rs:104-138`) loops over the targets
and mutates each attribute set (`:124-135`) with NO protection check.
`eval_set_attributes` and `eval_clear_attributes` both route here.

Reproduced:

```
$ printf '' | cargo run -q -p rum-cli -- --eval 'ClearAll[s]; s = 1; Protect[s]; SetAttributes[s, Listable]; Print[Attributes[s]]'
{Listable, Protected}                 # SetAttributes changed a Protected symbol, no message
```

Fix: inside the `for target in config.targets` loop, before mutating a target,
compute whether this operation is the Protected-removal escape:

```
let removes_protected =
    update == AttributeUpdate::Remove && config.attributes.contains(Attribute::Protected);
if self.symbol_is_protected(&target) && !removes_protected {
    self.push_protected_write_message(symbol, &target);
    continue; // skip this target, do not mutate
}
```

This blocks `SetAttributes[protected, …]` (Insert → never `removes_protected`)
and `ClearAttributes[protected, non-Protected-attrs]` (Remove but does not
contain Protected), while ALLOWING `ClearAttributes[protected, Protected]` and
`ClearAttributes[protected, {Protected, …}]` (the escape). `Protect`/`Unprotect`
go through the separate `eval_protect_like` path and are unaffected — do NOT
touch that path. (Verify `AttributeSet` exposes a `contains` you can call here;
it is used elsewhere in this file. `config.attributes` is the parsed set being
inserted/removed.)

Behavior to preserve (each is a required test assertion):
- `Protect[s]; ClearAttributes[s, Protected]` → succeeds, `s` becomes
  unprotected (escape hatch).
- `Protect[s]; Unprotect[s]` → still works.
- `SetAttributes[u, HoldFirst]` on an UNPROTECTED user symbol `u` → still works.

## Hole 3 — `SetOptions` mutates a protected symbol (reachable with a known option)

`apply_set_options_updates` (`crates/rum-runtime/src/runtime/options/builtins.rs`,
the function after `finish_set_options_from_resolved`) calls
`self.set_options_for_symbol(...)` with NO protection check. The `optnf`
unknown-option check sits in front of it, which is why an earlier probe with an
unregistered option was deflected — but a KNOWN option lands the write.

Reproduced (`FromCsv` is Protected after brief-210, and `Delimiter` is a
registered option):

```
$ printf '' | cargo run -q -p rum-cli -- --eval 'SetOptions[FromCsv, Delimiter -> ";"]; Print[Options[FromCsv]]'
{Delimiter -> ";", Headers -> False}  # changed a Protected symbol's options, no message
```

Fix: at the TOP of `apply_set_options_updates`, before it reads/mutates
`current`, add:

```
if !self.ensure_symbol_not_protected_for_write("SetOptions", target) {
    return self.invalid_call("SetOptions", original_args);
}
```

(`apply_set_options_updates` returns `Expr`, and the existing `optnf` path
already returns `self.invalid_call("SetOptions", original_args)` — mirror that
return shape. The guard emits the `wrsym` message.)

## Regression tests (add, each asserting message + unchanged state)

Place near the existing protected-symbol tests. Suggested homes:
- Hole 1 → the context/loader tests
  (`crates/rum-runtime/src/runtime/tests/context_and_loader.rs`): `Protect` a
  var, assign it, assert the value is unchanged and a `wrsym` message is
  emitted; and a companion asserting it is writable WITHOUT the `Protect`.
- Hole 2 → `crates/rum-runtime/src/runtime/tests/` attributes area (or a new
  test beside the existing attribute tests): `SetAttributes` on a protected
  symbol blocked + unchanged; `ClearAttributes[s, Protected]` escape works;
  `SetAttributes` on an unprotected user symbol still works.
- Hole 3 → the options tests (`crates/rum-runtime/src/runtime/options/` tests or
  wherever `SetOptions` is tested): `SetOptions` on a protected head with a
  known option blocked + options unchanged; an unprotected user symbol path
  still works. If a user symbol cannot carry options, `Unprotect` a system
  head, `SetOptions`, assert it now lands (proving the escape).

Each test fails before its fix and passes after — run once before to confirm it
catches the hole.

## Expected test ripple (report every edit; do not mask a regression)

After brief-210 every system builtin is protected, so any EXISTING test that
does `SetAttributes`/`SetOptions` on a system symbol without `Unprotect` will
now be blocked and may fail. That is correct new behavior. Where such a test
exists, add `Unprotect[<sym>]` before the now-blocked call — this is necessary
ripple, mechanically identical to brief-210's. But: LIST every such test you
edit in your report with the before/after, and if a failure looks like a real
semantic regression rather than "system symbol now needs Unprotect", STOP and
report it instead of editing. Do NOT change any expected-value assertion to
make a test pass — only add `Unprotect` where the new protection is the cause.

Run `cargo test --workspace` to COMPLETION (it stops at the first failing
crate; iterate until the whole tree is green — brief-210's Unevaluated failure
only surfaced after the rum-runtime failures were fixed).

## Scope

- MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/definitions.rs` (hole 1).
  - `crates/rum-runtime/src/runtime/attributes.rs` (hole 2).
  - `crates/rum-runtime/src/runtime/options/builtins.rs` (hole 3).
  - Test files: the three regression-test homes above, plus any existing test
    that needs the `Unprotect` ripple (reported per edit).
- MUST NOT touch: the four excluded session vars' default openness (they stay
  unprotected by default — you only add a guard that fires on explicit
  `Protect`), the `eval_protect_like` path, the catalog/mirror/reference,
  `TODO.md`, the tracker, the findings file, `Cargo.*`, git.

## Hard rules

- No `git commit`, branch, push, or git config.
- No new `unsafe`; `rum-core`/`rum-runtime` stay zero-`unsafe`.
- Reuse `ensure_symbol_not_protected_for_write` / `symbol_is_protected` /
  `push_protected_write_message` — do not hand-roll a second protection check
  or message.
- Preserve the escape hatches: `Unprotect[f]` and
  `ClearAttributes[f, Protected]` MUST still work. A fix that breaks them is
  wrong.
- These are cold mutation paths (options/attributes/config assignment), not the
  evaluation hot path — no microbench A/B. Do not add per-call cost beyond the
  one guard lookup, which is paid only on these mutation calls.
- Minimal, uniform diff; no drive-by reformatting or renames.
- Paste real verbatim output. If the workspace suite is not green, say so;
  never claim a total you did not see.

## Verification (MANDATORY — run all, paste verbatim)

```
# each regression test: fail before, pass after (paste both for at least one per hole)
cargo test --workspace
cargo fmt --check
# the three behavior probes, now fixed:
printf '' | cargo run -q -p rum-cli -- --eval 'Protect[$ContextPath]; $ContextPath = {"System`", "Leak`"}; Print[{$ContextPath, Attributes[$ContextPath]}]'
printf '' | cargo run -q -p rum-cli -- --eval 'ClearAll[s]; s = 1; Protect[s]; SetAttributes[s, Listable]; Print[Attributes[s]]'
printf '' | cargo run -q -p rum-cli -- --eval 'SetOptions[FromCsv, Delimiter -> ";"]; Print[Options[FromCsv]]'
# and the escape hatch still works:
printf '' | cargo run -q -p rum-cli -- --eval 'ClearAll[s]; s = 1; Protect[s]; ClearAttributes[s, Protected]; SetAttributes[s, Listable]; Print[Attributes[s]]'
```

Expect: probe 1 → `$ContextPath` unchanged + `Set::wrsym`; probe 2 →
`{Protected}` (Listable NOT added) + `SetAttributes::wrsym`; probe 3 → options
unchanged (`Delimiter -> ","`) + `SetOptions::wrsym`; escape probe → `{Listable}`
(ClearAttributes removed Protected, then SetAttributes worked).

## Report format (final message)

1. What changed — per file, line ranges, one-line why (three fixes + the
   regression tests).
2. Every existing test you edited with the `Unprotect` ripple, listed.
3. Verification — before/after for one regression test per hole, the workspace
   totals, fmt, and the four behavior probes, all verbatim.
4. Stop-loss / open questions (empty if none).
