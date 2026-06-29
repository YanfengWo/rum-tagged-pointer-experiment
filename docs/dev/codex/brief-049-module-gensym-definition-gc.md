# brief-049: Memory line Phase 1 — Module-gensym DEFINITION GC (the O(N)→O(1) leak fix)

## Goal

Stop the dominant memory leak: `Module` allocates a fresh gensym per local
(`x$N`), stores definitions on it during the body, and **never reclaims them** —
so `Do[Module[{x}, x=i^2; x], {i,1,20000}]` grows `MemoryInUse` by ~6 MB in rum vs
~80 B in WK (the cold `run_all` 50 GB OOM cause). Add a **sound, conservative
scope-exit GC**: when a `Module` activation finishes, reclaim the definitions of
the gensyms it created — but ONLY when it is provably safe (the gensym did not
escape into the result or into any retained session state). When done, the
Phase-0 comparator shows `module-leak-20000`'s rum `MemoryInUse` delta drop from
~5.97 MB to near WK's level (< ~100 KB), with `cargo test --workspace` still green.

**SOUNDNESS IS THE RED LINE.** A gensym that escaped (into the result, a global
definition, options/messages/syntax, Sow/Reap, Once, Monitor, …) and still has a
definition MUST be retained, or later evaluation of the escaped reference returns
an unbound symbol. This brief reclaims ONLY when escape is provably impossible;
when in any doubt it retains (= today's behavior, no regression). It is fine — and
expected — that this Phase-1 GC reclaims only "leaf" modules (bodies that do not
transitively write non-local state); that is exactly the dominant loop-local leak.
Reclaiming escaped-then-died gensyms (a root-set mark-sweep) is a deferred follow-up.

## Context pointers

- Read `AGENTS.md` (operational notes, the unmodified-`mma-hol` hard line),
  `docs/memory-management-plan.md` (Phase 1 is this brief), and the
  `2026-06-29` PROGRESS entries for the memory line (Phase 0 instruments landed in
  `9140fd4`).
- Phase 0 gave us the yardstick: `scripts/core-memory-microbench.sh` and
  `MaxMemoryUsed[]`. This brief is gated on the comparator showing the leak fixed.
- Key files / facts (verified):
  - `crates/rum-runtime/src/runtime/scoping/lexical.rs` — `eval_module:48`
    allocates gensyms via `allocate_fresh_symbol` into a `renames` map, then
    `resume_module_initializers:384` substitutes + evaluates initializers (storing
    each value as the gensym's `own_value` via `store_module_initializer_value:448`)
    and finally substitutes + evaluates the body via `self.evaluate(body)` at
    index>=len, returning **with no cleanup**. CONTRAST `eval_block:98` /
    `resume_block_initializers:467` / `resume_block_body:679` / `finish_block_scope:697`:
    Block captures `saved_state`, threads it + a `dynamic_depth` through the
    suspend-safe resume chain (`suspend_with_result_frame`), and restores on EVERY
    exit (Ok, Err, and resumed-after-InputRequired). **Your Module GC must mirror
    this exact suspend-safe finalizer structure.**
  - `crates/rum-runtime/src/runtime/scoping/unique.rs` — `allocate_fresh_symbol:81`
    (gensym name = `<prefix>$<id>`, `next_generated_symbol_id` monotonic).
  - `crates/rum-runtime/src/runtime/definition_clear.rs` — `clear_symbol_state:238`
    removes a symbol's definition + attributes/messages/options/syntax and bumps the
    epoch, and does NOT touch `removed_symbols`. **Use `clear_symbol_state(g, true,
    true, true, true, true)` as the reclaim primitive.** Do NOT use
    `remove_symbol_state:285` (it calls `mark_symbol_removed` → grows
    `removed_symbols` + `symbol_generations`, which would just MOVE the leak).
  - `crates/rum-runtime/src/runtime/evaluation_cache.rs` —
    `bump_evaluation_epoch_for_symbol:234` is called by **every** definition write
    (own/down/up/sub value Set/SetDelayed/TagSet/UpSet/Unset/AppendTo/Clear/…; it
    has `#[track_caller]`). This is the universal definition-write hook — your
    escape barrier for the DEFINITION channel goes here.
  - `crates/rum-runtime/src/runtime/definition_analysis_helpers.rs` —
    `expr_contains_symbol(expr, target):285` — reuse for the result-scan.
  - Non-definition capture chokepoints (these do NOT bump the epoch): options
    `set_options_for_symbol` (`options/core.rs`), syntax `self.syntax_information.insert`
    (`namespace_tools/syntax.rs:39`), messages `message_definitions.entry(key).or_default()`
    (`message_catalog.rs:73`), Once `once_values.insert` (`package_tools.rs:124`), Sow
    `bucket.values.push` (`debug_tools/reap.rs:213`), Monitor scope push
    (`monitor_scopes`).
  - `crates/rum-runtime/src/runtime/suspension/core.rs` — `suspend_with_result_frame:139`.
  - `crates/rum-runtime/src/runtime/model.rs` `Session:300` + `session_state.rs`
    `Default for Session:~360` — where state fields live + are initialized.

## The mechanism (precise — implement exactly this)

### M1. Per-activation escape tracking state

A `Module` activation tracks the gensyms it created, an `escaped` flag, and a
baseline of the append-only side-table sizes (the non-definition capture catch-all).

`model.rs` — add to `Session`:
```rust
pub(super) module_activations: Vec<ModuleActivation>,
```
and define (near the other small runtime structs in `model.rs`):
```rust
pub(super) struct ModuleActivation {
    pub(super) gensyms: Vec<Symbol>,
    pub(super) escaped: bool,
    pub(super) side_table_stamp: SideTableStamp,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct SideTableStamp {
    once_values: usize,
    monitor_scopes: usize,
    option_value_scopes: usize,
    dynamic_own_values: usize,
    reap_bucket_values: usize, // total Sow'd values across all reap scopes
}
```
`session_state.rs` `Default`: `module_activations: Vec::new(),`.

Add helpers on `Session`:
- `fn current_side_table_stamp(&self) -> SideTableStamp` — read the 5 sizes
  (`reap_bucket_values` = sum over `self.reap_scopes` of sum over `buckets_by_pattern`
  of `bucket.values.len()`; usually 0).
- `fn note_active_modules_escaped(&mut self)` — `for a in &mut self.module_activations
  { a.escaped = true; }` (trip ALL active activations — always sound/conservative).
  Cheap early return when the stack is empty.

### M2. The escape barriers (gated on a non-empty activation stack)

- **Definition channel** — in `bump_evaluation_epoch_for_symbol(symbol)`
  (`evaluation_cache.rs:234`), AFTER its existing body, add:
  ```rust
  if !self.module_activations.is_empty() {
      for activation in &mut self.module_activations {
          if !activation.gensyms.contains(symbol) {
              activation.escaped = true;
          }
      }
  }
  ```
  Rationale: a write to a symbol that is NOT this activation's own gensym may have
  captured this activation's gensym (e.g. `glob = Hold[x]`, or an enclosing module's
  local capturing an inner gensym). A write to the activation's OWN gensym is fine.
  The `is_empty()` guard keeps the no-module path (all top-level Sets, package
  loading) at one length-load + branch — NOT the eval hot path.
- **Non-definition keyed channels** (in-place replace possible, so barrier at the
  site) — at the top of `set_options_for_symbol`, the `SyntaxInformation` setter
  (`namespace_tools/syntax.rs:39` path), and the message-definition set path
  (`message_catalog.rs` `define_message`/the `entry(key).or_default()` writer):
  call `self.note_active_modules_escaped()`. These are off the hot path.
- **Append-only channels** (Once / Sow / Monitor / OptionValue scopes / dynamic
  bindings) — covered by the `side_table_stamp` comparison at scope exit (M4), no
  per-site barrier needed (they can only CAPTURE by growing).

### M3. Push the activation in `eval_module`, thread it suspend-safely

In `eval_module` (`lexical.rs:48`), after building `renames` (the fresh gensyms),
before `resume_module_initializers`:
```rust
let gensyms: Vec<Symbol> = renames
    .values()
    .filter_map(|e| expr_as_symbol(e).cloned())
    .collect();
let activation_depth = self.module_activations.len();
self.module_activations.push(ModuleActivation {
    gensyms,
    escaped: false,
    side_table_stamp: self.current_side_table_stamp(),
});
```
Thread `activation_depth` (a `Copy` `usize`) through `resume_module_initializers`
and into a new terminal finalizer. **Mirror `eval_block`'s structure exactly:** wrap
the body evaluation (the `index >= items.len()` branch in
`resume_module_initializers`) in `suspend_with_result_frame` with a
`finish_module_scope(activation_depth, outcome)` finalizer, just as
`resume_block_body` wraps its body with `finish_block_scope`. On the
`Err(EvalError::InputRequired { request })` arm, return it WITHOUT running the
finalizer (the frame re-invokes the finalizer on resume) — copy block's handling
verbatim. The initializer-suspension path (`suspend_with_frame_lazy`) already
re-enters `resume_module_initializers`; the activation stays on the stack across it
(it is popped only by the terminal finalizer), which is correct.

### M4. `finish_module_scope` — the conservative GC

```rust
fn finish_module_scope(
    &mut self,
    activation_depth: usize,
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    // Read the activation, decide the reclaim set, THEN pop, THEN clear.
    let activation = self.module_activations[activation_depth].clone(); // gensyms + flag + stamp
    let escaped = activation.escaped
        || self.current_side_table_stamp() != activation.side_table_stamp;

    let mut reclaim: Vec<Symbol> = Vec::new();
    if let Ok(result) = &outcome {
        if !escaped {
            // retained = activation gensyms reachable from the result, transitively
            // through retained gensyms' own definitions.
            let mut retained: BTreeSet<Symbol> = activation
                .gensyms
                .iter()
                .filter(|g| expr_contains_symbol(result, g))
                .cloned()
                .collect();
            // transitive closure over this activation's gensyms only (small set):
            loop {
                let mut grew = false;
                for g in &activation.gensyms {
                    if retained.contains(g) { continue; }
                    if let Some(def) = self.definitions.get(g) {
                        if retained.iter().any(|r| definition_references_symbol(def, r))
                            // a retained gensym's def referencing g, OR g's own def
                            // referencing a retained gensym — see note below.
                        { /* handled below */ }
                    }
                }
                // (implement the closure as described in the note; set grew=true when
                //  a new gensym is added)
                if !grew { break; }
            }
            reclaim = activation
                .gensyms
                .iter()
                .filter(|g| !retained.contains(g))
                .cloned()
                .collect();
        }
        // escaped => reclaim nothing (retain all).
    }
    // outcome is Err (Throw/Return/error): reclaim nothing — the error payload may
    // reference a gensym; conservative retain.

    self.module_activations.truncate(activation_depth); // pop self BEFORE clearing
    for g in reclaim {
        self.clear_symbol_state(&g, true, true, true, true, true);
    }
    outcome
}
```

**The transitive-closure direction (get this right):** a gensym `g` must be
RETAINED if it is reachable from a retained gensym. Reachability flows along
definition references: if retained gensym `r` has a definition whose stored exprs
contain `g`, then `g` is reachable via `r` → retain `g`. Implement
`definition_references_symbol(&SymbolDefinition, &Symbol) -> bool` checking
`own_value`, `delayed_own_value`, and every down/up/sub `value.lhs`/`value.rhs` via
`expr_contains_symbol`. Iterate to a fixpoint over the activation's (small) gensym
set: start with result-referenced gensyms, then repeatedly add any activation gensym
referenced by an already-retained gensym's definition. Reclaim the complement. (This
covers `Module[{a,b}, b=5; a:=b; Hold[a]]`: `a` is in the result, `a`'s
delayed_own_value references `b`, so `b` is retained too.)

