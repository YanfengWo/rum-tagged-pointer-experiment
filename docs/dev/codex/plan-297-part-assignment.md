# Plan 297 — positional `Part` assignment is currently a failing definition write

This is a design-only result. No Rust, scripts, generated reference, or
existing documentation was changed.

## Executive verdict

`Set[Part[sym, i], rhs]` does **not** currently assign a positional part.
After evaluating `rhs`, `Set` misses the Association-only lvalue handler,
expands every symbol under the held lhs as if it were normalizing a definition
lhs, rebuilds the whole value of `sym`, classifies the rebuilt expression as a
DownValue definition for `System`Part`, and rejects that definition because
`Part` is `Protected`. The measured O(m) work is therefore the cost of
rebuilding a failing lhs, not the cost of a working immutable-container
update.

I recommend fixing this, but treating it first as a missing language feature
and only second as an optimization. The safe sequence is:

1. establish and implement a narrow, documented, correct exact-positive
   positional `Part` lvalue path, initially using the generic rebuilding
   helper;
2. add checked-in measurements for that now-working behavior; and
3. replace the generic rebuild with the already-proved cache eviction plus
   copy-on-write spine mechanism.

This staging prevents a meaningless speedup claim against today's failed
assignments.

## 1. Verification of M1–M3

### Method

I used the current release `rum-cli`, whose build is newer than every runtime
source file touched by briefs 293–296. Every timing used the exact bare
`AbsoluteTiming[Do[...]]` statement: the container initialization was outside
the timed expression, no sink was added, and all `RUM_PROFILE_*` variables
were explicitly disabled. Each shape ran in three fresh processes through
`scripts/run-bounded.py --timeout 10`; the bound is over three orders of
magnitude above the brief's measured millisecond-scale workload. The tables
report all three runs and their median. No network or external engine was
used.

One semantic fact changes how every timing below must be read: each
`L[[...]] = ...` iteration emits `Set::wrsym: Symbol Part is Protected` and
leaves `L` unchanged. Redirecting the rendered diagnostics away from the
terminal did not disable their construction inside the timed evaluation.

### M1 — fixed operation count, varying container length

Probe:

```text
L = Table[0, {m}];
AbsoluteTiming[Do[L[[1]] = i, {i, 1, 100}]]
```

| m | run 1 | run 2 | run 3 | median | relative |
|---:|---:|---:|---:|---:|---:|
| 100 | 0.466086 ms | 0.498741 ms | 0.614134 ms | **0.498741 ms** (`498741/1000000000`) | 1.000x |
| 200 | 0.728288 ms | 0.782721 ms | 0.730576 ms | **0.730576 ms** (`45661/62500000`) | 1.465x |
| 400 | 1.274416 ms | 1.353693 ms | 1.339633 ms | **1.339633 ms** (`1339633/1000000000`) | 2.686x |
| 800 | 2.360605 ms | 2.432356 ms | 2.448089 ms | **2.432356 ms** (`608089/250000000`) | 4.877x |

Control:

```text
A = Table[0, {m}];
AbsoluteTiming[Do[AppendTo[A, i], {i, 1, 100}]]
```

| m | run 1 | run 2 | run 3 | median |
|---:|---:|---:|---:|---:|
| 100 | 0.133135 ms | 0.136848 ms | 0.134172 ms | **0.134172 ms** (`33543/250000000`) |
| 800 | 0.145373 ms | 0.150287 ms | 0.148758 ms | **0.148758 ms** (`74379/500000000`) |

The 8x larger initial AppendTo container costs 1.109x (+10.9%), while the
failed `Part`-Set path costs 4.877x. M1's scaling is confirmed. Claude's
representation-level conclusion was directionally right: a contiguous
`Arc<Vec<Expr>>` does not inherently require a whole-vector copy for a unique
single-slot change. The stronger finding is that the two probes do not
currently perform comparable operations: AppendTo mutates, while positional
Set rebuilds a rejected lhs.

### M2 — the failed fill loop is quadratic

Probe:

```text
qL = Table[0, {n}];
AbsoluteTiming[Do[qL[[i]] = i, {i, 1, n}]]
```

| n | run 1 | run 2 | run 3 | median |
|---:|---:|---:|---:|---:|
| 200 | 1.512975 ms | 1.438353 ms | 1.397253 ms | **1.438353 ms** (`1438353/1000000000`) |
| 400 | 5.156425 ms | 5.149448 ms | 5.185793 ms | **5.156425 ms** (`206257/40000000`) |

The same-run-shape median quotient is **3.585x** for 2x n, confirming M2's
quadratic accumulation. It is n attempts times a full normalization copy of
the n-element unchanged list, not n successful element writes.

### M3 — no `SharedArgs::into_vec`, about two call-vector allocations per attempt

The clean process-exit snapshot for the exact n=200 probe was:

```text
Null
rum-profile repr into_vec_total=0 into_vec_clone=0 into_owned_total=0 into_owned_clone=0 shared_args_alloc=417 shared_expr_alloc=418
```

The brief's exact 427/425 allocation totals differ by small probe/reporting
overhead, but both material facts are confirmed:

- `into_vec_total=0` and `into_vec_clone=0`; and
- call-vector/call-head allocations are approximately 2 per attempt.

The approximately 400 per-iteration allocations are the rebuilt current List
and then the rebuilt outer Part lhs. Setup, loop, and diagnostic expression
machinery account for the small remainder. No vector is consumed through
`SharedArgs::into_vec`; the copy site iterates a borrowed `SharedArgs`, clones
each `Expr` into a fresh `Vec`, and passes it to `Expr::call`.

### The semantic check the original measurements omitted

The release CLI produced:

```text
a = Table[0, {3}]; Do[a[[i]] = i, {i, 1, 3}]; a
! Set::wrsym: Symbol Part is Protected.
! Set::wrsym: Symbol Part is Protected.
! Set::wrsym: Symbol Part is Protected.
List[0, 0, 0]

