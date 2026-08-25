# brief-330 — plan the left-associative lowering fix (DESIGN ONLY)

**Scope: NONE — no source change.** Produce a plan and draft the implementation
briefs. Write no code.

## The defect

`lower_syntax_expr_with` (`crates/rum-runtime/src/runtime/lowering.rs:9`)
descends recursively. For a left-associative operator the parser builds
`((a op b) op c) op d`, so the depth sits in `lhs`, and
`lower_binary_sugar` (`:273`) reaches it through
`lower_syntax_expr_with(lhs, …)` at `:291`. One native frame per link.

The parser's own guard does not cover this. `MAX_OPERATOR_CHAIN_DEPTH = 128`
(`crates/rum-syntax/src/parser/front.rs:20`) counts recursive descent, and a
left-associative chain is parsed **iteratively** — the counter never moves. The
depth is real but deferred into the AST, and lowering pays it.

## Survey (measured by the director — do not re-derive)

All 47 `BinaryOp` variants, one 20 000-link chain each, `Hold`-wrapped so only
parse and lowering run, release CLI:

**Crashes — stack overflow, exit -6 (16):**
`+` `-` `*` `/` `.` `**` `<>` `::` `/.` `//.` `:` `|` `?` `~~` `//` `/*`

**Refused by the parser guard, exit 1 (24):**
`;` `/:` `=` `^=` `+=` `-=` `*=` `/=` `//=` `:=` `^:=` `->` `:>` `/;` `||` `&&`
`@` `/@` `//@` `@@` `@@@` `@*` `^` `|->`

**Decoded fine (8):** the comparison operators
`===` `=!=` `==` `!=` `<` `>` `<=` `>=`

The split is exactly left- versus right-associative. The comparisons survive
because `lower_comparison_chain` (`:501`) collects the chain first and emits one
flat n-ary call.

**But that path is not a clean template**: `collect_comparison_chain` (`:558`)
recurses too. Its frames are smaller, so the threshold is merely higher — a
50 000-link `==` chain decodes, a 200 000-link chain overflows the stack. Copying
its shape inherits its defect.

## Current lowered shapes (measured)

```
Hold[1+2+3+4]      → Hold[Plus[Plus[Plus[1, 2], 3], 4]]
Hold[a*b*c*d]      → Hold[Times[Times[Times[a, b], c], d]]
Hold["a"<>"b"<>"c"]→ Hold[StringJoin[StringJoin["a", "b"], "c"]]
Hold[a-b-c]        → Hold[Plus[Plus[a, Times[-1, b]], Times[-1, c]]]
Hold[a/b/c]        → Hold[Times[Times[a, Power[b, -1]], Power[c, -1]]]
Hold[a+b-c+d]      → Hold[Plus[Plus[Plus[a, b], Times[-1, c]], d]]
Hold[a==b==c]      → Hold[Equal[a, b, c]]          ← already flat
```

Nothing in the tree asserts the nested spelling: `Plus[Plus`, `Times[Times` and
`StringJoin[StringJoin` have zero occurrences across `crates/` and `docs/`. The
golden file `crates/rum-runtime/src/runtime/tests/lowering_golden.rs` does pin
`("a . b . c", "Dot[Dot[a, b], c]")`, so at least one shape decision is
already recorded there.

## What the plan must decide

1. **Per operator, flatten or keep nested.** Some of the sixteen have n-ary
   heads where `op(op(a,b),c)` and `op(a,b,c)` mean the same thing. Others are
   binary by nature — `a /. b /. c` applies two replacements in sequence and
   `a // f // g` is `g[f[a]]`; flattening those changes what they compute.
   Classify all sixteen, with the reasoning per operator, and mark which
   classifications you are confident about versus which need a ruling.

   Decide this from rum's own semantics — its evaluator, its attributes, its
   documented behavior, and the shapes already pinned in the golden file. **Do
   not consult another vendor's engine or documentation**; the testing policy in
   `AGENTS.md` is a hard line here.

2. **The iteration shape, for both classes.** Flattening is not a way to avoid
   iterating: the collection walk must itself be a loop, or it reproduces
   `collect_comparison_chain`'s ceiling one level up. For the operators that
   must stay nested, the rebuild is a fold from the innermost end — also a loop.
   Propose one structure that serves both, or say why two are needed.

3. **`Subtract` and `Divide` fold into other heads.** `a-b` already lowers
   through `Plus`/`Times` with a negated or reciprocal operand, so a mixed chain
   `a+b-c+d` is one `Plus` chain. State how mixed chains are collected — this is
   where an operator-at-a-time design tends to break.

4. **The comparison path.** `collect_comparison_chain` has the same defect at a
   higher threshold. Decide whether it joins this work or becomes its own step,
   and say why.

5. **Whether a limit is still wanted.** After the recursion is gone, is there
   any remaining per-chain ceiling, and should there be one? A chain of ten
   million links is still an allocation. Recommend, with a number and a reason.

6. **Blast radius.** Which tests, goldens, and docs change if shapes change;
   whether the formatter round-trip is affected (it consumes `SyntaxExpr`, not
   the lowered `Expr` — confirm rather than assume); and whether pattern
   matching against held expressions can observe the difference.

## Evidence standard

Every claim gets `path:line`. Where you assert an operator's arity semantics,
cite the evaluator or attribute table that establishes it, not a general
impression.

## Deliverable

1. A step-by-step plan: which files and functions change per step, what each
   step's gate test is, and the order.
2. The sixteen-operator classification table, with per-operator reasoning and a
   confidence mark.
3. Answers to the six questions above, and any open question you cannot settle
   from the repository — flag those for the director rather than guessing.
4. **Draft implementation briefs in `TEMPLATE.md` shape**, one per step. These
   are raw material for the director's own briefs, so make them complete:
   Scope with a MAY-modify list, hard rules, verification commands, and a report
   format.

Steps should be independently gateable. If the classification splits the
operators into groups that want different treatment, let the step boundaries
follow that split rather than the alphabet.

## Hard rules

- **No source changes.** No edits to `crates/`. Writing a new file under
  `docs/dev/codex/` for your draft briefs is expected; nothing else.
- Do NOT `git commit`, branch, push, or touch git config.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only. No other vendors' engines, no oracle harnesses, no compatibility claims.
- Reading and probing with the existing release CLI is fine and encouraged —
  feed stdin from `/dev/null`, it waits for input otherwise. Say which probes
  you ran and paste what they returned.
- If a question in this brief rests on a false premise, say so and correct it.
  The survey above is measured, but its interpretation is mine and may be wrong.

## Report format (your final message)

1. The classification table.
2. The plan, step by step.
3. Answers to the six questions; open questions called out separately.
4. Where the draft briefs were written.
