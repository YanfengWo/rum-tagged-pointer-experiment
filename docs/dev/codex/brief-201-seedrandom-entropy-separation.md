# Brief 201 — Separate the entropy reseed into `ReseedRandom`; make `SeedRandom` deterministic-only

Design-refinement brief (user ruling 2026-07-08), the symmetric twin of
brief-200 (`Now`). Same principle: strictly separate the stateful/impure
read from the pure/deterministic operation, and let the SPEC-level
capability (not a runtime check) gate it.

## The problem and the ruling

`SeedRandom` is dual-form and declared `Capability::Pure`:
- `SeedRandom[seed]` — deterministic reseed from an exact integer
  (reproducible, session-local; genuinely fine as `Pure`, works in
  `sandboxed()`).
- `SeedRandom[]` — reseeds from HOST ENTROPY (ambient authority), gated by a
  RUNTIME `random_capability_allowed` check inside the handler.

That is the same "stateful form crammed into an otherwise-pure symbol +
runtime capability check" anti-pattern brief-200 just removed from
datetime. User ruling: split it, symmetric with `Now` — the impure entropy
reseed becomes ONE dedicated entry, fully gated by spec-level
`Capability::Random` (dispatcher-enforced, like `RandomBytes`/
`RandomPassword`), and `SeedRandom` becomes deterministic-only.

## How capability gating works (already in place — use it)

`eval_dispatch/mod.rs` denies a builtin at call time when its declared
`spec.capability` is not allowed by the policy, returning a
`General::sandbox` message + `$Failed` (what `assert_sandbox_denied`
checks). `RandomBytes`/`RandomPassword` are the precedent: declared
`.with_capability(Capability::Random)`, NO manual check in their handlers.
So `ReseedRandom` needs ONLY the spec-level capability — NO
`random_capability_allowed` call.

## Change 1 — add `ReseedRandom` (`catalog/random.rs` + `random.rs`)

`catalog/random.rs`: add a spec next to `RandomBytes`/`RandomPassword`.
NOTE: the random family is NOT `Protected` yet (unit 7 will sweep that), so
`ReseedRandom` is born WITHOUT `Protected` to match its siblings — plain
`ordinary` + capability:

```rust
BuiltinSpec::ordinary(
    "ReseedRandom",
    "Reseeds the deterministic session PRNG from host entropy. Requires Capability::Random; denied under sandboxed().",
    random::eval_reseed_random,
)
.with_capability(Capability::Random),
```

`random.rs`: add `eval_reseed_random` — the entropy-reseed body lifted from
`SeedRandom[]`, WITHOUT the manual capability check (the dispatcher enforces
`Capability::Random`):

```rust
pub(crate) fn eval_reseed_random(
    session: &mut Session,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !args.is_empty() {
        return Ok(session.invalid_call_with_exact_arg_count("ReseedRandom", args, 0));
    }
    match session.seed_random_from_entropy() {
        Ok(()) => Ok(system_expr("Null")),
        Err(error) => {
            push_entropy_error(session, "ReseedRandom", error.message());
            Ok(system_expr("$Failed"))
        }
    }
}
```

(Confirm the handler signature — `eval_seed_random` takes
`(session, head, args)`; `eval_reseed_random` does not need `head`. Match
whatever the `BuiltinFn` signature requires — mirror a zero-`head` handler
if one exists, else keep `_head`.)

## Change 2 — `SeedRandom` becomes deterministic-only (`random.rs`)

`eval_seed_random` (`random.rs:11`): drop the `[]` entropy arm; require
exactly 1 integer seed:

```rust
    let [seed] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("SeedRandom", args, 1));
    };
    let Some(seed) = expr_as_exact_integer(seed) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    session.seed_random_with_integer(&seed);
    Ok(system_expr("Null"))
```

`SeedRandom[]` is now an arity error. `SeedRandom` stays `Capability::Pure`
(deterministic, session-local — works in `sandboxed()`). Then DELETE
`random_capability_allowed` (`random.rs:440`, now unused) and drop the
`Capability` import from `random.rs` if it becomes unused (it stays in
`catalog/random.rs` for `.with_capability`). Update `SeedRandom`'s catalog
summary to drop "or from host entropy in the zero-argument form".

## Change 3 — runtime mirror (`stdlib_attributes.rs`)

`ReseedRandom` has empty attributes (not Protected yet) → add it to
`STDLIB_EMPTY_ATTRIBUTE_NAMES` (the bare-name array, alphabetical, near the
other `Re…`/`Random…` names), NOT the `Spec::new` array. The drift test
enforces catalog↔mirror agreement.

