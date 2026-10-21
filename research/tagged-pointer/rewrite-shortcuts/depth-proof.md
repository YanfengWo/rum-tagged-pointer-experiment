# Input-depth shortcut for nonnegative Replace levels

This is an isolated, safe-Rust research candidate. `depth.patch` changes only
the input-level-selection calculation in `replace_owned`; it does not change
the default checkout's runtime. Apply it to a disposable source copy. Copy
`depth-tests.rs` to
`crates/rum-runtime/src/runtime/tests/rewrite_depth_shortcuts.rs` and append
`mod rewrite_depth_shortcuts;` to that copy's `runtime/tests/mod.rs`.

## Selection proof

The current source is
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`,
`Session::replace_owned`, immediately before `levels.matches(level, depth)`.
The predicate is implemented in `runtime/model/traversal.rs`:

1. `CasesLevelSpec::matches` converts the unsigned traversal level and depth
   to signed values, saturating unrepresentable values to `isize::MAX`.
2. Its special mixed-sign branch requires `min < 0`.
3. Otherwise it returns `contains_signed(level) || contains_signed(-depth)`.
4. Every finite expression has depth at least one, with or without heads:
   atoms return one; calls return one plus a maximum that is at least one.
5. If `min >= 0`, signed `-depth` is strictly negative, including after the
   signed saturation. It cannot satisfy `level >= min` in `contains_signed`.
   Thus `matches(level, actual_depth) == matches(level, 1)`.

The shortcut supplies one only to this predicate for `min >= 0`. Exact zero,
positive singleton/range specifications, `All`, and `Infinity` satisfy the
condition. Inverted ranges are also safe: the negative-depth branch still
cannot match. For `min < 0`, including pure-negative and mixed ranges, the
original expression-depth helper runs unchanged. `isize::MIN` negative
bounds therefore retain the original `checked_abs` handling.

## Data flow and observable behavior

The local input `depth` variable is consumed only by `levels.matches`. It is
not copied into `ReplaceOutcome` and is not captured in a suspension frame.
The following paths retain exact depths:

- A successful rule application computes the replacement expression's actual
  depth, both live and in `ReplaceOwnedSuspensionFrame::resume`.
- Unchanged atoms return one. Child traversal combines exact child depths.
- Changed `Sequence` splicing recomputes appended children’s actual depths;
  `SequenceHold` and parent-attribute checks remain unchanged.
- The kernel's explicit ``System`Association`` path preserves its existing
  value-oriented traversal and depth handling. Its Rule-entry reconstruction
  copies each original key. The negative-level depth helper still canonicalizes
  actual associations as before. Ordinary calls, including a user symbol named
  ``Global`Association``, keep ordinary child traversal. Head inclusion and
  compound-head depth remain unchanged.
- The `ReplaceOutcome` continuation payload and every stored frame field
  remain unchanged. A resumed walk invokes the same `replace_owned` method
  for subsequent nodes; a suspended rule application finishes through the
  existing frame and does not repeat level selection.

Selection booleans are equal, so the same rule plan is invoked at the same
nodes, in the same order. No target/rule/option evaluation, matcher call,
binding snapshot, condition, RHS evaluation, child merge, resource check, or
host-input request moves or disappears. The omitted helpers inspect only
the immutable expression; they receive no `Session`, do not evaluate code,
and do not poll limits. Temporary association decoding/cloning may disappear,
but output construction and retained argument capacities are unchanged.

In particular, rule application substitutes the RHS without eagerly evaluating
its ordinary body. Matcher guards can run during the walk; the final evaluator
then evaluates the reconstructed result. The guard/RHS regression distinguishes
these phases with a decimal side-effect log: three declining guards produce
`123`, followed by three selected ordinary RHS bodies producing `123999`.
Interleaving each RHS with its node's guard would incorrectly produce `192939`.

This is a source-level value/control-flow argument, not a claim that programs
observe identical elapsed time, allocator traffic, or behavior at an exact
wall-time deadline. Existing active limits still execute at the same logical
checkpoints; faster execution can naturally alter a real-time timeout result.
No cross-node cache, pointer identity table, layout change, or unsafe code is
introduced.

## Expected mechanism and validation boundary

Stage 29's flat `increment` workload records `5N + 1` depth-helper node visits:
the root input scan visits `N + 1`, the N child input scans visit N, and N
successful replacement-depth scans visit `3N`. This shortcut should remove
the first two terms, leaving `3N`. It leaves the `2N` matcher entries and
binding snapshots unchanged. For a positive-level no-match flat list, the
`2N + 1` input visits should disappear completely. These are operation-count
predictions, not measured time savings. Deep trees can avoid repeated subtree
scans, whereas negative levels retain their original cost.

The eight proposed runtime regression functions cover positive/root selection,
negative and mixed ranges, head inclusion, association keys, Sequence parent
attributes, guard/RHS ordering, input suspension/resumption, and RHS recursion
limits. They must pass on both base and candidate before timing. Full workspace
tests and an unchanged per-label core microbenchmark A/B remain required.
No test, build, or timing result is asserted by this document.

The kernel-only test fixture installs control and rewrite builtins but does
not install the standard library's Association constructor. An unqualified
`Association` in that fixture resolves as a Global ordinary call. The original
baseline rewrites integer Rule keys in that ordinary call; it does not exercise
the System association-specific branch. The corrected regression records that
fixture behavior separately and explicitly qualifies ``System`Association`` for
positive/negative tests of the actual association walk. A CLI with the standard
library installed resolves ordinary `Association` syntax differently. This
fixture distinction is not a change to the language's association semantics.

The existing `once.rs` is 33,446 bytes before this research patch, already in
the repository's split-plan range. The four added net lines specialize one
existing predicate and add no state or separate responsibility. This isolated
experiment keeps that function in place; a broader traversal refactor would
need a coherent live/resumed-walk module split rather than an arbitrary cut.
The proof and test modules each remain under 10 KB.

Before broader integration, the split plan is to extract the pure depth helpers
and level-selection policy into a small sibling module, then move association
child traversal together with its matching resume helpers. Keep generic walk
state and its continuation encoding together in `once.rs`. Perform and measure
that structural change separately so it cannot be confused with this shortcut's
timing effect.
