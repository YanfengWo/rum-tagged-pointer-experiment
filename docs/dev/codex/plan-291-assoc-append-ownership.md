# Plan 291 — Association / AppendTo incremental-update ownership

This is a design-only result. No Rust, tests, scripts, or existing
documentation were changed.

## 1. Verification table

All timing probes used the release `rum-cli`, `AbsoluteTiming`, three
repetitions per shape, and the minimum. Every `RUM_PROFILE_*` variable was
explicitly set to `0` for timing. Raw output was written to
`/tmp/p291timings.raw` before `/tmp/p291timings.parsed` was produced. No
external engine or network was used.

| Finding | Verdict | Evidence | Actual result |
|---|---|---|---|
| A — `SharedArgs::into_vec` has a unique-owner fast path, but the 500-iteration AppendTo probe never reaches it | **CONFIRMED** | `crates/rum-core/src/expr.rs:570-590`; `crates/rum-core/src/profiling.rs:41-55`; required release command using `/tmp/p291a.rum`, raw output in `/tmp/p291a.raw` | `into_vec_total=500 into_vec_clone=500`, so 0/500 fast-path hits. The final snapshot on this checkout says `shared_args_alloc=1019`, not the hypothesized 1023: `rum-profile repr into_vec_total=500 into_vec_clone=500 into_owned_total=0 into_owned_clone=0 shared_args_alloc=1019 shared_expr_alloc=18`. |
| B — the definition-table clone is the cause and replacing it with an ownership transfer is sufficient | **PARTIAL** | Clone sites are real at `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:299-303` and `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:411-417`. The additional owner is established at `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:297-317` and `crates/rum-runtime/src/runtime/model/cache.rs:105-113`. | Claude was right that each clone adds an owner, but wrong that removing it is sufficient. A stable call-cache entry strongly owns the same `SharedArgs`; after a simple `own_value.take()`, the local value and cache still leave the strong count at least 2. The critical-holder trace is in section 2. |
| C — Association pays a separate parse/scan/render round trip, and a hash lookup alone is not the first fix | **PARTIAL** | `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:27-39,179-195,241-274,527-572`; controlled probes in `/tmp/p291timings.raw` used size 400 versus 800 while scanning 400 keys, and size 600 versus 1200 while scanning 600 keys | The round trip does dominate and an index alone would leave it intact, but the “roughly 3n clones” diagnosis is incomplete. Both parse and render call `canonicalize_association_entry_forms`, whose duplicate scan is quadratic. Equal-scan, 2x-volume ratios were **3.8103x** and **3.7762x**, not 2.88x/2.96x. Building from empty scaled 6.2205x (100→200) and 7.0496x (200→400), consistent with a near-cubic total from summing quadratic per-update canonicalization. |
| D — the contiguous builtin ABI rules out tree/HAMT structural sharing | **PARTIAL** | `crates/rum-core/src/expr.rs:620-632`; `crates/rum-runtime/src/runtime/metadata.rs:282-304` | `SharedArgs` must currently expose `&[Expr]` and dereference to `Vec<Expr>`, and every builtin receives `&[Expr]`. A trie cannot be a transparent drop-in. “Rules out” is too strong: lazy flattening, a specialized expression variant, or an ABI change could materialize a slice, but each is a broad representation/ABI project. None is justified for these two mutation paths. |
| E — nested Part updates make the outer container dominate | **CONFIRMED** | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:317-370,432-453`; `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:173-200`; min-of-3 output below | At inner n=200: flat `15783/20000000 = 0.000789150 s`; outer m=10 `238977/250000000 = 0.000955908 s` (+0.000166758); m=100 `263279/200000000 = 0.001316395 s` (+0.000527245); m=1000 `4872701/1000000000 = 0.004872701 s` (+0.004083551, 5.17x the flat time). At m=1000, n=100/200/400 took 0.002323689/0.004872701/0.010443962 s, about 2.10x and 2.14x per doubling. A symbol-only ownership change does **not** extend automatically: `current_target` retains the selected child while `current` clones it, and `replace_exact_path` clones the child again. |

The complete raw timing minima and scaling calculations were:

```text
assoc-build-100        974177/250000000       0.003896708
assoc-build-200        6059831/250000000      0.024239324
assoc-build-400        170878299/1000000000   0.170878299
                         ratios 6.2205x, 7.0496x

append-flat-100        133201/500000000       0.000266402
append-flat-200        15783/20000000          0.000789150
append-flat-400        106759/50000000         0.002135180
append-flat-800        7254879/1000000000      0.007254879
                         ratios 2.9623x, 2.7057x, 3.3978x

assoc-size400-scan400  12147/10000000          0.001214700
assoc-size800-scan400  462843/100000000        0.004628430
                         ratio 3.8103x
assoc-size600-scan600  2717743/1000000000      0.002717743
assoc-size1200-scan600 5131357/500000000       0.010262714
                         ratio 3.7762x
