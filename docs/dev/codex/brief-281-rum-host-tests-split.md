# Brief 281 — split `rum-host/src/tests.rs` by responsibility

`crates/rum-host/src/tests.rs` is **68,639 bytes** — the only file in
the workspace over AGENTS.md's `35k` architecture-debt line, with no
documented exemption, and roughly twice that line. User ruling
2026-07-20: split it.

Unlike brief-280's attribute table (a flat lookup table with no
responsibility boundary, split along its name ordering), this file has
**real boundaries**: its 77 tests fall into five clusters that mirror
the embedding-line increments a1-a5. Split by responsibility, not by
size or alphabet.

## Current shape (verified, 2026-07-20)

- `lib.rs:74-75` declares `#[cfg(test)] mod tests;`. **This resolves to
  either `tests.rs` or `tests/mod.rs`, so `lib.rs` does NOT need to
  change.** Do not touch it.
- Lines 1-266: shared helpers — `LONG_RUNNING_EVAL`, `TWO_TO_200`,
  `host_session_with_large_iteration_budget`, `TempDirGuard` (+ its
  `impl`s incl. `Drop`), `successful_eval`, `failed_eval`, `rum_path`,
  `wl_string`, `byte_array_payload`, `string_payload`,
  `assert_sandbox_denied`, `assert_arity_not_sandbox_denial`,
  `ALL_CAPABILITIES`, `ExpectedProfileSurface`, `profile_allows`,
  `capability_probe_source`, `assert_profile_capability_surface`,
  `assert_profile_providers`.
- Line 267 onward: 77 `#[test]` functions.
- `crates/rum-host/src/lifecycle_contract_tests.rs` is a SEPARATE
  `#[cfg(test)] mod` that does NOT use any of these helpers. Leave it
  alone.

## Target shape

Keep the module path `crate::tests`. Turn it into a directory module
(`tests/mod.rs`, or `tests.rs` + `tests/` — either idiom, both exist in
this repo). The parent must end up containing ONLY `mod` declarations
(plus `use` lines if the idiom needs them) — no tests, no helpers.

Children, with the tests that belong to each:

**`common.rs`** — the shared helpers. A helper used by only ONE child
should move into that child instead; the profile-surface group
(`ALL_CAPABILITIES`, `ExpectedProfileSurface`, `profile_allows`,
`capability_probe_source`, `assert_profile_capability_surface`,
`assert_profile_providers`) is very likely single-child — check before
placing. Children reach shared helpers via `use super::common::*;`
(add `#[allow(unused_imports)]` only if a glob genuinely warns).

**`capability_profiles.rs`** — the capability/profile surface (~20):
`capability_policy_*` (3), `clock_policy_allows_composed_datetime_current_readers`,
`default_host_config_*` (4), `hermetic_profile_*` (2),
`sandboxed_host_config_*` (10), `sandboxed_profile_pins_*`,
`scripting_profile_pins_*`, `permissive_policy_*` (2),
`random_policy_allows_entropy_reseed`,
`trusted_host_file_names_uses_the_std_file_io_provider`.

**`typed_bridge.rs`** — typed call/eval data views (~15): all
`host_call_*` (11), `host_eval_exposes_*` (3),
`host_eval_keeps_delayed_association_out_of_typed_data_view`.

**`session_lifecycle.rs`** — session state and isolation (~12):
`host_session_reset_*` (3), `host_session_snapshot_*` (3),
`host_session_introspection_*` (2), `host_boundary_*` (2),
`host_session_and_cancel_handle_are_send`,
`parallel_host_sessions_move_to_threads_and_isolate_definitions`,
`host_reset_removes_print_sink_and_restores_buffering`.

**`eval_contract.rs`** — evaluation outcomes, termination, failures
(~12): `host_eval_*termination*` / `*recursion_limit*` /
`*memory_ceiling*` / `*ordinary_value*` (4), `host_eval_script_*` (2),
`host_eval_with_timeout_*`, `host_eval_with_unused_handle_*`,
`host_cancel_handle_interrupts_running_eval_from_another_thread`,
`host_failure_codes_report_session_reset_requirement`,
`host_failures_use_stable_failure_codes`,
`host_eval_file_uses_provider_and_missing_provider_is_other`.

