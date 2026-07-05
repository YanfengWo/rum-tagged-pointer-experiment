# Brief 134 — Clock capability; determinism-by-default terminology

Implementation brief. Third step of `docs/dev/fs-purity-plan.md` (capability
口径 correction decided 2026-07-06). Classification + wording only; zero
evaluator-semantics changes.

## Goal

`Capability::Pure` becomes a true statement: a `Pure` builtin's observable
result depends only on session state and arguments. Wall-clock readers move
to a new `Capability::Clock`; `HostConfig::sandboxed()` (still `{Pure}`)
therefore yields **deterministic** evaluation. The docs adopt the two
defensible claims — "no ambient authority by construction" and
"deterministic by default" — and stop using bare "pure" as a language-level
claim. `BlockRandom` and `MemoryConstrained` are audited and dispositioned.

## Audit rule (apply to the whole installed catalog)

A builtin is NOT `Pure` if its observable result or effect can depend on
anything besides session state and its arguments: wall clock, machine memory
state, randomness, environment. Mechanical helper: rg for
`SystemTime::now|Instant::now|elapsed\(` inside builtin implementations
(kernel + stdlib, non-test). NOTE: the eval-loop timeout/cancel machinery
(brief-059) and the stack guard (brief-131) legitimately read clocks/stack —
they are host-limit machinery, not builtin results; leave them alone.

## Known reclassifications (verify each against the implementation)

- `AbsoluteTime`, `AbsoluteTiming`, `Timing`, `RepeatedTiming` → `Clock`.
- `TimeConstrained` → `Clock` (observable outcome depends on the deadline).
- `MemoryConstrained`: audit. If its observable behavior depends on machine/
  allocator state → `Diagnostic`; if it is a deterministic cooperative work
  proxy → stays `Pure` with a reference limits note. Report the evidence.
- `BlockRandom`: audit. If NO random-generator builtins exist (expected),
  REMOVE it entirely (brief-132 pattern: archive verbatim copies under the
  mirrored path + MANIFEST entry, placeholder 移除 commit
  `（由 Claude 在提交时回填）`; note: future randomness = host-injected
  seeded RNG in a downstream capability crate). An inert compatibility shell
  with no backing feature does not ship.
- `DateObjectQ`, `TimeObjectQ`: expected to stay `Pure` (head predicates, no
  clock) — verify.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/capabilities.rs` (+ the
  `Capability` enum and its exhaustive match sites), the affected builtins'
  registrations/catalog rows, `crates/rum-host/src/**` (docs for the new
  variant; a new policy test), tests covering capability classification,
  `docs/reference/` via the generator, and EXACTLY these doc edits:
  - `docs/guide/resource-limits-and-sandbox.md` §Capability Classes: add
    `Clock` with its members; add a short paragraph: `sandboxed()` sessions
    are deterministic — results depend only on session state and source;
    grant `{Pure, Clock}` when scripts need time.
  - `docs/guide/design-boundaries.md`: a short new "Determinism" section
    stating the two claims (no ambient authority by construction;
    deterministic by default) and that FP referential transparency is NOT
    the claim (the language has session state by design).
  - `docs/guide/embedding.md` §Sandbox Configuration: one sentence on
    `{Pure, Clock}`.
  - `README.md`: in the opening sandbox sentence, add "evaluation is
    deterministic by default" phrased minimally.
- MAY write outside the repo: ONLY the BlockRandom archive copies +
  MANIFEST entry (if removal happens).
- MUST NOT touch: evaluator/dispatch logic, the host-limit clock machinery,
  matcher, scripts, other docs.

## New tests

- rum-host: a `{Pure, Clock}` policy session evaluates `AbsoluteTime[]`
  successfully while `Get[...]` is denied; a `sandboxed()` session gets the
  capability denial for `AbsoluteTime[]` (paste both).
- Classification tests updated to the new arms (the exhaustive match forces
  completeness at compile time).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference
awk '/^### /{name=$2} /- Capability:/{print $3, name}' docs/reference/builtins.md | grep -vE '`Pure`' | sort
# ^ paste the full non-Pure list — it is the product of this brief
rg -c '^### ' docs/reference/builtins.md
cargo run -q -p rum-cli -- --eval 'AbsoluteTime[]'      # trusted CLI default: still works (paste)
git status --short
```

Plus the two rum-host policy tests above with `--nocapture` output.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Classification and wording only — no behavior change for the trusted
  default (the CLI keeps whatever capability set it has today; if the CLI's
  default policy needs `Clock` added to stay unchanged, do that and say so).
- No new `unsafe`. Minimal diff. Stop-loss: same failure twice → STOP.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The full audit result — every non-Pure builtin with its class and one-line
   justification; the BlockRandom and MemoryConstrained verdicts with
   evidence.
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
