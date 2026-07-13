# Brief 217 — Fail-closed / truthful contracts: repetition bounds, ReplacePart Span, dead options

Fix brief — unit-9 batch 3 (safety/flexibility, M). Source:
`docs/dev/usability-pass/kernel-b-findings.md` (top-gaps #6/7/8/9; the
`Repeated`/`RepeatedNull`/`ReplacePart`/`SpellingCorrection`/`Once` gap
rows; Evidence E9/E10/E12) — read it FIRST. Four independent sub-fixes;
only (a)'s matcher half touches pattern compilation and carries the A/B
gate. All are WL-conformance or truthful-contract moves; no WL default
changes.

## (a) `Repeated`/`RepeatedNull` count shapes: unify + diagnose

Today (E9):

- Expression patterns accept all documented count shapes — `n`, `{n}`,
  `{min, max}`, `Infinity` (`matcher_and_symbols.rs:362-424`).
- STRING patterns accept only list bounds: `StringCases["aaa",
  Repeated["a", 2]]` stays INERT while `{2}` works
  (`rum-stdlib/src/strings/patterns.rs:306-329`, `:711-728`).
- Malformed bounds are SILENT in both: negative/reversed
  (`Repeated[p, -1]`, `Repeated[p, {2, 1}]`) become a quiet nonmatch in
  expression patterns and an inert call in string patterns.

Required:

- The string matcher accepts the same documented count shapes as the
  expression matcher (share or mirror the count-shape parser — say which
  in the report). `StringCases["aaa", Repeated["a", 2]]` → `{"aa"}`.
- Malformed bounds fail CLOSED with a message at pattern
  normalization/compile time (pick the existing message machinery the
  matchers already use; same tag family in both domains), instead of a
  silent false nonmatch / inert call. Zero-count edge (`{0}`,
  `{0, n}`) — probe WL-documented behavior; where WL defines it, match
  it; where uncertain, pin-and-document.
- Valid shapes' MATCH RESULTS are byte-identical to today (this fix
  changes acceptance and diagnostics, not matching semantics). The
  expression-matcher touch is in pattern COMPILE, but mark it "matcher
  core → A/B gate" anyway and let the numbers speak.

## (b) `ReplacePart` Span positions (the reference already promises them)

Today (E12): the reference says "exact, Span, and pattern-based position
specifications", but `ReplacePart[{a, b, c, d}, (2 ;; 4 ;; 2) -> x]`
silently returns the list unchanged — the exact parser recognizes only
integer/key selectors and a Span falls through as a literal pattern that
can never match integer paths (`replace/parsing.rs:48-121`).

Required:

- Span rules select the same positions `Part` would: `(2 ;; 4 ;; 2) ->
  x` on `{a,b,c,d}` → `{a, x, c, x}`; plain `(2 ;; 3)`, open ends
  (`;;`, `2 ;;`), negative endpoints — mirror the Part span semantics
  already implemented in `structure_data/selectors/parse.rs` (REUSE its
  span resolution; do not hand-roll a second one).
- Out-of-range/invalid spans get the same diagnostic class ReplacePart
  already uses for malformed positions — never a silent no-op.
- Immediate vs delayed RHS semantics per brief-216 (delayed inserts
  raw). Pattern positions and exact positions byte-identical.

## (c) `SpellingCorrection` on `Names`: reject the advertised lie

Today (E10): `Options[Names]` advertises `SpellingCorrection -> False`
and the call syntax accepts `-> True`, but nothing reads it
(`namespace_tools/names.rs:200-229`, `:322-327`;
`options/defaults.rs:76-80`): `True` is a silent no-op.

Required: until spelling correction exists, `SpellingCorrection -> True`
fails CLOSED — a message plus whatever failure shape `Names` already
uses for a bad option value today (PROBE that first and mirror it
exactly; do not invent a new shape). Default `False` stays
byte-identical and silent. Keep the option registered (discovery is
fine; the lie was the silent acceptance of `True`).

## (d) `Once` expiration options: fail closed on the unimplemented

Today: `Options[Once]` advertises `ExpirationDate -> None` and
`PersistenceTime -> None`, but runtime logic distinguishes ONLY integer
`PersistenceTime -> 0` (no-cache); every other known value — including
any `ExpirationDate` — silently means cache-forever
(`options/defaults.rs:72-75`, `package_tools.rs:599-607`).

Required: keep `Capability::Pure` (NO clock reads — expiration policies
needing wall time stay unimplemented); accept the implemented values
(`None`, integer `PersistenceTime -> 0`) byte-identically; any OTHER
value → message + fail closed (no silent cache-forever). Say in the
report which values remain accepted.

## Method

1. Survey tests for assertions encoding the old silences (inert string
   Repeated, silent span no-op, silent option acceptance). Old→new list.
2. Implement (b)/(c)/(d) first (builtin-local, no call planning), then
   (a) with its matcher touch last.
3. New tests per sub-fix: positive shapes + the fail-closed diagnostics
   + byte-identical defaults. Baseline-verify every preservation pin
   (the brief-216 lesson).

## Perf gate

One A/B for the batch (`scripts/core-language-microbench.sh`, saved
binaries, B C B C, profiling env OFF, idle machine; load appears → STOP
and report, the director runs final numbers). Watch labels:
`matchq-integer-success`/`matchq-integer-fail` and `string-repeat-small`
(the (a) compile touch); (b)/(c)/(d) have no labels — workspace suite
covers them.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes per sub-fix, before/after, batched
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- (a) `crates/rum-runtime/src/runtime/matcher_and_symbols.rs`;
  `crates/rum-stdlib/src/strings/patterns.rs`
- (b) `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/parsing.rs`
  (+ `replace/plan.rs` if selection planning needs it; reuse
  `structure_data/selectors/parse.rs` — read-only unless a helper needs
  `pub(crate)` visibility, name it if so)
- (c) `crates/rum-runtime/src/runtime/namespace_tools/names.rs`
- (d) `crates/rum-runtime/src/runtime/package_tools.rs`
- Options table if a default's VALUE must change: `crates/rum-runtime/src/runtime/options/defaults.rs` (registration text only, no new options)
- `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md`
  (regenerated; Limits MERGE only)
- Tests: the pattern/string/replace/namespace test families across both
  crates (list each file in the report), one new test file allowed.

MUST NOT touch: the evaluator core, `binding_substitution.rs`,
`replace_all.rs`, scoping, `Options`/`OptionValue` machinery itself,
capability declarations, anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`, no capability changes (`Once` stays `Pure`).
- Valid-input behavior is byte-identical everywhere; only silences
  become diagnostics. Baseline-verify preservation pins.
- WL comparison from documented knowledge only; uncertain edges
  pinned-and-documented.
- Stop-loss: if (a) cannot share/mirror the count parser without
  changing expression-match results, STOP and report; same if (b)
  cannot reuse the Part span resolution.

## Report format

1. Per-sub-fix summary (seam, mechanism, what is reused).
2. Test survey old→new.
3. Probe panels before/after verbatim, per sub-fix.
4. A/B table + noise-floor verdict (watch labels called out) + raw
   paths — or the load stop-loss note.
5. Verification outputs verbatim.
6. Deviations / open questions (empty if none).
