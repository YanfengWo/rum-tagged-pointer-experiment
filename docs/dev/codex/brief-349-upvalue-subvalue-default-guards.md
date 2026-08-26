# brief-349 — rhs `Condition` guards, step 2: `UpValues`, `SubValues`, `DefaultValues`

## Goal

Extend the guard substrate landed in `5ab6afc5` to the three remaining
definition tables. They fail today exactly the way `DownValues` did:

```text
u /: h[u, x_] := "up-hit" /; x > 0
u /: h[u, x_] := "up-other"
h[u, 3]  ->  "up-other"     (must be "up-hit";  UpValues[u] has 1 entry, must be 2)

s[1][x_] := "sub-hit" /; x > 0
s[1][x_] := "sub-other"
s[1][3]  ->  "sub-other"    (must be "sub-hit"; SubValues[s] has 1 entry, must be 2)

Default[dv] = 7 /; True
dv[a_., b_.] := {a, b}
dv[1]    ->  {1, 7 /; True} (must be {1, 7})
```

**Reuse the existing substrate.** `definitions/rhs_guards.rs` already provides
flattening, rebuilding, joint alpha-canonical identity, guarded-only matcher
compilation, and the shared exact-`True` evaluator with its suspension
handling. Do not add a second mechanism, and do not change the substrate's
semantics — if it needs a genuine extension, say so in the report and explain
why.

**Only these three tables.** Generic rewrite rules, `ReplacePart`, string
rules, the exception-handler scan, and `OwnValues` stay exactly as they are,
including their current wrong behavior.

## Context pointers

- `5ab6afc5` is the model to follow end to end — read its diff first.
- `crates/rum-runtime/src/runtime/definitions/rhs_guards.rs` — the substrate.
- `crates/rum-runtime/src/runtime/model/definitions.rs:62-69` `UpValue` and
  `:83-91` `SubValue` are structurally the same as `DownValue` was; `:80` shows
  where `guards` went. **Put the new field last** — moving it there was a
  measured decision in the previous step, to preserve existing field offsets.
- `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:206-240` —
  installation, which currently splits `DownValue` rhs guards and
  **explicitly preserves `DefaultValues`**. That exclusion is what this brief
  removes.
- `crates/rum-runtime/src/runtime/definitions/application/up_values.rs`,
  `sub_values.rs`, `default_values.rs` — the three application paths.
  Note `DefaultValues` are stored as `DownValue` entries whose lhs is
  `Default[sym]` (`default_values.rs:5-30`), so its guard field already
  exists; only the split exclusion and the application need work.
- Acceptance is mechanical: the named tests and A/B rows decide.

## Deliverable

- `UpValue` and `SubValue` carry an ordered guard sequence, split after
  evaluation for the immediate heads (`UpSet`, `TagSet`) and directly for the
  delayed ones (`UpSetDelayed`, `TagSetDelayed`, plain `SubValue`
  `SetDelayed`).
- Identity for both is lhs plus the ordered guard sequence, alpha-canonicalized
  jointly, exactly as `DownValue` does it. Two rules differing only by guard
  coexist; same lhs and same guards is still a redefinition in place.
- On a match, guards evaluate left to right with bindings substituted and the
  guarded body's option scope; only exact `True` accepts; a decline advances
  to the next candidate exactly as an lhs mismatch does.
- `DefaultValues` split their guards on installation and evaluate them when a
  default is consulted. **A declining guard means that default does not
  apply** — continue to the next default candidate, and if none applies the
  optional argument has no default, exactly as if the declining default had
  never been defined. Note that this runs user code *during matching*: report
  what that means for suspension safety, and make sure a suspension inside a
  default's guard resumes at the same candidate without replaying the match.
- `UpValues[u]`, `SubValues[s]`, `DefaultValues[d]`, `Definition[...]`, and
  package serialization rebuild the `Condition` form.
- Clearing stays an lhs selector: it removes every guard variant.

### Tests

