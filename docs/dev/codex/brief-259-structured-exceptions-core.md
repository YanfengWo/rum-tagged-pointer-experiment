# Brief 259 — structured exceptions core (wl15/wfr batch 3a)

## Goal

A folk structured-exception system with OBSERVABLY SEPARATE channels
from classic `Throw`/`Catch`, per the rulings in
`docs/dev/structured-exceptions-contract.md` §4 (E1-E6, all adopted
2026-07-18). When done these all work through `rum-cli`:

- `Exception[type, payload]` — inert 2-slot object (`type` a string,
  `payload` an Association). Constructor never throws. `Protected`.
- `RegisterExceptionType[type]` / `RegisterExceptionType[type, parent]`
  — registers a string type in a single-parent hierarchy; re-register
  with a DIFFERENT parent fails closed (message, no state change).
- `ThrowException[type]` / `ThrowException[type, payload]` — raises;
  on an UNREGISTERED type, fails closed (message + raises a root
  `"Exception"` type carrying the original as payload). Propagates to
  the nearest enclosing `CatchExceptions`, ancestry-aware.
- `CatchExceptions[expr, handlers]` — `handlers` is a list of rules
  matched against the `Exception[t, p]` object; first matching handler
  (ancestry-aware, see Dispatch) wins, its RHS is the result. No
  handler matches → the exception re-raises outward.
  `CatchExceptions[expr]` = catch-all (any exception → the exception
  object itself is the result).
- `ExceptionTypes[]` — the registered types as a list of
  `type -> parent` rules (parent `"Exception"` for roots), sorted.
- An uncaught `ThrowException` surfaces IN-BAND as a result value
  `Failure["UncaughtException", <|"Type" -> t, "Payload" -> p|>]` plus
  one `ThrowException::nocatch` message — NOT a host failure class.
- Classic `Throw`/`Catch` and `ThrowException`/`CatchExceptions` NEVER
  cross-catch (separate channels).
- `WithCleanup` runs its cleanup when an exception unwinds through it;
  exceptions pass through `Enclose` untouched.

## Mechanism (director-verified sites, 2026-07-18)

The non-local exit rides a NEW `EvalError` variant, mirroring how
`Throw` rides `EvalError::Thrown` (`kernel_data/control_data/nonlocal.rs`).
Director-verified by adding the variant and running `cargo check`:
exactly ONE non-wildcard match breaks — every other `EvalError` match
propagates the new variant for free via `Err(error) => Err(error)`.

- `crates/rum-runtime/src/runtime/session/session_state.rs:354-356` —
  add `ExceptionThrown { exception: Expr }` to `enum EvalError` (put it
  after `InputRequired`; give it a `#[error("uncaught exception:
  {exception}")]`).
- `crates/rum-runtime/src/runtime/session/session_requests.rs:386` —
  `session_request_error_from_eval_error_with_source`: the ONE
  exhaustive match. Add a defensive arm mapping `ExceptionThrown` to
  `SessionRequestError::Other { message: … }` (it should not normally
  be reached — the E4 boundary converts it to a value first).
- **E4 boundary** `crates/rum-runtime/src/runtime/session/session_eval.rs:22`
  — `finalize_top_level_eval_value`: intercept
  `Err(EvalError::ExceptionThrown { exception })` BEFORE the
  `TerminatedEvaluation` check; build the `Failure[…]` value from the
  exception's type/payload, `emit_non_loader_eval_message` a
  `ThrowException::nocatch` message, return `Ok((failure, None))` (no
  termination reason — it is an ordinary Success carrying a Failure).
- **The registry** is a session field mirroring `message_definitions`
  EXACTLY (MECHANICAL — director-verified these are ALL its lifecycle
  sites): declare `exception_types: BTreeMap<String, Option<String>>`
  (type → parent; `None` = child of implicit root `"Exception"`) at
  `crates/rum-runtime/src/runtime/model/session.rs:60` (beside
  `message_definitions`); init `BTreeMap::new()` in `Session::empty()`
  (`session_state.rs:397` area); add it to `struct LoaderSnapshot`
  (`session_state.rs:134-157`), to `capture_loader_snapshot`
  (`session_helpers.rs:41-66`, clone it), and to
  `restore_loader_snapshot` (`session_helpers.rs:67-90`, assign it).
  That is the full lifecycle (empty + snapshot round-trip); no separate
  reset path exists.
