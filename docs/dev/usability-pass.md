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
| 5 | arithmetic + number_theory + combinatorics + integer_digits + polynomials + calculus | 62 | brief-186 ✓ | 187-193 ✓ CLOSED |
| 6 | codecs (json/csv/ini/toml/xml/yaml/schema) + bytes + datetime | 34 | brief-194 ✓ | 195-199 ✓ CLOSED |
| 7 | character_classes + random (light re-check; 157-159 fresh) | 16 | brief-203 ✓ | 204-207 ✓ CLOSED |
| 8 | kernel A: definitions/values/attributes/protection | 55 | brief-208 ✓ | 209-213 ✓ CLOSED |
| 9 | kernel B: scoping/eval control/patterns/forms | 47 | brief-214 ✓ | |
| 10 | kernel C: system state/contexts/packages/paths/diagnostics/tests | 57 | | |

Counts for units 1-7 are exact (catalog arrays, 446 total — unit 7 = 16
after brief-201 split off `ReseedRandom`).

**Kernel list derived 2026-07-12** (it was "~160, enumerated at audit
time"): installed reference 612 − stdlib catalog names 453 = **159 kernel
builtins**. Unit 8 takes **55** of them (assignment 7, compound assignment
11, value tables + introspection + removal 12, attributes + protection 10,
options 4, messages 11 — the authoritative list is in brief-208); **104
remain** for units 9-10, split when their audit briefs are written.

## Unit 9 (kernel B) — pre-audit findings + split, 2026-07-12

Claude derived the unit-9/10 split mechanically and probed before writing
brief-214; the brief verifies rather than re-derives these.

- **Split derivation** (reproducible): reference 612 − stdlib catalog 453 =
  159 kernel − 55 (unit 8) = 104. Unit 9 = **47** (scoping 5, eval
  control/hold 12, pattern atoms + rule heads 12, structural/pattern-query
  11, forms + markers 7 — authoritative list in brief-214); unit 10 = the
  **57**-name complement (also pinned in brief-214). Registration spread:
  `builtin_catalog/` scoping.rs 12, rules.rs 12, strings.rs 13,
  namespace.rs 4, functional.rs 2, debug.rs 3, associations.rs 1.
- **`Protected` is DONE for this unit** — all 47 already carry it
  (brief-210's sweep). The battleground is SEMANTICS, not protection; no
  whole-registry section this time.
- **One real WL divergence found at probe time**: `f[Unevaluated[1 + 1]]` →
  `f[2]` for undefined `f` — WL restores the `Unevaluated` wrapper when no
  definition consumes the argument; rum evaluates the content into the
  surviving form. The BUILTIN path is correct (`Length[Unevaluated[1 + 1]]`
  → `2`, `Head[...]` → `Plus`). Brief-214 asks for the precise boundary +
  the missing-restore `file:line`.
- **Green on basics**: Module gensym (`x$1`), Block/With/Unique; `Evaluate`
  pierces `Hold` but not `HoldComplete` (WL-correct); `HoldPattern` in the
  matcher; `Repeated` count forms; `Longest`/`Shortest` in StringCases;
  `ReplaceAt` (WL-13.1 shape); Part span; Thread; Position; Level `{-1}`.
- **Two hard-coded-hold traces ordered** (the brief-208 `Set` discipline):
  rum's `Evaluate` carries `HoldAll` (WL: no hold attribute) — load-bearing
  or vestigial?; `Rule`/`RuleDelayed` carry no hold attribute yet behave
  WL-correctly (rhs held; `Rule[a, Sequence[1, 2]]` does not splice).
  CORRECTED BY THE AUDIT: the brief's premise "`SequenceHold` does not
  exist in rum's `Attribute` enum" was STALE — `SequenceHold` (and
  `NHoldAll`) are in the enum (`runtime/metadata.rs:32-48`, verified from
  source at the gate). The durable fact is narrower: the special-form
  dispatch path for `Rule`/`RuleDelayed` never consults those attributes,
  so their hold behavior is still evaluator-coded, and adding the
  attributes would be dead-weight metadata (audit trace confirmed).
- **Design-input duty wired in**: the parked units-8-10 hold design
  (`SubValuesHoldAll`-equivalent, `ReplaceAllUnheld`) gets a dedicated
  factual appendix in the findings file (how held args flow through
  matching/replacement today). Record, not design.

## Unit 9 (kernel B) triage — 2026-07-12

Audit brief-214: 47/47 (COVERAGE-OK), findings in
`usability-pass/kernel-b-findings.md`. Gate: scope clean (findings file
only), fmt clean, `cargo test` NOT rerun (docs-only diff, no code touched),
six load-bearing claims reproduced by hand at the gate (Module leak probe
verbatim; `Unevaluated` both faces; `HoldAll`-blocks-`Sequence`;
`ReleaseHold[HoldForm[…]]` miss; the E7 consumer-divergence probe incl.
`c = 110` and the `HoldAllComplete` splice bypass; `ReplacePart` Span
silent no-op + the reference's explicit Span promise). The audit also
CORRECTED a stale brief premise (`SequenceHold` exists — see the pre-audit
section above). Semantics, not protection, is the battleground: all 47
were already `Protected`.

**USER RULINGS (2026-07-13) on the two decision items:** (1) Module
error-path gensym leak → **eager reclamation** (= WL semantics; the Err
arm reuses the Ok arm's retained/reclaim computation with
`gensym_gc_extra_roots_for_outcome` as roots; escaped guard unchanged) =
**brief-220**. (2) staged `With[{…},{…},body]` → **build it**, as its own
batch immediately after the Module fix = **brief-221**. Dispatch order:
215 → 220 → 221 → 216 → 217 → 218 → 219.

### Unit-9 fixes landed

- **batch 4 — brief-218 (cd463b7):** pattern-query operator forms.
  `Position[patt]`/`Count[patt]`/`MemberQ[patt]`/`FreeQ[patt]` join the
  callable-wrapper family tables (`functional_data.rs`, mirroring the
  179/182 siblings; `Extract[pos]` was the in-tree precedent); 1-arg
  gates in `runtime/tree_walkers/` yield to the wrapper arm; 0-arg
  `argtu` + all data forms byte-identical. Gate: workspace 1452/0;
  probes; hashes matched both ways; independent A/B watch ≤ +1.8% in
  ±3.5..3.9% floor — Codex's honestly-reported `table-count-null` +7µs
  outlier did NOT reproduce (-1.84%) = noise, closed. Brief errata:
  tree_walkers path was stale (lives under `runtime/`, not
  `kernel_data/rewrite_data/`).
- **batch 3 — brief-217 rev2 (027dc74):** fail-closed contracts. (a)
  `Repeated`/`RepeatedNull` count shapes unified across expression and
  string matchers (numeric `n` was silently inert on strings); malformed
  bounds → `::range` + inert via a typed compile error reported by the
  session-aware caller (compile stays pure; Verbatim-guarded walk). (b)
  `ReplacePart` Span implemented (reference already promised it; reuses
  Part span selectors); malformed span → `ReplacePart::span` + inert;
  valid-but-out-of-range = pinned lenient-ignore (mirrors exact form).
  (c) `Names` `SpellingCorrection` non-False → `Names::optval` + inert.
  (d) `Once` unsupported expiration values → `Once::pers` + inert
  WITHOUT evaluating the held expr; stays `Pure`. Valid inputs
  byte-identical everywhere. rev1 stop-lossed on brief defects: the MAY
  seams were session-less pure code (diagnostics need the resolution
  caller) and the "mirror the existing failure shape" instruction
  pointed at silences — REV2 LESSON (with 216's): before instructing
  "mirror X", baseline-verify X exists. Gate: workspace 1448/0; probes;
  same-path binary hashes matched BOTH ways (repo-path build note
  works); independent A/B — watch labels ≤ +1.3% inside ±3.6..5.9%
  floor, and Codex's loop-empty +48% anomaly did NOT reproduce (+0.46%
  here) = its startup-floor-noise classification confirmed.
- **ruling fix 3 — brief-222 (8333a44):** replacement-produced `Sequence`
  splices into the parent rebuild (USER-RULED; the boundary 216 pinned).
  `ReplaceAllTraversalOutcome`/`ReplaceOutcome` thread a `changed` flag
  (continuation-serializable); a rewrite-CHANGED argument whose result is
  a `Sequence` call splices into the rebuilt parent unless the parent has
  `sequence_hold()`: `Hold[g[a]] /. a -> Sequence[1,2]` → `Hold[g[1,2]]`.
  Unchanged args (incl. pre-existing literal Sequences in held data) come
  back byte-identical; attribute lookup only runs on a changed
  Sequence-headed arg. Pinned-not-WL: `sh`/`hc` parents keep nesting; top
  level unchanged. `//.`/levelspec `Replace` consistent (once.rs shares
  the splice helper + recomputes spliced depth); `ReplacePart[…, pos ->
  Sequence[…]]` already spliced (recorded, untouched); 216's
  literal-in-RHS contract intact. Gate: workspace 1442/0; probe panel;
  Codex hit the load stop-loss twice (Spotlight) and shipped no numbers —
  correct; its parked binaries mismatched my rebuild (ITS OWN temp build
  path — build-path bytes; same-path stash rebuild matched 216's HEAD
  build byte-for-byte = determinism confirmed), so the A/B ran on my
  same-path pair: `replaceall-simple` +2.61% vs controls ±4..7% noise
  floor, no regression. GATE NOTE for future briefs: have Codex build
  A/B binaries IN THE REPO PATH so checksums are comparable.
- **batch 2 — brief-216 rev2 (606b96f):** rule firing in held replacement.
  (a) `ReplacePart`/`ReplaceAt` insert the binding-substituted delayed RHS
  UNEVALUATED (pure deletions — delayed rules join the shared plan path);
  held positions keep the raw CompoundExpression, side effects don't fire
  (E7 panel: `c` = 0 across all three consumers), unheld positions and
  immediate rules unchanged. (b) RHS materialization splices `Sequence`
  ONLY for a directly bound pattern symbol; literal `Sequence` calls stay
  one argument (`a /. (a :> hc[Sequence[1,2]])` → nested; ordinary heads
  splice at evaluation). `replace_all.rs` untouched. **rev1 stop-lossed on
  MY error**: a WL-derived "must not change" pin never baseline-verified
  (`Hold[g[a]] /. a -> Sequence[1,2]` — WL splices, rum never did).
  STANDING LESSON: baseline-verify every preservation pin; WL-correct ≠
  rum-current. **USER RULING #3 (2026-07-13): fix that divergence = 
  brief-222** (substitution is the only way to splice args into held code
  + rewrite output should have its runtime shape; rum's eval-only-splice
  was an unruled implementation artifact). Gate: workspace 1438/0; 12-
  position probe panel; A/B run by the director after the user idled the
  machine (Codex correctly REFUSED to benchmark under a VM's load and
  parked the binaries) — `replaceall-simple` -2.98%, controls +7.6/-3.9%
  noise floor, no regression.
- **decision fix 2 — brief-221 (c204050):** staged `With[{…}, {…}, body]`
  (USER-RULED). Arity ≥3 = nested-With equivalent via a structural
  desugar in a new `eval_staged_with`; 2-arg path byte-identical (the
  staged branch sits inside the `len != 2` error case). Pre-validation is
  structural-only — `compile_with_call_from_raw_args` parses symbol/raw-
  rhs pairs without evaluating, so initializer side effects fire exactly
  once (gate-probed: `c0 = 1`) and a malformed later list fails closed
  with the SAME `argform` text as the 2-arg form (test asserts message
  equality) BEFORE any substitution. Same-list sequential contract +
  `With[]`/`With[body]` argx pinned unchanged; shadowing across stages =
  nested rebinding (`With[{x=1},{x=x+1},x]` → 2, scope-aware
  substitution). Gate: workspace 1429/0 reproduced; probes + an extra
  side-effect single-fire probe; candidate = tree build; independent A/B
  `with-local` +0.76% inside ±5.7% noise. mma-hol suite WAIVED (no gensym
  machinery touched; 2-arg path structurally unchanged) — stated per the
  honesty rule.
- **decision fix 1 — brief-220 rev2 (3dd9d3f):** Module error-path gensym
  reclamation (USER-RULED: eager = WL semantics). `finish_module_scope`
  runs the shared `module_gensyms_to_reclaim` on BOTH arms — Ok passes
  `[result]` (byte-identical work), Err passes
  `gensym_gc_extra_roots_for_outcome(&outcome)`; escaped guard unchanged.
  Unreferenced locals now reclaimed immediately on Throw/Abort/Return/
  EvalError; payload escapes (`Throw[Hold[x]]`) survive; nested modules
  mirror the Ok arm's conservatism (inner reclaimed, outer deferred — the
  rev1 nested contract demanded MORE eagerness on Err than Ok has, Codex
  stop-lossed correctly, and the director verified on HEAD that Ok defers
  the outer local too before revising). Gate: workspace 1426/0 reproduced;
  probes reproduced; independent A/B (all six scoping watch labels
  negative, module-local -6.8%, with-local -7.8%; controls ±8..19% =
  noise floor); **mma-hol Strict FULL suite on rum 3126/0** (~127 min,
  RSS <1GB). **GATE TRAP FOUND: `FileNames` is not an installed rum
  builtin**, so the acceptance suite's post-bootstrap discovery
  (`Complement[FileNames[…], …]`) silently no-ops — a first "green" run
  had executed only kernel_tests (71 assertions) + the load chain while
  printing `failed: 0`/exit 0. Fixed by patching the SCRATCH copy's
  run_all.wls with an explicit 38-file list (procedure memory:
  `mma-hol-strict-gate-procedure`). `FileNames` absence = a unit-10
  paths-family data point (absent builtin silently no-ops a read-only
  discovery idiom). PARKED from rev1: activation-bookkeeping precision
  (descendant gensym writes mark ancestor activations escaped on both
  arms; fixing would benefit both — own design pass, not this line).
- **batch 1 — brief-215 (02ce0a8):** evaluator hold correctness.
  (a) `Unevaluated` restored on the true no-consumer path — sparse
  `(position, wrapper)` provenance recorded ONLY when a wrapper is
  stripped (`Option<Box<UnevaluatedRestorePlan>>`, pointer-sized per
  dispatch frame), applied only in terminal no-match reconstruction;
  kills `f[Unevaluated[1+1]]` → `f[2]` AND the worse false-match
  (`q[x_Integer]:=hit[x]` no longer fires). Fixed point asserted.
  (b) ordinary `HoldAll` now allows raw `Sequence` splicing
  (`Hold[Sequence[a,b]]` → `Hold[a,b]`); blocking = `sequence_hold()`
  exactly; stability wakes held literals containing a direct Sequence.
  (c) `ReleaseHold` gains `HoldForm` + releases exactly ONE outer layer;
  deep positions unchanged (reported boundary: `ReleaseHold[g[Hold[…]]]`
  inert as before — triage later if wanted). Pinned-not-WL edge
  documented in test: `Unevaluated[Sequence[…]]` provenance dropped after
  splicing. Gate: workspace 1422/0 reproduced; 16-position probe list
  reproduced verbatim; candidate binary byte-identical to tree; A/B
  reproduced independently (affected no-match labels -0.4..-3.4%, hit
  labels +1.6..+3.1%, controls ±5..7.6% = noise floor above delta).
  NOTE: Codex's FIRST provenance carrier fattened the dispatch frame and
  blew the near-stack-limit recursion stress test — it correctly fixed
  the FRAME COST (pointer-sized carrier), not the guard; verified no
  guard/stress files in the diff. Pre-existing debt spotted: dead
  `source_template` field warning in `vm_rhs_template/model.rs` (VM
  line, untouched); old `PrepareArguments` arm in `eval_dispatch/mod.rs`
  is now dead code (harmless hygiene item).

- **Module error-path gensym leak (trust boundary).** After
  `Catch[Module[{auditLocal = 1}, Throw[fail]]]`, the fresh local remains
  enumerable AND readable (`Names` shows it; `auditLocal$1` → `1`).
  `finish_module_scope` reclaims immediately only on `Ok`; error-path
  gensyms wait for the GC sweep (default threshold 200,000 —
  `module.rs:207-280`, `gensym_gc.rs:164-172`). Module gensym is the
  LCF-style encapsulation boundary for downstream mma-hol, so the fix
  choice (eager reclamation on error exits / lower GC threshold /
  accept-and-document) is the user's; any evaluator fix here also needs
  the mma-hol Strict-mode suite gate + A/B (standing rule).
- **Staged `With[{…}, {…}, body]` form (WL 12.2+).** Missing; arity-3 is
  an inert `argx` loop today. Bolting it on touches the same scoping
  special form; decide together with (or after) the Module ruling.

Fix batches (dispatch order — safety first; every WL-name default change
below is a move TOWARD documented WL semantics, the unit-5-sanctioned
class):

1. **brief-215 — evaluator hold correctness** (safety, L; ONE A/B gate for
   the batch): (a) `Unevaluated` wrapper restore on the true no-consumer
   path — today undefined heads lose it (`f[2]`) and, worse, a
   nonmatching definition can then FIRE (`q[x_Integer]` catches
   `Unevaluated[1+1]` → `hit[2]`); provenance must be recorded only when
   a wrapper is actually stripped (zero default-path cost), restore only
   in no-match reconstruction (`eval_call_prepare.rs:50-55`,
   `eval_core/call.rs:97-101`). (b) ordinary `HoldAll` must ALLOW raw
   `Sequence` splicing (WL: only `SequenceHold`/`HoldAllComplete` block);
   today `allow_sequence_splice:false` is hard-wired to holding. (c)
   `ReleaseHold` gains `HoldForm` (its own supported wrapper today
   missing) and the layer contract is pinned to WL's ONE layer per call
   (today it strips ALL consecutive layers: `ReleaseHold[Hold[Hold[x]]]`
   evaluates to the value — reproduced at the gate). MAY list must span
   every test asserting old non-splice / multi-strip behavior
   (cross-crate lesson).
2. **brief-216 — rule firing in held replacement** (safety, L; A/B):
   align `ReplacePart`/`ReplaceAt` delayed-RHS to WL insert-unevaluated
   (today both evaluate before insertion while `ReplaceAll` correctly
   inserts raw); stop substitution-time `Sequence` splicing from
   bypassing `HoldAllComplete` destinations
   (`binding_substitution.rs:281-323`). Record — NOT design —
   `ReplaceAllUnheld` implications (the design-input appendix in the
   findings file is the base).
3. **brief-217 — fail-closed / truthful contracts** (safety/flexibility,
   M): unify `Repeated`/`RepeatedNull` count shapes across expression and
   string matchers (string side rejects numeric `n`) + diagnose malformed
   bounds (matcher portion → A/B; the rest is builtin-local); implement
   the reference-PROMISED `ReplacePart` Span positions; reject
   `SpellingCorrection -> True` (advertised, never read); fail closed on
   unsupported known `Once` expiration values (keep `Pure`, no clock).
4. **brief-218 — pattern-query operator forms** (flexibility, M; A/B):
   `Position[patt]` / `Count[patt]` / `MemberQ[patt]` / `FreeQ[patt]`
   (WL-documented; the unit-1 deferral now unblocked; `Extract[pos]`
   already exists).
5. **brief-219 — presentation truthfulness** (convenience, S/M):
   terminal value rendering uses the existing transparent `HoldForm`
   output arm (today CLI prints the literal wrapper while `ToString`
   output-forms hide it); reference summaries for
   `InputForm`/`StandardForm` name their implemented `ToString` consumer
   (docgen MERGE, not replace — the brief-198 lesson). Renderer/docs
   only; no evaluator A/B if kept out of evaluation.

Close-out notes (no brief): `Rule`/`RuleDelayed` attribute additions
REJECTED (special-form path never consults them — dead metadata);
`Evaluate`'s `HoldAll` KEPT (load-bearing for standalone calls:
`builtin_adapters.rs:771-781`); `Nothing` list-only removal, `Sow` outside
`Reap`, `Once` cache lifecycle (session-owned, snapshot/reset-correct),
sequential `Block`/`With` initializers all confirmed OK-intended;
`Function` surplus-args behavior is the established contract.

