# REPL trace view — design

Design-first document for the second half of the REPL/CLI polish line
(user ruling 2026-07-20: design it, after the snapshot work). No code
is written until this is ruled on.

Everything in §1 was measured on `0cd0fc3`, not recalled.

## 1. Grounding — what actually exists today

### 1.1 The kernel already traces, and it is `Pure`

`Trace` and `TracePrint` are registered in
`builtin_catalog/debug.rs`, implemented in
`runtime/debug_tools/trace.rs` (525 lines). Both are `{HoldAll,
Protected}`.

**Correction to an earlier claim of mine: they are NOT `Diagnostic`-gated.**
`capabilities.rs:231-232` asserts `capability("Trace") == Capability::Pure`
and the same for `TracePrint`. That classification is honest — tracing
observes evaluation and touches nothing outside it — and it has a direct
design consequence: **a `:trace` command needs no capability gate and
works in a sandboxed profile.**

### 1.2 `Trace` returns a structured tree, `TracePrint` returns text

```
Trace[1+1]        → List[Hold[Plus[1, 1]], Hold[2]]
Trace[(1+2)*3]    → List[Hold[Times[Plus[1, 2], 3]],
                         List[Hold[Plus[1, 2]], Hold[3]],
                         Hold[9]]
```

Nesting depth reflects subevaluation depth — it is already a tree, which
is what a foldable view needs. `TracePrint[(1+2)*3]` prints the same
shape as indented text:

```
Times[Plus[1, 2], 3]
  Plus[1, 2]
  3
9
9
```

### 1.3 Filtering exists, with one trap and one sharp edge

`Trace[expr, form]` is implemented (levelspec and `Heads -> True|False`
too). Measured:

| form | result | verdict |
|---|---|---|
| `_Plus` | `List[List[Hold[Plus[1, 2]]]]` | works |
| `_Integer` | `List[List[Hold[3]], Hold[9]]` | works |
| `3` (literal) | `List[List[Hold[3]]]` | works |
| `_` | full trace | works |
| **`Plus`** (bare symbol) | **`List[]`** | **trap** |
| **`Plus[__]`** | **full trace** | **sharp edge** |

- **The trap**: a bare symbol is used as a literal pattern, so it matches
  only the symbol itself, which never appears as a step — the filter
  silently returns empty. `Trace[expr, f]` reads like the obvious way to
  say "steps involving `f`" and quietly yields nothing.
- **The sharp edge**: `Plus[__]` matches *everything*. This is NOT a
  `Trace` bug — it is brief-275's OneIdentity promoted-singleton
  matching working as designed. Verified directly:
  `MatchQ[9, Plus[__]]` → `True`, `MatchQ[9, Times[__]]` → `True`,
  `MatchQ[9, f[__]]` → `False` (no `OneIdentity`). So on
  `OneIdentity` heads, the single-argument-pattern spelling is useless
  as a filter.

**In rum the correct spelling for "steps whose head is `H`" is `_H`.**

### 1.4 Traces get large fast

```
Length[Trace[Table[i^2, {i, 20}]]]              → 62
StringLength[ToString[Trace[f[6]]]]  (recursive factorial) → 1705
```

A recursive definition at depth 6 already renders ~1.7 KB. Size control
is not a nicety; without it the first realistic `:trace` fills the
screen.

### 1.5 CORRECTED 2026-07-20 — one stream, two lossy views

**The section below was wrong twice and is superseded by this one.** It
called the host channel a "second mechanism" and priced it at a
`rum-host` API change. Reading `debug_tools/trace.rs` and
`definitions/application/mod.rs` shows otherwise:

**There is ONE event stream.** `push_trace_with_depth`
(`definitions/application/mod.rs:101`) fills `trace_events` alongside a
parallel `trace_event_depths`. Per rule application the kernel records
`source` — a 9-variant `RuleApplicationSource` (`UpValue`, `DownValue`,
`SubValue`, `Replace`, `ReplaceList`, `ReplaceAt`, `ReplaceAll`,
`ReplaceAllUnheld`, `ReplaceRepeated`) — plus the rule expression
itself, the input, and the result.

**Two views consume it, each discarding what the other keeps:**

| | keeps | discards |
|---|---|---|
| `Trace[]` builtin | depth (→ the tree) | **`source` and `rule`** — `trace.rs:272` destructures `RuleApplied { input, result, .. }` |
| host `HostTraceEvent` | `source`, `rule` | **depth** — the host type has no depth field |

So **nothing today shows both the tree shape and which rule fired**,
even though the kernel records both.

**And the cost is much lower than §1.5-superseded claimed.** `Trace[]`
does its own scoped capture: it saves `tracing_enabled`, sets it true,
captures, and restores (`trace.rs:154-156, 223`). A sibling builtin can
do exactly the same and simply *not* discard `source`/`rule`. That means
**no `rum-host` API change, no session config, no capability change** —
it stays `Pure` and is reachable from the REPL exactly the way `:trace`
reaches `Trace[]` today.

The `rum-host` forwarding question is real but separate: it only matters
for an *embedder* wanting always-on session tracing, not for a REPL
view.

### 1.5-superseded (kept for the record — DO NOT rely on this)

Recorded so nobody rediscovers it. `HostEvalSuccess` already carries
`pub trace: Vec<HostTraceEvent>` — "trace events collected when tracing
is enabled" — a host-level channel entirely separate from the `Trace[]`
builtin, with three event kinds (`model/outcome.rs:263`):

