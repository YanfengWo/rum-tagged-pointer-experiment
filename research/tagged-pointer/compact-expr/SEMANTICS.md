# Representation relation and unsafe obligations

This is a manual argument for the external prototype, not a machine-checked
proof of Rust soundness. It does not inherit the separate single-owner u64 proof.
The trusted foundation includes Rust's raw-pointer address operations, Arc's
documented raw ownership operations, and rum's existing Expr implementation.

## States and relation

An owner has exactly one of two private states on a 64-bit target:

* I(n): low three bits are 001, the upper 61 bits encode the signed integer n,
  where -2^60 <= n < 2^60. The pointer has no allocation provenance and is
  never dereferenced or passed to Arc.
* H(p): low three bits are 000; the stored pointer is exactly the live
  Arc<Payload> data pointer p with its original provenance. The owner holds one strong
  count. Payload is aligned to at least eight bytes and contains an immutable
  original Expr. Tags other than 001 and 000 are unreachable through safe APIs.

The abstraction relation alpha maps I(n) to Expr::small_integer(n) and H(p) to
the expression in p. The safe control uses the same relation, immediate range,
Payload allocation, and Arc sharing policy. The relation includes nested rum
symbol identities and SharedExpr/SharedArgs identities; it does not promise
the identity of the new outer Payload allocation or a borrowed &Expr for I(n).

The signed round trip follows fixed-width arithmetic: for a 61-bit signed n,
shifting its 64-bit two's-complement encoding left three discards only sign
extension bits. OR with 001 affects only the discarded low bits of the inverse.
Interpreting as i64 and arithmetic-shifting right three therefore recovers n.
The encoded address cannot be zero because its low bit is one. Integers outside
the range use H(p); no truncation occurs, including i64::MIN and i64::MAX.
An IntValue::Big is never normalized by this adapter, even if manually built
with a numerically small value: preservation includes the original variant.

## Preservation by public operations

`new` recognizes only the original Small variant within the range. In I it
creates a nonzero provenance-free address. In H, Arc::new establishes ownership,
Arc::into_raw transfers its count. The aligned, nonzero data pointer is stored
unchanged, so no heap address restoration is needed. The original expression
is moved intact into Payload.

`view` decodes I without touching memory. In H, it uses the unchanged Arc pointer
with its original provenance. The owning count keeps Payload initialized and alive;
the returned immutable reference has the borrow lifetime of &self. A caller
cannot drop that owner while using its borrow. Another owner can be dropped
concurrently because it releases only its own count. No mutable alias is exposed.

`clone` copies I. In H, &self guarantees a live count during increment_strong_count.
After increment, the new owner holds the added count. No allocating operation
or user callback can unwind between increment and construction of that owner.
Counter overflow follows Arc's abort behavior. Both owners have the same alpha.

`to_expr` reconstructs a Small integer or delegates to the original Expr::clone.
Thus it preserves original structural semantics and original child-sharing rules.
Hashing and serialization can be delegated to this original value; this prototype
does not supply replacement Hash/serde implementations or source compatibility.

`into_expr` transfers the unique owner's count to a temporary Arc, disabling
CompactExpr::drop with ManuallyDrop. Arc::try_unwrap moves out Expr if unique;
otherwise original Expr::clone gives an owned value and dropping the temporary
Arc releases only the consumed owner's count. If cloning panics, normal Arc
unwinding still releases that count. Both cases preserve alpha and child COW.

`drop` does nothing for I. For H it reconstructs and drops one Arc, consuming
exactly the owner's count. Arc releases the payload at the last strong drop and
keeps weak metadata alive for remaining Weak observers. A forgotten owner leaks
its count safely; eventual deallocation is not a soundness precondition.

`borrow` projects `view` into ExprRef / AtomRef / IntRef entirely in safe Rust.
An immediate becomes IntRef::Small(n). Heap Small integers, including values
outside the inline range, are also projected by value; a manual Big(7) stays
IntRef::Big and its as_i64 result stays None. Strings, BigInt and rationals borrow
their original storage. Symbols copy the original static-text/identity handle.
Calls borrow the original SharedExpr and SharedArgs handles, preserving child
identity and Weak liveness. No owned clone or allocation occurs in projection.
All references are lifetime-bounded by the original immutable view; copying the
view does not extend a reference's lifetime. The typed string escape is checked
by a separate compile-fail doc test. Recursive children still use original Expr.

`Send` and `Sync` rely on immutable Payload containing Send + Sync Expr, enforced
in production by a compile-time bound. No thread-local state or non-atomic
reference counter is introduced. Each thread owns its count; borrowed access is
immutable. These impls must be re-audited if mutable access or payload types change.

## Scope and remaining obligations

The raw pointer, tag tests, and ownership operations are private; no safe API
accepts external raw pointers or tags. There are five unsafe expressions across
borrow, count increment, ownership transfer and release (including the test-only
Weak observer), plus two unsafe trait impls. The test observer borrows an existing
count using ManuallyDrop<Arc<_>> solely to create a Weak; production exposes none.

Miri exercises selected paths under strict provenance with both Stacked Borrows
and Tree Borrows. This supplements the argument; it does not quantify over all
safe clients, thread schedules, future compiler models or platforms. Native
tests also reject a borrow escaping its owner. A complete proof needs a formal
Arc/raw-pointer model and a quantified representation/lifetime refinement.

The wrapper compresses top-level slots only. Call children remain original
32-byte Expr values. A heap fallback adds an allocation and atomic counts;
string construction/unique conversion can therefore cost more than original
Expr. Recursively compact children, borrowing pattern matches, equality/hashing,
serde, weak handles, deep drop behavior, and the HostSession embedding API still
need migration design and evaluator-level rum-vs-rum performance gates.

## Encoding refinement from v1

The published v1 stored H(p) at address p | 010 and recovered p by clearing
the low three bits with with_addr. V2 stores H(p) unchanged with low bits 000.
The abstraction relation alpha, integer encoding, payload and strong-count
ownership are unchanged. Alignment and Arc's non-null invariant distinguish
H(p) from I(n) with the same low-bit test, so each public operation simulates
the same original operation. Encoded addresses are private and are not serialized
or observable through the API. This removes a heap masking operation and a
provenance-preservation obligation; it does not eliminate raw Arc obligations.
The pinned v1 source and same-process comparison harness are retained separately.
