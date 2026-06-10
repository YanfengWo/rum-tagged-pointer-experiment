# Jupyter Host Migration Audit

Audit date: 2026-06-10.

## Decision

For `v0.1`, `rum-jupyter` should route kernel-facing request handling through
`rum-host` and provide a basic Notebook tech preview.

This does not make Jupyter the product. The product boundary remains the
symbolic kernel plus stable host-facing APIs. Jupyter is a protocol adapter and
contract validator.

## Current State

- `cargo test -p rum-jupyter` currently passes in the local checkout.
- During this audit, the default manual-smoke environment documented for
  Jupyter was not present: `~/.venv-jupyter/bin/python` was not found.
- `rum-jupyter` still depends directly on `rum-runtime` and `rum-stdlib`.
- `rum-jupyter` currently exposes some `rum-runtime` types through its public
  adapter model, so the migration is an API cleanup as well as an internal
  implementation change.

## Direct Runtime Couplings

Current runtime couplings found during audit:

- `crates/rum-jupyter/Cargo.toml`
  - depends on `rum-runtime`
  - depends on `rum-stdlib`
- `crates/rum-jupyter/src/model.rs`
  - exposes `SessionEvalResponse` through `KernelExecuteOutcome::Success`
  - exposes `SessionRequestFailure` through `KernelExecuteOutcome::Failure`
  - exposes `SessionInspectResponse` through `KernelInspectResponse::result`
- `crates/rum-jupyter/src/adapter/mod.rs`
  - stores `rum_runtime::Session` inside `KernelAdapter`
  - constructs adapters from `Session`
  - exposes `session()` and `session_mut()` accessors
  - calls `eval_request`, `resume_eval_request_with_input`,
    `inspect_request`, `complete_request`, and `is_complete_request`
  - maps `SessionCompletenessStatus` directly
- `crates/rum-jupyter/src/adapter/render.rs`
  - renders `RuntimeMessage`, `RuntimeInputRequest`, `SessionRequestFailure`,
    `SessionRequestError`, `SessionInspectResponse`, `TraceEvent`, and
    `RuleApplicationSource`
- `crates/rum-jupyter/src/bootstrap.rs`
  - creates event pumps from `rum_runtime::Session`
- `crates/rum-jupyter/src/socket.rs`
  - creates socket drivers from `rum_runtime::Session`
- `crates/rum-jupyter/src/zeromq.rs`
  - creates event pumps and socket drivers from `rum_runtime::Session`
- `crates/rum-jupyter/src/launcher.rs`
  - constructs `Session::new()`
  - calls `rum_stdlib::install_core`
- `crates/rum-jupyter/src/tests/**` and native/fake ZeroMQ tests
  - construct raw runtime sessions directly
  - assert runtime-specific failure shapes in stdin and render paths

## Host Coverage Already Available

`rum-host` already covers the main flows needed for this migration:

- `HostSessionBuilder` and `HostSession`
- default core-builtin installation through the host builder
- `HostSession::eval`
- `HostSession::resume_with_input`
- `HostSession::inspect`
- `HostSession::complete`
- `HostSession::is_complete`
- `HostSession::pending_input_request`
- `HostEvalOutcome::Success`
- `HostEvalOutcome::InputRequired`
- `HostEvalOutcome::Failure`
- host-facing messages, prints, trace events, failures, input requests, and
  structured text outputs

The migration therefore does not need a new host session concept. It mostly
needs Jupyter-side type mapping and constructor rewiring.

## Migration Work

Recommended implementation order:

1. Add a `rum-host` dependency to `rum-jupyter`.
2. Replace `KernelAdapter { session: Session }` with `HostSession`.
3. Replace `KernelAdapter::new(Session)` and `with_username(Session, ...)`
   with host-session constructors. If test ergonomics need it, keep a
   `KernelAdapter::new(HostSession)` shape and add a
   `KernelAdapter::default_host()` helper.
4. Change `bootstrap`, `socket`, `zeromq`, launcher, and tests to pass
   `HostSession` or to let the adapter construct the default host session.
5. Remove direct `rum_stdlib::install_core` from the Jupyter launcher; host
   session construction should own default builtin installation.
6. Replace `KernelExecuteOutcome::Success(SessionEvalResponse)` and
   `Failure(SessionRequestFailure)` with host-facing success, input-required,
   and failure cases.
7. Map `HostEvalOutcome::InputRequired` to stdin requests instead of treating
   input suspension as a runtime failure.
8. Render messages, prints, traces, failures, and text results from host
   facades.
9. Replace `KernelInspectResponse::result: Option<SessionInspectResponse>`
   with either `HostInspectResponse` or a Jupyter-owned inspect summary.
10. Route inspect, complete, and is-complete through host requests.
11. Remove `session()` and `session_mut()` from the public Jupyter adapter API,
    or replace them with narrower host-facing accessors only if tests or
    adapter users truly need them.
12. Update tests so `rum-jupyter` no longer constructs raw runtime sessions in
    ordinary adapter, pump, wire, bootstrap, socket, stdin, or ZeroMQ paths.
13. Add a guard test or dependency audit that fails if public `rum-jupyter`
    model types reintroduce `rum-runtime` response/failure/inspection types.
14. Provide one documented real-Notebook deploy path and keep other frontend
    installation styles out of scope.

## Host Gaps And Decisions

Open decisions before or during implementation:

- `HostStructuredOutput` is text-only. That is enough for the current
  `v0.1` basic Notebook target unless rich display is deliberately promoted.
- `HostTraceEvent::RuleApplied.source` is a string instead of the runtime
  `RuleApplicationSource` enum. Jupyter should render the string for now.
- `HostFailure` carries stable coarse failure codes plus a rendered message.
  Jupyter should map those codes to `ename` values instead of matching every
  runtime error variant.
- `HostInspectResponse` already contains target, found, symbol info,
  definitions, and message catalog data. Prefer using it directly before
  inventing a separate inspect payload.
- Interrupt/cancel should remain a Jupyter lifecycle flag for `v0.1` unless a
  real host-level cancellation contract is added.

## Basic Notebook Acceptance

The `v0.1` Notebook tech preview is considered sufficient when a real Notebook
frontend can use one documented deploy path for:

- `kernel_info`
- code cell execution
- `text/plain` execute results
- runtime messages and prints
- failures as Jupyter error replies and IOPub error outputs
- inspect requests
- completion requests
- is-complete requests
- stdin request/reply resume
- shutdown
- graceful replies for common unsupported messages

## Out Of Scope For `v0.1`

- robust interrupt/cancel semantics
- widgets and comm messages
- debug protocol
- graphics and broad rich display
- update-display and clear-output behavior
- broad frontend compatibility matrix
- global kernelspec installation policy beyond one documented deploy path
- Python adapter revival
