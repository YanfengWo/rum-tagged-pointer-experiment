# brief-333 — flatten the arithmetic and composition chains (plan-330 step 3)

Third of five steps from `docs/dev/codex/plan-330-left-chain-lowering.md`.
Steps 1 and 2 landed in `70af8378` and `62839ffa`.

**This is the first step that changes output.** Five tokens in three families
stop emitting nested binary calls and emit one n-ary call instead:

- additive: `+` and `-` → one `Plus`
- multiplicative: `*` and `/` → one `Times`
- composition: `/*` → one `RightComposition`

The other eleven operators keep the shapes step 2 preserved.

## Why these five are safe to flatten

`Plus`, `Times` and `RightComposition` are all attributed `Flat`, so
`op(op(a,b),c)` and `op(a,b,c)` are the same expression to the evaluator. That
is the whole licence for this step — it does not extend to the eleven others,
and it is not a reason to add `Flat` to anything.

## Current behavior (measured by the director — do not re-derive)

Shapes, `HoldComplete`-wrapped:

```
Plus[a,b] + c                 → Plus[Plus[a, b], c]
a + Plus[b,c]                 → Plus[a, Plus[b, c]]
Times[a,b] * c                → Times[Times[a, b], c]
a * Times[b,c]                → Times[a, Times[b, c]]
RightComposition[f,g] /* h    → RightComposition[RightComposition[f, g], h]

8/2/4                         → 1
8*2/4                         → Times[Times[8, 2], Power[4, -1]]
8/2*x                         → Times[4, x]
a/2/4                         → Times[Times[a, Power[2, -1]], Power[4, -1]]
```

Evaluated results, which must be identical after the change:

```
1+2+3+4 = 10     2*3*4 = 24      10-3-2 = 5
100/5/2 = 10     1+2-3+4-5 = -1  (f /* g)[x] = g[f[x]]
```

## Two things that are easy to get wrong

**1. Direct canonical calls stay opaque.** `docs/lowering-canonical-forms.md`
states the rule: "direct calls that already use canonical runtime heads are
preserved". So `Plus[a,b] + c` must remain `Plus[Plus[a, b], c]` — the inner
`Plus` was written by the user and is not a chain link.

This falls out of the collection rule for free: the lhs of that chain is a
`Call`, not an affected `Binary`, so collection stops and it becomes the base.
It only breaks if the builder *inspects* the base and absorbs a matching head.
Do not do that.

**2. The exact-literal quotient rule is narrower than it looks.** The four
measured lines above are the specification. `8/2/4` folds all the way to `1`;
`8*2/4` does not fold at all; `8/2*x` folds the quotient then multiplies.

The rule fires when the current accumulator and the rhs are both numeric atoms.
Once the accumulator becomes a chain-owned factor vector, it is no longer a
numeric atom, so the rule stops firing — which is why `8*2/4` will legitimately
change shape to `Times[8, 2, Power[4, -1]]` while still evaluating to 4. State
every shape change of this kind; do not quietly preserve or quietly drop the
fast path.

**Family transitions materialize.** Consecutive links in the active family
append to its vector; a different family first materializes that vector into
one `Expr`, then applies the next link. `a*b/c*d` is one `Times`; `a+b*c+d` is
not one of anything — the `*` is a separate expression inside a `Plus`.

## What must not change

- **Every evaluated result.** This is the hard gate. `Plus` being `Flat` means
  the flattening is invisible after evaluation; if any evaluated value moves,
  the change is wrong.
- The eleven operators step 2 left binary.
- Direct canonical-call opacity, above.
- `docs/reference/builtins.md` — no catalog spec changes here, so the generated
  reference must come out byte-identical. If it does not, stop and report.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs`
- `crates/rum-runtime/src/runtime/lowering/binary.rs` — only if dispatch needs it
- `crates/rum-runtime/src/runtime/tests/lowering_golden.rs`
- `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs`
- new test modules under `crates/rum-runtime/src/runtime/tests/`, plus the
  `mod.rs` line registering them
- `docs/lowering-canonical-forms.md`
- `docs/guide/syntax.md`

Both docs currently describe only the binary forms (`a + b` -> `Plus[a, b]`).
Those stay true; add what a chain of three or more does.

Out of scope: everything else. Do not add unconditional cost to the default
evaluation path.

## Deliverable

- The three families flattened, the eleven others untouched.
- Goldens for homogeneous and mixed arithmetic, the four literal cases above,
  and direct-call opacity.
- A held-shape observability test (`SameQ` on held expressions distinguishes
  the shapes; make that explicit rather than incidental).
- An evaluated-result test covering the six results above and more.
- The two docs updated.
- **A complete list of shape changes**, in the report — every input whose
  `FullForm` output differs, not just the intended families.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo run -q -p rum-stdlib --bin generate-builtin-reference && git diff --stat docs/reference/builtins.md
```

The reference must show no diff.

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise.

**Capture an evaluated-results corpus before you change anything, and diff it
after.** Cover the four arithmetic operators over integers, rationals, symbols
and mixed operands, at two through six operands, plus the composition cases.
This is the gate that proves flattening did not change meaning; a shape diff
alone cannot.

Then the deep probes, at 20 000 links for all five tokens:

```
Print[Head[Hold[ a <op> a <op> ... ]]];
```

`Hold` must print, as in step 2.

No microbench A/B: lowering runs once per parsed form. Say so if you believe
otherwise.

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
- Do not add `Flat` to any head, and do not change any catalog spec.
- **If you record current behavior in a golden and that behavior looks wrong,
  say so and annotate the row.** A defect written into a golden without comment
  becomes an expectation.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If flattening a family cannot preserve evaluated
  results, stop and report that family rather than weakening the gate.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — the three builders and the family-transition state, with line
   ranges.
2. The complete shape-change list.
3. The evaluated-results diff, and how you captured the baseline.
4. Verification — commands with verbatim output, including the builtin
   reference diff and the five deep probes.
5. Stop-loss / open questions.
