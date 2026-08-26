# brief-345 — `ValueQ` clones the whole session per call (design only)

## Goal

`ValueQ[x]` costs 4.6 us in a fresh session and **1.94 seconds** in a session
holding 32 000 definitions — a cost that scales with session history rather
than with the question asked. It captures a full rollback snapshot, trial-
evaluates, and restores, on every call. Produce a diagnosis and a
step-by-step repair plan with drafted follow-up implementation briefs.

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes
with the release `rum-cli` are in scope; leave no probe files in the
repository, and feed every probe `stdin < /dev/null`.

The answer is not obvious and three different shapes of repair are plausible.
Comparing them honestly, with measurements, is the deliverable — not picking
one and defending it.

## Measured evidence (director, release CLI at `c18b01f9`)

Definitions were installed as N distinct own-values (`v1 = 1` … `vN = N`) from
a generated script, then `ValueQ[qq]` — an undefined symbol, the cheapest
possible question — was called twenty times under `AbsoluteTiming`:

```text
defs=0      live v1=False  ValueQ[qq]=False  t20 = 92.143 us    ->   4.6 us/call
defs=2000   live v1=True   ValueQ[qq]=False  t20 = 1.8126 s     ->  90.6 ms/call
defs=8000   live v1=True   ValueQ[qq]=False  t20 = 6.6514 s     -> 332.6 ms/call
defs=32000  live v1=True   ValueQ[qq]=False  t20 = 38.783 s     ->   1.94 s/call
```

`live v1` proves the definitions were actually installed, and `ValueQ[qq]`
returning `False` proves the calls really ran. Growth from 2 000 to 32 000 is
roughly linear-to-superlinear; establish the true shape yourself.

## Context pointers

- Read `AGENTS.md` and `docs/testing-strategy.md` before touching anything.
- `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:37-56`
  — `eval_value_q`: unconditional `capture_evaluation_rollback_snapshot`,
  `evaluate`, `restore_evaluation_rollback_snapshot`, then a comparison
  against the original expression.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:40-102` —
  `capture_loader_snapshot` / `restore_loader_snapshot`: twenty-plus cloned
  collections including `definitions`, `attributes`, `message_definitions`,
  `messages`, `trace_events`, `symbol_generations`, `sealed_units`.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:104-158` — the
  evaluation-level wrapper around it.
- **The only two consumers** of this machinery are `eval_value_q` and
  `eval_file` (`session/session_eval.rs:340`, once per loaded file, where the
  cost is irrelevant). Verify that inventory yourself and report it — a repair
  that helps `ValueQ` must not silently change loader rollback.
- `TODO.md` records under Standing engineering debt: "`Monitor` refresh and
  `ValueQ` rollback remain best-effort unless their documented contract is
  deliberately promoted." Find out what that actually means in code before
  proposing anything.

## What the plan must contain

1. **The cost, decomposed.** Which of the twenty-plus cloned fields dominate,
   measured — not inferred from the field list. Counters and allocation counts
   before timings; a field that does not move a counter does not get a time
   quoted for it. State the true growth shape and whether it is linear in
   definition count, in total definition size, or in something else.

2. **The contract, established from the code and the docs.** What does
   `ValueQ` currently promise? What does the snapshot actually roll back, and
   what does it demonstrably fail to roll back (that is what "best-effort"
   must mean — find the cases)? Quote `docs/reference/builtins.md` and any
   guide text. **If you find a real correctness hole here, say so plainly and
   rank it above the performance work** — a silent wrong answer outranks a
   slow right one, and this project has twice found a feature gap while
   probing for speed.

