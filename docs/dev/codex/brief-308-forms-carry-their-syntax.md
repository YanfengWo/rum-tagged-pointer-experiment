# brief-308 — forms carry their parsed syntax

## Goal

A script form is currently parsed twice: once by the splitter to find its
boundary, and again by evaluation to build the expression. After this brief a
form produced by the linear splitter carries the `SyntaxExpr` the splitter
already built, evaluation lowers it directly, and the second parse is gone.
Observable behavior — values, messages, error text, spans, evaluation order —
is unchanged.

This is step 2 of the design in brief-306. Step 1 landed as `f5e1854`.

## The current shape

- `parse_script_forms` (`crates/rum-syntax/src/parser/script.rs:47`) lexes once,
  drives the parser form by form, and **throws the parsed `SyntaxExpr` away**,
  returning `Vec<String>`.
- `split_script_into_forms`
  (`crates/rum-runtime/src/runtime/context_and_load_helpers.rs:101`) returns
  those strings, or falls back to
  `split_script_into_forms_line_driven` when the whole-source lex fails.
- The script loop (`session/session_eval.rs:266-267`) calls
  `eval_text_with_terminated_handling(&form, …)` per form, and that function
  parses again at `session_eval.rs:199` before `lower_syntax_expr`.

`lower_syntax_expr` has no span dependency, so handing it a `SyntaxExpr` built
elsewhere is span-transparent.

## The shape to build

A form becomes two-state, not one:

- **Parsed** — carries both its text and the `SyntaxExpr` the splitter built.
  Evaluation lowers the syntax directly and never re-parses.
- **Unparsed** — carries only text. Produced by the legacy line-driven fallback,
  and by any form the splitter could not parse (a form poisoned by a hard error
  is emitted as text precisely so the error surfaces downstream). Evaluation
  handles it exactly as today.

Keeping the text on **both** states matters: it is what the differential test
compares, and what any future consumer of form text still expects.

**Why the error behavior is preserved by construction**: a hard-error form is
Unparsed, so evaluation parses it exactly as it does today and fails at the same
token with the same message and the same span. The parser is recursive descent
with no backtracking, so re-parsing the same text fails identically. Nothing
about the diagnostic path changes.

## Scope

Files you MAY modify:

- `crates/rum-syntax/src/parser/script.rs` and its module/export wiring
  (`parser.rs`, `lib.rs`)
- `crates/rum-runtime/src/runtime/context_and_load_helpers.rs`
- `crates/rum-runtime/src/runtime/session/session_eval.rs`
- `crates/rum-runtime/src/runtime/tests/context_and_loader.rs`

Out of scope: everything else. Not `crates/rum-repl/`, not the parser's own
boundary logic (step 1 is settled — do not revisit the stop sites), not the
known span-offset defect described below.

`parse_expr`, `parse_tokens`, and `eval_text` keep their existing signatures and
semantics. `eval_text` is the single-expression entry used by the REPL, the host
API, `ToExpression` and others; it must not change.

**Known defect, deliberately NOT fixed here**: error spans are computed per form
but rendered against the whole file, so an error at `2:7` reports `2:1`. Leave
it exactly as it is.

Do not add unconditional cost to the default evaluation path.

## Constraints that decide acceptance

1. **The differential test must stay meaningful.** `context_and_loader.rs:726`
   compares the linear splitter against the line-driven oracle over a 3000+
   source corpus. After this change it must still compare the **form text** of
   both implementations and still pass. Do not weaken it to accommodate the new
   type; adapt the comparison to extract text from either state.
2. **The fallback path still works.** A source whose lex fails still goes
   through the line-driven splitter and still evaluates correctly, now as
   Unparsed forms.
3. **Message and print ordering across forms is unchanged.** The loop
   accumulates messages and prints across forms and rolls back on error
   (`session_eval.rs:268-285`); that behavior is pinned by the loader rollback
   tests and must not move.
4. **A hard-error form still reports the same error.** Add a test that pins the
   exact message for a script whose second form has a syntax error, so a future
   change to the two-state handling cannot silently alter diagnostics.

## Deliverable

- `parse_script_forms` returning forms that carry their syntax.
- The two-state form threaded through `split_script_into_forms` into the script
  loop, with evaluation lowering Parsed forms directly.
- The differential test adapted and still passing.
- The pinned hard-error-message test from constraint 4.
- Before/after timings (below).

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
n = 10000
open("/tmp/b308-stmts.rum", "w").write(
    "\n".join("v%d = %d;" % (i, i) for i in range(1, n + 1)) + "\nPrint[\"last=\", v%d];\n" % n)
body = ",\n".join("  %d" % i for i in range(1, n + 1))
open("/tmp/b308-data.rum", "w").write(
    "data = {\n%s\n};\nPrint[\"len=\", Length[data]];\n" % body)
PY
for f in stmts data; do
  echo "--- $f ---"
  ./target/release/rum-cli /tmp/b308-$f.rum < /dev/null   # warm
  /usr/bin/time -p ./target/release/rum-cli /tmp/b308-$f.rum < /dev/null
done
```

Reference numbers on the current tree, measured by the director: the
10 000-statement script runs in **0.04 s**, the 10 000-line data literal in
**0.01 s**. Both should improve, the statement script most visibly, since it has
10 000 forms each currently parsed twice. Report both, and say plainly if a
number does not move — a mechanism that does not show up in the timing is a
finding, not something to explain away.

No microbench A/B: this is the script-loading path, not the evaluation hot path.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses. The differential test
  compares rum against rum and is fine.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- Minimal diff; no drive-by reformatting. In particular, do not restructure the
  script loop beyond what carrying the syntax requires.
- **Stop-loss is real.** If threading the two-state form forces a change to
  `eval_text`'s signature or to message/print ordering, STOP and report rather
  than reshaping the contract.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, with line ranges, and how the two states flow.
2. Verification — each command with verbatim output: test counts, the
   differential corpus still passing, the pinned error message, and the
   before/after timings.
3. The verdict — how many parses were removed per script, and what the timing
   shows.
4. Stop-loss / open questions.
