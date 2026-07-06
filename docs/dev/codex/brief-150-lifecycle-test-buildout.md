# Brief 150 — Host-lifecycle test buildout (implement the brief-149 hole list)

Implementation brief, TEST-ONLY: zero production-code changes. Implements
the measured hole list from the brief-149 diagnostic (baseline: region
79.00%, line 82.30%; artifacts under `/tmp/rum-149-*` if still present —
the authoritative list is embedded below). If any new test REVEALS a
production bug, STOP on that item, mark the test `#[ignore]` with a
`// BUG:` comment, and report it — do not fix production code in this
brief.

## P0 targets (all mandatory)

1. `session_request_error_from_eval_error_with_source`
   (session_requests.rs) — force every limit/cancel/io/no-provider/syntax
   branch through request-level tests; assert each `SessionRequestError`
   variant.
2. `failure_code` (rum-host outcome.rs) — a failure-code table test:
   recursion, stack, match-depth, generated/integer/iteration-work,
   replace-repeated, exit, input, other → each `HostFailureCode`.
3. `snapshot_input_failure` (rum-host session.rs) — restore a snapshot
   that immediately requires input; assert the synthesized failure.
4. `panic_is_complete_response` — panic through the completeness boundary;
   assert poison + invalid response.
5. Iterator spec matrix (`parse_do_spec_iteration_count`,
   `expr_as_signed_bigint`): Do/Table/Sum/Product with single count,
   `{i, vals}`, `{i, end}`, `{i, start, end, step}`, negative bounds via
   `Times[-1, n]`, zero-step and non-integer rejections.
6. `generated_list_memory_proxy_bytes` — a `MemoryConstrained[Table[...]]`
   case tripping the projected-size path before materialization.
7. The suspension continuation round-trips (ALL currently 0%):
   `PositionCollectionOutcome` into/from, `TraversalCountOutcome`
   into/from, `Vec<SymbolExprMap>` into/from (matcher bindings, incl.
   multiple bindings), `Option<CasesLevelSpec>` from,
   `Option<Vec<RewriteRuleInfo>>` from, `Option<Vec<Expr>>` from — each
   via a REAL suspend (Input[] inside the relevant builtin) → snapshot the
   continuation → resume → assert the completed result equals the
   uninterrupted run. These are the highest-risk items: never-executed
   deserializers. Expect to find bugs; follow the STOP rule per item.
8. `take_pending_input_request`, `set_context_path` (normalization +
   lookup), `normalize_path` (rum-host StdFileIo: existing file,
   directory probe → non-file, missing relative, canonicalize fallback).

## P1 targets (all mandatory)

- `HostConfig::with_file_io` / `with_entropy_source` (custom in-memory
  providers: assert Get/Needs reads through the injected provider; assert
  a fixed-bytes entropy source makes `SeedRandom[]`+draws deterministic),
  `with_tracing`, `with_asserts_enabled`.
- `HostSessionBuilder::with_capability_policy` (denied file+random policy
  outcomes), `with_core_builtins(false)` (a core builtin stays inert).
- `Session::set_resource_limits` / `set_capability_policy` /
  `enable_asserts` / `seed_random_with_bytes` (empty/short/long/high-byte
  seeds; contrast with integer seeding) / `random_u64` (pinned sequence).
- `FileProbe::non_file` via a directory probe; `FileIoProvider::from_arc`
  + `EntropyProvider::from_arc` (shared-Arc behavior + equality).

## P2 (cheap ones only)

Capability builder allow/deny matrix (covers builder plumbing en route);
`StdFileIo` error conversion via a failing read/write; the two host
request `From<String>` adapters; builder/session `default()`s; accessor
trios (`trace`/`messages`/`prints`). SKIP: Display/Debug impls and the
judged-unreachable `entropy_error` — record skips in the report.

## Placement

Host-boundary tests → `crates/rum-host/tests/lifecycle_contract.rs` (or a
new sibling integration file if it crowds); runtime API tests → the
`crates/rum-runtime/src/runtime/tests/` tree; iterator/limit matrices →
wherever the existing Do/Table limit tests live (stdlib part5/runtime
arithmetic — follow the neighbors). Follow existing naming/style norms.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# Coverage delta on the focus files (same tool, one bounded run):
scripts/run-bounded.py --timeout 1800 -- cargo llvm-cov --workspace --json --output-path /tmp/rum-150-cov.json
# paste a before/after per-file table for: rum-host/src/session.rs, config.rs,
# model/outcome.rs, file_io.rs; rum-runtime session_api.rs, session_requests.rs,
# file_io.rs, random.rs, capabilities.rs, suspension/*.rs, iteration/spec.rs
# (before numbers come from the brief-149 report, quoted verbatim)
```

The gate is the HOLE LIST: every P0/P1 item either covered (function no
longer 0%/thin) or reported as a discovered bug with an `#[ignore]`d test.
No percentage targets.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- TEST-ONLY: zero changes under any `src/` outside test modules/files
  (test-support helpers inside existing test trees are fine). If a test
  cannot be written without a production change, report it as a finding.
- Bugs found → `#[ignore]` + `// BUG:` + report; never adjust a test to
  mask one.
- No new `unsafe`, no new dependencies.
- Stop-loss: same failure twice on one item → mark it, move on, report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. Checklist disposition — every P0/P1/P2 item: covered / bug-found /
   skipped-with-reason.
2. Discovered bugs, each with the exact failing input and observed vs
   expected behavior (this section matters most).
3. Verification — commands + the before/after focus-file table.
4. Stop-loss / open questions.