```

The controlled Association probe constructed canonical associations outside
the timed region. The small arm updated its last key; the double-size arm
updated the key at the same position. Both arms therefore performed the same
number of `position()` comparisons, while parse/render volume doubled.

## 2. The critical unknown, answered

**A clone-to-take edit by itself does not make the `SharedArgs` unique.**
For the ordinary iteration after the first, the target list has three strong
owners at the point just before `into_vec`: the definition-table value, the
local `current` clone, and the call-stability cache entry. Taking the
definition value removes only the first, leaving the local and cache owners.
The implementation must deliberately remove or transfer the exact cache entry
before attempting exclusive mutation. Once that cache entry is removed, the
definition is mutated through safe `Arc::make_mut`; semantic aliases, tracing,
or retained results correctly trigger copy-on-write instead of being
overwritten.

### Holder inventory along `AppendTo[lst, i]`

| Potential holder | Live in the required probe at `Expr::unwrap_shared_args`? | Evidence and effect |
|---|---|---|
| `definitions[lst].own_value` | **Yes** | `current_symbol_held_update_value` clones it without removing it (`definition_updates.rs:299-303`), so the table retains one strong owner. |
| Local `current` / matched `args` | **Yes** | The clone is carried in `HeldUpdateCurrent::Ready` (`definition_updates.rs:311-314`), matched at `:129-135`, and its `SharedArgs` is consumed at `:145-147`. |
| Call-stability cache | **Yes** | Evaluation calls `cached_expr_is_evaluation_stable` before each pass (`resolution.rs:321-329`). A stable call miss inserts cloned `head` and `args` (`evaluation_cache/stability.rs:289-317`); `CallStabilityCacheEntry.args` is a strong `SharedArgs` (`model/cache.rs:105-113`). A symbol write does not remove that entry: `bump_evaluation_epoch_for_symbol` removes only symbol-keyed entries and updates dirty blooms (`evaluation_cache/stability.rs:56-88`). Even a dirty miss leaves the strong cache entry allocated. The 16K entry cap (`:9-12`) is not reached by 500 updates. |
| Builtin argument slice / `CallDispatchState` | **No target-value holder** | `CallDispatchState` owns the raw/evaluated arguments (`session/session_state.rs:87-113`), and ordinary dispatch borrows them (`eval_dispatch/mod.rs:115-117,122-149`). `AppendTo` has `HoldFirst` (`builtin_catalog/rules.rs:201-206`), so argument 0 remains the symbol `lst`, not its list value. Argument 1 is the evaluated `i`. |
| Active evaluation stack and fixed-point locals | **No target-value holder in this shape** | The stack clones the expression currently being evaluated (`eval_core/mod.rs:16-57`), which is the held `AppendTo[lst,i]` call, not `lst`'s value. The fixed-point `current` similarly owns that call (`resolution.rs:308-345`). |
| Suspension frame | **No in the required probe** | No input is requested. If item evaluation suspends, preparation records a continuation before the builtin handler is invoked (`eval_call_prepare.rs:375-413`), so no ownership window has started. Suspension frames retain call state (`suspension/core.rs:142-179,202-215`), not a value already removed by `AppendTo`. |
| Trace buffer | **No by default; conditional alias** | Tracing starts disabled (`session/session_state.rs:425`). When enabled, evaluation-pass events clone both sides (`definitions/application/mod.rs:69-93`; `model/messages.rs:25-42`), so a trace can retain a historical/current list. The proposed AppendTo fast path therefore falls back whenever tracing is observable. |
| Monitor scopes | **Not a direct target-value holder; an observability hazard** | Evaluation-pass tracing also drives EvaluationMonitor/StepMonitor callbacks (`definitions/application/mod.rs:69-96`; `debug_tools/monitor.rs:83-106`). Those callbacks can run user code and retain the value indirectly, so the fast path uses the existing table-fast-path condition: tracing disabled and either no monitor scope or monitor callbacks disabled (`control_data/branching_loops/iteration/table_fast_path.rs:219-224`). |
| Earlier returned `AppendTo` result | **No in `Do`; conditional elsewhere** | `Do` discards `Ok(_)` before the next iteration (`control_data/branching_loops/iteration/driver.rs:93-96`). `Table[AppendTo[...]]`, a user-held result, a second definition, or another containing expression is a legitimate semantic alias and must remain unchanged. |
| Source literal from `lst = {}` | **First iteration only** | `CompoundExpression` retains its full argument vector while advancing (`control_data/branching_loops.rs:302-343`), and immediate `Set` clones/stores the evaluated RHS (`definitions/definitions.rs:14-51,389-428`). That source-tree alias applies to the initial empty list, not later updated lists. |

The cache is therefore the hidden unconditional holder that falsifies the
simple ownership-transfer hypothesis. Conditional holders are not bugs; they
are exactly why the final mechanism must use copy-on-write. The existing
value-semantics probes produced:

```text
{<|1 -> 2|>, <|1 -> 2, 3 -> 4|>}
{{1, 2, 3}, {1, 2}}
```

for the Association and list alias cases respectively.

Step 6 deliberately introduces another strong holder for indexed
Associations: its identity witness. That holder is bounded and is taken,
pointer-validated, and dropped before both the stability-cache entry and the
definition-owned args enter COW mutation; leaving it live would recreate the
same bug diagnosed here.

## 3. Per-operator ownership-transfer safety

The prior “only ApplyTo can run user code in the update window” conclusion is
false. The following single release-CLI probe installed exact upvalues or
downvalues after assigning the target, then recorded what the callback saw:

```text
{"AddTo", u, 10}
{"SubtractFrom", u, 10}
{"TimesBy", u, 10}
{"DivideBy", u, 10}
{"ApplyTo", 1, 2}
{"AppendTo", f[], g[1]}
{"PrependTo", f[], g[1]}
{"Increment", u, 10}
{"Decrement", u, 10}
{"PreIncrement", u, 10}
{"PreDecrement", u, 10}
```

Raw output is in `/tmp/p291operators.raw`. Arithmetic examples used forms
such as `u /: Plus[u, 1] := (seen = x; 10)`; list examples assigned `x=f[]`
before defining `f[z_] := (seen=x; g[z])`.

All eleven operators are ordinary `HoldFirst` builtins
(`builtin_catalog/rules.rs:121-304`). Non-held arguments are evaluated during
argument preparation (`eval_call_prepare.rs:27-79,375-413`), then the
operator-specific handler builds an update and `finish_held_update` evaluates
it (`definition_updates.rs:387-404`). That second evaluation is the ownership
hazard.

| Operator | Update dispatch site | Can the post-read window run user code? | May a general path remove the own value? |
|---|---|---|---|
| AddTo | `definition_updates.rs:31-45`, builds `Plus[current, increment]` | **Yes**, an upvalue on an argument can match the newly formed `Plus`. | **No** |
| SubtractFrom | `:48-66`, builds `Subtract[current, decrement]` | **Yes**, exact or patterned upvalues can match `Subtract`. | **No** |
| TimesBy | `:69-83`, builds `Times[current, factor]` | **Yes**, through upvalues. | **No** |
| DivideBy | `:86-101`, builds `Divide[current, divisor]` | **Yes**, through upvalues. | **No** |
| ApplyTo | `:104-118`, builds `function[current]` | **Yes**, directly by design. | **No** |
| AppendTo | `:121-155`, preserves the current head and appends an argument | **Yes**, definitions/upvalues on that head or arguments can match the new arity/shape. A plain `List` also has `Nothing` normalization to preserve. | **Only on the proposed conservative, proved-stable plain-List fast path; otherwise no** |
| PrependTo | `:158-192`, preserves the current head and prepends an argument | **Yes**, for the same reason as AppendTo. | **No in this plan** |
| Increment | `:195-215`, builds `Plus[current,1]` | **Yes**, through upvalues. | **No** |
| Decrement | `:218-220,246-268`, builds `Plus[current,-1]` | **Yes**, through upvalues. | **No** |
| PreIncrement | `:222-231,246-268`, builds `Plus[current,1]` | **Yes**, through upvalues. | **No** |
| PreDecrement | `:234-243,246-268`, builds `Plus[current,-1]` | **Yes**, through upvalues. | **No** |

The proposed AppendTo fast path is deliberately narrower than “the current
head is List.” It requires all of the following before exclusive mutation:

1. an exact, currently valid stable-call proof for the current list;
2. a stable evaluated item;
3. the unmodified core System `List` attribute/definition state;
4. no active upvalue definitions (conservative but O(1));
5. an item that is not `System`Nothing;
6. no active `current_time_deadline` or `current_memory_limit` constraint; and
7. the host interrupt/timeout poll to have succeeded; and
8. no observable tracing/monitor callback
   (`!tracing_enabled && (monitor_scopes.is_empty() ||
   !monitor_callbacks_enabled)`).

If any condition is false, the existing clone-then-evaluate path runs with the
definition populated. For example, current behavior for a shape-dependent
upvalue is:

```text
ClearAll[x,u,seen]; x={u};
u /: List[u,1] := (seen=x; transformed);
{AppendTo[x,1],seen,x}
    => {transformed, {u}, transformed}