## Unit 8 (kernel A) — pre-audit findings, 2026-07-12

Claude probed before writing brief-208; these are the spine of the unit and
brief-208 verifies rather than re-derives them.

- The `Protected` **enforcement point works**: `RandomInteger[x_] := 99` →
  `! SetDelayed::wrsym: Symbol RandomInteger is Protected.` (brief-204 gave
  it the attribute).
- The `Protected` **enforcement surface is 129 of 612** — 483 installed
  builtins carry no `Protected`. `Plus[a_, b_] := 42` SUCCEEDS, after which
  `1 + 2` → `42`; same for `Times`/`Power`/`Equal`/`If`/`Map`/`Part`.
- **All 55 unit-8 names lack `Protected`, the protection machinery
  included**: `Attributes[Set]` = `{}`, `Attributes[Protect]` = `{HoldAll}`,
  and `Unprotect[q_] := 7` succeeds.
- The three prior `Protected` sweeps (162b pattern symbols, 196 codecs, 204
  random) were each unit-local; the surface was never swept whole. WL
  protects essentially every `System\`` builtin, so this is an
  attribute-hygiene / WL-conformance gap — and `Protected` is zero
  eval-cost (checked only on the definition-mutation path), unlike the
  call-planning attributes brief-193 rejected.
- **Honest boundary**: `Protected` is a mistake-guard, NOT a sandbox
  boundary — a program may always `Unprotect[Plus]`. Real plugin isolation
  (per-request definition rollback) belongs to the host-embedding line,
  where it is already on the gap list. Do not conflate the two.
