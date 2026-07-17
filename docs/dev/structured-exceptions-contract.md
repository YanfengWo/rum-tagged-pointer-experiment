# Structured exceptions contract (wl15/wfr pass, batch 3)

Status: contract draft for batch 3 of the wl15/wfr pass (TODO §ACTIVE
PLAN item 5), pending user green light on E1-E6. Source discipline:
WL15 survey facts only (named exceptions propagate to the nearest
enclosing catcher registered for that type; types form a registered
hierarchy — catching a parent catches registered subtypes; handlers
pattern-match the exception object and payload; uncaught surfaces as a
Failure). Everything below is a rum folk design from those facts.

## 1. Why design-first (recorded reasons)

- `Throw` crossing suspension/continuation frames: the new catcher must
  dispatch on REGISTRY ANCESTRY at unwind time; the contract must state
  what a catcher frame stores so a resumed suspension still dispatches
  correctly.
- The registry is session state: snapshot/restore, `reset()`, and
  per-request isolation must treat it like definitions (b4 contract).
  Everything here is `Capability::Pure`.
- Message-registry enforcement is live: every new builtin declares its
  tags at catalog time.

## 2. Probe results (2026-07-17, current tree)

1. `Catch[WithCleanup[Throw[42], Print["cleanup-ran"]]]` → prints
   `cleanup-ran`, returns `42`: cleanup runs while the throw unwinds
   through it, both caught and uncaught (`WithCleanup[Throw[42], …]`
   at top level runs cleanup, then `Throw::nocatch`). The E6 premise
   holds for the classic channel the new head will ride.
2. `Catch[Module[{a}, Throw[42]]]` → `42`: Throw crosses the Module
   staged body (unit-9 machinery) correctly.
3. Failure inventory is rich and reusable: `FailureQ` exists;
   `Failure[tag, assoc]` is a well-formed inert object;
   `Enclose[Confirm[$Failed]]` already builds a structured
   `Failure["ConfirmationFailed", <|MessageTemplate, …|>]` — the
   uncaught→Failure surface should reuse this object shape.
4. Probe hygiene: rum-cli probes must feed stdin (`printf '' | …`);
   two spurious hangs in this pass were the CLI filter mode waiting on
   an open stdin pipe, not evaluator bugs.

## 3. Proposed surface (folk scope)

- `Exception[type, payload]` — inert symbolic object; payload an
  Association (empty allowed). The constructor never throws.
- `ThrowException[type]` / `ThrowException[type, payload]` — raises;
  propagates to the nearest enclosing `CatchExceptions` whose
  registered-type set covers `type` (ancestry-aware).
- `CatchExceptions[expr, handlers]` — handlers = list of rules whose
  LHS pattern-match the `Exception[…]` object
  (`Exception[t_, p_] :> …`); a handler list that matches nothing
  re-raises outward. `CatchExceptions[expr]` = catch-all convenience.
- `RegisterExceptionType[type]` / `RegisterExceptionType[type, parent]`
  — registers; single-parent hierarchy; re-registration with a
  different parent fails closed.
- `ExceptionTypes[]` — registered list, parent-annotated (introspection
  for scripts and tests).

## 4. Decision points for the user

- **E1 object shape**: fixed 2-slot `Exception[type, payload]`
  (RECOMMEND — pattern matching stays trivial) vs variadic.
- **E2 type identity**: strings (`"Timeout"`) vs symbols. RECOMMEND
  strings: no symbol-creation/protection questions, snapshot-trivial,
  matches the codec family's fail-closed discipline. Implicit root
  ancestor for everything registered.
- **E3 relation to classic Throw/Catch**: SEPARATE channels — bare
  `Throw` not catchable by `CatchExceptions`, `ThrowException` not
  catchable by `Catch`. RECOMMEND separate (simple contracts, no
  tag-collision surprises); implementation may share the underlying
  non-local-exit machinery.
- **E4 uncaught behavior**: in-band — an uncaught `ThrowException`
  surfaces at the evaluation boundary as a
  `Failure["UncaughtException", <|"Type" -> …, "Payload" -> …|>]`
  result plus one message, NOT a host failure class (survey: "surfaces
  as a Failure"; consistent with the b1 in-band `TerminatedEvaluation`
  precedent). Reuse the Enclose-style Failure object shape (probe 3).
- **E5 registration discipline**: RECOMMEND registration-required
  (fail-closed): `ThrowException` on an unregistered type emits a
  message and raises a root-type exception carrying the original as
  payload, instead of silently inventing hierarchy. (Lenient
  auto-register hides typos.)
- **E6 interplay**: `WithCleanup` cleanup runs when an exception
  unwinds through it (probe 1 pins the channel's behavior); exceptions
  pass through `Enclose` untouched (`Enclose` confines Confirm-family
  failures, not exceptions) — both pinned by tests in 3a.

## 5. Non-goals

- No stack traces / no global handler hooks (folk scope).
- No bridging of kernel resource terminations: recursion/memory/timeout
  stay `TerminatedEvaluation`, NOT catchable exceptions — sandbox
  ceilings must not be script-absorbable (consistent with b2).
- No serialization of in-flight exceptions (snapshot captures the
  registry only, never an unwind in progress).

## 6. Batch split (implementation, post-green-light)

- **3a core**: registry + `Exception` inert head + ThrowException/
  CatchExceptions/RegisterExceptionType + ancestry dispatch +
  uncaught→Failure + declared messages + tests (suspension-crossing,
  WithCleanup/Enclose interplay, reset/snapshot round-trip).
- **3b polish**: `ExceptionTypes[]`, guide section, reference regen,
  close-out.
