# brief-322 — convert RumTokens offsets with one cursor

## Goal

`RumTokens` converts each token span's byte offsets to character offsets by
counting characters from the start of the source, twice per token, so
tokenizing is superlinear. Replace that with a single monotonic cursor over one
pass. Output must be identical.

## Measured today (release CLI, verified by the director — do not re-derive)

`RumTokens[StringRepeat["f[x] + 1; ", n]]`:

| tokens | time |
|---|---|
| 25 000 | 0.0495 s |
| 50 000 | 0.1263 s |
| 90 000 | **0.3734 s** |

1.8× the input for 2.96× the time. (Larger inputs hit
`GeneratedElementLimit`, which is why the series stops at 90 000.)

## The mechanism

`crates/rum-stdlib/src/strings/tokenize.rs:302-304`:

```rust
fn byte_to_char_offset(source: &str, byte_offset: usize) -> usize {
    source[..byte_offset].chars().count()
}
```

`byte_token_span_expr` (`:285-292`) calls it **twice per span**, once for the
start and once for the end. Each call rescans the source prefix, so the total is
quadratic in the source length.

## The fix, and the property it rests on

Spans are emitted in ascending order and do not overlap — tokens and trivia
partition the source — so all the byte offsets requested, across every span and
both ends of each, form a non-decreasing sequence. One cursor holding
`(byte_position, char_position)` can therefore answer each request by advancing
from the previous one, giving one pass over the source with no allocation.

**Verify that ordering rather than assuming it**: confirm at the call site that
spans are produced in source order and that a span's end is never before its
start. If any caller can request offsets out of order, say so and stop — the
cursor needs a different shape, and a silently wrong offset is worse than a slow
one.

## Output that must not change — measured, not assumed

Offsets are **character** positions, one-based and inclusive:

| input | result |
|---|---|
| `RumTokens["f[x]"]` | `{{"symbol",{1,1}},{"[",{2,2}},{"symbol",{3,3}},{"]",{4,4}}}` |
| `RumTokens["f[\"中文\"]"]` | the string token is `{3,6}` — four characters, not the ten bytes it occupies |
| `RumTokens["a (* c *) + b"]` | whitespace and comment trivia each carry spans: `{2,2}`, `{3,9}`, `{10,10}`, `{12,12}` |

The multi-byte case is the one that fails silently if the cursor advances by
bytes where it should advance by characters. Capture goldens from the
**pre-change build** and run them green against the unmodified code first, as
briefs 310 and 312 did.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/strings/tokenize.rs`
- the `rum-stdlib` test module for tokenization

Out of scope: everything else. In particular the error path re-lexes the source
(`nonempty_error_end` and its callers) — that is a bounded constant factor, not
a complexity problem, and it is not part of this change.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Offset conversion via one monotonic cursor, with the ordering property
  verified at the call site.
- Goldens covering ASCII, multi-byte characters, trivia spans, an empty source,
  and a source that is entirely one token. State the corpus size.
- Before/after scaling at 25 000 / 50 000 / 90 000 tokens.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (2500, 5000, 9000):
    open("/tmp/b322-%d.rum" % n, "w").write(
        's = StringRepeat["f[x] + 1; ", %d];\n'
        'Print["tokens=", Length[RumTokens[s]], " t=", AbsoluteTiming[Length[RumTokens[s]]]];\n'
        % n)
PY
for n in 2500 5000 9000; do ./target/release/rum-cli /tmp/b322-$n.rum < /dev/null; done
```

Token counts must be unchanged and growth must be linear.

No microbench A/B: `RumTokens` is a string builtin, not the evaluation hot path.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.**
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If offsets can be requested out of order, or a golden
  differs, STOP and report rather than adjusting anything.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges.
2. The ordering property, with the evidence you used to confirm it.
3. Verification — each command with verbatim output: test counts, corpus size,
   and the before/after scaling.
4. Stop-loss / open questions.