```

The fast path must not intercept that case.

## 4. Failure and restore story

The safest transfer is no removal at all. The plan does **not** hold
`definition.own_value = None` across `evaluate`, a suspension point, a
message path, or any other user callback. It removes the exact stability-cache
holder, then obtains exclusive mutable access to the `SharedArgs` already
inside the populated definition with safe `Arc::make_mut`.

The enforcement points are:

- The existing general compound path remains unchanged. It evaluates the
  update at `definition_updates.rs:394`, propagates `EvalError`/suspension at
  `:395-401`, and stores only afterward at `:403`. Since the current value was
  cloned, every message, error, interrupt, resource-limit error, and nested
  user evaluation leaves the old definition present.
- AppendTo fast-path validation, stable-proof acquisition, protection
  recheck, interrupt poll, and any fallible capacity reservation occur before
  the first element mutation. Nested time/memory constraints and observable
  tracing/monitor callbacks force fallback. No evaluator call is allowed
  between first mutation and commit.
- A small safe-Rust rollback guard owns the append undo operation. It pops the
  appended element on unwind unless committed. `Vec::try_reserve(1)` runs
  before `push`, so rollback itself does not allocate. Semantic aliases cause
  `Arc::make_mut` to copy before mutation and remain unchanged.
- For an exact Part append, cache entries on the ancestor path are removed
  first; `Arc::make_mut` walks the definition-owned spine. The same pop guard
  rolls back the leaf. The generic Part path remains the fallback for bulk,
  invalid, callback-capable, constrained, or otherwise unproved cases.
- `Set` evaluates the RHS before lvalue handling (`definitions/definitions.rs:
  14-51`). Association selectors evaluate and may suspend at
  `association_data.rs:420-455`; the current association is fetched again
  only at `:458-475`. The new direct mutation must stay at that finish point,
  after all RHS/selector evaluation.
- Association append/replace uses an undo guard: pop a newly appended Rule, or
  swap the old Rule back into an existing position. Missing/invalid keys emit
  their message before mutation or after rollback. Hash-index capacity is
  reserved before expression/index mutation, and a combined guard keeps the
  expression and index atomic.
- The direct Association paths also fall back while
  `current_time_deadline` or `current_memory_limit` is active and poll host
  interrupt/timeout before mutation.
  Construct the replacement Rule and reserve every Vec/HashMap capacity
  before changing an element; a failed reservation simply selects the
  existing path while the definition is still populated.

A panic inside nested evaluation therefore occurs only on a fallback path,
where the definition was never removed. A panic inside the small no-callback
mutation block runs the undo guard. The host boundary catches and poisons on a
panic (`crates/rum-host/src/session.rs:637-643`), but correctness must not
depend on a later reset.

If an implementation instead introduces `Option::take()` spanning
`Session::evaluate`, `suspend_with_frame`, message construction, or a
fallible resource check, it violates this plan and must be rejected. A
borrowed-slot RAII guard could restore on unwind, but it cannot preserve the
old observable value while user code runs without retaining another strong
owner, which defeats unique mutation.

## 5. Step-by-step implementation plan

### Step 1 — Add measured labels before changing runtime code

- **Change:** add the seven labels in section 6 to
  `scripts/core-language-microbench.sh`.
- **Invariant:** benchmark statements contain the workload itself and no
  per-iteration sink/Set beyond the mutation being measured. Timing remains
  `AbsoluteTiming` and profiling remains disabled.
- **Gate:** `bash -n`; run the suite once and retain all seven baseline rows
  plus `do-iterator-null`, `set-ownvalue`, `part-list`, `module-local`,
  `list-literal-small`, and `symbol-ownvalue-read`.
- **Expected win:** none; this creates the missing acceptance baseline.
- **Workload delivered:** neither.

### Step 2 — Add safe exclusive-args and cache-proof transfer primitives

- **Change:** add safe `SharedArgs::get_mut`/`make_mut` methods in
  `rum-core`; add a mutation-only evaluation-cache module that can validate
  and remove one pointer-matched stable-call entry, return its stable/bloom
  proof only when currently valid, reinsert an incrementally updated proof,
  and evict exact entries along a known structural path. A stale but
  pointer-matched entry is still removed because it remains a strong owner;
  it simply yields no reusable proof. Refactor the existing cache-validity
  predicate so normal lookup and mutation lookup cannot drift.
- **Files/functions:** `crates/rum-core/src/expr.rs` (`SharedArgs`);
  `crates/rum-runtime/src/runtime/evaluation_cache/{mod.rs,stability.rs,
  mutation_proofs.rs}`; `runtime/model/cache.rs` only if the proof type belongs
  with `CallStabilityCacheEntry`; `evaluation_cache/tests.rs`.
- **Invariant:** removing an entry changes performance only; stale/dirty
  entries may be evicted but never yield a proof, and pointer-mismatched
  entries are untouched. Reinsertion uses the current generation and dirty
  snapshot. `SharedArgs::make_mut` records a `shared_args_alloc` exactly when
  COW allocates, preserving representation-profiler accounting. No raw
  pointer dereference and no `unsafe`.
- **Gate:** focused core/runtime cache tests, workspace tests, then a full
  microbench A/B. Target labels must remain statistically unchanged because
  no caller uses the new primitives yet; every existing hot label is a
  regression sentinel.
- **Expected win:** none alone; it removes the hidden blocker for later steps.
- **Workload delivered:** neither.
- **Organization:** `expr.rs` is 24,363 chars, but the methods belong on the
  representation type and are only a few lines; splitting that impl would be
  fragmentation. New cache policy goes in a dedicated module instead of
  growing `stability.rs` (15,539 chars).

### Step 3 — Fast-path discarded-result AppendTo on a plain symbol List

- **Change:** before the generic `current_held_update_value` path, attempt the
  conservative predicate from section 3. Take the exact stable-call proof
  (dropping the cache's strong `SharedArgs`), use `Arc::make_mut` on the list
  stored in the definition, reserve/push under a rollback guard, bump the
  symbol epoch, and reinsert the stable proof with the item's bloom unioned
  in. Return a shallow clone of the updated value. Any failed predicate or
  alias is semantically safe: `make_mut` copies aliases, while callback-capable
  states use the old evaluated fallback.
- **Files/functions:** keep only a routing call in
  `definitions/definition_updates.rs::eval_append_to`; place policy and
  mutation in new `definitions/incremental_append.rs`; register it in
  `definitions/mod.rs`. Widen the existing interrupt poll only to
  `pub(in crate::runtime)` in `eval_core/mod.rs` if the new module must call
  it.
- **Invariant:** old value remains observable on every callback-capable path;
  `AppendTo[x,Nothing]`, arbitrary heads, user-modified List behavior,
  upvalues, tracing/monitor callbacks, protection, suspension, resource
  failures, returned-history aliases, and `a=b` value semantics remain
  unchanged.
- **Gate:** focused mutation tests, a resumed-input test, workspace tests.
  A/B target: both `appendto-list-incremental-*` rows. Same-tier sentinels:
  `do-iterator-null`, `set-ownvalue`, `symbol-ownvalue-read`,
  `list-literal-small`, `part-list`, and both Association rows.
- **Expected win:** normal discarded-result list accumulation changes from
  O(n²) copying/stability rescans to amortized O(n) total; the large/small
  timing ratio should approach 2 rather than the observed roughly 3+.
- **Workload delivered:** flat list accumulation.
- **Organization:** `definition_updates.rs` is already 19,682 chars; new
  policy must live in `incremental_append.rs`, not push it into split-plan
  range.

### Step 4 — Extend the proved List path through one exact Part spine

- **Change:** for a single exact positional Part target, compute the path
  without retaining a cloned selected value. Take the leaf stable proof,
  evict cache entries for each ancestor, then `Arc::make_mut` each args vector
  from the definition root to the selected list. Append under the same undo
  guard and seed the leaf proof. Leave bulk/multi/association-key Part targets
  on the generic path.
- **Files/functions:** `definitions/incremental_append.rs`;
  the small route in `definition_updates.rs`; cache proof tests only if Step 2
  did not already cover ancestor eviction. Reuse
  `exact_position_paths_from_part_target_selector_exprs`; do not change the
  general `replace_exact_path`.
- **Invariant:** only the selected exact path changes. Duplicate/bulk path
  semantics, selection shape, negative indices, messages, and aliases remain
  on existing code. Copy-on-write preserves an aliased outer or inner value.
- **Gate:** focused single-Part, nested, alias, retained-result, invalid,
  bulk-fallback, and user-callback tests; workspace tests. A/B target:
  `appendto-part-outer1000-incremental-*`; flat AppendTo labels must not
  regress. `part-list`, `set-ownvalue`, and `do-iterator-null` are same-tier
  sentinels.
- **Expected win:** remove the observed O(m) outer rebuild per append for the
  unique exact-path case. At m=1000/n=200, the 0.004083551 s outer overhead
  should largely disappear; total should approach the flat row plus O(depth).
- **Workload delivered:** nested/Part list accumulation.

### Step 5 — Replace Association parse/render with direct canonical-Rule mutation

- **Change:** split Association lvalue ownership into a coherent
  `kernel_data/association_lvalue.rs`. For a one-key function-call target and
  `Part[...,Key[k]]`, validate/scan the already canonical top-level Rule
  vector directly. The direct route borrows the definition slot after all
  selector evaluation and does not call the cloning
  `association_lvalue_current`; only the old fallback takes that clone.
  Require and take an exact valid Association stability proof: its existing
  duplicate/Rule-form check establishes that the stored top level is already
  canonical, so the direct route need not parse or canonicalize it again.
  Require stable key/RHS blooms, use `SharedArgs::make_mut`, and replace one
  Rule or append one Rule under an undo guard. Reinsert a conservative updated
  Association proof by unioning the new Rule's bloom. Preserve existing key
  order and convert an updated `RuleDelayed` to immediate `Rule`.
  Missing/stale proof and nested/multi-key forms retain the existing recursive
  fallback initially.
- **Files/functions:** move the lvalue block out of
  `kernel_data/association_data.rs:332-573`; add/register
  `kernel_data/association_lvalue.rs` in `kernel_data/mod.rs`; use the Step-2
  cache eviction helper; extend
  `crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs`.
- **Invariant:** only associations covered by a currently valid proof of
  canonical, unique, stable Rule forms are directly mutated.
  Invalid `Association[...]` forms still fall through; missing nested keys
  still emit `Set::kval`; RHS/selectors finish evaluation before mutation;
  active resource constraints use the old path; Rule order/head behavior and
  alias semantics remain exact.
- **Gate:** focused Association lvalue tests, all Association tests, workspace
  tests. A/B target: both `association-set-key-incremental-*` rows. Same-tier
  sentinels: `set-ownvalue`, `part-list`, both AppendTo families,
  `symbol-ownvalue-read`, `list-literal-small`, and
  `association-evaluate-small-repeat`.
- **Expected win:** remove both quadratic canonicalization passes and all
  whole-value render clones. With a linear key scan retained, building n
  unique keys drops from measured near-O(n³) to O(n²), with a large constant
  reduction.
- **Workload delivered:** Association key accumulation gets its first,
  dominant win, but not its final asymptotic target.
- **Organization:** moving the lvalue block shrinks the 16,640-char
  `association_data.rs` rather than adding another responsibility to it.

### Step 6 — Add a bounded, self-invalidating Association position index

- **Change:** cache `Expr -> position` per mutated symbol in a transient,
  bounded Session side table. Each entry retains a strong `SharedArgs`
  identity witness as well as the key map. Take the entry out before
  `Arc::make_mut`, then validate it with the target symbol and
  `SharedArgs::ptr_eq` against the current canonical Association args. The
  retained witness prevents address reuse, so this is not an unguarded
  integer-pointer/ABA check. After validation, move out the key map and drop
  the witness before `Arc::make_mut`; otherwise the index would recreate the
  same refcount blocker found in section 2. A mismatch drops and rebuilds the
  map once by scanning canonical Rules. Update expression and index atomically
  under a rollback guard, then reinsert an identity witness for the new args.
  Bound both entry count and total stored key positions; an oversized
  Association is not indexed. Keep the current Vec duplicate scan at arity
  eight or below and use a HashSet/HashMap above it, so large Association
  stability becomes O(n) without imposing hashing on the common tiny case.
  The checked-in small sentinel is the regression gate for this boundary.
- **Files/functions:** `runtime/model/cache.rs` (index entry/map alias);
  `runtime/model/session.rs` (transient field);
  `session/session_state.rs` (initialization);
  `session/session_helpers.rs` and
  `evaluation_cache/stability.rs` (clear on restore/full invalidation);
  `debug_tools/resources/memory.rs` (account for the bounded retained map);
  `eval_core/stability.rs::association_data_stability_with_bloom` (hybrid
  small-Vec/large-HashSet duplicate detection);
  `kernel_data/association_lvalue.rs`; Association lvalue and cache tests.
- **Invariant:** the index is only an accelerator. Identity mismatch always
  rebuilds; collision handling uses normal `HashMap<Expr,...>` Eq;
  clearing/restore cannot expose stale positions; expression and index commit
  or roll back together. Generic definition writes need no mechanical
  invalidation hook: immutable args plus a strong identity witness make a
  pointer mismatch fail closed, while reassignment to the exact same args
  leaves the map valid.
- **Gate:** tests for append, replacement, duplicate prevention, generic
  reassignment, an unrelated write retaining a valid entry, Clear/reassign,
  aliasing, snapshot restore, forced cache clear, both bounds,
  oversized-value bypass, memory accounting, and complex equal keys;
  workspace tests. A/B target: both Association labels, with large/small
  ratio approaching 2. `association-evaluate-small-repeat`, AppendTo, and all
  core hot sentinels must not regress.
- **Expected win:** average O(1) lookup/mutation after the first index build,
  hence amortized O(n) total for n unique key inserts; a later read remains
  O(n).
- **Workload delivered:** final Association accumulation target.

The steps are intentionally separable. A failed A/B can revert one step
without losing the earlier semantic or measurement groundwork. A HAMT/tree
representation, a changed builtin ABI, and a global weak-cache conversion are
not prerequisites.

Rejected alternatives:

- Do not generalize `own_value.take()` across compound assignments: every
  operator in section 3 can run user code after the read, so this would expose
  an unset symbol and still would not remove the cache owner.
- Do not start with an Association hash index: it leaves both quadratic
  canonicalization passes in place, which the controlled probe identifies as
  the larger cost.
- Do not convert the whole stability cache to weak references: that changes
  hit lifetime and performance across the evaluator to solve two mutation
  paths. Exact mutation-time eviction is smaller and independently gateable.
- Do not introduce a HAMT/tree or change the builtin slice ABI: the current
  contiguous contract makes that a representation-wide project, while the
  measured workloads can be addressed with localized safe COW.

## 6. Microbench label proposals

Add these exact statements after the existing basic list/Part labels and
before unrelated matcher/string tiers:

```rum
rumMicroOnce[
  "appendto-list-incremental-small",
  rumMicroN[250],
  Module[{lst = {}},
    Do[AppendTo[lst, i], {i, 1, 250 * rumMicroScale}]
  ]
];
rumMicroOnce[
  "appendto-list-incremental-large",
  rumMicroN[500],
  Module[{lst = {}},
    Do[AppendTo[lst, i], {i, 1, 500 * rumMicroScale}]
  ]
];
rumMicroOnce[
  "appendto-part-outer1000-incremental-small",
  rumMicroN[100],
  Module[{outer = Table[{}, {1000}]},
    Do[AppendTo[outer[[1]], i], {i, 1, 100 * rumMicroScale}]
  ]
];
rumMicroOnce[
  "appendto-part-outer1000-incremental-large",
  rumMicroN[200],
  Module[{outer = Table[{}, {1000}]},
    Do[AppendTo[outer[[1]], i], {i, 1, 200 * rumMicroScale}]
  ]
];
rumMicroOnce[
  "association-set-key-incremental-small",
  rumMicroN[100],
  Module[{assoc = <||>},
    Do[assoc[i] = i, {i, 1, 100 * rumMicroScale}]
  ]
];
rumMicroOnce[
  "association-set-key-incremental-large",
  rumMicroN[200],
  Module[{assoc = <||>},
    Do[assoc[i] = i, {i, 1, 200 * rumMicroScale}]
  ]
];
rumMicroOnce[
  "association-evaluate-small-repeat",
  rumMicroN[500],
  Do[<|a -> i, b -> 2|>, {i, 1, 500 * rumMicroScale}]
];
```

These use `rumMicroOnce` because each label is one stateful loop. The
expression is the exact statement; it has no added sink, `Print`, or
per-iteration result assignment. `Module` resets mutation state independently
for every repetition. The last label is a small-Association stability
sentinel for Step 6; varying `i` prevents every iteration from being only a
pointer-identical cache hit. The existing helper supplies min-of-seven
`AbsoluteTiming`.

The diagnostic minima imply roughly 0.28 seconds plus the tiny stability
sentinel of additional timed work at seven repetitions before warmup/driver
overhead, safely inside the current 180-second suite budget. Small/large
pairs make complexity changes visible without a second ad-hoc timing harness.

## 7. Draft follow-up implementation briefs

These are drafts in `TEMPLATE.md` shape. They are raw material for review and
rewrite, not ready for direct dispatch.

### Draft brief 291A — add incremental container microbench labels

#### Goal

Add stable baseline labels for flat AppendTo accumulation, exact-Part
AppendTo accumulation with a 1000-element outer list, and key-by-key
Association construction, plus a repeated tiny-Association stability
sentinel. Change no runtime behavior.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, and
  `docs/dev/codex/plan-291-assoc-append-ownership.md` sections 1 and 6.
- Key file: `scripts/core-language-microbench.sh`.
- Preserve its `AbsoluteTiming`, min-of-repetitions, scaling, raw-output, and
  profile-disable machinery.

#### Scope

- Files you MAY modify:
  - `scripts/core-language-microbench.sh` — add exactly the seven plan labels.
- Files you MUST NOT modify: every Rust file, `Cargo.*`, all docs, all other
  scripts, git.
- There is no signature ripple.

#### Deliverable

- The seven exact `rumMicroOnce` labels from plan section 6.
- One baseline output directory under `/tmp`.
- Verbatim seven target/sentinel rows plus the named same-tier neighbor rows.

#### Verification

```text
bash -n scripts/core-language-microbench.sh
scripts/run-bounded.py --timeout 240 -- \
  bash scripts/core-language-microbench.sh \
    --output-dir /tmp/p291-label-baseline \
    --rum-cli ./target/release/rum-cli
