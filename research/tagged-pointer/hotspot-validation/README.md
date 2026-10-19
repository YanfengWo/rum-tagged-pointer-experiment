# Original rum hotspot diagnostics

This study diagnoses three small rum-owned semantic paths in the unchanged
baseline. It does not optimize the evaluator and does not measure a speedup.
Inputs derive from the project's `core-language-microbench.sh`: iterator Table,
Part AppendTo, and level-one Replace traversal. No other engine is an oracle.

| Workload | Sizes | Checked result | Question |
| --- | --- | --- | --- |
| `table-pairs` | 64, 256, 1024 | Entire output equals the generated literal list of `{j,j+1}` pairs | How much shared expression construction accompanies iterator evaluation? |
| `append-part` | 64, 256, 1024 appends; fixed 1000 outer lists | Entire first list equals Range; all remaining 999 lists stay empty | Do repeated nested mutations trigger owned-vector COW copies? |
| `rewrite-wide` | 64, 256, 1024 integers | Entire replaced list equals shifted Range; source remains intact | How much expression construction accompanies level-one rule replacement? |

Each active input has a fresh-process empty-operation control with identical
setup and snapshots. Generated `.rum` inputs, stdout, stderr, input hashes,
selected original source hashes, and the baseline binary hash are retained.
Semantic checks execute after the diagnostic snapshot, outside the scoped
counter difference. The last `PASS` line proves execution reached those checks.

## Running

Prepare without running rum:

```sh
python3 -B research/tagged-pointer/hotspot-validation/run-study.py \
  --prepare-only --output-dir ../evidence/hotspot-validation
```

After all uninstrumented timing jobs have finished, execute serially through
the existing bounded runner. Use an existing unchanged baseline release binary:

```sh
scripts/run-bounded.py --timeout 240 -- python3 -B \
  research/tagged-pointer/hotspot-validation/run-study.py \
  --output-dir ../evidence/hotspot-validation \
  --rum-cli /absolute/path/to/baseline/rum-cli \
  --base-source ../base
```

The baseline commit is `77029938f999b14dbe5b0755bebdf98e51517ae6`.
No original source is modified. `run-metadata.json` records selected source
hashes and the exact binary used; a hash alone is not a fresh rebuild proof.

## Reading the data

`RUM_PROFILE_REPR=1` records `into_vec_total`, `into_vec_clone`,
`into_owned_total`, `into_owned_clone`, `shared_args_alloc`, and
`shared_expr_alloc`. These count selected COW and construction operations, not
all `Expr::clone` calls, Arc increments, bytes copied, or allocator calls.
`RumProfileReset[]` does not reset REPR counters: the analysis subtracts the
before snapshot from the after snapshot **in the same process**, then reports
the fresh-process empty-control difference separately. Raw cumulative values
are retained too. Setup/parsing and post-snapshot validation are outside the
delta; wrapper work remains visible. Subtracting an empty control is an
approximation, since active work can change caches and runtime state.
`into_vec_clone` and `into_owned_clone` cover their named conversion APIs;
`SharedArgs::make_mut` can allocate a COW vector without either increment.

`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, `RUM_PROFILE_LEXICAL_DEFER=1`, and
`RUM_PROFILE_BUILTIN_TIMING=1` enable existing builtin/runtime diagnostics after
setup. `RumProfileReset[]` starts that phase. Builtin self time subtracts nested
builtin scopes; evaluator, allocation, and rule work between those scopes may
remain attributed to Do/Table/Set/AppendTo. A large label is therefore a search
location, not a proven root cause. Builtin and runtime-path categories overlap.
Do not add them into a percentage or compare their instrumented durations as a
performance result. Reset/snapshot wrapper scopes also appear in the logs.

The original CLI's separate `alloc-count` feature plus `RUM_PROFILE_ALLOC=1`
can measure whole-process allocation count, cumulative requested bytes, and
peak live requested bytes. It includes parsing, setup, diagnostics, and teardown
and has no language-level reset. This study deliberately leaves those fields
uncollected; its REPR counts must not be relabeled as allocation totals or RSS.
The existing memory script has baseline/cleanup proxies but its `time -l`
RSS lane is macOS-specific. Use `--no-rss` on this Linux baseline.

These fixed cases are explanatory micro workloads. Representative application
workloads, sampled function stacks, full clone attribution, allocation site
attribution, and hardware cache/memory-bandwidth evidence remain missing.
Only a subsequent uninstrumented original-vs-candidate gate can establish an
optimization benefit. The complete original 112-label gate remains required
for changes to hot paths.

## Recorded baseline observations

All 18 fresh-process probes passed their semantic checks. The binary SHA-256
was checked against the earlier native baseline record before execution:
`2a621c6a0d25dba987d2a517391867869b84de50a2a976879879e25dfd14a54d`.
The table reports the same-process REPR delta **minus** its corresponding
empty-control delta. These are scoped operation counts, not allocator totals.

| Workload | Size | `shared_args_alloc` | `shared_expr_alloc` | `into_vec_clone` | `into_owned_clone` |
| --- | ---: | ---: | ---: | ---: | ---: |
| Table pairs | 64 | 199 | 8 | 0 | 0 |
| Table pairs | 256 | 775 | 8 | 0 | 0 |
| Table pairs | 1024 | 3079 | 8 | 0 | 0 |
| Part AppendTo | 64 | 71 | 6 | 0 | 0 |
| Part AppendTo | 256 | 263 | 6 | 0 | 0 |
| Part AppendTo | 1024 | 1031 | 6 | 0 | 0 |
| Wide Replace | 64 | 141 | 77 | 64 | 1 |
| Wide Replace | 256 | 525 | 269 | 256 | 1 |
| Wide Replace | 1024 | 2061 | 1037 | 1024 | 1 |

The Replace cases also record exactly N successful rewrite-rule matches and N
Plus builtin calls. They expose a concrete lead: one failed `into_vec`
uniqueness attempt per replaced element in this workload. The counter does not
identify its call site, vector length, copied bytes, or cost; the next audit
should locate that conversion before proposing a change. It could concern a
short or empty temporary vector rather than a wide-list copy.

Table constructs substantially more shared argument objects than Part
AppendTo here, but this does not establish that Table is wasteful or slower.
AppendTo shows no clones through the two named conversion APIs. Its N+7 shared
argument object count cannot rule out repeated wide `make_mut` copies:
allocation size and site attribution are still missing. Diagnostic timings
remain in the raw logs; they establish neither a bottleneck percentage nor an
optimization benefit. Each case has one diagnostic observation, not repeated
timing samples or a statistical confidence interval.