- Instrument warning: `docs/reference/builtins.md` backticks each attribute
  separately (`` `Listable`, `Protected` ``); a regex capturing only the
  first token under-reports (Claude's first count said 527 unprotected —
  wrong). Calibrate before trusting a count.

USER RULING (2026-07-12): fold this into the unit-8 audit — brief-208
carries a dedicated whole-registry `Protected` coverage section, and the
mechanical cross-cutting fix batch is triaged out of its data (rather than
inserting a blind fix batch first, or deferring the whole thing to the host
line).

### Unit-8 fixes landed (2026-07-12)

- **batch 1a — brief-209 (b0c7113):** REAL bug found at the gate. `ApplyTo[x, f]`
  where `f` protects `x` mid-update landed the write on a visibly `Protected`
  symbol (`x = 101`, no message); root cause = `store_held_update` did not
  re-run the protection guard the check step uses, and `ApplyTo`'s update
  (`f[current]`) runs user code in that window. Fixed by re-running the guard
  at the store step (Symbol + Part owner). The other ten compound assignments
  share the path but evaluate no user code in the window (fail closed). NOTE:
  my first gate pass wrongly ruled this "false" from a too-weak probe — see the
  CLAUDE.md diagnosis lesson and [[usability-pass-line]].
- **batch 2 — brief-210 (5872796):** whole-registry `Protected` sweep,
  129/612 → 608/612. Only `$AssertFunction`/`$ContextPath`/`$Packages`/`$Path`
  stay open (writable session vars). One `BuiltinSpec::protected()` const
  builder + `.protected()` per catalog entry + mirror lockstep + regenerated
  reference; constructor defaults unchanged (downstream plugins still choose).
  WL-conformant + reversible. Completeness proven by SWEEP-OK (exactly the 4
  exclusions unprotected); workspace 1400/0. Four tests updated for the new
  attribute sets (each value verified empirically — the test-control session's
  `If`/`Plus` keep harness attrs while `Identity`/`HoldAllComplete` fall
  through to the now-protected catalog).