rg 'appendto-|association-set-key-|do-iterator-null|set-ownvalue|part-list|module-local|list-literal-small|symbol-ownvalue-read' \
  /tmp/p291-label-baseline/timings.tsv
git diff --check
```

Paste the rows verbatim. State the total run time from `runs.tsv`.

#### Hard rules

- Do not commit, branch, push, touch git config, use network, or modify
  outside Scope.
- Use no external oracle. Profiling variables remain disabled for timing.
- Do not add a sink or wrapper inside the timed expression.
- Do not change existing labels, counts, repetitions, clock source, or parser.
- Stop if the added labels make the suite exceed its existing timeout.

#### Report format

1. One-line script change.
2. Verbatim verification and target/neighbor rows.
3. Added runtime budget.
4. Open questions or stop-loss.

### Draft brief 291B — exclusive SharedArgs and stability-proof primitives

#### Goal

Provide safe, internal primitives for copy-on-write mutation of `SharedArgs`
and for removing a pointer-matched call-stability cache entry, conditionally
transferring its proof when valid, and reinserting an updated proof without
changing evaluator behavior or normal-cache semantics.

#### Context pointers

- Read `AGENTS.md`, code-organization guidance, testing strategy, and plan
  sections 2, 4, and Step 2.
- Key files: `rum-core/src/expr.rs`,
  `evaluation_cache/stability.rs`, `model/cache.rs`.
- The cache currently pins `SharedArgs`; normal hit validity and mutation
  proof validity must be one predicate.

#### Scope

- Files you MAY modify/create:
  - `crates/rum-core/src/expr.rs` — safe `SharedArgs::get_mut` and
    `SharedArgs::make_mut` only.
  - `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs` — register the
    new module.
  - `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs`
    (new) — mutation-only take/reinsert/path-evict API.
  - `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs` — factor
    and reuse entry-validity/dirty logic; enumerate and update both call sites.
  - `crates/rum-runtime/src/runtime/model/cache.rs` — internal proof type only
    if required.
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` — focused
    cache ownership/validity tests.
