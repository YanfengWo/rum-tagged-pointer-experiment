# brief-050: Memory line Phase 1b — root-set mark-sweep GC for escaped-then-dead Module gensyms

## Goal

Phase 1 (brief-049) reclaims only "leaf" Module gensyms; it is too conservative
for the real theorem-prover workload, which is **deeply nested / recursive**
Module usage. Measured this session (rum-cli `--eval`, `MemoryInUse` delta =
definition proxy):

| workload | activations | `MemoryInUse` delta (definitions) |
| --- | --- | --- |
| flat leaf `Do[Module[{x}, x=i^2; x], 20000]` | 20 000 | 498 B (reclaimed) |
| recursive `f[n_]:=Module[{r}, If[n<=0,0,r=f[n-1]+1; r]]; Do[f[50], 2000]` | ~100 000 | **28,999,745 B** |
| same `Do[f[50], 10000]` | ~500 000 | **145,423,745 B** (linear, ~291 B/activation) |

Cause: Phase 1's escape barrier marks an OUTER activation escaped whenever an
INNER activation writes its own gensym (a non-own write from the outer's view), so
nested/recursive modules reclaim ~nothing → definitions accumulate. This is why the
cold `run_all.wls` still ballooned to 41 GB (watchdog-killed at the 40 GB limit) —
it dies in the cold tower load, which is all nested Module/tactic code. (The peak
RSS marginal cost is ~1.3 KB/activation; ~291 B is the proxy's definition estimate,
the rest is the real allocator cost of the SAME definitions — HashMap entry +
`SymbolDefinition` + Arc'd ownvalue + BigInt. So reclaiming the definition frees the
bulk of RSS; the interner `&'static` name residual is ~100 B/activation = Phase 2,
a separate smaller lever. The mark-sweep here is expected to bring the cold run from
41 GB-and-climbing to a few GB, unblocking it.)

Implement a **threshold-triggered root-set mark-sweep**: let escaped gensym
definitions accumulate, and when the candidate count crosses a threshold, mark all
gensyms reachable from the live root set and sweep (reclaim definitions of) the
unreachable ones. This is SOUND BY CONSTRUCTION (real reachability, not an
escape heuristic) and handles nesting/recursion correctly. When done: the
nested/recursive workloads above stay BOUNDED (memory plateaus instead of growing
linearly), `cargo test --workspace` stays green, and every soundness case still
returns the WK-correct value.

## Context pointers

- Read `AGENTS.md`, `docs/memory-management-plan.md` (Phase 1's "Optional
  completeness follow-up: a root-set mark-sweep" — THIS brief), and the
  `2026-06-29` PROGRESS entries (brief-048 instruments `9140fd4`, brief-049 leaf GC
  `57603f8`).
- This builds DIRECTLY on brief-049's machinery — reuse it, do not replace it:
  - `crates/rum-runtime/src/runtime/scoping/lexical.rs`: `eval_module` pushes a
    `ModuleActivation` (gensyms + `escaped` + `side_table_stamp`);
    `finish_module_scope` (the suspend-safe finalizer) computes the leaf reclaim set
    and calls `clear_symbol_state`. You hook the registry + the sweep trigger here.
  - `crates/rum-runtime/src/runtime/model.rs`: `Session` (`module_activations`,
    the brief-049 fields); `ModuleActivation`.
  - `crates/rum-runtime/src/runtime/definition_clear.rs`: `clear_symbol_state(sym,
    true,true,true,true,true)` — the reclaim primitive (does NOT touch
    `removed_symbols`; keep using it, NOT `remove_symbol_state`).
  - `crates/rum-runtime/src/runtime/definition_analysis_helpers.rs`:
    `expr_contains_symbol`, `definition_references_symbol` (brief-049).
  - `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`:
    `current_memory_proxy_bytes` — **the authoritative enumeration of every
    Expr-holding Session field.** Your root walk MUST mirror its field coverage
    (this is the completeness anchor — see M2).
- rum gensyms are `Symbol`s named `<prefix>$<id>` created by `allocate_fresh_symbol`
  (Module/Unique). Only Module-created gensyms are GC candidates here (a user/
  `Unique[]` `$`-symbol is re-referenceable by name and must persist even if
  currently unreferenced; a Module gensym is unnameable in source, so reclaiming it
  when unreachable is sound). The registry (M1) is populated ONLY from `eval_module`,
  which is how we distinguish them.

## The mechanism (precise — implement exactly this)

### M1. Candidate registry (which gensyms the sweep may consider)

`model.rs` add to `Session`:
```rust
pub(super) gc_candidate_gensyms: std::collections::HashSet<Symbol>,
pub(super) in_gensym_sweep: bool,
```
`session_state.rs` `Default`: `gc_candidate_gensyms: HashSet::new(), in_gensym_sweep: false,`.

Populate it ONLY with gensyms that a module exit RETAINED (escaped or
result-referenced) — i.e., the leak candidates. In `finish_module_scope`
(brief-049), after computing `reclaim` and before/at the truncate:
- the gensyms being reclaimed NOW (the leaf path): also remove them from
  `gc_candidate_gensyms` (in case a prior sweep added them — usually absent, a
  `HashSet::remove` is a cheap no-op).
- the gensyms RETAINED (this activation's `gensyms` minus `reclaim`): INSERT into
  `gc_candidate_gensyms`. These are the escaped/nested/result-retained ones that
  may become dead later and that the sweep will re-check.

Do NOT touch the registry on the flat-leaf fast path beyond the cheap remove —
leaf modules reclaim immediately (brief-049) and add nothing. The registry only
grows with genuinely-retained gensyms.

**The registry is ADVISORY, not authoritative.** Correctness comes from the MARK
(M2) — the registry only bounds WHICH gensyms the sweep bothers to check. A stale
entry (its definition was rolled back away) → `clear_symbol_state` is a harmless
no-op. A missing entry → that gensym is simply not reclaimed (a sound, bounded
non-reclamation). Therefore the registry is NOT added to the
`EvaluationRollbackSnapshot` (it would be expensive to clone and is not needed for
soundness). Document this with a comment.

### M2. The root walk — `for_each_root_expr` (THE completeness anchor)

Add `fn for_each_root_expr(&self, visit: &mut dyn FnMut(&Expr))` on `Session`
(put it next to the memory proxy or in a new `scoping/gensym_gc.rs` module). It
must visit EVERY `Expr` that the session retains, **mirroring
`current_memory_proxy_bytes` field-for-field** (which already enumerates all of
them for sizing). Cross-reference it in a comment: "Keep in sync with
`current_memory_proxy_bytes`; both must cover every Expr-holding Session field."

Visit the exprs in (at least — verify against the proxy):
- `self.definitions` — for each `(sym, def)`: own_value, delayed_own_value, and
  every up/down/sub `value.lhs` + `value.rhs`. **EXCEPTION:** SKIP entries whose
  `sym` is in `gc_candidate_gensyms` (a candidate's own definition is NOT a root —
  it is followed transitively in M3 only if the candidate is marked live). Visit
  all non-candidate (global + non-candidate-gensym) definitions.
- `self.options` (option values), `self.syntax_information`, `self.message_definitions`
  (immediate + delayed), `self.once_values`, `self.option_value_scopes` (explicit
  options), `self.dynamic_own_values` (the bound values), `self.evaluation_stack`
  (`entry.expr`), `self.reap_scopes` (patterns, handler, every bucket `value`),
  `self.monitor_scopes` (display_expr, evaluation_monitor, step_monitor),
  `self.pending_suspension` (its `active_stack` entry exprs), and
  `self.pending_eval_request` IF it carries an `Expr`.
- Skip the String-only fields (`prints`, `messages`, `last_message_list`,
  `trace_events`, `handled_non_loader_eval_errors`, input prompts) — they hold
  RENDERED TEXT, not live symbol references, so they cannot keep a gensym
  definition live. (Attributes hold no exprs either.)

If you find an Expr-holding Session field that `current_memory_proxy_bytes` sizes
but this list omits, ADD it and report it — a missed root = unsound reclamation.

### M3. Mark + sweep — `collect_and_sweep_dead_gensyms(&mut self, extra_roots: &[Expr])`

```
1. if self.gc_candidate_gensyms is empty -> return.
2. live: HashSet<Symbol> = {}; worklist: Vec<Symbol> = {}.
3. SEED from active modules (covers in-flight gensyms captured in native
   suspend/resume continuations — those belong to modules still on
   module_activations): for each activation in self.module_activations, for each
   g in activation.gensyms, if g in gc_candidate_gensyms and newly-live -> add to
   live + worklist.
4. SEED from roots: for each extra_root expr and via for_each_root_expr, walk the
   expr; for each Symbol leaf that is in gc_candidate_gensyms and newly-live ->
   add to live + worklist.
5. CLOSURE: while worklist non-empty, pop g; if let Some(def)=self.definitions.get(g):
   walk def's exprs (own/delayed/up/down/sub lhs+rhs) for Symbol leaves in
   gc_candidate_gensyms that are newly-live -> add to live + worklist.
6. SWEEP: dead = gc_candidate_gensyms - live. Set self.in_gensym_sweep=true; for g
   in dead: self.clear_symbol_state(&g, true,true,true,true,true); remove g from
   gc_candidate_gensyms. Set self.in_gensym_sweep=false.
```
Use a symbol-leaf walker (recurse `Expr::Call` head+args, match `Expr::Atom(
Atom::Symbol(s))`). Add a small `fn for_each_symbol_in_expr(expr, &mut dyn
FnMut(&Symbol))` if none exists; reuse `definition_references_symbol`'s traversal
shape.

**`in_gensym_sweep` guard:** in `bump_evaluation_epoch_for_symbol`'s brief-049
escape barrier, early-return when `self.in_gensym_sweep` is true — the sweep's own
`clear_symbol_state` calls bump epochs for swept gensyms, and we must NOT let those
re-trip active modules' escape flags (it would be O(active×swept) churn; the
markings would be spurious). Reclaiming a dead gensym is not a capture event.

### M4. Trigger — at the end of `finish_module_scope`

After the brief-049 reclaim + the M1 registry update + the activation truncate, if
`self.gc_candidate_gensyms.len() >= gensym_gc_threshold()` then call
`self.collect_and_sweep_dead_gensyms(&extra_roots)` where `extra_roots` is the
value being returned (`outcome`'s `Ok` expr, if any — it is flowing to the caller
and MUST be a root; an `Err` payload expr too if it carries one). This is a SAFE
point: the current module is done (popped), all still-active OUTER modules are on
`module_activations` (seeded as live in M3 step 3), and the eval stack / suspension
are walked as roots.

`gensym_gc_threshold()`: read once from `RUM_GENSYM_GC_THRESHOLD` (cold-load, like
the existing env reads), default **200_000**. A higher threshold = fewer sweeps
(less CPU) but more transient memory between sweeps; lower = tighter memory. Making
it env-tunable lets the gate force frequent sweeps (low value) to demonstrate the
bound, and lets me tune the cold run without recompiling.

## Soundness (the red line — read three times)

- **Soundness is the MARK, not the registry.** A gensym is reclaimed ONLY if it is
  unreachable from the COMPLETE root set. The registry just bounds the candidate
  set; a stale/superset or missing/subset registry are both safe (the mark protects
  any live gensym; a missing candidate is merely not reclaimed).
- **Active-module gensyms are always live (M3 step 3).** This is what makes the
  sweep safe w.r.t. the native suspend/resume continuation closures (which capture
  un-walkable `renames`/`bindings` maps holding gensyms): any such gensym belongs to
  a module whose `finish_module_scope` has NOT run, so its activation is still on
  `module_activations` → seeded live. Never reclaim a gensym of an active module.
- **Completeness of the root walk is load-bearing.** A missed Expr-holding field =
  a live gensym wrongly reclaimed = a silently wrong proof. Mirror
  `current_memory_proxy_bytes` exactly and cross-reference it.
- **Transitive closure both ways via definitions.** A live global whose RHS
  references gensym g keeps g live; a live g whose def references h keeps h live.
  Iterate to fixpoint (M3 steps 4-5).
- When in any doubt, the gensym stays live (not swept) = today's behavior.

## Scope

- Files you MAY modify / create:
  - `crates/rum-runtime/src/runtime/model.rs` (the 2 fields + accessor if needed),
  - `crates/rum-runtime/src/runtime/session_state.rs` (init),
  - `crates/rum-runtime/src/runtime/scoping/lexical.rs` (registry hooks in
    `finish_module_scope` + the trigger),
  - a NEW `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs` (or place in
    lexical.rs) for `for_each_root_expr`, `for_each_symbol_in_expr`,
    `collect_and_sweep_dead_gensyms`, `gensym_gc_threshold` — wire the module in
    `scoping/mod.rs` if new,
  - `crates/rum-runtime/src/runtime/evaluation_cache.rs` (the `in_gensym_sweep`
    early-return in the barrier),
  - `crates/rum-runtime/src/runtime/definition_analysis_helpers.rs` (only if you add
    the symbol-leaf walker here),
  - a test module under `crates/rum-runtime/src/runtime/tests/` for the
    nested/recursive bounded-memory + soundness cases.
- MUST NOT touch:
  - any `mma-hol` clone / the acceptance standard (never edit it to pass);
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol`/serde shapes — the interner
    `&'static` name reclamation is Phase 2, explicitly NOT this brief;
  - the matcher/evaluator dispatch beyond the named hook points;
  - brief-049's leaf-reclaim logic / suspend-safe finalizer STRUCTURE (extend it,
    don't rewrite it);
  - the `EvaluationRollbackSnapshot` (do NOT add the registry to it — M1);
  - `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` (incl. `docs/dev/codex/**`) except
    appending to your report; `.git`, git config, `Cargo.lock`.
- Default-path cost: the registry touch is only at module exit (a `HashSet`
  insert/remove for retained gensyms — escaped modules only; flat-leaf modules pay
  one cheap `remove` no-op). The sweep is amortized (threshold-gated). The barrier
  gains one `bool` check (`in_gensym_sweep`). No new per-eval-step cost.

## rum / Rust pitfalls (read twice)

1. **Mark is authoritative; registry is advisory.** Do not try to keep the registry
   perfectly in sync with rollback — it is not needed and the clone would be costly.
2. **Active-module gensyms MUST be seeded live** (M3 step 3) or you will reclaim a
   gensym captured in a live continuation = unsound. This is the single most
   important safety step.
3. **`for_each_root_expr` must mirror `current_memory_proxy_bytes`.** Walk the same
   fields. Cross-reference in a comment. A missed field is unsound.
4. **`in_gensym_sweep` guard** in the escape barrier, or the sweep's own clears churn
   the active modules' escape flags.
5. **`clear_symbol_state`, never `remove_symbol_state`** (the latter grows
   `removed_symbols`).
6. **Skip candidate defs in the root SEED** (M2) — a candidate's own definition is
   not a root; only follow it transitively if the candidate is marked live.
7. **Stop-loss.** If a soundness smoke prints an `x$N`/unbound symbol, or
   `cargo test --workspace` regresses for a reason you cannot make sound, STOP and
   report — never relax the mark to force the memory win.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# (a) Bounded memory under nesting/recursion — force frequent sweeps with a low
#     threshold; the delta must PLATEAU, not grow linearly with activations:
echo "f[50]x2000  (~100k acts):"; RUM_GENSYM_GC_THRESHOLD=20000 $BIN --eval 'f[n_]:=Module[{r}, If[n<=0,0, r=f[n-1]+1; r]]; b=MemoryInUse[]; Do[f[50], {i,1,2000}]; MemoryInUse[]-b'
echo "f[50]x10000 (~500k acts):"; RUM_GENSYM_GC_THRESHOLD=20000 $BIN --eval 'f[n_]:=Module[{r}, If[n<=0,0, r=f[n-1]+1; r]]; b=MemoryInUse[]; Do[f[50], {i,1,10000}]; MemoryInUse[]-b'
echo "f[50]x40000 (~2M acts):";  RUM_GENSYM_GC_THRESHOLD=20000 $BIN --eval 'f[n_]:=Module[{r}, If[n<=0,0, r=f[n-1]+1; r]]; b=MemoryInUse[]; Do[f[50], {i,1,40000}]; MemoryInUse[]-b'
echo "nested-2 x20000:"; RUM_GENSYM_GC_THRESHOLD=20000 $BIN --eval 'b=MemoryInUse[]; Do[Module[{a}, a=Module[{c}, c=i; c]; a], {i,1,20000}]; MemoryInUse[]-b'

# (b) Soundness — every line must print the WK-correct value, never an x$N:
$BIN --eval 'g=Module[{x}, x=7; Hold[x]]; g[[1]]'                              # 7
$BIN --eval 'Module[{x}, x=7; gg:=x; Null]; gg'                               # 7
$BIN --eval 'Module[{a,b}, b=5; a:=b; Hold[a]] // First'                      # 5
$BIN --eval 'mk[n_]:=Module[{c=n}, Function[x, x+c]]; ff=mk[3]; ff[5]'         # 8 (closure escapes, survives sweeps)
$BIN --eval 'RUM_GENSYM_GC_THRESHOLD=1; f[n_]:=Module[{r}, If[n<=0,0, r=f[n-1]+1; r]]; f[50]'  # 50 (aggressive sweep mid-recursion must NOT break it)
RUM_GENSYM_GC_THRESHOLD=1 $BIN --eval 'g[n_]:=Module[{acc}, acc=n*n; acc]; Sum[g[i], {i,1,20}]'  # 2870

# (c) Comparator unchanged on the leaf/representation rows:
bash scripts/core-memory-microbench.sh --engine both --output-dir /tmp/cdx_brief_050_mem; cat /tmp/cdx_brief_050_mem/comparison.tsv
```

Paste every line verbatim. The headline gate (a): the `f[50]` delta must be
BOUNDED — roughly the same at x10000 and x40000 (≈ threshold-bounded, NOT the ~5x
of activations it would be without the sweep). Add a rum-runtime test module
asserting: (1) with a low `gensym_gc_threshold`, a recursive/nested Module loop's
post-loop `MemoryInUse` delta is bounded (e.g. < 2_000_000 for f[50]×10000, vs the
145 MB without the sweep); (2) the closure/result/global-def/cross-gensym soundness
cases still evaluate correctly; (3) `MaxMemoryUsed[]`/`MemoryInUse[]` still
integers. (If reading the env per-eval is awkward in tests, expose a
`pub(in crate::runtime)` setter for the threshold so tests can set it low.)

## Hard rules

- Do NOT `git commit`/branch/push or touch git config. Leave changes in the working
  tree; Claude runs the authoritative gate (incl. re-running the cold `run_all.wls`
  under the watchdog to confirm the cold load now stays bounded) and commits.
- Do NOT modify files outside Scope. If a needed root source is not in M2's list
  (an Expr-holding Session field the proxy sizes but M2 omits), ADD it to the walk
  and report it — never guess; soundness depends on the root set being complete.
- Full machine access, nothing outside this repo, no network. Commands: edits in
  scope, `cargo`, release `rum-cli`, the memory comparator, `wolframscript` (only via
  the comparator), shell for the probes.
- Never edit any `mma-hol` file as the fix or the acceptance path.
- Stop-loss: same failure twice, the memory does not plateau, or a soundness smoke
  prints an `x$N`/unbound symbol → STOP and report.
- Minimal diff; no drive-by reformatting/renames.
- Paste real verbatim output; if a run did not reach green / the memory did not
  plateau, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, line ranges + one-line why (the registry fields, the
   `finish_module_scope` hooks, `for_each_root_expr` + the mark-sweep + threshold,
   the `in_gensym_sweep` barrier guard, the tests).
2. Verification — `fmt`, `cargo test --workspace` result line, the (a) plateau
   deltas, the (b) soundness lines, and (c) comparison.tsv — all verbatim.
3. Verdict — the bounded-memory plateau numbers (x2000 vs x10000 vs x40000), the
   soundness results, whether the comparator leaf/representation rows are unchanged.
4. Stop-loss / open questions — any root source you were unsure about, any
   conservative non-reclamation you noticed (empty if none).