Pop the activation (`truncate`) BEFORE the `clear_symbol_state` loop so the barrier
inside `clear_symbol_state`'s epoch bump does not see this activation. (For nested
modules an enclosing activation is already `escaped` from the inner body's writes, so
any incidental trip is a harmless no-op.)

## Worked soundness cases (your tests must cover these)

1. **Leaf reclaim** `Do[Module[{x}, x = i^2; x], {i,1,N}]` — body writes only `x$N`
   (own gensym → no trip), no side-table growth, result is an Integer (no gensym) →
   reclaim `x$N`'s ownvalue each iteration → `MemoryInUse` stays flat. THE TARGET.
2. **Result escape** `Module[{x}, x = 7; Hold[x]]` → result `Hold[x$N]` contains the
   gensym → retained; its ownvalue stays. (`g = Module[{x}, x=7; Hold[x]]; ReleaseHold[g]`
   must still give `7`.)
3. **Global-definition escape** `Module[{x}, x = 7; gg := x; Null]` → `gg`’s
   SetDelayed write bumps the epoch for `gg` (non-own) → escaped → retain `x$N` →
   `gg` still evaluates to `7`.
4. **Cross-gensym** `Module[{a, b}, b = 5; a := b; Hold[a]]` → `a` in result, `a`'s
   def references `b` → both retained. `ReleaseHold` of the result gives `5`.