- No public builtin/API signature changes. No other caller may use the new
  primitives in this brief.
- MUST NOT modify stdlib behavior, definitions, Association code, scripts,
  docs, Cargo files, or git.

#### Deliverable

- Safe methods only; no `unsafe` and no raw-pointer dereference.
- `make_mut` preserves `RUM_PROFILE_REPR` allocation accounting: no count for
  the unique case, one `shared_args_alloc` count for COW.
- Tests proving: a strong cache entry blocks uniqueness; taking the exact
  valid entry releases it; a dirty/generation-stale but pointer-matched entry
  is evicted without yielding a proof; a pointer mismatch is neither removed
  nor proved; reinsertion hits with the current bloom; path eviction removes
  only named nodes; unique versus aliased `make_mut` produces the expected
  pointer identity and allocation count.
- Split assessment for the touched 24K `expr.rs`.
- Before/after full microbench rows.

#### Verification

```text
cargo fmt --check
cargo test -p rum-core
RUM_PROFILE_REPR=1 cargo test -p rum-core shared_args_make_mut_profiles_cow -- --test-threads=1
cargo test -p rum-runtime evaluation_cache
cargo test --workspace
scripts/run-bounded.py --timeout 240 -- \
  bash scripts/core-language-microbench.sh --output-dir /tmp/p291b-new
```

