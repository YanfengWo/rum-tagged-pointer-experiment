# Brief 282 — freeze REPL rendering with full-output snapshots

`crates/rum-repl/src/tests.rs` has 23 tests covering `:help`,
`:inspect`, `:messages`, `:parse`, `:format`, `:lower`, `:complete`,
history, multiline input, and result/message rendering. Coverage is not
the gap — **assertion shape is**: 48 `contains(...)` calls against 1
`assert_eq!`. Each test verifies that a few substrings appear somewhere
in the output, so nothing catches a reordered command list, a deleted
help line, changed indentation or alignment, or a stray blank line.

User ruling 2026-07-20: **freeze all of it.** Every rendering surface
in this file gets pinned as a whole.

## What to do

For each of the 23 tests in `crates/rum-repl/src/tests.rs`, replace the
trailing block of `assert!(rendered.contains(...))` calls with a single
assertion pinning the ENTIRE captured output:

```rust
assert_eq!(rendered, expected);
```

The tests already capture full output (`let mut output = Vec::new();`
… `String::from_utf8(output).unwrap()`), so no restructuring is needed —
only the assertion changes.

**Do not guess the expected text. Capture it.** Run each test, read the
actual rendered string, and pin THAT. (This brief deliberately contains
no expected output: the REPL only starts on a TTY, so it cannot be
probed from a pipe, and a hand-written expectation would be an
invention.)

Write each expected value so a human can read it as the screen it
represents — a raw string literal (`r#"…"#`) with one rendered line per
source line is preferred over an escaped one-liner. Match the file's
existing style where one exists.

## Non-determinism — find it before you freeze it

Some outputs may contain content that varies per run: temp history
paths (`temp_history_path` builds a nanosecond-stamped filename),
absolute paths, timings, or anything environment-derived.

**Before converting a test, check whether its output contains such a
segment.** For any that does:

- do NOT freeze the varying segment;
- either normalize it (e.g. replace the varying run of text with a
  fixed placeholder before comparing) or keep a narrow `contains`
  assertion for that ONE segment while pinning everything else;
- add a short comment saying which segment is unpinned and why;
- list every such case in your report.

A snapshot that is flaky is worse than the `contains` assertion it
replaced. If you cannot make a test deterministic without changing
product behavior, leave that test as-is and report it — that is an
acceptable outcome, not a failure.

## Hard constraints

- **No new dependencies.** Do not add `insta`, `expect_test`,
  `goldenfile`, or any other snapshot crate. This crate's dependency
  list is three path deps and stays that way. Plain `assert_eq!` on
  strings is the mechanism.
- **No production code changes.** This is a test-only brief. If a
  rendering looks wrong to you, report it — do not fix it here.
- Do NOT rename, add, delete, merge, or split any test.
- Do NOT change the input scripts the tests feed the REPL, or the
  session setup (`eval_text` calls). Only the assertions change.
- If a helper would reduce repetition (e.g. one that renders a
  multi-line mismatch readably, since `assert_eq!` on long multi-line
  strings diffs poorly), adding it to this file is fine.

## Negative calibration (MANDATORY — a snapshot that cannot fail is worthless)

A frozen expectation is only worth something if it actually bites.
Prove it:

1. Pick one snapshot covering `:help`.
2. Temporarily perturb the rendering it pins — the smallest possible
   change, e.g. delete ONE line from the help text, or change one
   character of indentation, in whatever source produces it.
3. Confirm that test now FAILS, and paste the failure.
4. Revert the perturbation and confirm green again.

Repeat for one `:inspect` snapshot and one message-rendering snapshot
(three calibrations total, one per surface family). Paste each.

If a perturbation does NOT fail the snapshot, the snapshot is too loose
— fix it and say so.

## Scope

- Files you MAY modify: `crates/rum-repl/src/tests.rs` ONLY.
- Files/areas you MUST NOT touch: `crates/rum-repl/src/lib.rs`,
  `crates/rum-repl/src/commands.rs` (its 3 tests are command-PARSING
  tests, not rendering — out of scope), `crates/rum-repl/Cargo.toml`,
  any other crate, `TODO.md`, `PROGRESS.md`, `AGENTS.md`, `.git`,
  `Cargo.lock`.
  The negative calibration in the previous section necessarily perturbs
  a MUST-NOT file temporarily — that is expected and permitted **only**
  as a revert-immediately probe. It must not appear in the final diff.
  Confirm `git status` shows only `tests.rs` modified when you finish.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-repl
cargo test --workspace
```

Baseline is `1654 passed / 0 failed` on `bd9e617`; state your totals.
The test COUNT must not change — 23 tests in this file before and
after. Known sandbox caveat: loopback-bind ZeroMQ tests may fail under
the sandbox; if so run and paste the two supplemental splits
(`--workspace --exclude rum-jupyter`; `-p rum-jupyter --lib -- --skip
zeromq::tests::native`) and say so. Claude re-runs the authoritative
gate outside the sandbox.

Also report the final `contains(` count in the file — it should be near
zero, with every survivor explained as a documented non-determinism
carve-out.

No microbench: test-only change.

## Stop-loss

- A test's output cannot be made deterministic without a product
  change → leave that test, report it.
- More than a third of the tests turn out to need carve-outs → STOP and
  report; that would mean the surface is not as stable as assumed and
  the ruling should be revisited before continuing.
- The work requires a production-code change → STOP and report.

(`git checkout --` fails under the sandbox with `Unable to create
'.git/index.lock': Operation not permitted` — restore by scoped patch
instead and say so.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.

## Report format

1. What you changed, with the before/after `contains(` count.
2. Verbatim verification output (fmt, `-p rum-repl` and workspace
   totals, test count).
3. The three negative calibrations, each with its failure output.
4. Every non-determinism carve-out: which test, which segment, why.
5. Stop-loss / open questions.
