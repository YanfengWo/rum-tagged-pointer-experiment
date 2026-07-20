# Brief 279 — non-OneIdentity `Flat` singleton wrapped candidates (3)

Third and final attempt. **Brief-278's implementation SUCCEEDED** — it
achieved both acceptance criteria and the singleton omission row, with
no first-match result changed anywhere:

```text
MatchQ[g[1,2], g[a_g, b_]]                    True   (was False)
MatchQ[g[1],   g[a_g]]                        True   (was False)
ReplaceList[g[1], g[a_, b_:0] :> {a, b}]      List[List[1, 0], List[g[1], 0]]
```

It stop-lossed for one reason only: the new contract legitimately adds
wrapped rows to an existing `ReplaceList` assertion in
`definitions_and_rules_part3.rs:323`, a file that was outside Scope.
That was the correct call under the brief as written. **This brief is a
pure Scope delta** — re-do brief-278's implementation exactly as you
did it, and additionally respec that assertion.

Read `docs/dev/codex/brief-277-*.md` (Specification + Implementation
guidance) and `docs/dev/codex/brief-278-*.md` (the `optional.rs`
collect-shaped omission arm). Both stand as written.

## Ruling that governs the respec (user, 2026-07-20)

The 2^n candidate growth is **intended and accepted**. The user's
ruling, verbatim in substance: this combination is a form of
algorithmic stress test; we only need small examples to give correct
results; efficiency is not a consideration; WL itself does no heuristic
pruning, so neither do we. **Do not suppress or prune candidates to
keep row counts down.** A larger, semantically complete candidate set
is the correct outcome.

## The respec (the ONLY delta from brief-278)

`crates/rum-runtime/src/runtime/tests/definitions_and_rules_part3.rs`,
test `flat_and_orderless_flat_heads_regroup_for_match_replace_and_replacelist`.

Its `ReplaceList` assertion currently pins:

```text
ReplaceList[f[a, b, c], f[x_, y_] :> List[x, y]]
  == List[List[a, f[b, c]], List[f[a, b], c]]
```

`f` is `Flat` without `OneIdentity`, so under the new contract each
single-value slot also offers its wrapped form (`a` → `f[a]`,
`c` → `f[c]`). Re-pin this assertion to the ACHIEVED output — read it
from actual test output, do not hand-derive it.

The other assertions in that same test (`MatchQ[f[a, b, c], f[x_, y_]]`,
`f[a, b, c] /. f[x_, y_] :> List[x, y]`, and the whole `Plus` half)
are first-match or `OneIdentity`-head cases and **must NOT change**. If
any of them does, that is a bug — STOP, do not re-pin it.

## Scope (delta from brief-278)

- ADDED to MAY-modify:
  - `crates/rum-runtime/src/runtime/tests/definitions_and_rules_part3.rs`
    — the single `ReplaceList` assertion named above, nothing else;
  - `crates/rum-runtime/src/runtime/tests/thread_and_dispatch.rs`
    — permitted ONLY if a collect-all assertion there actually changes.
    Its two `Flat` tests (lines 177, 208) use only flattening, `MatchQ`
    and `/.`, so they are expected to be UNTOUCHED; if one does change,
    that means a first-match result moved — STOP and report instead of
    re-pinning.
- Everything else in brief-278's Scope stands, including its MUST-NOT
  list (`core.rs`, `sequence/entry.rs`, `selected.rs`, `kernel_data/**`).

**The acceptance rule that decides re-pin vs. STOP, for ANY test in the
workspace:**
- a collect-all (`ReplaceList`) assertion GAINING rows, on a
  `Flat`-without-`OneIdentity` head, with every pre-existing row still
  present and in its original relative order → legitimate, re-pin it;
- a first-match (`MatchQ`, `/.`, `ReplaceAll`) result changing, OR any
  pre-existing collect-all row disappearing, changing value, or moving
  relative to another pre-existing row, OR any change involving a
  `OneIdentity` head → NOT legitimate. STOP and report.

If the acceptance rule says re-pin but the file is outside the MAY list
above, STOP and report which file — do not widen scope yourself. (This
has happened twice and both stops were correct; a third is fine.)

## Verification

As brief-278, in full — including the microbench A/B (expected FLAT)
and every probe, which brief-278 could not reach because it rolled back
first. Baseline is `1653 passed / 0 failed` on `cb6ebfc`; state your
totals and list every test whose expectation you re-pinned.

Note for the rollback path: `git checkout --` fails under the sandbox
with `Unable to create '.git/index.lock': Operation not permitted`.
Restoring by scoped patch, as the last two runs did, is correct — say
so if you do it.

## Hard rules / Report format

Unchanged from brief-277.