- `EvaluationPass { before, after }`
- `DefinitionStored { symbol, kind, value }`
- `RuleApplied { source, input, rule, result }`

| | `Trace[]` builtin | host trace channel |
|---|---|---|
| scope | per-expression | session-wide once enabled |
| shape | nested tree (nesting = depth) | flat event stream |
| content | expression before/after | **also which rule fired**, and definition stores |
| REPL reach | works today, no new API | needs a new `rum-host` method |
| filtering | `Trace[expr, form]` | none (would be REPL-side) |

Its distinctive value is `RuleApplied`: it names the rule that fired,
which is the first question when debugging "why did this evaluate to
that", and which `Trace[]` cannot answer.

Its cost is that it is enabled at session construction
(`HostConfig::with_tracing`). The kernel does have a runtime toggle —
`Session::enable_tracing(&mut self, bool)`
(`session/session_api.rs:121`) — but **`rum-host` does not forward it**,
and the REPL is a pure `rum-host` consumer with zero `rum-runtime`
dependency (standing ruling from the embedding line). Using this channel
per-invocation would therefore require adding a `HostSession` setter —
an embedding-API change, not a REPL change.

**This does not change the Option-A ruling** (Option A needs no new API
and delivers the readable tree), but it is the natural next axis: a
later `:trace --rules` could surface `RuleApplied` once the enable/
disable seam exists. Deliberately out of scope for the first cut.

### 1.6 The REPL side

Current command set: `:help :parse :format :lower :complete :history
:inspect :messages :quit`. There is no `:trace`. Commands live in
`rum-repl/src/commands.rs`; the REPL is a line-oriented reader over
`Repl::new(input, output)` with no cursor addressing, no alternate
screen, and no paging today. Its full rendering surface was frozen with
snapshots in brief-282, so any new output is pinned from day one.

## 2. The decision this document asks for

Tracing already works. What is missing is a REPL *surface*. The
question is how much surface — three coherent options, in ascending
cost.

### Option A — thin view (recommended)

`:trace EXPR` evaluates `Trace[EXPR]` and renders the tree as indented
text, truncated by default, with two suffix flags mirroring what the
kernel already accepts:

```
:trace EXPR                  whole trace, truncated to N lines
:trace --form=_Plus EXPR     kernel form filter
:trace --depth=2 EXPR        kernel levelspec
:trace --full EXPR           no truncation
```

- Renders REPL-side from the structured `Trace[]` result, not by calling
  `TracePrint` — the REPL owns its output stream (a b3 hard line from
  the host-embedding work), and reusing the structured value keeps
  rendering under snapshot control.
- Truncation prints `… N more steps (:trace --full to see all)`.
- **`--form` takes the pattern through verbatim**, so §1.3's trap is
  reachable. Mitigation: when a bare symbol is passed, the REPL rewrites
  `--form=f` to `_f` and says so in one line (`filtering on _f`). That
  turns the trap into the obvious behavior without touching kernel
  semantics.

Cost: one command, one renderer, no new kernel work, no new state. Fits
the REPL's line-oriented model as it is.

### Option B — interactive pager

Everything in A, plus an interactive mode: arrow-key navigation, fold
and unfold subtrees, jump to next match.

Cost: needs cursor addressing and raw-mode input, which the REPL does
not have today — that is a terminal-capability layer (`rum-terminal`
would grow), plus an input model the current line reader cannot express.
It also breaks the snapshot testing approach, since an interactive
session is not a captured string.

**Not recommended now**: it is a much larger project than the debt entry
implies, and it would be built before there is any evidence about how
people actually read traces.

### Option C — declare the status quo final

No `:trace`. Users write `TracePrint[expr]` themselves. Close the entry
as WON'T-DO.

Defensible — the capability genuinely already exists and is `Pure`, so
nothing is blocked. But typing `TracePrint[...]` around an expression is
exactly the kind of friction a REPL is supposed to remove, and the
current filter trap (§1.3) means the natural first attempt at filtering
returns empty with no explanation.

## 3. Recommendation

**Option A**, plus one small kernel-adjacent question to rule on
separately:

> Should `Trace[expr, f]` with a bare symbol `f` keep returning `List[]`,
> or should it mean "steps whose head is `f`"?

Option A's `--form` rewrite handles this at the REPL layer without
touching the kernel, which is the conservative choice and my
recommendation. Changing the kernel would be a WL-conformance argument,
and this project has a standing rule against chasing a WK-derived spec
for its own sake — the matcher line just spent five briefs learning that
lesson. If the REPL-layer rewrite proves insufficient in use, the kernel
question can be reopened with evidence.

## 4. Non-goals

- No interactive/raw-mode terminal work (that is Option B).
- No new capability — tracing is `Pure` (§1.1).
- No change to `Trace`/`TracePrint` kernel semantics under Option A.
- No trace *storage* (no `:trace` history, no re-viewing a previous
  trace); each invocation is self-contained.

## 5. If Option A is ruled

Implementation splits cleanly into one brief:

1. `:trace` command parsing (`commands.rs`), including the three flags
   and the bare-symbol rewrite.
2. Tree renderer over the structured `Trace[]` result, with truncation.
3. Snapshot tests per brief-282's standard — full-output `assert_eq!`,
   negative-calibrated.

Open sub-question for the brief, not for this ruling: the default
truncation limit. I would start at 40 lines and pin it in a snapshot so
changing it is a deliberate act.
