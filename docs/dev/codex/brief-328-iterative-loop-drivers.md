# brief-328 — make the loop and Module-initializer drivers iterative

## Goal

`While`, `For`, `Until` and `Module` bindings with initializers each drive their
work by having one resume handler call the next directly, so every iteration or
binding costs native stack frames that no guard accounts for. An ordinary
20 000-iteration loop terminates the process. Convert all four to loops, with
suspension and resume preserved.

## Measured today (release CLI, verified by the director — do not re-derive)

```
i = 0; While[i < 5000, i++]        → 5000, exit 0
i = 0; While[i < 20000, i++]       → thread 'main' has overflowed its stack, exit -6
i = 0; While[i < 100000, i++]      → exit -6
While[True, 1]                     → exit -6 in 0.1s
Module[{x1=1, …, x4000=1}, 0]      → 0, exit 0
Module[{x1=1, …, x8000=1}, 0]      → exit -6
Module[{y1, …, y8000}, 0]          → 0, exit 0   (uninitialized bindings are already iterative)
```

A 20 000-iteration `While` is ordinary code, so this is a correctness defect
before it is anything else: legitimate programs fail, and they fail by killing
the process rather than by reporting a limit.

## Why the existing guards do not cover it

Three guards exist and all three measure something else:

- The native-stack check polls in `evaluate`, keyed on `evaluation_depth`
  (`eval_core/mod.rs:41-45`, stride at `metadata.rs:6`). A loop body's evaluation
  depth stays at 1-2 no matter how many iterations run, so the poll never fires.
- The iteration-work limit is applied where an iterator spec is compiled
  (`branching_loops/iteration/spec.rs:278,292`). `While`, `For` and `Until` have
  no iteration count to compile, so nothing is accounted.
- Interrupt polling runs every 1024 evaluations (`eval_core/mod.rs:13,84-98`),
  which is why a deadline does not help either: the process dies before the
  deadline is consulted.

So this cannot be fixed by tightening a limit. The recursion itself has to go.

## The four sites

1. **`While`** — `resume_while_test` (`branching_loops.rs:99-118`) →
   `resume_while_after_test` (`:120-146`, calls `resume_while_after_body` at
   `:144`) → `resume_while_after_body` (`:148-158`, calls `resume_while_test`
   at `:154`). Roughly three frames per iteration.
2. **`For`** — the same shape in the same file.
3. **`Until`** — the same shape; its threshold is higher, so verify it is
   actually converted rather than merely improved.
4. **`Module` with initializers** — `resume_module_initializers`
   (`scoping/lexical/module.rs:67`) returns after evaluating one initializer,
   and `resume_module_after_initializer` (`:167`) calls it again for the next
   binding. The uninitialized branch is already iterative, and its regression
   test at `tests/thread_and_dispatch.rs:69` uses 12 800 uninitialized bindings —
   which is why this path stayed green.

## The shape to copy

This conversion has been done six times in this repository. The template is a
loop carrying its index plus a trailing `pending: Option<Outcome>` parameter, so
a suspension frame re-enters the loop at the stored index with the child's
outcome: `f8d8f55`, `e0efd95`, and the tree walkers before them. Copy that
structure rather than inventing a variant.

## What must not change

- **Iteration semantics**: the test is evaluated before each body, the body's
  value is discarded, `Break` and `Continue` behave as they do now, and the loop
  value is what it is today.
- **`For`'s increment** runs after the body and before the next test.
- **`Module`**: initializer order, exactly-once evaluation, gensym lifetime and
  cleanup, and error propagation from an initializer.
- **Suspension**: all four can run user code, so a suspended loop must resume at
  the same iteration or binding with the same accumulated state. Add a test per
  site that suspends mid-loop and checks the result.
- **Messages** on every failure path.

`While[True, 1]` will still not terminate after this change — an unbounded loop
is a program that does not end, and bounding it is a separate decision the
director has not made. What must change is that it no longer kills the process:
it should run until an existing deadline, interrupt, or resource limit stops it.
Report what it does after your change.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs`
- `crates/rum-runtime/src/runtime/scoping/lexical/module.rs`
- test modules under `crates/rum-runtime/src/runtime/tests/`

Out of scope: everything else — the guards themselves, the iterator-spec path
(`Do`, `Table`, `Sum`, `Product` already compile a count), and `rum-stdlib`.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- All four drivers iterating, with suspension and resume preserved.
- Per site, a test well past the measured threshold: 100 000 iterations for the
  loops, 30 000 initialized bindings for `Module`. Report each test's debug
  runtime; say so rather than trimming if one exceeds ~10 s.
- A mid-loop suspension test per site.
- The `While[True, 1]` behavior after the change.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
cat > /tmp/b328-1.rum <<'EOF'
i = 0; While[i < 100000, i++]; Print["while: ", i];
EOF
cat > /tmp/b328-2.rum <<'EOF'
For[i = 0, i < 100000, i++, x = 1]; Print["for: ", i];
EOF
cat > /tmp/b328-3.rum <<'EOF'
i = 0; Until[i >= 100000, i++]; Print["until: ", i];
EOF
python3 -c "
n=30000
print('Print[\"module: \", Module[{%s}, 0]];' % ','.join('x%d=1'%i for i in range(1,n+1)))" > /tmp/b328-4.rum
for i in 1 2 3 4; do echo "--- $i ---"; ./target/release/rum-cli /tmp/b328-$i.rum < /dev/null; echo "exit=$?"; done
```

All four must exit 0 with the right values. **An exit code of -6 or 134 is a
failure of this brief.**

**This is the evaluation hot path, so the microbench A/B is a hard gate.** Run
`scripts/core-language-microbench.sh` on base and candidate and paste the loop
and `Module` labels plus same-tier neighbors. State plainly whether the machine
was quiet.

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
- **Stop-loss is real.** If a site's suspension shape resists the conversion,
  stop and report that site rather than weakening it.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per driver, with line ranges, and how suspension state is
   carried.
2. Verification — each command with verbatim output and exit codes, the new
   tests' runtimes, and the A/B rows with neighbors.
3. What `While[True, 1]` does now.
4. Stop-loss / open questions.
