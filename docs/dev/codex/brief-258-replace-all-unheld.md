# Brief 258 — `ReplaceAllUnheld` (wl15/wfr batch 2c)

## Goal

A new builtin `ReplaceAllUnheld[expr, rules]` (+ the operator form
`ReplaceAllUnheld[rules][expr]`) that applies rules like `ReplaceAll`
but ONLY to parts that would evaluate — held argument positions are
left verbatim. It is a SEPARATE builtin (user ruling C1); `ReplaceAll`
itself is untouched and its `replaceall-simple` microbench label must
not regress. The contract simplifies because `ReplaceAllUnheld`, like
`ReplaceAll`, evaluates its target first — so the only difference from
`ReplaceAll` is a structural descent mask.

## Contract (probed on HEAD 2026-07-18; measured, not predicted)

Both forms evaluate the target first (ReplaceAll does; mirror it). Then
traverse the EVALUATED tree top-down, exactly like `ReplaceAll` EXCEPT:
at each `Call` node `h[a1,…,an]`, an argument position that `h` HOLDS
is left verbatim (not descended, not rewritten inside). The head is
always descended; the whole node is match-tested first (root-first,
then descend — identical to ReplaceAll).

The hold mask at a node comes from the node's head:
- a symbol head → its attributes (`HoldFirst` holds pos 1, `HoldRest`
  holds pos ≥2, `HoldAll`/`HoldAllComplete` hold all);
- a compound head whose base symbol carries `SubValuesHoldAll` → all
  outer args held (consistency with brief-255; use the SAME
  attribute resolution `build_call_dispatch_plan` uses at
  `eval_dispatch/mod.rs:15-22`).
- Use the head's ORIGINAL attributes at that node — capture the mask
  BEFORE the head is rewritten by the rules (see PITFALL 1).

Contrast table (VERIFY each on HEAD as `x /. rules`, then confirm your
candidate's `ReplaceAllUnheld[x, rules]` differs exactly as shown):

| input `x`, `rules = Plus -> Times` | `ReplaceAll` (HEAD today) | `ReplaceAllUnheld` (target) |
|---|---|---|
| `Hold[1+1] + 2` | `Times[2, Hold[Times[1, 1]]]` | `Times[2, Hold[1 + 1]]` |
| `hr[1+1, 2+2]` (`hr` HoldRest) | `hr[2, Times[2, 2]]` | `hr[2, 2 + 2]` |
| `Hold[a[1+1]]` | `Hold[a[Times[1, 1]]]` | `Hold[a[1 + 1]]` (whole held subtree verbatim) |
| `HoldComplete[1+1]` | `HoldComplete[Times[1, 1]]` | `HoldComplete[1 + 1]` |
| `1+1` (no hold) | `Times[1, 1]` | `Times[1, 1]` (identical — nothing held) |

Result handling (evaluate unless self-evaluating atom) and delayed-RHS
handling: IDENTICAL to `ReplaceAll`. The RuleDelayed consumer-divergence
defect is explicitly OUT of scope (design doc §5 C4) — do not touch it.

Rules-shape errors (`ReplaceAllUnheld[x, notarule]`): parity with
`ReplaceAll`'s `argform`/`argx` messages.

## Context pointers (all sites director-verified 2026-07-18)

- Read `AGENTS.md` and `docs/dev/wl15-wfr-hold-design.md` §5 (C1-C4).
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs:4-12` —
  `enum RewriteDataBuiltin`; add `ReplaceAllUnheld`. Director-verified:
  the ONLY exhaustive match is the dispatch at `:178-184` — add the one
  arm `RewriteDataBuiltin::ReplaceAllUnheld => self.eval_replace_all_unheld(args),`.
  The other two mentions (`predicates/arrays.rs:105`,
  `test_builtins.rs:30`) are constructions, not matches — no ripple.
- `crates/rum-runtime/src/runtime/model/messages.rs:13-21` —
  `enum RuleApplicationSource`; add `ReplaceAllUnheld` next to
  `ReplaceAll`. Director-verified: this enum is `Copy` and NEVER
  exhaustively matched anywhere — adding the variant is ZERO mechanical
  ripple. Its only consumer is the mask logic you add in the traversal
  (below). For message attribution it behaves like `ReplaceAll`; if any
  message path needs a source→symbol mapping, mirror `ReplaceAll`'s.
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:4-56`
  — `eval_replace_all`: copy its suspension-safe shape into
  `eval_replace_all_unheld` (evaluate target → drive rules → apply →
  evaluate result). Arity/argform handling identical.
- `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:46-153`
  — the shared traversal (`apply_replace_all_owned` /
  `_after_miss` / `_args`). This is where the mask lives. STRATEGY
  (director decision): thread the mask through this ONE traversal keyed
  on `source == ReplaceAllUnheld` — do NOT duplicate the whole
  suspension-safe traversal. In `apply_replace_all_after_miss`, before
  rewriting the head, compute the hold mask from the ORIGINAL head;
  in `apply_replace_all_args`, when the source is `ReplaceAllUnheld`
  AND the current argument position is held, push the argument VERBATIM
  (skip the recursive `apply_replace_all_owned` for it). For every
  other source the added condition is one discriminant check that
  short-circuits — the `replaceall-simple` A/B is the veto if that
  costs anything.
- Hold-position logic: `metadata.rs` has the per-position hold test
  (the `HoldAttribute`/`AttributeSet` `holds_position`-style method
  near `:20-28`); reuse it, do not re-derive. Head attributes via the
  session's cached lookup (`cached_attribute_set_for_head` /
  `cached_attribute_set_for_symbol`, as `build_call_dispatch_plan`
  uses).
