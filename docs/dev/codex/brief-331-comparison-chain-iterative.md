# brief-331 — make comparison-chain lowering iterative (plan-330 step 1)

First of five steps from `docs/dev/codex/plan-330-left-chain-lowering.md`.
**Nothing in this step may change what lowering outputs.** The sixteen
operators keep their current shapes; only the traversal changes.

## The defect

`collect_comparison_chain` (`crates/rum-runtime/src/runtime/lowering.rs:558`)
recurses into both `lhs` and `rhs` of every comparison link. It looks safe
because comparison chains already lower to one flat n-ary call, but the
collection walk itself is recursive — it merely has smaller frames than the
lowering it protects against.

Measured today (release CLI, `Hold`-wrapped, verified by the director — do not
re-derive):

```
Hold[1 == 1 == ... ]   50 000 links → decodes, exit 0
Hold[1 == 1 == ... ]  200 000 links → thread 'main' has overflowed its stack, exit -6
```

That threshold is the reason this step comes first: every later step in
plan-330 routes more operators through chain collection, and a recursive
collector would hand the new code the same ceiling one level up.

## What to build

Replace the recursion with an explicit work stack. The plan's shape: seed a
LIFO stack with `Visit(rhs)`, `EmitOp(root_op)`, `Visit(lhs)`; a comparison
`Visit` pushes its rhs, an operator marker, and its lhs in reverse execution
order; a leaf appends one operand. That reproduces today's in-order operand and
operator vectors with no native recursive call.

Take the shape as a starting point, not a specification — if the in-order
guarantee is cleaner another way, say so and build that instead.

Also split the binary lowering out of `lowering.rs`. It is 22 671 bytes, inside
the 20k–35k band where `docs/code-organization-guidelines.md:70-81` requires a
split assessment when the file is touched. Binary lowering versus the rest is a
real responsibility boundary; a comparison submodule under it is a second.
Report the assessment you make and the boundary you chose.

## What must not change

- **Byte-identical output for every input.** Uniform chains (`a<b<c`), mixed
  chains (`a<b<=c`, which become `And[...]`), single comparisons, and chains
  with parenthesized left or right subtrees.
- **Operand and operator order**, which is what the flat n-ary and the `And`
  fallback both depend on.
- **Symbol resolver call count and order.** `lower_syntax_expr_with` takes a
  generic `FnMut` (`lowering.rs:9-15`), so how many times it is called and in
  what order is part of the contract, not an implementation detail.
- Every other operator's lowering. This step touches comparisons only.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/lowering.rs`
- new modules under `crates/rum-runtime/src/runtime/lowering/`
- `crates/rum-runtime/src/runtime/tests/lowering_golden.rs` — additions only
- new test modules under `crates/rum-runtime/src/runtime/tests/`, plus the
  `mod.rs` line registering them
- `crates/rum-runtime/src/runtime/mod.rs` if the module split requires it

Out of scope: the parser, the formatter, `SyntaxExpr` itself, and the fifteen
non-comparison operators.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Comparison collection iterative, output unchanged.
- The binary/comparison module split, with your assessment.
- A resolver call-count and call-order test.
- A long-chain test. **Inspect the result without `to_string`** — `Display` for
  a deep expression is itself recursive, so a rendering-based assertion tests
  the wrong thing and may crash on a shape that is actually correct.
- Existing comparison goldens unchanged; new goldens for parenthesized-subtree
  cases.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY' > /tmp/b331-1.rum
print('Print[Head[Hold[' + '=='.join(['1']*400000) + ']]];')
PY
python3 - <<'PY' > /tmp/b331-2.rum
print('Print[Head[Hold[' + ' < '.join(['a']*200000) + ']]];')
PY
cat > /tmp/b331-3.rum <<'EOF'
Print[FullForm[Hold[a == b == c]]];
Print[FullForm[Hold[a < b <= c]]];
Print[FullForm[Hold[(a == b) == c]]];
Print[FullForm[Hold[a == (b == c)]]];
EOF
for i in 1 2 3; do echo "--- $i ---"; ./target/release/rum-cli /tmp/b331-$i.rum < /dev/null; echo "exit=$?"; done
```

The first two must exit 0. **An exit code of -6 or 134 is a failure of this
brief.** The third must print exactly what it prints today — capture that
before you change anything and diff it after.

If the first two reach a *different* limit after your change — `SyntaxExpr`
destruction, or rendering — report which one, with evidence. Do not
misattribute it to the collector you just fixed, and do not raise the length
until it passes.

No microbench A/B: lowering runs once per parsed form, not per evaluation
step. Say so if you believe otherwise.

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
- Do not change `SyntaxExpr`'s representation or its drop behavior. That is a
  separate decision the director has not made.
- **This step is output-preserving.** If you find a bug in the current
  comparison lowering, report it — do not fix it here. A behavior fix folded
  into a traversal change cannot be reviewed as either.
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — the stack structure, the module boundary, with line ranges.
2. How you established output is byte-identical, including the resolver
   contract.
3. Verification — each command with verbatim output and exit codes.
4. The organization assessment.
5. Stop-loss / open questions, including any current-behavior bug you found and
   deliberately did not fix.