- **batch 3 — brief-211 (d6cbd34):** closed the three protection-enforcement
  holes. (a) The `$ContextPath`/`$Packages`/`$Path` special setters
  (`definitions.rs`) returned before the guard — now check first, still open by
  default (guard fires only after an explicit `Protect`). (b) `SetAttributes`/
  `ClearAttributes` (`attributes.rs`) skip a protected target UNLESS the op
  removes `Protected` — so `ClearAttributes[f, Protected]` and combined removal
  stay the escape, alongside `Unprotect` (separate path). (c) `SetOptions`
  (`options/builtins.rs`) guards before writing — the hole was reachable only
  with a KNOWN option (the `optnf` unknown-option check had masked it; I
  confirmed it at brief time with `SetOptions[FromCsv, Delimiter -> ";"]`). All
  reuse `ensure_symbol_not_protected_for_write`. Five regression tests + nine
  `Unprotect`-ripple edits (existing tests doing option/attribute writes on
  now-protected system symbols). Workspace 1405/0.
- **batch 4 — brief-212 (87b9067):** round-trippable value-table assignment.
  `OwnValues`/`DownValues`/`UpValues`/`SubValues[f] = rules` install the rules
  as `f`'s table (atomic replace). **Scope correction found at diagnosis time:**
  the audit's E7 "silent false round-trip" had ALREADY been closed by batch 2 —
  once the inspector heads became `Protected`, the assignment failed loudly
  (`Set::wrsym`) instead of silently defining the inspector. So batch 4 is the
  FEATURE half (real WL round-trip), not a safety fix; the user chose to build
  it. New `value_table_assignment.rs` reuses `store_definition_target` (the
  TagSet/UpSet install that compiles matchers + derived fields — hand-rolling a
  partial install would silently break matching) and `classify_tagged_definition`
  (head-matching identical to `f /: … := …`). Contract: parse+validate the whole
  list before ANY mutation → a bad later rule can never leave a partial table;
  `<Inspector>::setval` names the offending rule. Cross-symbol copy fails closed
  (rules' lhs head is `f`, not `g` — it could never fire); working form is
  `DownValues[g] = (DownValues[f] /. f -> g)`. DefaultValues deferred (not a
  `TaggedDefinitionTarget` kind). Workspace 1415/0 (10 new tests). A/B: the
  affected path (immediate `head[args]=value`) was raw-faster on the candidate;
  the UNAFFECTED controls moved more (+3.6% / +1.7% vs +1.8%), so the noise floor
  sits above the delta — no regression.
- **batch 5 — brief-213 (868d2c6), closing:** (a) wildcard name-pattern targets.
  `Clear`/`ClearAll`/`Remove`/`Protect`/`Unprotect` share ONE normalizer, and the
  wildcard matcher already existed (it is what `Names` uses) — so one
  `normalize_clear_target_expr` gave all five pattern support. Per-target
  protection preserved (a `Protected` match emits `wrsym` and survives while
  unprotected matches are still cleared); a no-match pattern is a silent no-op
  and creates no literal wildcard-named symbol. Note `@` is NOT a rum wildcard
  (no `@` handling in `names.rs`), so keying on `*` matches the matcher's real
  capability. (b) `Information` now carries `{HoldFirst, Protected}` — it used
  to EVALUATE its argument (`Information[xq]` → inert `Information[5]`) while
  its sibling `Definition` already held correctly; plus the builtin catalog
  summary as the `Usage` fallback (was the bare qualified name) and a new
  `Messages` field. Workspace 1418/0; reference regenerated.
- **DEFERRED — builtin message-tag registry.** `Messages[Set]` / `MessageTags[Set]`
  are empty even right after `Set::argx` fires: builtin diagnostics come from
  ~108 ad-hoc `push_message` sites with NO static catalog of a builtin's possible
  tags/templates. Building one is its own project (a design + a sweep of every
  call site) and it pairs naturally with docgen — the builtin reference should
  list each builtin's message tags from the same source. Batch 5's new
  `Messages` field lights up for free once the registry exists; builtins
  honestly report an empty list today. **Candidate for a later line, or for the
  units 9-10 / docgen work.**

**UNIT 8 CLOSED (2026-07-12).** 208 audit (55 names, 55/55) + fixes 209-213.
Cumulative units 1-8 = **501 builtins swept** (446 stdlib + 55 kernel A).
Kernel list derived exactly: reference 612 − stdlib catalog 453 = 159 kernel;
unit 8 took 55, leaving **104 for units 9-10**.

The unit's arc, in one line each:
- **The audit found the protection surface was 129/612** — `Plus[a_,b_] := 42`
  succeeded and `1+2` then returned `42`.
- **209 (batch 1a)** — a REAL bug surfaced at the gate: `ApplyTo[x, f]` where `f`
  protects `x` mid-update landed the write on a visibly `Protected` symbol
  (`store_held_update` skipped the recheck; `ApplyTo` is the one compound
  assignment whose update runs user code in that window). My first gate pass
  wrongly ruled it "false" from a too-weak probe — see the CLAUDE.md lesson.
- **210 (batch 2)** — whole-registry `Protected` sweep, 129/612 → **608/612**
  (only the four writable session vars stay open).
- **211 (batch 3)** — the three enforcement holes closed (`$ContextPath`/
  `$Packages`/`$Path` setters, `SetAttributes`, `SetOptions`), escape hatches
  (`Unprotect`, `ClearAttributes[f, Protected]`) preserved.
- **212 (batch 4)** — round-trippable value tables (the WL feature; batch 2 had
  already killed the silent false round-trip).
- **213 (batch 5)** — name patterns + `Information`.

Next: **unit 9** (kernel B: scoping / eval control / patterns / forms), then
unit 10, THEN the dun host-embedding line (user sequencing ruling 2026-07-08).

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

(At dispatch, the cheap-flexibility batch above was split by feature:
190 = `Commonest[list,n]` + `Quantile[list,{q..}]`, 191 = `Total` level
forms, 192 = `PowerMod` modular inverse; the attribute batch became 193.)

**UNIT 5 CLOSED (2026-07-07).** Landed: 186 audit + 187 (argument/arity
validation: Fibonacci arity, integer_digits base<2 / negative length, D
malformed specs) + 188 (clean WL values: `Mod[m,0]->m`,
`PartitionsP/Q[-1]->0`) + 189 (Coefficient/CoefficientList
target-containment guard — the precise Laurent-leak version, not a naive
contains-target) + 190 (`Commonest[list,n]`, `Quantile[list,{q..}]`) +
191 (Total level forms n / {k} / Infinity) + 192 (PowerMod
negative-exponent modular inverse via extended Euclid) + 193 (attribute
hygiene, CONSERVATIVE). Workspace 1373 -> 1385; reference: five
Attributes lines (Abs/GCD/LCM/Max/Min). Cumulative units 1-5 = 396
builtins swept, all fixes compat.

Close-out decisions recorded (no further brief):
- **Arithmetic singularities stay INERT (USER decision).** 1/0, 0^0,
  (-1)!, zero divisor/modulus/multiple: no domain message, no
  ComplexInfinity/Indeterminate arithmetic (rum has them only as inert
  symbols). Memory: [[rum-rationals-only-by-design]].
- **`Orderless`/`Flat`/`OneIdentity` on GCD/LCM/Max/Min — REJECTED
  (no-default-cost).** rum honors them by per-call arg sort/flatten in
  `eval_call_prepare`; adding them to folk numeric reducers is hot-path
  cost for a marginal symbolic-canonicalization benefit and would flip
  the pinned `FullForm[Max[x,3]]=="Max[x, 3]"` ordering. Only the
  zero-cost `Protected` + `NumericFunction` were added (brief-193). Same
  disposition class as the unit-4 Equivalent Flat/OneIdentity rejection.
- **`FromDigits[{2},2]->2` is WL-conformant** (lenient list form), not a
  gap; several arithmetic-domain "safety" audit rows were reclassified to
  OK-intended folk boundaries by the singularity decision.

Gate lesson reinforced: a behavior/attribute change's MAY list must span
the RUNTIME-side mirror too — brief-193's catalog attribute change
rippled into `runtime/builtin_catalog/stdlib_attributes.rs` (the kernel's
attribute table), outside the stdlib-only MAY list; the sync was
necessary, not scope creep (cf. the brief-185 cross-crate lesson).

## Unit 6 (codecs+bytes+datetime) triage — 2026-07-07

Audit brief-194: 34/34, findings in
`usability-pass/codecs-bytes-datetime-findings.md`. Nine families (json 2 +
csv 2 + ini 2 + toml 2 + xml 3 + yaml 2 + schema 2 + bytes 11 + datetime 8).
The healthiest unit so far: the highest-value axis this unit — fail-closed
parsing — held everywhere. Every `From*` parser and decoder fails closed
with a message + `$Failed` on malformed input (reproduced myself: `FromJson`
malformed/dup-key, `ToJson[1/3]` non-terminating, CSV/INI/TOML/XML/YAML
malformed, `FromHexString`/`FromBase64`/`FromDateString` invalid, out-of-range
date lists). The `Capability::Clock` gate on the `DateList[]`/`DateString[]`
now-forms is correct (rum-host test `sandboxed_..._blocks_datetime_current_
readers...` passes; with-arg deterministic forms stay available). Folk-codec /
rationals-only / determinism / no-ambient-clock boundaries all correctly
marked OK-intended, not gaps.

**ONE real behavioral safety finding** (reproduced): digest heads
`Sha256`/`Sha512`/`Sha1`/`Md5` return an INERT call on an unknown output-format
second argument (`Sha256["abc", "Base64"] -> Sha256["abc", "Base64"]`) instead
of failing closed — the generic format parser in `bytes/digest.rs:75`. The rest
are hygiene: all 34 heads lack `Protected` (`ConformSchema`/`SchemaErrors` carry
`HoldRest` — preserve it); `Crc32` lacks the non-security Limits note that
`Md5`/`Sha1` have; implemented options are not `Options[...]`-discoverable;
datetime ergonomics (`DateString` format enum, `DayOfWeek` output selector);
operator forms for the pipeline verbs. No user-decision-level item this unit.

Fix batches (dispatch order — safety first):

1. **brief-195 — digest fail-closed + Crc32 note** (safety, S; the only
   behavior change): unknown digest output format → message + `$Failed`
   (matches the fail-closed principle), not inert. Add the `Crc32`
   non-security Limits note (parity with `Md5`/`Sha1`). rum-stdlib
   (`bytes/digest.rs`) + docgen/reference. MAY list must span any test that
   asserts the old inert form (cross-crate lesson). No call planning, no A/B.
2. **brief-196 — Protected sweep** (safety/convenience, S): add `Protected`
   to all 34 codec/bytes/date heads, PRESERVING `HoldRest` on
   `ConformSchema`/`SchemaErrors`. `Protected` is not attribute-driven
   planning (no `eval_call_prepare` arg reshaping like `Flat`/`Orderless`) —
   NO A/B — but the catalog change ripples into
   `runtime/builtin_catalog/stdlib_attributes.rs` (the brief-193 gate lesson;
   pre-include it in MAY). Same class as brief-162/193.
3. **brief-197 — option discovery registration** (flexibility/convenience,
   S): register the already-implemented option defaults for discovery via
   `Options[...]` — `ToJson` `"Indent"`, `FromCsv`/`ToCsv`
   `"Delimiter"`/`"Headers"`, `ToXml` `"Indent"`,
   `ConformSchema`/`SchemaErrors` `"AllowExtraKeys"`. The unit-2/3 discovery
   pattern: implementations already read options directly, the defaults table
   feeds only `Options`/`OptionValue`, so behavior is byte-identical. No call
   planning.
4. **brief-198 — datetime ergonomics** (flexibility, S/M): `DateString[date,
   fmt]` with a documented format enum (`"ISODateTime"`/`"ISODate"`/
   `"ISOTime"`, fail-closed on unknown name), `DayOfWeek[date, sel]` output
   selector (`"Name"`/`"ISOInteger"` or equiv). Keep UTC, exact seconds, no
   locale/time-zone database, fixed-duration-only arithmetic.
5. **brief-199 — pipeline operator forms** (flexibility, M; A/B GATE):
   `DatePlus[spec]`, `ConformSchema[schema]`, `SchemaErrors[schema]` — unary
   spec/schema forms are not data forms (E11: they currently fail at wrapper
   construction). Touches call planning → core-language microbench A/B (the
   175/182 protocol).

**UNIT 6 CLOSED (2026-07-08).** Landed: 194 audit + 195 (digest
fail-closed on unknown output format + `"Hex"` alias + `Crc32` non-security
note) + 196 (`Protected` on all 34 heads; `HoldRest` preserved on the two
schema heads; runtime mirror + drift test) + 197 (option-discovery
registration for `ToJson`/`ToXml`/`FromCsv`/`ToCsv`/`ConformSchema`/
`SchemaErrors`) + 198 (`DateString[date, fmt]` ISODateTime/ISODate/ISOTime +
`DayOfWeek[date, sel]` Name/ISOInteger) + 199 (`DatePlus[spec]` operator
form, A/B clean). Workspace 1385 → ~1391; reference: `Crc32` +
DateString/DayOfWeek Limits, 34 Attributes lines → `Protected`. Cumulative
units 1-6 = 430 builtins swept, all fixes compat (defaults byte-identical).
The healthiest unit: the highest-value axis (fail-closed parsing) held
everywhere — no silent-wrong parser bug; only one real behavioral fix (the
digest inert-on-unknown-format).

Close-out decisions (no further brief):
- **`ConformSchema[schema]` / `SchemaErrors[schema]` operator forms —
  DEFERRED (hold semantics).** Both heads carry `HoldRest`, so the direct
  `ConformSchema[value, schema]` holds the schema (position 2), but the
  operator capture `ConformSchema[schema]` sits the schema at position 1
  where it is EVALUATED — a divergence for non-fixed-point schemas
  (arithmetic/side-effecting values; `Condition` is `{}` not `HoldAll`, so
  patterns are fixed points in practice but not guaranteed). Shipping on a
  guess is out; needs its own hold-semantics design pass. `DatePlus` has no
  hold and a self-evaluating spec, so it shipped clean.
- **Datetime/random stateful↔pure separation line (user rulings 2026-07-08;
  briefs 200-202, all landed).** The pure/impure DUAL-FORM anti-pattern (one
  symbol mixing a pure form and a stateful form, "hard to categorize") is
  eliminated. FINAL shape (brief-202 corrected brief-200's naming):
  - `AbsoluteTime[]` — the SINGLE numeric wall-clock reader, `Capability::
    Clock`, spec-level (dispatcher-enforced, denied in `sandboxed()`, like
    `RandomBytes`; NOT a runtime check), reader-ONLY (kernel, in the
    debug/timing group). Its old dual-form datelist conversion + dead helpers
    were deleted (dedup vs stdlib `calendar`).
  - `FromDateList[datelist]` — NEW pure converter (`Capability::Pure`,
    `Protected`, stdlib), the inverse of `DateList`, mirrors `FromDateString`,
    reuses `calendar::timestamp_from_date_list_expr`.
  - `DateList`/`DateString` — genuinely pure conversions (their zero-arg
    now-forms removed in brief-200; `DateList[]`/`DateString[]` = arity
    errors). Current time = `DateList[AbsoluteTime[]]` / `DateString[
    AbsoluteTime[]]`.
  - `Now` — REMOVED entirely (brief-200 added it, brief-202 removed it):
    RESERVED for downstream rum-os (the DateObject / time-zone-aware layer;
    WL's `Now` is a DateObject, which rum has no equivalent of). rum's
    numeric timestamp is WL's `AbsoluteTime` sense, not WL's `Now`.
  - `SeedRandom` split (brief-201): `ReseedRandom[]` (`Capability::Random`,
    spec-level) for the entropy reseed; `SeedRandom[seed]` deterministic-only
    (`Pure`, works in sandbox). Deliberate WL divergence, safety over
    WL-conformance.
  Whole-surface audit (2026-07-08): the two anti-pattern directions are
  (A) Pure-declared + a hidden stateful form gated by a RUNTIME capability
  check — now ZERO across the codebase (both cases, datetime + SeedRandom,
  fixed); stdlib has zero runtime capability checks, all impurity is
  spec-level. (B) capability-declared + an over-gated PURE sub-form —
  `AbsoluteTime` was the one clear case (fixed). Remaining mild candidate for
  the kernel sweep: **`TestReport`** is `Capability::Clock` because its report
  includes elapsed time, though the test-RUNNING is pure — review in units
  8-10 (could report pass/fail purely + timing behind an option). The other
  Clock builtins (`Timing`/`AbsoluteTiming`/`RepeatedTiming`/`EvaluationData`/
  `TimeConstrained`/`Pause`) are intrinsically timing — no surprise pure form.
  **Standing rule: stateful reads are single-purpose symbols gated by a
  spec-level capability; never crammed into an otherwise-pure symbol, and
  never gated by a runtime check.**
  **Layering (still holds):** `Today`/`Yesterday`/`Tomorrow`, time zones,
  locale, DST, `DateObject`, `Now` are deliberately ABSENT — wall-clock
  conveniences + tz/locale belong DOWNSTREAM in rum-os (OS authority),
  reusing these pure primitives; stdlib stays UTC-only, exact-seconds, no tz
  database. See [[usability-pass-line]].

Gate lesson (added to the standing set): the docgen `LIMITS` table is
keyed by name (one note per builtin) — a brief that says "add a Limits
note" to a builtin that ALREADY has one will OVERWRITE it. brief-198's
Codex run silently replaced `DateString`'s existing ISO-only/terminating-
second/Clock note with the format-arg note; caught at the gate and MERGED
(original constraints preserved + format note appended). When directing a
docgen Limits note, check for an existing entry and instruct MERGE, not
replace.

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

## Unit 7 (character_classes + random) triage — 2026-07-08

Audit brief-203: 16/16 (count reconciled 15→16 — brief-201 split
`SeedRandom` and added `ReseedRandom`), findings in
`usability-pass/character-classes-random-findings.md`. The lightest unit:
the two FRESH arcs both held — the `RandomPassword` policy surface (briefs
157-159: `Require`/`NoAdjacentRepeats`/`Exclude`/`IgnoreCase`, fail-closed
after 10000 attempts, entropy-backed) and the `SeedRandom`/`ReseedRandom`
stateful↔pure separation (briefs 200-202: deterministic heads `Pure`, the
three entropy heads `Capability::Random`, `sandboxed()` denies only the
entropy heads). No fail-closed or capability-label hole; the deterministic
PRNG contract and the no-`RandomReal` exact-number boundary are correct.
Reproduced the central claim myself (`Attributes /@` the 11 random heads =
ten `{}` + `BlockRandom` `{HoldAll}`; the 5 character-class heads already
`{Protected}`).

Fix batches (dispatch order — safety first, all compat / defaults
byte-identical):

1. **brief-204 — random Protected sweep** (safety, S): add `Protected` to
   all 11 random heads, PRESERVING `BlockRandom`'s `HoldAll` and the three
   entropy heads' `Capability::Random`; sync the runtime mirror
   (`stdlib_attributes.rs`, new `ATTR_HOLD_ALL_PROTECTED` const + 10 moved
   empty→Protected + BlockRandom's const changed) + regen reference +
   attribute test. The brief-196 class — NOT attribute-driven planning, NO
   A/B. The 5 character-class heads are already `{Protected}` (untouched).
2. **brief-205 — nested `{dims}` output for `RandomInteger`/`RandomChoice`**
   (flexibility, M): a shared `parse_dims` + `build_random_array` for the
   two draw functions whose second arg is a SAMPLE COUNT (today scalar-only
   → `{dims}` inert), reusing the per-draw primitives; allocation bounded by
   a PREFIX-PRODUCT element-limit enforce (catches `{10^20, 0}` where leaf
   count is 0 but the top level would allocate 10^20). Per-dispatch
   generation, NOT call planning — no A/B. **DEFERRED out of 205 (WL surface
   uncertain — unit-5 "never guess" discipline):** `RandomString[spec,
   {dims}]` and `RandomString[]` — `RandomString`'s second arg is the string
   LENGTH, not a sample count, so the tensor form + zero-arg convenience have
   no confident WL target; revisit only with a confirmed WL surface.
3. **brief-206 — weighted `RandomChoice`** (flexibility/safety, M):
   `RandomChoice[weights -> elements(, n)]` with EXACT non-negative
   rational weights. No floats: scale weights to a common denominator →
   integer weights → uniform integer draw over the integer-weight sum →
   cumulative bucket map (exact, unbiased). Fail closed on length
   mismatch / negative / zero-total. Not call planning — no A/B.
4. **brief-207 — RandomPassword option discovery + `WhitespaceCharacters`
   constant** (convenience, S): register the `RandomPassword` policy-key
   defaults for `Options[...]` discovery (the unit-6/197 pattern; string-
   keyed rules, implementation already reads them directly → behavior
   byte-identical); add one `Protected` ASCII `WhitespaceCharacters`
   string constant mirroring the existing `WhitespaceCharacter`
   string-pattern atom's set (folk, ASCII-only — no Unicode alphabet
   machinery). No A/B.

Standing guardrails carried into the fixes: deterministic-by-default is
the contract (only the three entropy heads touch host entropy, behind
`Capability::Random`); no floats / no distribution zoo (`RandomReal`
deliberately absent); the 200-202 separation stays settled (do NOT re-merge
`SeedRandom`/`ReseedRandom`); character classes stay ASCII folk (Unicode
defers to the future unicode line). No user-decision-level item this unit.

**UNIT 7 CLOSED (2026-07-08).** Landed: 203 audit + 204 (`Protected` on all
11 random heads; `BlockRandom` keeps `HoldAll`, the three entropy heads keep
`Capability::Random`; runtime mirror + drift test) + 205 (nested `{dims}`
tensor output for `RandomInteger`/`RandomChoice` via shared
`parse_dims`/`build_random_array`, prefix-product element-limit fail-close) +
206 (weighted `RandomChoice[weights -> elements(, n / {dims})]`, exact
non-negative rational weights, lcm-scaled integer draw, no floats,
fail-closed on malformed specs) + 207 (`WhitespaceCharacters` = `" \t\n\r"`
protected ASCII constant + `RandomPassword` option-key discovery
registration). Workspace ~1391 → ~1395 (reproduced 539 in the big suite,
0 failed); reference 611 → 612 (`WhitespaceCharacters`). Cumulative units 1-7
= 446 builtins swept, all fixes compat (defaults byte-identical). The
lightest/healthiest unit: both FRESH arcs (157-159 RandomPassword, 200-202
stateful↔pure separation) confirmed held — no fail-closed, capability-label,
or determinism hole.

Close-out decisions / deferrals (no further brief):
- **`RandomString[spec, {dims}]` + `RandomString[]` — DEFERRED (WL surface
  uncertain; unit-5 "never guess").** `RandomString`'s second argument is the
  string LENGTH, not a sample count, so the tensor `{dims}` form and the
  zero-arg convenience have no confident WL target. 205 shipped `{dims}` only
  for the two draw functions whose second arg IS a sample count
  (`RandomInteger`/`RandomChoice`). Revisit only with a confirmed WL surface.
- **String-keyed option-discovery wart — CROSS-CUTTING, deferred to the
  host-embedding line.** Registered option keys render as SYMBOLS (`Require`)
  while the accepted call syntax uses STRINGS (`"Require"`) — affects
  `FromCsv`/`ToCsv`/`ConformSchema`/`SchemaErrors`/`RandomPassword` (all via
  the brief-197 `system_rule_option` mechanism). 207 followed that precedent
  for consistency (makes the names discoverable). A uniform
  string-keyed-discovery mechanism is the right fix, best done in the
  host-embedding API line (dun's machine-readable-options ask is the consumer)
  — see [[dun-plugin-requirements-line]].
- **`Dimensions` absent — candidate for a later structure touch.** rum has no
  `Dimensions` builtin (it stays inert); noticed while probing 205's nested
  output. Structure/array family (unit-2 CLOSED); a small future addition
  candidate, not blocking.

Next: unit 8 (kernel A — definitions/values/attributes/protection, ~50), per
the dun sequencing ruling (finish the usability pass units 8-10, THEN open the
host-embedding API line).
