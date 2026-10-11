# Symbol sharing refinement argument

This is a manual argument for a safe conversion policy, conditional on the
pre-existing recursive representation API being sound. It is not a machine
proof of that unsafe owner's implementation or of the complete rum evaluator.
The new library and its bridge contain no unsafe operations. No private Symbol
fields, reconstructed integer addresses or unvalidated symbol ids are used.

## Observations preserved

The public recursive API observes atom variants/values, Call structure,
Head/Args alias classes, Args COW and Weak liveness, owned extraction, Clone,
Drop and Send/Sync. Symbol views return a Symbol copy, rather than a reference
to the private Body's Symbol field. Atom-payload Arc identity/counts are not
exposed. Allocation counts, footprints and timings deliberately change.

The source Symbol is copied as a complete value. Its Eq/Hash include its
interned text id and optional generated identity. Consequently the same
printed name in another context, a plain symbol and a generated symbol, and
two different generated identities remain separate cache keys. Equal full
symbols may share; no cache key is made from only the rendered name or id.
All non-Symbol atoms still call the ordinary atom constructor independently.
Manual Big variants, rational values and strings are not normalized by this
policy. The bridge's existing capacity normalization remains unchanged.

## Bridge and value refinement

The stack/memo bridge is derived from the pinned existing from_expr algorithm.
The analyzer checks that only the atom factory, function signature and imports
change, allowing formatting/import order. Plain and Shared instantiate this
same bridge. Head and Args memos are still keyed by the original live handles,
not by their values. Each first-seen handle constructs its own output Head or
Args; subsequent sightings reuse that handle's memoized output. Sharing Symbol
payloads therefore neither merges distinct Heads/Args nor splits an alias class.

For each atom factory invocation:

- A non-Symbol produces the same ordinary owned node as Plain.
- A cache miss produces an ordinary Symbol node containing the exact copied
  Symbol. An optional map entry holds a clone of that node.
- A hit returns a clone of a previously created node with an equal full Symbol.

All three cases project to the original atom value and variant. Induct over
the bridge's completion of physical nodes and vectors: child values and alias
classes are preserved, and Call reconstruction applies the same constructor.
This gives the same value/Head/Args observation as Plain on finite acyclic
input graphs, without expanding shared subgraphs into their logical tree.
The 80-level shared DAG test specifically avoids equality of an exponentially
expanded tree; it checks bounded conversion, shape and the terminal value.

## Ownership and lifecycle

On insertion the graph node and map entry each own a normal clone count.
On a hit the returned graph node owns one further normal clone count. The
map's temporary counts drop before shared returns; no map, input reference,
global cache or new borrowed lifetime escapes. Final graph owners keep the
immutable atom payload alive through the existing Clone/Drop operations.
Only normal safe constructors and clones acquire these counts. The change
adds no Arc raw-pointer reconstruction or manual count adjustment.

Args COW copies the vector and clones its node owners through the existing
API; replacing a slot drops exactly that slot's owner. Another vector's
equal Symbol remains valid. Mutable access to a cached Symbol payload is not
provided. Consuming a shared atom through into_parts obtains its value through
the existing shared-body clone fallback, leaving sibling owners alive.

WeakArgs still observes the Args Arc, which has not been coalesced. Weak-only
COW and strong-sharing COW retain the existing pointer/liveness behavior.
The explicit iterative release uses the old last-owner rule on each Body,
Head and Args. Shared Symbol bodies are leaves, so deferring their destruction
to their final owner does not create a recursive subtree or a reference cycle.
Ordinary Drop remains recursive for Calls; automatic deep-safe Drop is not
introduced. The old Send/Sync argument remains conditional on the same immutable
Arc ownership implementation; the local cache is gone before graph transfer.

## Limits and evidence

At most limit distinct Symbol entries are inserted. There is no eviction;
after the table fills, existing keys can still hit and misses build ordinary
nodes. The bound concerns entries, not exact HashMap capacity or bytes. The
map grows on demand and never reserves an arbitrary user-supplied limit in
advance. Zero selects Plain with zero diagnostic counters. Cache-enabled
construction includes lookup and diagnostic increment costs in the benchmark;
there is no global mutex or cache lookup in the node's read API.

Every source Symbol physical slot is visited once by the memoized bridge.
Visits = entries + hits + uncached misses. Safe/Compact statistics must match.
In the pinned x86_64 layout, each hit avoids a 48-byte Symbol Body allocation,
so Plain retained heap minus Shared retained heap equals 48*hits. This relation
is checked on all 127 pinned snapshots plus a repeated-symbol synthetic control.
It excludes the temporary map (already gone), root stack, original graph,
pre-existing intern pool and allocator overhead. Peak/call counters include
the temporary map and its extra construction work.

Eight focused tests run natively and under strict-provenance Stacked/Tree
Borrows Miri for both owners: full identities, bounded fallback, alias/COW/Weak,
non-Symbol variants, consuming a cached atom, DAG conversion, shared leaf release.
Deep explicit release covers 10,000 levels natively and 24 under Miri. Full
roundtrip and checksum checks cover 128 snapshots in the native study; COW
checks modified checksums and source stability for Call roots. Native accounting
checks 640 release windows. Miri does not run the full Session/stdlib corpus
program. These finite checks support this argument without establishing full
contextual refinement or universal performance improvement.
