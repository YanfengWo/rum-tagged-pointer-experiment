# Brief 216 — Rule firing in held replacement: delayed-RHS insertion + substitution-time Sequence splice

Fix brief — unit-9 batch 2 (safety, L). Source:
`docs/dev/usability-pass/kernel-b-findings.md` (top-gaps #4, the
`RuleDelayed`/`HoldComplete` `gap` rows, Evidence E7, and the
hold-design appendix §"Held Arguments Through Matching And Replacement
Today") — read it FIRST. Both changes below move WL names TOWARD
documented WL semantics (the sanctioned class). They live in the
rewrite/substitution machinery and share one microbench A/B gate.

## Change (a) — delayed-RHS insertion is consumer-INDEPENDENT

Today (audit E7, gate-reproduced):

```
c = 0;
ra = Hold[a] /. (a :> (c = c + 1; b));          → Hold[CompoundExpression[Set[c, Plus[c, 1]], b]]   (correct: inserted raw)
rp = ReplacePart[Hold[a], {1} :> (c = c + 10; b)];   → Hold[b], c += 10   (WRONG: evaluated before insertion)
rt = ReplaceAt[Hold[a], a :> (c = c + 100; b), {1}]; → Hold[b], c += 100  (WRONG: same)
```

`ReplaceAll`'s applicator substitutes bindings into the raw RHS and
inserts WITHOUT evaluating; only the fully rebuilt RESULT is evaluated
(`definitions/application/replace_all.rs:93-166`,
`kernel_data/rewrite_data/replace/all.rs:49-54`). That is the WL
contract for `:>`. `ReplacePart` explicitly evaluates each delayed RHS
before insertion (`kernel_data/structure_data/replace/replace_part.rs:199-215`)
and `ReplaceAt` evaluates the matched replacement before writing the
path (`replace_at.rs:111-128`).

Required behavior:

- `ReplacePart`/`ReplaceAt` with `:>` insert the substituted-but-
  UNEVALUATED RHS; the rebuilt result then evaluates normally (exactly
  the `ReplaceAll` shape). Consequences to pin in tests:
  - held position: `ReplacePart[Hold[a], {1} :> (c += 10; b)]` →
    `Hold[CompoundExpression[…]]`, `c` UNCHANGED; same for `ReplaceAt`.
  - unheld position: `ReplacePart[{1, 2, 3}, 2 :> 10 + 10]` →
    `{1, 20, 3}` — same visible value as today, because result
    evaluation reaches the inserted RHS (assert this explicitly: the
    common case must not change).
  - `->` (immediate) rules unchanged: RHS evaluated at rule
    construction, inserted as a value (today's behavior everywhere).
- `ReplaceAll`'s behavior is the reference — do NOT touch its files;
  if the fix wants shared code, extract FROM the ReplacePart/ReplaceAt
  side only, or reuse an existing shared helper. Do not redesign the
  applicator.

## Change (b) — RHS materialization must not splice literal `Sequence`

Today (audit E7, gate-reproduced):

```
SetAttributes[hc, HoldAllComplete];
a /. (a :> hc[Sequence[1, 2]])   → hc[1, 2]              (WRONG)
hc[Sequence[1, 2]]               → hc[Sequence[1, 2]]     (direct call: correct)
```

Seam: `substitution/binding_substitution.rs:281-323`
(`substitute_rule_call_changed_with_arg_alphas`) splices ANY `Sequence`
argument — substituted OR literal — while rebuilding calls inside the
inserted RHS, before evaluation can consult the destination head's
`sequence_hold()`.

Required behavior:

- A LITERAL `Sequence[…]` argument in the RHS (no pattern binding
  involved) is inserted as-is; whether it splices is then evaluation's
  decision, which already respects `sequence_hold()` (post-brief-215:
  ordinary heads splice raw, `SequenceHold`/`HoldAllComplete` block).
  So `a /. (a :> hc[Sequence[1, 2]])` → `hc[Sequence[1, 2]]` and
  `a /. (a :> f[Sequence[1, 2]])` → `f[1, 2]` (via evaluation).
- **Pattern-variable Sequence bindings keep splicing structurally** —
  WL substitutes sequence bindings into the surrounding call regardless
  of hold attributes: `f[1, 2] /. f[x__] :> hc[x]` → `hc[1, 2]`, and
  inside held wrappers `Hold[g[a]] /. a -> Sequence[1, 2]` →
  `Hold[g[1, 2]]`. Pin both in tests; they must NOT change.
- Where the two interact and the WL surface is not confidently
  documented (e.g. `f[1, 2] /. f[x__] :> hc[Sequence[x]]`), pick the
  tightest self-consistent rule (binding-splice first, literal head
  preserved), DOCUMENT it in a test comment as rum's pinned contract,
  and list it in the report as pinned-not-WL (the brief-215
  discipline).

## Method

1. Survey tests first: every assertion encoding pre-insertion delayed
   evaluation in ReplacePart/ReplaceAt, or substitution-time literal
   splicing. List each hit with old→new in the report.
2. Implement (b) first (one seam), then (a) per consumer, each with
   tests green before the next.
3. New tests: the E7 panel as the pinned contract (final `c` values
   asserted!); the unheld-position value-equality assertions; the
   literal-vs-binding Sequence matrix above; `ReplaceAt` operator form
   + multi-position forms still green.

## Perf gate

`scripts/core-language-microbench.sh`, saved binaries, alternating
runs, profiling env OFF. Watch label: `replaceall-simple` (rule
application goes through binding substitution — the (b) seam is on its
path). The ReplacePart/ReplaceAt changes have no microbench label; the
workspace suite covers them.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes before/after: the E7 panel (with final c asserted), the
#   literal/binding Sequence matrix, the unheld-position value checks
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/` siblings
  (parsing/plan) ONLY if the insertion seam requires it — name every file
  touched and why
- `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md`
  (regenerated; Limits MERGE only) if a contract note is warranted
- Tests: the rewrite/holding test families across rum-runtime and
  rum-stdlib (list each file in the report), one new test file allowed.

MUST NOT touch: `replace_all.rs` / `rewrite_data/replace/all.rs` (the
reference behavior), the evaluator core (`eval_call_prepare.rs`,
`eval_core/`), the matcher (`pattern_matching/`), scoping, lowering,
anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new
  dependencies, no new `unsafe`.
- One A/B for the batch; profiling env OFF; never time under load.
- WL comparison from documented knowledge only; uncertain edges get
  pinned-and-documented, never guessed.
- Stop-loss: if (a) cannot be done without touching the ReplaceAll
  applicator, or (b) breaks sequence-variable splicing anywhere, STOP
  and report instead of widening scope.

## Report format

1. Per-change summary: seams, mechanism, what shares code with what.
2. Test survey (old-behavior assertions), old→new.
3. Probe panel before/after verbatim, final `c` values called out; the
   pinned-not-WL rows if any.
4. A/B table + noise-floor verdict (replaceall-simple called out) +
   raw paths.
5. Verification outputs verbatim.
6. Deviations / open questions (empty if none).
