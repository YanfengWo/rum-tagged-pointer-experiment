# Value preservation and resource-observation boundary

The candidate is six inserted lines in the private `flatten_flat_call_args`
helper. The only caller in the pinned source is the Flat arm of
`normalize_call_args_by_attributes`. It runs after argument preparation and
before the existing Orderless sorting and List Nothing handling. No dispatch,
matching, RHS substitution, evaluation, tracing or continuation code is changed
by the optimization patch. The additional instrumentation is a separate
diagnostic overlay and is never a production timing binary.

## Conditional sequence lemma

Let a head be h and an owned argument sequence A = [a1,...,an]. The original
helper returns the concatenation of these blocks, in order:

- children(ai), when ai is an immediate Call whose head is structurally equal
  to h;
- [ai], otherwise.

If no immediate argument has head h, every block is [ai]. Their concatenation
is precisely A. Returning the original owned Vec therefore preserves every
element, order, expression value and existing child owner identity. It moves
no child into another owner and performs no child clone or evaluation.

The predicate uses the same pure structural head equality as the original
match guard. It does not call language definitions. If the predicate fails,
the original one-level flatten loop runs unchanged. In particular it does not
replace the language's existing one-level flattening with a recursively
flattened alternate algorithm. A positive nested-Flat control exercises this
fallback in all four variants.

The additional `capacity == len` eligibility condition prevents retaining an
oversized input buffer merely to avoid reconstruction. The returned capacity
is exactly the output length. This does not assert that capacity is always the
same as the original helper's newly grown output Vec.

## What this does not prove

Resource observations are a real boundary. `expr_memory_proxy_bytes` in
`runtime/debug_tools/resources/memory.rs` includes spare argument capacity.
ByteCount, MemoryInUse and cooperative memory ceilings depend on that model.
The docs call it a deterministic structural proxy, not RSS or allocator data;
they do not supply a cross-version numerical-equivalence guarantee.

Measured resource examples in a separate source process are:

| Expression held by a Flat+Orderless user head | Original ByteCount | Candidate ByteCount |
| --- | ---: | ---: |
| f[a,b] | 216 | 152 |
| f[a,b,c] | 224 | 192 |
| f[a,b,c,d,e] | 368 | 272 |

Full expression values and enclosing-List alias checks pass. These changed
numbers mean that a program branching on ByteCount, or a constraint near a
modeled-memory threshold, can observe different behavior. Tight threshold
behavior is not exhaustively characterized. Consequently the candidate is
**not** claimed fully contextually equivalent under all resource-observing
programs.

A separate construction-history probe establishes that the unchanged original
already gives different ByteCount values for expressions whose SameQ check
passes: a literal Flat+Orderless two-argument call reports 211, while the same
value constructed with Apply reports 147. The Flat candidate reports 147 for
both. This supports the representation-sensitive capacity model; it does not
prove resource-branch or tight-threshold equivalence. The candidate's changed
numbers are consistent with that model, rather than evidence of an incorrect
ordinary expression value. Resource behavior remains a separate validation
boundary from value preservation and the deferred performance gate.

The diagnostic counters themselves allocate maps and strings and modify code
layout. Their capacities and logical counts are not a total allocator-byte
measurement. No elapsed-time comparison uses these binaries. Fixed-point
root clones, guard scanning, delayed RHS evaluation, dispatch precedence,
custom definitions and runtime capability checks retain their original code.
The guard/body acceptance case still checks that all guard work precedes final
body evaluation. Lexical diagnostic phases cover synchronous probes only;
they are not continuation state across InputRequired/resumption.

The four diagnostic variants each pass the existing workspace's 2241 checks
(7 ignored,29 result groups), including the three preceding copy/alias/order
regressions. The 24 generated sources also validate complete expected values
and input aliases on an uninstrumented baseline and every diagnostic variant,
with counter-on/off output equality. This is finite validation plus the local
sequence argument, not a proof of all evaluator contexts or API soundness.
No unsafe is added and the independent u64 proof work is unchanged.
