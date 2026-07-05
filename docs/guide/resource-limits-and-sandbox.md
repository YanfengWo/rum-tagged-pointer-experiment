# Resource Limits And Sandbox

The runtime is designed to be embedded by a caller that chooses limits and
capabilities.

Limits bound work.

Capabilities bound access to builtin families.

The host hardening document gives the implementation background:
[host embedding and hardening](../host-embedding-and-hardening.md).

## Caller-Set Limits

`HostResourceLimits` exposes these caller-set budgets:

- `recursion_limit`
- `replace_repeated_limit`
- `evaluation_pass_limit`
- `generated_element_limit`
- `integer_work_limit`
- `iteration_work_limit`

The parser also has a source nesting depth guard.

The orderless sequence matcher has a match-depth guard.

The default CLI session uses the runtime defaults.

## Generated Elements

Generated-element limits apply to builtins that would construct too many
elements.

```text
$ cargo run -q -p rum-cli -- --eval 'Range[1000000000000]'
TerminatedEvaluation["GeneratedElementLimit"]
! General::reslim: evaluation exceeded generated element limit (100000)
```

The value identifies the reason.

The message includes the configured limit.

## Iteration Work

Iteration-work limits apply to loops and traversal-style work.

```text
$ cargo run -q -p rum-cli -- --eval 'Do[x = 1, 100000000]'
TerminatedEvaluation["IterationWorkLimit"]
! General::reslim: evaluation exceeded iteration work limit (100000)
```

The session remains usable after a recoverable terminated evaluation.

## Integer Work

Integer-work limits apply to selected arithmetic and number-theory loops.

For example, large prime-index work can trip the budget:

```text
$ cargo run -q -p rum-cli -- --eval 'Prime[1000000000]'
TerminatedEvaluation["IntegerWorkLimit"]
! General::reslim: evaluation exceeded integer work limit (100000)
```

The exact builtin coverage is in the standard library and reference limit
notes.

## Recursion And Pass Limits

`recursion_limit` guards recursive evaluator re-entry.

`evaluation_pass_limit` guards top-level fixed-point evaluation.

An evaluation cycle in the fixed-point loop is reported separately, but the
recoverable top-level value reason is currently `IterationLimit`.

Use these limits for recursive definitions, generated rewrite systems, and
hosted workloads that should not monopolize a session.

## ReplaceRepeated Limit

`replace_repeated_limit` bounds `ReplaceRepeated` and the `//.` sugar.

If repeated replacement does not stabilize within the limit, the runtime
reports an iteration-limit condition.

This is separate from traversal work limits.

## Parse Depth

The parser has `MAX_PARSE_DEPTH = 128`.

That protects source-evaluation paths from pathologically nested input.

Parse-depth overflow is a syntax failure, not a normal evaluated expression.

The CLI reports it as an error with a source span.

## Match Depth

The current orderless sequence matcher has `MAX_MATCH_DEPTH = 512`.

When the match-depth guard trips through a recoverable top-level evaluation,
the terminated reason is `PatternMatchRecursionLimit` and the message tag is
`General::matchreclim`.

The patterns chapter lists known `Flat` plus `Orderless` plus `Optional`
corners.

## Interrupt And Cancel

`rum-host` exposes `HostSession::cancel_handle()`.

The host obtains a handle before starting evaluation and can call `cancel()`
from another thread.

The evaluator checks the cancel flag at cooperative poll points.

A cancelled evaluation returns `TerminatedEvaluation["Cancelled"]` on the
recoverable value path.

The handle is invalidated by `reset()`.

## Wall-Clock Timeout

`HostSession::eval_with_timeout(source, duration)` installs a per-evaluation
deadline.

When the deadline is exceeded, the recoverable value path returns
`TerminatedEvaluation["TimeConstraint"]`.

The same mechanism is also used by timeout-aware host adapters.

## Capability Classes

Every builtin is classified by capability in the generated reference.

The current capability classes are:

- `Pure`
- `FileRead`
- `FileWrite`
- `Diagnostic`

Most language and rewrite work is `Pure`.

Filesystem readers such as `Get`, `Needs`, `FindFile`, `FileNames`,
`FileExistsQ`, and `Import` are `FileRead`.

Filesystem writers such as `Save`, `DumpSave`, and `FileTemplateApply` are
`FileWrite`.

Instrumentation and resource-disclosure builtins such as `RumProfileSnapshot`,
`RumProfileReset`, `MemoryInUse`, and `MaxMemoryUsed` are `Diagnostic`.

## Sandboxed Host Configuration

`HostConfig::sandboxed()` installs a `{Pure}` policy.

That denies every non-pure builtin at dispatch.

Dispatch enforcement matters: runtime-constructed calls such as
`Symbol["Get"][...]` or parsed strings through `ToExpression[...]` are still
checked when the builtin dispatch is reached.

A denied call emits a `General::sandbox` message and returns `$Failed`.

Fine-grained policies can allow, for example, `{Pure, FileRead}` for read-only
file access or `{Pure, Diagnostic}` for instrumentation without file access.

Role or identity mapping is the caller's job. `rum` supplies the capability
partition, policy object, and dispatch enforcement.

## Memory Watchdog Pattern

The host-side memory-budget pattern uses `cancel_handle()`.

The host monitors process RSS with platform code and calls `cancel()` if the
process exceeds its memory budget.

This keeps `rum-core`, `rum-runtime`, and `rum-host` free of platform-specific
RSS polling while still giving a real process-wide memory bound to native
hosts.

`MemoryConstrained` exists as a cooperative memory-proxy subset, but a host
RSS watchdog is the hardening pattern for embedding.

## Poison Recovery

Public `HostSession` entrypoints catch panics from guest execution.

A caught panic returns `HostFailureCode::Panic` and poisons the session.

Further calls fail with `HostFailureCode::Poisoned` until the host calls
`reset()`.

Stack-overflow aborts are not caught by this boundary. The parser and matcher
depth guards prevent the covered source-evaluation paths that would otherwise
reach the known deep-source crash class.

