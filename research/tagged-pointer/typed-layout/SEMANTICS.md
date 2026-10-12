# Conditional semantic and ownership argument

This is a manual argument tied to the implementation and assumptions below.
Native tests, compile-fail examples and strict-provenance Miri exercise it;
they are not a machine-checked proof or exhaustive soundness certification.
The earlier independent u64 proof does not automatically prove this owner.

## Assumptions and refinement map

The pinned core interner issues distinct u32 ids and retains immutable SymbolText
for the process lifetime. The proposed safe accessors return authentic text and
reconstruct Symbol using text's own id, never a caller-supplied id. Its additional
`SymbolText::interned_id` accessor exposes an immutable numeric observation.
All three methods are appended only in the isolated core source copy.

Let D map a live node to a core Expr value by its tag: the Small i64 bits map
exactly; plain/generated symbols map from the authentic text and full identity;
string bytes and length map to an equal owned UTF-8 String; Big/Rational map to
their original variants and values; Call maps recursively to its head and
ordered args. D ignores physical leaf sharing but respects observable values
and the independent head/args sharing handled by the bridge. No normalization
of Big(7), integer truncation or generated identity truncation occurs.

All safely constructible states satisfy:

1. The discriminant matches the allocation type or scalar kind.
2. Each owned raw Arc pointer represents exactly one outstanding strong count
   belonging to this owner. Call represents two counts.
3. Tagged sized pointers have three zero low bits before tagging. The str
   branch checks this dynamically and stores its exact byte length.
4. Tagged pointers retain their original provenance. Cleared tag pointers and
   reconstructed str metadata recover the exact original allocation payload.
5. Plain's cached id equals its authentic text id; generated's identity is
   nonzero and complete. Text is permanent and immutable.

Private fields, private raw constructors and the sealed Tree trait prevent
safe clients from forging a state. Cached ids are derived only from Symbol.
Safe24 establishes corresponding value/ownership invariants using its enum.

## Operation obligations

Construction transfers ownership with Arc::into_raw. Each typed heap branch
chooses the matching tag before returning. str's unaligned branch recovers the
original fat pointer exactly, then creates the checked Atom fallback; no altered
pointer is dereferenced. Call has no fallible callback between its two transfers.
Small/static states transfer no refcounts.

Cloning a heap node increments exactly the selected typed count. A source
`&self` keeps it live. Call increments both counts, then copies the private words.
Arc count overflow aborts rather than unwinding midway. Scalar/static states copy
without count edits. Thus D(clone(n))=D(n) and both owners independently release.

Borrowed views recover references under the source owner's lifetime and never
edit a count. The saved str byte length derives from valid immutable UTF-8.
HeadRef/ArgsRef store NonNull payload pointers with lifetime markers, not fake
Arc references. `.node()` and `.slice()` may outlive their temporary wrappers,
but never the source borrow. Compile-fail examples cover escapes, consuming a
borrowed owner and mutating borrowed args. Clone_handle increments a live count
and immediately transfers it into a real Arc.

Inspect's scalar key agrees with Symbol's equality/hash pair; it is intentionally
not a refinement of name lookup or lexical Ord. Full view and D continue using
the authentic text, not the cached id to manufacture a Symbol. Plain cache
consistency is inductively maintained by construction and word-preserving clone.
Here hash agreement means equal keys hash equally within the key type, not that
SymbolKey and Symbol feed identical bytes to a hasher. Their Hash implementations
differ; heterogeneous map lookup and hash-operation speed are not established.

Consumption suppresses the old owner using ManuallyDrop and transfers each raw
count to an exact typed Arc local. Call transfers both to Head/Args. The sized
leaf either unwraps unique payload or clones the shared value before returning
the core Box variant. str creates the core String while its recovered Arc is
live, then releases that count. On unwind, recovered locals release their counts;
the suppressed owner is not released twice. There is no user callback in the
two-count recovery interval.

Drop recovers and releases exactly those counts; Call recovers both locals before
dropping either so ordinary unwind cleanup covers the other. No other state has
a pointer-owning second word. Static/scalar states release nothing. Default
recursive Drop can exhaust the stack on very deep trees; explicit iterative
release is a separate API with its own last-owner rule. This is a limitation,
not a proof of stack-independent default destruction.

Args::make_mut delegates to Arc's COW contract: unique args can mutate directly;
shared args clone nodes (thus typed counts), and weak-only aliases are dissociated.
Prior views enforce an immutable borrow of the source owner; safe code cannot
consume/mutate that source during the view. Edits preserve the original graph's
value. Alias-preserving conversion memoizes head and args independently; project
preserves their sharing through corresponding independent caches.

Typed16's unsafe Send/Sync requires immutable Arc payloads with Send+Sync bounds,
permanent immutable text and never-dereferenced scalars. Borrowed handle bounds
match their shared Arc payload requirements. Native scoped-thread tests cover
owner transfer and borrowed arg clone; concurrency execution is not exhaustive.

## Evidence and gaps

Tests cover complete i64 boundaries, identity 1/u64::MAX and contextual symbols,
manual Big(7), large Big/Rational, empty/Unicode/NUL strings at length boundaries,
shared clone/consume, independent aliases, COW/weak-owner behavior, thread
transfer, and explicit deep release. Both Stacked and Tree Borrows run with
strict provenance. Full snapshot projection equality, full/key checksums,
edited/source COW equality and allocation return-to-baseline are checked.

The pinned allocator yields aligned str data in measured windows. The fallback
is present and manually reviewed but has no injected unaligned-Arc fixture.
Miri coverage is bounded and does not quantify every safe context, allocator,
target, LLVM optimization, panic path or interleaving. A full contextual-refinement
proof, independent unsafe audit, target fallback policy and evaluator-level
rum-vs-rum performance gate remain work to do before production adoption.
