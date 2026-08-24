# brief-312 — indexed TOML decoding

## Goal

`FromToml` is quadratic in the number of keys in a table: every insertion scans
every key already in that table. Give the lookup an adaptive index so decoding
is linear, with the parsed result, the key order, and the duplicate-key errors
all unchanged.

## Measured today (release CLI, verified by the director — do not re-derive)

A flat TOML document of `n` top-level keys (`k1 = 1`, `k2 = 2`, …), decoded via
`FromToml`:

| n | time |
|---|---|
| 2000 | 0.0062 s |
| 4000 | 0.0210 s |
| 8000 | 0.0928 s |
| 16000 | **0.2893 s** |

3.1–4.4× per doubling.

## The mechanism

`crates/rum-stdlib/src/toml/model.rs:56-60`:

```rust
pub(super) fn entry_index(&self, key: &str) -> Option<usize> {
    self.entries.iter().position(|(existing, _)| existing == key)
}
```

Every key insertion and every path step calls this, so a table of `n` keys costs
O(n²) string comparisons. `table_at_path_mut` (`decode.rs:85`) walks from the
root per key-value line, and each step of that walk goes through the same linear
lookup — so nested documents pay it per level too.

## The fix

Keep `entries: Vec<(String, TomlValue)>` as the source of truth — **key order is
observable** (`FromToml["z = 1\na = 2\nm = 3"]` yields `<|"z" -> 1, "a" -> 2,
"m" -> 3|>`, insertion order, not sorted) — and add a lazily built
`HashMap<String, usize>` mirror consulted only for lookup, built once a table
grows past a threshold.

Follow the in-tree pattern rather than inventing one: `csv.rs:455`
(`HEADER_INDEX_THRESHOLD = 16`), `associations/grouping.rs:10` and
`structure_ops/stable_groups.rs:5` (`GROUP_INDEX_THRESHOLD = 16`). Below the
threshold the existing linear scan stays, so small tables allocate nothing new —
a TOML document of five keys must not start building hash maps.

The mirror has to stay correct across mutation: entries are pushed, and values
inside them are replaced in place. Whatever invalidation rule you choose, state
it explicitly in a comment, and make sure a table that grows past the threshold
*after* some entries already exist ends up with a complete index.

## Behavior that must not change

1. **Key order** — insertion order, as above.
2. **Duplicate-key rejection.** All three forms currently yield `$Failed`:
   a repeated top-level key (`a = 1\na = 2`), a repeated key inside a table
   (`[t]\nx = 1\nx = 2`), and a repeated table header (`[t]\nx = 1\n[t]\ny = 2`).
   The messages that accompany them must not change either — capture them before
   you start and assert them afterwards.
3. **Error message text and the offsets they carry** for malformed input
   generally.

Pin these with golden values captured from the **pre-change build**, and — as in
brief-310 — run the golden test green against the unmodified decoder first, so
the goldens cannot be fitted to the new code afterwards.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/toml/model.rs`
- `crates/rum-stdlib/src/toml/decode.rs`
- the `rum-stdlib` TOML test module

Out of scope: everything else, including `toml/encode.rs` (its quadratic was
fixed in `d3db9f3` — do not revisit it) and `rum-runtime`.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Adaptive index behind a stated threshold, with the small-input path unchanged.
- Golden tests for the three duplicate forms, key order, nested tables, arrays
  of tables, and inline tables. State the corpus size.
- Before/after scaling for n = 2000 / 4000 / 8000 / 16000.
- A nested-document measurement too — a document with many keys spread across
  many tables — since `table_at_path_mut` pays the same lookup per level.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise. Note the kernel's own
`AbsoluteTiming` is the right instrument here; `/usr/bin/time`'s resolution is
coarser than the smaller probes:

```
python3 - <<'PY'
for n in (2000, 4000, 8000, 16000):
    doc = "\\n".join("k%d = %d" % (i, i) for i in range(1, n + 1))
    open("/tmp/b312-%d.rum" % n, "w").write(
        's = "%s";\nPrint["n=%d keys=", Length[Keys[FromToml[s]]], " t=", AbsoluteTiming[Length[FromToml[s]]]];\n'
        % (doc, n))
PY
for n in 2000 4000 8000 16000; do ./target/release/rum-cli /tmp/b312-$n.rum < /dev/null; done
```

Each must report the same `keys=` as before, and growth must be linear rather
than ~4× per doubling.

No microbench A/B: `FromToml` is a codec builtin, not the evaluation hot path.
Say so if you believe otherwise.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses. Golden output captured
  from rum itself is fine.
- **No new `unsafe`.**
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If a golden value differs, STOP and report both strings
  rather than adjusting the golden.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges, plus the invalidation rule you
   chose for the mirror and why it is correct.
2. Verification — each command with verbatim output: test counts, corpus size,
   the `keys=` equality, and the before/after scaling for both the flat and the
   nested documents.
3. The verdict — achieved complexity, and the threshold you used.
4. Stop-loss / open questions.