- `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs:956-969`
  — `rewrite_operator_symbol`: add
  `1 if is_system_symbol_name(head, "ReplaceAllUnheld") => Some("ReplaceAllUnheld"),`
  for the operator form (that is the whole operator-form change; the
  wrapper family already rebuilds `Head[data, tail…]`).
- `crates/rum-stdlib/src/rewrite_builtins.rs:45-51` — add
  `eval_replace_all_unheld` shim dispatching
  `RewriteDataBuiltin::ReplaceAllUnheld`.
- `crates/rum-stdlib/src/catalog/rewrite.rs:89-100` — add the
  `BuiltinSpec::special("ReplaceAllUnheld", …).with_messages([argform,
  ARGX_EXACT]).protected()` entry mirroring `ReplaceAll`.
- Reference: regenerate via `cargo run -p rum-stdlib --bin
  generate-builtin-reference`; the drift test enforces agreement.
- Tests: new module (mirror where ReplaceAll behavior is tested —
  `crates/rum-runtime/src/runtime/tests/` for the traversal, e.g. a
  new `replace_all_unheld.rs` + its `mod` line). Existing ReplaceAll
  tests must stay green UNCHANGED (byte-identity evidence).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs`
  - `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs`
    (or a sibling file for the new eval fn if cleaner — same module)
  - `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs`
  - `crates/rum-runtime/src/runtime/model/messages.rs` (add the
    `RuleApplicationSource::ReplaceAllUnheld` variant — no match ripple)
  - `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
    (one line in `rewrite_operator_symbol`)
  - `crates/rum-stdlib/src/rewrite_builtins.rs`
  - `crates/rum-stdlib/src/catalog/rewrite.rs`
  - a NEW runtime test file + its `mod` registration in
    `crates/rum-runtime/src/runtime/tests/mod.rs`
  - `docs/reference/builtins.md` (regeneration output only)
- MUST NOT touch: `ReplaceAll`'s own eval fn or its message set (its
  behavior is frozen — you only ADD a parallel path); brief-255/256/257
  code beyond the one `rewrite_operator_symbol` line; everything else.
- No unconditional default-path cost on `ReplaceAll`: the mask branch
  must short-circuit for every non-`ReplaceAllUnheld` source, and the
  `replaceall-simple` A/B must show no regression.

## Design decisions (ruled — implement, do not re-litigate)

1. Separate builtin, never a `ReplaceAll` option (C1).
2. Target is evaluated first (mirror ReplaceAll); the traversal then
   masks held arg positions.
