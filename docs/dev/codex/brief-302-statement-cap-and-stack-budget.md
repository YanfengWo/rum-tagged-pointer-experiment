# brief-302 — land the statement cap, fix the operator cap to the stack budget

## Goal

Finish what brief-301 stopped short of, so the whole change can land. When you
are done: the operator-chain cap is a number the guard can actually reach on a
small stack; a form holding too many `;`-separated statements fails with its own
actionable message instead of aborting the process; and every test passes on
libtest's ordinary worker thread with no big-stack helper.

## Starting state — READ THIS FIRST

**The working tree already contains brief-301's uncommitted changes.** You are
continuing on top of them, not starting from a clean tree. Do not revert them,
and do not re-do the parts that are already correct:

- `MAX_DELIMITER_NESTING_DEPTH` / `MAX_OPERATOR_CHAIN_DEPTH` constants and their
  two independent counters — done, correct.
- `parse_compound_expression_chain` (iterative `;` collection, right fold,
  `Null` + `break` for an empty statement) — done, correct.
- `lower_compound_expression_sugar` (iterative `;` spine in lowering) — done,
  correct.
- The docs edits and the existing new tests — done; you will extend them.

brief-301 correctly hit its own stop-loss clause at Step 4 and left three things
undone: the statement cap, the dedicated error variant, and the docs text for a
third cap. This brief lands those, and corrects one number that brief-301 was
told to take on faith.

## Measurements already established — do not re-derive

- Debug parser: ~4.6 KiB of native stack per operator-chain link. A 2 MiB
  thread aborts at 452 `^` links.
- **Therefore `MAX_OPERATOR_CHAIN_DEPTH = 1024` is unreachable on a 2 MiB
  stack**: the process overflows long before the guard fires. Verified by
  temporarily disabling the `with_big_stack` helper and running
  `cargo test -p rum-syntax`:
  `thread 'parser::tests::basics::pins_power_chain_operator_depth_boundary' has
  overflowed its stack / fatal runtime error: stack overflow (SIGABRT)`.
- Statement chains, all-management measured on the current tree: the first thing
  to die is the evaluator's per-item resume recursion
  (`resume_compound_expression_from_index` /
  `resume_compound_expression_after_item`,
  `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:302-346`,
  ~1.5 KiB/item), aborting at 1408 statements on a 2 MiB thread. Parser and
  lowering, both now iterative, reach 14532.
- Release CLI on the main thread, current tree: 2000 and 7000 statements run;
  **10000 statements abort with a stack overflow**.
- Release CLI, current tree: `a` followed by 10000 space-separated `;` (i.e.
  consecutive EMPTY statements) **aborts with a stack overflow**; 1000 is fine.

## Context pointers

- `AGENTS.md` first (operational notes, Testing Policy hard lines).
- `docs/dev/codex/brief-301-parse-depth-split-and-statement-chains.md` — the
  brief you are continuing; its Step 4b describes the error variant in detail
  and its ripple analysis still applies verbatim.

## Scope

Files you MAY modify:

- `crates/rum-syntax/src/parser/front.rs`
- `crates/rum-syntax/src/error.rs`
- `crates/rum-syntax/src/parser/tests/basics.rs`
- `crates/rum-syntax/src/parser/tests/apply_compound.rs`
- `crates/rum-runtime/src/runtime/tests/lowering_and_context.rs`
- `docs/guide/resource-limits-and-sandbox.md`
- `docs/dev/publication-audit/capability-consistency-findings.md`

Everything else is out of scope. In particular do NOT touch
`crates/rum-runtime/src/runtime/lowering.rs` (already correct),
`branching_loops.rs` (the evaluator resume recursion is a separate lane), or
`context_and_load_helpers.rs`.

## Step 1 — correct the operator-chain cap to the stack budget

Change `MAX_OPERATOR_CHAIN_DEPTH` from 1024 to **128**.

The rationale matters, so record it in the constant's doc comment: this cap
exists to fail gracefully *before* the native stack runs out, so it must be
sized by the stack budget, not by semantic analogy with the evaluator's
`DEFAULT_RECURSION_LIMIT`. At ~4.6 KiB per link a 2 MiB thread — Rust's default
for a spawned thread, and the realistic worst case for an embedder — dies at
452 links, so a cap of 1024 can never fire there. 128 keeps roughly a 3.5×
margin on that measurement and matches the delimiter cap, which `2a23d5e` chose
the same way.

Then **delete the `with_big_stack` helper** in `parser/tests/basics.rs` and run
its three call sites as ordinary tests. With a reachable cap they no longer need
an oversized stack. If any of them still overflows on libtest's default thread,
STOP and report — that would mean 128 is also too high.

## Step 2 — add the statement cap, counted along the whole spine

Add `MAX_STATEMENT_CHAIN_DEPTH = 256` and enforce it.

**The counting rule is the crux of this brief — read it twice.** A naive guard
placed inside `parse_compound_expression_chain`, counting that one collection
loop, does NOT work. Consecutive empty statements (`a ; ; ; …`) push `Null`,
`break` immediately, and return to the climbing loop in
`parse_binary_expr_internal`, which then re-enters the chain helper for the next
`;`. Each collection loop therefore holds at most two operands while the spine
grows without bound — measured above, 10000 such statements abort the process
today.

