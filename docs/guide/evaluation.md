# Evaluation

Evaluation starts after source has been parsed and lowered.

At a high level, a session:

1. parses source into syntax
2. lowers syntax into core expressions
3. evaluates expressions to a fixed point
4. returns the final value and any runtime messages

The runtime audit records the current evaluator model in more detail:
[runtime evaluation audit](../runtime-evaluation-audit.md).

## Fixed Point

Top-level evaluation repeats until the expression stops changing.

User definitions re-enter the same evaluator as ordinary source.

```text
$ cargo run -q -p rum-cli -- --eval 'f[x_] := x + 1; f[3]'
4
```

The first expression stores a delayed downvalue.

The second expression evaluates through that downvalue and then through exact
arithmetic.

A common memoization idiom is `f[x_] := f[x] = rhs`: the delayed definition
computes a value once for a matching argument, then stores an immediate
definition for that concrete call.

```text
$ cargo run -q -p rum-cli -- --eval 'calls = 0; fib[0] = 0; fib[1] = 1; fib[n_Integer] := fib[n] = (calls = calls + 1; fib[n - 1] + fib[n - 2]); {fib[8], fib[8], calls}'
List[21, 21, 7]
```

The counter increments for `fib[2]` through `fib[8]`; the second `fib[8]` reads
the stored immediate value.

## Call Pipeline

For calls, the evaluator currently follows this staged shape:

1. evaluate the head
2. build a dispatch plan from builtin metadata, attributes, and head traits
3. let special forms inspect raw arguments
4. evaluate ordinary arguments according to hold attributes
5. apply attribute-driven preprocessing
6. try upvalues attached to evaluated arguments
7. try downvalues on the evaluated head
8. try subvalues on nested heads
9. try builtin dispatch
10. try pure-function application
11. reconstruct the call if nothing fires

This order is important for rules, held forms, and embedding outcomes.

## Held Arguments

Hold attributes control which arguments evaluate before a builtin or definition
sees them.

`Hold` has `HoldAll`, so it holds its argument unevaluated.

`ReleaseHold` then evaluates the held expression.

`Abs` is `Listable`, so it threads over lists.

```text
$ cargo run -q -p rum-cli -- --eval 'x = 3; {Hold[x + 1], ReleaseHold[Hold[x + 1]], Abs[{-2, 3, -4}]}'
List[Hold[Plus[x, 1]], 4, List[2, 3, 4]]
```

The held expression keeps `x` as a symbol.

The released expression evaluates `x + 1` to `4`.

The listable call maps `Abs` over each list element.

## Attribute Families

Attributes are data on heads.

The current core includes useful coverage for:

- hold behavior: `HoldAll`, `HoldFirst`, `HoldRest`, `HoldAllComplete`
- threading behavior: `Listable`
- associative and ordering behavior: `Flat`, `Orderless`, `OneIdentity`
- sequence behavior: `SequenceHold`
- selected protection and numeric metadata used by the builtin catalog

The implementation is still an early subset. The reference lists attributes for
each builtin entry.

Attribute-driven behavior is part of evaluation, not syntax lowering.

## Evaluation Order

The evaluator is head first.

After the head is stable, the call's argument evaluation is controlled by the
evaluated head's hold behavior.

Definitions see evaluated arguments unless a hold attribute or special form
keeps an argument raw.

Upvalues run before ordinary downvalues for ordinary calls unless
`HoldAllComplete` blocks them.

Subvalues run after downvalues once the nested head is stable.

## Messages

Messages are runtime outputs attached to an evaluation.

The CLI prints the final value and then the emitted message lines.

```text
$ cargo run -q -p rum-cli -- --eval 'Check[Message[General::stop]; 1, fallback]'
fallback
! General::stop: message emitted
```

`Message` emits `General::stop`.

`Check` sees that a message was emitted and returns the fallback.

`Quiet` suppresses selected message output. If a message is quieted, `Check`
does not treat it as a visible message for fallback selection.

## Limits As Values

