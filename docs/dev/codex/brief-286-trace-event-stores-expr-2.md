# Brief 286 — `TraceEvent` stores `Expr` (2)

Re-brief of 285, which **stop-lossed correctly** on my scope omission:
`push_definition_stored_trace` has six callers that build the payload
string, and none of their files were in the MAY list. Codex stopped
before touching anything; the tree was clean. The fault was the brief's.

**Read `docs/dev/codex/brief-285-trace-event-stores-expr.md` in full** —
its motivation, design, acceptance rules, performance requirements and
stop-losses all stand unchanged. This file states only the delta.

## Scope delta

ADDED to the MAY-modify list:

- `crates/rum-runtime/src/runtime/definitions/definition_tags.rs`
- `crates/rum-runtime/src/runtime/definitions/definitions.rs`

— for the six `push_definition_stored_trace` call sites and nothing
else. Everything else in brief-285's Scope, including its MUST-NOT
list, stands.

## The six call sites, and exactly what each becomes

`push_definition_stored_trace`'s `value` closure changes from
`FnOnce() -> String` to `FnOnce() -> Expr`. Both `lhs` and `rhs` are
already `Expr` at every site.

**Group A — three sites that render the RHS alone** (`|| rhs.to_string()`):

| file | site | becomes |
|---|---|---|
| `definition_tags.rs` | `DelayedOwnValue` (~line 192) | `rhs.clone()` |
| `definition_tags.rs` | `OwnValue` (~line 200) | `rhs.clone()` |
| `definitions.rs` | `OwnValue` (~line 422) | `rhs.clone()` |

**Group B — three sites that build a composite definition string**
(`format!("{} {} {}", lhs, if delayed { ":=" } else { "=" }, rhs)`):

| file | site | becomes |
|---|---|---|
| `definition_tags.rs` | `DownValue` (~line 231) | `SetDelayed[lhs, rhs]` when `delayed`, else `Set[lhs, rhs]` |
| `definition_tags.rs` | `SubValue` (~line 252) | same rule |
| `definition_tags.rs` | `UpValue` (~line 274) | same rule |

## Why Group B's replacement is exact, not a redesign

`DefinitionStored.value` is not a rendered expression — it is a
rendered *definition form*. But `Trace[]` immediately parses it back
(`trace_expr_from_text`), and the parse of `lhs := rhs` is precisely
`SetDelayed[lhs, rhs]`. Building that `Expr` directly therefore
reproduces today's output exactly, while skipping the round trip.

Verified on the release CLI at `a333c9e`, before writing this brief:

```
Trace[f[x_] := x + 1]
  → List[Hold[SetDelayed[f[Pattern[x, Blank[]]], Plus[x, 1]]], Hold[Null]]

Trace[g = 5]
  → List[Hold[Set[g, 5]], Hold[5]]

Trace[h /: p[h] := 7]
  → List[Hold[TagSetDelayed[h, p[h], 7]], Hold[SetDelayed[p[h], 7]], Hold[Null]]
```

Note the third: for an `UpValue`, the composite string is built from the
DEFINITION's lhs (`p[h]`), not the tag symbol — so `SetDelayed[lhs, rhs]`
with the `lhs` already in scope reproduces it. Do not reach for the tag.

These three expressions **must appear in the corpus** required by
brief-285's acceptance section, and their output must be unchanged. If
any of them changes, stop and report — unlike the fidelity differences
brief-285 anticipates elsewhere, these three are pinned above as
measured facts.

## Everything else

Unchanged from brief-285: the `Expr`-payload change to `TraceEvent`, the
collapsed empty-payload branch (with the early-return guard untouched),
deleting `trace_expr_from_text`, the `expr_memory_proxy_bytes` swap, the
host-boundary rendering with `HostTraceEvent` keeping its `String`
fields, the 25-expression corpus with per-difference judgments, the
before/after timing table, the microbench A/B, the stop-losses, the
hard rules, and the report format.

Baseline is `1662 passed / 0 failed` on `a333c9e`.
