# brief-047: Stream `Print` output live from rum-cli (long `--eval`/`--file` runs are currently a black box)

## Goal

A single long `rum-cli --eval`/`--file` run shows NO output until it EXITS: rum's
`Print` is captured into `Session.prints` (a `Vec<String>`) and the CLI only emits
it AFTER the top-level request returns (`print_outcome` in
`crates/rum-cli/src/main.rs`). So a multi-hour run (e.g. a full test suite) writes a
0-byte log until the very end, and a `kill -9` discards everything. This is
APP-LEVEL capture, not OS pipe buffering — `script`/`stdbuf`/PTY do NOT fix it
(verified). Add an optional **print sink** on `Session` that `push_print` invokes
immediately; rum-cli registers a sink that writes each line to stdout and flushes,
so progress appears live. When done: `rum-cli --eval 'Do[Print["tick ", i]; <slow>,
{i,1,5}]' > log` shows ticks in `log` AS THEY HAPPEN, not all at the end.

**NOT in scope — `Pause` is fine.** An earlier observation that "`Pause` is a no-op"
was a MISATTRIBUTION (a background ticker had completed because enough wall-clock
elapsed across steps, not because `Pause` returned instantly). Verified:
`rum-cli --eval 'Pause[3]; "done"'` takes 3.01s real. Do NOT touch `Pause`
(`builtins_input.rs` `eval_pause`).

## Root cause (already diagnosed — implement, don't re-investigate)

- `Print` → `Session::eval_print` (`crates/rum-runtime/src/runtime/builtins_input.rs:97`)
  → `self.push_print(rendered)` (`session_helpers.rs:156`), which does
  `self.prints.push(text.into())`.
- The request methods (`eval_request`, `eval_file_request`, …) build the response by
  slicing `self.prints` (`session_requests.rs` `split_off(checkpoint.print_checkpoint)`)
  and returning it as `SessionEvalResponse.prints`.
- rum-cli emits them only at the end: `print_outcome` (`main.rs:134`) loops
  `for printed in &outcome.prints { println!("{printed}") }`, and
  `print_failure_side_effects` (`main.rs:146`) does the same on failure.
- The REPL flushes per INPUT (`rum-repl/src/lib.rs`), not per `Print` within one
  eval — so it does not stream a long eval either.

## The change (precise)

### 1. rum-runtime — add an opt-in print sink on `Session`

- `crates/rum-runtime/src/runtime/model.rs` (`pub struct Session`, ~:300): add a
  field
  ```rust
  print_sink: Option<Box<dyn FnMut(&str) + Send>>,
  ```
  (`+ Send` only if `Session` already requires it — match the struct's existing
  auto-traits; if `Session` is not `Send`, drop `+ Send`.) Initialize to `None`
  wherever `Session` is constructed (`Session::new` / `Default`); it is NOT part of
  serializable/snapshot state and NOT part of the rollback snapshot.
- Public API on `Session` (same impl block style as other public setters):
  ```rust
  pub fn set_print_sink(&mut self, sink: Box<dyn FnMut(&str) + Send>) { self.print_sink = Some(sink); }
  pub fn clear_print_sink(&mut self) { self.print_sink = None; }
  ```
- `push_print` (`session_helpers.rs:156`): stream when a sink is set, and SKIP
  accumulation in that mode (so a long streaming run does not also balloon the
  `prints` Vec — bounded memory):
  ```rust
  pub(super) fn push_print(&mut self, text: impl Into<String>) {
      let text = text.into();
      if let Some(sink) = self.print_sink.as_mut() {
          sink(&text);           // streamed live; do NOT also accumulate
      } else {
          self.prints.push(text);
      }
  }
  ```
  Rationale for skip-accumulate: the only consumer of `Session.prints` is the
  response API (`SessionEvalResponse.prints`); a caller that registers a sink is
  taking responsibility for output itself and does not read `outcome.prints` (the
  CLI, below, stops reading it). Non-sink consumers (rum-host/jupyter) are
  UNCHANGED — they never set a sink, so accumulation is exactly as before. (If you
  find a code path that reads `self.prints` mid-eval other than the response slice,
  STOP and report — then we accumulate AND stream instead.)

### 2. rum-cli — register a flushing stdout sink and stop double-printing

- `crates/rum-cli/src/main.rs`, in the `--eval` and `--file`/path arms (and inside
  `eval_file`): BEFORE the `session.eval_request(...)` / `session.eval_file_request(...)`
  call, register the sink:
  ```rust
  session.set_print_sink(Box::new(|text| {
      use std::io::Write;
      let mut out = io::stdout().lock();
      let _ = writeln!(out, "{text}");
      let _ = out.flush();
  }));
  ```
- `print_outcome` and `print_failure_side_effects`: REMOVE the
  `for printed in &outcome.prints { println!(...) }` / `&failure.prints` loops —
  prints are now streamed live via the sink, so re-emitting them would DOUBLE the
  output. KEEP the `messages` (stderr) loop and the final `value` println. (Messages
  streaming is out of scope for this brief; leave messages emitted at the end.)
