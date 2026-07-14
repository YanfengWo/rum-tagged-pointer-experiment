# Brief 233 — stdout ownership: mechanical hard line + bounded print output

Implementation brief — host-embedding line, item b3 (see
`docs/dev/host-embedding-line.md`, concern (b) protocol ask 1). The
embedder must own the host process's stdout completely: in a protocol
host, stdout IS the wire, so a stray `Print` reaching it corrupts framing
and kills the process. The good news, established by the line's
diagnosis: **the kernel already writes zero bytes to stdout** — `Print`
goes to a per-request buffer (`push_print` → `prints`, surfaced as
`HostEvalSuccess.prints`), and env-gated profiling writes to stderr only.
This brief does not change that behavior; it **pins it mechanically** so
it cannot regress, and it locks down the one remaining sharp edge
(unbounded print accumulation).

This is hygiene work in the same family as the existing `std::fs` ban:
turn a property we currently hold by discipline into one the test suite
enforces.

## Current facts (verified 2026-07-15 — do not re-derive, just preserve)

- Kernel library code (`rum-core/src`, `rum-runtime/src`,
  `rum-stdlib/src`, excluding `/tests/` and `/bin/`) contains **zero**
  stdout writes. The only `println!` in those trees is
  `rum-stdlib/src/bin/generate-builtin-reference.rs` — a codegen tool
  binary whose stdout IS its product. It must stay allowed.
- `Print` → `Session::push_print` (`session/session_helpers.rs:176`):
  writes to the optional `print_sink` if one is installed, else pushes
  onto the `prints` buffer.
- `Session::set_print_sink` (`model/session.rs:103`) is a kernel API a
  host may use to redirect print output. `rum-cli` installs a
  stdout-writing sink (`rum-cli/src/main.rs:285`) — correct, because the
  CLI owns its stdout. **`rum-host` does not expose a sink at all**, so
  an embedded session's `Print` can only reach the `prints` buffer. That
  default is exactly right for a protocol host; keep it.
- `prints` is already counted by the memory proxy
  (`debug_tools/resources/memory.rs:190`), so brief-232's request-level
  ceiling ALREADY bounds print floods. That is a free property nobody
  tested — test it (change 2).

## Required change

1. **Hygiene test — the stdout ban.** In
   `crates/rum-runtime/src/runtime/tests/hygiene.rs`, add a second test
   next to `kernel_crates_do_not_use_std_fs_outside_tests`, mirroring its
   structure (same three roots, same recursive walk, same violation-list
   assertion). It fails if kernel library code writes to stdout.
   - Detect: `println!`, `print!`, `io::stdout`, `stdout()`.
   - **Exclusions**: paths containing `/tests/` (as today) AND paths
     containing `/bin/` (codegen tools own their stdout). State the
     `/bin/` exclusion in the assertion message.
   - **INSTRUMENT TRAP — read twice.** The string `"eprintln!"`
     CONTAINS the substring `"println!"`, and `"eprint!"` contains
     `"print!"`. A naive `contents.contains("println!")` therefore
     reports every env-gated profiling line in the tree as a violation
     (there are dozens — `evaluation_cache/profile/report.rs` alone has
     many). The check MUST require that the match is not preceded by an
     identifier character (`e`, or any `[A-Za-z0-9_]`) — i.e. do a
     manual scan for each needle and reject a hit whose preceding byte
     is alphanumeric or `_`. stderr writes are legitimate and must
     remain allowed.
   - **Self-calibrate the instrument (mandatory).** A hygiene test that
     cannot fail is worthless. Temporarily inject a `println!("x");`
     into one kernel library file, run ONLY this test, and paste the
     verbatim FAILURE output showing that file listed; then remove the
     injection and show the test passing on the clean tree. Both outputs
     go in the report. (Also confirm the run is clean while the tree
     still contains all the existing `eprintln!` profiling lines — that
     is the trap check.)