- **The builtins** go in a NEW kernel file
  `crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs`
  (mod-register it next to `nonlocal`/`cleanup` in
  `kernel_data/control_data/mod.rs`), registered in the kernel
  `builtin_catalog/` (locate the control-family catalog file the way
  `Throw`/`Catch` are registered — `builtin_catalog/*.rs`; grep
  `"Throw"`). Attributes: `Exception` `{Protected}`;
  `CatchExceptions` `{HoldAll, Protected}` (like `Catch` — body +
  handlers held); `ThrowException`/`RegisterExceptionType`/
  `ExceptionTypes` ordinary `{Protected}` (args evaluate).

## Dispatch (write this contract exactly; it is a design decision)

For an exception `Exception[t, p]` reaching `CatchExceptions[expr,
handlers]`:

1. Build the type chain `T = [t, parent(t), parent(parent(t)), …]`
   walking the registry to the root, then the implicit `"Exception"`
   root last. (If `t` is unregistered — only possible for the root
   fallback exception — `T = ["Exception"]`.)
2. Try handlers in LIST ORDER (author priority). A handler `H` matches
   iff there EXISTS an ancestor `a ∈ T` such that `H`'s LHS pattern
   matches `Exception[a, p]`; use the NEAREST such `a` (smallest index
   in `T`) to bind. The FIRST handler (list order) that matches at any
   level wins; evaluate its RHS with the bindings; that is the result.
3. No handler matches any level → re-raise the ORIGINAL
   `Exception[t, p]` (propagate the `ExceptionThrown` outward
   unchanged).

Use the existing matcher (`match_expr_owned` / the same path `Catch`
uses at `nonlocal.rs:194`) per (handler, ancestor); do not hand-roll
pattern matching. Keep it suspension-safe (the `drive_internal_value` /
`suspend_with_frame` shapes `Catch` uses).

`CatchExceptions[expr]` (no handlers): any `ExceptionThrown` is caught
and the `Exception[t, p]` OBJECT is the result (no re-raise).

## E5 fail-closed detail

