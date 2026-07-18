# Lazy pure sequences — design (2026-07-18)

Origin: WL15 core survey item 3 (incremental/lazy collections). User
rulings 2026-07-18:

- rum implements **only the pure half** — immutable, re-runnable lazy
  sequences. The **stateful half** (`IncrementalObject`/`NextValue`
  mutable cursors, streaming from an external source) is **WON'T-DO in
  the kernel**; if a downstream ever needs it, it belongs in a
  downstream package, never in the current kernel.
- Surface value model is an **opaque `LazyList` head** (not
  list-transparent).
- v1 covers the **broad set** (generators + transformers + combinatorial
  streams).

Design rationale is grounded in the Haskell precedent (see the
conversation record): Haskell's *pure* laziness (thunks / lazy lists)
does not interact badly with snapshot/reset/cancel because thunk update
is semantically invisible and idempotent — there is no cursor, no
consumption, no position. The problems only appear when a stateful
handle is disguised as a pure value (lazy IO, `hGetContents`), which the
Haskell community judged an anti-pattern and replaced with explicit
scoped streams. rum takes the pure half and refuses the disguise.

## 1. Substrate constraints (from the codebase)

- `Expr` is `Atom(Integer|Rational|String|Symbol) | Call{head, args}`
  and derives `Serialize/Deserialize/Hash` (`rum-core/src/expr.rs`).
  **Adding a new `Expr` variant is off the table** (every exhaustive
  match, the serializer, the matcher would ripple). A lazy value must
  therefore be a **reserved-head `Call`**, exactly like `Association` /
  `ByteArray` today.
- Structured values already live as heads (`Association[...]` etc.), so
  the pattern is proven.

## 2. Representation

A lazy sequence is the opaque value

```
LazyList[<descriptor>]
```

- `LazyList` is `{HoldAllComplete, Protected}` — its content never
  evaluates on its own, so the value is **inert** (a normal form). It is
  a pure `Expr`, so it **serializes, snapshots, and restores for free**
  and re-derives identically. This is the whole payoff of taking the
  pure half.
- `<descriptor>` is a **pull-based descriptor tree** of inert internal
  nodes: a source at the leaf, transformers wrapping it. Example value:
  `LazyList[LTake[LMap[f, LRange[1, 1]], 5]]`.
- Realization is a **Rust pull interpreter** over the descriptor: to
  pull the next element from `LTake[inner, n]` pull from `inner` up to
  `n`; from `LMap[f, inner]` pull from `inner` and evaluate `f[x]`; from
  `LRange[start, step]` yield `start`, advance. Each lazy operation is
  **one descriptor node + one interpreter arm** — this is what makes the
  broad set cheap to extend.

Chosen over the alternative "unfold coalgebra" (`LazyList[stepFn, seed]`
with symbolic function composition) because transformer composition
there needs symbolic `Function` gymnastics on every step; the descriptor
tree keeps composition in Rust and stays serializable.

**Memoization: NONE in v1.** Each realization re-derives from the
descriptor (pure, deterministic, snapshot-trivial). Shared memoization
(a side cache keyed off descriptor identity, never part of the value,
never serialized) is a possible later optimization, not v1. Because
realization is pull-based we never retain realized elements, so the
Haskell "retained-thunk space leak" does not apply — memory is bounded
by descriptor size, not stream length.

## 3. Surface

### Generators (return a `LazyList`)
- `LazyRange[]` / `LazyRange[start]` / `LazyRange[start, step]` —
  infinite arithmetic stream (rationals allowed). `LazyRange[a, b, step]`
  bounded parity with `Range`.
- `LazyTable[body, {i, ...}]` — lazy `Table`; body held, one element per
  iteration.

### Transformers (take a `LazyList`, return a `LazyList` — stay lazy, composable)
- `LazyMap[f, lz]`
- `LazySelect[pred, lz]`
- `LazyTake[lz, n]` / `LazyTake[lz, UpTo[n]]` — cap, stays lazy.
- `LazyDrop[lz, n]` — skip, stays lazy.
- Broad combinatorial / streaming (v1 per user "broad" ruling):
  `LazySubsets`, `LazyTuples`, `LazyPermutations`, `LazyFoldList`,
  `LazyNestList`.

