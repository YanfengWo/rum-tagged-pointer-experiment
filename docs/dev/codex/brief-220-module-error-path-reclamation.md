# Brief 220 — Module error-path gensym reclamation (USER-RULED; unit-9 decision fix)

Fix brief — the unit-9 audit's trust-boundary finding, ruled by the user
2026-07-13: **eager reclamation on error paths** (= WL semantics). Source:
`docs/dev/usability-pass/kernel-b-findings.md` (the Module `gap` row + E3)
and the tracker's unit-9 triage section. Read both FIRST.

## The defect (gate-reproduced)

```
{Names["Global`auditLocal$*"], Catch[Module[{auditLocal = 1}, Throw[fail]]],
 Names["Global`auditLocal$*"], Global`auditLocal$1}
→ List[List[], fail, List["Global`auditLocal$1"], 1]
```

After a caught non-local exit, the Module local's definition remains
ENUMERABLE and READABLE until the gensym GC threshold sweep (default
200,000 candidates). WL removes unreferenced locals promptly on every exit
path; escaped locals survive while referenced. rum's Ok arm already
implements exactly that shape — the Err arm just skips it.

## The seam (verified at diagnosis)

`finish_module_scope`
(`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:207-284`):

- The retained/reclaim computation — roots scan (`expr_contains_symbol`) +
  transitive closure through definitions (`definition_references_symbol`) +
  `clear_symbol_state` — runs ONLY under `if let Ok(result)`.
- On `Err`, everything becomes a GC candidate and waits for the sweep.
- `gensym_gc_extra_roots_for_outcome(&outcome)`
  (`scoping/gensym_gc.rs`) ALREADY extracts reference roots from an `Err`
  outcome — the threshold sweep uses it today.

## Required change

Run the SAME retained/reclaim computation on the `Err` arm, with the
reference roots taken from `gensym_gc_extra_roots_for_outcome(&outcome)`
instead of the Ok result. Factor the computation over a list of root
expressions so the Ok arm passes `[result]` and the Err arm passes the
extra roots; the transitive closure and the reclaim step are shared,
byte-identical logic.

Contract after the fix:

- `Catch[Module[{v = 1}, Throw[fail]]]` → `fail`; `Names["Global`v$*"]` →
  `{}`; the gensym's definition is gone. Same for `Abort[]`
  (`CheckAbort[Module[{w = 1}, Abort[]], caught]`) and for an `EvalError`
  propagating out of the body.
- **Escape through the payload survives**: `Catch[Module[{x = 1},
  Throw[Hold[x]]]]` → `Hold[x$n]` whose `ReleaseHold` still evaluates to
  `1` — the gensym is in the extra roots, so it is retained (and remains a
  GC candidate for the backstop sweep, as today).
- **The `escaped` guard is UNCHANGED**: if `activation.escaped` or the
  side-table stamp moved (definitions were created during the body), no
  eager reclamation happens on EITHER arm — conservative, exactly as
  today. A `Throw` after `g[y_] := x + y` therefore still defers to the
  GC; assert that in a test and mark it intended.
- **Nested Modules**: an inner `Throw` propagating through two activations
  reclaims each level's unreferenced locals as the frames unwind. Test
  `Catch[Module[{a = 1}, Module[{b = 2}, Throw[fail]]]]` → both `a$n` and
  `b$n` gone.
- Ok-path behavior, the renaming scheme, activation bookkeeping,
  `gensym_gc` threshold and sweep logic: **byte-identical**. This brief
  adds reclamation to the Err arm and nothing else.

## Hard boundaries

- Module gensym is a TRUST BOUNDARY for the downstream mma-hol application
  (LCF-style encapsulation). Do not touch the renaming scheme, `Unique`,
  `$ModuleNumber`-adjacent behavior, or Block/With. The acceptance-suite
  gate against mma-hol is run by the director at the gate — not your job,
  but it is the reason scope discipline here is absolute.
- Zero new cost on the Ok path (the shared helper must not add allocation
  or extra scanning to the Ok arm). The Err arm gains the same bounded scan
  the Ok arm already does; error paths are not hot paths.
- No new `unsafe`, no dependencies, no attribute/capability changes.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace          # full gate

# probes, before AND after (batch in one --eval list where possible):
# 1. the defect probe above (after: Names -> {}, symbol undefined, no message)
# 2. Abort variant via CheckAbort
# 3. payload escape: Catch[Module[{x=1},Throw[Hold[x]]]] then ReleaseHold -> 1
# 4. nested-module double reclaim
# 5. escaped-guard conservatism (definition created in body -> deferred to GC)
# 6. Ok-path regression: Module[{x}, x] -> x$n; Module[{x=2}, x+1] -> 3;
#    a closure escaping via the RESULT retains its local (existing tests)
```

Also run the microbench A/B (`scripts/core-language-microbench.sh`,
rum-vs-rum, saved binaries, alternating, profiling env OFF): the
`module-local` / `block-local` / `with-local` labels are the watch list —
the Ok arm must not move beyond the control-label noise floor.

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/scoping/lexical/module.rs`
- `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs` (only if the
  root-extraction helper needs a signature/visibility touch — keep the
  sweep logic itself unchanged)
- Tests: `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs`,
  `crates/rum-stdlib/src/tests/holding_and_control_part3.rs`,
  `crates/rum-stdlib/src/tests/incidental_holding_and_control_part3.rs`,
  plus one new test file if needed (name it in the report).

MUST NOT touch: Block/With implementations, `unique.rs`, the evaluator
core, the matcher, catalog/reference (no behavior surface changes — the
reference's Module entry stays accurate), anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies.
- Never claim a behavior you did not run; paste probe transcripts verbatim.
- WL comparison from documented knowledge only; no external engines.
- Stop-loss: if the Err-arm reclaim cannot reuse the Ok-arm computation
  without changing Ok-arm behavior, STOP and report the coupling instead of
  redesigning the activation bookkeeping.

## Report format

1. The change: where the shared helper lives, what the Err arm passes as
   roots, and why the Ok arm is byte-identical.
2. Probes 1-6 before/after, verbatim.
3. A/B per-label table + noise-floor verdict (module/block/with labels
   called out) + raw-file paths.
4. Verification outputs verbatim (fmt, workspace totals, git status).
5. Deviations / open questions (empty if none).
