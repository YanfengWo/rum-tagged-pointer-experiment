# Brief 237 — trusted-host contract surface: streaming prints, script/file eval, queued input

Implementation brief — embedding API line, item a2a (see
`docs/dev/embedding-api-line.md`, findings F1/F5/F6). Four small additive
methods on `HostSession` that lift already-existing, already-tested
runtime `Session` capabilities up to the embedding contract. **Zero
kernel changes** — every method is a guarded forward plus the existing
outcome mapping. This is the surface half of the dogfood ruling; the
`rum-cli`/`rum-repl` ports themselves are follow-up briefs and MUST NOT
be attempted here.

Background (survey evidence): `rum-cli` bypasses the contract because
these four things exist only on the raw `Session` —
`set_print_sink` (streaming prints, `rum-cli/src/main.rs:285-304`),
`eval_script_request`/`eval_file_request` (`main.rs:201,207`), and
`queue_input_response` (`main.rs:240-264`, where the CLI smuggles argv
through the input-suspension machinery for lack of a contract channel).

## Pre-diagnosed seams (verified this session — follow these)

- `Session::set_print_sink(Box<dyn FnMut(&str) + Send>)` /
  `clear_print_sink()`
  (`crates/rum-runtime/src/runtime/model/session.rs:109-115`, field at
  `:76`). Semantics of `push_print`
  (`runtime/session/session_helpers.rs:178-185`): when a sink is
  installed the text goes ONLY to the sink — the per-request `prints`
  buffer stays empty. Exclusive, not tee.
- `Session::eval_script_request(&str)` and
  `Session::eval_file_request(impl AsRef<Path>)`
  (`runtime/session/session_requests.rs:228,246`) return the same
  `SessionEvalResponse` shape as `eval_request` (value, termination,
  messages, prints, trace) — the existing
  `eval_success_from_runtime`/`eval_failure_from_runtime` mapping
  applies unchanged. `rum-host` already calls `eval_script_request`
  internally (`restore_from_bytes`, `rum-host/src/session.rs:204`).
- `Session::eval_file` reads through the FileIo provider
  (`runtime/session/session_eval.rs:294-314`,
  `EvalError::NoFileIoProvider` when absent) and rolls back loader
  state on error. A `sandboxed()` host has no provider — `eval_file`
  fails there by construction.
- `Session::queue_input_response(impl Into<String>)`
  (`runtime/session/session_api.rs:144-147`).
