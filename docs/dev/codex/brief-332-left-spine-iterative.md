# brief-332 — one iterative left spine for all sixteen operators (plan-330 step 2)

Second of five steps from `docs/dev/codex/plan-330-left-chain-lowering.md`.
Step 1 landed in `70af8378`.

**This step changes no output.** All sixteen operators keep emitting exactly
the binary shapes they emit today, including the nine that step 3 and step 4
will later flatten. Only the traversal becomes iterative.

## The defect

`lower_binary_sugar` reaches a left chain's depth through
`lower_syntax_expr_with(lhs, …)`. For `((a op b) op c) op d` that is one native
frame per link, and the parser's `MAX_OPERATOR_CHAIN_DEPTH` never sees it: a
left-associative chain is parsed iteratively, so the counter does not move.

The sixteen affected operators, measured at 20 000 links each (director's
survey, do not re-derive):

```
+  -  *  /  .  **  <>  ::  /.  //.  :  |  ?  ~~  //  /*
```

All sixteen overflow the stack today. The other 32 `BinaryOp` variants are
already safe — 24 are right-associative and the parser guard refuses them, 8 are
comparisons and step 1 made their collection iterative.

## What to build

Per the plan: a `left_chain.rs` under `lowering/binary/`, a borrowed link record
`(BinaryOp, &SyntaxExpr)`, a predicate covering the sixteen, and a
`collect_left_spine` loop. Seed with the outer `(op, rhs)`, walk down the lhs
pushing `(op, rhs)` while it is any affected operator, reverse once. Then lower
the innermost base expression and fold the links in source order into an `Expr`
accumulator, applying each link's current transformation.

**Do not stop collecting at an operator-family transition.** Beyond covering
ordinary mixed source like `a+b-c+d`, this is what stops a hand-built
`SyntaxExpr` from alternating unrelated affected operators to rebuild recursive
depth.

## Four places this goes wrong

Each is a case where "lower the rhs, apply the operator" is not what the
current code does:

1. **`MessageName`** lowers its rhs, and then for a non-symbol lhs with a
   non-symbol/string tag, `lower_message_tag_expr` lowers that same raw rhs a
   *second* time (`lowering.rs:290-305,579-590` before the step-1 move). Keep
   that call count and order. It looks like a redundancy worth cleaning up; it
   is not this step's business.
2. **`:`** picks its head from the accumulated lhs, not from the source: a
   symbol yields `Pattern`, anything else yields `Optional`. The fold must
   consult the accumulator at each link.
3. **`//`** inverts — the link is `rhs[accumulator]`, not `accumulator op rhs`.
4. **`/`** has a fast path that folds two exact numeric literals immediately,
   distinct from the general reciprocal-multiplication form. A chain builder
   that only knows the general form will silently change literal quotients.

## What must not change

- **Byte-identical output for every input**, short chains included.
- **Symbol resolver call count and order.** `lower_syntax_expr_with` takes a
  generic `FnMut`, so both are observable through it — the `MessageName` double
  call included.
- Comparison lowering, the parser, the formatter, `SyntaxExpr` itself.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/lowering/binary.rs`
- new modules under `crates/rum-runtime/src/runtime/lowering/binary/`
- `crates/rum-runtime/src/runtime/lowering.rs` — only if the dispatch hook needs it
- `crates/rum-runtime/src/runtime/tests/lowering_golden.rs` — additions only
- new test modules under `crates/rum-runtime/src/runtime/tests/`, plus the
  `mod.rs` line registering them

Out of scope: everything else. Do not add unconditional cost to the default
evaluation path.

## Deliverable

- All sixteen folded iteratively, output unchanged.
- Exact three-link goldens for all sixteen.
- A cross-family chain, and a hand-built `SyntaxExpr` alternating affected
  operators (the adversarial shape the collection rule exists for).
- Resolver call-count and call-order tests, including the `MessageName` case.
- Long-chain tests. **Inspect results without `to_string`** — `Display` is
  recursive, so a rendering-based assertion tests the wrong thing and can abort
  on a shape that is correct.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise. For each of the sixteen
operators, at 20 000 links:

```
Print[Head[Hold[ a <op> a <op> ... ]]];
```

**The gate is that `Hold` prints.** That is the proof lowering ran to
completion, and it is what this step is responsible for.

An abort *after* `Hold` prints is a different limit and does not fail this
brief — but say which one it is, with evidence. Step 1 hit exactly this:
lowering completed, and the abort moved to `SyntaxExpr`'s recursive drop glue.
This step will additionally produce deeply nested `Expr` values, whose own drop
may recurse; that threshold is currently unmeasured, because every other way of
building one is refused by the delimiter guard. If you find it, report the
number — it is useful input for step 5's ceiling.

Do not lower a chain length to make a probe pass, and do not credit another
limit's abort to the fold you just wrote.

No microbench A/B: lowering runs once per parsed form, not per evaluation step.
Say so if you believe otherwise.

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
- **This step is output-preserving.** If you find a bug in current lowering,
  report it — do not fix it here.
- **If you record current behavior in a golden and that behavior looks wrong,
  say so in the report and annotate the row.** A defect written into a golden
  without comment becomes an expectation, and the next person to touch it will
  defend it. Step 1 added four parenthesized comparison rows that encode a real
  defect — `1 == (1 == 1)` is True while `1 == True` stays unevaluated — and
  reported them as clean; the annotation had to be added at the gate.
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — the collection and fold structure, with line ranges, and how
   each of the four special cases is handled.
2. How you established output is byte-identical, including the resolver
   contract.
3. Verification — the sixteen-operator probe table with exit codes and whether
   `Hold` printed; any later limit you reached, named and evidenced.
4. Stop-loss / open questions, including any current-behavior bug you found and
   deliberately did not fix.