3. Mask from the head's ORIGINAL attributes at each node (PITFALL 1).
4. Compound head + `SubValuesHoldAll` base → all outer args held.
5. Operator form mirrors ReplaceAll's (already in the rewrite wrapper
   family; one match line).
6. Delayed-RHS + result-evaluation identical to ReplaceAll; the
   consumer-divergence defect stays OUT of scope.
7. STOP-LOSS: if threading the mask forces a change to ReplaceAll's own
   observable behavior, or the traversal rework balloons beyond the arg-
   descent decision, STOP and report (the fallback is a dedicated
   duplicated traversal, my call, not yours to improvise).

## Deliverable

- The builtin + operator form + tests (measured expectations — probe
  the release CLI FIRST, then pin): the five contrast-table rows, a
  compound-head `SubValuesHoldAll` case, a head-rewrite case
  (`ReplaceAllUnheld[Hold[1+1], Hold -> List]` — probe and pin whatever
  it actually does; do not predict), rules-shape error parity, the
  operator form on two inputs, and a no-op parity row (`1+1` identical
  to ReplaceAll).
- Existing ReplaceAll suite green UNCHANGED.
- Microbench A/B (MANDATORY, interleaved B C B C, best-of-2/side):
  `replaceall-simple` is the HARD veto label; also report the
  downvalue tier + neighbors. FAIL = a label's Δ outside the
  unaffected-controls |Δ| distribution AND same direction across the
  two rounds. Panel has no ReplaceAllUnheld label — say so.

## rum / Rust pitfalls (read twice)

1. **The head is rewritten before the args are descended**
   (`apply_replace_all_after_miss` rewrites `head` first, then calls
   `apply_replace_all_args` with the REWRITTEN head). The hold mask
   must come from the ORIGINAL head — capture it before the head
   rewrite and pass it into the arg loop. Using the rewritten head's
   attributes is a bug (e.g. `Hold -> List` would un-mask).
2. Expected values MEASURED via `printf '' | … --eval '…'`, never
   predicted; feed stdin.
3. Suspension safety: the traversal is `drive_internal_value`-based;
   preserve that shape when adding the skip branch — a verbatim push
   is trivial, do not introduce an eager evaluate.
4. `Expr`/`SharedArgs` are `Arc`-shared; the verbatim skip is an `Arc`
   clone/move, no deep clone.
5. Minimal diff to ReplaceAll's path — the ONLY change there is the
   masked-skip branch guarded by source.
6. Stop-loss is real (decision 7).

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace --exclude rum-jupyter   # sandbox loopback exclusion; note it
```

Then the A/B, and paste VERBATIM (all via `printf '' | cargo run -q
-p rum-cli -- --eval …`):

```
ReplaceAllUnheld[Hold[1+1] + 2, Plus -> Times]
Hold[1+1] + 2 /. Plus -> Times
ReplaceAllUnheld[hr[1+1, 2+2], Plus -> Times]  (after SetAttributes[hr, HoldRest])
ReplaceAllUnheld[Hold[a[1+1]], Plus -> Times]
ReplaceAllUnheld[HoldComplete[1+1], Plus -> Times]
ReplaceAllUnheld[1+1, Plus -> Times]
ReplaceAllUnheld[Plus -> Times][Hold[1+1] + 2]
ReplaceAllUnheld[Hold[1+1], notarule]
```

## Hard rules

- No `git commit`/branch/push; leave changes in the tree.
- Nothing outside Scope; must-touch conflicts → STOP and report.
- Nothing outside this repo, no network; commands: Scope edits,
  `cargo`, release `rum-cli`, microbench scripts, `python3`.
- Testing Policy hard line; no new `unsafe`; minimal diff; probe budget
  best-of-2 per A/B side; verbatim output only, never faked.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why (include every
   mechanical match-ripple site).
2. Verification — commands + verbatim output.
3. Verdict — the contrast table (before/after) + the A/B reading with
   the stated noise floor and the `replaceall-simple` verdict.
4. Stop-loss / open questions (empty if none).
