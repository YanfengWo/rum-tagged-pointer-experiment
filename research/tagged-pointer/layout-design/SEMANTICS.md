# Value, alias and ownership argument

This manual argument is conditional on the pinned 64-bit target, checked
alignments, standard Arc/Weak raw-count and COW contracts, original Atom contracts,
and the reviewed Symbol extension in the isolated dependency copy. It is not a
machine proof, evaluator specification or universal performance claim. Candidate
kernel files and the original representation are unchanged.

## Abstraction and exact values

Define A(n) as the original Expr denoted by a node. Integer views denote
Atom::Integer(Small(i)); other atom views denote the exact original variant and
data. A Call denotes the recursively abstracted head and ordered arguments.
The definition is over finite acyclic physical graphs and can retain shared
subgraphs rather than materializing every logical occurrence.

Safe24 and Tagged16 store every Small i64 directly. Tagged16's second word
preserves all integer bits, including MIN and MAX; its numeric pointer carrier
is never dereferenced. Tagged8 stores Small values in [-2^60, 2^60-1] as
((i as usize) << 3) | 1 and recovers them by signed right shift. All other Small
values fall back to an owned Body and remain Small. Big(7) remains Big instead
of being normalized to Small. Strings, Rational and Big data retain their
original values; consuming extraction returns the original Atom variant.

Write a Symbol as (t, g), where t is the permanently interned context/name
handle and g is None or a full NonZeroU64 identity. Safe24 stores these two
components directly; Tagged16 tags t's pointer and puts g's complete bits in
its second word, with zero representing None. Tagged8 uses a direct text pointer
only for None and stores generated symbols in Body without changing them.
None, different contexts, and distinct generated identities never coalesce.

The safe Symbol extension returns an authentic opaque static handle and derives
the reconstructed Symbol id from it, without field-layout casts or caller-supplied
ids. Original equality/hash, ordering, rendering and serialization are preserved
without interner lookup. Permanent lifetime is an explicit API proposal.

## Alias relation and bridges

Head<N> owns Arc<N>; Args<N> owns Arc<Vec<N>>. Integer or Symbol heads still have
their own Head allocation. Independent Head and Args identities are observations;
atom-payload and outer-Call identities are not exposed. A bridge preserves an
alias class by mapping each live source Head/Args identity to exactly one output
handle of the same category. Equal values in distinct allocations remain distinct.

The common forward bridge serves original_copy and all new layouts. Children
precede Call assembly. Separate Head/Args memo keys are live source addresses,
used only for identity and never reconstructed or dereferenced. Borrowing the
source prevents address reuse, while memo values own output counts. The reverse
bridge uses the same rule over borrowed new Head/Args projections.

Induct over completed atom, Head, Args and Call tasks. Atom construction preserves
A directly. A first-seen Head/Args gets a fresh output handle containing already
completed values; later sightings clone that same handle. Call construction keeps
the two categories independent. Thus value, order and alias classes are preserved,
and nested DAG conversion does not repeatedly expand an already-completed vector.
Temporary maps disappear before return. Addresses are not preserved across the
representation boundary or between separate conversions. Weak observers remain
attached to their own representation's allocations.

## Closed representation and strict provenance

Safe24 has private Repr storage; its constructors establish the same direct
Small/Symbol/Call and Arc<Atom> policy as Tagged16. Tagged fields and raw constructors
are private, and Tree is sealed. Safe callers cannot manufacture a new tag, raw
pointer, inconsistent Symbol id, or unmatched count.

Tagged16 has four states. Tag 1 holds numeric Small bits and owns no allocation.
Tag 2 holds a provenance-preserving pointer to permanent SymbolText and numeric
identity bits. Tag 0 holds the exact live Arc<Atom> data pointer and one count.
Tag 3 holds the tagged Head Arc data pointer plus the unchanged Args Arc data
pointer, owning one count for each independently. Arc pointers and text pointers
have checked alignment sufficient for the reserved low bits. map_addr adds or
clears only those bits, retaining provenance. Restoration happens before access.
No numeric carrier is converted into a dereferenced or Arc-owned pointer.

Tagged8 tag 1 is a non-dereferenced provenance-free integer carrier; tag 2 is a
provenance-preserving permanent text pointer; tag 0 is the unchanged data pointer
of one Arc<Payload> count, with Payload containing Body. Compile-time
checks also cover SymbolText alignment. Its integer/static-symbol clone and Drop
do not touch Arc. Generated identities are never encoded in unused pointer bits.

