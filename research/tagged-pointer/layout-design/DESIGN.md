# Expression layout alternatives

This study separates node width from ownership and allocation policy. Earlier
representations added an Arc payload to symbols and Calls that original rum stores
directly in Expr. These alternatives remove that cost where lifetime permits or
two shared handles fit in the node. They remain external research types, not
replacement kernel types or evidence of evaluator acceleration.

## Three initial alternatives

| Representation | Integer | Symbol | Call | Other atom |
| --- | --- | --- | --- | --- |
| Safe24 | Full i64 directly | Text handle and full generated identity | Independent Head and Args handles directly | Arc<Atom> |
| Tagged16 | Full i64 in a second word | Tagged text pointer and full generated identity | Two independently owned Arc pointers directly | Tagged Arc<Atom> pointer |
| Tagged8 | Signed 61-bit directly | Plain symbol's tagged static text pointer | Arc<Body> fallback | Arc<Body> fallback |

Names describe the intended and checked layout on the pinned x86_64 target,
not a portable Rust enum ABI promise. Safe24 uses a two-field 16-byte symbol
payload rather than the existing 24-byte Symbol. A safe enum discriminant adds a
word to its largest two-word payload. Tagged16 replaces that discriminant with
low address bits while retaining each pointer's provenance. Its second word
stores either an Args pointer, all integer bits, or all generated-identity bits.
Bits that represent numbers are never dereferenced or passed to Arc.

Safe24 controls Tagged16's policy: their comparison isolates discriminant
compression. Comparison with original rum measures the complete layout policy.

Tagged8 intentionally uses the previous broad Body allocation strategy for
generated symbols, Calls, and owned atoms. Ordinary symbols use permanent
interned text and require no new allocation or reference-count operation.
Generated identities cannot be truncated, so those symbols use fallback storage.
Small values outside the 61-bit range also fall back while preserving the
original Small variant. This is a conservative 8-byte candidate, not the lower
bound on the cost of all possible 8-byte representations.

Tagged16 avoids outer Call allocation and keeps symbol clones as copies.
Tagged8 saves a further word per slot but adds an Arc and indirection for every
physical Call and generated symbol. Call-heavy workloads can favor the larger
representation. Neither design needs caching to make ordinary symbols cheap.

## Safe Symbol access proposal

Original SymbolText instances are created by the interner and kept permanently;
their fields are private. Original Symbol equality and hashing use the text id
and optional generated identity. The id is redundant when a canonical text
handle is available, but it must be restored exactly when returning a Symbol.

A small additive safe API exposes `&'static SymbolText` and constructs a Symbol
from it plus `Option<NonZeroU64>`. It derives id from the handle's private id,
never from a caller-supplied integer, without re-interning, locking, or allocation.
The original Symbol implements context, equality/hash, ordering and serialization.
Generated identity retains all 64 bits; None is distinct from every valid identity.

Permanent lifetime becomes an explicit public contract. SymbolText remains opaque
without mutable fields or raw constructors. Private-layout casts, fabricated
pointers and unchecked ids are excluded. The extension is entirely safe Rust.

Candidate kernel files remain unchanged. A reviewable patch applies only to an
isolated rum-core copy. The dependency diff is disclosed and hash checked;
upstream acceptance of this API is a separate decision.

## Public observations and borrowing

Every representation preserves exact atom variants and values, including manual
Big values that fit i64, string contents, and full symbol identity. Head and Args
remain separate sharing identities. Conversion preserves alias classes rather
than old addresses, and never merges equal but separately allocated handles.
Args COW and Weak follow the existing standard Arc behavior. No atom-payload or
outer-Call allocation identity is exposed as part of the contract.

Direct Calls cannot reuse the old external View's references to physically stored
Head/Args wrappers. Lending a recovered temporary Arc wrapper has an invalid
lifetime. Reinterpreting raw words as wrapper references is unnecessary.

Borrowed projections expose the head, parameter slice, identity keys and owned
handle cloning. `node()`/`slice()` retain the source borrow lifetime after the
projection is dropped; views change no counts. Borrowed handles are Send/Sync
when N is Send+Sync. Owned recovery acquires one count. Child borrows prevent
mutation through the corresponding Args owner.

This changes earlier external research interfaces, not the unchanged kernel API.
A later integration must document its own source-compatibility boundary.

## Ownership obligations