**`editor_surface.rs`** — editor/IO-facing surface (~8):
`host_parse_inspect_complete_and_is_complete_cover_editor_flows`,
`host_lower_dump_returns_canonical_text_and_rendered_syntax_failures`,
`host_print_sink_streams_exclusively_and_clear_restores_buffering`,
`host_message_catalog_supports_global_symbol_and_invalid_queries`,
`host_processes_inline_multiline_csv_data`,
`host_evaluates_multiline_string_literals_as_ordinary_strings`,
`host_suspend_resume_flow_wraps_input_requests`,
`host_queued_input_completes_without_suspension`,
`host_eval_reports_value_messages_prints_and_structured_outputs`,
`host_eval_renders_top_level_hold_form_transparently_in_canonical_style`.

These counts are approximate and the clusters are a judgment call at
the margins. **If a test reads as a better fit elsewhere, move it and
say so in the report** — the boundary matters more than my exact
assignment. What is NOT negotiable: every test lands in exactly one
child, and no test is dropped.

## Hard constraint: this is a pure move

- Do NOT rename any test function.
- Do NOT edit any test body, assertion, or helper body.
- Do NOT add, delete, merge, or split any test.
- Do NOT change any `use` semantics beyond what the move mechanically
  requires (`use super::*` → `use super::super::*` / `use super::common::*`
  and the like).

If a test fails to compile after moving because it used a helper you
placed in a different child, move the helper to `common.rs` — do not
duplicate it.

## Proof that nothing changed

1. BEFORE any edit, on `HEAD`, run and save:
   ```
   cargo test -p rum-host --lib -- --list > /tmp/brief281_tests_base.txt
   ```
2. After the split, run the same into `/tmp/brief281_tests_cand.txt`.
3. The two must contain the SAME SET of test names. Full-path names
   will differ (`tests::foo` → `tests::typed_bridge::foo`), so compare
   the LEAF names:
   ```
   sed 's/.*:://' /tmp/brief281_tests_base.txt | sort > /tmp/b.txt
   sed 's/.*:://' /tmp/brief281_tests_cand.txt | sort > /tmp/c.txt
   diff /tmp/b.txt /tmp/c.txt
   ```
   That diff MUST be empty. Paste the commands and the (empty) output,
   plus the test count from each list.

## Scope

- Files you MAY modify:
  - `crates/rum-host/src/tests.rs` (becomes the parent shell, or is
    replaced by `tests/mod.rs`);
  - NEW `crates/rum-host/src/tests/*.rs`.
- Files/areas you MUST NOT touch: `crates/rum-host/src/lib.rs` (the
  `mod tests;` declaration already resolves either way),
  `lifecycle_contract_tests.rs`, `crates/rum-host/tests/**` (the
  integration tests are a different tree), any non-test source in
  `rum-host`, any other crate, `TODO.md`, `PROGRESS.md`, `AGENTS.md`,
  `.git`, `Cargo.lock`. If you believe you must, STOP and report.
- No new `unsafe`. No production-code change of any kind — if a test
  move appears to require touching `session.rs`/`config.rs`/etc., STOP
  and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Baseline is `1654 passed / 0 failed` on `a8ea4f2`; state your totals.
The known sandbox caveat applies: loopback-bind ZeroMQ tests may fail
under the sandbox — if so, run and paste the two supplemental splits
(`--workspace --exclude rum-jupyter`; `-p rum-jupyter --lib -- --skip
zeromq::tests::native`) and say so. Claude re-runs the authoritative
gate outside the sandbox.

Report the final byte size of every file created or modified
(`ls -l`), and confirm each is under `20k`.

No microbench is required: test-only change, zero production code
touched.

## Stop-loss

- A test cannot be compiled in any child without editing its body →
  STOP, report which test and why.
- The leaf-name diff is non-empty and you cannot make it empty without
  adding or removing a test → STOP.
- The split requires touching a MUST-NOT file → STOP.

Revert everything and report. (`git checkout --` fails under the
sandbox with `Unable to create '.git/index.lock': Operation not
permitted` — restore by scoped patch instead and say so.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.

## Report format

1. What you changed, file by file, with `ls -l` sizes and the test
   count per child.
2. Verbatim verification output (fmt, test totals, the empty leaf-name
   diff).
3. Any test you assigned to a different cluster than this brief
   suggested, and why.
4. Stop-loss / open questions.
