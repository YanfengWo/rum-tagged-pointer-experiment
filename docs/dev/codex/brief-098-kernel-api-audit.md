# Brief 098 — DIAGNOSTIC: audit what the builtin families actually import from the kernel

## Goal

Stage-2 input for the crate-rebalance line (`docs/crate-rebalance-plan.md`). The
builtin family modules compile against the runtime's shared internal namespace
(`use super::*` over the giant glob in `runtime/mod.rs`; 528
`pub(in crate::runtime)` symbols). Before we can define the explicit kernel API
they will import once they move to their own crate, we need the measured
dependency sets. Deliverable is EVIDENCE ONLY — tables, no source change left in
the tree (`git status` must be clean at the end).

## Context pointers

- Read `AGENTS.md` and `docs/crate-rebalance-plan.md` (Stage 2) first.
- Brief-097 landed: dispatch is open (`BuiltinFn` fn pointers, catalog references
  `Session::eval_*` / `builtin_adapters::*` directly).
- The builtin family modules to audit (all under `crates/rum-runtime/src/runtime/`):
  `arithmetic`, `associations/`, `calculus`, `combinatorics`, `control_flow`,
  `functional_ops/`, `integer_digits`, `logic/`, `number_theory`, `polynomials/`,
  `predicates/`, `strings/`, `structure_ops/`, `rewrite_builtins`.
  If one of these turns out not to be a builtin family (kernel machinery), note
  that in the report and still audit it.
- Every file in these modules starts with `use super::*;` — that single line is
  the coupling to measure.
- `crates/rum-runtime/src/runtime/mod.rs` — the glob (`use self::{…::*}`) plus the
  std/rum_core/rum_syntax imports that `super::*` also re-exports.

## Method (compile-error-driven; per family)

For each family module F, one at a time:

1. In EVERY `.rs` file of F, comment out the `use super::*;` line
   (`// use super::*; // brief-098 audit`).
2. `cargo check -p rum-runtime 2>&1` and collect every unresolved name from the
   `cannot find …` / `use of undeclared …` / method-resolution errors
   (`python3` parsing is fine; capture the raw log to
   `/tmp/cdx_brief_098_raw_<family>.log` FIRST, then parse — never inline-grep
   and lose it).
3. Restore the files EXACTLY (git checkout of the touched files), verify
   `git status` clean, then move to the next family.

Note the method's known blind spot and cover it separately: inherent-method calls
(`self.evaluate(…)`, `session.push_message(…)`) never need imports, so compile
errors will not list them. Axis 2 is a grep/script pass: for each family, extract
the set of `self.<method>(`/`session.<method>(` calls whose method is DEFINED
outside the family modules, with call counts. (Method-definition locations via
`grep -rn "fn <name>" crates/rum-runtime/src/runtime/` — heuristic precision is
acceptable for this axis; say so where ambiguous.)

## Deliverable (write the full tables to `/tmp/cdx_brief_098_report.md`, paste key rows verbatim in your final message)

1. **Per-family table**: family → number of distinct unresolved free items →
   breakdown by origin class: (a) `rum_core`/`rum_syntax`/std re-exports (trivial:
   the moved crate imports them directly), (b) runtime-internal items, grouped by
   DEFINING module (which kernel module owns each item), (c) items defined in
   ANOTHER family (family→family coupling — important: these block independent
   moves; list them explicitly).
2. **Global promotion-candidate list**: the union of runtime-internal items, one
   row per item: name, kind (fn/type/const/macro), defining module, how many
   families use it. Sort by family-fan-out descending.
3. **The split**: how many items are used by ≥3 families (clear kernel-API
   prelude candidates) vs exactly 1 family (candidates to move WITH that family
   instead of promoting); how many of the 528 `pub(in crate::runtime)` symbols no
   family uses at all (they stay kernel-private).
4. **Axis-2 table**: kernel `Session` methods called from family code, with
   per-family call counts — the future kernel-API *method* surface (e.g.
   `evaluate`, `suspend_with_frame`, `push_message`, …).
5. Clean-tree proof: final `git status --short` output (must be empty).

## rum / Rust pitfalls

1. Persist every raw compile log to a file before parsing (measurement hygiene —
   re-runs cost minutes each).
2. One family at a time; restoring between families keeps error sets attributable.
3. `cargo check`, never `cargo build`/`test`, for the audit loop (speed).
4. Unresolved-name errors can cascade (a missing type makes later errors noisier);
   the FIRST error per name is the signal — dedupe by identifier.
5. Some names resolve via the prelude even without `super::*` (std prelude,
   crate-root `use`) — absence from the error list for such names is fine; the
   audit measures the `super::*` dependency, not total imports.
6. Stop-loss: if a family will not produce a parseable error set after two
   attempts, record what happened and move on — a 13/14-family table with one
   annotated gap is still the deliverable.

## Verification (MANDATORY)

Diagnostic brief: the evidence is the deliverable. Paste verbatim in the final
message: the per-family counts table, the top-20 rows of the promotion-candidate
list, the ≥3-families / 1-family / unused split totals, the top-15 axis-2 methods,
and the empty `git status --short`. Confirm `cargo check -p rum-runtime` is green
after the last restore.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- The ONLY source edits allowed are the temporary `use super::*;` comment-outs,
  and every one of them MUST be restored before you finish — the tree ends clean.
  No other file may be touched; nothing under any `mma-hol` clone, ever.
- Full machine access, but touch NOTHING outside this repo, no network. Only
  `cargo check`, file edits per the method, `grep`/`rg`, and `python3`.
- No new `unsafe` (trivially satisfied — no lasting source change).
- Probe budget: ~14 `cargo check` runs for axis 1 + one final green check. Do not
  loop beyond that; a suspicious family gets ONE re-run, then report as-is.
- You MUST paste real verbatim output; if a step did not produce evidence, say so
  explicitly — never fake it.

## Report format (your final message)

1. What was audited — the family list, plus "diagnostic only, no source change;
   tree clean" with the `git status` proof.
2. The tables (per-family, promotion candidates top rows, the split, axis-2).
3. The finding — 2-3 sentences: how big is the real kernel API, where the
   family→family couplings are, and which family looks cheapest to move first.
4. Stop-loss / open questions (empty if none).
