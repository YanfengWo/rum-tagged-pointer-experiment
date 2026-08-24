# brief-321 — index Reap's tag buckets

## Goal

Every `Sow` scans all existing buckets to find its tag, comparing `Expr` values
deeply, so a `Reap` collecting `n` distinct tags is quadratic. Give the bucket
list an adaptive index, keeping bucket order and every observable result
unchanged.

This is the last item from the complexity survey. brief-309 fixed `Reap`'s
*rendering* side; this is the *collection* side it deliberately left alone
because the fix needs a type it could not touch.

## Measured today (release CLI, verified by the director — do not re-derive)

`Reap[Do[Sow[i, i], {i, 1, n}]]` — every `Sow` uses a new tag:

| n | time |
|---|---|
| 2000 | 0.0151 s |
| 4000 | 0.0413 s |
| 8000 | **0.1289 s** |

2.7–3.1× per doubling. The single-tag control (`Sow[i, 1]`) is linear at
1.9/4.5/7.1 ms across the same widths — that contrast is what isolates the
collection scan from the rest of `Reap`.

## The mechanism

`crates/rum-runtime/src/runtime/debug_tools/reap.rs:236`:

```rust
fn record_sown_value(buckets: &mut Vec<ReapBucket>, tag: Expr, value: Expr) {
    if let Some(bucket) = buckets.iter_mut().find(|bucket| bucket.tag == tag) {
        bucket.values.push(value);
        return;
    }
    buckets.push(ReapBucket { tag, values: vec![value] });
}
```

`buckets` is one pattern's group from
`ReapScope::buckets_by_pattern: Vec<Vec<ReapBucket>>`
(`crates/rum-runtime/src/runtime/session/session_state.rs:175-181`), and
`ReapBucket` is `{ tag: Expr, values: Vec<Expr> }` (`:170-173`).

## Behaviour that must not change — measured, not assumed

| call | result |
|---|---|
| `Reap[Sow[1,"b"]; Sow[2,"a"]; Sow[3,"b"]]` | `{3, {{1,3},{2}}}` — buckets in **first-appearance** order, not sorted |
| `Reap[Sow[1,x]; Reap[Sow[2,y]]]` | `{{2,{{2}}}, {{1},{2}}}` — the inner `Reap` takes its own, the outer still sees both tags |
| `Reap[Sow[1,"aa"]; Sow[2,"ab"], _String]` | `{2, {{1},{2}}}` — the pattern selects which sows are captured |

So the `Vec` stays the source of truth for order, and any index is a lookup
mirror only. Nested scopes each have their own bucket groups, so an index
belongs to a group, not to the session.

## The fix

Add a lazily built `tag → position` index per bucket group, consulted only for
lookup, built once a group passes a threshold. Follow the in-tree pattern
(`16` in `csv.rs`, `grouping.rs`, `stable_groups.rs`, and the TOML decoder in
`9e8800b`) so small reaps allocate nothing new — most `Reap` calls have a
handful of tags at most.

`Expr` implements `Hash`, and `HashMap<Expr, usize>` already appears in
`structure_data/replace/plan.rs`, so the mechanism exists in-tree.

Three implementation traps to handle explicitly:

1. **`ReapScope` derives `Clone, Debug, PartialEq, Eq`.** Adding a field
   changes all four. State how equality stays correct — if the index is a pure
   function of the buckets, equal buckets give equal indexes, but say so rather
   than assuming it, and consider whether the index should participate in
   equality at all.
2. **`ReapScope` is cloned** on some paths. A cloned index is wasted work if the
   clone is short-lived; decide whether to clone or drop it, and justify.
3. **Invalidation.** Buckets are appended to, and a bucket's `values` grow in
   place. If nothing removes or reorders buckets, the index never needs
   invalidating — verify that claim across the file rather than assuming it, the
   way `9e8800b` did for the TOML decoder.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/debug_tools/reap.rs`
- `crates/rum-runtime/src/runtime/session/session_state.rs` — only the `Reap`
  types and only what the index requires
- test modules under `crates/rum-runtime/src/runtime/tests/`

Out of scope: everything else, including `Reap`'s rendering path (fixed in
`e0efd95` — do not revisit) and the `Sow` dispatch itself.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- The adaptive index behind a stated threshold, with the small-reap path
  unchanged.
- Tests pinning the three behaviours above, plus a wide case.
- Before/after scaling at n = 2000 / 4000 / 8000, and the single-tag control
  showing it did not regress.
- The three traps answered.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise. Use the kernel's own
`AbsoluteTiming`; `/usr/bin/time` is coarser than these probes:

```
python3 - <<'PY'
for n in (2000, 4000, 8000):
    open("/tmp/b321-distinct-%d.rum" % n, "w").write(
        'Print["distinct n=%d: ", AbsoluteTiming[Length[Last[Reap[Do[Sow[i, i], {i, 1, %d}]]]]]];\n' % (n, n))
    open("/tmp/b321-single-%d.rum" % n, "w").write(
        'Print["single n=%d: ", AbsoluteTiming[Length[Last[Reap[Do[Sow[i, 1], {i, 1, %d}]]]]]];\n' % (n, n))
PY
for n in 2000 4000 8000; do ./target/release/rum-cli /tmp/b321-distinct-$n.rum < /dev/null; done
for n in 2000 4000 8000; do ./target/release/rum-cli /tmp/b321-single-$n.rum < /dev/null; done
```

The distinct series must become linear; the single-tag series must not regress.

`Reap` is a diagnostic builtin rather than an evaluation hot path, so no
microbench A/B is required unless your change touches something shared. Say so
if you believe it does.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If bucket order or nesting behaviour cannot be
  preserved, STOP and report rather than changing an expected value.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges, and the threshold you used.
2. The three traps, answered with evidence.
3. Verification — each command with verbatim output: test counts, the pinned
   behaviours, and both scaling series.
4. Stop-loss / open questions.
