# brief-078 — gensym-GC coverage: the untested escape/root-enumeration branches

## Goal

`crates/rum-runtime/src/runtime/scoping/gensym_gc.rs` (230 lines, ZERO profiling — all
real memory-line logic from briefs 049/050) is 51% region-covered (78 missed regions).
The existing `tests/scoping_gensym_gc.rs` already covers the CORE scenarios (dead
reclaimed, escape-via-global kept, extra-roots, transitive marking, Unique-exclusion).
The residual gap is the LESS-common branches in root / definition enumeration — the
different DEFINITION KINDS a gensym can escape into, and the outcome/edge paths. Fill
them: a gensym escaping into ANY definition kind must survive GC (soundness), and a truly
dead one must be reclaimed. TEST-ONLY: NO product-code change. If a test reveals the GC
actually reclaims a live gensym (a soundness bug) or leaks a dead one, REPORT it (leave
the failing test) — do not weaken it.

## What's under-tested (target these branches)

Read `gensym_gc.rs`: `collect_and_sweep_dead_gensyms`, `for_each_root_expr`,
`for_each_definition_expr`, `seed_live_gensyms_from_expr`, `gensym_gc_extra_roots_for_outcome`,
`gensym_gc_threshold`. The existing tests exercise own-value/down-value escapes; the
uncovered branches are likely the OTHER definition kinds + edges:
1. **Escape via each definition kind keeps the gensym live** — a Module gensym that escapes
   into a global symbol's DOWN-VALUE, UP-VALUE, SUB-VALUE, OWN-VALUE, ATTRIBUTES, and
   MESSAGES respectively must NOT be reclaimed after `collect_and_sweep_dead_gensyms`
   (each drives a distinct arm of `for_each_root_expr` / `for_each_definition_expr`).
2. **`gensym_gc_extra_roots_for_outcome`** — the outcome→extra-roots path (a gensym only
   reachable through the eval outcome/value is kept when passed as an extra root; dropped
   otherwise). Cover the outcome shapes it handles.
3. **Transitive escape through a chain of definition kinds** (gensym A in a down-value that
   references gensym B in an up-value → both kept).
4. **Edges:** empty candidate set → early return (no-op); a gensym present in a
   `module_activation` (still-active scope) is kept even if not otherwise rooted; threshold
   (`gensym_gc_threshold` / `RUM_GENSYM_GC_THRESHOLD`) parsing (valid/invalid/default).
5. **Dead in every kind:** the negative of (1) — after the rooting definition is cleared,
   the gensym IS reclaimed.

Identify which arms the existing tests don't hit (each definition-kind branch of
`for_each_root_expr`/`for_each_definition_expr`) and add a focused escape+reclaim pair per
kind. Assert via the existing helper `retained_candidate_definition_count` (or equivalent):
count before/after sweep, and that the specific gensym's definition survives/is gone.

## rum / test pitfalls (read twice)

1. **Soundness is the contract:** a live (escaped) gensym must NEVER be swept; a dead one
   must be. Assert both directions. A test that only checks "sweep runs" is not a test.
2. **Call `collect_and_sweep_dead_gensyms` directly** (it is `pub(in crate::runtime)`, so
   crate-internal tests can) + inspect state — matching the existing `scoping_gensym_gc.rs`
   style. Set up escapes via WL evaluation (define a Module producing a gensym, then write
   it into a global's up/sub/down-value/attribute/message).
3. **Cover each DEFINITION KIND as a distinct arm** — that's where the 78 missed regions
   are (own vs down vs up vs sub vs attributes vs messages enumeration).
4. **Report soundness bugs as findings** (leave the failing test); do not weaken.
5. Match `tests/scoping_gensym_gc.rs` style; extend that file (or add cases to it).

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs` (extend) + its
  mod wiring if needed. TEST-ONLY. NO product-code change under non-test `src`.
- MUST NOT: edit any `mma-hol` file; `Expr`/serde; no new `unsafe`; no `git commit`.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime scoping_gensym_gc      # the GC tests green (paste count)
cargo test -p rum-runtime                        # no regression (paste the total)
cargo llvm-cov -p rum-runtime --summary-only 2>&1 | grep scoping/gensym_gc.rs
# gensym_gc.rs region coverage should be well above 51% now (paste before 51% -> after)
```

Paste: the GC-test count (before → after), the `-p rum-runtime` total, the gensym_gc.rs
coverage before/after, and — if a test reveals a GC soundness bug — the failure + analysis.

## Hard rules

- No `git commit`/branch/push. Never edit any `mma-hol` file. Test-only. No new `unsafe`.
- Real soundness assertions (escaped→kept AND dead→reclaimed), per definition kind.
- Report soundness bugs as findings; do not weaken.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the new GC test cases per definition kind / edge; confirm no product-code
   change.
2. Verification — GC-test count, `-p rum-runtime` total, gensym_gc.rs coverage before/after
   — verbatim.
3. Findings — any GC soundness bug (live reclaimed / dead leaked), or "GC keeps every
   escape kind live and reclaims every dead gensym on all covered cases."
4. Open questions — any definition-kind arm you couldn't reach or an ambiguous escape case.
