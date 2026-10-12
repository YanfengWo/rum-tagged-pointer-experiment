# Typed 16-byte representation

This is an external research package for pinned rum commit
`77029938f999b14dbe5b0755bebdf98e51517ae6`. It does not replace the evaluator.
The unchanged stage22 Tagged16 is the measured `previous16` control.

## Representation

`Typed16` is two private pointer words on the checked 64-bit target. The first
word's low three address bits discriminate these closed states:

| Tag | First word, after clearing tag | Second word | Ownership |
| --- | --- | --- | --- |
| 0 | Arc<Atom> data | unused | one strong count; string alignment fallback |
| 1 | no dereferenceable pointer | all i64 bits | none |
| 2 | permanent SymbolText | full nonzero u64 identity | permanent borrowed text |
| 3 | Arc<Typed16> head data | Arc<Vec<Typed16>> args data | two independent strong counts |
| 4 | Arc<str> byte data | exact byte length | one unsized strong count |
| 5 | Arc<BigInt> data | unused | one typed strong count |
| 6 | Arc<Rational> data | unused | one typed strong count |
| 7 | permanent SymbolText | cached u32 interned id | permanent borrowed text |

Heap tagging preserves pointer provenance with `map_addr`. Scalar words use
`without_provenance` and are never dereferenced. No u64-to-pointer provenance
recovery, transmute of core private layouts or fabricated Arc object is used.
Sized payload alignments are checked statically. Arc<str>'s byte pointer has no
public eight-byte alignment guarantee, so the constructor checks the actual
address before tagging. An unaligned pointer is recovered with its exact original
metadata and copied into the existing checked Arc<Atom> fallback.

## Observe the value needed by the caller

`view()` preserves the full AtomRef interface, including a complete Symbol.
It reconstructs symbols from authentic permanent text; it still reads the
interned id through SymbolText. `inspect()` returns a shallow read-only view
with `SymbolKey` for equality/hash identity. Ordinary symbols use the cached id
without chasing SymbolText. Generated symbols retain the whole identity and
load id from text because both pointer words are occupied.

SymbolKey is opaque, has no raw public constructor and no conversion back to
Symbol. Its equality/hash pair agrees with core Symbol's id/identity pair.
It deliberately has no Ord: core Symbol ordering uses context/name, so dense
numeric ids must not substitute for lexical ordering. Keys belong to the live
process interner; they are not serialization identities or cross-process keys.
Clients requiring names, context, serialization or ordering continue to use
the complete Symbol observation. This interface therefore needs deliberate
caller adoption; it is not a transparent speedup for every existing kernel.

## Payload and construction controls

Strings use Arc<str> instead of Arc<Atom> plus String's allocation. Big/Rational
move their sized value into the typed Arc, retaining their original enum variant
(even Big(7)). Their internal arithmetic buffers remain those of core values.
Cloning shares immutable leaf payloads. Consuming a shared leaf recreates the
core owned value; Arc<str> consumption always copies into a String.

`from_expr` borrows source atoms and constructs their final typed payload directly.
`from_expr_moved` clones each source Atom and consumes that clone. Despite its
name this does not consume the entire source graph. Comparing these two paths
exposes temporary String/Box allocations that a representation-only benchmark
could conceal. Original, new and actual previous16 all use the same work/memo
algorithm; source head and argument aliases remain independently shared.

Safe24 uses a safe enum with the same cached-symbol and typed-payload policy.
Its common borrowed head/args projection still contains unsafe internals. It is
a storage policy control, not a claim that this entire package has zero unsafe.

## Costs to distinguish

For physical node count P, heap slots S=P-1 (root is on the stack), original
retained requests R, strings with byte lengths L, and Big/Rational count B:

* Previous16: R + 48 * owned_atoms - 16*S.
* Typed16: R + sum(16 + round_up(L,8) - L) + 16*B - 16*S.
* Safe24: Typed16 + 8*S.

These equations describe the measured System request layout on the pinned
target. Arc<str> padding is not assumed for correctness. The fallback can change
the physical equation on another target. Input, root stack, permanent interner,
allocator overhead and RSS are outside retained-request measurements. Borrow
and cloned-Atom paths retain identical final graphs but can differ in transient
allocation count and peak. Original wrappers still pay disabled profiling
branches; new wrappers omit them. Shared leaf COW differs from original deep
copy. Timing changes therefore cannot all be attributed to tag width.

Default Drop remains recursive. The separate explicit release API iteratively
drains only uniquely owned containers, retaining the shared-owner semantics.
