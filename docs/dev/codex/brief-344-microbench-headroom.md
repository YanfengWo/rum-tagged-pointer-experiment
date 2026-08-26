# brief-344 — the microbench must not fail quietly, and must run above scale 1

## Goal

`scripts/core-language-microbench.sh --scale 5` dies inside rum on
`evaluation exceeded iteration work limit (100000)`, writes a `timings.tsv`
containing only its header row, prints one `warning:` line, and **exits 0**.
Two separate things are wrong: the harness has no headroom above `--scale 1`,
and when it has none it produces a file that looks like a result.

Fix the loud-failure half unconditionally. Then diagnose the headroom half and
fix it **in the harness script only** — if it cannot be fixed there, stop and
report rather than reaching into the kernel.

This is the instrument every A/B in this project depends on, and it has lied
once already: before `dd4bdefc` it never closed stdin, so a run either
finished in four seconds or burned its timeout and produced a header-only
timings file — the same silent-empty shape as this one. Every A/B taken before
that fix is suspect. That is the standard this brief is held to.

## Context pointers

- Read `AGENTS.md` and `docs/testing-strategy.md` before touching anything.
- **Reproduction** (director, release CLI at `4f5b85f1`):

  ```text
  $ bash scripts/core-language-microbench.sh --output-dir <d1>          # scale 1
  $ wc -l <d1>/timings.tsv
        80

  $ bash scripts/core-language-microbench.sh --scale 5 --output-dir <d2>
  warning: rum exited with 1; see <d2>/raw/rum.stderr
  core microbench output: <d2>
  $ echo $?
  0
  $ wc -l <d2>/timings.tsv
         1
  $ cat <d2>/raw/rum.stderr
  ! evaluation exceeded iteration work limit (100000)
  error: evaluation exceeded iteration work limit (100000)
  $ tail -1 <d2>/raw/rum.stdout
  RUM_CORE_MICRO_BEGIN
  ```