3. **Three repair shapes, compared on the same axes** (correctness risk,
   default-path cost, blast radius, expected win). Do not pre-select one:
   - **(a) Elide the snapshot for provably effect-free targets.** What
     existing analysis could certify "no observable side effect", and is it
     sound for that purpose or only for cache/stability decisions? What
     fraction of realistic `ValueQ` calls would qualify — in particular the
     dominant `ValueQ[undefined-symbol]` and `ValueQ[symbol-with-own-value]`
     shapes?
   - **(b) Bounded/targeted rollback**: record what the trial evaluation
     touched and undo exactly that. Where would the recording live, and what
     does it cost on the ordinary assignment path when no `ValueQ` is active?
     **A per-assignment branch, allocation, or lock paid when the feature is
     off is a rejected design** — say honestly whether the mechanism can be
     confined to the trial window.
   - **(c) Make the snapshot itself cheap** — shared/copy-on-write ownership
     so capture is O(1) and only mutated collections are deep-copied. What
     would that cost the hot paths that read and write those collections, and
     what would it do to `eval_file`? No new dependencies.

4. **A recommendation with its evidence**, and the smallest first step that
   delivers a measurable win without foreclosing the others.

5. **A step-by-step plan**, each step naming files/functions, the invariant it
   preserves, its gate tests, its ripple/call-site inventory, and which steps
   need the microbench A/B.

6. **Drafted follow-up implementation briefs**, in `TEMPLATE.md` shape — real
   drafts, not step summaries. They are raw material for the director's own
   briefs.

7. **Open questions for the director**, each with your recommendation. The
   `ValueQ` contract question — whether "best-effort" gets promoted to a
   promise, stays as it is, or gets documented as a limitation — is the
   director's to settle, not yours; give your recommendation and the evidence.

## Scope

- Files you MAY modify: **NONE**. Design-only. Put probes in `/tmp`.
- Files/areas you MUST NOT touch: everything, including `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`. Your report is
  the deliverable.

## rum / Rust pitfalls (read twice)

1. **Counters beat timings.** Allocation and call counters are immune to
   machine load; a conclusion that rests only on a wall time is weaker than
   one that rests on a counter. Report counters first.
2. **Every probe carries a liveness assertion** — something proving the work
   happened (a printed value, a definition count, a counter delta). A probe
   that measures a failing path scales beautifully while measuring nothing.
3. **Check that time scales with N before trusting any single timing.** A
   number that does not grow with the workload is measuring startup.
4. **The microbench now runs up to `--scale 10`** and fails loudly above it
   (`d9c713e1`); it produces no `timings.tsv` when the run failed. If you get
   a header-only or missing file, the run failed — do not report its absence
   as a flat result.
5. **No default-path cost.** Any counter or instrumentation goes behind the
   env-gated cold-load `AtomicU8` pattern. This applies to anything you
   propose, not just to what you write.
6. **`Expr`/`SharedExpr`/`SharedArgs` are `Arc`-shared.** Before proposing a
   copy-on-write redesign, establish what is already shared and what is
   genuinely deep-cloned — the answer may be narrower than the field list
   suggests.
7. **Stop-loss is real.** If the decomposition does not resolve after two
   serious attempts, report what you established and where the trail ends.

## Verification (MANDATORY)

No source change, so no test gate. The evidence IS the deliverable:

- Paste the field-level cost decomposition with the counters backing it.
- Paste the verbatim source lines (`path:line`) for every mechanism you name.
- Paste the consumer inventory for the snapshot machinery.
- Paste the probes for any contract hole you claim, including the liveness
  assertion in each.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config, and do NOT modify
  any file in the repository.
- Touch nothing outside this repo except `/tmp`; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No black-box comparison against another vendor's engine, no
  oracle harness, no compatibility matrix, no third-party engine output.
- No new `unsafe` may appear in any drafted plan for `rum-core`/`rum-runtime`,
  and no new dependencies.
- Bounded probes: `scripts/run-bounded.py --timeout SECONDS -- <command>`;
  size bounds from a measured run of the same shape and state them. Note that
  a 32 000-definition probe takes tens of seconds per twenty calls — derive
  your bounds from the table above rather than guessing.

## Report format (your final message)

1. Diagnostic only, no source change — state it.
2. The cost decomposition, counters first.
3. The contract as it actually is, with any correctness hole ranked ahead of
   the performance work.
4. The three repair shapes compared on the same axes.
5. Your recommendation and the smallest first step.
6. The step plan with per-step gates, ripple, and cost.
7. The drafted follow-up implementation briefs.
8. Open questions for the director, each with your recommendation.
9. Stop-loss — where you stopped and why (empty if none).