Run the same script on the exact base build into `/tmp/p291b-base`. Paste all
seven new labels and every existing label that changes by more than noise;
paste the named core neighbors unconditionally.

#### Hard rules

- No git/network/out-of-scope writes; no external engine.
- No `unsafe`; no public serde/type representation change.
- No new normal-path allocation, lock, or diagnostic branch. Mutation APIs
  are cold until called.
- Cache validity must not be duplicated with subtly different logic.
- Any hot-label regression is a defect. Stop after two failed attempts for
  the same cause.

#### Report format

1. Per-file change and split assessment.
2. Verbatim tests and A/B rows.
3. Proof that runtime behavior has no new caller.
4. Stop-loss/open questions.

### Draft brief 291C — safe flat-list AppendTo fast path

#### Goal

Make discarded-result `AppendTo` accumulation on an ordinary, unaliased
symbol List amortized linear while preserving every callback, normalization,
resource, suspension, protection, and value-semantics case through a
conservative fallback.

#### Context pointers

- Read plan sections 2–5 and the 291B cache API.
- Key paths: `definition_updates.rs::eval_append_to`,
  argument preparation, stable-call cache, and store-time protection guard.
- Reproduce the upvalue witness, alias witness, and tracing/Monitor behavior
  before editing.

#### Scope

- Files you MAY modify/create:
  - `crates/rum-runtime/src/runtime/definitions/mod.rs` — register one module.
  - `crates/rum-runtime/src/runtime/definitions/incremental_append.rs` (new)
    — predicate, rollback guard, and symbol-list mutation.
  - `crates/rum-runtime/src/runtime/definitions/definition_updates.rs` —
    route the single `eval_append_to` call site; no other operator changes.
  - `crates/rum-runtime/src/runtime/eval_core/mod.rs` — only widen
    `poll_host_interrupt_if_due` to `pub(in crate::runtime)` if required.
  - `crates/rum-stdlib/src/tests/incidental_definitions_updates.rs` — semantic
    regressions.
  - `crates/rum-stdlib/src/tests/session_requests/functional_sequence_resume.rs`
    — AppendTo item suspension/resume regression.
- All cache primitive signatures and call sites are fixed by 291B; if another
  file is needed, stop and report the missing scope.
- MUST NOT modify PrependTo/other operators, Part handling, Association code,
  docs, scripts, Cargo files, or git.

#### Deliverable

- Exact conservative predicate from plan section 3.
- Safe rollback of a pushed element; capacity reservation precedes mutation.
- Fallback tests for `Nothing`, a matching List upvalue, an arbitrary current
  head/downvalue, modified List metadata, active constraint, input
  suspension, enabled tracing, and an enabled Monitor callback.
- Value tests for `alias=lst`, retained `Table[AppendTo[...]]` history, and
  protection.
- A/B showing target improvement with no neighbor regression.

#### Verification

```text
cargo fmt --check
cargo test -p rum-stdlib held_update
cargo test -p rum-stdlib resume_eval_request_with_input
cargo test --workspace
scripts/run-bounded.py --timeout 240 -- \
  bash scripts/core-language-microbench.sh --output-dir /tmp/p291c-new
```

Run an identical base into `/tmp/p291c-base`. Paste both flat AppendTo rows;
also paste `do-iterator-null`, `set-ownvalue`, `symbol-ownvalue-read`,
`list-literal-small`, `part-list`, both Association build rows, and
`association-evaluate-small-repeat`.

#### Hard rules

- No git/network/out-of-scope writes; no external oracle; no `unsafe`.
- Never remove the own value across evaluation/suspension/message/resource
  work.
- If the no-callback predicate is not proved, fall back.
- Preserve `AppendTo[x,Nothing]`, upvalue/downvalue behavior, store-time
  protection, tracing/Monitor events, and copy-on-write aliases exactly.
- Hot-path A/B is mandatory. Stop if the large row does not materially move
  after two attempts.

#### Report format

1. Per-file mechanism.
2. Verbatim semantic/suspension tests.
3. Target and neighbor A/B rows with ratios.
4. Fallback cases and stop-loss.

### Draft brief 291D — exact-Part List AppendTo fast path

#### Goal

Extend the proven 291C List append to one exact positional Part target without
rebuilding the outer container, while leaving every bulk/complex Part form on
the existing implementation.

#### Context pointers

