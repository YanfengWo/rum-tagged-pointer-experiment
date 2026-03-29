# Known Differences: Ham vs Mathematica

Tracked via `!!!` lines in golden files. This document summarizes each known
difference, its root cause, and what is needed to resolve it.

## 1. `0^0` (arithmetic.txt)

| | Value |
|---|---|
| Ham | `1` |
| Mathematica | `Indeterminate` (with `Power::indet` message) |

**Type:** intentional (pending feature)

**Root cause:** Ham treats `0^0 = 1` (common convention in combinatorics/algebra).
Mathematica returns `Indeterminate` and emits a diagnostic message.
The correct Mathematica-compatible behavior requires the message system to be
implemented first.

**Resolution:** Implement the message system, then add `Power::indet` handling
to the `Power` built-in.

## 2. Specific-value override precedence (definitions.txt)

| | `f[1]=10; f[x_]:=x+1; f[1]` |
|---|---|
| Ham | `2` |
| Mathematica | `10` |

**Type:** temporary (bug)

**Root cause:** When a specific value (`f[1] = 10`) is defined *before* a
general pattern (`f[x_] := x + 1`), Ham overwrites the specific rule with the
general one. Mathematica always prioritizes specific values over general
patterns, regardless of definition order.

Note: when the specific value is defined *after* the general pattern
(`f[x_]:=x+1; f[1]=10; f[1]`), both Ham and Mathematica agree on `10`.

**Resolution:** Fix the rule storage/lookup in the kernel to sort specific rules
before general patterns, matching Mathematica's specificity-based dispatch.

## 3. `Unevaluated` top-level wrapper (evaluation_control.txt)

| | `Unevaluated[1 + 1]` |
|---|---|
| Ham | `Unevaluated[1 + 1]` |
| Mathematica | `1 + 1` |

**Type:** temporary (behavior gap)

**Root cause:** In Mathematica, `Unevaluated[expr]` at the top level strips the
wrapper and returns `expr` without evaluating it. Ham keeps the `Unevaluated`
wrapper in the output.

**Resolution:** Adjust the evaluator so that `Unevaluated` at the top level
returns its argument unevaluated, with the wrapper stripped.
