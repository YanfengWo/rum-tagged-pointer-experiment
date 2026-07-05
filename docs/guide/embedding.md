# Embedding

`rum-host` is the narrow Rust embedding facade.

Use it instead of depending directly on runtime internals.

The facade owns session construction, request entrypoints, structured outcomes,
input suspension, cancellation, timeout, reset, snapshots, and capability
policy.

The crate-level rustdoc is the full API reference. This chapter is the quick
start.

## Minimal Example

The repository includes a compile-checked example at
`crates/rum-host/examples/basic_embedding.rs`.

```rust
use rum_host::{HostEvalOutcome, HostSession};

fn main() {
    let mut session = HostSession::new();

    match session.eval("rule[x_Integer] := x + 1; rule[41]") {
        HostEvalOutcome::Success(success) => {
            println!("value: {}", success.value.text);
        }
        HostEvalOutcome::InputRequired(suspension) => {
            println!("input required: {}", suspension.request.prompt);
        }
        HostEvalOutcome::Failure(failure) => {
            eprintln!("error: {}", failure.message);
        }
    }
}
```

Run it from the workspace:

```text
$ cargo run -q -p rum-host --example basic_embedding
value: 42
```

`HostSession::new()` installs the core builtins with the default unrestricted
trusted-host configuration.

## Building A Session

Use `HostSession::new()` for the default configuration.

Use `HostSession::builder()` when you need to change configuration:

- tracing
- capability policy
- whether core builtins are installed
- the full `HostConfig`

`HostConfig` carries:

- `resource_limits`
- `capability_policy`
- `current_context`
- `context_path`
- `package_search_paths`
- `file_io`
- `tracing_enabled`
- `asserts_enabled`
- `install_core_builtins`

The default trusted configuration installs `StdFileIo`; sandboxed
configuration omits a file provider.

## Evaluating

`HostSession::eval(source)` returns `HostEvalOutcome`.

The variants are:

- `Success(HostEvalSuccess)`
- `InputRequired(HostInputSuspension)`
- `Failure(HostFailure)`

Do not treat input suspension as a failure.

It is a resumable state for `Input` and `InputString`.

## Success Values

`HostEvalSuccess` includes:

- `value`
- `messages`
- `prints`
- `trace`
- `outputs`

`value.text` is the rendered expression.

`value.typed` is a transport-friendly structured value:

- small integers
- big integers as decimal strings
- rationals as numerator and denominator strings
- symbols
- strings
- lists up to the typed depth cap
- other expressions as rendered text with a head name

There is no floating-point typed variant because the language boundary is exact
rationals only.

## Messages, Prints, And Outputs

Messages are exposed as `HostMessage` values with:

- `symbol`
- `tag`
- `text`

Print output is exposed as strings.

Structured outputs are a flattened adapter-friendly stream with kinds:

- result
- print
- message
- trace

Adapters can map these into their own transport without depending on runtime
internal structs.

## Failures

`HostFailure` includes:

- a coarse `HostFailureCode`
- a user-facing message
- messages, prints, trace, and structured outputs collected before failure

Failure codes include syntax failures, load failures, evaluation cycles,
recursion limits, match recursion limits, cancellation, timeout, resource
limits, panic, poisoned session, exit, and other.

Recoverable resource-limit outcomes often appear as successful values containing
`TerminatedEvaluation[...]` rather than `HostFailure`.

## Resumable Input

`Input` and `InputString` can return `HostEvalOutcome::InputRequired`.

The suspension contains:

- `request.kind`
- `request.prompt`
- `request.password`
- outputs collected before the suspension

Resume with `HostSession::resume_with_input(input)`.

The session keeps the continuation. The host should not replay the original
source to answer input.

`pending_input_request()` lets a host inspect whether the session is currently
waiting for input.

The host crate has a public contract test for this flow:

```text
$ cargo test -q -p rum-host public_contract_preserves_input_suspension_and_resume_boundaries -- --exact

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
```

## Reset

`HostSession::reset()` rebuilds the session from its original configuration.

It drops user definitions, attributes, contexts, loaded packages, messages, and
gensyms.

It also clears a poisoned session and invalidates old cancel handles.

Obtain a new `cancel_handle()` after reset.

## Snapshot And Restore

`snapshot_to_bytes()` serializes user definitions and attributes for defined
symbols to a UTF-8 source image.

`restore_from_bytes(bytes)` resets the session and evaluates that source image.

This is an in-memory text image, not a binary snapshot.

It does not capture attribute-only symbols, `$` variables, options, messages,
the gensym counter, or unrelated context state.

Use this when the host wants persistence without filesystem capability.

## Sandbox Configuration

Use `HostConfig::sandboxed()` for untrusted source.

It installs a `{Pure}` capability policy.

Use `{Pure, Clock}` when untrusted scripts need time builtins such as
`AbsoluteTime`, `Timing`, or `TimeConstrained`.

Construct the session through `HostSession::builder().with_config(...)` and
pass `HostConfig::sandboxed()`.

That API shape is the same one used by the host tests. The sandboxed policy
blocks file, clock, and diagnostic builtins at dispatch, including
runtime-constructed calls.

## Cancel And Timeout

Use `cancel_handle()` to cancel from another thread.

Use `eval_with_timeout(source, duration)` for a per-evaluation wall-clock
deadline.

Both are cooperative checks. The session remains usable after normal
recoverable cancellation and timeout outcomes.

## Threading Contract

`HostSession` is `Send`.

Move one session to one worker thread.

It is intentionally not a shared mutable object. Do not share one session
concurrently across threads.

Use one session per worker when you need parallel evaluation.

`HostCancelHandle` is cloneable and can cross threads.

## Editor Helpers

The facade also exposes parser, completion, inspection, and completeness
requests:

- `parse`
- `inspect`
- `inspect_symbol`
- `complete`
- `is_complete`

Those helpers are for adapters and editors.

They do not expose matcher IR, planner internals, or mutable evaluator state.