- Read plan Finding E, Step 4, exact-path helpers, and 291C.
- Key code:
  `definition_updates.rs::current_part_held_update_value`,
  `structure_data/exact_paths.rs`, and cache path eviction from 291B.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/incremental_append.rs` —
    exact-path planning and spine COW mutation.
  - `crates/rum-runtime/src/runtime/definitions/definition_updates.rs` —
    route exact Part to the helper.
  - `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs` and
    `evaluation_cache/tests.rs` only if ancestor eviction was explicitly left
    unimplemented by 291B.
  - `crates/rum-stdlib/src/tests/incidental_definitions_updates.rs` — Part
    semantic/fallback tests.
- Existing helper signatures remain unchanged. Do not modify
  `replace_exact_path`.
- MUST NOT modify bulk Part behavior, Association, scripts/docs/Cargo/git, or
  other mutation operators.

#### Deliverable

- Unique-spine mutation for a single exact positional path.
- Tests for positive/negative indices, nested depth, outer alias, inner alias,
  retained results, invalid selector, empty/bulk/multi fallback, matching
  upvalue fallback, and protection.
- A/B on both nested labels and flat regression labels.

#### Verification

```text
cargo fmt --check
cargo test -p rum-stdlib held_updates_mutate_single_exact_part_targets
cargo test -p rum-stdlib held_updates_mutate_bulk_exact_part_targets
cargo test --workspace
scripts/run-bounded.py --timeout 240 -- \
  bash scripts/core-language-microbench.sh --output-dir /tmp/p291d-new
```

Compare with `/tmp/p291d-base`; paste both Part rows, both flat rows,
`part-list`, `set-ownvalue`, and `do-iterator-null`.

#### Hard rules

- No git/network/out-of-scope writes, external oracle, or `unsafe`.
- Do not clone a selected child merely to plan the fast path.
- Any non-single-exact selector falls back unchanged.
- Roll back the leaf on unwind; never expose a missing definition.
- A regression on flat AppendTo or Part neighbors rejects the step.

#### Report format

1. Exact fast-path boundary.
2. Verbatim tests.
3. Nested and flat A/B rows.
4. Cases intentionally left on fallback.

### Draft brief 291E — direct canonical Association lvalue mutation

#### Goal

Remove the parse/canonicalize/render round trip from a single-key Association
lvalue update by mutating the canonical Rule vector with copy-on-write,
without yet adding a hash index or changing nested-key behavior.

#### Context pointers

- Read plan Finding C, failure story, and Step 5.
- Key code:
  `association_data.rs:27-39,179-274,332-573`,
  `definitions/definitions.rs:14-51,318-339`, and the 291B eviction API.
- A valid immediate Association RHS is evaluated/canonicalized by
  `rum-stdlib/src/associations/basic.rs:7-59` before Set stores it.

#### Scope

- Files you MAY modify/create:
  - `crates/rum-runtime/src/runtime/kernel_data/association_data.rs` — move the
    lvalue block and retain data parse/render helpers.
  - `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs` (new) —
    moved lvalue flow, direct single-key updater, rollback guards, existing
    recursive fallback.
  - `crates/rum-runtime/src/runtime/kernel_data/mod.rs` — register/re-export
    the new module.
  - `crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs` —
    semantic and alias cases.
- The moved functions/call sites are:
  `try_set_association_lvalue`,
  `normalize_association_set_lvalue_target`,
  `association_lvalue_current`,
  `resume_association_set_lvalue_selector_evaluation`,
  `finish_association_set_lvalue`,
  `set_association_part_path`,
  `association_part_key_path`,
  `association_part_key_selector`,
  `set_association_key_path`, and
  `upsert_immediate_association_entry`.
- MUST NOT modify Association stdlib construction, generic parse/render
  semantics, nested-key semantics, scripts/docs/Cargo/git, or other Set paths.

#### Deliverable

- A verbatim responsibility move where behavior is unchanged, plus the
  isolated single-key direct updater. The direct updater must borrow the
  definition-owned value after selector evaluation and must not retain the
  `association_lvalue_current` clone; the recursive fallback keeps the clone.
- Take a currently valid Association stability proof before mutation; use it
  as the canonical/unique Rule-form certificate, then reinsert a conservative
  proof with the stable key/RHS bloom unioned in. Missing/stale proof falls
  back.
- Tests for insert, replace/order, RuleDelayed-to-Rule, function and Key Part
  forms, invalid Association fallback, missing nested key, selector side
  effects, protected target, active resource constraints, allocation
  preflight/rollback, interrupt polling, absent/stale/dirty proof fallback,
  proof reinsertion, and the required `a`/`b` alias contract.
- Before/after small/large Association rows and scaling ratio.

#### Verification

```text
cargo fmt --check
cargo test -p rum-stdlib associations_set_lvalue
cargo test -p rum-stdlib associations
cargo test --workspace
scripts/run-bounded.py --timeout 240 -- \
  bash scripts/core-language-microbench.sh --output-dir /tmp/p291e-new
