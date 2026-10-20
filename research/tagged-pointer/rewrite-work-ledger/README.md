# Synchronous rewrite work ledger and Flat prototype

This stage records deterministic work while the desktop is in use. It does
not run a formal performance comparison. The new safe-Rust prototype avoids
rebuilding an owned Flat argument Vec when it has exact capacity and no
immediate same-head nested Call. Default runtime source remains unchanged.

## Findings

For Replace[Range[N], x_Integer :> x+1, {1}], N=8/64/256/1024, exact operation
counts are:

| Work | Pinned original | Wide-copy candidate | Flat candidate | Combined |
| --- | ---: | ---: | ---: | ---: |
| Traversal entries | N+1 | N+1 | N+1 | N+1 |
| Depth-helper entries | 5N+1 | 5N+1 | 5N+1 | 5N+1 |
| Matcher and binding-snapshot entries | 2N | 2N | 2N | 2N |
| Snapshot named items / RHS instantiations | N / N | N / N | N / N | N / N |
| Plus builtin / integer fast-path calls | N / N | N / N | N / N | N / N |
| Wide input Vec copies | 1 | 0 | 1 | 0 |
| Shared two-slot Vec copies | N−1 | N−1 | N−1 | N−1 |
| Fresh no-op Flat Vec outputs | N−1 | N−1 | 0 | 0 |
| Reused Flat Vec outputs | 0 | 0 | N−1 | N−1 |

At N=1024 this means 2048 matcher/snapshot entries, 1024 RHS instantiations,
1024 integer Plus calls, and 5121 depth-helper entries. The new prototype
removes 1023 fresh Vec constructions in this path, while the two-slot shared
copies remain. These counts do not establish which phase dominates time.

Changing only the generated rule body to 1+x gives the same integer results
but zero two-slot normalization copies or no-op Flat rebuilds in all variants:
the positive integers are already in canonical order. This is a controlled
workload observation, not a blanket source-rewriting suggestion; custom
attributes, definitions and side effects require the original evaluator order.

Misses enter no matcher or RHS instantiation in this corpus. Their depth
helper count is2N+1. Identity rules use3N+1 depth entries, opaque single-call
bodies4N+1, and the nested-Flat control6N+1. The nested control retains N
actual flatten reconstructions even with the Flat prototype. These controls
separate work by semantics rather than inferring bottlenecks from total time.

## Design and validation

`ledger.rs` is injected only into a disposable rum-core source copy. It uses
a cached AtomicU8 gate and thread-local synchronous phase stack. Operation
events aggregate by phase,kind,detail,amount and flush at root completion.
Details are UTF-8 hex encoded. A strict parser requires root begin/end,
unique rows and valid units. The module uses no clocks, unsafe or dependencies.

`patch_core.py` records shared-owner construction and unique/copy conversion
events. `patch_runtime.py` separates target evaluation, traversal, matcher,
RHS instantiation, result evaluation and normalization. The latter records
actual sort comparisons, flatten input/output/capacity and integer Plus
success. It does not record every comparator in the earlier orderedness check.
Logical depth/matcher entries are function entries, not total machine steps.
Capacity is in Expr slots, not allocation bytes or retained process memory.

Four variants use one temporary source root and target directory: base,wide,
wide-flat,flat. The optimization patches are applied before identical event
instrumentation; all other changed source bytes are restored between variants.
Third-party dependency artifacts are reused. Saved earlier production binaries
retain their hashes. Build concurrency is limited to2, and every long child
command is bounded.

Each variant freshly passes **2241 workspace checks,7 ignored,29 groups**,
format checking and the preceding copy/alias/guard-order acceptance source.
The diagnostic corpus contains six cases at four sizes,24 sources. Main runs
include192 counter-on roots (two repeats per variant/case/size),96 counter-off
controls and4 resource-observation processes. Each repeat pair has identical
ledgers; the 24 value/alias sources have equal counter-on/off and uninstrumented
baseline stdout. Resource differences are recorded separately. A separate
25-process baseline preflight and 13-process small pilot are retained, plus
two construction-history/resource probes on the original and Flat binaries.

New source modules are below10k bytes. Instrumentation touches pre-existing
20k–35k modules (core expression owners, arithmetic, matcher, binding
substitution and the Replace walker); those changes exist only in disposable
diagnostic copies. Persistent instrumentation would require normal module
organization review. The only new optimization patch touches the smaller
private expression-helper module, without a new public production API.

## Semantic boundary and next experiments

The local no-nesting argument preserves expression elements, ordering and
child identities, and leaves actual nested flattening unchanged. However
**resource observations differ**: ByteCount of a two-argument Flat+Orderless
user expression changes 216→152; three arguments 224→192; five 368→272.
MemoryInUse changes correspondingly in the generated probe. A program testing
those numbers or a tight modeled-memory ceiling can observe this. See the
[value argument and limitations](SEMANTICS.md). The candidate is not claimed
fully contextually equivalent or ready for default integration.

The unchanged original also reports different ByteCount values for two
SameQ-equal expressions constructed differently: a literal two-argument call
reports 211, while construction with Apply reports 147. The Flat variant
reports 147 for both. This confirms that current resource observations can
depend on representation and construction history; it does not prove identical
resource-branch or tight-memory-threshold behavior for the candidate.

The formal performance gate is explicitly
`DEFERRED_USER_ACTIVE_DESKTOP`. Builds/tests and count probes can run under
ordinary desktop use, but small elapsed differences can be distorted by CPU,
memory, disk, temperature and scheduling contention. Earlier timing stages
retained A/A controls but no external-application load trace; those controls
do not exclude every external interference source.

When the desktop is idle, build fresh **uninstrumented** base,wide,flat and
combined variants at the same source root. Use the unchanged full112-label
suite per variant, both A/A controls, and retained hit/reversed/miss/nested
cases. Report every regression and keep resource-API observations separate.
Do not infer speedup from removed Vec counts. The earlier candidates remain
experiments. Other next candidates are reuse of the existing safe Named/Blank
fast matcher and reducing repeated depth walks; neither is implemented here.

## Replaying the diagnostic stage

This lab script assumes the preceding study's offline Rust1.85.0 cache and
saved original production binary. It creates new /var/tmp source/target paths
and refuses an existing nonempty evidence directory. Published machine paths
are redacted; resolve equivalent local paths in the script before independent
replay. This is not a portable one-command production build package.

```bash
python3 research/tagged-pointer/rewrite-work-ledger/build.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/rewrite-work-ledger/run.py /path/to/NEW_EVIDENCE --preflight
python3 research/tagged-pointer/rewrite-work-ledger/run.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/rewrite-work-ledger/check-capacity.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/rewrite-work-ledger/analyze.py /path/to/NEW_EVIDENCE
```

The optional --pilot runs the six smallest sources after the first successful
variant. It is not required for the main analysis. Raw inputs,outputs,tests,
source/binary hashes, all counts and resource values are in the
[summary](../evidence/rewrite-work-ledger/summary.json) and
[full report](../evidence/rewrite-work-ledger/report.md). No unsafe owner code
or formal u64 proof is changed by this stage.
