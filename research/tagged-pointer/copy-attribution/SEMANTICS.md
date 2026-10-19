# Deferred source-vector materialization

This is an isolated candidate against rum commit
`77029938f999b14dbe5b0755bebdf98e51517ae6`. The patch changes only
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`.
The Association branch, matcher, rule substitution, evaluation, and dispatch
are unchanged. No unsafe code or representation change is introduced.

## Ordinary traversal

The ordinary Call branch retains its source `SharedArgs` instead of eagerly
materializing a `Vec<Expr>`. `replace_call_children_owned` only reads that
source through `get(child_index).cloned()`. Its mutable output remains the
same independent `rebuilt_args` vector. Each selected child is still cloned
once before recursive replacement, in the same order; pending-outcome merges,
Sequence splicing, depth calculation, and rule application are unchanged.
The input is never mutated through the retained handle.

On successful completion this avoids the source-vector materialization.
It does not remove RHS construction or arithmetic normalization, and is not
evidence of a speedup or of the origin of an observed clone count.

## Suspension and roots

Both newly deferred `into_vec` calls are confined to `InputRequired` arms:

1. If replacing an included head suspends, the owned source handle is
   converted immediately before constructing the existing
   `ReplaceChildrenOwnedSuspensionFrame2`.
2. If replacing a child suspends, the handle is converted immediately before
   constructing the existing `SuspendedReplaceCallChildren` state.

At the first site no source-argument borrow exists. At the second site
`get(...).cloned()` has already produced an owned child; its temporary shared
borrow ends before `replace_owned` is called. That call returns an owned
result, and no reference into `args` crosses the subsequent conversion.
The borrow checker must confirm this when the isolated candidate is built.

`into_vec` either recovers the unique source vector or clones its elements;
`FrozenRewriteQueue::freeze` then retains all source elements exactly as in
the original suspended state. The cursor, rule plan, accumulated output,
flags, depth, and root-traced frame types are unchanged. The already-suspended
drivers and their retry behavior are unchanged. Thus this patch neither
removes a frame root nor replaces it with a borrowed reference.

The conversion happens later than before, so reference counts, allocation
timing, and physical storage identity intentionally differ. This is a
source-level preservation argument, not a formal proof of complete runtime
equivalence or of resource-limit behavior.

## Required validation

Before adoption, attribute clone sites and vector lengths in an instrumented
copy. Build the uninstrumented isolated candidate; run formatting, runtime
tests and the broader relevant suite; verify original source aliases remain
unchanged. Cover misses, nested calls, Heads, Sequence, delayed rules, errors,
resource limits, and mid-traversal suspension/resumption. Run the original
full rum-vs-rum timing suite serially and report all regressions. These gates
are obligations, not checks already completed by this template.

## Organization assessment

The pinned once.rs implementation is about 33k bytes and falls in the project's
split-assessment range. This experiment keeps the four ownership edits together
so its effect can be isolated from module/code-generation changes. It adds no
new traversal algorithm or inline tests to that file. Before a production
integration, a coherent future boundary is to extract the ordinary Call child
walker into a sibling call_children module, with the Association walker in its
own sibling, leaving the Replace entry and resolution orchestration in once.rs.
That organizational change needs its own timing gate; it is not folded into
this controlled experiment. The three new tests live in a separate test module.
