# brief-363 — finish brief-362: settle the suspended-candidate test

## Goal

brief-362's watermark implementation is in the working tree and looks sound:
36 of its 37 focused tests pass, including the sweep-count bound, the
reclamation bound from both sides, partial-sweep rearm, `ClearAll`, `Remove`,
snapshot restore, and saturating arithmetic. One test it wrote itself fails
consistently, and the mandatory gate was never run.

**Do not redo the implementation.** Settle the one test, then run the gate.

## Part 1 — first, separate two very different explanations

The failing test (`scoping_gensym_gc.rs:479`) builds a candidate that is
reachable through an ordinary global definition
(`root = Hold[x]`, via `allocate_retained_candidate`), suspends inside
`List[List[Hold[candidate], InputString["gc: "]]]`, sweeps during the
suspension, and asserts the candidate survives. It does not.

There are two possible causes and they have opposite severities:

- **(a)** The root walk misses the candidate *only* while a suspension is
  pending — the continuation frame holds it opaquely, outside
  `pending_suspension.active_stack`, which is the only suspension state
  `for_each_root_expr` visits (`gensym_gc.rs:187-191`).
- **(b)** The root walk misses a candidate reachable through an ordinary
  global definition **regardless of suspension** — which would be a serious
  pre-existing liveness defect, not a test problem.

**Determine which, with a control**: the same `allocate_retained_candidate`
followed by an immediate `collect_and_sweep_dead_gensyms`, with no suspension
at all. Report the result verbatim before doing anything else.

- If **(b)**, STOP and report. That outranks everything in this brief and
  needs its own diagnosis.
- If **(a)**, proceed to Part 2.

## Part 2 — under (a), the ruling

**Do not change which gensyms are considered live.** Enumerating continuation
closure frames is a real design change with its own risk, and it is not this
brief's job.

Instead:

- Rewrite the test to use a suspension root the sweep already recognizes —
  something reaching `pending_suspension.active_stack` — so it tests what
  brief-362 actually needed to preserve: that suspended state stays live
  across the watermark change. If no source-level construction reaches that
  state, say so and drop the test rather than weakening it into something that
  passes without asserting anything.
- **Answer the reachability question with evidence**: can a real host reach
  the (a) scenario — a sweep running while a continuation frame holds the only
  reference to a gensym? Consider whether a host can evaluate anything else
  while a request is suspended, whether `Monitor` callbacks or nested trials
  can, and whether a resumption path can itself cross the threshold. Give a
  verdict with the code that supports it.
  - If it **is** reachable, that is a latent liveness defect: report it
    clearly for its own brief. Do not fix it here.
  - If it is **not** reachable, say so and record why.

## Part 3 — the gate brief-362 never reached

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B: `--scale 5`, same-binary noise floor **first, three runs each
way, reported as the spread**. Labels:

```
module-local            module-chain-escape-small    module-chain-escape-large
block-local             with-local                   do-small
matcher-single-tiny     symbol-ownvalue-read
```

Plus the `module-retained-gensym-gc-batch` label brief-362 added, reported
across two batch counts to show per-batch cost is now flat rather than
climbing. Name the untouched controls; never the suite median.

**Note on `rum-jupyter`**: an intermittent native ZeroMQ ordering failure
(`native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages`)
appeared once during my own workspace run and did not reproduce in three
isolated runs or two further workspace runs. It is a known flake, unrelated to
this work. If you see it, note it and move on — do not chase it.

## Scope

- Files you MAY modify: `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs`;
  `scripts/core-language-microbench.sh`. Implementation files **only** if
  Part 1 finds (b) — and then only to report, not to fix.
- **The general accept/stop criterion**: *any change to which gensyms are
  considered live is a stop.*
- MUST NOT touch: `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Do not weaken the test to make it pass.** A test that no longer asserts
   liveness is worse than a deleted one, because it looks like coverage.
2. **The control in Part 1 is the whole point** — run it before theorizing.
3. **Counters before timings.**
4. **Stop-loss is real**, but note that the previous stop was correct and this
   brief exists to resolve it.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. **Part 1's control result, verbatim, and the verdict: (a) or (b).**
2. What changed — per file.
3. The reachability verdict for the (a) scenario, with supporting code.
4. Verification — each command with verbatim output.
5. The A/B — counters first, then the retained-Module label across two batch
   counts, with a three-run-each-way noise floor and the controls named.
6. Stop-loss / open questions.