a = {1, 2}; t = a[[1]]; a[[1]] = a[[2]]; a[[2]] = t; a
! Set::wrsym: Symbol Part is Protected.
! Set::wrsym: Symbol Part is Protected.
List[1, 2]

m = {{1, 2}, {3, 4}}; m[[2, 1]] = 9; m
! Set::wrsym: Symbol Part is Protected.
List[List[1, 2], List[3, 4]]

x = {1, 2, 3}; y = x; y[[1]] = 9; {x, y}
! Set::wrsym: Symbol Part is Protected.
List[List[1, 2, 3], List[1, 2, 3]]
```

Claude was wrong first about `replace_exact_path`, as the brief already says,
and also about the premise that a working positional assignment was being
timed. `replace_exact_path`'s runtime assignment caller remains compound
updates at
`crates/rum-runtime/src/runtime/definitions/definition_updates.rs:443-456`;
plain `Set` never enters that module.

## 2. Actual execution path and exact copy site

The complete ordinary path is:

1. `Session::evaluate` enters the fixed-point evaluator at
   `crates/rum-runtime/src/runtime/resolution.rs:291-305`, and an unstable call
   reaches `evaluate_once_unstable_after_input_check_with_cycle_tracking` at
   `resolution.rs:321-346`.
2. `evaluate_inner_unstable` recognizes a call and invokes `evaluate_call` at
   `crates/rum-runtime/src/runtime/eval_core/mod.rs:100-120`.
3. `evaluate_call` evaluates the Set head and builds/executes its dispatch
   plan at `crates/rum-runtime/src/runtime/eval_core/call.rs:4-18,57-77`.
   `Set` is registered as a special form at
   `crates/rum-runtime/src/runtime/builtin_catalog/rules.rs:94-99`, so
   `build_call_dispatch_plan` returns a one-stage special-form plan at
   `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:4-13`.
4. The special-form stage passes the raw arguments directly at
   `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:72-83`; builtin
   dispatch calls the adapter at `eval_dispatch/mod.rs:122-149`, and
   `builtin_set` calls `Session::eval_set` at
   `crates/rum-runtime/src/runtime/builtin_adapters.rs:143-149`.
5. `eval_set` clones the held lhs, evaluates the rhs to a fixed point, and
   resumes assignment at
   `crates/rum-runtime/src/runtime/definitions/definitions.rs:14-51,115-117`.
   This is the only ordinary evaluator call that is guaranteed before
   lvalue handling.
6. `finish_definition_assignment` rejects the symbol-own-value fast shape at
   `definitions.rs:257-265`, then tries list destructuring and the dedicated
   Association lvalue handler at `definitions.rs:318-339`.
   `try_set_association_lvalue` recognizes the syntactic Part target, but
   `association_lvalue_current` requires an `Association` head and returns
   `None` for a List at
   `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:389-402,413-442`.
7. With no positional lvalue handler, Set falls through to definition-lhs
   normalization at `definitions.rs:341-349`.
   `normalize_definition_lhs_call_heads` recursively normalizes the entire
   expression at `definitions.rs:472-475`; despite its name, the generic call
   branch recursively visits the call's arguments too at
   `definitions.rs:552-563`.
8. When that recursion reaches the symbol `L`, the symbol branch calls
   `resolve_definition_lhs_head_value` at `definitions.rs:479-484`. That
   function clones `L`'s own value and recursively normalizes it at
   `definitions.rs:576-600`.
9. The List call is copied here:

   ```rust
   let normalized_args = args
       .iter()
       .map(|arg| self.normalize_definition_lhs_expr(arg, ...))
       .collect();
   Expr::call(normalized_head, normalized_args)
   ```

   The borrowed iteration and fresh collection are
   `definitions.rs:552-563`; `Expr::call` allocates a new `SharedExpr` and
   `SharedArgs` at `crates/rum-core/src/expr.rs:729-733`. The same generic
   branch then rebuilds the enclosing `Part` call. This explains both the
   O(m) atom-copy volume and the roughly two `shared_args_alloc` /
   `shared_expr_alloc` events per attempt, while bypassing
   `SharedArgs::into_vec` at `crates/rum-core/src/expr.rs:580-590`.
10. The rebuilt lhs is classified as a DownValue target for its head at
    `definitions.rs:348-378` and
    `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:277-289`.
    Because that head is `System`Part, the protected-write check rejects it
    and returns the already-evaluated rhs. The message is emitted by
    `crates/rum-runtime/src/runtime/attributes.rs:212-227`. No definition of
    `L` is changed.

The direct answer to “what copies the Vec, if not `into_vec`?” is therefore:
the `.iter().map(...).collect::<Vec<Expr>>()` in
`definitions.rs:556-563`, followed by `Expr::call`. It is a borrowed deep
lhs-normalization rebuild, not an ownership-unwrapping rebuild.

## 3. Why Steps 3–4 do not cover this, and how close they are

They do not cover the live execution path at all. The only caller of
`try_incremental_append_to_plain_list` is `eval_append_to` at
`crates/rum-runtime/src/runtime/definitions/definition_updates.rs:121-159`.
`Set` is a separate special-form path through `definitions.rs`, and there is
no call from it into `definition_updates.rs` or `incremental_append.rs`.
Briefs 294 and 295 explicitly scoped the mechanism to AppendTo, so this
operator boundary was deliberate; the missing positional Set lvalue was a
separate unobserved feature gap, not a missed branch inside Step 4.

Mechanically, however, Step 4 is close to the eventual fast implementation:

- `incremental_append_target` already recognizes a symbol or
  `Part[symbol,...]` at
  `crates/rum-runtime/src/runtime/definitions/incremental_append.rs:204-217`.
- `incremental_append_plain_list_plan` already validates a positive exact
  selector spine, normalizes it to zero-based indexes, and gathers ancestor
  call-cache keys at `incremental_append.rs:92-154`.
- `exact_plain_list_leaf_mut` already walks a List spine with
  `SharedArgs::make_mut` at `incremental_append.rs:223-242`.
- the generic rollback shape is already present in
  `DefinitionValueRollback` at `incremental_append.rs:244-275`.
- pointer-matched cache eviction already exists at
  `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs:98-124`.

The positional operation differs at the endpoint: it needs the parent List's
mutable args and a guarded `mem::replace` of one slot, while AppendTo descends
to the selected List and pushes into that selected List's args.

It also differs in stability-proof policy. AppendTo bypasses evaluation of a
newly enlarged call, so it needs a valid proof and strict List/upvalue
conditions before safely reinserting an updated proof. Positional Set should
perform a structural store after all rhs/selector evaluation and should not
evaluate the rebuilt List at all. It can therefore evict pointer-matched
parent/ancestor entries and leave them evicted; it should **not** claim that
an old stable call remains stable after an arbitrary argument replacement.
Subsequent evaluation will repopulate the cache normally. Repeated held Set
operations remain exclusive because no cache entry is recreated merely by
storing a part.

## 4. Reachability

### User-facing idioms

| Idiom | Shape | Current result |
|---|---|---|
| Array fill | `Do[a[[i]] = f[i], {i, 1, n}]` | Every rhs evaluates, every write emits `Set::wrsym`, `a` is unchanged. |
| Element swap | `t=a[[i]]; a[[i]]=a[[j]]; a[[j]]=t` | Reads work; both writes fail, so no swap occurs. |
| Matrix/tensor update | `m[[i,j]] = v` | The full nested List lhs is normalized/rebuilt, then rejected. |
| Aliased update | `x={1,2,3}; y=x; y[[1]]=9` | Both values remain unchanged; this does not exercise copy-on-write value semantics because no mutation occurs. |
| Association key Part assignment | `a[[Key[k]]] = v` | This is a different dedicated path and works; see `crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs:31-38,112-119`. |
| Compound Part update | `a[[i]] += v`, `AppendTo[a[[i]],v]`, etc. | These are implemented by the held-update machinery, not Set; representative tests are in `crates/rum-stdlib/src/tests/incidental_definitions_updates.rs:101-213`. |

These are common source-language idioms, so the feature is worth adding even
though the repository does not currently rely on it internally.

### Repository reachability

A source/test search for `[[...]] =` found no positional List Set in
`rum-runtime` or `rum-stdlib`. The only direct Part-Set forms are Association
key tests:

- `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:549-570`;
- `crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs:31-38`;
- `crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs:112-119`.

The standard library itself does not use positional Set internally. It uses
Part reads and the working compound-assignment/association mutation APIs.
Thus current in-repo reachability is effectively zero, while source-language
user reachability is high.

The current normative reference also does not promise this feature:
`docs/reference/builtins.md:4663-4676` says Set assigns “to a symbol or
pattern.” Part is documented only as extraction at
`docs/reference/builtins.md:3733-3747`. Adding positional lvalue behavior is
therefore an intentional contract expansion, not a test repair for an already
documented guarantee. The catalog and generated reference must change in the
same semantic step.

## 5. Observability boundary

### What happens today

`Set` is a special form, so its raw lhs is never ordinary argument-prepared.
The observable order for a positional Part-shaped lhs is:

1. evaluate the rhs completely, including user code and suspension
   (`definitions.rs:19-46`);
2. perform an Association shape probe, which clones one `Expr`/Arc and checks
   its head but runs no user code (`association_data.rs:389-442`);
3. recursively substitute stored own/delayed values while normalizing the
   definition lhs, without calling `Session::evaluate`
   (`definitions.rs:472-600`);
4. emit `Set::wrsym` for `Part`; and
5. return the evaluated rhs without mutating the owner.

In particular, a positional selector is not generally evaluated today. This
probe:

```text
a = {1}; order = {};
a[[(AppendTo[order, selector]; 1)]] = (AppendTo[order, rhs]; 9);
{order, a}
```

returns:

```text
! Set::wrsym: Symbol Part is Protected.
List[List[rhs], List[1]]
```

Only the rhs side effect runs.

### Proposed contract

For the new narrow lvalue form, follow the already-tested Association Set
ordering at
`crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs:147-175`:

1. Set evaluates the rhs first;
2. it evaluates positional selectors left-to-right, suspension-safely;
3. it re-fetches the owner's current value after all callbacks;
4. it rechecks owner protection and the exact List path;
5. it performs a structural replacement with no evaluator call in the
   mutation window;
6. it bumps the owner epoch and returns the evaluated rhs.

The rhs and selectors can run arbitrary user code before mutation, including
reassigning, clearing, or protecting the target. Therefore no pointer, path,
protection result, or borrowed definition slot obtained before selector
evaluation may be reused afterward.

Unlike AppendTo, positional replacement does not construct and evaluate a new
call head. No List downvalue/upvalue is supposed to run after the store, so
there is no semantic need for AppendTo's “default List definitions” condition.
The first implementation and fast path should nevertheless remain narrow:
plain symbol owner, one exact path made of positive machine-sized integer
selectors, and a plain List at every ancestor. Tracing, monitors, active
resource constraints, pending suspension/input, dynamic own-value frames, a
removed target, or a protected target should select the generic copy/store
path (or reject the write as appropriate), because they can observe identity,
failure, or lifecycle details.

No call-stability proof should be reinserted after replacement. Eviction is
performance-only and safe; claiming stability for the changed call is not
generally safe because a new argument shape can make definitions match later.

## 6. Recommended implementation plan

### Step 1 — add correct exact-positive List Part assignment semantics

- **Change:** route a syntactic `Part[plainSymbol, selectors...]` after the
  existing Association lvalue attempt but before definition-lhs
  normalization. Evaluate selectors left-to-right after the already-evaluated
  rhs, then re-fetch and recheck the owner. Accept one exact path consisting
  of one or more positive machine-sized integer selectors through plain Lists.
  Initially rebuild with the existing
  `exact_position_paths_from_part_target_selector_exprs` plus
  `replace_exact_path`, then store and bump the owner epoch. Consume
  recognized-but-invalid positional targets with normal Part/Set diagnostics;
  never fall through to defining protected `System`Part.
- **Files/functions:** keep only a routing call in
  `crates/rum-runtime/src/runtime/definitions/definitions.rs`;
  add `definitions/part_assignment.rs` and register it in
  `definitions/mod.rs`; expose only the narrow existing Part failure
  diagnostic wrapper if required through
  `kernel_data/structure_data/{mod.rs,selectors.rs}`; update Set's summary in
  `builtin_catalog/rules.rs` and regenerate `docs/reference/builtins.md`; add
  a focused `runtime/tests/part_assignment.rs` and a focused request-resume
  test module.
- **Invariant:** rhs first, selectors left-to-right, no store before all
  evaluation completes, protection rechecked after callbacks, and
  `x={1,2,3}; y=x; y[[1]]=9` leaves x unchanged. Association Key lvalues,
  compound Part updates, list destructuring, ordinary definition lhs
  normalization, invalid messages, and suspension/resume retain their
  behavior.
- **Gate:** focused Part-assignment and request-resume tests; all Association
  lvalue and mutation-operator tests; workspace tests; generated-reference
  drift; microbench A/B on existing Set/Part/AppendTo/Association neighbors.
  Record a representation-counter baseline for the working n=200 fill.
- **Expected win:** semantic, not performance. The operation works and no
  longer emits a false protected-Part diagnostic. The generic rebuild is
  expected to remain O(m) per assignment and supplies the honest base for
  Step 3.
- **Organization:** `definitions.rs` is 28,979 chars, in the split-plan range.
  This step adds only a route there. The new policy/continuation belongs in
  `part_assignment.rs`; the future split boundary for `definitions.rs` is the
  existing lhs normalization cluster at `definitions.rs:472-600`, not part of
  this semantic patch.

### Step 2 — add positional-assignment measurement labels

- **Change:** add four `rumMicroOnce` labels at the end of the generated source
  in `scripts/core-language-microbench.sh`:
  `set-part-fixed100-len100`, `set-part-fixed100-len800`,
  `set-part-fill-small` (n=200), and `set-part-fill-large` (n=400). Initialize
  each dedicated top-level List immediately before its label but outside the
  exact timed `Do[...]`, so the fixed-operation pair varies only the List
  length and does not time `Table`.
- **Files/functions:** generated benchmark heredoc and its `ClearAll` list in
  `scripts/core-language-microbench.sh` only.
- **Invariant:** exact working statement, no sink, no extra Set inside the
  timed loop, `AbsoluteTiming` unchanged, all profiling disabled, existing
  labels untouched, and small/large fill arms differ only in n.
- **Gate:** `bash -n`; two bounded suite runs; generated-source inspection;
  standalone AbsoluteTiming cross-checks; complete row count; same-tier
  existing label comparison. Persist raw TSV before parsing.
- **Expected win:** none. This commits the working O(m)/O(n²) baseline needed
  for the fast-path A/B.
- **Organization:** no Rust.

### Step 3 — replace the generic rebuild with cache eviction plus spine COW

- **Change:** after Step 1's evaluation/re-fetch/recheck phase, attempt a
  conservative direct route. Borrow-plan the normalized root-to-parent List
  path and collect pointer keys without retaining cloned children. Poll
  interrupt, recheck protection, evict pointer-matched call-cache entries for
  root through parent, take the definition value under the shared rollback
  guard, `SharedArgs::make_mut` each ancestor, and replace the one parent slot
  under a swap-back guard. Commit, bump the owner epoch, and leave the changed
  call proofs evicted. On any predicate failure, use Step 1's generic path.
- **Files/functions:** `definitions/part_assignment.rs`;
  `definitions/incremental_append.rs` only to move the existing common
  environment predicate/definition-value rollback verbatim;
  new `definitions/incremental_mutation.rs` for those shared primitives;
  `definitions/mod.rs`; focused Part-assignment and cache tests.
  `evaluation_cache/mutation_proofs.rs` should remain unchanged unless a
  caller-shape adjustment is proven necessary—the existing
  `evict_call_stability_keys` API fits.
- **Invariant:** only the selected slot changes; every semantic alias at root,
  intermediate List, selected child, or a second position remains unchanged.
  No evaluator call or fallible resource check occurs after the first slot
  mutation. A failure after replacement swaps the old item back and restores
  the definition. Cache eviction changes performance only. AppendTo's flat
  and Part fast paths retain exactly their current predicate.
- **Gate:** alias-at-every-level, callback ordering, protection race,
  tracing/monitor/resource fallback, interrupt-before-mutation,
  suspension/resume, and injected rollback tests; workspace tests. Primary
  measured signals are the n=200 representation counters, the
  len800/len100 fixed-work quotient, and the fill-large/fill-small quotient.
  Recheck both AppendTo counter probes and same-tier normalized sentinels.
- **Expected win:** for an unaliased, unconstrained plain-List path, one slot
  replacement becomes O(depth), independent of container width. The
  fixed-work length quotient should approach 1; fill 200→400 should approach
  2 instead of about 4. Semantic aliases still legitimately cause COW and an
  O(m) copy.
- **Organization:** moving the generic guard/predicate out of the
  10,084-char `incremental_append.rs` improves its responsibility boundary;
  no new logic goes in the 28,979-char `definitions.rs`.

## 7. Draft follow-up implementation briefs

These are raw drafts in `TEMPLATE.md` shape. They are intentionally separated
so the performance step is compared against a working semantic base.

### Draft brief 298 — implement exact-positive positional Part lvalues for Set

#### Goal

Make `list[[i]] = rhs` and `matrix[[i,j]] = rhs` work for a plain symbol owner
and one exact path of positive machine-sized integer selectors through plain
Lists. Set must evaluate rhs first, evaluate selectors left-to-right, re-fetch
and recheck the target, store exactly one replacement, return rhs, preserve
value semantics, and never fall through to a protected definition on
`System`Part. This brief is correctness-first; the generic rebuild may remain
O(m).

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, and
  `docs/dev/codex/plan-297-part-assignment.md`.
- Current Set entry and rhs continuation:
  `crates/rum-runtime/src/runtime/definitions/definitions.rs:14-51,115-117`.
- Current lvalue ordering and bad fallback:
  `definitions.rs:257-385`.
- Association selector continuation/order:
  `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:389-526`.
- Exact path helpers:
  `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs:22-67`.
- Generic rebuilding store primitive:
  `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:173-200`.
- Contract source:
  `crates/rum-runtime/src/runtime/builtin_catalog/rules.rs:94-120`;
  generated result `docs/reference/builtins.md:4663-4676`.
- Acceptance is the named behavior tests, generated-reference drift test, and
  hot-path sentinel A/B—not comparison with another engine.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/definitions.rs` — routing call
    only, before definition-lhs normalization;
  - `crates/rum-runtime/src/runtime/definitions/mod.rs` — module wiring;
  - `crates/rum-runtime/src/runtime/builtin_catalog/rules.rs` — Set summary
    only;
  - `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs` — only
    a narrow diagnostic facade if needed;
  - `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs` —
    only the matching diagnostic visibility/wrapper if needed;
  - `crates/rum-runtime/src/runtime/tests/mod.rs` — test module declaration;
  - `crates/rum-runtime/src/runtime/tests/session_requests/mod.rs` — request
    test module declaration;
  - `crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs` —
    regression assertions only if the new route needs explicit precedence
    coverage;
  - `docs/reference/builtins.md` — regenerated output only, never hand-edited.