2. **Bounded print output — lock the free property.** Add a test proving
   a print flood under a request memory ceiling terminates with the
   classified outcome instead of growing without bound: with
   `eval_memory_limit = Some(<small>)`, evaluate a loop that prints
   repeatedly (e.g. `Do[Print["x"], {100000}]`), and assert the outcome
   is the in-band `TerminatedEvaluation["MemoryLimit"]` with
   `termination == Some(...::MemoryLimit)` and a `General::memlim`
   message. Put it where brief-232's ceiling tests live (runtime
   session-request tests and/or `rum-host/src/tests.rs` — your pick;
   name it in the report). If the flood does NOT trip the ceiling
   (e.g. because the prints buffer is not sampled often enough by the
   64-check stride), STOP and report the observed behavior with numbers
   — that is a finding, not something to force.
3. **Embedder contract — documentation.** Two doc edits, no behavior
   change:
   - `docs/guide/embedding.md`, in `## Messages, Prints, And Outputs`
     (line ~156): a short paragraph stating the contract — the kernel
     never writes to stdout or stderr on behalf of evaluated code;
     `Print` output is buffered per request and returned in `prints` /
     `outputs`; the embedder therefore owns its process's stdout
     completely, which is what makes rum usable as a host that speaks a
     framed protocol on stdout. Note that a host may opt into
     redirection with the kernel's `set_print_sink` (as `rum-cli` does
     to stream to its own stdout), and that print volume is bounded by
     the request memory ceiling
     (`HostResourceLimits::eval_memory_limit`, brief-232).
   - `Session::set_print_sink` rustdoc (`model/session.rs:103`): state
     that installing a sink is the host's explicit choice about where
     print output goes, that no sink means output is buffered into the
     request response, and that a host owning a protocol channel on
     stdout must not install a sink that writes to it.

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/tests/hygiene.rs` (the new test)
- `crates/rum-runtime/src/runtime/model/session.rs` (rustdoc on
  `set_print_sink` ONLY — no code change)
- ONE of: the runtime session-request test module, or
  `crates/rum-host/src/tests.rs` (the print-flood test; name it)
- `docs/guide/embedding.md` (the contract paragraph — this doc file is
  explicitly IN scope, overriding the template's default docs ban)

MUST NOT touch: `push_print`, `set_print_sink`/`clear_print_sink` code,
the `prints` buffer type or its memory accounting, `rum-cli`'s stdout
sink, the existing `std::fs` hygiene test, any builtin, anything else.
No new public API in this brief.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# the instrument self-calibration (change 1): the injected-violation
# FAILURE output, then the clean-tree PASS output
git status --short            # only MAY-listed files; injection removed
```

- **No microbench A/B**: this brief adds tests and documentation only;
  no evaluation-path code changes. State that.
- Probe note: `rum-cli` binds non-TTY stdin as `$ScriptInput`, so any
  probe you run through the binary must be given `< /dev/null` or it can
  block forever. (This bit the director's gate on brief-232.)

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- The hygiene test must pass on the CLEAN tree with all existing
  `eprintln!` profiling lines present. If it flags them, your needle
  matching is wrong — fix the matcher, do not touch the profiling code.
- Do not "fix" the codegen bin's `println!`; exclude `/bin/` instead.
- Behavior stays byte-identical: this brief changes no evaluation
  semantics and no rendered output.
- Stop-loss: if the print-flood test cannot trip the ceiling, report the
  numbers and stop — do not change the stride, the accounting, or the
  buffer to force it.

## Report format

1. What changed — per file, line ranges, one-line why.
2. The hygiene instrument: the needle-matching rule you used, and the
   verbatim FAIL (injected violation) + PASS (clean tree) outputs.
3. The print-flood test: name, the ceiling used, and what it asserts.
4. Verification — commands with verbatim outputs (test totals).
5. Deviations / open questions (empty if none).
