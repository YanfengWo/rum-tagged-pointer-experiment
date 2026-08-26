# brief-361 — REPL re-parses every line; the Jupyter kernel busy-spins

Two independent host-layer defects, neither on an evaluation path. They share
a brief because each is small; they do **not** share an implementation.

## Part A — the REPL re-parses the whole buffer per line

`crates/rum-repl/src/lib.rs:121` calls `session.is_complete(pending)` on the
entire accumulated buffer after appending each line, so pasting an L-line
definition parses O(L^2) characters.

### The shape of the repair

Most incomplete input is incomplete because a delimiter is open. Maintain an
incremental delimiter/string state over the appended text and, when the buffer
is provably unbalanced, report `Incomplete` **without** calling `is_complete`.
Only when the text is balanced does the full check run — that is what catches
the case balance cannot predict, a form whose delimiters are closed but whose
expression is unfinished (`1 +` continued on the next line), which `TODO.md`
already records as deliberately unpredicted.

**The verdict for every input must be identical to today's.** This changes
when the full check is invoked, never what it concludes.

### The traps

The fast check must agree with the lexer about what a delimiter is:

- a bracket inside a string literal is not a delimiter;
- a bracket inside a comment is not a delimiter, including nested comments if
  rum has them — check;
- an escaped quote does not end a string;
- rum's own bracket kinds, all of them — find the list rather than assuming
  three;
- a form that is already `Invalid` must still be reported `Invalid`, not
  `Incomplete`, so the fast path must not swallow it.

If the fast check and the real lexer can disagree on any input, the repair is
wrong. Prefer reusing the lexer's own tokenizer over hand-rolling a scanner,
if it can be driven incrementally.

### Tests

- A completeness matrix asserting identical verdicts to the current build for:
  balanced single line, unbalanced open bracket, nested brackets, a bracket
  inside a string, a bracket inside a comment, an escaped quote, an unfinished
  trailing operator across lines, a syntactically invalid form, an empty line,
  and a line that is only whitespace or only a comment.
- A cost test: appending L lines performs work proportional to L, not L^2 —
  assert via a counter of characters or tokens examined, not a timing.

## Part B — the Jupyter kernel burns a core while a cell runs

`crates/rum-jupyter/src/socket.rs:329-334`: when `execute_running()` is true
and no event is ready, the loop calls `std::thread::yield_now()` and continues,
skipping the launcher's idle sleep. A long-running cell therefore spins a full
core and ping-pongs against the zmq worker.

### The shape of the repair

Back off with a small sleep instead of yielding. Keep it short enough that
interactive latency does not regress — the existing idle path's sleep is the
reference point for what the project already considers acceptable.

### The traps

- Do not delay delivery of an interrupt or a shutdown request; those must stay
  as responsive as they are now.
- Do not change the loop's exit conditions.
- A fixed sleep is fine; if you propose escalating backoff, state its cap.

### Tests

- The existing kernel tests stay green.
- A test that a shutdown request during a running execution is still observed
  promptly.
- State plainly whether CPU use can be asserted in a test here; if it cannot,
  say so rather than inventing a proxy.

## Scope

- Files you MAY modify: `crates/rum-repl/src/**`, `crates/rum-jupyter/src/**`,
  and their tests. Touch `rum-syntax` only if the fast check reuses its
  tokenizer and that requires a non-breaking addition — report it if so.
- **The general accept/stop criterion**: *any completeness verdict change, or
  any change to kernel loop exit conditions, is a stop.*
- MUST NOT touch: `rum-runtime`, `rum-stdlib`, `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, `docs/**`, `.git`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **The two parts are independent.** If one stalls, land the other and report.
2. **The fast check is a filter, not a parser.** It may only ever say "not yet
   balanced, so certainly incomplete"; every other conclusion goes to the real
   check.
3. **`rum-jupyter` has a known intermittent ZeroMQ ordering flake** (seen in
   briefs 165 and 242, green in isolation both times). If a jupyter test fails
   once and passes in isolation, say so explicitly rather than chasing it.
4. **Stop-loss is real.** If the same step fails twice for the same reason,
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-repl
cargo test -p rum-jupyter
cargo test --workspace
```

No microbench A/B: neither part touches an evaluation path. If you believe one
does, that is a design error — report it.

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

1. Part A — what changed, the delimiter/string/comment cases you had to
   handle, and how you established the fast check cannot disagree with the
   lexer.
2. Part A — the completeness matrix, verbatim, and the linear-cost counter.
3. Part B — what changed, the sleep you chose and why, and the
   interrupt/shutdown responsiveness evidence.
4. Verification — each command with verbatim output.
5. Stop-loss / open questions.