- Files you MAY create:
  - `crates/rum-runtime/src/runtime/definitions/part_assignment.rs`;
  - `crates/rum-runtime/src/runtime/tests/part_assignment.rs`;
  - `crates/rum-runtime/src/runtime/tests/session_requests/part_assignment_resume.rs`.
- Files you MUST NOT touch: everything else, especially
  `incremental_append.rs`, `definition_updates.rs`,
  `evaluation_cache/**`, `rum-core`, scripts, Cargo files, other docs, and
  `.git`.
- Do not add unconditional instrumentation or unrelated lhs-normalization
  cleanup.

#### Deliverable

1. A syntactic positional Part route after Association lvalues and before
   generic definition-lhs normalization.
2. Suspension-safe selector evaluation after rhs evaluation, left-to-right,
   with target state re-fetched afterward.
3. The deliberately narrow accepted shape: plain symbol owner; one or more
   evaluated positive machine-sized integer selectors; exactly one path;
   plain List at every ancestor.
4. A generic `replace_exact_path`-based update and normal symbol-epoch bump.
5. Recognized invalid Part targets produce the owned Part/Set diagnostic and
   never try to write a DownValue on `Part`.
6. Updated Set catalog text plus regenerated reference.
7. Focused tests for:
   - flat and nested success;
   - array fill, swap, and matrix update;
   - computed selectors;
   - rhs-before-selector and left-to-right selector order;
   - selector reassigns/clears/protects the target;
   - initially protected target;
   - Module and Block/localized owners;
   - alias value semantics;
   - absent own value, atom/non-List owner, zero/negative/bulk/Span/Key,
     out-of-range, and excessive-depth failures;
   - Association Key Part precedence and all existing Association behavior;
   - a selector that suspends for host input and resumes exactly once.