### Realizers (force — return an ordinary `List` or element)
- `Normal[lz]` — full realization to a `List`. On an infinite stream it
  runs under the **existing iteration ceiling** and **fails closed**
  (`TerminatedEvaluation` tag), never hangs.
- `Take[lz, n]` — forces the first `n`, returns a `List` (bounded, safe
  on infinite streams). `First`/`Rest` realizer parity: `First[lz]` →
  element; `Rest[lz]` → `LazyList` (tail stays lazy).
- `Part[lz, i]` — realizes and returns the i-th element.

**Realizer vs `Lazy*` transformer is the key ergonomic line**: `Take[lz,
n]` *forces* to a `List`; `LazyTake[lz, n]` *stays lazy*. Same for
`Drop`/`LazyDrop`.

## 4. Opaque semantics (from the "opaque `LazyList`" ruling)

- **Not a `List`.** `LazyList[...]` does **not** match `_List`, `{__}`,
  `{___}`; it does **not** auto-force under any list operation. It
  matches `_LazyList` / `Blank[LazyList]`.
- **No accidental realization.** Only the sanctioned realizers (§3) force
  it; everything else sees an opaque atom-like value.
- **Equality** is structural over the descriptor (cheap, no forcing):
  two `LazyList`s are equal iff their descriptors are equal.
- **Formatting**: prints as an elided summary — `LazyList[<stream>]` (or
  `LazyList[<generator-shape>]`), never the elements (may be infinite).
- **`Length[LazyList[...]]`** is left inert (a non-list) — consistent
  with opaqueness; realize first if a count is wanted. No special-case.

## 5. Purity, safety, capability

- Every lazy builtin is **`Capability::Pure`** — deterministic,
  re-runnable, no side effects. **No new capability.**
- **Safety property (the reason the pure half is sandbox-safe):** the
  realize loop runs under the same iteration ceiling and the same
  `cancel`/`timeout` checks as `Do`/`Table`. An infinite stream realized
  in full hits the ceiling and returns a `TerminatedEvaluation`; it
  cannot hang the host. Cancellation interrupts the realize loop cleanly
  (no torn state — there is no state).
- **Snapshot/restore**: a `LazyList` is an ordinary inert `Expr`, so it
  round-trips with the session image and re-derives identically. No
  cursor to serialize, no position to lose. (Round-trip test required in
  batch 1.)

## 6. Non-goals (explicit)

- No `IncrementalObject` / `NextValue` mutable cursor — WON'T-DO in
  kernel (user 2026-07-18); downstream package only if ever pulled.
- No list-transparency / auto-forcing.
- No memoization in v1.
- No lazy streaming from external sources (that is a stateful handle =
  the WON'T-DO half).

## 7. Batch plan

1. **Mechanism (spine).** `LazyList` opaque wrapper
   (`{HoldAllComplete, Protected}`); descriptor-tree representation; the
   Rust pull interpreter; `LazyRange`; realizers
   `Normal`/`Take`/`First`/`Rest`/`Part`; opaque semantics
   (pattern non-match, formatting summary, structural equality);
   iteration-ceiling + cancel integration; snapshot round-trip test.
2. **Transformers.** `LazyMap`/`LazySelect`/`LazyTake`/`LazyDrop`/
   `LazyTable` — each one descriptor node + one interpreter arm.
3. **Broad combinatorial/streaming.** `LazySubsets`/`LazyTuples`/
   `LazyPermutations`/`LazyFoldList`/`LazyNestList`.
4. **Guide + close-out.** Language-guide section (measured examples);
   reference regen; tracker close-out.

Each batch gates before the next (director gate per CLAUDE.md). No
eval-hot-path change is expected (lazy builtins are new symbols, cold);
A/B only if a realizer touches a shared iteration primitive.