5. **Options/Sow capture** `Module[{x}, x = 7; Options[gg] = {o -> Hold[x]}; Null]`
   (setter barrier) and `Reap[Module[{x}, x = 7; Sow[Hold[x]]; Null]]` (side-table
   stamp) → escaped → retain `x$N`; the captured held `x$N` still resolves to `7`.
6. **Nested is conservative** `Module[{r}, r = Module[{t}, t = 1; t]; r]` → the inner
   leaf reclaims `t`; the outer sees a non-own write (`t`) during its body → outer
   escaped → retains `r` (sound; conservative). Result is `1`.
7. **Error path** `Catch[Module[{x}, x = 7; Throw[x]]]` → Err outcome → reclaim
   nothing → `7` (the thrown value) is correct.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model.rs` (the `ModuleActivation`/`SideTableStamp`
    types + the `Session` field),
  - `crates/rum-runtime/src/runtime/session_state.rs` (init the field),
  - `crates/rum-runtime/src/runtime/scoping/lexical.rs` (push activation in
    `eval_module`; thread `activation_depth`; the suspend-safe body frame +
    `finish_module_scope`; the closure helpers),
  - `crates/rum-runtime/src/runtime/evaluation_cache.rs` (the definition-channel
    barrier in `bump_evaluation_epoch_for_symbol`),
  - `crates/rum-runtime/src/runtime/options/core.rs`,
    `crates/rum-runtime/src/runtime/namespace_tools/syntax.rs`,
    `crates/rum-runtime/src/runtime/message_catalog.rs` (the 3 keyed-setter barriers),
  - `crates/rum-runtime/src/runtime/definition_analysis_helpers.rs` ONLY if you add
    `definition_references_symbol` here (else put it in `lexical.rs`),
  - a test module under `crates/rum-runtime/src/runtime/tests/` for the soundness cases.
- MUST NOT touch:
  - anything under any `mma-hol` clone or the acceptance standard — never edit it to
    pass (`AGENTS.md`);
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol` and any public type/serde shape —
    the interner reclamation is Phase 2, NOT this brief;
  - the matcher / pattern engine / evaluator dispatch beyond the named scoping +
    epoch-bump sites;
  - `eval_block` / `eval_with` behavior (only READ them as the pattern to mirror);
  - snapshot/serde of `Session` (`module_activations` is transient runtime state, NOT
    serialized — like `print_sink`/`peak_memory_proxy_bytes`);
  - `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` (incl. `docs/dev/codex/**`) except
    appending to your own report; `.git`, git config, `Cargo.lock`.
