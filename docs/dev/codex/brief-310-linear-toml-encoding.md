# brief-310 — linear TOML encoding

## Goal

`ToToml` is quadratic in the number of top-level entries, with a per-step
allocation on top. Encoding 8000 table entries takes 6.9 s today. Make it
linear, with the emitted TOML **byte-for-byte identical** to what it produces
now.

## Measured today (release CLI, verified by the director — do not re-derive)

An association of `n` table-valued entries followed by one scalar entry —
`<|"t1" -> <|"x" -> 1|>, …, "z" -> 0|>`:

| n | time |
|---|---|
| 2000 | 0.41 s |
| 4000 | 1.76 s |
| 8000 | **6.90 s** |

~4× per doubling. An all-scalar association of the same width is ~0.006 s,
because the predicate below short-circuits — that contrast is the tell.

## The mechanism — two costs stacked

`crates/rum-stdlib/src/toml/encode.rs`:

- `has_plain_assignment_after(entries, index)` (`:186-190`) scans
  `entries[index + 1..]` and calls `table_like` on each. It is called from four
  sites — `:37`, `:51`, `:90`, `:103` — i.e. **twice per entry** in each of the
  two encoding loops. That alone is O(n²).
- `table_like` (`:182-184`) answers a yes/no question by calling
  `association_entries_from_expr`, which returns `Option<Vec<(Expr, Expr)>>` and
  builds it via `parse_association_entries_deep` +
  `canonicalize_association_entries`
  (`rum-runtime/src/runtime/kernel_data/association_data.rs:16-28`). So every
  step of that scan **constructs and canonicalizes the whole entry table of the
  value it is merely classifying**, then throws it away.

Multiplied out: O(n² · m) where m is the size of an entry's value.

## The fix

Two independent changes; do both.

1. **Precompute instead of rescanning.** For each table being encoded, compute
   once, in O(n), the index of the last entry whose value is not table-like.
   Then `has_plain_assignment_after(i)` is `last_plain_index > i`. Keep the
   function name and call sites if that reads better — only its cost changes.
2. **Make the predicate allocation-free.** `table_like` needs only the shape:
   is this an `Association` call, or a list of associations. Answer that by
   inspecting the expression directly rather than by building an entry table.
   Prefer implementing the shape check locally in `encode.rs` over widening
   `rum-runtime`'s API; if you find a good reason to do the latter, say so and
   stop rather than deciding unilaterally.

## The invariant that decides acceptance

**Output bytes must not change.** TOML forbids a bare assignment after a
section header, and this predicate is exactly what enforces that ordering — so a
mistake here is a silent correctness change in emitted files, not a formatting
nit.

Pin it the way step 1 of the splitter work was pinned: **capture golden output
from the current tree before changing anything.** Build the release CLI first,
generate the TOML text for a corpus of shapes, save those strings into the test
module, and assert equality after the change. The corpus must include, at
minimum: all-scalar tables; all-table tables; a scalar after a table (the shape
that triggers the quadratic); a scalar *between* two tables; nested tables;
lists of associations; an empty association; an empty nested association; and
mixed key types that reach the error paths. State the corpus size.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/toml/encode.rs`
- the `rum-stdlib` test module for TOML encoding

Out of scope: everything else — in particular `crates/rum-stdlib/src/toml/`'s
decoder (`decode.rs`, `model.rs`) has its own quadratic, which is a separate
brief, and `rum-runtime`'s association helpers.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- `ToToml` linear in the entry count, with the allocating predicate gone.
- The golden-output test with its corpus.
- Before/after scaling numbers for n = 2000 / 4000 / 8000, plus the all-scalar
  control showing it did not regress.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (2000, 4000, 8000):
    parts = ", ".join('"t%d" -> <|"x" -> %d|>' % (i, i) for i in range(1, n + 1))
    open("/tmp/b310-%d.rum" % n, "w").write(
        'a = <|%s, "z" -> 0|>;\nPrint["len=", StringLength[ToToml[a]]];\n' % parts)
scal = ", ".join('"k%d" -> %d' % (i, i) for i in range(1, 8001))
open("/tmp/b310-scalar.rum", "w").write(
    'a = <|%s|>;\nPrint["len=", StringLength[ToToml[a]]];\n' % scal)
PY
for f in 2000 4000 8000 scalar; do
  echo "--- $f ---"; /usr/bin/time -p ./target/release/rum-cli /tmp/b310-$f.rum < /dev/null
done
```

Each must print the same `len=` as before your change — record those lengths
from the pre-change build and compare. Growth must be linear rather than ~4×
per doubling.

No microbench A/B: `ToToml` is not on the evaluation hot path. Say so if you
believe otherwise.

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
- **Stop-loss is real.** If any corpus shape's output differs, STOP and report
  the shape and both strings rather than adjusting the golden value.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges.
2. Verification — each command with verbatim output: test counts, the corpus
   size, the `len=` equality, and the before/after scaling table.
3. The verdict — achieved complexity, and what the per-step allocation cost.
4. Stop-loss / open questions.
