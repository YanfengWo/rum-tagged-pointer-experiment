# brief-335 — a fixed ceiling on left-associative chains (plan-330 step 5)

Last of five steps from `docs/dev/codex/plan-330-left-chain-lowering.md`.
Steps 1-4 landed in `70af8378`, `62839ffa`, `541ba6af` and `91c20e9d`.

Steps 1-4 removed the recursion. This step adds the policy limit that the
remaining recursions — ones this plan does not remove — still require.

## Why a ceiling at all, when the recursion is gone

Three recursions were in play; only lowering's was removed:

- the formatter's `SyntaxExpr` walk, and
- `SyntaxExpr`'s own drop glue, which is a property of its boxed-children
  representation and cannot go without changing that representation.

So some link is always the weakest, and the ceiling is structural rather than
evasive. Iteration and the ceiling do different jobs: iteration makes the
accepted depths safe on any stack; the ceiling refuses the depths that would
still reach the formatter and the drop.

Measured brackets (director's, do not re-derive):

```
8 MiB main thread: a + chain completes at 100 000, aborts at 200 000 in drop
2 MiB stack:       the formatter passed 8 000 links, aborted by 12 000
```

4 096 sits about 2.9x below the first observed failure and 32x above the
existing recursive-parser guard.

## What to build

`MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS = 4_096`, counted in binary links, defined
once in `rum-syntax` and used by both the parser and the runtime collector.
Exactly 4 096 is admitted; link 4 097 is rejected at its operator token.

**Why not reuse `MAX_OPERATOR_CHAIN_DEPTH`:** that guards native stack for
recursive right-associative parsing. The new loop has no per-link native frame;
this ceiling is an allocation and work policy. Different thing, so it gets its
own name, its own error, and a much larger value.

Enforce in two places. The parser covers ordinary source. The runtime collector
covers a `SyntaxExpr` built programmatically, which never passes through the
parser at all — bound both the sixteen-operator lhs vector and the total
comparison-chain operator vector, including comparison subtrees reached through
explicit parentheses.

**Count implicit `Times` too.** `a b c` is a left chain; it currently lowers to
`Times[a, b, c]` and completes at 100 000 juxtaposed factors, so it is real
reach, not a curiosity.

**Counting scope:** one parser counter owns one precedence-climbing lhs spine.
An independent nested or right-hand spine starts fresh — `f[a+b+c] + g[d+e+f]`
is three separate spines, not one.

## The error variant

Add `SyntaxError::LeftAssociativeChainTooLong { max, span }`, reporting
`left-associative chain exceeded limit 4096 at bytes {span}`, with the span
captured from the offending operator token before advancing.

Two arms to touch, and they differ in a way that matters:

1. **`span()` (`crates/rum-syntax/src/error.rs:50`) is an exhaustive match.**
   The compiler forces you to handle the new variant.
2. **`is_incomplete()` (`:40`) is a whitelist of four variants.** The compiler
   says nothing. **Do not add the new variant to it.** A limit error is
   terminal, not "keep reading" — `split_script_into_forms` treats an
   incomplete error as a signal to accumulate more input, so a limit error
   listed there makes the rest of a script vanish silently. That exact mistake
   was made and caught on `StatementChainTooLong`; do not repeat it.

## What must change, deliberately

The 20 000-link probes from steps 2-4 must now return the new diagnostic
instead of succeeding. **They stay as tests** — retargeted from "proves
iteration" to "rejects gracefully, never aborts". Do not delete them.

## What must not change

- Chains at or under 4 096 links, in shape and in evaluated result.
- The three existing parser limits, their values, and their messages.
- `docs/reference/builtins.md` — no catalog change, so it regenerates
  byte-identical.

## Scope

Files you MAY modify:

- `crates/rum-syntax/src/parser/front.rs`
- `crates/rum-syntax/src/error.rs`
- `crates/rum-syntax/src/lib.rs` — only to export the constant
- `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs`
- `crates/rum-runtime/src/runtime/lowering/binary/comparison.rs`
- test modules under `crates/rum-syntax/src/parser/tests/` and
  `crates/rum-runtime/src/runtime/tests/`, plus their `mod.rs` registrations
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/dev/publication-audit/capability-consistency-findings.md` — the
  parse-limit table only

Out of scope: everything else. Do not add unconditional cost to the default
evaluation path.

## Deliverable

- The constant, shared; both enforcement points; the error variant.
- Tests at exactly 4 096 (accepted) and 4 097 (rejected), for homogeneous and
  mixed chains, implicit multiplication, and independent spines.
- The retargeted 20 000-link graceful-rejection regressions.
- `is_incomplete() == false` for the new variant, as its own test.
- Span and caret diagnostics.
- A direct-AST test that bypasses the parser entirely.
- The two docs updated.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo run -q -p rum-stdlib --bin generate-builtin-reference && git diff --stat docs/reference/builtins.md
```

Then, with a fresh `cargo build --release -p rum-cli`, stdin from `/dev/null`.

**The 2 MiB thread run is the condition on which this limit was approved.** A
main-thread pass is not evidence: the embedding host's default thread is 2 MiB,
and that is the deployment the ceiling exists for. On a dedicated 2 MiB thread,
run a 4 096-link chain through the **complete** round trip — parse, lower,
format, and drop — and show it completing. Do the same for the rejection path
at 4 097 and at 20 000.

**Stop-loss, and it is a real one:** if 4 096 aborts on the 2 MiB stack in
drop, lowering, or the formatter, STOP and report to the director. Do not ship
a nominal limit whose own accepted boundary aborts, and do not quietly lower
the constant to make it pass — the number is the director's, and changing it
is the director's call.

No microbench A/B: this is parse-time policy. Say so if you believe otherwise.

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
- Do not change `SyntaxExpr`'s representation or its drop behavior.
- The constant is exactly 4 096 unless the director rules otherwise after a
  stop-loss report.
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — the constant, both enforcement points, the counting scope,
   and the diagnostic, with line ranges.
2. The boundary table: 4 096 and 4 097, homogeneous, mixed, implicit, and
   independent spines.
3. `is_incomplete()`, span/caret, and script-splitting results.
4. **The 2 MiB thread results**, for the full round trip at the accepted
   boundary and for both rejection paths.
5. Verification — commands with verbatim output and the reference diff.
6. Stop-loss / open questions.