8. A representation snapshot for the now-working n=200 fill, explicitly
   labeled as the baseline for a later optimization.

#### rum / Rust pitfalls (read twice)

1. Set is a special form. Do not route it through ordinary argument
   preparation or evaluate the held owner expression.
2. `eval_set` has already evaluated rhs. Do not evaluate rhs a second time.
3. A selector can suspend or mutate/protect the target. Never retain a
   borrowed definition, path, or protection result across selector
   evaluation.
4. Preserve the continuation shape with `suspend_with_frame`; add the explicit
   queued-input versus resumed-input equivalence test.
5. Keep Association Key lvalues ahead of positional List handling.
6. The initial implementation is allowed to copy. Do not smuggle the Step-3
   cache/COW optimization into this correctness brief.
7. `definitions.rs` is in the 20k–35k split-plan range. Add only the route and
   report that lhs normalization is the future split boundary.

#### Verification (MANDATORY — run and report verbatim)

Use `scripts/run-bounded.py` with bounds derived from a prior run of the same
shape:

```text
cargo fmt --check
cargo run -p rum-stdlib --bin generate-builtin-reference
cargo test -p rum-runtime part_assignment
cargo test -p rum-runtime session_requests
cargo test -p rum-stdlib associations_set_lvalue
cargo test -p rum-stdlib builtin_reference_docgen
cargo test --workspace
git diff --check
```