Each tag must identify both its value interpretation and its ownership rule:

- Immediate integers copy bits and release nothing.
- Static symbol text pointers borrow permanently live immutable storage and
  release nothing; generated identity is numeric data.
- Owned atom pointers own exactly one Arc strong count.
- Direct Calls own two distinct strong counts, one for Head and one for Args.
- Tagged8 fallback nodes own exactly one Body count; that Body owns its children.

Tagging modifies addresses with provenance-preserving pointer operations. Every
pointer is restored to its exact Arc data address before count operations; text
pointers are restored before creating references. The alignment used for tags
needs a checked invariant, not an assumed allocator accident. Non-pointer words
use provenance-free addresses solely as bit containers. There is no public safe
raw constructor or mutable representation access.

Clone acquires the counts its returned owner will release. Consumption suppresses
the old owner with ManuallyDrop before transferring its counts into standard Arc
operations. No fallible callback may intervene while one of a Call's two counts
is unowned. Atom views and Call borrows cannot outlive the node. Send and Sync
require the same immutable, thread-safe payload conditions as their Arc storage.
Weak-only COW must disassociate old Weak observers; strong-sharing COW must leave
other owners' values and handles intact.

Explicit release may drain last-owned Head/Args iteratively. Ordinary Drop stays
recursive. Finite Miri/lifecycle tests are not a complete soundness proof.

## Allocation model and fair comparisons

On the pinned layout, Arc<Atom> requests 48 bytes: a 32-byte Atom plus the Arc
header. Original Big and Rational already contain Box allocations, and String
already owns a buffer. Moving those atoms into Arc does not replace their inner
allocations. If inner buffer sizes and capacities match, the new cost is the
additional 48-byte outer Arc, offset by smaller node slots elsewhere.

Let S count slots across unique Args vectors, H unique Head allocations, and O
physical atom nodes other than Small integers and Symbols, including every Big.
With identical alias classes, normalized Vec capacity, and matching inner
allocations, retained requested-heap deltas relative to original are:

```
Tagged16 - Original = 48*O - 16*(S + H)
Safe24   - Original = 48*O -  8*(S + H)
```

Check these predictions against counters. Root stack bytes are separate. A
Tagged8 broad fallback requests 48 bytes for each physical Call, generated
symbol, non-inline Small integer, and owned atom; its slot/head savings are
24*(S+H). Pre-existing intern storage, source graphs, allocator metadata, and
fragmentation are outside this model. Peak and call counts include bridge scratch
and can differ even when retained layout matches the formula.

Use equivalent construction tasks: original Expr handle cloning is not a fair
baseline for recursive conversion. A common alias-preserving bridge needs an
original output case, distinct from direct construction. Normalize capacities
consistently. Do not time original two-bridge normalization against one new bridge.

Traversal checks values; DAG-aware conversion verifies sharing without expansion.
Clone, extraction, COW and release are separate operations. Baselines include
cheap original Symbol copies. Timing builds exclude counters. Rotate cases across
multiple runs, validate checksums and preserve per-run ratios. These controls do
not establish statistical significance or eliminate host variation.

## Deferred alternatives and acceptance

A typed Tagged8 could separately tag Arc<CallParts>, Arc<SymbolParts>, and Arc<Atom>.
Two-word Call/SymbolParts request 32 bytes instead of 48. Each still adds an
allocation, and tag-specific ownership rules add complexity. Investigate this
after the conservative candidate if fallback cost matters.

Arena/generational indices may eliminate Arc counts and improve locality, but
cross-arena values, validity, reclamation, lifetimes, COW, Weak and thread transfer
need new contracts. Reset-only reclamation retains dead expressions indefinitely.
This is a separate API/lifecycle project rather than a transparent owning-value
optimization.

Initial acceptance requires checked target sizes, exact roundtrip and alias
behavior, full symbol/integer boundaries, owned-variant preservation, borrow
compile-fail cases, COW/Weak checks, native reclamation windows, and Miri under
both borrow models. Local memory/operation results then choose a representation
to investigate further, including accepting a safe representation when it wins.
Universal superiority is not required or assumed.

End-to-end performance needs a later rum-vs-rum evaluator integration preserving
documented behavior and reporting hot-label regressions. This isolated study
establishes neither an older C speed ratio nor complete unsafe-API soundness.
It does not compare against other vendors' engines.