- The silent-exit site is `scripts/core-language-microbench.sh:750-756`
  (`run_engine`'s trailing `echo "warning: … exited with …"`), after which the
  script writes `timings.tsv` from an empty measurements file and reports
  success.
- The nesting that produces the product is `rumMicroBench` at
  `scripts/core-language-microbench.sh:274-291`: an outer
  `Do[…, {k, 1, rumMicroReps}]` wrapped around the timed
  `Do[expr, {i, 1, n}]`, with `n = iterations * scale`.
- The budget that refuses it: `DEFAULT_ITERATION_WORK_LIMIT` at
  `crates/rum-runtime/src/runtime/metadata.rs:10`, charged through
  `enforce_iteration_work_limit_for_big_count`. The propagation rule is
  deliberately conservative — commit `d74f3218` propagates the iteration
  product **only into retaining constructs** — and `TODO.md` records under
  "Deferred, with evidence" that the decision is made from construct nesting
  rather than data flow, on purpose. **Do not change that rule, and do not
  argue with it here.** Your job is to find out exactly which labels and which
  nesting shape hit it, and whether the harness can express its repetition
  loop without accumulating a product it does not need.
- Acceptance is mechanical: the named checks below decide.

## Scope

- Files you MAY modify:
  - `scripts/core-language-microbench.sh`
  - `scripts/core-memory-microbench.sh` — **only** if it shares the
    quiet-failure shape; check it and say what you found either way.
- Files/areas you MUST NOT touch: all Rust source, `crates/**`, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`.
- **Explicitly forbidden repairs**, each of which trades a measurement
  problem for a worse one:
  - changing `DEFAULT_ITERATION_WORK_LIMIT` or any resource-budget rule;
  - adding a CLI flag or environment override to raise the limit — the CLI is
    published surface and this is not the brief that expands it;
  - moving the repetition loop out of rum into the shell, which would fold
    process startup and a cold cache into every measurement;
  - lowering the default `--scale 1` iteration counts, or changing which
    labels exist, or changing what each label measures.

## Deliverable

### Part 1 — fail loudly (unconditional)

- A run whose engine exits non-zero, or which produces zero measurement rows,
  makes the script **exit non-zero** with a message naming the cause and the
  stderr path.
- Such a run must not leave behind a `timings.tsv` that a later reader could
  mistake for a valid result. Either do not write it, or write it in a form
  that is unmistakably an error. State which you chose and why.
- The existing `warning:` line stays useful for a partial run: say explicitly
  how you distinguish "engine failed" from "engine succeeded with warnings",
  if that distinction exists at all in the current output.

### Part 2 — headroom (diagnose, then fix in the harness)

- The exact inventory: for `--scale 5` and `--scale 10`, which labels exceed
  the limit, at what `n`, and with what product. Compute it, do not estimate.
- The precise nesting rule that charges the product, read from the source and
  quoted with `path:line` — including whether the outer `Do` is charged
  because of what it retains, and what shapes are *not* charged.
- A harness-level repair that lets `--scale 5` complete, if one exists within
  Scope, preserving exactly:
  - the min-of-reps method and the one untimed warm evaluation before the reps;
  - the timed region containing **only** the bare expression loop — no
    per-iteration assignment, no sink, no extra builtin inside the timing;
  - the monotonic in-kernel `AbsoluteTiming` source (never wall-clock
    differencing);
  - the label set, the label semantics, and the `n` reported per label.
- If no repair inside Scope preserves all of the above, **stop and report
  that**, with the options you rejected and why. A quiet degradation of the
  measurement method is a worse outcome than a documented ceiling.

## rum / Rust pitfalls (read twice)

1. **The instrument is under test here, not the engine.** Every claim about
   what the harness now does needs a run whose output you paste, not a reading
   of the script.
2. **A liveness assertion is mandatory**: a green exit code is not evidence
   that measurements happened. Check the row count and the label set, and say
   what you checked.
3. **Do not "fix" the failure by making the workload smaller.** Reducing
   iteration counts to fit under the budget silently changes every number the
   suite has ever produced.
4. **Do not normalize, smooth, or post-process timings** anywhere in this
   change. The harness reports minima of raw reps; keep it that way.
5. **`--scale 1` must stay byte-comparable.** The label set and per-label `n`
   at scale 1 must be identical before and after your change; only timing
   values may differ, and only as noise.
6. **Stop-loss is real.** If Part 2 fails twice for the same reason, deliver
   Part 1 alone and report Part 2's dead end. Part 1 landing on its own is a
   successful outcome for this brief.

## Verification (MANDATORY — you run it; iterate to green)

No Rust source changes, so no `cargo test` gate is required — but run
`cargo fmt --check` to prove the tree is untouched, and paste `git status
--short` showing only the script(s) modified.

Then, pasting verbatim output and the exit code for each:

```
bash scripts/core-language-microbench.sh --output-dir <base-scale1>
wc -l <base-scale1>/timings.tsv
bash scripts/core-language-microbench.sh --scale 5 --output-dir <scale5>
wc -l <scale5>/timings.tsv
```

- Scale 1: 80 lines, the same label set as before your change. Paste a diff of
  the label columns against a scale-1 run made from the unmodified script —
  it must be empty.
- Scale 5: either a full row set (Part 2 succeeded) or a non-zero exit with a
  clear message and no misleading `timings.tsv` (Part 2 stopped, Part 1
  works).
- A deliberate negative calibration: force a failure that is **not** the
  budget (an unreadable `--rum-cli` path is the easy one) and show the script
  exits non-zero for that too. A guard that only catches the one failure you
  reproduced is not a guard.

Size every bound with `scripts/run-bounded.py --timeout SECONDS -- <command>`
from a measured run of the same shape; a scale-5 run is roughly five times a
scale-1 run, and state the bound you chose.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only. No
  third-party engine appears in this harness, its output, or your report.
- No new `unsafe` anywhere; this brief changes no Rust at all.
- Minimal diff: no drive-by reformatting or reorganizing of the script beyond
  what the two parts require.
- Paste real verbatim output. If a run did not reach green, say so explicitly.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Part 1 — the loud-failure behavior, with the verbatim runs and exit codes,
   including the negative calibration.
3. Part 2 — the over-limit inventory (labels, `n`, products), the quoted
   propagation rule, and either the repair or the dead end with rejected
   options.
4. The scale-1 label-set diff, verbatim.
5. Whether `core-memory-microbench.sh` shares the defect, and what you did.
6. Stop-loss / open questions — where you stopped and why (empty if none).