For either tagged representation, a borrowed Atom reference is created only
while the containing node holds its count. It cannot outlive the node. Owned
storage is immutable through the public view, and mutation requires an independently
owned Atom obtained through consumption, or standard Args COW.

## Counts, consumption and borrowed projections

Each heap-owning Clone increments exactly the counts owned by its returned value.
Tagged16 Call Clone increments both Head and Args; Tagged8 Clone increments its
Body count. No fallible callback occurs after acquisition and before establishing
the new owner. Refcount overflow follows Arc's abort behavior, rather than exposing
a partially initialized owner through unwinding. Static symbols and integers copy.

Drop reconstructs ordinary Arc owners from the exact original data pointers and
releases each owned count once. Tagged16 Call Drop establishes both Arc locals
before dropping either, so ordinary unwinding during one release can also release
the other. No allocation is reconstructed for a numeric/static-symbol state.

Consuming methods use ManuallyDrop before transferring raw counts. A direct
Tagged16 Call returns its existing Head/Args without a new outer owner. Arc-backed
atoms and Tagged8 bodies use try_unwrap: uniqueness moves the value; otherwise
the still-live Arc permits cloning its value, followed by release of the consumed
count. Other owners remain valid. Standard Head/Args into_vec semantics likewise
move unique storage or clone shared storage. Each result owns its own lifetime.

HeadRef<'a,N> and ArgsRef<'a,N> contain an exact Arc payload pointer and a phantom
shared borrow. Their private unsafe constructors require a live immutable source
owner for all of 'a. Every public creation path ties 'a to &self: Head/Args.borrow
uses their Arc, and node.view uses the node's owned counts. No reference to a
temporary Arc wrapper is created and no wrapper layout is reinterpreted.

node()/slice() return references for the source lifetime 'a, so a reference may
outlive the small projection object but cannot outlive its source owner. Deref
can instead borrow for the projection's shorter lifetime. clone_handle acquires
one count while the source remains live, then immediately constructs the returned
standard owner. Reading a view acquires no count. Rust borrow checking excludes
source destruction or unique Args mutation while a source-derived reference is
still used. Compile-fail cases exercise atom escape, Call escape, consumption
with a live projection, and mutation with a live child borrow.

## COW, Weak, threads and release

Args get_mut and make_mut delegate to Arc. With other strong owners, make_mut
clones the vector and its node owners; their Head/Args aliases survive node Clone,
and unchanged owners retain their values. With only Weak observers, make_mut
disassociates those observers. Weak liveness and pointer identity use standard
Weak operations. Static symbol copies need no count, but generated Symbol values
remain independent by identity. Consuming one shared owned atom clones its data
when necessary, so it cannot expose mutable data shared with a sibling.

Tagged owners' Send/Sync arguments reduce to immutable Arc payloads, permanently
shared SymbolText, and numeric states. Atom and SymbolText bounds are checked;
recursive children follow the same closed invariant. Borrowed projections require
N: Send + Sync for both traits, matching the Arc ownership they can acquire.
Phantom shared borrows preserve their lifetime across scoped thread transfer.
The API exposes neither raw mutation nor interior mutable representation fields.

Explicit release consumes nodes on a work stack. A direct Call yields its handles;
a Tagged8 Body yields its handles only when its count is last. Arc::into_inner
then yields each Head/Args value only for its last owner, putting resulting children
on the stack. Shared owners release only their counts. In a single-threaded
last-owner run this drains the graph without recursive node destruction and
without unfolding its aliases. Owned atoms are leaves.

Ordinary Drop remains recursive, including last-owned Head/Args. Deep-safe explicit
release is not automatic, and other threads may release their owners by ordinary
Drop. Unwinding and allocation failure are not proven stack-independent. Stack
exhaustion may abort; no universal implicit destruction-depth guarantee is given.

## Evidence boundary

Native tests, strict-provenance Miri under both borrow models, compile-fail cases,
reclamation windows and source hashes check finite instances. Counts and sizes
belong in experiment records. Miri library tests cover neither the complete
Session corpus nor every execution. This argument proves no full raw-Arc memory
safety, contextual refinement or resource-exhaustion behavior. Revisit its
conditions whenever layouts, APIs, ownership or targets change.