Run the core-language microbench on base and candidate because this changes
Set's live lvalue path.
Report normalized movement for `set-ownvalue`, `part-list`,
`symbol-ownvalue-read`, `do-iterator-null`, both AppendTo families, and both
Association incremental rows. No target speedup is required in this brief.
Run the n=200 `RUM_PROFILE_REPR=1` working-fill probe once and paste the full
counter line.

#### Hard rules

- Do NOT commit, branch, push, or touch git config.
- Do NOT modify files outside Scope.
- No `unsafe`; `rum-core` and `rum-runtime` stay zero-unsafe.
- Tests assert rum's newly documented contract only. No external oracle,
  differential harness, compatibility matrix, or third-party output.
- Value semantics are mandatory:
  `x={1,2,3}; y=x; y[[1]]=9` must leave x unchanged.
- RHS/selectors finish before mutation; no evaluator call occurs in the store
  window.
- Do not change `Expr`, `SharedExpr`, `SharedArgs`, or the builtin `&[Expr]`
  ABI.
- No performance claim against today's failed protected-Part path.
- Touch nothing outside the repo except `/tmp`; no network.

#### Report format

1. The exact accepted lvalue boundary and route, with `path:line`.
2. Behavior-test list, including aliasing, order, protection race, and
   suspend/resume.
