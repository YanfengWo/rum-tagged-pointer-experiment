# brief-315 — reuse the read-side index on the Association write path

## Goal

After the cliff fix (`f8d8f55`), incremental Association writes are a clean
quadratic again: every write scans the whole entry list. Remove that scan by
reusing the read side's already-validated index, so building an Association key
by key is linear.

This is Step B of the design in brief-313. Step A landed; do not revisit it.

## Measured today (release CLI, verified by the director — do not re-derive)

`a = <||>; Do[a[i] = i, {i, 1, n}]`, on the current tree:

| n | time |
|---|---|
| 8000 | 0.61 s |
| 16000 | 2.41 s |
| 20000 | 3.64 s |
| 32000 | **8.67 s** |

16000 → 32000 is 3.6× for 2× the work: quadratic, smooth, no cliff.

## The mechanism, and the part that is easy to get wrong

`direct_association_rule_plan`
(`crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs:127-167`)
walks every entry. **It does not stop when it finds the key**, and that is not
an oversight: the loop has two jobs.

1. Locate the key (`rule_args[0] == *key`).
2. Validate that *every* entry is a two-argument `Rule` or `RuleDelayed` —
   returning `None` for anything else, which sends the write down the slow path.

The mutation window that follows depends on job 2 having covered the whole list.
**An optimization that only speeds up job 1 — "break once found" — is wrong**,
and wrong in the silent way: it would let a malformed entry past the guard and
into `make_mut`.

The read side already solved exactly this. `AssociationLookupCache`
(`crates/rum-runtime/src/runtime/model/cache.rs:96-120`) validates the whole
list once, inserts an index, and afterwards reuses it on pointer identity. Its
witness is weak (`WeakSharedArgs`) specifically so it does not pin the COW data
— which is why reusing it does not recreate the extra strong holder that
`plan-291` removed, and why `Arc::make_mut` can still mutate in place.

## The shape to build

- Look the position up through the read-side cache's probe instead of scanning.
- Keep the existing full scan for the cases the index does not cover: below the
  index threshold (small Associations, where a scan is cheaper than a probe) and
  above whatever upper bound the read side already applies. Behaviour there is
  unchanged.
- **Maintain the index on append.** A direct write that appends a new key must
  record it, in O(1), or the index goes stale. This is not optional bookkeeping:
  the design brief found that a stale index would report a freshly appended key
  as `Missing`. That window is currently masked because writes after a wide read
  copy rather than mutate in place — so the moment the write path becomes more
  in-place, the window opens. Replacement of an existing key's value needs no
  index maintenance; say why in a comment.

## Two things to check before writing code

1. **The completeness invariant depends on entries having no duplicate keys.**
   Index positions are only meaningful if a key appears once. The design brief
   marked this as *not* guaranteed by construction — it relies on every producer
   of an Association `own_value` canonicalizing first, and the TOML decode path
   among others was never checked. **Verify it before relying on it.** If you
   find a producer that can install a duplicate-keyed Association, stop and
   report: that changes the design, and may be a defect in its own right.
2. **Audit every `Arc::make_mut` call site that can reach an Association's
   args**, and confirm the direct write is still the only in-place mutator. The
   design brief lists `association_lvalue.rs:70`, `part_assignment.rs:371`, and
   `incremental_append.rs:65,217` as the candidates, with the latter two gated
   to Lists. Confirm that gating rather than assuming it.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs`
- `crates/rum-runtime/src/runtime/model/cache.rs` — only what exposing the probe
  and the append hook requires
- `crates/rum-runtime/src/runtime/kernel_data/association_data.rs` — likewise,
  only if the index maintenance genuinely needs it; say why
- test modules under `crates/rum-runtime/src/runtime/`

Out of scope: everything else. Not the stability cache or its cap (Step A is
settled), not `AssociateTo` in `rum-stdlib` (its own item), and not the read
path's own behaviour.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- The write path using the index, with the scan retained outside the indexed
  range.
- O(1) index maintenance on append, with a test that appends after a wide read
  and then reads the appended key back — the exact stale window named above.
- Before/after timings at n = 8000 / 16000 / 20000 / 32000, and an allocation
  counter reading showing writes still mutate in place (one shared-args
  allocation per write, as Step A established).
- The two pre-checks answered.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (8000, 16000, 20000, 32000):
    open("/tmp/b315-%d.rum" % n, "w").write(
        'a = <||>;\nPrint["n=%d t=", AbsoluteTiming[Do[a[i] = i, {i, 1, %d}]], " len=", Length[a]];\n'
        % (n, n))
PY
for n in 8000 16000 20000 32000; do ./target/release/rum-cli /tmp/b315-$n.rum < /dev/null; done
```

Growth must be linear rather than ~3.6× per doubling, with the correct `len=`.

**The microbench A/B is a hard gate here**, and the risk is specific: the probe
adds work to every direct write, including small Associations that never
benefit from an index. Run `scripts/core-language-microbench.sh` on base and
candidate and paste the association labels **and** their same-tier neighbors. A
regression on a small-Association label is a defect to fix, not to explain. If
the machine is not quiet, say so and lead with counters.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- **Ownership is the trap in this area.** Anything held across the mutation
  window makes `make_mut` copy and turns an O(1) write into an O(n) one. If your
  change holds a strong reference there, stop and report.
- **Do not weaken the validation.** Every entry must still be checked before the
  mutation window, whatever path the position lookup takes.
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges, and where validation now
   happens relative to the lookup.
2. The two pre-checks, answered with evidence.
3. Verification — each command with verbatim output: test counts, the timing
   table, the allocation counter, the stale-window test, and the A/B rows.
4. Stop-loss / open questions.