- `HostSession::reset()` rebuilds the inner `Session`
  (`rum-host/src/session.rs:170-173`) — a print sink, being a runtime
  field of the inner session, does NOT survive reset (same nature as
  cancel-handle invalidation, which reset's rustdoc already documents).
- Guard pattern for new methods: copy `eval`
  (`rum-host/src/session.rs:271-288`) — `poisoned` precheck, then
  `self.guard(...)`, mapping via the shared helpers.

## Required change (all in `crates/rum-host`)

1. **`HostSession::set_print_sink(&mut self, sink: impl FnMut(&str) + Send + 'static)`**
   and **`clear_print_sink(&mut self)`** — forward to the inner session
   (box internally; do not expose `Box` in the signature). Rustdoc must
   state: (i) exclusive routing — while a sink is installed,
   `HostEvalSuccess.prints` / `HostFailure.prints` come back empty and
   every `Print` line reaches the sink as it happens, during evaluation;
   (ii) the sink does not survive `reset()` (mirror the cancel-handle
   wording) — reinstall after reset; (iii) the sink is called on the
   evaluating thread and must not panic (a panic poisons the session
   like any other caught panic).
2. **`HostSession::eval_script(&mut self, source: &str) -> HostEvalOutcome`**
   — guarded forward to `eval_script_request`, mapped exactly like
   `eval`. Rustdoc: multi-form script semantics (forms evaluated in
   order; the response describes the final form's value; `termination`
   follows the brief-231 last-form rule; messages/prints aggregate
   across forms).
3. **`HostSession::eval_file(&mut self, path: impl AsRef<Path>) -> HostEvalOutcome`**
   — same pattern over `eval_file_request`. Rustdoc: reads through the
   configured FileIo provider; a session without one (e.g.
   `sandboxed()`) fails — probe the actual `HostFailureCode` the
   `NoFileIoProvider` path maps to today, pin it in a test, and report
   it. Do NOT add or change any `HostFailureCode` variant.
4. **`HostSession::queue_input_response(&mut self, input: impl Into<String>)`**
   — plain forward (no guard needed: it is a queue push that cannot
   panic; state in rustdoc that `reset()` clears the queue and that
   this is the intended way to pre-supply input to `Input`/
   `InputString` in batch/script runs).
5. **Script-suspension behavior: pin the status quo, do not change it.**
   Determine what `eval_script` does today when a form suspends on
   `Input` with an empty queue (expected: the suspension surfaces as
   `HostEvalOutcome::InputRequired` through the existing mapping) and
   what happens to the REMAINING forms after `resume_with_input`.
   Write both down in the rustdoc/guide and pin the observable half in
   a test. If the observed behavior looks broken (e.g. panics or
   corrupts state), STOP and report instead of fixing.
6. **Guide.** `docs/guide/embedding.md` gains a "Streaming prints and
   batch input" section: exclusive-sink semantics, reset survival rule,
   the queue-then-eval pattern for scripts (replacing source-string
   interpolation of host data), and a one-paragraph pointer that
   `eval_script`/`eval_file` are the script-shaped entries (file reads
   go through the FileIo provider; sandboxed sessions have none).

## Tests (`crates/rum-host/src/tests.rs`)

a. Sink streaming: install a sink collecting into an
   `Arc<Mutex<Vec<String>>>`, eval a multi-`Print` program — sink saw
   the lines in order AND the outcome's `prints` is empty; after
   `clear_print_sink()` the next eval's `prints` is populated again.
b. Sink vs reset: install a sink, `reset()`, eval a `Print` — the sink
   is NOT called; the outcome's `prints` carries the line.
c. `eval_script`: a three-form script (definition; `Print`; expression)
   — success value is the last form's, prints aggregated; a script with
   a syntax error fails with the existing syntax classification.
d. `eval_file`: with a FileIo provider (reuse the existing test
   provider fixture if `rum-host` tests have one, else add a minimal
   in-memory `FileIo` impl in the test module), a two-form file
   evaluates and defines symbols; on `sandboxed()` (no provider) it
   fails — pin the observed `HostFailureCode`.
e. Queued input: `queue_input_response("hello")` then
   `eval("InputString[]")` completes WITHOUT suspension and returns
   `"hello"` (requires a policy that allows `Capability::Interactive` —
   reuse the existing Input-suspension test setup).
f. Script suspension status quo (per change 5): a script whose middle
   form calls `InputString[]` with an empty queue returns
   `InputRequired`; pin what your probe shows for the post-resume
   remainder and name the finding in the report.

## Scope

MAY modify:
- `crates/rum-host/src/session.rs`
- `crates/rum-host/src/lib.rs` (re-exports/docs only, if needed)
- `crates/rum-host/src/tests.rs`
- `docs/guide/embedding.md`

MUST NOT touch: anything in `rum-runtime`/`rum-core`/`rum-syntax`/
`rum-stdlib` (zero kernel changes — the seams all exist),
`rum-cli`/`rum-repl`/`rum-terminal`/`rum-jupyter` (ports are follow-up
briefs), `HostFailureCode`/`HostTyped`/`HostEvalRequest` definitions,
`docs/**` beyond the one listed file.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

- **No microbench A/B**: additive host-layer methods only; no kernel
  code changes at all; state this in the report.

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- Additive only: no existing rum-host test may need its expected value
  changed. If one does, STOP and report.
- Stop-loss: if any needed behavior turns out to require a kernel-side
  change (e.g. the script-suspension status quo is unpinnable without
  one), STOP and report the finding instead of expanding scope.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Probed facts: the `HostFailureCode` for provider-less `eval_file`;
   the script-suspension status quo (suspension surfacing + post-resume
   remainder behavior).
3. Test survey — each new test/assertion, named.
4. Verification — the exact commands with verbatim output (test totals).
5. The no-A/B justification (one sentence).
6. Deviations / open questions (empty if none).
