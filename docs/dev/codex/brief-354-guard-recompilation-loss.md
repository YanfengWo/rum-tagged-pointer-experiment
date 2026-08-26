# brief-354 — guarded `DownValue` matchers lose their guard on recompilation

## Goal

The rhs-guard line (briefs 347-353) has a hole. A guarded `DownValue` stops
enforcing its guard as soon as the attribute revision is bumped — which any
`SetAttributes` anywhere in the session does. Fix it, and prove the other
lanes are safe by mechanism rather than by probe.

Measured on `9711c4fa`:

```text
g1[x_] := x^2 /; x > 0
g1[-3]                                   ->  g1[-3]     correct
SetAttributes[unrelatedSym, HoldFirst]
g1[-3]                                   ->  9          WRONG, guard bypassed
```

`ClearAttributes` does it too, as does setting an attribute on the guarded
symbol itself. So does anything that restores a loader snapshot:

```text
b[x_] := x^2 /; x > 0
t = b[-3]           ->  b[-3]
ValueQ[b[-3]]       ->  False
b[-3]               ->  9                WRONG
```

The damage is **symbol-wide**, not per expression, and it happens whether the
guard would have accepted or declined:

```text
ValueQ[b4[-3]];   b4[-5]  ->  25    b4[-3]  ->  9
ValueQ[b3[3]];    b3[-3]  ->  9
```

`UpValues`, `SubValues`, `DefaultValues` and generic rewrite rules were **not**
reproducible this way — confirm that from the code, do not take my probes as
proof.

## The mechanism to confirm

`restore_loader_snapshot` calls `bump_attribute_revision()`
(`session/session_helpers.rs`), and `SetAttributes`/`ClearAttributes` do the
same. A stale `matcher_attribute_revision` triggers cold recompilation of the
`DownValue` matcher at
`definitions/application/down_values.rs:258-264`, which brief-348 claimed
reconstructs the guarded matcher wrapper. It evidently does not, or another
recompilation path exists that does not. **Find the real path and say what it
was**, with `path:line` evidence.

## Deliverable

- A guarded `DownValue` enforces its guard after any number of attribute
  revision bumps, from every trigger: `SetAttributes` and `ClearAttributes` on
  the guarded symbol and on unrelated symbols, `ValueQ`, `eval_file` rollback,
  and any other caller of `bump_attribute_revision` you find — enumerate them.
- The same guarantee proved for `UpValues`, `SubValues`, `DefaultValues` and
  generic rewrite rules: either they never recompile this way, or they
  reconstruct correctly. State which, per lane, with evidence.
- **Extend the whole-line regression test** added in `7c1e91d8` with an
  attribute-bump dimension: every lane's guarded rule must still be enforced
  after a `SetAttributes` on an unrelated symbol and after a `ValueQ`. This
  was the blind spot that let the defect through six gates.
- If the guard wrapper can be made structurally impossible to lose — for
  instance by reconstructing it from the stored guard sequence at every
  compilation entry point rather than at the one that happened to be patched —
  prefer that over adding a second patched call site, and say why your choice
  is the robust one.

### Tests

Exact values, never absence of error:

- Each trigger above, per lane, guard accepting and declining.
- Repeated bumps: two `SetAttributes` in a row, then evaluate.
- A guard on a symbol that itself gains an attribute (`Listable` is the
  interesting one, since it changes matching).
- Interleaving: define guarded rule, bump, define another guarded rule for the
  same symbol, bump, evaluate both.
- The `ValueQ` sequence from the Goal, asserting the value **after** the
  `ValueQ`, not only the `ValueQ` result — that assertion order is what my
  gate missed.

### A/B

This is a correctness fix on a cold path, so the gate is the test suite. Run
the microbench only if you touch a hot path; if you do, `--scale 5` with the
same-binary noise floor reported first, and these labels:

```
downvalue-exact-hit  downvalue-pattern-hit  downvalue-exact-miss
matcher-single-tiny  symbol-ownvalue-read
```

## Scope

- Files you MAY modify: the definition matcher compilation and recompilation
  paths under `crates/rum-runtime/src/runtime/definitions/`;
  `model/definitions.rs` if the fix belongs in the data model;
  `definitions/rhs_guards.rs`; runtime and stdlib tests.
- **Mechanical ripple is in scope but must be reported**, each file justified.
- **The general accept/stop criterion**: *any behavior change that occurs only
  when a guarded definition exists is legitimate; any behavior change
  observable with no guarded definition present is a stop.*
- MUST NOT touch: `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`, `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **The unguarded path must stay free.** Whatever makes the guard
   reconstruction robust must not add work to unguarded recompilation.
2. **Do not fix this by disabling recompilation** for guarded definitions —
   attribute changes must still be reflected in the matcher.
3. **Assert after the trigger.** A test that only checks the guard before the
   bump is the test that let this through; every new test asserts the value
   after the bump.
4. **Reuse the substrate**; do not add a second guard mechanism.
5. **Stop-loss is real.** If the same step fails twice for the same reason,
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. The real mechanism, with `path:line` evidence, and why brief-348's claimed
   fix did not hold.
2. What changed — per file, mechanically-forced files listed separately.
3. The `bump_attribute_revision` caller inventory.
4. Per-lane safety: which lanes recompile, which reconstruct, which never do.
5. Verification — each command with verbatim output.
6. The exact-value test matrix, including the extended whole-line test.
7. Stop-loss / open questions.
