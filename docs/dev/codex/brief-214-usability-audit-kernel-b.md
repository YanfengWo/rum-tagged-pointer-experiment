# Brief 214 — Usability audit: kernel B (scoping / eval control / patterns / forms; 47 builtins)

Audit brief — sweep unit 9 of the Usability Pass, the second kernel unit.
Read `docs/dev/usability-pass.md` FIRST: the three axes, the compat rule, the
coverage discipline, and the closed-unit ledgers (units 1-8) — they show the
judgment standard, the corrections method, and the operator-form
disambiguation rule. This is an AUDIT: you change NO behavior and NO code; the
only file you create is the findings document.

This unit owns the surface by which a rum program **controls evaluation**:
scoping constructs, the hold family, pattern atoms and rule heads, the
structural/pattern-query functions, and the display-form heads. Unit 8 swept
the layer that defines symbols; this unit sweeps the layer that decides what
gets evaluated, matched, and rewritten. A silent semantic divergence here is
worth more than any single builtin's ergonomics.

## Count reconciliation (do this first, and state it)

Reproduce the standing derivation (unit-8 verification step; Claude re-ran it
today), then the unit-9/10 split:

```
installed reference (docs/reference/builtins.md `### ` headings) = 612
stdlib catalog names (crates/rum-stdlib/src/catalog/*.rs BuiltinSpec entries,
  name = FIRST string arg — it sits on the line AFTER `BuiltinSpec::…(`) = 453
612 - 453 = 159 kernel builtins
159 - 55 (unit 8, listed in brief-208) = 104
104 = 47 (THIS unit) + 57 (unit 10)
```

**Unit 9 = these 47 names** (authoritative; one row MINIMUM per name, grouped
as below, this order):

- **Scoping (5):** `Block`, `Module`, `With`, `Function`, `Unique`
- **Eval control / hold family (12):** `Hold`, `HoldComplete`, `HoldForm`,
  `HoldPattern`, `ReleaseHold`, `Evaluate`, `Unevaluated`, `Once`,
  `Sequence`, `Nothing`, `Sow`, `Reap`
- **Pattern atoms + rule heads (12):** `Repeated`, `RepeatedNull`, `Longest`,
  `Shortest`, `IgnoreCase`, `Overlaps`, `Whitespace`, `NumberString`,
  `SpellingCorrection`, `Rule`, `RuleDelayed`, `NormalizeRules`
- **Structural / pattern-query (11):** `Part`, `Extract`, `ReplacePart`,
  `ReplaceAt`, `Position`, `Count`, `MemberQ`, `FreeQ`, `Level`, `Thread`,
  `MissingQ`
- **Forms + marker symbols (7):** `InputForm`, `OutputForm`, `StandardForm`,
  `TraditionalForm`, `Short`, `Automatic`, `$Failed`

**Unit 10 = the 57-name complement** (assert it in your reconciliation; do
NOT sweep them): `$Aborted`, `$Canceled`, `$Context`, `$ContextPath`,
`$InputFileName`, `$Packages`, `$Path`, `AbsoluteTime`, `AbsoluteTiming`,
`Begin`, `BeginPackage`, `ByteArray`, `ByteCount`, `Context`, `Contexts`,
`DeclarePackage`, `Dialog`, `DiffObject`, `DirectoryName`, `DumpSave`,
`Echo`, `EchoFunction`, `EchoLabel`, `End`, `EndPackage`, `EvaluationData`,
`File`, `FileNameJoin`, `FileNameTake`, `FindFile`, `Get`, `Input`,
`InputString`, `MaxMemoryUsed`, `MemoryConstrained`, `MemoryInUse`,
`Monitor`, `NameQ`, `Names`, `Needs`, `ParentDirectory`, `Pause`, `Print`,
`RepeatedTiming`, `ResolveContextAliases`, `RumProfileReset`,
`RumProfileSnapshot`, `Save`, `Stack`, `SymbolName`, `TemplateObject`,
`TestCreate`, `TestReport`, `TimeConstrained`, `Timing`, `Trace`,
`TracePrint`.

(The four `HoldAll`/`HoldFirst`/`HoldRest`/`HoldAllComplete` ATTRIBUTE
symbols were unit 8. THIS unit has the `Hold`/`HoldForm`/`HoldPattern`/
`HoldComplete` HEADS.)

## Head start: what Claude already found (VERIFY, do not re-derive)

Three probe batches, run before this brief. Reproduce each as evidence in
your appendix, then build on them — they are the spine of the unit.

1. **`Protected` is DONE for this unit — it is NOT the battleground.** All
   47 names already carry `Protected` (brief-210's whole-registry sweep
   covered them; checked against the reference with a whole-line match, not
   the first-backtick regex trap). 14 carry Hold-class attributes:
   `Block`/`Evaluate`/`Function`/`Hold`/`HoldForm`/`HoldPattern`/`Module`/
   `NormalizeRules`/`Once`/`ReleaseHold`/`With` = `HoldAll`;
   `HoldComplete`/`Unevaluated` = `HoldAllComplete`; `Reap` = `HoldFirst`.
   No unit-8-style whole-registry section this time; per-row attribute notes
   suffice.
2. **Scoping + hold-family basics are green** (one batched eval):
   ```
   {Module[{x}, x], Module[{x = 2}, x + 1], Block[{y = 5}, y * 2],
    With[{z = 3}, z^2], Unique[], Unique["t"], Hold[1 + 1],
    ReleaseHold[Hold[1 + 1]], Hold[Evaluate[1 + 1]],
    HoldComplete[Evaluate[1 + 1]], f[Unevaluated[1 + 1]], Once[1 + 1]}
   → List[x$1, 3, 10, 9, $3, t4, Hold[Plus[1, 1]], 2, Hold[2],
      HoldComplete[Evaluate[Plus[1, 1]]], f[2], 2]
   ```
   Module gensyms (`x$1`); `Evaluate` pierces `Hold` but NOT `HoldComplete`
   (WL-correct) — but note the eleventh element:
3. **ONE found WL divergence — `f[Unevaluated[1 + 1]]` → `f[2]` for an
   undefined head `f`.** WL passes the argument unevaluated and RESTORES the
   `Unevaluated` wrapper when no definition consumes it (the result stays
   `f[Unevaluated[1 + 1]]`); rum evaluated the content into the surviving
   form — a silent divergence on the no-match path. The BUILTIN path is
   correct: `Length[Unevaluated[1 + 1]]` → `2`, `Head[Unevaluated[1 + 1]]`
   → `Plus`. Map the boundary: which call paths honor the wrapper, which
   strip-and-evaluate, and where in the evaluator the restore step is
   missing (`file:line`). This is the unit's known real finding — bound it
   precisely so the fix brief is small.
4. **Pattern atoms + structural queries are green on basics** (one batched
   eval):
   ```
   {Length[Unevaluated[1 + 1]], Head[Unevaluated[1 + 1]],
    MatchQ[{1, 1, 2}, {Repeated[1], 2}], MatchQ[{1, 1, 1}, {Repeated[1, {2}]}],
    StringCases["aaa", Longest[RepeatedNull["a"]]],
    StringCases["aaa", Shortest[Repeated["a"]]],
    MatchQ[Hold[1 + 1], Hold[HoldPattern[1 + 1]]],
    ReplaceAt[{1, 2, 3}, x_ :> x + 10, 2], ReplacePart[{1, 2, 3}, 2 -> b],
    Part[{1, 2, 3, 4, 5}, 2 ;; 4], Thread[f[{1, 2}, {a, b}]],
    Position[{1, {1, 2}}, 1], Level[{1, {2, {3}}}, {-1}],
    Count[{1, 2, 1, 3}, 1], Rule[a, Sequence[1, 2]]}
   → List[2, Plus, True, False, List["aaa", ""], List["a", "a", "a"], True,
      List[1, 12, 3], List[1, b, 3], List[2, 3, 4], List[f[1, a], f[2, b]],
      List[List[1], List[2, 1]], List[1, 2, 3], 2, Rule[a, Sequence[1, 2]]]
   ```
5. **Two attribute curiosities to TRACE, not to "fix":** (a) rum's
   `Evaluate` carries `HoldAll` where WL's `Evaluate` has no hold attribute —
   is the attribute load-bearing for the piercing mechanism, or vestigial?
   (b) `Rule`/`RuleDelayed` show NO hold attribute, yet `RuleDelayed`'s rhs
   is plainly held and `Rule[a, Sequence[1, 2]]` does not splice — WL gets
   both from `SequenceHold` + `HoldRest`, attributes rum's `Attribute` enum
   does not even have. So the behavior is hard-coded (evaluator/lowering) —
   find WHERE (`file:line`) and state what an attribute addition would
   actually do (no-op / duplicate / behavior change). This is the brief-208
   `Set` trace discipline; the trace is itself a deliverable.

## The scope guardrail (READ TWICE — this governs the whole unit)

- **Hold semantics live in the EVALUATOR CORE.** Any behavioral fix proposal
  in the hold family / argument evaluation touches the eval hot path — mark
  every such row "touches evaluator core → needs the microbench A/B gate"
  and size it honestly. This unit's fix batches will be few and heavy, not
  many and light; the audit's job is to bound them precisely.
- **`Module` gensym is a trust boundary** (LCF-style encapsulation — the
  downstream mma-hol application builds its sealed values on Module's
  renaming). Record Module/Block/With semantics factually, including edge
  cases; propose NO change to the renaming/scoping semantics. If you find a
  genuine scoping leak, report it as a finding with evidence and STOP — the
  fix decision is above this brief's pay grade.
- **No new attribute KINDS.** rum's `Attribute` enum (kernel_api) has no
  `SequenceHold`/`NHoldAll`/etc. Where WL semantics rest on such an
  attribute, note the mechanism difference factually; do NOT propose adding
  the enum variant (a separate, larger design question).
- **brief-193 standing ruling:** attribute additions that change argument
  evaluation or reordering are REJECTED unless independently justified. Do
  not propose "align `Evaluate`'s attributes to WL" or "add `HoldRest` to
  `RuleDelayed`" as hygiene — trace what the implementation actually reads
  first (deliverable 5 above); most such rows should end as "hard-coded,
  attribute would be dead weight — OK/intended".
- **Session state is `Capability::Pure` — correctly.** `Sow`/`Reap`/`Once`
  mutate SESSION state (snapshotable, resettable, deterministic); the
  stateful↔pure separation line (briefs 200-202) governs OUTSIDE-WORLD reads
  only. Do not file capability findings for them. What IS worth checking:
  where `Once`'s cache lives, whether it survives `reset()`/snapshot
  correctly, and whether `Sow` outside any `Reap` is handled per WL (value
  returned, sown expression discarded).
- **Forms are folk scope.** `InputForm`/`OutputForm`/`StandardForm`/
  `TraditionalForm`/`Short` — establish what each actually DOES in rum's
  renderer (wrapper recognized? truncation? plain inert?) and whether the
  reference says so honestly. An inert form head with honest docs is an OK
  row; an inert form head SILENTLY dropped or mis-rendered is a finding. Do
  NOT propose a WL typesetting/pretty-printer arms race.
- **Design-input duty (record, do NOT design).** The parked units-8-10 hold
  design (TODO.md: WL15 item (a) `SubValuesHoldAll`-equivalent; WFR item
  `ReplaceAllUnheld`) takes THIS unit's hold rows as input. Add a dedicated
  appendix section (see Findings file format) describing how held arguments
  flow through pattern matching and replacement TODAY. No proposals there.
- **Compat rule (standing user directive).** WL names keep WL-conformant
  DEFAULT behavior; improvements land as new options or rum-native siblings.
  Never change what an existing WL call form returns.

## What IS a legitimate finding here (the three axes, tuned to this unit)

- **Safety — silent semantic divergence in the evaluation machinery (the
  main event).** The `Unevaluated` no-match leak is the worked example; hunt
  its siblings: does `Sequence` splice correctly in held vs unheld argument
  positions; does `Nothing` vanish ONLY inside `List` (WL contract) or more
  broadly; does `Block` restore the old value when the body throws/aborts;
  does `Module` leak the temporary on error paths; does `With` substitute
  into held sub-positions (WL does — it is textual); does `ReleaseHold`
  strip exactly one layer and leave `HoldPattern` intact (WL: it removes
  `Hold`/`HoldForm`/`HoldComplete`/`ReleaseHold`... check rum against its
  own docs); does `HoldForm` render invisibly in output (WL displays the
  contents) or as a literal wrapper? A silent wrong answer beats any
  ergonomics row.
- **Safety — matcher correctness at the atoms.** `Repeated[p, n]`,
  `Repeated[p, {min, max}]`, `Repeated[p, {n}]` count forms (positive AND
  negative probes); `Longest`/`Shortest` in EXPRESSION patterns (not just
  string patterns — WL documents both; if rum's are string-only, that is an
  honest-docs row, not necessarily a gap); `IgnoreCase`/`Overlaps`/
  `SpellingCorrection` — where are they actually consumed, and is an
  unsupported placement inert-and-silent (finding) or messaged (OK)?
- **Flexibility — WL-documented forms this surface lacks.** CHECK, not
  assume: `Part` negative indices/spans, step spans (`1 ;; 5 ;; 2`), `All`,
  multi-dimensional specs; `Extract[expr, {pos1, pos2}]` multi-extract and
  the three-arg `Extract[expr, pos, h]` (wrap-in-head — pairs with `Hold`
  for unevaluated extraction, a scripting affordance); `ReplacePart`
  list-of-positions and multi-rule forms; `Position[expr, patt, levelspec,
  n]` + `Heads` option; `Count`/`MemberQ`/`FreeQ` levelspec forms; `Level[
  expr, spec, f]` third arg + `Heads` option; `Thread[f[...], h]` second
  arg; `Function[{x, y}, body]` named-parameter forms, `Function[x, body]`,
  and the three-arg attributes form `Function[{x}, body, HoldAll]`;
  `With[{...}, {...}, body]` multi-declaration form; `Unique[{"a", "b"}]`
  list form. Apply the operator-form disambiguation rule: WL documents
  operator forms for `Extract[pos]`, `Position[patt]`, `Count[patt]`,
  `MemberQ[patt]`, `FreeQ[patt]` — note that units 1-2 DEFERRED
  `Count`/`Position` operator forms while the perf moratorium held; the
  moratorium is lifted, so these are legitimate candidates now, each marked
  "touches call planning → A/B gate".
- **Convenience.** `Short[expr]`/`Short[expr, n]` truncation for large
  outputs (a REPL affordance); `Hold`-family round-trip ergonomics
  (`ReleaseHold` on nested holds); `NormalizeRules` (rum-native — judge
  intrinsically: is its contract documented, discoverable, composable?);
  `MissingQ` vs the `Missing[...]` family coherence.
- **Origin marking.** `NormalizeRules` is rum-native; everything else in the
  47 is WL-inherited (`ReplaceAt` is modern WL, 13.1+; `Once` 11.x+;
  `Nothing` 10.x+) — verify each against your WL knowledge and mark Origin
  honestly. For rum-native heads judge on the three axes intrinsically, not
  on WL parity.

## Inputs

- **Name list**: the 47 above. Reproduce the reconciliation (612 / 453 / 159
  / 55 / 47 / 57) as verification step 1.
- **Kernel registration** (`crates/rum-runtime/src/runtime/builtin_catalog/`),
  47 = 12 + 12 + 13 + 4 + 2 + 3 + 1:
  `scoping.rs` (12: With, Module, Unique, Block, Hold, HoldComplete,
  Unevaluated, Evaluate, HoldPattern, ReleaseHold, Sequence, Thread);
  `rules.rs` (12: Part, Extract, ReplacePart, ReplaceAt, Rule, RuleDelayed,
  Count, MemberQ, FreeQ, Level, Position, NormalizeRules);
  `strings.rs` (13: Whitespace, NumberString, IgnoreCase, Overlaps,
  Repeated, RepeatedNull, Shortest, Longest, InputForm, OutputForm,
  StandardForm, TraditionalForm, Automatic);
  `namespace.rs` (4: $Failed, SpellingCorrection, Nothing, Once);
  `functional.rs` (2: Function, HoldForm); `debug.rs` (3: Short, Sow, Reap);
  `associations.rs` (1: MissingQ).
- **Implementations**: `runtime/scoping/` (`lexical/`, `unique.rs`,
  `gensym_gc.rs`, `callable.rs`); `runtime/hold_thread.rs`;
  `runtime/pattern_matching/` (`core.rs`, `sequence/`, `outcome.rs`) +
  `pattern_matching_interpreted.rs` + `matcher_and_symbols.rs`;
  `runtime/kernel_data/` (`structure_data/` — `replace/replace_part.rs`,
  `replace/replace_at.rs`; `rewrite_data/`); `runtime/lowering.rs` +
  `runtime/eval_call_prepare.rs` + `runtime/eval_core/` +
  `runtime/eval_dispatch/` (the hold/pierce/splice special cases live in
  this layer — the key files for the hard-coded-hold traces).
- **Contracts**: `docs/reference/builtins.md` (all 47 have entries);
  `docs/guide/` chapters on evaluation/patterns if present; `AGENTS.md`.
- **Parked-design pointers** (for the design-input appendix ONLY):
  `TODO.md` WL15 survey item (a) (`SubValuesHoldAll`-equivalent, "design
  with the kernel units 8-10 audit") and the WFR survey's
  `ReplaceAllUnheld` deferral.
- **WL comparison**: documented WL surfaces from your knowledge (hold
  semantics, pattern atom count forms, operator forms). Design reference
  ONLY — never run an external engine.

## Findings file format

Write `docs/dev/usability-pass/kernel-b-findings.md`. Same as units 1-8 (the
eight existing findings files are the template): the seven-column table
(`Builtin | Origin | Verdict | Axis | Finding | Proposal | Size`), `OK` rows
REQUIRED (the all-rows table IS the coverage proof), one row per axis-gap;
then an **Evidence appendix** (verbatim probe transcripts, stdin fed:
`printf '' | cargo run -q -p rum-cli -- --eval '…'`; batch related checks;
`file:line` citations acceptable for code-visible gaps) and a **Triage
sketch** (2-5 candidate fix batches ranked by script-user value, noting which
touch the evaluator core / call planning and therefore need the A/B gate).

PLUS — unique to this unit — a dedicated section:

### `## Hold-semantics design input (units 8-10 parked item)`

Factual record, no proposals: (a) the hard-coded-hold trace table — for each
head whose hold/splice behavior is NOT attribute-driven (`Rule`,
`RuleDelayed`, `Evaluate`'s pierce, `Unevaluated`, `Sequence` splicing,
anything else you find), WHERE the behavior lives (`file:line`) and what an
attribute addition would actually do; (b) how a held argument flows through
pattern matching and replacement today (is a rule's rhs evaluated before
insertion into held positions? what would `ReplaceAllUnheld` need?); (c) the
`SubValues` hold question's unit-9 half: what the evaluator does between
matching `f[a][b]` and evaluating the args of the outer call. This section is
design input for the parked `SubValuesHoldAll` / `ReplaceAllUnheld` work —
make it complete enough that the design pass never re-derives it.

For every row, mark clearly whether an apparent gap is a REAL usability
finding or an intentional design boundary (folk scope / WL-conformant default
/ hard-coded hold / session-state-is-Pure — an `OK` row with a one-line
"intended" note, NOT a gap).

## Probe budget

Cap ~40 probe evals total (47 builtins; the attribute questions are already
answered by the reference file with no eval at all, and the head-start probes
cover the basics — spend the budget on the divergence boundaries: the
`Unevaluated` no-match paths, the scoping error paths, the pattern-atom count
forms, the Part/Extract/ReplacePart spec surface). Expected wall time 40-60
min. Batch related checks into one `--eval` list — note `expr ; expr` is
CompoundExpression (only the last value prints); use a `{...}` list. Feed
stdin (`printf '' | …`) so the CLI does not block. Anything needing more than
the cap becomes an open question, not a guess.

## Verification (MANDATORY — run all, paste verbatim)

```
# 1. count reconciliation — paste the numbers
grep -c '^### ' docs/reference/builtins.md                    # 612
# your derivation of stdlib catalog names (453), the kernel diff (159),
# minus unit 8's 55 = 104; assert the 47 are all in it and 57 remain

# 2. coverage — every one of the 47 names appears as a table row;
#    must print COVERAGE-OK 47/47 (paste the check script + output)

git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create: `docs/dev/usability-pass/kernel-b-findings.md` — the ONLY file
  in the diff.
- MUST NOT touch: any crate code, tests, guide, reference, `TODO.md`, the
  tracker, the other findings files, or anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies, no
  code changes.
- Every `gap` row is evidence-backed: a probe transcript or a `file:line`
  citation. Never claim a behavior you did not run.
- WL comparison from documented knowledge only; no external engines
  (Testing Policy, `AGENTS.md`).
- The design boundaries are hard lines: do NOT propose changes to Module's
  renaming semantics; do NOT propose new attribute kinds; do NOT propose
  hold-family behavior changes without the "evaluator core → A/B gate" mark;
  do NOT propose changing a WL name's default behavior; do NOT design the
  parked hold work (record input only).
- **Calibrate your instrument before trusting it** (`diagnostic-tool-trust`):
  the reference backticks each attribute separately — a first-token regex
  under-reports; `;` in `--eval` is CompoundExpression — a probe that shows
  one value did not run one check. Cross-check script-derived counts against
  a handful of live probes and say that you did.
- Stop-loss: if the same probe misbehaves twice, STOP and report it.

## Report format

1. Count reconciliation (612 / 453 / 159 / 55 / 47 / 57) and the coverage
   check output verbatim (47/47).
2. The `Unevaluated` divergence boundary: which paths honor the wrapper,
   which leak, and the `file:line` of the missing restore — the headline.
3. Top gaps by script-user value, one line each, safety first (any silent
   semantic divergence in scoping/hold/matcher machinery ranks above every
   ergonomics row).
4. The hard-coded-hold traces (Rule/RuleDelayed, Evaluate's pierce,
   Sequence splicing): where each lives, what an attribute would do.
5. The triage sketch (mark which batches touch the evaluator core / call
   planning and therefore need the microbench A/B gate).
6. Verification outputs verbatim.
7. Stop-loss / open questions (empty if none).
