# brief-301 — split the parse-depth guard and make `;` chains iterative

## Goal

`MAX_PARSE_DEPTH = 128` currently guards two structurally unrelated kinds of
recursion, and since `28afe35` it also limits how many `;`-separated statements
may appear inside a single parsed form. When you are done: the delimiter cap and
the operator-chain cap are two separately named constants with two independent
counters; `;` chains no longer consume recursion depth in either the parser or
lowering; a form that still holds too many statements fails with its own
message that says what went wrong and how to fix it; and the regression is
pinned by tests.

This is an implementation brief. The plan behind it is settled — do not
redesign it. Where a number must be measured rather than assumed, this brief
says exactly what to measure and what rule turns the measurement into the
number.

## Background (already established — do not re-derive)

- `2a23d5e` (2026-06-30) introduced the guard for **delimiter** recursion only.
  Its commit message: "Add a MAX_PARSE_DEPTH=128 guard on the delimiter
  recursion (`[...]`, `[[...]]`, `(...)`, `{...}`, `<|...|>`)". 128 was chosen
  because the parser uses several Rust frames per delimiter.
- `28afe35` (2026-08-23) wired that same guard into binary-operator recursion
  via `parse_nested_binary_expr` / `parse_nested_binary_expr_internal`. Since
  `CompoundExpression` (`;`) is right-associative
  (`crates/rum-syntax/src/ast.rs:386`), statement count now consumes the cap.
- Measured on the current tree: a single form holding 130 `;`-separated
  statements fails with `parse nesting exceeded limit 128`; 100 parses. With
  `front.rs` reverted to `28afe35~1`, 1000 parse and ~5000 overflow the native
  stack.
- Script **files** are unaffected, because `split_script_into_forms`
  (`crates/rum-runtime/src/runtime/context_and_load_helpers.rs:100`) splits a
  script line-by-line into independently parsed forms. A 10 000-line script of
  one statement per line runs in 0.04 s today. The regression is confined to a
  single form: a long one-line `;` chain, or a long right-associative chain.
- `lower_syntax_expr_with` (`crates/rum-runtime/src/runtime/lowering.rs:9`) is a
  recursive descent whose depth on a `;` spine also equals the statement count.
  Its *output* is already flat: `push_compound_expression_item`
  (`lowering.rs:530`) folds nested `;` into one n-ary
  `CompoundExpression[a, b, c, …]`. **This is why the parser and lowering must
  be changed together**: making only the parser iterative would turn today's
  graceful `ParseDepthExceeded` into a native stack overflow in lowering.

## Context pointers

- Read `AGENTS.md` (operational notes, the Testing Policy hard lines) first.
- Key files:
  - `crates/rum-syntax/src/parser/front.rs` — the guard, the two helpers, the
    precedence-climbing loop.
  - `crates/rum-runtime/src/runtime/lowering.rs` — the recursive descent whose
    `;` spine must become iterative.
  - `crates/rum-syntax/src/ast.rs:383-411` — `is_right_associative`.
- Acceptance is mechanical: the named tests and the named measurements decide.

## Scope

Files you MAY modify:

- `crates/rum-syntax/src/parser/front.rs`
- `crates/rum-syntax/src/error.rs`
- `crates/rum-syntax/src/parser/tests/support.rs`
- `crates/rum-syntax/src/parser/tests/basics.rs`
- `crates/rum-syntax/src/parser/tests/apply_compound.rs`
- `crates/rum-runtime/src/runtime/lowering.rs`
- `crates/rum-runtime/src/runtime/tests/lowering_and_context.rs`
- `docs/guide/resource-limits-and-sandbox.md` (the §Parse Depth text at :113)
- `docs/dev/publication-audit/capability-consistency-findings.md` (the row
  at :714 — update the constant name/lines, and add a row for the new cap)

Files/areas you MUST NOT touch: everything else. Specifically not
`crates/rum-core/src/expr.rs`, not `AGENTS.md` / `PROGRESS.md` / `TODO.md` /
`CHANGELOG.md`, not `.git`, not `Cargo.lock`, and not
`context_and_load_helpers.rs` (its O(N²) re-parse is a separate, already-filed
issue — do not fix it here, do not "improve" it in passing).

Do NOT add unconditional cost to the default evaluation path.

## Step 1 — split the constant and the counter

Replace the single `MAX_PARSE_DEPTH` / `parse_depth` pair with two:

- `MAX_DELIMITER_NESTING_DEPTH = 128`, counted by `delimiter_depth`. This name
  is chosen deliberately over "bracket": the guard also covers `( … )`, which
  "bracket" does not describe. Keeps exactly the five delimiter call sites
  (`front.rs:289`, `:300`, `:524`, `:532`, `:540` on the current tree).
- `MAX_OPERATOR_CHAIN_DEPTH = 1024`, counted by `operator_chain_depth`. Used by
  `parse_nested_binary_expr` / `parse_nested_binary_expr_internal` (`front.rs:100`,
  `:110`) and therefore by the prefix-operator and remaining right-associative
  chains.

