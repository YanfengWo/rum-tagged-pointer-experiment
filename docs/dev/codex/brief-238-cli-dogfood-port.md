# Brief 238 — rum-cli dogfood port onto the rum-host contract (+ two contract gaps it exposed)

Implementation brief — embedding API line, item a2b (see
`docs/dev/embedding-api-line.md`; dogfood ruling: in-tree trusted hosts
ride the contract). Port `rum-cli`'s non-interactive paths from raw
`rum_runtime::Session` onto `HostSession`, byte-preserving all CLI
output (the 27 `tests/cli.rs` tests pin it — their expectations MUST NOT
change). The port survey exposed two real contract gaps; fix them first
(part A), then port (part B). **Microbench A/B is a hard gate** (part C)
— `rum-cli` is the perf instrument.

## Pre-diagnosed facts (verified this session)

- `HostConfig` already carries providers: public `file_io`/
  `entropy_source` fields + `with_file_io`/`with_entropy_source`
  (`rum-host/src/config.rs:128-151,225-243`). No gap there.
- Message rendering is reconstructible byte-for-byte:
  `RuntimeMessage` Display is `{symbol}::{tag}: {text}`
  (`runtime/model/messages.rs:106-109`); `HostMessage` carries all
  three fields; `rum-terminal`'s line format is `! {Display}`.
- GAP 1 — value rendering: `eval_success_from_runtime` renders
  `value.text` via `Display` (`rum-host/src/model/outcome.rs:347`),
  but the CLI renders via
  `render_to_string_expr(&value, CanonicalHoldFormTransparent)`
  (`rum-terminal/src/lib.rs:51-56`). The exact difference
  (`runtime/kernel_data/text_rendering.rs:25-29`):
  CanonicalHoldFormTransparent = Display with ONE top-level `HoldForm`
  peeled. Without the fix, a CLI on the contract would print
  `HoldForm[x]` where it must print `x`.
- GAP 2 — syntax diagnostics: `SessionRequestError::Syntax` Display
  writes only `diagnostic.message` (single-line summary,
  `runtime/model/requests.rs:6`), but the CLI's stderr prints the full
  `diagnostic.rendered` (`rum-terminal` `render_request_failure`).
  `HostFailure.message` therefore loses the rendered diagnostic — a
  contract host cannot reproduce the CLI's syntax stderr.
- `--parse`/`--format` need no session at all: `rum-syntax` exposes
  `parse_with_diagnostics` (`rum-syntax/src/document.rs:326`) and
  `ParsedSource::debug_dump`; the CLI already imports
  `format_parsed_source`/`parse_format_cli_args` from `rum_syntax`.

## Part A — contract gaps (rum-host)

1. **`HostValue.text` becomes display-rendered.** In
   `eval_success_from_runtime`, render `text` with
   `render_to_string_expr(&response.value,
   ToStringFormat::CanonicalHoldFormTransparent)` (both are exported
   via `rum_runtime::kernel_api`). Update the `HostValue.text` rustdoc:
   canonical-rendered with one top-level `HoldForm` peeled, matching
   what the CLI front end shows; `HostTyped` keeps the structural view.
   REVISION (2026-07-16, after the first dispatch's correct stop-loss):
   the renderer peels `HoldForm` and then uses the CANONICAL
   `Expr::to_string()` style — for `HoldForm[1+1]` that is
   `Plus[1, 1]`, NOT infix `1 + 1`. The ONLY output this may change
   anywhere is a top-level-`HoldForm` success value (e.g. jupyter's
   text/plain for `HoldForm[1+1]` goes `HoldForm[Plus[1, 1]]` →
   `Plus[1, 1]`; that direction is the fix). If any existing test
   expectation changes for any OTHER reason, STOP.
2. **`HostFailure.syntax_rendered: Option<String>`** — documented new
   field: `Some(diagnostic.rendered)` iff the failure maps from
   `SessionRequestError::Syntax` (populate at the
   `host_failure_from_runtime` seam; the diagnostic is on the error
   variant), `None` everywhere else. MECHANICAL literal ripple, same
   16-site set as brief-236: `outcome.rs` (5), `rum-host/src/session.rs`
   (5), `rum-jupyter/src/adapter/render.rs` (2), `worker.rs` (4), plus
   any test literals (list each).
3. Tests (`rum-host/src/tests.rs`): a top-level `HoldForm[1+1]` eval has
   `value.text == "Plus[1, 1]"` (canonical style, `HoldForm` peeled —
   matching the frozen CLI expectation at
   `rum-cli/tests/cli.rs:191-202`); a syntax failure carries
   `syntax_rendered == Some(...)` matching the runtime diagnostic's
   `rendered`, and a non-syntax failure carries `None`.

## Part B — the CLI port (`crates/rum-cli/src/main.rs`)

Replace the raw-`Session` non-interactive paths with `HostSession`:

1. **Construction**: `HostSession::builder().with_config(
   HostConfig::default()
     .with_file_io(StdFileIo::provider())
     .with_entropy_source(StdEntropySource::provider())).build()`.
   The REPL branch (`run_repl`) KEEPS the raw `Session` construction
   for now — `rum_repl::Repl::run` takes `&mut Session`; unifying it is
   a2c's job. Mark the dual construction with a short comment naming
   a2c. `install_core` for the host path comes from the config default
   (`install_core_builtins: true`).
2. **Eval paths**: `--eval` → `eval`; stdin-filter and `-` →
   `eval_script`; `--file`/path → `eval_file`. Success output =
   `value.text` (Part A makes it byte-identical to today's
   `render_eval_value`); messages → stderr as
   `! {symbol}::{tag}: {text}` per line (byte-identical to
   `render_runtime_message`); failure stderr: syntax failures print
   `syntax_rendered`, others print `! {message}`, preceded by the
   failure's `messages` lines — mirror `render_request_failure`'s
   composition exactly.
3. **Streaming prints**: `HostSession::set_print_sink` with the same
   EPIPE closure; the interrupt store becomes
   `cancel_handle()`'s flag (grab the handle before installing the
   sink; the CLI never resets, so handle invalidation is moot).