Exact values or exact table listings, never just absence of error. For each of
the three tables:

- guarded hit and guarded decline-with-fallback;
- coexistence: two rules with the same lhs and different guards, both present
  in the table listing, tried in order;
- redefinition: same lhs and same guard replaces in place;
- nested guards, and a two-guard rule where the second declines;
- non-boolean guard results decline (symbol, integer, unevaluated call);
- a guard that suspends (`InputString`) resumes at the same candidate with no
  replay — assert a counter, as `matcher_resume.rs` does;
- immediate versus delayed installation heads;
- clearing removes all guard variants;
- inspection round-trip: the rebuilt form re-installs to the same behavior.

Plus, for `DefaultValues` specifically: a declining default leaves the
optional argument without a default, and the surrounding match behaves as if
that default did not exist.

### A/B

Base versus candidate for the existing rows plus new stable labels for the
two tables that have none:

```
upvalue-candidates-tiny  upvalue-candidates-wide-small  upvalue-candidates-wide-large
downvalue-exact-hit      downvalue-pattern-hit
matcher-single-tiny      set-ownvalue                   symbol-ownvalue-read
```

Add: an unguarded subvalue hit and miss, an unguarded default hit and miss,
and one guarded row per table.

**On judging the result**: the previous step reported two regressions at
`--scale 1` that did not survive re-measurement — one even reversed sign — and
the same binary run twice moved those labels by up to 3.8%. Run at
`--scale 5`, and before calling anything a regression, run the *same* binary
twice and report that spread as your noise floor. Name the untouched control
labels you judge against; never the suite median.

## Scope

- Files you MAY modify: `model/definitions.rs`; `definitions/rhs_guards.rs`
  only if a genuine extension is needed; `definitions/definition_tags.rs`;
  `definitions/definition_analysis_helpers.rs`;
  `definitions/definition_clear.rs`; `definitions/application/up_values.rs`,
  `sub_values.rs`, `default_values.rs`; the `definition_catalog` entries
  needed for reconstruction; runtime tests; `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported** — adding a field to
  `UpValue`/`SubValue` will break exhaustive literals, equality, inspection,
  package save, GC traversal, and memory accounting, the same set `5ab6afc5`
  touched. Fix those compile sites and justify each file in the report. If a
  site needs a real decision rather than a mechanical fix, STOP and report.
- **The general accept/stop criterion** — use this, not the file list: *any
  behavior change that occurs only when a guarded definition exists is
  legitimate; any behavior change observable with no guarded definition
  present is a stop.*
- MUST NOT touch: generic rewrite rules, `ReplacePart`, string rules,
  exception handlers, `OwnValues`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  `docs/**`, `.git`, `Cargo.lock`, `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **The unguarded path must cost nothing** — no new per-candidate branch,
   allocation, or clone. Empty guards use the same `Box::default()` shape and
   unguarded compilation returns the original matcher directly.
2. **Guards run user code**, so preserve `suspend_with_frame*` continuation
   shape. The default-value path is the riskiest: it runs during matching.
3. **Reuse the substrate.** A second guard mechanism, or a divergent
   accept rule, is worse than the bug.
4. **Identity is syntactic after alpha-canonicalization** — do not simplify
   `True`, reorder guards, or use semantic equivalence.
5. **Counters before timings**; a claim that a counter does not move needs no
   timing at all.
6. **Stop-loss is real.** If a step fails twice for the same reason, STOP and
   report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B as described above, pasting per-label rows and the same-binary
noise-floor spread.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no third-party engine comparison anywhere.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`,
  with bounds derived from measured runs of the same shape.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What changed — per file, mechanically-forced files listed separately and
   justified.
2. Anything the substrate needed, and why it was a genuine extension.
3. Verification — each command with verbatim output.
4. The exact-value test matrix, per table.
5. What running a guard during matching means for the default-value path.
6. The A/B — counters first, then rows, with the noise floor from the same
   binary run twice and the control labels named.
7. Stop-loss / open questions.