`ThrowException[t, p]` when `t` is not registered: push
`ThrowException::unreg`, then raise
`ExceptionThrown { exception: Exception["Exception", <|"OriginalType"
-> t, "OriginalPayload" -> p|>>] }` (the root type, original carried in
payload). `RegisterExceptionType[t, parent]` re-registering `t` with a
parent different from the stored one: push
`RegisterExceptionType::reregister`, no state change, return the
existing registration (`t -> stored-parent`). Registering `parent` need
not pre-exist (folk scope — a forward parent is allowed; ancestry walk
stops at an unregistered parent as if it were a root child).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/session/session_state.rs` (enum
    variant + registry field init + LoaderSnapshot field)
  - `crates/rum-runtime/src/runtime/model/session.rs` (registry field)
  - `crates/rum-runtime/src/runtime/session/session_helpers.rs`
    (snapshot capture/restore — MECHANICAL, mirror message_definitions)
  - `crates/rum-runtime/src/runtime/session/session_requests.rs` (the
    one defensive match arm)
  - `crates/rum-runtime/src/runtime/session/session_eval.rs` (E4
    boundary)
  - NEW `crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs`
    + its `mod` line in `kernel_data/control_data/mod.rs`
  - the kernel `builtin_catalog/*.rs` file registering the control
    family (the five new entries + their message declarations)
  - `crates/rum-runtime/src/runtime/tests/` — a new test module + its
    `mod` line in `tests/mod.rs`
  - `docs/reference/builtins.md` (regeneration output only)
- MUST NOT touch: `Throw`/`Catch`/`WithCleanup`/`Enclose` eval code
  (their behavior is frozen; you only ADD the parallel channel and rely
  on WithCleanup's existing wildcard `Err(error)` cleanup path);
  brief-255/256/257/258 code; everything else. If any OTHER `EvalError`
  match turns out non-exhaustive, that is a MECHANICAL propagate-arm —
  add `Err(error) => Err(error)`-equivalent and note it, do not
  redesign.
- No unconditional default-path cost: the new channel only exists when
  an exception is thrown; no per-eval-step cost.

## Deliverable

- The variant + registry + five builtins + E4 boundary + dispatch.
- Tests (measured expectations — probe the release CLI FIRST, then
  pin) covering AT LEAST:
  - register + `ExceptionTypes[]` round-trip; re-register-different-
    parent fail-closed.
  - throw/catch by exact type; catch by a registered PARENT type
    (ancestry); handler order priority; no-match re-raise to an outer
    catcher.
  - `CatchExceptions[expr]` catch-all returns the object.
  - E5 unregistered-type throw → root exception + `unreg` message.
  - E4 uncaught → `Failure["UncaughtException", …]` value + `nocatch`
    message (probe at top level).
  - E3 separation BOTH directions: `Catch[ThrowException[…]]` does NOT
    catch (exception propagates); `CatchExceptions[Throw[…]]` does NOT
    catch (throw propagates).
  - E6: `CatchExceptions[WithCleanup[ThrowException["T"], Print["c"]],
    {…}]` runs the cleanup; `Enclose[ThrowException["T"]]` lets the
    exception through (probe the exact behavior first).
  - registry survives a snapshot/restore round-trip (mirror an existing
    message_definitions snapshot test if one exists).
- Reference regenerated; drift test green.

## rum / Rust pitfalls (read twice)

1. Register types BEFORE throwing in every probe/test; feed stdin
   (`printf '' | … --eval '…'`). Expected values MEASURED, never
   predicted.
2. Message-registry enforcement is LIVE (brief-253): every
   `push_message` tag MUST be declared on its builtin's catalog entry
   (`ThrowException::unreg`, `ThrowException::nocatch`,
   `RegisterExceptionType::reregister`, plus any argx/argform you add).
   A missing declaration trips the debug assertion.
3. Suspension safety: `CatchExceptions` evaluates a body that may
   suspend (InputRequired) exactly like `Catch` — reuse the
   `suspend_with_result_frame_lazy` / `drive_internal_value` shapes
   from `nonlocal.rs`; do not collapse them.
4. The E4 Failure must be a VALUE (Ok), not an error — return through
   `finalize_top_level_eval_value` as `Ok((failure, None))`.
5. `Expr`/`SharedArgs` are `Arc`-shared; clone the exception object by
   `Arc`, not deep copy.
6. Stop-loss: if the ripple exceeds the sites named here (more than a
   couple of mechanical propagate-arms), or the dispatch needs matcher
   changes, STOP and report — do not redesign the matcher.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace --exclude rum-jupyter   # sandbox loopback exclusion; note it
```

No microbench A/B: the new channel adds no default-path cost (no
eval-hot-path change; the mask/dispatch runs only on an actual throw).
State that explicitly.

Paste VERBATIM live probes (register first; feed stdin):

```
RegisterExceptionType["NetError"]; RegisterExceptionType["Timeout", "NetError"]; ExceptionTypes[]
CatchExceptions[ThrowException["Timeout", <|"ms" -> 30|>], {Exception["NetError", p_] :> {"caught-parent", p}}]
CatchExceptions[ThrowException["Timeout"], {Exception["Other", _] :> "no", Exception["Timeout", _] :> "yes"}]
CatchExceptions[ThrowException["NetError"]]
ThrowException["Unregistered"]
Catch[ThrowException["NetError"]]
CatchExceptions[Throw[42], {Exception[_, _] :> "x"}]
CatchExceptions[WithCleanup[ThrowException["NetError"], Print["cleanup"]], {Exception[_, _] :> "caught"}]
Enclose[ThrowException["NetError"]]
```

## Hard rules

- No `git commit`/branch/push; leave changes in the tree.
- Nothing outside Scope; must-touch conflicts → STOP and report.
- Nothing outside this repo, no network; commands: Scope edits,
  `cargo`, release `rum-cli`, `python3`.
- Testing Policy hard line; no new `unsafe`; minimal diff; verbatim
  output only, never faked.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why (every mechanical
   propagate-arm listed).
2. Verification — commands + verbatim output.
3. Verdict — a behavior table for the probe list; the E3 separation and
   E4/E5/E6 outcomes called out explicitly.
4. Stop-loss / open questions (empty if none).
