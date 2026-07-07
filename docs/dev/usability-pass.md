# Usability Pass — tracker

Status: ACTIVE (opened 2026-07-07). Mainline after the breadth lines
(stdlib expansion, codec family, hardening coverage, A+B groups) closed.

The "works" layer is in; this pass raises the "works well" layer. The
`RandomPassword` arc (briefs 157-159) is the worked example: "works"
first (157), then flexibility (158), convenience + safety (159).

## Directives (user, 2026-07-07)

- **High coverage.** Every installed builtin is swept — the WL-inherited
  surface included, not only rum-native additions. 605 installed at open:
  25 stdlib catalog families = 445, kernel surface = 160.
- **Compatible enhancement.** Where a WL-inherited builtin can be
  improved, improve it WITHOUT breaking WL shape: default behavior stays
  WL-conformant; new powers arrive as new options (or as rum-native
  sibling functions). Never change what an existing WL call form
  returns.

## The three axes (review lenses)

- **Flexibility** — can it meet complex real-world needs? Options where
  WL has them; operator/curried forms; escape hatches for the
  non-default case; composability with the rest of the library.
- **Convenience** — can a user turn intent into code easily? Named
  constants over magic literals; sensible defaults; readable option
  surfaces; the common case short.
- **Safety** — does the design prevent foot-guns? `Protected` where
  constant-like; fail-closed on malformed input (message + `$Failed`,
  never a silent wrong answer); capability gating; determinism where
  surprising non-determinism would bite.

## Method

1. **Audit briefs** sweep one unit each and write a findings file
   `docs/dev/usability-pass/<unit>-findings.md`: one row per builtin —
   including "OK" rows; the all-rows table IS the coverage proof — and
   every claimed gap backed by a probe transcript or a code citation.
   Audit briefs change no behavior.
2. Claude triages findings into **fix briefs** (the 158/159 shape:
   small, axis-labeled, compat-checked).
3. Standing guards: folk scope (no CAS creep — heavy inputs get a
   message/fallback, not an algorithm arms race); testing policy (WL
   documentation knowledge may inform design; no external-engine runs);
   the feature-gating policy for anything dependency-adjacent.

## Sweep units

| # | Unit | Count | Audit | Fixes |
|---|------|-------|-------|-------|
| 1 | strings | 99 | brief-161 ✓ | 162b-168 ✓ CLOSED |
| 2 | structure + functional | 89 | brief-169 ✓ | 170-175 ✓ CLOSED |
| 3 | predicates + associations | 83 | brief-176 ✓ | 177-180 ✓ CLOSED |
| 4 | control + logic + rewrite | 63 | brief-181 ✓ | 182-185 ✓ CLOSED |
| 5 | arithmetic + number_theory + combinatorics + integer_digits + polynomials + calculus | 62 | brief-186 ✓ | 187… (triage below) |
| 6 | codecs (json/csv/ini/toml/xml/yaml/schema) + bytes + datetime | 34 | | |
| 7 | character_classes + random (light re-check; 157-159 fresh) | 15 | | |
| 8 | kernel A: definitions/values/attributes/protection | ~50 | | |
| 9 | kernel B: scoping/eval control/patterns/forms | ~55 | | |
| 10 | kernel C: system state/contexts/packages/paths/diagnostics/tests | ~55 | | |

Counts for units 1-7 are exact (catalog arrays, 445 total); units 8-10
split the 160-name kernel list (installed reference minus stdlib
catalog) and are enumerated exactly when their audit brief is written.

## Unit 1 (strings) triage — 2026-07-07

Audit brief-161: 101 rows / 99 names, findings in
`usability-pass/strings-findings.md`. Fix batches, dispatch order:

