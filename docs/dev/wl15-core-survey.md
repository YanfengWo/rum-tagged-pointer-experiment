# WL 15 core-language survey (2026-07-07)

Source: the public Version 15 launch post and public reference pages,
surveyed for core-language items inside rum's scope (facts re-expressed
in our own words; no documentation text copied). Most of V15 is outside
rum's scope by design (AI assistants, notebooks, cloud, curated data,
numerics/data-science). Four core-language findings follow.

## 1. SubValuesHoldAll-equivalent — WORTH DOING, parked on the perf freeze

WL 15 adds a `SubValuesHoldAll` attribute: in `f[a…][s…]…` the normal
arguments `a…` evaluate as usual, but arguments in subvalue positions
`s…` stay unevaluated (`Sequence` still flattens, `Unevaluated` is
stripped, upvalues of the held arguments still fire — i.e. it mirrors
`HoldAll`, not `HoldAllComplete`; `Evaluate` re-enables evaluation per
argument). This closes a long-standing WL gap: attributes only governed
the outermost head's direct arguments, so hold-y functions could not
have operator forms (`AppendTo[…]`-style curried application always
evaluated the final argument).

rum today (probed 2026-07-07): subvalue DEFINITIONS work
(`f[a_][x_] := …`, `SubValues[f]` lists the rule) and subvalue
arguments evaluate before binding (`f[1][2+3]` binds `x` to `5`,
`Hold[x]` → `Hold[5]`) — the pre-15 standard behavior. `AppendTo`/
`AssociateTo` carry `HoldFirst`; no operator forms.

Assessment: a rum equivalent would generalize the callable-wrapper
operator-form mechanism (brief-164) to hold-y functions (mutation
pipelines like `AppendTo[val]` applied to a held symbol). The
implementation site is the argument-evaluation decision for
compound-head calls — call-planning territory, so it is FROZEN under
the perf-measurement moratorium. Revisit when (a) the freeze lifts and
(b) the kernel units 8-10 audit surveys evaluation control; both feed
one design.

## 2. Structured exceptions — the most valuable in-scope item; design-first candidate

WL 15 adds `ThrowException` / `CatchExceptions` /
`RegisterExceptionType` and a symbolic `Exception` object: named
exceptions propagate to the nearest enclosing catcher registered for
that type; types form a registered hierarchy (catching a parent
catches registered subtypes); handlers pattern-match the exception
object and its payload; an uncaught-in-scope exception surfaces as a
`Failure`.

rum today: classic tagged `Throw`/`Catch` work; `Failure` objects
exist. Assessment: structured exceptions fit rum's embedded-scripting
direction directly (script robustness, sandbox error discipline,
host-visible failure taxonomy) and need no external dependencies —
symbolic objects + a registry + handler dispatch by pattern, all folk
scope. Proposed as its own small design-first line (contract doc, then
briefs) rather than a usability-pass batch; interacts with the
suspension-safe helper boundary (Throw crossing continuations), which
is exactly why the design comes first.

## 3. Incremental/lazy collections — PARKED

WL 15 adds `IncrementalObject`/`NextValue` and incremental variants of
`Select`/`Permutations`/`Subsets`/`Tuples`/`Map`/`FoldList`/`Take`/
`Range` (lazy enumeration of large spaces). Parked for rum: stateful
handles interact with snapshot/reset/cancel exactly like streams/ports
(deferred for the same reason). Revisit only with a concrete
downstream use case.

## 4. Noted, low value for rum

- One-argument `Monitor[expr]` (auto progress for top-level iteration):
  rum's `Monitor` is a best-effort display concern; skip.
- `ModelFit`, `TimeSeriesEvents`/`EventSeries*`, `Ordinal`/`Nominal`:
  numerics/data-science, out of scope by design.
