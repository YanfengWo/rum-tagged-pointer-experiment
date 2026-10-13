# Native dispatch cache: semantic boundary

Only the BuiltinDispatchSpecCache alias changes in isolated variants. Session
lookup, builtin registration/invalidation and result handling remain original.
Keys are born at insertion from a complete Symbol; get receives the existing
Symbol directly and returns the original Option<BuiltinSpec> value. No Expr
conversion or full-symbol output reconstruction is performed in a cache hit.

The public map API is safe and forbids unsafe in this wrapper. Its tagged owner
dependency still contains the previously audited unsafe representation. Native
safe24 uses the same policy; packed16 stores only id and full identity in two
u64 words and requires no pointer encoding. Raw24 controls the container/wrapper
change separately from width. Native-cache lookup needs equality/hash identity,
not names, lexical ordering, serialization or a general expression value.

All representations identify keys by authentic interned id and identity. Plain
identity maps to zero; generated identities are nonzero and retain every bit.
Equal pairs are precisely core Symbol equality. The hash stream is the same
single u64 write as pinned Symbol::hash. Query uses hashbrown's safe Equivalent
interface: no fake borrowed Symbol/Arc/struct reference, private-layout cast or
unsafe raw table entry is fabricated. Stored and query hash/equivalence agree.

Forced-zero-hash tests exercise collision correctness, replacement, context and
identity distinction, cached None versus absent key, clear/invalidation and
u64::MAX. A byte-recording hasher checks the exact core stream. Native sizes are
asserted for every policy. Strict-provenance Miri covers both borrowing models;
the unchanged full workspace checks run for each isolated native variant.

The tagged Symbol owner keeps authentic permanent text and scalar identity;
views and destruction are limited to Symbol states by its private wrapper's
constructor. The dependency's broader owning/borrowing proof remains bounded
and is not replaced by these cache tests. No complete API soundness proof or
cross-platform guarantee follows. Default core/runtime files remain unchanged;
the patch recipe makes the small integration reviewable in a separate copy.
