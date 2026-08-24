# brief-317 — ReplacePart in one pass

## Goal

`ReplacePart` walks the whole tree once **per rule**, so it is O(rules × tree),
and it recurses once per rule, so it aborts the process on a wide rule list.
Fix both, with its observable results — including the conflict and overlap rules
— unchanged.

## Measured today (release CLI, verified by the director — do not re-derive)

`lst = Range[n]; rules = Table[{i} -> 0, {i, 1, n}]; ReplacePart[lst, rules]`:

| n rules (= tree size) | time |
|---|---|
| 500 | 0.0020 s |
| 1000 | 0.0061 s |
| 2000 | **0.0175 s** |
| 20000 | **`exit=-6`, stack overflow** |

2.9–3.1× per doubling, and the crash at 20 000 is new information — the survey
recorded the quadratic but not the abort.

## The mechanism

`crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:49-69`
(`apply_replace_part_rules_owned`) processes one rule at a time:

1. `resolve_rule_targets_owned` collects that rule's targets by walking the
   whole expression;
2. `build_replace_plan_from_targets` + `apply_replace_plan_owned` rebuild the
   whole expression;
3. the `drive_internal_value` continuation calls the function again for
   `rule_index + 1`, which is the per-rule native stack frame.

## The semantics that make this harder than it looks

Rules are applied **sequentially, each to the result of the last**. Measured:

| call | result |
|---|---|
| `ReplacePart[{1,2,3}, {{1} -> "a", {1} -> "b"}]` | `{"b", 2, 3}` — the later rule wins |
| `ReplacePart[{{1,2},{3,4}}, {{1} -> x, {1,2} -> y}]` | `{x, {3,4}}` — `{1,2}` no longer exists after `{1}` was replaced, and is skipped |
| `ReplacePart[{1,2}, {{5} -> z}]` | `{1,2}` — an out-of-range position is skipped silently |

**The second row is the trap.** Merging every rule's targets into one plan
collected against the *original* tree would let `{1,2}` match something that
sequential application has already destroyed. A naive "collect all targets, then
rebuild once" is therefore **not** equivalent, and the difference is silent.

So the plan must either:

- prove that merging preserves sequential semantics — including that a later
  rule sees the earlier rules' effects, and that a later rule targeting a
  position destroyed by an earlier one still drops out; or
- keep the sequential structure and remove the cost some other way.

`structure_data/replace/plan.rs` already resolves conflicts inside a plan trie
(that is what `b443185` did), so part of the machinery exists. Whether it
extends to *cross-rule* sequencing is the question, not the assumption.

## Acceptable outcomes, in order of preference

1. **One pass, proven equivalent.** Best, if the argument holds.
2. **Iterative but still per-rule.** If merging cannot be shown equivalent,
   converting the recursion to a loop still removes the crash — a real fix on
   its own, since a 20 000-rule call currently kills the process. Report the
   quadratic as remaining, with evidence for why merging was rejected.
3. **Stop and report.** If neither is reachable without changing results.

Do not trade semantics for speed. A result that differs on any shape in the
table above is a defect, not a tradeoff.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/plan.rs` —
  only if the merge genuinely needs it; say why
- test modules under `crates/rum-runtime/src/runtime/tests/`

Out of scope: everything else, including `Replace`/`ReplaceAll`/`ReplaceList`
and the `rum-stdlib` structure ops.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- The crash gone: a 20 000-rule call must exit 0 with the right result.
- Whichever of outcomes 1 or 2 you reach, with the equivalence argument or the
  rejection argument spelled out.
- Tests pinning the three behaviours in the table above, plus a wide case.
- Before/after scaling at n = 500 / 1000 / 2000, and at 20 000 if outcome 1.
- Suspension safety preserved: rules can run user code, so a suspended
  `ReplacePart` must resume at the same rule with the same partial result. Add a
  mid-walk suspension test, as brief-304 and brief-309 did.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (500, 1000, 2000, 20000):
    open("/tmp/b317-%d.rum" % n, "w").write(
        "lst = Range[%d];\nrules = Table[{i} -> 0, {i, 1, %d}];\n"
        'Print["n=%d t=", AbsoluteTiming[Total[ReplacePart[lst, rules]]]];\n'
        % (n, n, n))
PY
for n in 500 1000 2000 20000; do echo "--- $n ---"; ./target/release/rum-cli /tmp/b317-$n.rum < /dev/null; echo "exit=$?"; done
```

All must exit 0 with a total of 0. **An exit code of -6 / 134 is a failure of
this brief.**

`ReplacePart` sits on the evaluation path, so run the microbench A/B and paste
the replacepart labels plus same-tier neighbors, with a plain statement of
whether the machine was quiet.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- **Do not trade a crash for an O(n²), or semantics for either.**
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges, and which outcome you reached.
2. The equivalence argument (outcome 1) or the rejection argument (outcome 2),
   with the shapes you tested it against.
3. Verification — each command with verbatim output and exit codes, the pinned
   behaviours, the scaling table, and the A/B rows.
4. Stop-loss / open questions.