```

Compare with `/tmp/p291e-base`. Paste both Association rows, all four
AppendTo rows, `set-ownvalue`, `part-list`, `symbol-ownvalue-read`, and
`list-literal-small`, plus `association-evaluate-small-repeat`.

#### Hard rules

- No git/network/out-of-scope writes, external oracle, or `unsafe`.
- RHS and every selector finish evaluation before mutation.
- An active `current_time_deadline` or `current_memory_limit` uses the old
  path; poll interrupt/timeout and prebuild/reserve all new storage before
  mutation.
- Never bypass parse/canonicalize without a currently valid proof that every
  top-level argument is a unique Rule/RuleDelayed form.
- Do not add an index in this brief: one variable per A/B.
- Direct mutation is valid only for canonical top-level Rule forms; otherwise
  fallback.
- Do not clone the current Association on the direct path; safe COW alone
  preserves genuine semantic aliases.
- Preserve order, messages, delayed-head conversion, and aliases exactly.
- If the target rows do not move materially, stop rather than adding a second
  optimization.

#### Report format

1. Split map and per-file change.
2. Verbatim semantic tests.
3. Association and neighbor A/B rows plus scaling.
4. Fallback boundary/open questions.

### Draft brief 291F — bounded Association key-position index

#### Goal

After direct mutation is proven, eliminate its remaining linear scan with a
bounded, fail-closed position index and make Association stability duplicate
checking linear, yielding amortized-linear unique-key accumulation.

#### Context pointers

- Read plan Step 6, 291E, cache invalidation code, Session initialization and
  snapshot restore.
- The index is transient accelerator state, not language state and not part
  of snapshots/serde.
- `Expr` Hash and Eq define key identity; no custom comparison semantics.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model/cache.rs` — internal index entry/map
    types and cap metadata.
  - `crates/rum-runtime/src/runtime/model/session.rs` — one transient Session
    field.
  - `crates/rum-runtime/src/runtime/session/session_state.rs` — initialize it.
  - `crates/rum-runtime/src/runtime/session/session_helpers.rs` — clear it
    during loader/snapshot restore.
  - `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs` — clear it
    on full cache invalidation.
  - `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs` — include
    the retained identity witnesses and key maps in the memory proxy.
  - `crates/rum-runtime/src/runtime/eval_core/stability.rs` — replace only the
    Association duplicate-key Vec scan with HashSet/HashMap logic.
  - `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs` — take,
    validate, rebuild, update, and reinsert index atomically.
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` and
    `crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs` —
    cache/integration regressions.
- Enumerated initialization/clear/memory-accounting ripple is complete; do
  not add mechanical invalidation to every own-value writer. A strong
  `SharedArgs` identity witness plus `SharedArgs::ptr_eq` is the validity
  contract.
- MUST NOT change Expr/SharedArgs representation, public Session API,
  snapshots, docs/scripts/Cargo/git, or nested Association semantics.

#### Deliverable

- Per-symbol index carrying a strong args identity witness; validation uses
  `SharedArgs::ptr_eq`, then drops that witness before COW mutation.
- Explicit caps on both indexed-symbol count and total stored key positions;
  oversized values are not cached.
- Combined expression/index rollback with all capacity reservations before
  mutation.
- Thresholded duplicate detection in Association stability: retain the
  current Vec path through arity eight and use a HashSet/HashMap above it.
- Tests for cache hit/rebuild, unrelated writes retaining a valid entry,
  target reassignment identity mismatch, Clear/reassign, restore, both caps,
  oversized-value bypass, equal/complex keys, replacement/order, aliases,
  and missing keys.
- A/B and small/large scaling evidence.

#### Verification

```text
cargo fmt --check
cargo test -p rum-runtime evaluation_cache
cargo test -p rum-stdlib associations_set_lvalue
cargo test -p rum-stdlib associations
cargo test --workspace
scripts/run-bounded.py --timeout 240 -- \
  bash scripts/core-language-microbench.sh --output-dir /tmp/p291f-new
```

Compare with `/tmp/p291f-base`. Paste both Association rows and their
large/small ratios, `association-evaluate-small-repeat`, all four AppendTo
rows, and the core neighbor set.

#### Hard rules

- No git/network/out-of-scope writes, external oracle, or `unsafe`.
- Identity mismatch always rebuilds. Never trust a numeric pointer key
  without the live strong witness that prevents address reuse.
- Drop the taken identity witness before `Arc::make_mut`; the index must not
  become another hidden strong owner.
- Index state is transient and bounded; no snapshot/public-state expansion.
- Hash collisions must fall through `Expr::eq`.
- Keep arity eight and below on the Vec path; do not regress the checked-in
  tiny-Association sentinel to obtain the large-value win.
- No normal evaluator-path index lookup; cost is paid only by Association
  mutation and Association stability.
- Reject any same-tier regression. Stop after two failures with one cause.

#### Report format

1. Cache validity and bound.
2. Verbatim tests.
3. Association scaling and all A/B rows.
4. Remaining limits/stop-loss.

## 8. Risks and open questions

| Risk or open question | Current position | Evidence that would settle it |
|---|---|---|
| The conservative flat-List predicate may disable the win in sessions with any unrelated upvalue. | Correctness-first fallback is acceptable for the target probe, which has no upvalues. Do not weaken it in the first implementation. | Add env-gated hit/fallback-reason counters only if the target A/B unexpectedly misses; run rum-vs-rum workloads with representative upvalue registries. A later design could use a symbol/upvalue bloom. |
| Incrementally reusing a stability proof could miss a shape-dependent evaluator effect. | Only core, unmodified List with no upvalues, no `Nothing`, stable current/item, no active `current_time_deadline`/`current_memory_limit`, and no observable trace/Monitor callback qualifies. All other shapes evaluate normally. | Unit-test each predicate negation plus the exact `List[u,1]` upvalue witness, tracing, and Monitor callbacks. Run the full workspace and target A/B. |
| `Arc::make_mut` might still copy in the nominal unique case because an unenumerated strong holder remains. | Removing the exact stability entry should leave the definition as sole owner in discarded-result `Do`; other holders are intentional COW cases. | Step-2 pointer-identity/get_mut tests and Step-3 A/B. If the flat row does not move, add `RUM_PROFILE_REPR`-gated mutation-path counts rather than guessing. |
| Cache proof reinsertion could make a mutated expression stable under stale generation/bloom data. | Reinsertion must use current generation/dirty snapshot and union the item bloom; normal and mutation validity share one helper. | Dirty-symbol, rewritten-dirty, generation, pointer mismatch, and newly defined upvalue tests in 291B/291C. |
| Direct Part mutation could alter bulk/duplicate selection semantics. | Scope is one exact positional path only; everything else falls back. | Test-name coverage for existing single/bulk/duplicate/empty/invalid Part cases and explicit route counters under test configuration. |
| The transient Association index is itself a strong owner and can pin old values. | Take, validate, and drop its `SharedArgs` witness before COW. Bound both indexed symbols and total key positions; bypass an oversized value. Generic reassignment is safe because the live witness prevents pointer ABA and identity mismatch rebuilds. | Pointer-identity/strong-count unit tests, target-reassignment and unrelated-write tests, both cap tests, memory-proxy tests, and a workload whose RHS writes another symbol. |
| Hashing large symbolic keys can be O(key size) and the runtime has a deterministic hasher. | This is still better than scanning all entries; collision resolution preserves Eq semantics. It does not promise adversarial constant time. | Benchmark integer, short-string, and large-expression keys; inspect collision distributions. A hasher change is out of scope. |
| Final Association read remains expensive unless stability duplicate detection is fixed. | Step 6 explicitly makes that pass linear. | Add a label or focused timing for one read of a built association if Step-6 target rows improve but end-to-end use does not. |
| Nested Association key-path updates remain round-trip based after the single-key fast path. | Not needed for the reported `a[k]=v` workload; preserving the old fallback keeps this plan small. | Profile a dedicated nested-key label after Steps 5–6. Promote only if it is a real remaining target. |
| Panic rollback after mutation but before epoch/cache bookkeeping. | Mutation itself is guarded and contains no user callback. Structure bookkeeping immediately after commit and do not call fallible evaluation there. | Add test-only injected panics at pre-push, post-push/pre-commit, and index/expression mutation points; catch in a unit test and inspect the definition/index. No production panic hook or default-path branch. |
| A tree/HAMT might eventually outperform COW for alias-heavy histories. | Rejected here because the builtin slice/Deref ABI makes it a broad representation project, and alias-heavy history must preserve old values anyway. | Only revisit with profiles showing the planned COW/index path still dominated after all six gates, plus a separate ABI/representation design brief. |