3. The old four failing idioms and their new outputs.
4. Generated-reference diff summary.
5. Test totals and `cargo fmt --check`.
6. Existing-label A/B sentinel table.
7. Working n=200 representation counters.
8. `definitions.rs` split assessment.
9. Confirmation that `git status` contains only Scope files.

### Draft brief 299 — add positional Part assignment microbench labels

#### Goal

Add four checked-in labels that separately measure dependence on container
length and accumulation complexity for the now-working exact-positive
positional Set path. This brief changes no Rust and promises no speedup.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/dev/codex/plan-297-part-assignment.md`, and brief 292's measurement
  discipline.
- `scripts/core-language-microbench.sh:266-327` contains the existing
  incremental-update labels and placement convention.
- `rumMicroOnce` uses monotonic `AbsoluteTiming`, one untimed warm evaluation,
  and min-of-repetitions. Do not change the harness.

#### Scope

- File you MAY modify:
  - `scripts/core-language-microbench.sh` — only the generated benchmark
    source heredoc and its `ClearAll` symbol list.
- Files you MUST NOT touch: every Rust file, every other script, all docs,
  Cargo files, `.git`, and git config.

#### Deliverable

Add these four labels immediately before `RUM_CORE_MICRO_END`, after all
existing labels:

| Label | Setup outside timed statement | Exact timed statement |
|---|---|---|
| `set-part-fixed100-len100` | dedicated 100-element zero List | `Do[L[[1]] = i, {i,1,100}]` |
| `set-part-fixed100-len800` | dedicated 800-element zero List | the identical 100-write loop |
| `set-part-fill-small` | dedicated 200-element zero List | `Do[L[[i]] = i, {i,1,200}]` |
| `set-part-fill-large` | dedicated 400-element zero List | the same shape with n=400 |

Use a distinct `gMicro...` symbol for each label. Do not put `Table`, a sink,
Print, or a validating read inside the timed expression. Assert outside timing
in an independent probe that each List was actually updated.

#### rum / Rust pitfalls (read twice)

1. The fixed-work pair must vary only initial List length.
2. The fill pair must vary only n.
3. Initialization inside the timed expression would contaminate the
   fixed-work length quotient; keep it outside.
4. The heredoc is unquoted; do not add an unintended `$` or backtick.
5. Profiling must be off for every timing.
6. Persist raw output before parsing and use within-run quotients; do not
   normalize against the suite median.

#### Verification (MANDATORY — run and report verbatim)

1. `bash -n scripts/core-language-microbench.sh`.
2. Inspect the generated `.wls` and quote all four exact statements.
3. Run two bounded suite repetitions into distinct output directories; both
   must have `status=ok`, `exit_code=0`, and a complete label table.
4. Cross-check all four labels with standalone min-of-5 AbsoluteTiming probes
   under `/tmp`.
5. Report the len800/len100 quotient and fill-large/fill-small quotient from
   each run.
6. Compare all pre-existing labels between runs and flag any target-neighbor
   disturbance.
7. Report suite wall time and timeout headroom.
8. Rust format/tests are not required because no Rust changes; say so.

#### Hard rules

- Do NOT commit, branch, push, or touch git config.
- Modify exactly the one Scope file.
- No Rust, docs, generated artifacts, or second benchmark harness.
- No external engine or network.
- Do not change existing label statements, harness repetitions, timing
  primitive, parser, TSV format, or profiling environment.
- A label that does not scale with its intended variable is invalid; diagnose
  it rather than reporting it.

#### Report format

1. Four verbatim `timings.tsv` rows from each run, raw and decimal.
2. Both quotients per run and their interpretation.
3. Standalone cross-check table.
4. Existing-label disturbance summary.
5. Suite wall time/headroom.
6. Generated-source inspection result.
7. Confirmation that `git status` shows only
   `scripts/core-language-microbench.sh`.

### Draft brief 300 — incremental copy-on-write positional Part assignment

#### Goal

Make the exact-positive plain-List Part Set path from brief 298 O(depth) per
unaliased replacement by evicting cache-only owners and mutating the
definition's List spine through `SharedArgs::make_mut`, without changing any
observable behavior or regressing the existing AppendTo fast paths.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/dev/codex/plan-297-part-assignment.md`, and the final brief-298/299
  reports.
