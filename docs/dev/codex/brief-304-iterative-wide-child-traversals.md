# brief-304 — make wide child traversals iterative (crash fix)

## Goal

`Replace`, `Cases`, `Count`, `Position`, and `DeleteCases` abort the process
with a native stack overflow on an ordinary wide flat list. Each walks its
children by calling itself from inside a `drive_internal_value` continuation, so
the native stack depth equals the element count. When you are done these five —
plus `ReplaceAll`, whose threshold is merely higher — walk their children in an
explicit loop, and a 50 000-element call succeeds.

This is a crash fix, not an optimization. Do not take the opportunity to change
matching, rule order, or message behavior.

## Measured today (release CLI, verified by the director — do not re-derive)

| call | result |
|---|---|
| `Replace[Range[10000], z -> 0]` | `exit=-6`, `thread 'main' has overflowed its stack` |
| `Cases[Range[10000], _Integer]` | `exit=-6` |
| `Count[Range[10000], _Integer]` | `exit=-6` |
| `Position[Range[10000], 5]` | `exit=-6` |
| `DeleteCases[Range[10000], 5]` | `exit=-6` |
| `Range[50000] /. z -> 0` | `exit=-6` (10 000 still succeeds) |
| the same calls at 5 000 | succeed |

The defect predates this batch: `origin/main` aborts identically. It is not a
regression, it is a shipped bug.

**Why the existing guard does not catch it**: the native-stack guard polls only
in `evaluate` (`crates/rum-runtime/src/runtime/eval_core/mod.rs:41-45`), keyed on
`evaluation_depth`. These continuation chains never increment
`evaluation_depth`, so the guard never fires. Raising a limit is therefore not
the fix, and adding a poll inside `drive_internal_value` is **not** a substitute
either — that would merely convert a crash into a refusal, when the whole point
is that a 50 000-element list is a perfectly reasonable input. Make the walk
iterative.

## The shape to copy

`crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:69-107`
is the in-repo template, already doing exactly this for `Do`:

```rust
loop {
    let outcome = /* one step */;
    match outcome {
        Ok(_) | Err(EvalError::Continue) => { iteration += 1; }      // loop, no recursion
        Err(EvalError::Break) => return Ok(system_expr("Null")),
        Err(EvalError::InputRequired { request }) => {
            self.push_suspension_frame(move |session, outcome| {
                session.resume_do_after_iteration(args, spec_index, spec, iteration, outcome)
            });
            return Err(EvalError::InputRequired { request });
        }
        Err(error) => return Err(error),
    }
}
```

The load-bearing part is the suspension arm: state (including the element index
and everything accumulated so far) is moved into a suspension frame, and the
paired `resume_*` re-enters the loop at that index. Preserve that property at
every site — a suspended traversal must resume at the same element with the same
accumulated output, exactly as today.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs`
- `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs`
- `crates/rum-runtime/src/runtime/tree_walkers/position.rs`
- the test modules under `crates/rum-runtime/src/runtime/tests/` needed for the
  new tests

Everything else is out of scope. Deliberately **not** in this brief, even though
they share the defect — they are the next batch, and mixing them in would make
this diff unreviewable:

- `kernel_data/rewrite_data/replace/list.rs` (`ReplaceList`, recurses per RULE —
  a different axis; measured to abort at 10 000 rules)
- `debug_tools/reap.rs` (bucket rendering)
- `definitions/application/{down,sub,up}_values.rs`
- everything in `rum-stdlib` (`StringApply`, the `StringReplace` list walk)

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Each of the six walkers converted from continuation self-recursion to an
  explicit index/worklist loop, with suspension and resume preserved.
- Per builtin, a test asserting a 30 000-element call returns the right result.
  30 000 is well past every measured abort threshold while keeping the debug
  test suite tolerable; report each new test's runtime, and if any single test
  exceeds ~10 s in debug, say so rather than silently trimming the size.
- The A/B rows described below.

## rum / Rust pitfalls (read twice)

1. **Suspension safety is the hard part.** These are `eval_*`/`resume_*` paths.
   Do not collapse a `suspend_with_frame*` into an eager call; do not drop the
   typed continuation state. If a site's suspension shape resists the loop
   conversion, STOP and report that site rather than weakening it.
2. **Accumulated state must move into the loop, not be rebuilt.** The point is
   to remove stack frames, not to add a per-element copy of the output vector —
   that would trade a crash for an O(n²).
3. **Head handling and level specs are unchanged.** Several of these walkers
   treat the head specially and thread a level spec through. Keep both exactly
   as they are; this brief changes control flow only.
4. **Hot-path change ⇒ microbench A/B is a hard gate**, per `AGENTS.md`. Run
   `scripts/core-language-microbench.sh` on base and on your change, and paste
   the per-label rows for the affected labels (`Replace`, `ReplaceAll`, `Cases`,
   `Count`, `Position` families) **and their same-tier neighbors**. Profiling env
   vars OFF. If another CPU-heavy workload is running, say so and do not report
   timings — counters first, timings only if the machine is quiet.
5. **One variable per measurement.** Convert, test, measure — do not tune while
   measuring.
6. **Stop-loss is real.** If the same site fails twice for the same reason, stop
   and report with the partial work left in the tree.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli`, run each of these and
paste the verbatim output **and exit code**. Feed stdin from `/dev/null` — the
CLI waits for input otherwise:

```
for f in replace cases count position deletecases replaceall; do :; done
# Replace
echo 'Print[Head[Replace[Range[50000], z -> 0]]];'            > /tmp/b304-1.rum
echo 'Print[Length[Cases[Range[50000], _Integer]]];'          > /tmp/b304-2.rum
echo 'Print[Count[Range[50000], _Integer]];'                  > /tmp/b304-3.rum
echo 'Print[Length[Position[Range[50000], 5]]];'              > /tmp/b304-4.rum
echo 'Print[Length[DeleteCases[Range[50000], 5]]];'           > /tmp/b304-5.rum
echo 'Print[Head[Range[50000] /. z -> 0]];'                   > /tmp/b304-6.rum
for i in 1 2 3 4 5 6; do
  echo "--- $i ---"; ./target/release/rum-cli /tmp/b304-$i.rum < /dev/null; echo "exit=$?"
done
```

Every one must exit 0 with the correct value. **An exit code of -6 / 134
anywhere is a failure of this brief.**

Then the A/B rows from pitfall 4.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- Minimal diff; no drive-by reformatting or renaming.
- Paste real verbatim output. If a run did not reach green, say so explicitly.

## Report format (your final message)

1. What changed — per file and per walker, with line ranges and how the
   suspension state is now carried.
2. Verification — each command with exact verbatim output and exit codes,
   including the six 50 000-element probes and the new tests' runtimes.
3. The A/B rows — affected labels plus same-tier neighbors, with a plain
   statement of whether the machine was quiet.
4. Stop-loss / open questions — any site whose suspension shape resisted the
   conversion (empty if none).