## Change 4 — docgen + reference regen (`docgen.rs`)

`SeedRandom`'s LIMITS note ends with "The zero-argument entropy form
requires Capability::Random." — now WRONG. EDIT it to drop that sentence
(keep the xoshiro256**/splitmix64 description; do NOT overwrite the whole
note — the docgen name-keyed merge lesson). Add a `ReseedRandom` note
(e.g. "reseeds the session PRNG from host entropy; requires
Capability::Random and is denied under sandboxed()."). Regenerate:

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

`builtins.md` diff = the new `ReseedRandom` entry + the edited `SeedRandom`
Limits line.

## Change 5 — tests

- `crates/rum-stdlib/src/tests/catalog_capabilities.rs`: add `ReseedRandom`
  → `Capability::Random`; `SeedRandom` stays `Pure`.
- the random unit tests (wherever `SeedRandom` is tested — likely
  `crates/rum-stdlib/src/tests/*random*` or an incidental file): `SeedRandom[]`
  now `argx`; `ReseedRandom[]` reseeds (returns `Null` in unrestricted
  policy); `SeedRandom[42]` still deterministic/reproducible.
- `crates/rum-host/src/tests.rs`: the entropy capability tests deny
  `SeedRandom[]` today (e.g. `assert_sandbox_denied("SeedRandom[]")` and the
  `["RandomBytes[8]", "RandomPassword[8]", "SeedRandom[]"]` set) — replace
  those `SeedRandom[]` denials with `ReseedRandom[]`; confirm `SeedRandom[42]`
  is NOT denied in sandbox (deterministic); under a Random-allowed policy,
  `ReseedRandom[]` succeeds.

## Change 6 — guide (`docs/guide/resource-limits-and-sandbox.md`)

Line ~185 says `SeedRandom[]` reseeds from host entropy. Update: the
entropy reseed is now `ReseedRandom[]` (requires `Random`); `SeedRandom`
takes an exact integer seed and is deterministic/`Pure`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{SeedRandom[42], RandomInteger[100], SeedRandom[42], RandomInteger[100], ReseedRandom[], Attributes[ReseedRandom]}'
printf '' | cargo run -q -p rum-cli -- --eval '{SeedRandom[], ReseedRandom[5]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — reference
regenerated, +1 builtin, capability + host tests updated). First probe:
`SeedRandom[42]` then `RandomInteger[100]` gives the SAME value both times
(deterministic reproducibility intact); `ReseedRandom[]` → `Null` (trusted
CLI); `Attributes[ReseedRandom]` → `{}` (not Protected yet). Second probe:
`SeedRandom[]` and `ReseedRandom[5]` both `argx` errors. In `sandboxed()`,
`ReseedRandom[]` is denied (`General::sandbox`) while `SeedRandom[42]`
works (host tests).

## Scope

- MAY modify: `crates/rum-stdlib/src/catalog/random.rs`,
  `crates/rum-stdlib/src/random.rs`,
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`,
  `crates/rum-stdlib/src/docgen.rs`, `docs/reference/builtins.md`
  (regenerated), `crates/rum-stdlib/src/tests/catalog_capabilities.rs`, the
  random test file(s) under `crates/rum-stdlib/src/tests/**`,
  `crates/rum-host/src/tests.rs`,
  `docs/guide/resource-limits-and-sandbox.md`.
- MUST NOT touch: any other random function (RandomInteger/RandomChoice/
  RandomSample/RandomString/RandomUuid/RandomBytes/RandomPassword unchanged);
  `AbsoluteTime` or any kernel/timing builtin; `Now`/datetime; `TODO.md`,
  the tracker, findings files, other briefs, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- `ReseedRandom` is gated by SPEC-level `Capability::Random` ONLY — NO
  manual check in the handler; DELETE `random_capability_allowed` once
  unused.
- `SeedRandom` stays `Capability::Pure`, deterministic-only; `SeedRandom[]`
  becomes an arity error, NOT a sandbox denial. `SeedRandom[seed]`
  reproducibility MUST be byte-identical (verify in the probe).
- `ReseedRandom` is NOT `Protected` (match the not-yet-swept random family;
  unit 7 protects them together).
- docgen LIMITS is name-keyed: EDIT the `SeedRandom` note (drop the entropy
  sentence); do NOT overwrite its other content. Regenerate `builtins.md`
  with the generator bin — do NOT hand-edit it.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the `ReseedRandom` spec+handler, the `SeedRandom`
   entropy-arm removal + `random_capability_allowed` deletion, the mirror,
   docgen+regen, the tests, the guide; line ranges.
2. Verification — all commands verbatim (test totals + both probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
