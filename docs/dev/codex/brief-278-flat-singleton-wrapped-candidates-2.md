# Brief 278 — non-OneIdentity `Flat` singleton wrapped candidates (2)

Second attempt at what was brief-276 Part B / brief-277. Brief-277's
implementation was **largely correct** and stop-lossed for a good
reason: the singleton omission row cannot be produced without touching
`sequence/orderless/optional.rs`, which brief-277 listed as MUST-NOT.
That diagnosis was verified independently and is correct.

This brief admits `optional.rs` and names the exact fix needed there.
Everything else carries over from brief-277 unchanged — **re-read
`docs/dev/codex/brief-277-flat-singleton-wrapped-candidates.md` for the
Specification and the Implementation guidance sections; they stand as
written and are not repeated here.** This file states only the DELTA.

## Why the previous attempt fell short (the delta's root cause)

There is a pre-existing structural asymmetry in `optional.rs`. Compare
the two collect-side entry points:

- `collect_compiled_orderless_optional_matches_owned` (line 86, non-flat)
  — present arm calls `collect_compiled_orderless_arg_choice_matches_owned`
  (collect-shaped) and omission arm calls
  `collect_compiled_orderless_optional_omission_matches_owned` (line 191,
  also collect-shaped). Symmetric: both arms return `Vec<SymbolExprMap>`.
- `collect_compiled_orderless_flat_optional_matches_owned` (line 120, flat)
  — present arm calls the collect-shaped sibling, but the omission arm
  calls `match_compiled_orderless_flat_optional_omission_owned` (line 141),
  a FIRST-MATCH function, and then pushes AT MOST ONE binding (lines
  149-152).

So the flat collect path has a match-shaped omission arm: it can pick
bare or wrapped, never collect both. That is a latent under-collection
that exists today independently of this feature — it is simply not
observable while bare is the only candidate. The wrapped-singleton work
exposes it.

**There is no `collect_compiled_orderless_flat_optional_omission_matches_owned`;
it must be written.** Both helpers it needs are already visible from
`optional.rs`:
- `match_compiled_orderless_flat_selected_values_owned` (`selected.rs:43`),
- `collect_compiled_orderless_arg_matches_with_flat_head_owned`
  (`sequence/entry.rs:175`).

Write it by mirroring the non-flat `collect_compiled_orderless_optional_omission_matches_owned`
(line 191) arm for arm, substituting those two flat-aware helpers for
their non-flat counterparts, then have
`collect_compiled_orderless_flat_optional_matches_owned` call it and
`append` its `Vec` (replacing the current push-at-most-one shape). Do
NOT change `match_compiled_orderless_flat_optional_omission_owned`
itself — the first-match path is correct as-is and is still used by the
match-side entry point.

## Scope (the ONLY delta from brief-277)

- ADDED to the MAY-modify list:
  `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs`
  — for the collect-shaped omission arm described above, and nothing
  else. Do not thread `one_identity` through this file; brief-277's
  local-derivation design still stands.
- Everything else in brief-277's Scope section is unchanged, including
  the MUST-NOT list — except that `optional.rs` moves off it. `core.rs`,
  `sequence/entry.rs`, `selected.rs` and `kernel_data/**` remain
  MUST-NOT.
- The no-default-cost rule is unchanged and still binding: no
  `OneIdentity` head may see any new work or allocation.

## Ground Truth

Measured on `cb6ebfc` (current `HEAD`, BEFORE any change) — the
baseline:

```text
SetAttributes[g, Flat]; MatchQ[g[1,2], g[a_g, b_]]              False
SetAttributes[g, Flat]; MatchQ[g[1],   g[a_g]]                  False
SetAttributes[k, Flat]; ReplaceList[k[1],   k[a_] :> a]         List[1]
SetAttributes[k, Flat]; ReplaceList[k[1,2], k[a_,b_] :> {a,b}]  List[List[1, 2]]
SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1], g[a_, b_:0] :> {a, b}]  List[List[1, 0]]
```

The first two are the real correctness gap this line exists to close:
under `Flat`, `g[g[1], 2]` flattens to `g[1, 2]`, so `a_g` binding
`g[1]` is a legitimate match that rum currently misses. **Both must
become `True`** — treat them as the acceptance criterion, more than any
enumeration count.

Measured on brief-277's (now reverted) implementation, which was
correct for every case except the singleton omission — expect to
REPRODUCE these; verify from actual output and pin what you observe:

```text
SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1,2], g[a_, b_:0] :> {a, b}]
  List[List[2, 1], List[g[2], 1], List[2, g[1]], List[g[2], g[1]],
       List[1, 2], List[g[1], 2], List[1, g[2]], List[g[1], g[2]],
       List[g[1, 2], 0]]

SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1,2], g[a_, b___, c_:0] :> {a, {b}, c}]
  List[List[2, List[], 1], List[g[2], List[], 1], List[2, List[], g[1]],
       List[g[2], List[], g[1]], List[1, List[], 2], List[g[1], List[], 2],
       List[1, List[], g[2]], List[g[1], List[], g[2]], List[1, List[2], 0]]
```

The row count grows because each single-value slot independently
offers bare and wrapped, so an n-slot pattern yields a 2^n cross
product. **This is intended and semantically correct** — each row is a
genuinely distinct valid decomposition under `Flat` — so do not try to
suppress rows to keep the count down. It is confined to user-defined
`Flat`-without-`OneIdentity` heads; no rum built-in reaches it.

THE DELTA THIS BRIEF EXISTS FOR — the singleton omission case, which
brief-277 could not reach:

```text
SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1], g[a_, b_:0] :> {a, b}]
  currently:  List[List[1, 0]]
  must gain:  a List[g[1], 0] row, after the List[1, 0] row
```

Read every final pinned value from actual release-CLI output. Do not
hand-derive.

## Verification

As brief-277, with these corrections and additions:

- Baseline is `1653 passed / 0 failed` on `cb6ebfc`; state your totals.
- **Brief-277's `k` probe was malformed** (`ReplaceList[k[1], k[a_]]` —
  `ReplaceList` needs a rule as its second argument, so it errored
  rather than probing anything). Use these instead, to exercise the
  plain non-orderless `Flat` half that the `g` probes never reach:

```
target/release/rum-cli --eval 'SetAttributes[k, Flat]; ReplaceList[k[1], k[a_] :> a]'
target/release/rum-cli --eval 'SetAttributes[k, Flat]; ReplaceList[k[1,2], k[a_,b_] :> {a,b}]'
target/release/rum-cli --eval 'SetAttributes[k, Flat]; MatchQ[k[1], k[a_k]]'
```

- Add the two acceptance probes:

```
target/release/rum-cli --eval 'SetAttributes[g, Flat]; MatchQ[g[1,2], g[a_g, b_]]'
target/release/rum-cli --eval 'SetAttributes[g, Flat]; MatchQ[g[1], g[a_g]]'
```

- Keep every brief-277 probe (`g` family plus the `1+1`,
  `MatchQ[5, x_ + y_.]`, `Union[{1,2},{2,3}]` regressions) and the
  microbench A/B with its FLAT expectation.

## Stop-loss

As brief-277. Note that its trigger fired correctly last time and the
rollback was clean — if a MUST-NOT file becomes necessary again, stop
again and say which one and why. Do not widen scope silently.

If `git checkout --` fails with `Unable to create '.git/index.lock':
Operation not permitted`, that is the sandbox refusing to write under
`.git` — restore the affected files by scoped patch instead, as the
previous run did successfully, and say so.

## Hard rules / Report format

Unchanged from brief-277.