- Working generic path: `definitions/part_assignment.rs` from brief 298.
- Existing spine mechanism and rollback:
  `crates/rum-runtime/src/runtime/definitions/incremental_append.rs:7-185,223-301`.
- Safe COW primitive:
  `crates/rum-core/src/expr.rs:593-603`.
- Cache eviction:
  `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs:98-124`.
- Follow the checked-in four labels from brief 299. Compare against the
  working generic base, never against pre-298 failed assignments.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/part_assignment.rs` — main
    planning, predicate, direct mutation, and test-only failure hook;
  - `crates/rum-runtime/src/runtime/definitions/incremental_append.rs` — move
    common predicate/definition rollback verbatim and call the shared helper;
  - `crates/rum-runtime/src/runtime/definitions/mod.rs` — shared module wiring;
  - `crates/rum-runtime/src/runtime/tests/part_assignment.rs` — behavior,
    alias, fallback, rollback, and identity tests;
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` — only
    pointer-matched path-eviction/identity coverage if not already sufficient.
- File you MAY create:
  - `crates/rum-runtime/src/runtime/definitions/incremental_mutation.rs` —
    shared no-callback predicate and `DefinitionValueRollback`.
- Files you MUST NOT touch: everything else. In particular do not change
  `definitions.rs`, `definition_updates.rs`, structure-data generic helpers,
  `rum-core`, scripts/labels, docs/reference, Cargo files, or `.git`.
- `evaluation_cache/mutation_proofs.rs` is deliberately not in Scope: its
  existing pointer-matched eviction API fits. If it proves insufficient, stop
  and report the required API rather than widening Scope.

#### Deliverable

1. A borrow-only direct plan that:
   - validates the same exact-positive List-only boundary as brief 298;
   - normalizes the selected path to parent indexes;
   - gathers root-through-parent `CallStabilityCacheKey`s without retaining a
     cloned selected value.
2. Exact extraction of the common unobservable-environment predicate and
   definition-value rollback from `incremental_append.rs`, preserving
   AppendTo behavior byte-for-byte apart from call paths/visibility.
3. Before mutation: interrupt poll, protection recheck, all path checks, and
   pointer-matched cache eviction.
4. During mutation: take the definition under rollback, COW each ancestor,
   replace one slot under a swap-back guard, inject the test failure point,
   then commit.
5. After mutation: bump the owner symbol epoch and return rhs. Do not reinsert
   a stability proof for the changed parent or ancestors.
6. Generic brief-298 fallback for every failed predicate, alias-safe but
   potentially O(m).
7. Tests for root/inner/selected aliases, same child at two positions,
   three-level paths, returned rhs, tracing, monitor, active time/memory
   constraints, dynamic owner, protected target, interrupt, and rollback
   after replacement.

#### rum / Rust pitfalls (read twice)

1. Do not hold a cloned root/child while trying to make the definition
   exclusive; that recreates the diagnosed owner.
2. Evict every pointer-matched call cache entry from root through the parent
   before `make_mut`. Semantic aliases must remain and trigger COW.