Rationale for 1024, so you do not have to guess: rum's own evaluator default is
`DEFAULT_RECURSION_LIMIT = 1024` (`crates/rum-runtime/src/runtime/metadata.rs:4`).
A parse cap materially above that would only admit ASTs that the default session
then refuses to evaluate.

Two independent counters mean the worst case is interleaved — 128 delimiters
each wrapping an operator chain. Step 4 measures whether that combination is
stack-safe.

`SyntaxError::ParseDepthExceeded { max, span }` keeps its shape and its message
text unchanged; both of these two guards keep reporting through it. The
statement-chain cap gets its own variant instead — see Step 4b.

## Step 2 — make `;` chains iterative in the parser

In `parse_binary_expr_internal`, when the climbing loop meets
`BinaryOp::CompoundExpression` and it passes the existing precedence check,
hand the whole chain to a new iterative helper instead of recursing. The helper
collects operands into a `Vec` and then folds right, so the AST shape is
unchanged.

Three behaviours must be preserved exactly; each has a test in Step 5:

1. **Right-associative AST shape.** Fold from the right so the result is
   identical to what the recursive form produced. The existing golden test at
   `crates/rum-syntax/src/parser/tests/apply_compound.rs:97` (`"a; b;"` →
   `"a; b; Null"`) must pass untouched.
2. **Trailing `;` yields `Null`.** Keep the existing
   `!self.token_starts_expression()` branch (`front.rs:184`).
3. **A mid-chain empty statement stays locally left-leaning.** In the recursive
   form, `a; ; b` parses as `(a; Null); b` — the `Null` branch returns to the
   outer climbing loop, which then consumes the second `;`. Reproduce this by
   **breaking** out of the collection loop after pushing `Null`, not by
   continuing it; continuing would produce `a; (Null; b)`, a silent shape
   change. (Note: `a;;b` is NOT this case — `;;` lexes as a single `Span` token
   and never reaches `CompoundExpression`. Do not use it as a test for this.)

The `;` chain must no longer increment either counter for its own links.
Operands *inside* each statement still go through the operator-chain guard
normally.

## Step 3 — make the `;` spine iterative in lowering

`lower_binary_sugar` (`lowering.rs:273`) recurses into both sides before
matching on the operator. For `BinaryOp::CompoundExpression` specifically, walk
the right spine iteratively — collect the spine's left operands in a loop
descending through `rhs`, lower each one, then assemble via the existing
`push_compound_expression_item` so the flat n-ary output is byte-identical to
today's.

Everything else in `lower_binary_sugar` keeps its current recursive shape. Do
not restructure the whole function.

## Step 4 — measure, then set the statement cap

After Steps 2 and 3, a `;` chain no longer recurses in the parser or in
lowering, but the nested `SyntaxExpr` tree is still dropped recursively, and
other consumers may still walk it. So measure where a long chain actually dies
now:

Binary-search the smallest `;`-chain length in a **single form** that aborts the
process, under the configuration that fails first:

- a `cargo test` debug binary (debug frames are fatter than release), and
- a thread with a 2 MiB stack (`std::thread::Builder::new().stack_size(2 << 20)`),
  which is the worst realistic embedding case, alongside the default main thread.

Report the measured numbers verbatim. Then set:

- `MAX_STATEMENT_CHAIN_DEPTH = min(4096, measured_minimum_crash / 4)`

and guard the `;` collection loop with it, reporting through the same
`ParseDepthExceeded`. If `measured_minimum_crash / 4` lands below 2048, do NOT
pick a number yourself — stop and report the measurement, because that would
mean the spine dies far earlier than expected and the fix needs rethinking.

Also measure the interleaved worst case from Step 1 (128 delimiters each
wrapping a ~1024-long operator chain) and report whether it survives. If it
aborts, report it — do not unilaterally lower `MAX_OPERATOR_CHAIN_DEPTH`.

## Step 4b — give the statement-chain cap its own message

Hitting this cap means "one form holds too many `;`-separated statements",
which is a different problem from "this expression nests too deeply". Reporting
it as `parse nesting exceeded limit N` would actively mislead, so add a
dedicated variant in `crates/rum-syntax/src/error.rs`:

```rust
#[error("statement chain exceeded limit {max} at bytes {span}; split the statements across separate lines")]
StatementChainTooLong { max: usize, span: Span },
```

The "split across separate lines" advice is concrete and correct, not filler:
`split_script_into_forms` parses a script line by line, so breaking one long
`;` line into several lines makes the same program parse and run. Keep the
wording aligned with the existing style (`"... exceeded limit {max} at bytes
{span}"`).

Two mechanical ripples in the same file — both are compile- or behaviour-level,
not cosmetic:

1. **`SyntaxError::span()` (`error.rs:46`) is an exhaustive match with no `_`
   arm.** The new variant MUST be added there, returning `Some(*span)`, or the
   crate will not compile. Without it the CLI also loses its `--> line:col`
   caret.
