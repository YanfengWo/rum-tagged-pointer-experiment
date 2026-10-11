# Conditional refinement and bounded prefix decision

This manual argument composes the unchanged Plain and Shared APIs from c62eb6ba.
It assumes their recursive owners are sound. It is not a machine proof of the
unsafe owner, a profitability theorem or a refinement of the full evaluator.
The new policy and study introduce no unsafe; the optional allocation counter
is included from the previously audited source.

## Read-only probe and termination

SmallRoot returns without a traversal or scratch table if the source is an atom
or its root Call has fewer than 64 Args. Large roots start a preorder traversal
of physical Expr slots, with separate Head and Args handle memo sets. Handle
keys come from the existing safe ptr_key API and are used only for equality;
they are never converted into pointers or dereferenced. The immutable source
borrow keeps handles live while their keys are used.

A first-seen Head schedules its target; a first-seen Args schedules a lazy slice
iterator. A duplicate handle skips its storage. Call slots count even if their
Head/Args are duplicates, matching physical Expr slot accounting. Each iterator
step consumes one element, then schedules its remaining iterator and that Expr.
No operation enqueues or scans an entire wide vector. Thus a root with millions
of arguments still samples at most 256 Expr nodes. The node counter includes
Calls and non-Symbol atoms, so a long Call-only chain also hits the budget.

Each visited Call creates at most two handle tasks, each visited argument
creates at most one iterator continuation, and each handle task either skips
or starts a finite iterator/target task. Until the budget or repeat threshold
stops execution, scratch tasks and memo entries are O(256). The limit is on
logical entries/tasks, not HashSet capacity or exact allocated bytes. All scratch
storage is dropped when probe returns, before conversion starts. Probe never
mutates the source or retains a reference in its returned integer/enum record.

## Decisions and value preservation

Symbol membership uses full Symbol Eq/Hash, including context and generated
identity. repeats counts a physical Symbol value already present in the probe's
set; it does not count revisits of the same shared Head/Args storage. Sampling
stops at eight repeats and chooses Shared, or at 256 nodes/exhaustion and chooses
Plain. SmallRoot and Plain both invoke the unchanged Plain conversion; Shared
invokes the unchanged bounded sharing conversion with capacity 128.

The probe produces no output nodes. Consequently each result graph is produced
entirely by one previously justified conversion. Selection does not create a
mixture of policies or introduce new alias classes. Value/variant projection,
independent Head/Args identity, COW, Weak liveness, owned extraction, Clone/Drop
and Send/Sync follow the chosen branch's conditional argument. Local scratch
state cannot escape or create cycles. Ordinary Call Drop remains recursive.

Since probe scratch has disappeared before construction, selective retained
requested heap must equal the selected branch's retained heap. This is checked
for both owners on every snapshot. Peak requests and allocation calls can grow
because the probe itself allocates; construction timing must include it.

## No profitability guarantee

The prefix can be unrepresentative: late repetitions beyond the sample are
missed, while eight early repeats may precede a mostly distinct tail. The root
size gate intentionally also skips small repeating expressions and deep graphs
with narrow roots. Even a detected repeated symbol might fall outside the full
sharing table's first 128 distinct keys, so detection need not imply cache hits.
The constants are fixed experimental choices, not workload-independent optima.

The late-tail control places 300 distinct symbols before 15,700 copies of the
root head Symbol f: the prefix chooses Plain although full sharing would hit the
already cached head. The early-tail control places eight f copies before 15,992
distinct symbols: the probe chooses Shared although only eight payloads are
saved. Neither control is presented as an actual evaluator workload.

Nine focused tests cover bounds, identity, alias/COW/Weak and both misleading
prefixes under native execution and Stacked/Tree Borrows Miri. The deep test
probes 10,000 levels natively and 300 under Miri, without constructing a compact
deep graph; it drains the original chain iteratively. Native corpus checks cover
130 snapshots, 780 value equalities plus scan/COW source stability; allocation
checks cover 910 windows. Miri checks the library tests rather than full runtime
Session/stdlib snapshots. Finite checks do not establish complete contextual
refinement or universally faster conversion.
