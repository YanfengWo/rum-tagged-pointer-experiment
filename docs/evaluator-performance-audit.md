# Evaluator Performance Audit: `mma-hol` Bool Load

Maintainer note: this is an external performance review captured on
2026-06-10 for the `rum` state immediately after commit `fddf4fd` (`Optimize
Bool proof-load evaluation`) and sibling `mma-hol` HEAD `96cf163`. It is kept
as a historical audit artifact and as a baseline for the follow-up evaluator
optimization series; later timings may differ after the fixes described here.

Date: 2026-06-10. Measured against sibling `mma-hol` HEAD `96cf163` (clean
working tree) with a release build of `rum-cli` on the local machine.

This document records a measured diagnosis of why dependency-preloaded
`Bool.wl` loading is roughly 25x slower in `rum` than in WolframKernel, plus a
prioritized remediation plan. It extends the open question at the end of the
`2026-06-10 Bool Block Profiling` entry in `PROGRESS.md`: the suspicion there
("owned-expression cloning, recursive pure-function traversal, repeated
derived-rule workloads") is confirmed by sampling, and this audit narrows it to
specific code paths with measured weights.

## TL;DR

The bottleneck is not any single slow builtin or algorithm. It is the
expression representation plus the clone-everywhere evaluator style built on
top of it:

- roughly three quarters of all CPU time goes to allocating, copying, and
  freeing expression trees (`malloc`/`free`/`Expr::clone`/`drop`), plus deep
  structural `memcmp`
- the single largest direct producer of clones is the top-level fixed-point
  loop in `Session::evaluate` itself, which deep-clones the current expression
  twice per pass and deep-compares it once per pass, at every recursion level
- a further ~5% is a linear string scan over the entire builtin catalog that
  runs on essentially every node visit

WolframKernel loads the same file in ~2.4s because its expressions are
refcounted shared DAG nodes with interned symbols and an "already evaluated"
cache bit, so re-scanning a normal-form subterm is O(1) and "copying" a term
is a pointer operation. `rum` currently pays full-tree cost for each of those
operations, and HOL proof terms are exactly the workload that maximizes that
cost: large, deep, repeatedly rebuilt by derived rules.

## Measurement Setup

Probe (preload the dependency chain, then load full `Bool.wl`):

```wolfram
Get["/path/to/mma-hol/ErrorUtil.wl"];
Get["/path/to/mma-hol/Types.wl"];
Get["/path/to/mma-hol/Terms.wl"];
Get["/path/to/mma-hol/Kernel.wl"];
Get["/path/to/mma-hol/Bootstrap.wl"];
Get["/path/to/mma-hol/Equal.wl"];
Get["/path/to/mma-hol/Bool.wl"];
HOL`Bool`TRUTH
```

Run and profile:

```bash
CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release -p rum-cli
target/release/rum-cli --file probe.wl &
sample $(pgrep -x rum-cli) 30 -file sample.txt   # macOS sampling profiler
```

Observed: 73s wall / 61s user for the full probe (consistent with the ~60s
release figure already in `PROGRESS.md`). WolframKernel reference on the same
machine: ~2.4s.

## Flat Profile (top of stack, 16,466 samples over 30s)

| Frames | Share | Meaning |
| --- | --- | --- |
| `nanov2_malloc` / `nanov2_free` / other allocator frames | ~55% | raw allocation and free traffic |
| `Expr::clone` + `String::clone` + `drop_in_place<Expr>` | ~15% | deep tree copies and their destruction |
| `_platform_memcmp` | ~7% | deep structural equality + catalog string compares |
| `builtin_dispatch_spec_for_symbol` + `default_system_attribute_set` | ~5% | linear scan of the builtin catalog |
| `expr_is_evaluation_stable` (self) | ~1% | full-tree stability pre-scan (also induces the catalog lookups above) |

Attributing `Expr::clone` to its direct callers across all sampled stacks, the
ranking is:

1. `Session::evaluate` (fixed-point loop) — by a wide margin
2. `run_call_dispatch_stage`
3. `resume_prepared_call_arguments` / `plan_call_argument_preparation`
4. `substitute_rule_bindings`
5. `fast_match_definition_pattern`

## Root Causes

### 1. `Expr` is a fully owned deep tree (the multiplier for everything else)

`crates/rum-core/src/expr.rs` — `Expr::Call { head: Box<Expr>, args: Vec<Expr> }`.

- `clone()` is a full recursive copy: one heap allocation per `Call` node plus
  per-atom payloads.
- `Symbol` holds `context: String` and `name: String` with no interning, so
  cloning a symbol is two heap allocations and comparing/hashing one is a
  string operation. The `memcmp` profile share is largely this.
- `PartialEq` and `Hash` are O(tree size).

Every cost below is this representation multiplied by some traversal.

### 2. The fixed-point loop clones and compares the whole tree per pass

`crates/rum-runtime/src/runtime/resolution.rs`, `Session::evaluate` (and its
twin `resume_fixed_point_from`):

- `current.clone()` is fed to `evaluate_once` while another full
  `current.clone()` (plus `fixed_point_seen.clone()`) is moved into the
  suspend/resume closure **unconditionally**, even though `InputString`
  suspension essentially never fires in this workload
- `next == current` is a full deep structural comparison per pass

Because `evaluate` re-enters recursively for every argument at every level,
these per-pass costs multiply along term depth. This is the top producer of
clone traffic in the profile.

Note the asymmetry: `evaluate_call` in
`crates/rum-runtime/src/runtime/eval_core.rs` already does this correctly —
it only clones the resume state inside the `Err(EvalError::InputRequired)`
branch. The fixed-point loop has not received the same treatment.

### 3. Full-tree stability scan on every `evaluate_inner` entry

`crates/rum-runtime/src/runtime/eval_core.rs`,
`expr_is_evaluation_stable`: a recursive walk over the whole subtree that, per
node, consults attribute sets, the builtin dispatch catalog, and the
definitions map (hash lookups keyed by two-`String` `Symbol`s). It runs at
every recursion level, so inner subterms are re-scanned once per enclosing
level.

### 4. Builtin metadata lookup is a linear string scan

`crates/rum-runtime/src/runtime/builtin_catalog.rs`,
`default_builtin_dispatch_spec`:

```rust
core_builtin_specs().iter().find(|spec| spec.name == name)
```

This scans the entire catalog (hundreds of specs across 11 category slices)
with string comparison. It is reached from `default_system_attribute_set` and
`builtin_dispatch_spec_for_symbol`, which run on essentially every call
dispatch and every stability-scan node. Direct cost ~5% of samples, plus a
substantial share of the `memcmp` bucket.

### 5. DownValue dispatch redoes all preparation per call

`crates/rum-runtime/src/runtime/definition_application.rs`,
`try_apply_down_values`:

- `self.definitions.get(symbol).cloned()` deep-clones the **entire**
  `SymbolDefinition` — every rule's lhs and rhs — on every dispatch of every
  user-defined function
- `Expr::call(head.clone(), args.to_vec())` rebuilds the candidate call (one
  more full copy of all argument trees), then
  `normalize_definition_candidate_for_matching` may rebuild it again
- `DownValue { lhs, rhs, delayed }` (`runtime/model.rs`) stores **no compiled
  matcher**, so `compile_matcher_for_expr` (which itself runs
  `normalize_match_expr`, another full pattern-tree rebuild with attribute
  lookups) is re-run per rule per call whenever the fast matcher declines
- on the slow path, `normalized_candidate.clone()` happens once more per rule
  attempted

For HOL derived rules (`MP`, `GEN`, `SPEC`, `TRANS`, `DISCH`, `BETACONV`, ...)
the arguments are the large theorem terms and the call counts are in the tens
of thousands, so this is O(rules × term size) allocation per call.

### 6. Matching and substitution are owned end to end

- `match_compiled_owned` (`runtime/pattern_matching/core.rs`) snapshots
  `bindings.clone()` at **every matcher node**; the `BTreeMap<Symbol, Expr>`
  values are whole bound subterms, so one pattern match clones bound HOL terms
  once per pattern node
- `substitute_rule_bindings` (`runtime/binding_substitution.rs`) and
  `substitute_lexical_bindings` (`runtime/lexical_substitution.rs`) deep-clone
  the bound value at every occurrence and rebuild every subtree even when no
  substitution occurred inside it (no occurs-check short-circuit)

### 7. The suspend/resume style forces owned `'static` state everywhere

The continuation-passing structure (`drive_internal_value`,
`suspend_with_frame`, `move` closures) requires owned, `'static` captures, so
values are cloned ahead of time "in case we suspend" at many boundaries. The
2026-06-10 optimization pass already delayed some of these; the fixed-point
loop (cause 2) is the largest remaining instance of the eager pattern.

This also explains the debug-vs-release gap (231s vs 60s): clone-dominated
code is disproportionately slow without inlining and with debug-mode
`Vec`/`String` copy paths. Fixing the representation will compress that gap
too.

## Why WolframKernel Does The Same Work In 2.4s

For each mechanism above, the Wolfram kernel has the inverse design:

- expressions are refcounted, structure-shared DAG nodes: "copy" is a pointer
  + refcount bump, structural sharing is pervasive
- symbols are interned: comparison is pointer equality, hashing is trivial
- expressions carry an "already evaluated" cache (tied to definition-change
  epochs), so the outer evaluate-to-fixed-point re-pass over a normal-form
  term is O(1), not a re-traversal
- definition dispatch is hash-keyed on the head with precompiled patterns

`rum` currently pays full-tree cost at each of these points, and the costs
compound multiplicatively on deep HOL terms.

## Recommendations

### Quick wins (days; expected combined recovery on the order of 20–40%)

1. **Hash the builtin catalog.** Replace the linear `find` in
   `default_builtin_dispatch_spec` with a
   `OnceLock<HashMap<&'static str, BuiltinSpec>>` (or `phf`). Mechanical
   change, immediately removes ~5%+ and part of the `memcmp` bucket.
2. **Delay resume-state cloning in the fixed-point loop.** Restructure
   `Session::evaluate` / `resume_fixed_point_from` so the resume frame
   (`current_for_resume`, `seen_for_resume`) is only constructed in the
   `Err(EvalError::InputRequired)` path, mirroring the pattern already used by
   `evaluate_call` in `eval_core.rs`. Saves one full-tree clone per pass at
   every recursion level.
3. **Cache compiled matchers on stored definitions.** Add a compiled-matcher
   field to `DownValue` / `UpValue` / `SubValue` (populated at definition
   insertion, which is the natural cache point already flagged in `TODO.md`),
   and stop re-running `compile_matcher_for_expr` + `normalize_match_expr` per
   rule per call.
4. **Stop cloning whole `SymbolDefinition`s in dispatch.** In
   `try_apply_down_values` (and the up/sub variants), iterate by index over a
   borrowed or `Rc`-wrapped rule list instead of `definitions.get(..).cloned()`,
   and build the candidate call once instead of head-clone + `args.to_vec()` +
   re-normalization.
5. **Replace per-node binding snapshots with an undo trail.** In
   `match_compiled_owned`, record which keys a sub-match inserted and roll
   back on failure, instead of cloning the entire `BTreeMap` (with deep-cloned
   bound terms) at every matcher node.

### Structural fixes (the path to the same order of magnitude as Wolfram)

6. **Make `Expr` a shared, refcounted structure.** `Rc<ExprNode>` (or an arena
   with node ids). `clone()` becomes a refcount bump; this single change
   removes the majority of the allocator traffic that currently dominates the
   profile, makes the remaining eager clones cheap, and allows a pointer-
   equality fast path in `PartialEq`. This is preferable to incrementally
   converting individual paths to borrowed traversal: borrowed paths fight the
   `'static` resume closures point by point for fragmented gains, while the
   representation change makes every clone site cheap at once.
7. **Intern symbols.** A global symbol table mapping
   (context, name) → integer id; `Symbol` becomes `Copy`. Comparison and
   hashing become integer operations, and the definitions/attributes maps get
   cheap keys.
8. **Replace stability re-scans with an evaluated-epoch cache.** Maintain a
   definitions/attributes version counter, bumped by `Set`/`SetDelayed`/
   attribute changes; cache on shared nodes "was in normal form at epoch N".
   This subsumes `expr_is_evaluation_stable` and makes the fixed-point
   re-pass O(1) for unchanged subterms — the same mechanism that makes the
   Wolfram kernel fast on this workload. Requires (6) to have a place to hang
   the cache bit.

Recommended order: 1–5 first (independent, low risk, measurable one by one
with the probe above), then 6 → 7 → 8 as one deliberate representation
migration. Re-run the timed probe plus a 30s `sample` after each step;
the flat-profile shares above are the baseline to compare against.

## Non-Issues (measured, no action needed now)

- `Atom::Integer` as pure `BigInt` (open question in `TODO.md`): BigInt does
  not appear among the hot frames; the small-int hybrid is not currently
  justified.
- `Part` access: already ruled out by earlier probes (`PROGRESS.md`), and the
  profile agrees — `eval_part` is far down the clone-caller list.
- Parser/loader: absent from the hot stacks; the cost is in evaluation.

## Side Finding (correctness of tooling, not performance)

`rum-cli` swallows `Print` output: `--eval 'Print["hello"]; 1+1'` prints only
`2`. Print output appears to be captured into the session rather than rendered
by `print_outcome` in `crates/rum-cli/src/main.rs`. This is harmless for
benchmarks but actively misleading when using `Print` markers to bisect slow
package loads — worth fixing before more load-time triage.
