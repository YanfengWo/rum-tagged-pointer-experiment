# brief-323 — share the compiled rule plan across a Replace traversal

## Goal

`ReplaceAll` compiles its rules once per traversal and can index exact rules.
`Replace` and `ReplaceAt` hand a freshly cloned rule vector to every visited
node, so each node pays for the clone and a linear scan. Share one compiled plan
across the traversal so cost stops scaling with the rule count at every node.

This is the last item from the original P1 audit.

## Measured today (release CLI, verified by the director — do not re-derive)

`Replace[Range[500], rules, {1}]` with `rules = Table[i -> 0, {i, 1, r}]`:

| rules | time |
|---|---|
| 50 | 0.0026 s |
| 100 | 0.0046 s |
| 200 | **0.0093 s** |

Linear in the rule count at a fixed tree size — i.e. O(nodes × rules) overall.

## The mechanism, and the comment that documents it

`Replace` (`kernel_data/rewrite_data/replace/once.rs:165`) and `ReplaceAt`
(`structure_data/replace/replace_at.rs:111`) both call
`try_apply_rewrite_rules_once_owned` with `rules.clone()` per target.

That helper
(`definitions/application/replace_all.rs:61-74`) then does:

```rust
// Replace and ReplaceAt currently hand this helper a fresh rule Vec for
// each target. Building an index here would merely replace one linear
// scan with one linear index build; ReplaceAll shares its plan across
// the whole traversal and is the indexed caller.
let rules = RewriteRulePlan::linear(rules);
```

The comment is correct about today's callers, and it names exactly what has to
change: **the callers must stop handing over a fresh vector**. `ReplaceAll`
already shows the shape — `RewriteRulePlan::from_slice(rules)` built once in
`apply_replace_all` (`:57`) and carried through the traversal.

Update that comment as part of the change; leaving it describing a state that no
longer holds is worse than having no comment.

## What must not change

- **Rule order.** The first matching rule wins; that is observable.
- **Held evaluation and message behavior** on the rule right-hand sides.
- **Suspension shape.** Rules can run user code, so a suspended traversal must
  resume at the same node with the same accumulated state. Whatever carries the
  plan must survive that — an `Arc` is the obvious candidate, but say what you
  chose and why.
- `ReplaceList` shares the helper too. Check whether it has the same per-target
  clone; if it does and the fix is the same shape, include it and say so, but do
  not restructure its per-rule walk — brief-317 settled that.

## A second, optional part — measure before deciding

`once.rs:159-163` computes subtree depth on **every** visited node:

```rust
let depth = if include_heads { replace_expr_depth_with_heads(&expr) }
            else { replace_expr_depth(&expr) };
```

but `CasesLevelSpec::matches` only consults `depth` when the spec's minimum is
negative. Tree depth is bounded by `DEFAULT_EVALUATION_PASS_LIMIT` (256), so
this is a **bounded constant factor, not a complexity reduction** — the original
audit item conflated the two, and `TODO.md` now records the distinction.

Whether it is worth doing is a measurement question, not a judgement call:
compute depth only when the level spec can consult it, measure the difference on
a non-negative spec, and report the number. If it does not move, say so and
leave the code alone rather than landing a change that buys nothing.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs`
- `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs` — the
  helper's signature and its comment
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs` —
  only if `ReplaceList` has the same per-target clone
- test modules under `crates/rum-runtime/src/runtime/tests/`

Out of scope: everything else, including `ReplaceAll`'s own traversal (already
correct), the matcher, and `rum-stdlib`.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- One compiled plan per traversal, shared across targets, with the exact-rule
  index available to `Replace`/`ReplaceAt` as it already is to `ReplaceAll`.
- The stale comment updated.
- Before/after timings at 50 / 100 / 200 / 400 rules.
- The depth measurement and your verdict on it.
- Tests pinning rule order and a suspension mid-traversal that resumes at the
  same node.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for r in (50, 100, 200, 400):
    open("/tmp/b323-%d.rum" % r, "w").write(
        "expr = Range[500];\nrules = Table[i -> 0, {i, 1, %d}];\n"
        'Print["rules=%d t=", AbsoluteTiming[Head[Replace[expr, rules, {1}]]]];\n' % (r, r))
PY
for r in 50 100 200 400; do ./target/release/rum-cli /tmp/b323-$r.rum < /dev/null; done
```

Time must stop scaling linearly with the rule count.

**This is the replacement hot path, so the microbench A/B is a hard gate.** Run
`scripts/core-language-microbench.sh` on base and candidate and paste the
replace/replaceall/replacepart labels plus same-tier neighbors. A regression on
any of them is a defect to fix, not to explain. State plainly whether the
machine was quiet.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If sharing the plan cannot preserve the suspension
  shape, STOP and report rather than weakening it.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges, and what carries the plan
   across suspension.
2. Verification — each command with verbatim output: test counts, the rule-count
   scaling, and the A/B rows with neighbors.
3. The depth measurement and verdict.
4. Stop-loss / open questions.
