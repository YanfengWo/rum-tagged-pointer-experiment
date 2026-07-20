# Lossless rule-aware trace — design

Layer 1 of the trace line. The substrate landed in brief-286
(`TraceEvent` stores `Expr`); this document designs the symbol that
stops throwing information away.

User rulings already in hand (2026-07-20): a **new symbol**, not an
option on `Trace`; and **lossless** — nothing the event stream carries
gets dropped.

Grounding measured on `c79632a`.

## 1. What `Trace[]` discards today

`structured_trace_expr_from_events` (`debug_tools/trace.rs:245`)
destructures four fields away:

| event | used | **discarded** |
|---|---|---|
| `EvaluationPass { before, after }` | both | — |
| `DefinitionStored { symbol, kind, value }` | `value` | **`symbol`, `kind`** |
| `RuleApplied { source, input, rule, result }` | `input`, `result` | **`source`, `rule`** |

So four fields, not two. `kind` is `DefinitionKind` (5 variants:
`OwnValue`, `DelayedOwnValue`, `UpValue`, `DownValue`, `SubValue`);
`source` is `RuleApplicationSource` (9 variants: `UpValue`, `DownValue`,
`SubValue`, `Replace`, `ReplaceList`, `ReplaceAt`, `ReplaceAll`,
`ReplaceAllUnheld`, `ReplaceRepeated`).

Depth is not discarded — it is what builds the tree — so a lossless view
keeps the tree AND the four fields.

**The headline capability**: `rule` identifies the rule that fired.
CORRECTED 2026-07-21 (this document originally illustrated it as a whole
rule, `Hold[f[x_] :> x + 1]` — that was written from imagination, not
measured): every emission site passes the rule's **left-hand side only**
(`&lhs`, or `&rule.lhs` in `replace_all.rs`). The RHS template is never
recorded — `result` holds the substituted outcome, not the template. The
LHS is nonetheless a sufficient identifier, since two definitions cannot
share one.
With several overlapping `DownValue`s on one symbol, "which definition
matched" is the actual debugging question, and no current view answers
it.

## 2. The decision: output shape

The tree nesting stays as it is (`List` nesting = subevaluation depth).
The question is what a STEP looks like.

### Option A — Association per step (recommended)

```
TraceDetail[f[2]]
→ List[
    <|"Step" -> "RuleApplied", "Source" -> "DownValue",
      "Input" -> Hold[f[2]], "Rule" -> Hold[f[x_]],
      "Result" -> Hold[2 + 1]|>,
    List[ <|"Step" -> "EvaluationPass", "Before" -> Hold[2 + 1],
            "After" -> Hold[3]|> ],
    …
  ]
```

- Queryable with the tools the row-workload line already built
  (`Select`, `SortBy`, `Lookup`, `MaximalBy` over a `List` of
  associations is a first-class workload in rum).
- Self-describing: a reader does not need to memorize argument
  positions.
- Extensible: a future field is a new key, not a new argument slot that
  breaks every consumer.
- Cost: verbose. A trace of any size is a wall of text unless rendered,
  which is what the REPL view is for.

### Option B — typed head per step

```
TraceRule[DownValue, Hold[f[2]], Hold[f[x_] :> x + 1], Hold[2 + 1]]
TracePass[Hold[2 + 1], Hold[3]]
```

- Compact, pattern-matchable (`Cases[t, TraceRule[DownValue, ___]]`
  reads well).
- Cost: positional. Adding a field later breaks consumers, and the
  reader must know the order. Two heads also means consumers must handle
  a heterogeneous list.

### Option C — keep `Hold[…]` leaves, attach metadata only where it exists

Mixed list of bare `Hold[…]` and annotated steps. Rejected: a consumer
must handle both shapes at every position, which is the worst of both.

**Recommendation: Option A.** The queryability argument is decisive
given what rum's association tooling can already do, and the
extensibility argument matters because this event stream has grown
before and will again.

## 3. Naming

`TraceDetail` is my proposal. Alternatives considered: `TraceRules`
(understates it — definition and pass steps are included too),
`TraceFull` (suggests "no truncation" rather than "no information
loss"), `TraceAnnotated` (accurate, clumsy).

Not a strong opinion; say the word if you prefer another.

## 4. Argument surface

Mirror `Trace` exactly — `TraceDetail[expr]`, `TraceDetail[expr, form]`,
`TraceDetail[expr, form, levelspec]`, `Heads -> True|False` — so
everything learned about one transfers to the other. `{HoldAll,
Protected}` and `Capability::Pure`, same as `Trace`.

One open sub-question: the `form` filter currently matches against the
step expression. With associations as steps, does `form` match the
association or the underlying expression? **Recommendation: the
underlying expression**, so `TraceDetail[expr, _Plus]` keeps meaning
what `Trace[expr, _Plus]` means. Filtering on the annotation is what
`Select` over the result is for.

## 5. Non-goals

- No REPL `:trace --rules` in the same brief. Land the kernel symbol
  first, pin it, then decide the REPL surface separately — the same
  sequencing that worked for `Trace` → `:trace`.
- No change to `Trace` itself. It stays the compact view; this is the
  detailed sibling.
- No new capability (`Pure`, like `Trace`).
- No host-channel work (`HostTraceEvent` forwarding stays as design doc
  `repl-trace-view-design.md` §1.5 left it).

## 6. If ruled

One brief: the new builtin (registration chain, structured builder
reuse, argument parsing mirroring `Trace`), reference-doc entry, and
tests including a case with several overlapping `DownValue`s where the
whole point is seeing WHICH one fired.