3. Do not take/reinsert an old stability proof. Replacement can make a
   previously stable call match definitions; eviction must force normal
   recomputation.
4. No evaluator call, message construction, resource check, or allocation
   reservation after the first slot replacement.
5. Fixed-length replacement needs no Vec reserve. Use `mem::replace` plus a
   safe-Rust swap-back guard.
6. Preserve AppendTo's exact predicate and counter win when extracting shared
   helpers.
7. No unconditional diagnostic counter or default-path heap work.

#### Verification (MANDATORY — run and report verbatim)

Correctness:

```text
cargo fmt --check
cargo test -p rum-runtime part_assignment
cargo test -p rum-runtime mutation_operator_forms
cargo test -p rum-runtime evaluation_cache
cargo test --workspace
git diff --check
```

Use bounds derived from prior same-shape runs and the bounded runner.

Counters, with `RUM_PROFILE_REPR=1`:

- working generic base and candidate for
  `qL=Table[0,{200}]; Do[qL[[i]]=i,{i,1,200}]`;
- existing flat AppendTo 500-iteration probe;
- existing Part AppendTo outer1000/200 probe.

The positional candidate should drive its per-update `into_vec_total` /
`into_vec_clone` contribution toward zero. Existing AppendTo counter lines
must remain at their post-295 levels.

Timing:

- at least three base and three candidate suite runs, interleaved;
- report `set-part-fixed100-len800 / -len100` within each run;
- report `set-part-fill-large / -small` within each run;
- report the existing flat AppendTo large/small and Part-vs-flat quotients;
- normalize only against explicitly untouched control labels, excluding all
  target rows;
- report `set-ownvalue`, `part-list`, `symbol-ownvalue-read`,
  `do-iterator-null`, `list-literal-small`, both Association incremental rows,
  and all four AppendTo rows.

Stop if counters do not move; a timing change without the representation
signal is noise.

#### Hard rules

- Do NOT commit, branch, push, or touch git config.
- Do NOT modify files outside Scope.
- No `unsafe`.
- No semantic expansion beyond brief 298's exact-positive plain-List path.
- Preserve value semantics at every alias depth.
- Preserve suspension/resume and all generic fallback behavior.
- Do not change `Expr` representation or builtin `&[Expr]` ABI.
- Hot-path A/B and same-tier sentinels are mandatory.
- Testing Policy is a hard line; no external oracle or network.
- If the direct predicate rejects the checked-in target labels, stop and
  report why rather than widening it.

#### Report format

1. Positional representation counters, generic base versus candidate.
2. Fixed-length and fill-complexity quotients, 3+ runs each side.
3. Confirmation that both AppendTo paths retain counters and quotients.
4. The exact predicate and cache keys evicted, with `path:line`.
5. Alias-at-every-level and injected rollback tests.
6. Tracing/monitor/resource/dynamic/protection fallbacks.
7. Normalized sentinel table.
8. Test totals and `cargo fmt --check`.
9. Which common helpers moved verbatim.
10. Confirmation that `git status` contains only Scope files.

## 8. Risks and open questions

| Risk / question | Finding or mitigation |
|---|---|
| Is this an optimization or a feature? | It is first a feature. Current documented Set behavior does not include positional Part lvalues, and the operation fails. Ratify the narrow contract before implementation. |
| Are current M1/M2 values a usable speed baseline? | No. They time failed protected-Part definition attempts. Use brief 298's working generic path as the performance base. |
| What selector surface should v1 accept? | Recommend one path of positive machine-sized integer selectors through Lists. Defer zero/head, negative, All, Span, list/duplicate/bulk, and mixed Key forms. This matches the already-proved Step-4 shape and covers fill/swap/matrix idioms. |
| Association overlap | Keep `a[[Key[k]]] = v` on the Association handler and run it first. Numeric Association Part assignment is outside this List-only plan and must not be accidentally claimed. |
| Selector observability | Current positional selectors do not run; the proposed rhs-then-selectors order is a deliberate new contract aligned with Association Set. Suspension and target-mutating selector tests are mandatory. |
| User code after mutation | None is required for a structural Set store. Keep all evaluation before mutation and do not evaluate the changed container afterward. |
| Stability proof reuse | Unsafe in general after argument replacement. Evict changed call entries and let later evaluation recompute; do not union a bloom into an old proof. |
| Alias cost | An actual semantic alias must force `Arc::make_mut` to copy. O(m) in aliased cases is required value semantics, not a regression. |
| Trace/monitor/resource behavior | Use the generic copy/store path in observable or constrained environments until dedicated tests prove a broader direct boundary. |
| Rollback identity | Rollback must restore the semantic value. COW performed before an injected failure may change internal Arc identity; no public contract promises identity. |
| Large source files | `definitions.rs` is 28,979 chars. Keep the route tiny and new policy in its own module; record lhs normalization as the future split boundary. `incremental_append.rs` is 10,084 chars, so moving generic mutation helpers out is preferable to adding another responsibility. |
| In-repo demand | No positional List Set use exists in runtime/stdlib tests or implementation today. The justification is user-language ergonomics and consistency with existing Part compound updates, not an internal caller. |
| Representation/ABI alternatives | Out of scope and unnecessary. Safe Arc COW plus exact cache eviction is sufficient; no `unsafe`, Expr redesign, or builtin slice ABI change is proposed. |
