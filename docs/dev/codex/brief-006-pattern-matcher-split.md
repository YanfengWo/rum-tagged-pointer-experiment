# Brief 006 — Split the pattern-matcher cost for `downvalue-pattern-hit` (diagnostic)

## Goal

After brief-005's systemic win, the re-ranked worst primitive is
`downvalue-pattern-hit` (~`7.2x` WK); it benefited least because its dominant
cost is the matcher, not the shared instrumentation that brief-005 fixed. A
per-phase view shows `down_values.fast_match` ~`518 ns/call` is the fattest
downvalue-specific phase for matching `fPattern[1]` against
`fPattern[x_Integer]`. Split that matcher cost into its sub-parts so the
director can prescribe a precise matcher fast path. **No source change.**

The matcher entry is `fast_match_definition_pattern`
(`crates/rum-runtime/src/runtime/definition_application.rs` ~line 895). Note for
your attribution: it RE-INTERPRETS the raw pattern `Expr` every call — running
a chain of ~11 `pattern_*` predicate probes (`pattern_verbatim_inner`,
`pattern_named_pattern`, `pattern_condition`, `pattern_test`, `pattern_except`,
`pattern_optional`, `pattern_alternatives`, `pattern_key_value_pattern_entries`,
`decompose_sequence_pattern`, `pattern_sequence_items`,
`pattern_orderless_sequence_items`) before it reaches the actual match — even
though a COMPILED `MatcherNode` already exists on the `DownValue`. It also calls
`attribute_set_for_head`, `match_blank` (the `_Integer` type check), and
`bind_pattern_name` (a `SymbolExprMap` insert for `x -> 1`). The diagnostic must
say which of these dominate.

## Context pointers

- Read `AGENTS.md` and `docs/performance-stage0-baseline.md` first.
- Reducers: `scripts/sample-self-time.py`, `scripts/sample-attribute-callers.py`.
- The label is `fPattern[x_Integer] := x + 1` timed as repeated `fPattern[1]`.

## Scope

- Files you MAY modify: NONE (diagnostic). Scratch files only under `/tmp`.
- MUST NOT touch `crates/`, `docs/`, `scripts/`, `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, any `mma-hol` clone, `.git`. No source instrumentation.

## Probe (use EXACTLY — pitfall 2b, no sink Set, profiling OFF for the sample)

`/tmp/brief006-pat.wl`:

```wolfram
fPattern[x_Integer] := x + 1;
Print[AbsoluteTiming[Do[Do[fPattern[1], {i, 1, 100000}], {j, 1, 30}]]];
```

Confirm cleanliness: a counter run (`RUM_PROFILE_EVALUATION_CACHE=1`) must show
`symbol_invalidations` ~`1`, `down_value_scans`/`down_value_applications` ~`1`
per app.

## Deliverable

1. A `sample` capture (release `rum-cli`, profiling OFF) reduced with
   `scripts/sample-self-time.py` (top ~25 self-time rows), and
   `scripts/sample-attribute-callers.py` for `fast_match_definition_pattern`
   and for its top callees.
2. Attribute the matcher self time to its sub-parts. Specifically quantify how
   much goes to:
   - the `pattern_*` predicate-probe chain (re-parsing the pattern Expr),
   - `bind_pattern_name` / `SymbolExprMap` (HashMap) insert/alloc for the single
     binding,
   - `attribute_set_for_head` / `cached_attribute_set_for_symbol`,
   - `match_blank` (the `_Integer` head/type check),
   - `strip_hold_pattern` and structural `Expr` clones/compares.
3. Corroborate with the per-phase timer
   (`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, `runtime-path-self-timing`) for the
   relative split (absolute ns inflated — relative only).
4. A one-paragraph finding: which sub-part dominates the ~`518 ns` matcher
   cost, and whether the evidence supports the hypothesis that the matcher
   should use the precompiled `MatcherNode` instead of re-probing the raw
   pattern Expr (i.e. is the predicate-probe chain a large share?).

## rum / Rust pitfalls (read twice)

1. Build/sample the RELEASE binary; profiling OFF for the sample, ON only for
   the per-phase corroboration and counters.
2. Probe = the label's exact statement; no sink `Set` (pitfall 2b); verify with
   `symbol_invalidations` ~`1`.
3. `sample` needs a live pid; background the probe, then `sample`.

## Verification (MANDATORY — evidence, no test gate)

Paste VERBATIM: the probe `.wl`; the counter cleanliness line; the top ~25
self-time rows; the attribution tables for `fast_match_definition_pattern` and
its top callees; the per-phase corroboration; and the sub-part split.

If `sample` cannot attach or attribution is inconclusive after two captures,
STOP and report what you have plus what is missing.

## Hard rules

- NO source change. NO git. NO edits outside `/tmp`. Repo + `/tmp` only, no
  network. Commands: `cargo build`, release `rum-cli`, `sample`,
  `scripts/sample-*.py`, `python3`.
- Never edit any `mma-hol` file.
- Stop-loss: inconclusive after two captures → STOP and report.
- Paste real verbatim output; if a step produced no evidence, say so.

## Report format

1. "Diagnostic only, no source change." Probe `.wl` + counter cleanliness.
2. Verification — verbatim self-time rows, attribution tables, per-phase split.
3. The finding — which sub-part dominates the matcher cost, and whether the
   precompiled-`MatcherNode` hypothesis holds, naming functions, in one
   paragraph.
4. Stop-loss / open questions (empty if none).
