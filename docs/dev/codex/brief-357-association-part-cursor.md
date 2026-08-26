# brief-357 — `Part`/`Extract` on an association must not materialize every entry

## Goal

Reading one value out of an association costs the whole association. Measured
on `12f1401a`, 200 reads:

| keys | `a[[Key[k]]]` per read | `a[[3]]` per read |
|---:|---:|---:|
| 4 000 | 1.32 ms | 1.30 ms |
| 8 000 | 3.03 ms | 2.99 ms |
| 16 000 | 6.54 ms | 6.44 ms |
| 32 000 | **12.02 ms** | 12.34 ms |

Strictly linear in the association's size, for a read of one element.

The cause is `select_parts_for_selector`
(`kernel_data/structure_data/selectors/apply.rs:214-220`): it calls
`association_entries_from_expr` before looking at the selector, which walks
every entry and deep-clones every key and value, and then searches that vector
linearly.

**Semantics must not change at all.** This is a performance repair; every
answer, every message, and every `Missing[...]` shape stays byte-identical.

## Context pointers

- `kernel_data/structure_data/selectors/apply.rs:214-250` — the entry point
  and the `Single`/`Key`/`Multi` arms.
- `kernel_data/association_data.rs:16-28` — `association_entries_from_expr`,
  and `parse_association_entries_deep:54-62` beneath it.
  `canonicalize_association_entries` decides duplicate-key resolution —
  whatever rule it implements (last occurrence wins, or otherwise) must hold
  identically on any borrowed path. Establish that rule and state it.
- `model/cache.rs` — the existing `AssociationLookupCache` read-side index,
  with its probe/admission API and its size thresholds. Reuse it rather than
  building a second index. Its documented limits are a standing decision: do
  not change `MAX_KEYS` or the admission policy in this brief.
- `kernel_data/association_data.rs:4` — `ASSOCIATION_INDEX_THRESHOLD`, below
  which a linear scan is deliberate and must stay.
- **Dead arm to check**: the `PartSelector::Key` arm at `apply.rs:234-243`
  calls `association_entries_from_expr` again, but the early return at the top
  means it is only reached when the expression is *not* an association, where
  that call returns `None`. Confirm this and remove it if so — but only if it
  is genuinely unreachable.

## Deliverable

- Single `Key` and single positional selectors resolve **without**
  materializing the entry vector: index probe where the cache applies, direct
  positional access otherwise.
- `Multi` selectors materialize at most **once** per call, not once per
  sub-selector.
- Nested selectors do not re-materialize the same association per level.
- Byte-identical behavior for: a present key, an absent key
  (`Missing[...]` shape and content), duplicate keys, a positional index in
  and out of range, negative indices, `Span`, `All`, mixed `Multi`, nested
  associations, associations below `ASSOCIATION_INDEX_THRESHOLD`, and every
  `pmix`/`partw`/`partd` message — same text, same order, same count.
- Suspension safety preserved: selectors are suspension-capable; do not
  collapse a `suspend_with_frame*` into an eager call.

### Tests

- Exact values and exact messages for every shape listed above.
- Duplicate keys: the resolution rule you established, asserted directly.
- A wide association (100 000 keys) read by key and by position completes and
  returns the right value — the linearity is the defect, so the test that
  proves the repair is a wide one.
- Reads interleaved with writes, so the cache's invalidation still holds.
- Below-threshold associations still take the linear path and return the same
  answers.

### A/B

`--scale 5`, same-binary noise floor **first**, then base versus candidate.
Add stable labels for a keyed read and a positional read at two association
widths, so the report can show the cost is no longer linear in width — that,
not the multiple, is the property to demonstrate.

Same-tier neighbors that must not move:

```
association-set-key-incremental-small   association-set-key-incremental-large
association-small-literal-repeat        deletecases-wide-association-no-match
set-part-fill-small                     set-part-fill-large
set-part-fixed100-len100                set-part-fixed100-len800
matcher-single-tiny                     symbol-ownvalue-read
```

Name the untouched controls; never the suite median. A movement smaller than
your measured floor is not a regression; if a required row lands within a
third of a percentage point of the floor, re-measure the pair.

## Scope

- Files you MAY modify: the selector modules under
  `kernel_data/structure_data/selectors/`; `kernel_data/association_data.rs`;
  `model/cache.rs` **only** if the existing probe API needs a borrowing entry
  point (not a policy change); runtime tests;
  `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported**, each file justified.
- **The general accept/stop criterion**: *any behavior change at all is a
  stop.* This brief changes cost, not answers.
- MUST NOT touch: the association write path, `MAX_KEYS` or the cache
  admission policy, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`, `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **Borrowed traversal, not owned clones.** `Expr`/`SharedExpr`/`SharedArgs`
   are `Arc`-shared; prefer borrowing and `Arc` clone over deep clone on any
   path that runs per read.
2. **Do not pin association storage with strong references** in the cache —
   that was a deliberate design constraint in the existing index.
3. **The duplicate-key rule is observable.** Establish it from the current
   canonicalization and preserve it exactly.
4. **Messages are part of the contract**: same text, same order, same count,
   including when several selectors fail in one call.
5. **Below the index threshold the linear scan is deliberate** — do not
   "optimize" it into an allocation.
6. **Counters before timings**: an allocation counter showing the per-read
   entry clones disappear is stronger evidence than any timing.
7. **Stop-loss is real.** If the same step fails twice for the same reason,
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
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`;
  size bounds from the table above — a 200-read probe at 32 000 keys takes
  about 2.4 s today.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What changed — per file, mechanically-forced files listed separately.
2. The duplicate-key resolution rule, and where you established it.
3. Whether the `PartSelector::Key` arm was genuinely unreachable.
4. Verification — each command with verbatim output.
5. The exact-value and exact-message matrix.
6. The A/B — allocation counters first, then rows at two widths showing the
   cost is no longer width-linear, with the noise floor and controls named.
7. Stop-loss / open questions.
