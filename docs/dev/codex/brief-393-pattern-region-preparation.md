# brief-393 — preparation must not evaluate inside pattern regions

Found by the brief-391 review, reproduced by the director. This is family B's
own invariant failing on the pattern forms nobody enumerated.

## The defect

`docs/guide/evaluation.md` and the preparation contract say preparation must
never apply a rule to pattern syntax. The policy implements that by listing the
wrappers it protects, and the list is short.

Director's probes on the current tree:

```
typeAlias = Integer;
fV[Verbatim[Blank[typeAlias]]] = 7
  stores  fV[Verbatim[_Integer]]          <- Verbatim's contents were evaluated

cnt = 0; sideCall[] := (cnt = cnt + 1; zzz);
fP[PatternSequence[sideCall[], b_]] = 8
  stores  fP[PatternSequence[zzz, b_]]    with cnt == 1
                                          <- the call ran and was replaced

fN[x_] = 9
  stores  fN[x_]                          <- ordinary binders are correct
```

`Verbatim` means verbatim; evaluating its contents contradicts what the wrapper
is for. brief-391 named seven forms the policy omits while the matcher supports
them: `Verbatim`, `PatternSequence`, `OrderlessPatternSequence`,
`KeyValuePattern`, `Longest`, `Shortest`, and `OptionsPattern`
(`definition_lhs_preparation.rs:691` for the planner and detector;
`rewrite_data/cases/match_q.rs:192`, `rewrite_data/matcher_and_symbols.rs:218`,
and `rewrite_data/options/patterns.rs:138` for what the matcher recognizes).

**The ordinary binder case is correct, so the policy's shape is right and only
its list is short.** That is the whole defect and also the whole trap.

## Goal, and the part that matters

Adding seven names to a list would close today's probes and leave the next
pattern form to be added exposed in exactly the same way. **Do not do that as
the whole repair.**

The list is the defect. Find the arrangement where preparation cannot be wrong
about a pattern form it has not heard of. Options to weigh, and you may propose
better:

- **A single source of truth.** The matcher already knows which heads are
  pattern syntax. If preparation can consult that same source rather than keep
  a parallel list, the two cannot drift.
- **Invert the default.** Treat an unrecognized head inside an lhs
  conservatively — do not evaluate — so a new pattern form is safe before
  anyone teaches preparation about it, and only heads known to be ordinary
  calls get evaluated. State what this costs: some currently-evaluated position
  may stop evaluating, and that is a behavior change to enumerate, not to
  discover later.
- **Make drift fail the build.** If neither of the above is achievable, a test
  that derives the expected set from the matcher's own registry and fails when
  the two differ is the fallback — weaker, because it catches drift rather than
  preventing it, but it is enforcement rather than vigilance.

Whichever you choose, say plainly which property it buys and what it costs. If
you conclude the list is genuinely irreducible, that is a finding worth
reporting, but argue it rather than assuming it.

**Per-form semantics are not uniform**, so the repair is not one flag:

- `Verbatim` — contents entirely untouched.
- Pattern binders (`Pattern[name, p]`) — the name stays raw, the pattern is
  prepared.
- `PatternTest`, `Condition` — the pattern operand is prepared, the predicate
  stays deferred.
- `HoldPattern`, `MessageName`, `Default` — opaque to user evaluation.
- The sequence and quantifier wrappers — establish what each requires; do not
  assume they match any of the above.

Report the table you establish; it is part of the deliverable.

## Required first step — inventory

Before changing anything, enumerate every head the matcher treats as pattern
syntax, with `path:line`, and for each: what preparation currently does with
it, and what it should do. Say which of the seven named above are the whole set
and whether there are more.

## Scope

Files you MAY modify: `crates/rum-runtime/src/runtime/definitions/
definition_lhs_preparation.rs` and its submodules, whatever single source of
truth your chosen approach requires, `docs/guide/evaluation.md`, and the
runtime and stdlib test modules covering lhs preparation and patterns.

**The MAY list is a floor, not a ceiling** for purely mechanical ripple — a
call site forced by a changed signature, an exhaustive match arm forced by a
new variant, a re-export forced by a moved item. List each under "mechanical
ripple". Anything changing behavior in an unlisted file: STOP and report.

Out of scope: the matcher's own semantics — you may read from it, not change
what it recognizes; `Expr`/`Symbol` representation; `TODO.md`, `PROGRESS.md`,
`AGENTS.md`, other `docs/**`.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### Test matrix

- Each of the seven named forms, asserting the stored identity keeps what must
  stay raw. `Verbatim[Blank[typeAlias]]` with `typeAlias = Integer` must store
  `Verbatim[Blank[typeAlias]]`.
- A liveness counter inside each form that must not evaluate — the
  `PatternSequence[sideCall[], b_]` shape — asserting the counter stays zero.
  A test that only checks the stored form would pass if evaluation happened
  and the result coincided.
- The forms that must still prepare — an active position outside any pattern
  region, a `PatternTest` pattern operand — asserting they still do. **This
  repair can over-correct**, and an lhs that stops preparing where it should is
  as wrong as one that prepares where it should not.
- Ordinary binders and typed blanks unchanged.
- Whatever mechanism you chose to prevent drift, exercised.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`.
- **Do not change what the matcher recognizes as pattern syntax.** If the
  matcher's set looks wrong, report it; this brief aligns preparation to the
  matcher, not the reverse.
- No unconditional cost on the ordinary evaluation path. Preparation runs per
  assignment; say what a pattern-free lhs pays.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`.
- **Stop-loss** — report rather than improvise if: no single source of truth
  exists and inverting the default would change behavior you cannot enumerate;
  a form's correct treatment is genuinely ambiguous; or aligning to the matcher
  would require changing it.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. The inventory — every pattern head, what preparation does, what it should
   do, with `path:line`.
2. The approach chosen, the property it buys, and its cost.
3. What changed — per file, line ranges, one-line why.
4. Mechanical ripple.
5. Verification — `fmt`, workspace counts, and the per-form matrix including
   the zero-counter cases.
6. Stop-loss / open questions.
