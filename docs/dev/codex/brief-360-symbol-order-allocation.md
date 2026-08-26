# brief-360 — symbol ordering allocates two `String`s per comparison

## Goal

Sorting symbols costs 5.9-6.7x what sorting integers costs, for the same
element count. Measured on `ada528c4`:

| n | symbols | integers | ratio |
|---:|---:|---:|---:|
| 4 000 | 621.7 us | 93.5 us | 6.65x |
| 8 000 | 1 312.7 us | 200.4 us | 6.55x |
| 16 000 | 2 836.8 us | 477.7 us | 5.94x |

The cause is `compare_atom_order` in
`crates/rum-stdlib/src/structure_ops/ordering_sets.rs`, whose `Symbol` arm is

```rust
(Atom::Symbol(lhs), Atom::Symbol(rhs)) => lhs.to_string().cmp(&rhs.to_string()),
```

`Symbol::to_string()` goes through `qualified_name()`, which allocates a
`String` per side per comparison — two allocations for every one of the
n log n comparisons.

**The sort order must not change by one element.** This is a cost repair.

## The trap — read this before designing anything

`Symbol` has an allocation-free `Ord` (`rum-core/src/expr.rs`, comparing
context then name then identity). **It is not the same order** as the
qualified-name comparison used today, and swapping to it is therefore an
answer change, not an optimization.

Verified on the current build:

```text
Sort[{Global`z, Global`x`a}]   ->   {Global`x`a, z}
```

Today's rule compares the joined text: `"Global`x`a"` versus `"Global`z"`
differ at position 8, `x` < `z`, so the nested-context symbol sorts first.
`Symbol::cmp` compares contexts first, and `"Global`"` is a prefix of
`"Global`x`"` and therefore smaller, which would put `z` first. Different
answer.

So the repair is: **reproduce the joined-text byte order exactly, without
allocating** — compare the context bytes, then continue into the name bytes,
as if the two strings had been concatenated. Do not substitute `Symbol::cmp`,
and do not "improve" the ordering.

## Deliverable

- An allocation-free comparator producing byte-identical ordering to
  `qualified_name()` comparison, used by the `Symbol` arm.
- The `_ =>` fallback arm allocates too. It is reached only for atoms sharing
  an `atom_order_rank` but differing in type (`Integer` versus `Rational` at
  rank 0). Handle it or leave it, but **say which and why**, with its
  allocation cost measured — do not silently leave a second allocation site
  unexamined.
- Every consumer of this comparator keeps its exact output: `Sort`, `SortBy`,
  `Union`, `Intersection`, `Complement`, `KeySort`, `Ordering`, `Min`/`Max`
  over symbols, canonical ordering for `Orderless` heads, and anything else
  you find — enumerate the consumers.

### Tests

Exact orderings, asserted as full lists:

- Same context, differing names, including names where one is a prefix of the
  other (`a` versus `ab`).
- Different contexts, including the nested-prefix case above that
  distinguishes the two candidate rules — assert `{Global`x`a, z}` explicitly,
  so a future change to `Symbol::cmp` fails loudly.
- Contexts differing only after a backtick, and names containing characters
  around `` ` `` (0x60) in ASCII — `_` (0x5F) and `a` (0x61) are the
  interesting neighbours.
- Symbols with identities (gensyms) that compare equal on text.
- Mixed atom types in one sort, exercising the rank ordering and the fallback
  arm.
- Sort stability where the current implementation promises it.
- A wide sort (16 000 symbols) returning the same list as the base build —
  generate the expectation from the base binary and pin it.

### A/B

`--scale 5`, same-binary noise floor **first, from at least three runs each
way, reported as the spread rather than a two-run difference**. Two runs give
one random sample and have already produced a false stop-loss in this line.

Add stable labels for a symbol sort at two widths. Same-tier neighbors that
must not move:

```
sort-default-wide      sortby-default-wide     reversesort-default-wide
keysort-default-wide   keysortby-default-wide
matcher-single-tiny    symbol-ownvalue-read
```

Report allocation counters first — the per-comparison `String`s disappearing
is stronger evidence than any timing.

## Scope

- Files you MAY modify: `crates/rum-stdlib/src/structure_ops/ordering_sets.rs`;
  `crates/rum-core/src/expr.rs` **only** to add a non-allocating helper that
  does not change any existing `Ord`/`PartialOrd` impl; stdlib tests;
  `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported**, each file justified.
- **The general accept/stop criterion**: *any ordering change is a stop.*
- MUST NOT touch: the existing `Symbol`/`Expr` `Ord` implementations,
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **`Symbol::cmp` is a different order.** The brief proves it above; a design
   that reaches it by another route is the same rejection.
2. **Compare bytes, not chars**, if that is what `String` comparison does
   today — match the existing semantics exactly, including for non-ASCII
   symbol names.
3. **The identity tiebreak**: today's comparison is on qualified text only, so
   two symbols with the same text but different identities compare equal here.
   Preserve that, whatever `Symbol::cmp` does.
4. **Counters before timings.**
5. **Noise floor from three runs each way**, reported as a spread.
6. **Stop-loss is real.** If the same step fails twice for the same reason,
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B as described.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What changed — per file.
2. The consumer inventory for the comparator.
3. What you did about the `_ =>` fallback arm, and why.
4. Verification — each command with verbatim output.
5. The exact-ordering test matrix, including the nested-context case.
6. The A/B — allocation counters first, then rows, with a three-run-each-way
   noise floor and the controls named.
7. Stop-loss / open questions.