- Default-path cost: the only added cost on the definition-write path is the
  `module_activations.is_empty()` guard (one length load) when no module is active.
  When a module IS active, the per-write loop is over a tiny gensym set. The GC at
  module exit (`clear_symbol_state` per reclaimed gensym) is NEW per-Module work and
  is the intended trade (it replaces unbounded growth). A modest `module-local` /
  `do-small` timing increase is acceptable; a regression on NON-module labels
  (`set-ownvalue`, `literal-integer`) or a large (>~25%) module-label regression is NOT.

## rum / Rust pitfalls (read twice)

1. **Soundness > reclamation.** When unsure, RETAIN. Every "retain all" path (escaped
   flag, side-table change, Err outcome, gensym in result/closure) is correct-and-safe.
   Only the provably-dead set is reclaimed.
2. **Mirror `eval_block` for suspend-safety.** The activation must be popped on EVERY
   terminal exit and survive InputRequired suspend/resume. Do not invent a new
   suspension shape; copy `resume_block_body`/`finish_block_scope` structure. If the
   body frame’s `Err(InputRequired)` is mishandled, a resumed module corrupts the
   activation stack — test a suspending module if you can, else reason it through
   against the block code.
3. **`clear_symbol_state`, not `remove_symbol_state`.** The latter grows
   `removed_symbols`/`symbol_generations` — it would move the leak, not fix it.
