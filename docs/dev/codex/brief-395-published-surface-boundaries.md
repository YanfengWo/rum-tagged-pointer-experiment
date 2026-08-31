# brief-395 — two boundaries that hold inside the crate and not across it

The brief-390 and brief-391 reviews each found a value whose construction was
described as restricted, and each was reachable from outside the crate. They
are one shape and are repaired together.

## The two instances

**A public installer stores an unprepared left-hand side.**
`Session::install_system_down_value` (`session/session_api.rs:225`) is `pub`,
takes an arbitrary `Expr`, and mints the trusted token internally, so
preparation never runs (`prepared_store.rs:385`). `#[doc(hidden)]` does not
restrain a downstream Rust caller. brief-391 compiled an external crate,
assigned `i391 = 2`, installed `ExternalRaw391[i391]`, and observed:

```
lookup=ExternalRaw391[2]
stored=List[RuleDelayed[HoldPattern[ExternalRaw391[i391]], 7]]
```

The stored identity is unreachable by ordinary lookup — precisely the defect
class briefs 376-383 closed, reintroduced through a published API.

**A typed selector wrapper can be built from raw expressions.**
`EvaluatedPartSelectors` is meant to be obtainable only by exactly-once
evaluation or by a stable-literal proof. brief-390 compiled an external crate
that built one through the exported continuation decoder (`kernel_api.rs:18`
and `:100`, `evaluated_part_selectors.rs:68`) and reached the parser with it.
The forged value failed to parse, so today's impact is small — but the
guarantee is the entire reason that type exists.

**Why they are one item.** In both cases the boundary is enforced within the
repository and evaporates at the published surface, and in both cases the test
that was supposed to police it — the sealing call-site inventory
(`definition_store_sealing.rs:130`) — scans in-repository callers and
structurally cannot see an external one. **A source-scanning test cannot
establish a property about code that does not exist yet.**

## Goal

Make both guarantees hold for a caller outside the crate, and make the
mechanism one that a source scan is not required to enforce.

Directions to weigh; propose better if you see it:

- **Narrow the export.** If nothing outside needs
  `install_system_down_value` or the continuation decoder as published, remove
  or restrict them. Establish who actually calls each — `rum-stdlib`, the
  binaries, tests — before assuming anything needs the current visibility.
- **Move the obligation into the type.** If the installer must stay public,
  it can prepare its own argument rather than trust the caller, so a raw lhs
  becomes impossible to store rather than merely discouraged. Say what that
  costs on the static-catalog path, which is the one legitimate caller and runs
  at startup for every builtin.
- **Keep the surface and make it safe.** A published constructor that performs
  the preparation itself is not a hole; a published constructor that accepts an
  already-built token is. Distinguish the two for both instances.

**The trusted static-catalog path must keep working**, and so must whole-state
restoration. They are the two deliberate non-preparing paths; the question is
whether they can stay reachable without also being forgeable.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The external-caller gate

The reviews demonstrated both holes by compiling an external crate. **Do the
same in reverse**: after the change, an external crate attempting each of the
two constructions must fail to compile, and you must paste the compiler error.
A test inside the repository cannot demonstrate this — that is the whole
finding.

Keep the probe crate under `/tmp`, paste the errors, and delete it. If a
construction still compiles but is now harmless, that is a weaker result and
you must say so explicitly rather than calling it fixed.

Also confirm the legitimate paths still work: the static catalog installs, and
a snapshot restores.

## Scope

Files you MAY modify: `crates/rum-runtime/src/runtime/session/session_api.rs`,
`crates/rum-runtime/src/runtime/kernel_api.rs`,
`crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/**`,
`crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs`, the
call sites those changes force, and the test modules covering sealing and
selectors.

**The MAY list is a floor, not a ceiling** for purely mechanical ripple — a
call site forced by a changed signature, a re-export forced by a moved item.
List each under "mechanical ripple". Anything changing behavior in an unlisted
file: STOP and report.

Out of scope: `Expr`/`Symbol` representation, the evaluator's dispatch,
`Cargo.lock`, `TODO.md`, `PROGRESS.md`, `AGENTS.md`, `docs/**` other than a
doc comment on a changed item.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`.
- **No unconditional cost on the ordinary evaluation path.** If the installer
  starts preparing, that cost lands on catalog installation at startup — say
  what it is, measured, not estimated.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`.
- **Stop-loss** — report rather than improvise if: a consumer genuinely needs
  the current published surface and cannot be served by a preparing
  constructor; narrowing would break the binaries or `rum-stdlib` in a way that
  needs a design decision; or startup cost becomes measurable.
- Paste real verbatim output, including the external crate's compiler errors.

## Report format

1. Who calls each published item today, with `path:line`.
2. The approach for each instance, and what it guarantees.
3. What changed — per file, line ranges.
4. Mechanical ripple.
5. Verification — `fmt`, workspace counts, the external crate's compiler
   errors for both constructions, and confirmation that catalog installation
   and snapshot restore still work.
6. Stop-loss / open questions.