- Do NOT register a sink for `--parse`/`--format` (no eval) or anywhere a sink isn't
  wanted.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model.rs` (the `Session` field + setter; `new`/`Default` init),
  - `crates/rum-runtime/src/runtime/session_helpers.rs` (`push_print`),
  - `crates/rum-cli/src/main.rs` (register sink in `--eval`/`--file`/path arms; drop the prints loops in `print_outcome`/`print_failure_side_effects`),
  - a test module under `crates/rum-runtime/src/runtime/tests/` for the sink unit test.
- MUST NOT touch: `Pause`/`eval_pause`; the kernel evaluator/matcher; `Expr`/`Symbol`/serde
  shapes; `rum-host`/`rum-jupyter` (they must keep working unchanged via the
  non-sink accumulation path); any `mma-hol`/snapshot; `AGENTS.md`/`PROGRESS.md`/
  `TODO.md`/`docs/**` except this report; git.
- The sink field/check is opt-in: when no sink is set (the default, and ALL
  rum-host/jupyter/test consumers), behavior and cost are IDENTICAL to today. Do NOT
  add any per-eval-step cost; `push_print` runs once per `Print`, and the
  `Option::as_mut` check there is negligible.

## rum / Rust pitfalls (read twice)

1. **Backward compatibility is the gate.** rum-host/jupyter/tests do NOT set a sink,
   so they take the `else` branch and accumulate into `Session.prints` exactly as
   today — `SessionEvalResponse.prints` stays populated for them. Only the CLog path
   changes. `cargo test --workspace` must stay green.
2. **No double output.** If you register the sink but forget to remove the
   `print_outcome` prints loop, every line prints twice. Verify the CLI check below
   shows each tick exactly once.
3. **Suspension-safety.** The sink lives on `Session`, so it persists across
   suspend/resume (`--eval` that requests input, multi-statement evals). Do not
   reset it mid-request. Prints from a resumed continuation must also stream.
4. **Rollback.** Streamed output cannot be un-streamed; that is correct and
   WK-consistent (Print side effects during an aborted eval are visible). The
   rollback snapshot still restores `self.prints` for the non-sink path; do not add
   the sink to the rollback snapshot.
5. **`Send` bound.** Match `Session`'s existing auto-traits. If the workspace builds
   with `+ Send` on the boxed sink, keep it; if `Session` is `!Send` and adding the
   bound breaks construction, drop `+ Send`. The CLI closure (locking stdout) is
   `Send` either way.
6. **Minimal diff**; one field + one setter + the `push_print` branch + the CLI
   wiring + one test. No reformatting.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace            # all green; rum-host/jupyter unaffected
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# (a) live streaming end-to-end — the log must grow BEFORE the process exits:
$BIN --eval 'f[n_]:=If[n<2,n,f[n-1]+f[n-2]]; Do[Print["tick ", i]; f[27], {i,1,5}]' > /tmp/cdx047_stream.log 2>&1 &
PID=$!
sleep 8
echo "MID-RUN tick count (expect >0 and <5):"; grep -c '^tick ' /tmp/cdx047_stream.log
wait $PID
echo "FINAL tick count (expect 5):"; grep -c '^tick ' /tmp/cdx047_stream.log

# (b) no double-printing — exactly 3 lines:
$BIN --eval 'Do[Print["x", i], {i,1,3}]'      # x1 / x2 / x3, each once

# (c) Pause still works (control — unchanged):
/usr/bin/time -p $BIN --eval 'Pause[2]; "ok"'  # ~2s real, prints ok
```

Paste every line verbatim. Add a rum-runtime unit test: register a sink capturing
into a `Vec<String>` (via e.g. `Rc<RefCell<Vec<String>>>` / a channel), eval
`Do[Print[i], {i,1,3}]`, assert the sink received `["1","2","3"]` in order AND that
`outcome.prints` is empty in sink mode; and a second test that with NO sink,
`outcome.prints == ["1","2","3"]` (unchanged behavior).

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes in the working tree; Claude gates +
  commits.
- Do NOT modify files outside Scope; do NOT touch `Pause`, the evaluator, or
  rum-host/jupyter behavior; never edit any `mma-hol`/snapshot.
- Full machine access, nothing outside this repo, no network. Commands: edits in
  scope, `cargo`, release `rum-cli`, `sleep`/shell for the streaming check.
- Minimal diff; paste real verbatim output; if `cargo test --workspace` regresses or
  the streaming check fails, STOP and report — never fake a result.

## Report format

1. What changed — the `Session` sink field + setter, `push_print`, the CLI wiring +
   dropped prints loops, the test; line ranges + one-line why each.
2. Verification — `fmt`, `cargo test --workspace` count, and (a)/(b)/(c) verbatim.
3. Verdict — live streaming works (mid-run count >0 <5, final 5), no double output,
   rum-host/jupyter unaffected (no sink → accumulation unchanged), Pause untouched.
4. Stop-loss / open questions.