1. **brief-162/162b — attribute hygiene** (safety, all S): `Protected`
   on the 18 string-pattern symbols (Evidence E1); case-conversion
   alignment (`ToUpperCase` gains `Listable`+`Protected`;
   `Capitalize`/`Decapitalize` gain `Listable`). 162 stop-lossed
   correctly: `Listable` upgrades nested/mixed-list INERT forms to
   threading (flat string lists and plain strings stay byte-identical).
   RESOLVED (162b): align upward — rum already committed to `Listable`
   threading via `ToLowerCase`/`StringReverse`; a split semantics
   within the case-conversion family is the worse state; inert→useful
   is the same enhancement class as the operator-form batch. rum's own
   reference is the contract authority for the nested/mixed cases.
2. **brief-164 — operator forms, the WL-documented set only**
   (flexibility): `StringContainsQ`/`StartsQ`/`EndsQ`/`MatchQ`/
   `FreeQ`/`Replace`/`Cases` + `StringDelete` (audit false-OK: WL
   documents `StringDelete[patt]`). rum ALREADY has the curried
   mechanism (`Select[f][…]`/`Map[f][…]`/`SortBy[f][…]` work) — this
   batch extends it, no new evaluator seam. The audit's
   `Take`/`Drop`/`Count`/`Position` operator forms are rum EXTENSIONS
   on error paths — deferred, separate decision. `StringSplit` is
   correctly excluded (its unary call is already a meaningful data
   form; WL has no operator form for it either). SPILLOVER for later
   units: `Cases[patt][…]` (unit 2) and `ReplaceAll[rules][…]`
   (unit 4) lack their WL-documented operator forms today.
3. **Overlaps parity + Options registration** (flexibility):
   `StringCases` `Overlaps -> True`, `StringPosition`
   `Overlaps -> False`, register `Nearest`'s `DistanceFunction`.
4. **Scripting ergonomics options** (convenience): `StringWrap`
   preserve-newlines, `StringIndent` blank-lines, `AlignColumns`
   separator, `UrlEncode`/`UrlDecode` form mode, `UrlParse` options,
   `ParseArgs` short options, `StringRiffle` 3-arg form,
   `StringPadLeft`/`Right` list padding, `ToCharacterCode` encoding
   arg, `StringToByteArray` `"UTF-8"` alias.
5. **Fail-closed strict modes** (safety): `TemplateApply`,
   `Uncompress`, `HtmlUnescape`, `HammingDistance`.
6. **brief-163 — crypto surface, plan A (user decision 2026-07-07).**
   The audit flagged `Encrypt`'s toy XOR-SHA256 as a hazard; triage
   found more: `strings/encoding.rs` HAND-ROLLED SHA-256 (violates the
   digest policy: RustCrypto, never hand-rolled) and duplicated
   `bytes/digest.rs`. Decided: remove
   `Encrypt`/`Decrypt`/`EncryptedObject` (policy: encryption/signing
   never in kernel — the guide already said so), delete the hand-rolled
   SHA-256, re-plumb `Hash` onto the RustCrypto digests behind the
   `hash` feature, and extend its method names to
   SHA1/SHA512/MD5/CRC32.

