# Recursive representation relation and ownership argument

This is a manual argument for this external prototype, not a machine-checked
proof of Rust soundness or a replacement specification for the rum evaluator.
The upstream kernel, serializers, equality/hash implementations and embedding
interfaces remain unchanged. Assumptions include the Rust 1.85 standard Arc
contract, the 64-bit pointer target and the original Atom types' contracts.

## Value and alias relation

Write A(n) for a node's logical original Expr. A tagged immediate i decodes to
Atom::Integer(Small(i)). A heap Atom preserves its exact variant and data; manual
Big(7) stays Big. A heap Call maps its Head and Args recursively to the original
Call. Symbols retain generated identity. Rational data, full i64 values and
string contents (including embedded NUL) are preserved without normalization.

The safe control uses exactly the same recursive Body, Head and Args policy,
inline range and allocation boundaries. Its enum discriminant replaces pointer
classification; its child slots are 16 bytes instead of 8 on the tested target.
Original Expr is 32 bytes and stores Call inline: both recursive wrappers add an
Arc payload for each non-immediate value, which can increase allocation costs.

Head<N> is an independent Arc<N>, even for immediate integer heads. Removing it
would erase observable head-sharing identity. Args<N> is Arc<Vec<N>>. Atom payloads
and Call bodies are immutable; argument mutation uses only standard Arc COW.
These choices deliberately preserve shared-head/shared-list identities at the
cost of extra payload indirection, instead of assuming slot savings are free.

Each bridge uses separate memo maps for Head identities and Args identities.
Original-to-new keys are original Arc data addresses; new-to-original keys are
new Head/Args Arc addresses. Addresses are identity keys only, never dereferenced
or turned into pointers. The entire borrowed input remains alive, so live keys
cannot be reused for another allocation during that bridge. Memo values own
counts, keeping completed output aliases alive until the final root is built.
Thus repeated occurrences of the same input Head/Args map to the same output
Head/Args. Equal values from distinct input allocations are not coalesced.

Both bridges process nodes using explicit stacks. Completed children precede
their Call, and their ordered suffix is drained to create its Args. The memo
prevents exponential unfolding of nested DAGs. Bridges do not preserve original
addresses across the representation boundary, or share results across separate
bridge invocations. Existing original WeakSharedArgs observes only original
allocations. New WeakArgs observes only new allocations. Reverse conversion
creates another namespace, preserving alias classes within that result.

This is observational value/alias preservation, not binary-layout compatibility.
No original serializer, Hash, Eq, mutable atom projection or HostSession is
implemented for the new types. Shared immutable strings change the cost of Clone
and COW relative to original Expr, which clones owned string buffers.

## Private packed owner

Packed<T> has two closed states. I(i) uses bits ((i as usize) << 3) | 1 for
[-2^60, 2^60-1], created with without_provenance_mut. Bits are nonzero and never
dereferenced, passed to Arc, or used for allocation access. Arithmetic right
shift on the signed 64-bit word recovers i. Full i64 values outside this range
use heap Atom storage. No safe public raw constructor or field mutation exists.

H(p) is Arc::into_raw's unchanged data pointer to Payload<T>, aligned to at least
8 by repr(align(8)). Its low three bits are zero and its provenance is preserved.
One Packed owner owns exactly one strong count. &self holds the count alive for
view(), whose returned references are bounded by self. Clone acquires exactly
one count before returning another owner. Ordinary Drop reconstructs exactly one
Arc with from_raw and releases it. Immediate Clone and Drop touch no allocation.

into_owned uses ManuallyDrop before transferring the owned count to Arc. A last
owner moves out the Body; otherwise it clones Body through Arc::try_unwrap's
returned live owner. take similarly transfers its count to Arc::into_inner,
returning the Body only for the last owner. Neither path releases a count twice.
After raw-count acquisition or transfer there is no fallible user callback
before establishing ownership. Clone overflow has Arc's abort behavior.

Packed<T>'s Send and Sync implementations both require T: Send + Sync, matching
Arc's requirements. Compile-time production assertions enforce these bounds for
both recursive node types. Their recursive immutable structure has no interior
mutable state. The generic packed constructor is private; no public arbitrary
payload API or unbounded Send implementation is introduced.

## COW, borrows and release

view() projects Small/Big, Rational, String, Symbol and recursive Call without
owned clones or allocation. Its lifetime follows &self. A live child borrowed
through Args prevents mutable access to that Args handle. get_mut follows Arc's
strong/weak uniqueness checks. make_mut clones child owners when strong-shared;
with only Weak observers it disassociates those observers. Weak liveness and
pointer comparison use standard Weak methods. No raw mutation is used.

into_parts moves a unique Call's handles, or clones their counts when shared.
Subsequent Args COW preserves other Call owners. For Atom it moves/clones owned
Atom data; this permits mutation only after acquiring a separate owned value.
Public Tree is sealed, preventing arbitrary implementations in the bridge.

release_iterative consumes its root and uses a Vec work stack. A last payload
owner yields its Body; last Head/Args owners similarly yield children. Shared
owners release only their count. Each reachable last-owned Body is processed
once, so no recursive destruction is needed along this explicit release path
in a single-threaded last-owner run. Weak counts do not keep values alive. Other
threads may independently use ordinary Drop on their owners.

Ordinary Drop remains recursive. Deep conversion does not make implicit cleanup
stack-independent: callers needing the explicit path must invoke it for the
last owners. Ordinary Drop was tested at depth 128, conversion and explicit
release at depth 10000 on the tested native target; Miri uses smaller depths.
These are test points, not universal depth limits. Unwinding and allocation
failure are not proven stack-independent. Stack exhaustion can abort a process;
this prototype does not promise arbitrary-depth automatic destruction.

## Evidence boundary

Native semantic tests, borrow-escape compile-fail cases and strict-provenance
Miri under both borrow models exercise these obligations. A nested shared DAG
test has 80 native levels (2^80 logical leaves), verifying conversion preserves
aliasing instead of expanding it. Miri uses eight levels. Allocation-counter
code is separate, disabled by default and separately exercised under Miri.
All checks remain finite. The independent u64 formal model does not cover this
heterogeneous recursive Arc implementation. No evaluator speedup is inferred.
