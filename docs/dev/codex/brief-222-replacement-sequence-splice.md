# Brief 222 — Splice replacement-produced Sequence into the held parent rebuild (USER-RULED)

Fix brief — unit-9 follow-up, ruled by the user 2026-07-13: the recorded
boundary from brief-216 is now a FIX. Source:
`docs/dev/usability-pass/kernel-b-findings.md` (hold-design appendix) +
brief-216's report/commit (606b96f) — read both FIRST. This brief REOPENS
exactly the fence brief-216 kept: the `ReplaceAll` traversal rebuild.

## Today (pinned by brief-216 as a known WL divergence)

```
Hold[g[a]] /. (a -> Sequence[1, 2])   → Hold[g[Sequence[1, 2]]]   (WL: Hold[g[1, 2]])
```

The traversal rebuild pushes each rewritten argument back as exactly ONE
argument slot (`definitions/application/replace_all.rs:80-91`,
`apply_replace_all_args` — `rewritten_args.push(rewritten)`), so a
replacement that turns an argument into a `Sequence` never splices. In
unheld parents evaluation hides this (it splices afterwards); under hold
the nested `Sequence` survives visibly.

Why the user ruled to fix: substitution is the ONLY way to splice an
argument sequence into held code (the capability is otherwise absent),
and a rewrite's output should already have its runtime shape.

## Required behavior

- **A rewrite-CHANGED argument whose result is a `Sequence[…]` call
  splices into the parent's rebuilt argument list**:
  `Hold[g[a]] /. a -> Sequence[1, 2]` → `Hold[g[1, 2]]`;
  `Hold[a] /. a -> Sequence[1, 2]` → `Hold[1, 2]` (the held head itself
  is a parent like any other); multi-position:
  `Hold[g[a, x, a]] /. a -> Sequence[1, 2]` → `Hold[g[1, 2, x, 1, 2]]`.
- **An UNCHANGED argument never splices**, even if it already is a
  literal `Sequence` call: `Hold[g[Sequence[1, 2]]] /. x -> y` (no match
  inside) must stay `Hold[g[Sequence[1, 2]]]` — held data untouched by
  the rule must come back byte-identical. Discriminate by "the rule
  application actually rewrote this argument" (the traversal already
  knows change vs no-change; do NOT compare shapes after the fact if a
  cheaper change-flag exists).
- **`SequenceHold`/`HoldAllComplete` parents do not receive the splice**:
  gate the splice on the immediate parent's `sequence_hold()` being
  false. `sh[a] /. a -> Sequence[1, 2]` → `sh[Sequence[1, 2]]` — this
  corner's WL surface is NOT confidently documented; rum pins the
  attribute-respecting rule for internal consistency with the
  brief-215/216 splice story. Document it in a test comment as
  pinned-not-WL and list it in the report.
- **Top level (no parent) unchanged**: `a /. a -> Sequence[1, 2]` keeps
  today's result (probe and pin it — there is nothing to splice into).
- **Delayed rules too**: `Hold[g[a]] /. (a :> Sequence[1, 2])` behaves
  the same (the RHS materializes to a Sequence and the parent rebuild
  splices it). Note the interaction with brief-216: a literal
  `Sequence` INSIDE a larger RHS (`a :> hc[Sequence[1, 2]]`) still does
  NOT splice at materialization — that contract is untouched; THIS brief
  is only about the traversal's parent-rebuild step when the WHOLE
  argument's rewrite result is a Sequence.
- **`ReplaceRepeated` and levelspec `Replace`** share the traversal (or
  wrap it) — probe both and make/keep them consistent with `ReplaceAll`;
  report which code they actually share.
- **Probe-and-record only (do NOT fix here)**: `ReplacePart[{1, 2, 3},
  2 -> Sequence[a, b]]` — path-based replacement with a Sequence value
  is a DIFFERENT seam; record current behavior verbatim as a boundary
  row for later triage.

## Method

1. Survey tests: brief-216's pinned-divergence test flips to the WL
   value (update its comment — it stops being a divergence); find any
   other assertion encoding the nested no-splice shape. List old→new.
2. Implement in the traversal rebuild only. The unchanged-argument fast
   path must stay zero-cost (the change-detection the traversal already
   has; no new comparisons on the no-match path).
3. New tests: the probe panel above (changed-splice, unchanged-literal
   preserved, sh/hc parents pinned, top level, delayed variant,
   multi-position, ReplaceRepeated/Replace consistency).

## Perf gate

`scripts/core-language-microbench.sh`, saved binaries, alternating
B C B C, profiling env OFF, idle machine (if a VM or other load appears,
STOP and report — never benchmark under load; the director will run the
final numbers if needed). Watch label: `replaceall-simple`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# the probe panel above, before/after
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs`
  and rewrite_data siblings ONLY if Replace/ReplaceRepeated consistency
  requires it (name every file + why)
- Tests: `crates/rum-runtime/src/runtime/tests/rule_rhs_materialization.rs`,
  the rewrite/holding test families (list each), one new test file allowed
- `crates/rum-stdlib/src/docgen.rs` (UPDATE the ReplaceAll note brief-216
  added — it currently documents the nested behavior; MERGE, keep the
  delayed-RHS half) + `docs/reference/builtins.md` (regenerated)

MUST NOT touch: `binding_substitution.rs` (216's landed seam),
`replace_part.rs`/`replace_at.rs` (probe-and-record only), the evaluator
core, the matcher, scoping, anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`.
- Baseline-verify EVERY preservation pin before asserting it (the
  brief-216 rev1 lesson); paste the baseline probe outputs.
- WL comparison from documented knowledge only; uncertain corners are
  pinned-and-documented, never guessed.
- Stop-loss: if the change-detection flag does not exist and the only
  discriminator is a structural comparison on every argument, STOP and
  report the cost problem instead of paying it on the hot path.

## Report format

1. The change: where the splice happens, how change-vs-unchanged is
   discriminated, why the no-match path pays nothing.
2. Test survey old→new (the flipped divergence pin called out).
3. Probe panel before/after verbatim; the pinned-not-WL rows (sh/hc
   parents, top level) and the ReplacePart boundary row.
4. A/B table + noise-floor verdict (replaceall-simple) + raw paths — or
   the load stop-loss note.
5. Verification outputs verbatim.
6. Deviations / open questions (empty if none).