**UNIT 1 CLOSED (2026-07-07).** Landed: 161 audit + 162b/163/164/165/
166/167/168 (plus 162's correct stop-loss). Workspace 1339/0 →
1345/0; reference 605 → 602 (plan A removed three). Left on purpose:
`Take`/`Drop`/`Count`/`Position` operator forms (deferred until the
perf-measurement freeze lifts — they graze call planning);
`LongestCommonSequence` sibling name (small later batch);
`Alphabet`/`RemoveDiacritics`/`SequenceAlignment` escape hatches
(future unicode line). Debts: rum-jupyter `signature.rs` hand-rolled
SHA-256 (TODO); rum-jupyter transient test failure (watch). Spillover:
`Cases`/`ReplaceAll` operator forms → units 2/4. Brief-craft lessons
now standing: MAY lists pre-include `docgen.rs` (LIMITS notes) and
`options/defaults.rs`; probes feed stdin; audits get an all-rows
coverage check.

## Unit 2 (structure+functional) triage — 2026-07-07

Audit brief-169: 89/89, findings in
`usability-pass/structure-functional-findings.md`. Mechanism fact
grounding the first batch: the options-defaults table
(`options/defaults.rs`) is a DISCOVERY surface — implementations never
consult it at call time (verified: zero `options_for_symbol` callers
in kernel_data/stdlib); registered defaults feed `Options`/
`OptionValue` only. Fix batches, dispatch order:

1. **brief-170 — Heads options registration** (safety, S): register
   `Heads -> False` for `Map`/`MapIndexed`/`MapAll`/`Scan`/`Apply`/
   `MapApply`/`Comap`/`ComapApply` — the Cases/Depth discovery
   pattern; hard-coded behavior defaults unchanged; NO per-call
   SetOptions wiring (new mechanism + hot path + frozen).
2. **brief-171 — spec flexibility**: `Subsets` advertised size specs
   (its own message currently lies — safety), `PadLeft`/`PadRight`
   list dimensions, `Partition` padded/offset forms, `Transpose`
   general permutations, `Reverse[expr, All]`, `Riffle` positional
   forms.
3. **brief-172 — collection comparison controls**: `DeleteDuplicates`/
   `Tally` test args, set-ops `SameTest`, `ReverseSort` comparator,
   `SortBy` 3-arg ordering, `GatherBy` function-list hierarchical
   grouping (today: silent wrong-ish singletons), `MaxBy`/`MinBy`
   top-k. Folk note: predicate dedup documented O(n²).
4. **brief-173 — fail-closed diagnostics**: `MapAt` missing-target
   message (WL parity), `SubsetMap` path messages, `Inner`/`Dot`
   shape-mismatch messages.
5. **brief-174 — WFR-inspired additions** (idea-level; see
   `docs/dev/wfr-core-survey.md`): `SelectPositions`, `Duplicates`,
   `BinarySearch` — no operator forms until the thaw.

**UNIT 2 CLOSED (2026-07-07).** Landed: 169 audit + 170 (Heads
registration) + 171/171b (spec flexibility; 171's inverted Partition
anchor caught at gate, fixed in 171b) + 175 (14 operator forms +
wrapper early-exit, a net perf win) + 172 (comparison controls;
GatherBy silent-singleton bug fixed) + 173 (fail-closed diagnostics) +
174 (SelectPositions/Duplicates/BinarySearch). Workspace 1345 → 1356;
reference +3. Left for later: operator forms for the three new builtins
(future matcher touch); `SubValuesHoldAll`-style hold operator forms
(units 8-10 hold design). Gate lessons added: review OUTPUT SEMANTICS,
not just green tests (171 Partition); rename any source file that
credits an inspiration source (174 `list_lookup`).

THAWED 2026-07-07 (perf moratorium lifted by the user): the unit-2
operator forms — `Take`/`Drop`/`Insert`/`Delete`/`TakeList`/
`Partition`/`Riffle`/`GatherBy`/`MaxBy`/`MinBy`/`TakeLargestBy`/
`TakeSmallestBy`/`DeleteDuplicatesBy`/`ReverseSortBy` — go as
brief-175 after 171, WITH the restored microbench A/B hard gate
(saved-binaries alternating-run protocol). The unit-1 leftovers
(`StringTake`/`Drop`/`Count`/`Position` operator forms) join 175 ONLY
if WL documentation confirms those forms exist; otherwise they stay a
separate design decision.

**UNIT 3 CLOSED (2026-07-07).** Landed: 176 audit + 177 (comparison
controls) + 178 (array predicate test/rank forms) + 179 (ten predicate
operator forms; `OrderedQ[p]`/`Lookup[key,default]` excluded on
data-form collision) + 180 (KeyGroupBy/AssociationKeyDeflatten+Flatten/
MergeNested). Workspace 1358 → 1366; reference +4. No silent-wrong-
answer bugs in this family. Cumulative: units 1-3 = 271 builtins
swept, all fixes compat (defaults byte-identical). Next: unit 4
(control+logic+rewrite, 63).

## Unit 4 (control+logic+rewrite) triage — 2026-07-07

Audit brief-181: 63/63, findings in
`usability-pass/control-logic-rewrite-findings.md`. Two standout
findings beyond routine gaps + routine batches:

- **`Exit` capability mislabel — USER DECISION PENDING.** `Exit` is
  `Capability::Pure` (default via `ordinary`), so `sandboxed()`
  installs it; a sandboxed computation can raise
  `EvalError::Exit`→`SessionRequestError::Exit`, a host-lifecycle
  request that a naive host could act on (DoS escape). Kernel is clean
  (signal only), but the LABEL is wrong — host lifecycle control is
  ambient authority, not Pure (same logic as Clock/Random in the
  FS-purity line). `Capability::Process` already exists. Recommended
  fix: relabel `Exit` to `Capability::Process`; sandboxed drops it,
  trusted CLI keeps it. Not dispatched until the user picks the
  capability.
- **`Implies` short-circuit — REJECTED (compat).** WL `Implies` is
  `{Protected}`, does NOT hold its args; `Implies[False, x]` evaluates
  `x`. rum matches WL. The audit's short-circuit proposal would
  diverge — rejected per the WL-conformant-defaults rule.
- **`Equivalent` Flat/OneIdentity — DEFERRED (verify WL first).** WL
  `Equivalent` is believed `{Orderless, Protected}` (not Flat); do not
  add associative attributes without confirming the WL surface.

Routine fix batches (dispatch order):

1. **brief-182 — rewrite operator forms** (call planning; A/B gate;
   the unit-2/3 spillover): `Cases[patt]`, `DeleteCases[patt]`,
   `Replace[rules]`, `ReplaceAll[rules]`, `ReplaceRepeated[rules]`. All
   data forms are ≥2-arg → 1-arg operator forms unambiguous.
2. **brief-183 — rewrite options** (pure-ish): `ReplaceRepeated`
   `MaxIterations -> Automatic` (per-call runaway budget; safety),
   `Replace` `Heads -> False` traversal (parity with Cases/DeleteCases).
3. **brief-184 — iteration diagnostics** (low-risk safety):
   `Sum`/`Product` malformed-iterator messages matching `Do`/`Table`.
4. **Exit capability** (pending user decision).

(NOTE — the batches were renumbered at dispatch: brief-183 = the
user-decided Exit capability; brief-184 = rewrite options; brief-185 =
Sum/Product diagnostics.)

**UNIT 4 CLOSED (2026-07-07).** Landed: 181 audit + 182 (five rewrite
operator forms, A/B clean) + 183 (Exit → Capability::Process, USER
decision; sandboxed() drops it, trusted CLI byte-identical) + 184
(rewrite options: `ReplaceRepeated` `MaxIterations -> Automatic` clamped
to the session ceiling with malformed-value fail-close, `Replace`
`Heads -> False` levelspec head traversal) + 185 (`Sum`/`Product`
`argform` parity with `Do`/`Table` on the shared `parse_do_spec` `None`
verdict; inert return byte-identical). Workspace 1366 → 1373; reference
unchanged (operator forms and options are not in the generated
reference). Cumulative: units 1-4 = 334 builtins swept, all fixes compat
(defaults byte-identical).

Close-out decisions (no further brief):
- **`Equivalent` Flat/OneIdentity — REJECTED (conservative).** rum
  registers `Equivalent` as `{Orderless}` (`logic.rs:39`). Adding
  `Flat`/`OneIdentity` is declined: the WL surface is unconfirmed and
  the testing policy forbids oracle-checking it; folk scope gains
  nothing from associative flattening of a biconditional; and the change
  would touch attribute-driven planning (an A/B gate) for zero
  user-facing benefit. Same disposition as the `Implies` short-circuit
  (rum matches WL `{Protected}`, non-held — kept). A SEPARATE
  attribute-hygiene question remains a future candidate (not this unit):
  rum's boolean heads diverge from WL more broadly — `Equivalent` and
  several others lack `Protected`, and `And`/`Or` are `{HoldAll}` in rum
  (short-circuit semantics) vs WL's `{Flat, OneIdentity, HoldAll,
  Protected}`; that HoldAll divergence is likely intentional and any
  touch there needs its own design pass.

Gate lesson (added to the standing set): a behavior-change brief's
MAY-modify list must span EVERY test file that asserts the old behavior,
across crates — brief-185's message addition rippled into a rum-stdlib
test (`holding_and_control_part5`) that encoded the old silence, outside
the runtime-only MAY list; updating it was necessary (a red tree
otherwise), not scope creep. Pre-include cross-crate behavior tests in
the MAY list next time.

## Unit 5 (math) triage — 2026-07-07

Audit brief-186: 62/62, findings in `usability-pass/math-findings.md`.
Six families (arithmetic 31 + number_theory 9 + combinatorics 11 +
integer_digits 5 + polynomials 5 + calculus 1). Folk-scope guardrail
held: Factor/Together/Expand/Floor/Ceiling/rationals-only correctly
marked OK-intended, not gaps. Reproduced the key findings myself
(coverage 62/62; `FromDigits[{2},2]->2` silent bug; zero-domain inerts;
`GCD`/`LCM`=`{Listable}`, `Max`/`Min`=`{}`).

**AUDIT CORRECTIONS + the USER singularity decision (2026-07-07):**
- **Arithmetic singularities stay INERT — folk-conservative (USER
  decision).** For `1/0`, `0^0`, `(-1)!`, and zero divisor/modulus/
  multiple in `Divide`/`Quotient`/`PowerMod`/`Round`, WL returns special
  values rum does not compute with (`ComplexInfinity`/`Indeterminate` are
  inert symbols only; source has zero references to them). The user chose
  the folk-conservative path: keep these INERT (today's behavior), emit
  NO domain-error message, introduce NO special-value arithmetic. The
  audit's "arithmetic-runtime fail-closed" batch is therefore RECLASSIFIED
  as OK-intended folk boundary rows, NOT gaps. Only clean WL-value cases
  are fixed (see below).
- **`FromDigits[{2}, 2] -> 2` is WL-conformant, not a bug.** WL's LIST
  form of `FromDigits` is a lenient base-weighted sum (no per-digit range
  check; only the STRING form is strict) — rum's `2` matches. Reclassified
  OK; the audit's #1 "silent bug" was a false gap.
- **Negative/zero args are NOT a blanket fail-close.** Where WL DEFINES a
  value, rum returns it (PartitionsP/Q[-1] -> 0). Where WL genuinely
  errors on a MALFORMED ARGUMENT (arity, base < 2, a non-variable
  derivative target), rum messages (matches its existing `Prime::intpp` /
  arity pattern — this is argument validation, not a singularity). DEFER
  where the WL surface is uncertain — never guess (Equivalent-close-out
  discipline; documented WL knowledge only, no oracle).
- **`Coefficient[x+1/x, x, 1]` -> `1` is a CORRECT value**, not a bug. The
  real defect is `CoefficientList[x+1/x, x]` -> `{x^-1, 1}` (a returned
  coefficient still contains the target variable). Fix CoefficientList
  target-containment; do NOT make scalar `Coefficient` reject valid
  queries.
- **Attribute-hygiene changes symbolic-arg OUTPUT.** Adding `Orderless`/
  `Flat`/`OneIdentity` to `GCD`/`LCM`/`Max`/`Min` is WL-correct but
  canonicalizes symbolic-arg ordering (`Max[y,x]` -> sorted) — a visible
  MOVE TOWARD WL-conformance (allowed by the compat directive), not a
  regression. Touches attribute-driven planning → A/B gate required.

Fix batches (dispatch order, reshaped by the singularity decision):

1. **brief-187 — argument/arity validation** (safety, S; WL clearly
   ERRORS on the malformed argument — NOT a singularity): `Fibonacci`
   arity (`Fibonacci[]`, `Fibonacci[1,2]`); invalid base (< 2) for
   `IntegerLength`/`IntegerDigits`/`IntegerReverse`/`DigitCount` and
   negative length for `IntegerDigits`; malformed `D` specs that are
   clearly not a valid variable/order (`D[x^2,1]`, `D[x^2,{x,-1}]`;
   DEFER the symbolic-order `D[x^2,{x,a}]`). rum-stdlib; mirror the
   existing arity/domain message machinery. No A/B.
2. **brief-188 — clean WL-value fixes** (folk-conservative correctness):
   `Mod[m,0] -> m`; `PartitionsP[-1] -> 0`, `PartitionsQ[-1] -> 0`; and
   any other combinatorics/number-theory negative/zero case where WL
   gives a plain integer rum can represent — per function, DEFER where
   uncertain. Small and careful; no messages, no special values.
3. **brief-189 — polynomial target-containment** (safety):
   `CoefficientList` target-containment guard for non-polynomial residues
   (Coefficient only where genuinely wrong). No new CAS.
4. **brief-190 — cheap flexibility**: `Total[list,n]`/`Total[list,
   {level}]` level forms, `Commonest[list,n]`, list-valued
   `Quantile[list, {q..}]`, negative-exponent `PowerMod` modular inverse
   (extended-GCD, message on non-invertible base). Folk-cheap only.
5. **brief-191 — attribute hygiene** (A/B gate): `GCD`/`LCM`/`Max`/`Min`
   grouping attributes (Orderless/Flat/OneIdentity, WL-correct); `Abs`
   and small `NumericFunction`/`Protected` omissions. Microbench A/B
   required (attribute-driven planning).

## Unit 3 (predicates+associations) triage — 2026-07-07

Audit brief-176: 83/83, findings in
`usability-pass/predicates-associations-findings.md`. Healthier family
than structure (no silent-wrong-answer bugs). Fix batches, dispatch
order (perf moratorium lifted, so operator-form batches allowed with
the A/B gate):

1. **brief-177 — comparison controls** (pure per-builtin, no call
   planning): `SameTest -> SameQ` for `DuplicateFreeQ`/`IntersectingQ`/
   `DisjointQ`/`SubsetQ`/`ContainsAll`/`ContainsAny`/`ContainsExactly`/
   `ContainsNone`/`ContainsOnly`; `OrderedQ[list, p]` ordering
   predicate; `LexicographicOrder` comparator; `KeySortBy[assoc, f, p]`
   3-arg (mirror `KeySort`'s comparator path). Register the SameTest
   defaults.
2. **brief-178 — array predicate forms** (pure): `VectorQ[expr, test]`,
   `MatrixQ[expr, test]`, `ArrayQ` shape-pattern + element-test forms.
3. **brief-179 — operator forms** (call planning; A/B gate, the 175
   pattern): `ContainsAll/Any/Exactly/None/Only[needles]`,
   `Between[range]`, `AllTrue/AnyTrue/NoneTrue[crit]`,
   `OrderedQ[p]`, `Lookup[key, default]`, and the kernel `MatchQ[patt]`
   (arity exemption in rum-runtime's MatchQ path). Disambiguation rule
   as in 175: only arities that cannot be a data form.
4. **brief-180 — WFR association additions** (new folk builtins,
   idea-level; `list_lookup`-style neutral naming): `KeyGroupBy`,
   `AssociationKeyDeflatten` + `AssociationKeyFlatten` twin,
   `MergeNested` (config layering — pairs with codecs + `ConformSchema`).

Corrections to audit findings (so fix briefs do not inherit them):
`LongestCommonSubsequence`'s contiguous behavior IS the WL contract
(the non-contiguous classic is WL's `LongestCommonSequence` — a
sibling-name candidate for a later batch); the `Encrypt` "add a real
authenticated method" proposal is rejected (policy);
language/normalization escape hatches (`Alphabet` beyond English,
`RemoveDiacritics` combining marks, `SequenceAlignment` scoring) defer
to a future unicode line (unblocked by the feature skeleton).