Several evaluator limits are converted into
[`TerminatedEvaluation`](../reference/builtins.md#terminatedevaluation) values
when the top-level request path handles them as recoverable evaluation
outcomes.

```text
$ cargo run -q -p rum-cli -- --eval 'Do[x = 1, 100000000]'
TerminatedEvaluation["IterationWorkLimit"]
! General::reslim: evaluation exceeded iteration work limit (100000)
```

The value says which budget was tripped.

The message gives the concrete limit and category.

Host APIs expose the same rendered value in successful outcomes for these
recoverable cases.

## Cycles And Pass Limits

Fixed-point evaluation has guards.

If the same expression reappears in the same fixed-point loop, the runtime
reports an evaluation cycle.

If evaluation keeps changing without stabilizing past the configured pass
limit, the runtime reports an evaluation pass limit.

Both map to the `IterationLimit` terminated-evaluation reason in the current
top-level value path.

The resource chapter lists the caller-set limits.

## Recursion

Recursive evaluator re-entry is guarded by `recursion_limit`.

Native stack headroom is guarded separately by `native_stack_limit`; unbounded
definitions can therefore terminate as `TerminatedEvaluation["StackLimit"]`
before the logical recursion counter reaches `recursion_limit`.

Those limits are separate from the fixed-point pass limit and the pattern-match
depth guard.

Hosts should set resource limits according to their workload, especially when
evaluating untrusted source.

## Suspension

Host-driven input is part of the evaluator model.

`Input` and `InputString` can suspend a request. The host then resumes the same
request with input instead of replaying the whole source string.

The embedding chapter shows the `HostEvalOutcome::InputRequired` path.

## Structured Exceptions

Structured exceptions are a typed, hierarchical alternative to tagged
`Throw`/`Catch`.

An exception type is a string registered in a single-parent hierarchy.

```text
$ cargo run -q -p rum-cli -- --eval 'RegisterExceptionType["NetError"]; RegisterExceptionType["Timeout", "NetError"]; ExceptionTypes[]'
List[Rule["NetError", "Exception"], Rule["Timeout", "NetError"]]
```

Registration is required. Every registered type descends from the implicit
root `"Exception"`. Re-registering a type with a different parent is refused
and leaves the registration unchanged.

`ThrowException[type, payload]` raises. The payload is an `Association`.
`CatchExceptions[expr, handlers]` runs `expr` and matches each raised
`Exception[type, payload]` object against the handler rules.

A handler registered for a parent type catches its registered subtypes.

```text
$ cargo run -q -p rum-cli -- --eval 'RegisterExceptionType["NetError"]; RegisterExceptionType["Timeout", "NetError"]; CatchExceptions[ThrowException["Timeout", <|"ms" -> 30|>], {Exception["NetError", p_] :> {"caught-by-parent", p}}]'
List["caught-by-parent", Association[Rule["ms", 30]]]
```

Handlers are tried in list order. Within a handler, the nearest matching
ancestor binds. A handler list that matches nothing re-raises the exception
outward. `CatchExceptions[expr]` without handlers catches any exception and
returns the exception object.

An uncaught exception is an in-band value at the request boundary, not a host
failure. It becomes a `Failure["UncaughtException", …]` result carrying the
type and payload, plus a `ThrowException::nocatch` message.

```text
$ cargo run -q -p rum-cli -- --eval 'RegisterExceptionType["NetError"]; ThrowException["NetError", <|"url" -> "x"|>]'
Failure["UncaughtException", Association[Rule["Type", "NetError"], Rule["Payload", Association[Rule["url", "x"]]]]]
! ThrowException::nocatch: Uncaught Exception["NetError", Association[Rule["url", "x"]]] returned to top level.
```

Throwing an unregistered type fails closed the same way: a message plus a root
`Exception` carrying the original type and payload.

The two channels are separate. `Catch` never catches a `ThrowException`, and
`CatchExceptions` never catches a classic `Throw`. `WithCleanup` still runs its
cleanup when an exception unwinds through it. `Enclose` does not confine
exceptions.

The registry is session state. It is captured and restored with the loader
snapshot like other definitions.