4. **Script input**: `queue_input_response` (contract) replaces
   `session.queue_input_response`; the `$ScriptInput`/`$ScriptArgs`
   binding evals stay as they are (they are ordinary `eval` calls).
5. **Exit codes**: a failure with `code == HostFailureCode::Exit` exits
   with `exit_status` (a1's field) — drop the
   `SessionRequestError::Exit` matching. Other failures keep today's
   exit-code behavior (documented in `print_usage`: 1 on
   syntax/load/eval failure).
6. **`--parse`/`--format`**: drop the session entirely — call
   `rum_syntax::parse_with_diagnostics` directly; `--parse` prints
   `debug_dump()`, `--format` prints `format_parsed_source`; parse
   failures print the diagnostic's `rendered` to stderr as today.
7. Imports: the non-REPL paths should no longer name
   `rum_runtime::Session*` types except where the REPL branch and
   `exit_code`/failure plumbing genuinely need them — the goal is that
   evaluation flows ONLY through `rum-host`. Do not remove the
   `rum-runtime` dependency (REPL branch still needs it).

Byte-preservation gate: all 27 `tests/cli.rs` tests pass UNCHANGED. If
any CLI output cannot be reproduced through the contract, STOP and
report the missing datum — that is a new contract gap for the director,
not something to patch around.

## Part C — microbench A/B (HARD GATE)

`rum-cli` is the measurement vehicle. After the port builds green:

```
scripts/core-language-microbench.sh   # baseline = pre-port HEAD, candidate = ported tree
```

- BASELINES ALREADY EXIST (first dispatch ran them before its
  stop-loss): `/tmp/rum-brief-238-a2b/baseline-round-1/timings.tsv`,
  `/tmp/rum-brief-238-a2b/baseline-round-2/timings.tsv`, preserved
  baseline binary `/tmp/rum-brief-238-a2b/baseline-rum-cli`
  (SHA-256 `e0296c99904257ee493a7bf14898294dd6aadf2e61cd86a4cc7811588e514c88`).
  Verify the checksum and reuse them if intact; re-run baselines only
  if missing or the checksum mismatches.
- Run TWO candidate rounds. Persist raw `timings.tsv` per run and
  paste the per-label `rum_seconds` rows for every label in the
  report, flagging any label whose candidate-vs-baseline delta exceeds
  the spread between the two baseline runs (the noise floor).
- Expectation: the port adds one guard/outcome-mapping layer per
  PROCESS (start-up), not per evaluation step — labels should move
  within noise. If any label regresses beyond the baseline-vs-baseline
  noise floor in BOTH rounds, STOP and report (do not optimize).
- Profiling env vars OFF; no other load on the machine claims — state
  how you ran it.

## Scope

MAY modify:
- `crates/rum-host/src/model/outcome.rs`
- `crates/rum-host/src/session.rs` (MECHANICAL literal ripple)
- `crates/rum-host/src/lib.rs` (docs/re-exports only, if needed)
- `crates/rum-host/src/tests.rs`
- `crates/rum-host/src/lifecycle_contract_tests.rs`,
  `crates/rum-host/tests/*.rs` (MECHANICAL only)
- `crates/rum-jupyter/src/adapter/worker.rs`,
  `crates/rum-jupyter/src/adapter/render.rs` (MECHANICAL literal ripple
  only)
- `rum-jupyter` test files ONLY if a pinned expectation is exactly the
  top-level-`HoldForm` rendering change (list each; anything else =
  STOP)
- `crates/rum-cli/src/main.rs`
- `crates/rum-cli/Cargo.toml` (only if an import demands it; do not
  drop rum-runtime)
- `crates/rum-cli/tests/cli.rs` (ADDITIVE tests only; existing
  expectations frozen)
- `docs/guide/embedding.md` (value-rendering + syntax_rendered notes)

MUST NOT touch: anything in `rum-runtime`/`rum-core`/`rum-syntax`/
`rum-stdlib`, `rum-repl`/`rum-terminal` (a2c), `HostFailureCode`
variants, `HostTyped`, the alloc-count feature / global allocator
setup in `rum-cli`, `docs/**` beyond the listed file.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

Plus the Part C A/B evidence (raw tsv paths + per-label rows + noise
assessment).

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- The 27 existing CLI test expectations are FROZEN. A needed
  expectation change anywhere outside the exact top-level-`HoldForm`
  case means STOP and report.
- Stop-loss: any output irreproducible through the contract, any
  needed kernel change, or a reproducible A/B regression → STOP,
  report, leave the tree in its current state.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Byte-preservation survey: how each CLI output path maps to contract
   data (value/messages/syntax/exit), and confirmation no `tests/cli.rs`
   expectation changed.
3. Any expectation changes from the top-level-`HoldForm` rendering fix
   (each listed with old → new).
4. Verification — commands with verbatim output (test totals).
5. Part C A/B — raw file paths, per-label rows, noise-floor analysis,
   your verdict.
6. Deviations / open questions (empty if none).