So count the spine, not the loop: keep a counter local to the
`parse_binary_expr_internal` call that owns the climbing loop, increment it once
per `;` link consumed — **including the trailing-`Null` branch and every
re-entry into the chain helper** — and fail once it would exceed
`MAX_STATEMENT_CHAIN_DEPTH`. A function-local counter is the right tool: the
climbing loop owns exactly one spine, so the count starts fresh for an
independent `;` spine elsewhere in the input (e.g. `f[a;b]` and `g[c;d]` must
not accumulate together — pin this with a test).

## Step 3 — the dedicated error variant

As specified in brief-301 Step 4b, add to `crates/rum-syntax/src/error.rs`:

```rust
#[error("statement chain exceeded limit {max} at bytes {span}; split the statements across separate lines")]
StatementChainTooLong { max: usize, span: Span },
```

The advice is concrete and correct: `split_script_into_forms` parses a script
line by line, so breaking one long `;` line into several lines makes the same
program parse and run.

Two mechanical ripples in that same file:

1. **`SyntaxError::span()` is an exhaustive match with no `_` arm** — the new
   variant MUST be added, returning `Some(*span)`, or the crate will not
   compile, and the CLI would lose its `--> line:col` caret.
2. **`SyntaxError::is_incomplete()` is a `matches!` allow-list** — the new
   variant MUST NOT be added to it, and you must prove it stays false. If this
   error were ever classified as "incomplete",
   `split_script_into_forms` (`context_and_load_helpers.rs:128`) would keep
   appending later lines to `current` forever, silently swallowing the rest of
   the script instead of reporting anything.

`related_span` / `related_label` have `_` arms and need no change.
`ParseDepthExceeded` keeps its shape, its message, and its two current users.

## Step 4 — tests

All of these must pass on libtest's ordinary worker thread. No big-stack helper.

1. `parser/tests/basics.rs` — `MAX_OPERATOR_CHAIN_DEPTH ± 1` boundary for `^`,
   as ordinary tests (the `with_big_stack` wrappers removed).
2. `parser/tests/apply_compound.rs` — `MAX_STATEMENT_CHAIN_DEPTH` statements
   parse; one more returns `StatementChainTooLong` (NOT `ParseDepthExceeded`),
   and the rendered message contains both the limit and the "separate lines"
   advice.
3. `parser/tests/apply_compound.rs` — **the empty-statement spine**: `a` followed
   by well over `MAX_STATEMENT_CHAIN_DEPTH` space-separated `;` returns
   `StatementChainTooLong` rather than overflowing. This is the regression this
   brief exists to prevent; without the Step 2 counting rule it aborts.
4. `parser/tests/apply_compound.rs` — independent spines do not accumulate:
   something like `f[a;b]; g[c;d]` (and a case with two separate deep-ish spines
   well under the cap each) parses fine.
5. `parser/tests/apply_compound.rs` — the existing `a; ; b` shape pin and the
   `"a; b;"` golden test still pass unchanged.
6. `runtime/tests/lowering_and_context.rs` — the swallowing regression: a script
   whose FIRST line is an over-cap `;` chain and whose LATER lines are ordinary
   statements must still split into multiple forms. Assert on
   `split_script_into_forms` output directly, and assert `is_incomplete()` is
   false for the new variant.

## Step 5 — docs

- `docs/guide/resource-limits-and-sandbox.md` — the §Parse Depth section must
  describe all three caps with their current values.
- `docs/dev/publication-audit/capability-consistency-findings.md` — the rows for
  these caps must match the final constant names, values, and `path:line`.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli`, paste verbatim output for
each of these — the first three must print a graceful error and **must not
abort**:

```
python3 -c "print('; '.join('x%d = %d'%(i,i) for i in range(300)))" > /tmp/b302-a.rum
python3 -c "print('a' + ' ;'*10000)" > /tmp/b302-b.rum
python3 -c "print('; '.join('x%d = %d'%(i,i) for i in range(10000)))" > /tmp/b302-c.rum
python3 -c "print('\n'.join('v%d = %d;'%(i,i) for i in range(1,10001)) + '\nPrint[\"DONE\"]')" > /tmp/b302-d.rum
for f in a b c d; do echo "--- $f ---"; ./target/release/rum-cli /tmp/b302-$f.rum; echo "exit=$?"; done
```

Expected: `a`, `b`, `c` report the statement-chain limit with the "separate
lines" advice and a byte span, exit non-zero, and do NOT say "parse nesting" and
do NOT abort. `d` — the 10 000-line script — must still print `DONE` and exit 0,
proving the per-line script path is untouched.

Report the exit codes. An exit code of 134 anywhere is a stack overflow and is a
failure of this brief.

No microbench A/B: no evaluation hot path is touched.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes in
  the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Only file edits within Scope,
  `cargo`, the release `rum-cli`, and `python3` for probes.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If a cap still cannot be reached on libtest's default
  stack, or the empty-statement spine still overflows after Step 2, STOP and
  report the measurement rather than inventing a workaround or enlarging a
  stack to hide it.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command with exact verbatim output, including the four
   CLI probes and their exit codes.
3. The verdict — the three constants' final values, and confirmation that no
   test needs an enlarged stack.
4. Stop-loss / open questions — where you stopped and why (empty if none).
