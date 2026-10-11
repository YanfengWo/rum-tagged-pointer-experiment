# Pinned rum expression census

This stage evaluates the unchanged recursive prototype from revision
209b08afa2061b65289e8731bb6fcbb71e15475a on expressions taken from rum's
own performance suite at upstream revision 77029938f999b14dbe5b0755bebdf98e51517ae6.
No evaluator, core representation or unsafe owner changed.

The corpus contains all 112 timing labels' lowered bodies and 15 selected
evaluated setup right-hand sides at scale 1. The final retained-gensym label
uses a custom AbsoluteTiming body; the other 111 use rumMicroBench/rumMicroOnce.
Inputs are lowered without executing the benchmark's setup or timed workloads.
Results evaluate the saved RHSs after defining only rumMicroScale/rumMicroN.
They are expression snapshots, not a dynamic allocation trace or complete
evaluator benchmark. In particular, these results do not measure runtime
generated symbols, definition stores or intermediate evaluator objects.

corpus/PIN.json records exact byte offsets, individual source hashes, original
generated-source and generator hashes, dependency source hashes and scope.
All 112 labels match the saved baseline timings, and all 127 extracted spans
were checked against their original bytes before publication. Some generated
fixtures exceed normal source-size guidance; they retain the original expressions
intact for reproducibility. JSON inventories are generated records. Handwritten
Rust/Python modules remain below 10k characters.

## Checks

Both default and allocation configurations pass three focused graph tests.
Each of the census and allocation commands checks full roundtrip equality
for Compact and Safe on every snapshot: 254 equalities per command. Every
one of the 381 fresh-graph allocation windows returns to its prior live-byte
baseline after release. Measured Compact/Safe/original differences match the
independent layout cost model on all 127 graphs. Formatting passes.

The tests distinguish physical storage from logical multiplicity, preserve
generated symbol identity and manually constructed Big variants, retain source
capacity information, and compute an 80-level DAG's saturated logical size
without exponential unfolding. The logical count flag conservatively marks
a count reaching u64::MAX, including an exact-limit count. This census adds no unsafe code. The optional
counter reuses the unchanged System allocator wrapper tested under both Miri
models in the preceding recursive stage. The full census/session/stdlib program
was not run under Miri; prior owner checks and manual arguments remain their
own evidence, not a complete soundness proof. Unchanged kernel tests are reused
from the preceding baseline rather than rerun here.

## Measurements

Retained **requested heap bytes** below exclude the root stack object (original
32 bytes, Safe 16, Compact 8), source graph, previously interned symbols,
allocator metadata and fragmentation. They are not RSS. Each fresh normalized
original graph is produced through Safe roundtrip, preserving Head/Args alias
classes and discarding spare Vec/String capacity. This compares layouts on
equally normalized graphs. The source graph's lengths and capacities are reported
separately. Original construction peaks/calls include two bridges and are not
fair construction-cost comparators; no timings are collected in this stage.

| Evaluated snapshot | Original normalized | Safe | Compact | Compact / original |
| --- | ---: | ---: | ---: | ---: |
| Integer list, 1,600 values | 51,288 | 25,768 | 12,960 | 0.253 |
| Symbol list, 16,000 values | 512,088 | 1,024,168 | 896,160 | 1.750 |
| Integer Association, 32,000 rules | 5,888,088 | 6,912,168 | 5,888,160 | 1.000 |
| ReplacePart rules, 1,600 entries | 294,608 | 422,704 | 371,480 | 1.261 |
| Three simple rules | 640 | 960 | 856 | 1.338 |
| One 32,000-byte string | 32,000 | 32,048 | 32,048 | 1.002 |

Among the 112 lowered inputs, Compact retains less heap than normalized original
in four snapshots, equal heap in one, and more in 107. Among the 15 selected
evaluated results, it retains less in five and more in ten. These counts give
one vote per selected snapshot and have no workload-frequency or time weighting.
They are not a percentage of real evaluator allocations or performance losses.
Complete counts, sharing, depths, capacities and allocation measurements are in
../evidence/expression-census/summary.json and its raw JSON files.

## Why this layout loses

Let P be physical Expr slots (root + unique Head targets + slots of unique
argument vectors), I the slots holding inline-eligible integers, and S=P-1.
In this pinned x86_64 layout every other physical node needs a 48-byte
Arc<Payload<Body>>. Narrowing Head targets and argument-vector slots saves
24 bytes per S slot for Compact and 16 for Safe. Other owned atom data and
Arc<Vec> headers cancel between normalized graphs:

    Compact heap - original heap = 48*(P-I) - 24*S
    Safe heap - original heap    = 48*(P-I) - 16*S
    Safe heap - Compact heap     = 8*S

These equalities match every measured graph in this corpus; they depend on
the pinned layout and current conversion policy. Including the single root's
stack size makes Compact's difference 24*(P-2*I): roughly half the physical
slots must be inline integers before this policy saves total modeled storage.
An 8-byte node does not imply an 8-byte total cost for a boxed symbol or Call.

The Association has 64,000 integer slots, 32,001 symbol slots and 32,001 Calls:
integer savings almost exactly cancel boxing. Those symbol slots contain only
two distinct symbol values, while the wide symbol list contains 16,001 distinct
values including its head. A repeated-symbol cache can help the Association,
but cannot solve the distinct-symbol list's boxing cost. Source spare capacity
is another independent cause: the 32,000-rule Association has 41,728 spare
argument-vector slots. Normalization removes that capacity in every layout;
that improvement must not be credited exclusively to tagged pointers.

## Next experiment and boundaries

The next bounded experiment should share immutable Symbol payloads by full
Symbol equality, with the same policy in Safe and Compact, while retaining
separate Head and Args identities. Test generated identities and contexts,
unchanged COW/Weak behavior, distinct-symbol fallback, cache bounds, bridge
peaks and read/construction costs. It is an opt-in prototype until measured.
Repeated-symbol sharing is a policy optimization and cannot establish an
unsafe-specific advantage by itself.

Distinct symbols require a separate representation study. The current public
Symbol API exposes an interned numeric id but no safe constructor from that
id or from the private canonical SymbolText reference. Do not assume arbitrary
ids can be reconstructed or recover private fields through layout casts.
A future safe core access API or another owner design must be reviewed and
measured before adding a direct interned-symbol tag. No such core change or
full runtime migration is included here.

On Linux with Python 3 and Rust 1.85.0, run `bash run-study.sh`.
The committed corpus is sufficient; extraction of the original generated
source is unnecessary. CARGO_TARGET_DIR can select native temporary storage.
Evidence defaults to target/census-evidence; analyze.py verifies corpus and
dependency hashes, checks, accounting, layout deltas and complete row coverage.
Full kernel integration still requires rum-vs-rum per-label evaluator A/B and
lifecycle/embedding validation. No external engine is used as an oracle.