2. **`SyntaxError::is_incomplete()` (`error.rs:36`) is a `matches!` allow-list.**
   The new variant MUST NOT be added to it, and you must prove it stays false.
   This is the dangerous one: if the statement-chain error were ever classified
   as "incomplete", `split_script_into_forms`
   (`crates/rum-runtime/src/runtime/context_and_load_helpers.rs:128`) would keep
   appending later lines to `current` forever, silently swallowing the rest of
   the script instead of reporting anything.

Do not add the variant to any other classification helper. `related_span` and
`related_label` have `_` arms and need no change.

## Step 5 — tests

Add, in the files listed in Scope:

1. `parser/tests/apply_compound.rs` — a long single-form `;` chain (use a length
   comfortably above 128 and below the Step 4 cap, e.g. 2000) parses
   successfully. This is the regression pin.
2. `parser/tests/apply_compound.rs` — `a; ; b` produces the locally
   left-leaning shape from Step 2.3, asserted on the AST, not on `to_string`.
3. `parser/tests/basics.rs` — rename the two existing delimiter tests to the new
   constant; add a `MAX_DELIMITER_NESTING_DEPTH ± 1` boundary pin.
4. `parser/tests/basics.rs` — the two 16 000-deep chain tests added by `28afe35`
   (`rejects_deep_right_associative_chain_without_panicking`,
   `rejects_deep_prefix_chain_without_panicking`) must now assert against
   `MAX_OPERATOR_CHAIN_DEPTH`; add a `MAX_OPERATOR_CHAIN_DEPTH ± 1` boundary pin
   for `^`.
5. `runtime/tests/lowering_and_context.rs` — a long `;` chain lowers to a flat
   n-ary `CompoundExpression` with the correct arity and element order, proving
   Step 3 did not change the output shape.
6. `parser/tests/basics.rs` — a `;` chain one past `MAX_STATEMENT_CHAIN_DEPTH`
   returns `StatementChainTooLong` (NOT `ParseDepthExceeded`), and its rendered
   message contains both the limit and the "separate lines" advice.
7. `runtime/tests/lowering_and_context.rs` — the swallowing regression from
   Step 4b.2: a script whose FIRST line is an over-long `;` chain and whose
   LATER lines are ordinary statements must still split into multiple forms —
   i.e. the over-long line is reported and the following lines are not absorbed
   into it. Assert on `split_script_into_forms` output directly, and assert
   `is_incomplete()` is false for the new variant.

## Step 6 — docs

- `docs/guide/resource-limits-and-sandbox.md:113` — the sentence "The parser has
  `MAX_PARSE_DEPTH = 128`." and its surrounding §Parse Depth section must
  describe the three caps accurately.
- `docs/dev/publication-audit/capability-consistency-findings.md:714` — update
  the constant name and `path:line` reference, and add a row for the new caps.

`PROGRESS.md:786` mentions the old name in an append-only historical entry.
**Do not touch it** — it is a record of what was true then.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Plus, pasted verbatim:

- the Step 4 binary-search numbers (both stack configurations, and the
  interleaved worst case);
- the final values chosen for all three constants;
- proof the regression is fixed, via the release CLI on a single-form chain that
  fails on the current tree:

```
cargo build --release -p rum-cli
python3 -c "print('; '.join('x%d = %d'%(i,i) for i in range(200)) + '; Print[\"OK\"]')" > /tmp/brief301.rum
./target/release/rum-cli /tmp/brief301.rum
```

This prints `parse nesting exceeded limit 128` before your change and must print
`OK` after it.

Then show the new message on a chain that exceeds the Step 4 cap — substitute
your measured `MAX_STATEMENT_CHAIN_DEPTH` for `<CAP>`:

```
python3 -c "n=<CAP>+10; print('; '.join('x%d = %d'%(i,i) for i in range(n)))" > /tmp/brief301-over.rum
./target/release/rum-cli /tmp/brief301-over.rum
```

Paste its verbatim output. It must name the statement-chain limit and the
"separate lines" advice, must NOT say "parse nesting", and must still point at
a byte span.

No microbench A/B is required: this brief does not touch an evaluation hot path.
If you believe it does, say so and run the gate.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes in
  the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- Touch nothing outside this repo; no network. The only commands you run are
  file edits within Scope, `cargo`, the release `rum-cli`, and `python3` for
  generating probe inputs and parsing output.
- **Testing Policy is a hard line** (`AGENTS.md`): tests verify rum's own
  documented behavior. No comparisons against other vendors' engines, no oracle
  harnesses, no third-party engine outputs.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- Minimal diff: no drive-by reformatting or renames outside the task.
- **Stop-loss is real.** If the same step fails twice for the same reason, STOP
  and report. In particular, if Step 4's measurement contradicts this brief's
  expectations, report the numbers instead of inventing a workaround.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command, with exact verbatim output (test counts, the
   binary-search numbers, the CLI before/after).
3. The verdict — the three constants' final values and the evidence for each.
4. Stop-loss / open questions — where you stopped and why (empty if none).