4. **Pop before clear.** Truncate the activation stack before the `clear_symbol_state`
   loop so the epoch-bump barrier inside it does not see the just-finished activation.
5. **Transitive closure direction.** Retain `g` if a RETAINED gensym’s definition
   references `g` (reachability from roots), iterate to fixpoint. Over the activation’s
   small gensym set only — never scan all `self.definitions`.
6. **`is_empty()` guard on the barrier.** Keep the no-module definition-write path at
   a single length-load. No allocation/Mutex there.
7. **Stop-loss.** If `cargo test --workspace` regresses for a reason you cannot make
   sound, or the comparator does NOT show the leak drop, STOP and report — do not relax
   a retain condition to force the win.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo build --release -p rum-cli

# (a) The leak is fixed — module-leak-20000 rum MemoryInUse delta must collapse:
bash scripts/core-memory-microbench.sh --engine both --output-dir /tmp/cdx_brief_049_mem
echo "=== comparison.tsv ==="; cat /tmp/cdx_brief_049_mem/comparison.tsv

# (b) No timing regression on the scoping neighbors (and none on non-module labels):
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_049_micro
grep -E '^(set-ownvalue|module-local|block-local|with-local|do-small|literal-integer)\b' /tmp/cdx_brief_049_micro/ratios.tsv

# (c) Soundness smoke (each must print the WK-correct value, NOT an x$N symbol):
BIN=./target/release/rum-cli
$BIN --eval 'g = Module[{x}, x = 7; Hold[x]]; ReleaseHold[g]'                 # 7
$BIN --eval 'Module[{x}, x = 7; gg := x; Null]; gg'                          # 7
$BIN --eval 'Module[{a, b}, b = 5; a := b; Hold[a]] // ReleaseHold'          # 5
$BIN --eval 'Catch[Module[{x}, x = 7; Throw[x]]]'                            # 7
$BIN --eval 'before=MemoryInUse[]; Do[Module[{x}, x=i^2; x], {i,1,20000}]; MemoryInUse[]-before'  # near 0
```

Paste every line verbatim. The headline gate: comparison.tsv `module-leak-20000`
`rum_mem_delta` drops from ~5,966,099 to under ~100,000 (toward WK's ~80). Add a
rum-runtime test module asserting the 7 worked cases above (use `eval_text` +
value/`MemoryInUse` deltas; for the leak case assert the post-loop `MemoryInUse`
delta is bounded, e.g. < 200_000, vs the multi-MB it would be without the GC).

## Hard rules

- Do NOT `git commit`/branch/push or touch git config. Leave changes in the working
  tree; Claude runs the authoritative gate (incl. a Module-heavy mma-hol validation)
  and commits.
- Do NOT modify files outside Scope. If a needed capture channel is NOT in M2's list
  (you find another `Session` field that stores caller-provided `Expr`s under a
  non-gensym key), STOP and report it rather than guessing — soundness depends on the
  channel list being complete.
- Full machine access, nothing outside this repo, no network. Commands: edits in
  scope, `cargo`, release `rum-cli`, the two microbench scripts, `wolframscript` (only
  via the memory comparator), `python3`/shell for parsing.
- Never edit any `mma-hol` file as the fix or the acceptance path.
- Stop-loss: same failure twice, or the leak does not drop, or a soundness smoke
  prints an `x$N`/unbound symbol → STOP and report.
- Minimal diff; no drive-by reformatting/renames.
- Paste real verbatim output; if a run did not reach green / the leak did not drop,
  say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, line ranges + one-line why (the activation state, the 4
   barrier sites, the `eval_module` push + suspend-safe `finish_module_scope`, the
   closure helpers, the tests).
2. Verification — `fmt`, `cargo test --workspace` result line, comparison.tsv,
   the microbench neighbor rows, and the (c) soundness smoke lines, all verbatim.
3. Verdict — the `module-leak-20000` before/after `rum_mem_delta` (the headline), the
   7 soundness cases’ results, and the scoping-label timing deltas (state any regression).
4. Stop-loss / open questions — anything you retained-conservatively that surprised
   you, any capture channel you were unsure about (empty if none).
