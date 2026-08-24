# brief-316 — index the key-selection builtins

## Goal

`KeyTake`, `KeyDrop` and `KeyDropFrom` compare every selected key against every
entry, so all three are quadratic. Replace the pairwise membership test with a
hash lookup, keeping their observable output — including the order rules, which
differ between `KeyTake` and `KeyDrop` — exactly as it is.

## Measured today (release CLI, verified by the director — do not re-derive)

An association of `n` integer keys, selecting `n/2` of them:

| n | `KeyTake` | `KeyDrop` | `KeyDropFrom` |
|---|---|---|---|
| 2000 | 0.0095 s | 0.0075 s | 0.0078 s |
| 4000 | 0.0352 s | 0.0282 s | 0.0288 s |
| 8000 | **0.1386 s** | **0.1094 s** | **0.1095 s** |

3.7–3.9× per doubling in all three. `KeyDropFrom` was listed in an earlier
survey on a structural reading only; the measurement above confirms it.

## The mechanism

`crates/rum-stdlib/src/associations/basic.rs:722-743` holds both halves:

- **`KeyTake`** loops over the selected keys and, for each, filters the whole
  entry list — O(k · m).
- **`KeyDrop`** loops over the entries and, for each, runs `any` over the whole
  selected-key list — O(m · k).

`crates/rum-stdlib/src/associations/mutation.rs:355-369`
(`apply_key_drop_from_value`) repeats the `KeyDrop` shape verbatim.

The in-tree model to follow is `KeyComplement`
(`crates/rum-stdlib/src/associations/transforms/key_collections.rs:277-288`),
which collects the excluded keys into a `HashSet` once and then filters.

## Behaviour that must not change — measured, not assumed

| case | current result |
|---|---|
| `KeyTake[<\|"a"->1,"b"->2,"c"->3\|>, {"c","a"}]` | `<\|"c"->3, "a"->1\|>` — **selection order, not association order** |
| `KeyTake[<\|"a"->1,"b"->2\|>, {"a","a"}]` | `<\|"a"->1\|>` — a repeated selection appears once |
| `KeyTake[<\|"a"->1\|>, {"a","zz"}]` | `<\|"a"->1\|>` — a missing key is skipped silently |
| `KeyDrop[<\|"a"->1,"b"->2,"c"->3\|>, {"b"}]` | `<\|"a"->1,"c"->3\|>` — **association order** |
| `KeyDrop[<\|"a"->1,"b"->2\|>, {"b","b","zz"}]` | `<\|"a"->1\|>` — repeats and misses are harmless |

**The two order rules are opposite**, and that is the part a careless rewrite
breaks. `KeyDrop` and `KeyDropFrom` filter the entry list, so a `HashSet` of
selected keys is a drop-in replacement. `KeyTake` emits in *selection* order, so
it needs a key → entry lookup built from the entries and then walked in
selection order — not a filter over entries.

Note that the repeated-selection case currently collapses in the rendering step
(`render_association_entry_forms_preserving_heads`), not in the loop. Whatever
you build must keep that end result; do not assume the loop is what dedupes.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/associations/basic.rs`
- `crates/rum-stdlib/src/associations/mutation.rs`
- the `rum-stdlib` test modules for these builtins

Out of scope: everything else — including `KeyComplement` itself (already
correct — read it, do not touch it) and `rum-runtime`.

Do not add unconditional cost to the default evaluation path.

## A judgement call to make and justify

Several in-tree index sites gate on a threshold (`16` in `csv.rs`,
`grouping.rs`, `stable_groups.rs`, and the TOML decoder landed in `9e8800b`), so
small inputs never allocate a hash structure. `KeyComplement` has no such gate.

Decide which of these fits here and **say why in the report**: building a
`HashSet` of two keys to filter three entries may well cost more than the
pairwise scan it replaces. If you add a threshold, keep the existing scan below
it unchanged. If you argue none is needed, back that with a small-input
measurement rather than an opinion.

## Deliverable

- All three builtins using a hash-based membership test above whatever
  threshold you justify.
- Tests pinning the five behaviours in the table above, plus a wide case per
  builtin.
- Before/after scaling at n = 2000 / 4000 / 8000, and a small-input measurement
  supporting your threshold decision.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise. Use the kernel's own
`AbsoluteTiming`; `/usr/bin/time`'s resolution is coarser than these probes:

```
python3 - <<'PY'
for n in (2000, 4000, 8000):
    open("/tmp/b316-%d.rum" % n, "w").write(
        "a = Association[Table[i -> i, {i, 1, %d}]];\n"
        "ks = Table[i, {i, 1, %d}];\n"
        'Print["KeyTake n=%d: ", AbsoluteTiming[Length[KeyTake[a, ks]]]];\n'
        'Print["KeyDrop n=%d: ", AbsoluteTiming[Length[KeyDrop[a, ks]]]];\n'
        'Print["KeyDropFrom n=%d: ", AbsoluteTiming[KeyDropFrom[a, ks]; Length[a]]];\n'
        % (n, n // 2, n, n, n))
PY
for n in 2000 4000 8000; do ./target/release/rum-cli /tmp/b316-$n.rum < /dev/null; done
```

Growth must be linear rather than ~3.8× per doubling, with the same lengths.

No microbench A/B unless your threshold decision makes small associations pay
something new — in which case run it and paste the association labels plus
same-tier neighbors.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.**
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If any pinned behaviour cannot be preserved by the hash
  approach, STOP and report the case rather than changing the expected value.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges, and how each order rule is
   preserved.
2. Your threshold decision and the measurement behind it.
3. Verification — each command with verbatim output: test counts, the five
   pinned behaviours, and the before/after scaling table.
4. Stop-loss / open questions.
