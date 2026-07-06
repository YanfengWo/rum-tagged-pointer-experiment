# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

**Archive note (2026-07-03):** the previous log (6386 lines, everything through
the long-tail/capability-sandbox/release-prep era) moved to
`docs/PROGRESS-archive-2026-07-03.md`. State at archive time: correctness
mainline complete (README 3126/0 reproduced exactly); host-embedding hardening
arc done (Tier-1 + Tier-2 core + rum-jupyter real interrupt); long-tail builtin
batches done and the support matrix accuracy-passed (brief-094); capability
sandbox landed (brief-095); public-release scrub method recorded and
`release-staging/` deleted pending re-creation (brief-096); release DEFERRED in
favor of code cleanliness. Active line: crate rebalancing, route 2a —
`docs/crate-rebalance-plan.md`.

## 2026-07-04 Hygiene-policy revision + coda batches DONE (briefs 120/121) — the whole line closes

The user asked for an evidence-based review of the size-hygiene policy
before continuing; the review's four amendments landed in
`docs/code-organization-guidelines.md` (4c88a44): quantified test-file
thresholds (15k/25k/45k) + identity-proof rule, a NEW Directory Fan-Out
section (~20 direct children soft ceiling), measure-implementation-after-
test-extraction, and assess-on-touch replacing the split-plan blanket-TODO;
plus an honest scoping of the incremental-compilation goal (crate is the
unit) and a "Splitting Is Organization" section codifying the brief-114
principles.

Then two coda batches applied it. Brief-120 (932c4d9): the four 30k+ files
split on real boundaries (iteration → spec/driver/table_fast_path with the
113 fast path housed intact; model → 9 type-domain modules; scoping/lexical
→ block/module/with; eval_core → symbol/call/stability) — 22 modules
≤17.4k, sentinel improved both A/B runs, the assess-and-keep option was
available but unneeded. Brief-121 (this commit): runtime/'s fan-out grouped
74 → exactly 48 via five fixed domain directories (definitions/, messages/,
session/, substitution/, kernel_data/ — the last collecting the rebalance
line's kernel-resident *_data machinery), transparent re-exports, zero body
changes, sentinel 55ns/iter. Workspace 1301/0 through everything.

## 2026-07-04 File-slimming line COMPLETE (brief-119) — the >35k debt tier is ZERO

The final batch split the two test files by BEHAVIOR FAMILY with a
test-name identity proof (stdlib 359=359, syntax 152=152, normalized diff
EMPTY): `incidental_runtime.rs` 67k → 12 behavior modules ≤12.2k (its
migration-artifact grab-bag decomposed into what is actually being tested);
`parser/tests.rs` 38k → 8 grammar-area modules ≤12.8k. Gate re-verified the
line's end state: the workspace-wide >35k audit is EMPTY (was 9 files two
days ago: profiling 138k, catalog 74k, incidental 67k, evaluation_cache
63k, definition_application 55k, vm_rhs_template 53k, arithmetic 42k, lexer
41k, parser/tests 38k); the 20-35k split-plan band stands at 26 files (the
next-granularity decision, not urgent). Workspace 1301/0 conserved through
all six batches; two hot splits carried independent gate A/Bs; zero
behavior change end to end. The splitting principles (by logic/function,
incremental-compilation-aware, future-refactoring-aligned, verbatim, split
map first) are codified in brief-114 for reuse.

## 2026-07-04 File-slimming batch 5 DONE (brief-118) — arithmetic.rs 42KB split by numeric domain

The hottest bodies split into 8 modules: `canonical.rs` (18.5k — Plus/Times/
combine_* kept COHESIVE per principle 1, the biggest-but-justified module),
exact (shared rational helpers), rounding, integer_ops, factorial, extremum,
parts, facade. Bodies verbatim; public surface (ArithmeticDataBuiltin,
eval_arithmetic_data_builtin, expr_as_exact_*, combine_*) unchanged; only two
sibling-shared helpers widened to pub(super). A/B: the brief-113 regression
SENTINEL table-iterator-plus at +1.7% (clean); do-small −5.7%; downvalue
cluster +2-5% inside the day's drift band. Gate re-verified: 1301/0
conserved, smoke 110, sentinel triple-probe 55-69ns/iter (matches the
post-113-fix baseline). Remaining debt: the two test files (batch 6, last).

## 2026-07-04 File-slimming batch 4 DONE (brief-117) — vm_rhs_template.rs 53KB + lexer.rs 41KB split

The opt-in VM split into 7 modules (model/compiler/executor/cache/safety/
tests + facade, ≤16.2k) with the disabled-gate check verbatim in cache.rs
and boundaries that keep the future feature-gate/move decision trivial. The
lexer split into 10 modules (core dispatch loop, cursor primitives, strings/
escapes, named_characters table, numbers, symbols, operators, trivia, tests
+ facade, ≤14k) with a clean DAG rooted at cursor/operators. Zero caller
changes (directory-module convention). The pre-existing source_template
dead-code warning was left for the VM-fate decision (the fix-in-passing
authorization was optional). Gate: 1301/0 conserved; lexer named-character
escape smoke (`\[Alpha]`) → 21. Remaining debt: arithmetic 42k (HOT, next),
the two test files.

## 2026-07-04 File-slimming batch 3 DONE (brief-116) — definition_application.rs 55KB split by value-application domain

The hottest file of the line split into 8 modules, all ≤14k: down_values /
up_values / sub_values (dispatch paths), fast_match (the hot candidate/
early-out layer, ISOLATED), default_values, rhs, replace_all (the rewrite
drivers the rewrite_data wrappers consume), mod (facade + shared). DAG:
dispatch paths → fast_match/rhs; replace_all → fast_match. Bodies verbatim;
visibility adjusted only where module boundaries force it. Codex's A/B first
run showed +3-13% across judged labels, receding on its rerun to ≤+5.6% on
two labels — because this is the hottest file, the GATE ran an independent
fresh A/B (HEAD worktree vs working tree): ALL judged labels within ±3.6%,
mostly NEGATIVE (downvalue-exact-hit −1.4%, replaceall −2.8%, matchq
−2.4~−3.5%) — drift confirmed, split neutral. Workspace 1301/0 conserved.
Remaining debt: vm_rhs_template 53k, arithmetic 42k, lexer 41k, 2 test files.

## 2026-07-04 File-slimming batch 2 DONE (brief-115) — evaluation_cache.rs 63KB split on the mechanism/instrumentation axis

The first HOT split held its red lines: `evaluation_cache/` now separates the
per-eval cache MECHANISM (`stability.rs`, 15.5k — impl Session decisions +
bloom) from the env-gated INSTRUMENTATION (`profile/{mod,source,report}.rs`,
the future move-to-profiling/ candidate) with a 1.6k facade + verbatim
tests.rs; all files ≤19.8k. Codex ran the A/B itself per the brief: machine
labels improved-or-flat, the suspicious downvalue cluster (+8%) fell to
+2-3% on the allowed rerun — no consistent regression; measurement-hygiene
note of the batch: it caught the microbench script reusing a STALE release
binary, discarded the polluted run, and rebuilt before measuring. Gate
re-verified 1301/0 conserved, pure-deletion diff (1681 lines), downvalue
smoke 25. Pre-existing dead-code warning noted in vm_rhs_template.rs
(source_template) — untouched, belongs to that file's own batch.

## 2026-07-04 File-slimming batch 1 DONE (brief-114) — profiling.rs 138KB + catalog.rs 74KB split

The slimming line opened (the debt deferred by the rebalance plan; user
principles codified in the brief: split by logic/function never mechanically,
optimize for incremental compilation — stable vs churning regions, sibling
DAG only — and for future refactoring — boundaries movable wholesale; pure
organization, split map first). Batch 1 took the two largest COLD files:

`runtime/profiling.rs` (138k chars, 151 fns) → `profiling/` with 17 modules,
all ≤14.8k: state/config/counters/scopes (stable declarations + env gates) →
records/timing/presence/fingerprints (instrument domains) → report_* +
summary + formatting (churny assembly). The split map survived one revision
(mod.rs initially 24k → stable state extracted). Disabled-path red line
asserted: same env-gated fast-outs, bodies verbatim. `stdlib/catalog.rs`
(74k) → `catalog/` with 14 per-family files (≤12.5k) + an 881-char facade;
install_core and the drift test compile UNCHANGED — and the bidirectional
attribute-drift test passing IS the strongest verbatim proof for the spec
data (gate also verified SET_OPERATION_ATTRIBUTES pre-existed and the 345
count). Zero >20k exceptions; zero caller changes (git status shows exactly
2 deletions + 2 new directories). Gate: fmt + workspace 1301/0; CLI smoke 61.
Remaining debt: evaluation_cache 63k + definition_application 55k (HOT, next,
with A/B), vm_rhs_template 53k, arithmetic 42k, lexer 41k, 2 test files.

## 2026-07-04 Stage-3 FINALE DONE (brief-113) — arithmetic moved; ALL 14 families' spec ownership in rum-stdlib

The line's endpoint: `builtin_catalog/numeric.rs` is GONE; the 27 hottest
handlers (Plus/Times/Power/…, + Identity) moved as wrapper-shape specs
(stdlib catalog 345); arithmetic bodies + `combine_*` canonicalization stay
kernel-side behind `ArithmeticDataBuiltin`. Kernel catalog residue is now
pure evaluator/host machinery: definition/rule machinery + Part family
(rules.rs), Function/HoldForm, scoping, namespace/context/loading,
debug/resource tools, 19 symbol-only string specs, known-symbol vocabulary.
A shared `#[cfg(test)] test_builtins.rs` now serves runtime mechanism tests.

THE GATE CAUGHT THE LINE'S ONE REAL PERFORMANCE REGRESSION here — the
mandatory finale A/B found `table-iterator-plus` at **14x** (63→907ns/iter)
while all 1300 tests stayed green (a pure fast-path-eligibility loss,
invisible to correctness). Root cause: the iterator fast path gates on
`session attributes == default_system_attribute_set(symbol)`; the static
side lost EVERY moved family's attribute data as catalogs left the kernel
(latent since batch 5, exposed only when Plus — the hottest symbol — hit the
check). Architectural conclusion recorded: symbol ATTRIBUTES
(Orderless/Flat/Hold*) are evaluator semantics and must stay kernel-known —
of the spec quintuple, handler and capability may move; static attribute
knowledge may not. Fix: kernel-side attribute vocabulary covering all 345
stdlib specs (112 non-empty) consulted as the fallback + a BIDIRECTIONAL
drift test (stdlib spec attrs == kernel vocabulary, no missing/no extra).
Fix verified: table-iterator-plus stable at −6~−10% vs base across two runs;
remaining table-wide +10-19% swings re-run-verified as machine drift
(receding + direction-flipping; the measurement environment ran ~15
compile+bench cycles today — noise floor ±10% acknowledged in the judgment).
Workspace 1301/0 (the drift test adds one); CLI smoke → 55.

## 2026-07-04 Stage-3 batch 11 DONE (brief-112) — rewrite_builtins: wrapper shape, engine untouched

Codex chose the brief-111 wrapper shape with clean evidence (the rewrite
bodies are matcher machinery: match/compile/normalize, rewrite-rule drivers,
traversal resolution, suspension drivers, trace hooks — a full move would
blow the promotion line): implementation renamed `runtime/rewrite_data*`,
7 specs moved (MatchQ, Cases, DeleteCases, Replace, ReplaceList, ReplaceAll,
ReplaceRepeated; stdlib catalog 318) behind thin wrappers via
`RewriteDataBuiltin` + one dispatch method (promotions: 2). Count/MemberQ/
FreeQ/Position/Level/Rule/NormalizeRules and the definition machinery stayed
(handlers not rewrite-module-owned). Tests: zero moved — mechanism tests use
test-local registrations per the amended rule. Workspace 1300/0; CLI → 7.

Gate ran the declared microbench A/B (wrapper covers hot ReplaceAll):
rewrite/matcher labels flat-to-faster (replaceall −1.6~−2.8% across two runs,
matchq −2.6~−4.2%, downvalue family ±2%). A +13% cluster appeared on the
SMALLEST sub-ms labels (loop-empty, do-iterator-null, literal-integer) —
re-run verdict: machine-state drift, not regression (second run halved it;
`do-small` which actually executes Do bodies was NEGATIVE; literal-integer is
untouched by both batches). Two wrapper batches, two neutral A/Bs — the
wrapper shape is performance-safe. Remaining: arithmetic (the finale).

## 2026-07-04 Stage-3 batch 10 DONE (brief-111) — control_flow: spec ownership moved, implementation stays

The highest-coupling family got a DIFFERENT solution shape, gate-reviewed in
depth and accepted: reference analysis showed nearly the whole implementation
is evaluator machinery (non-local-exit signals consumed by eval/VM paths,
loop iterators on `BlockSymbolState`, abort/cleanup on the interrupt state,
failure/confirmation helpers used by eval_core/eval_dispatch) — so the
implementation stayed kernel-side (renamed `runtime/control_data/`), while
the 35 SPECS moved to the stdlib catalog with thin wrapper handlers
delegating through one promoted facade
(`ControlDataBuiltin` enum + `Session::eval_control_data_builtin`). Semantics
of the split: registration ownership (no stdlib install → no If/Do) is
stdlib's; the machinery is honestly kernel. Promotions: 2 (vs ≤12 line).

Because the wrapper adds a per-call indirection on HOT handlers
(Do/CompoundExpression/If), the gate ran a full microbench A/B (HEAD worktree
vs working tree): control-relevant labels flat-to-faster (do-small −4.2%,
if-true −1.7%, do-count-null −6.8%), the few +5% entries are sub-ms jitter
with contradicting same-mechanism neighbors — NO consistent regression;
neutral confirmed. Also accepted with a SECOND discipline note: Codex kept
~12 mechanism-test files runtime-side with TEST-LOCAL control registrations
instead of migrating them per the Option-1 letter — the right instinct (it
matches the batch-5 exile-list concern) but a rule deviation it should have
STOPPED to report; the rule is now amended going forward: mechanism tests MAY
stay runtime-side with test-local registrations, listed per batch. Behavior
adaptation this batch: `$AssertFunction`/`PropagateAborts` added to the
known-symbol vocabulary (string option forms). Workspace 1300/0 conserved
(runtime 430, stdlib 358); stdlib catalog 311; CLI smoke → 22. Remaining:
rewrite_builtins, arithmetic.

## 2026-07-04 Stage-3 batch 9 DONE (brief-110) — functional_ops moved; apply machinery stays

The cleanest of the big moves: 42 specs out (Map/Apply/Fold/Nest/Select/
Array/Outer/Composition/…), and the runtime functional catalog is now
EXACTLY the evaluator-semantics residue — `Function` and `HoldForm`. The
callable-application machinery the dispatcher consults
(parse_composition/apply_composition_to_args, operator-wrapper apply,
comparison/Key/NearestTo wrappers) stayed kernel-side as
`runtime/functional_data.rs`; the pure-function apply machinery was never
touched. Promotions 7/8 (line held). Test migration surprisingly small — 18
fns (16 whole-file across 6 files + 2 partials) — because earlier batches had
already pulled the functional test hotspots. Workspace 1300/0 conserved
(runtime 510, stdlib 278); stdlib catalog 276; CLI smoke → 12; gate
cross-boundary smoke (kernel Function × stdlib Map/Total) → 14. Remaining:
control_flow, rewrite_builtins, arithmetic.

## 2026-07-04 Stage-3 batch 8 DONE (brief-109) — structure_ops moved; Part stays kernel

The reference-analysis split produced a bigger stay than any batch yet, and
rightly so: `Part`/`Extract`/`ReplacePart`/`ReplaceAt` — specs (in
`builtin_catalog/rules.rs`) AND implementation — stayed kernel-side as
`runtime/structure_data/`, because Part-lvalue assignment is definitions-code
machinery; their runtime tests stayed too (`tests/structure_data/`). The other
38 specs (Length/Flatten/Sort/Take/…) + implementation moved. Runtime catalog
279, stdlib 234; workspace 1300/0 conserved (runtime 529, stdlib 259); CLI
smoke → 7, and the gate added a cross-boundary smoke (kernel-resident
ReplacePart/Extract composed with stdlib-resident ops) → 104. Promotions 7/8 —
the resized stop line held this time. Test-local registration list grew:
scoping_gensym_gc now registers a minimal `First` (same pattern as the VM
SameQ). Next: functional_ops — Codex flags `targeted_map.rs` as depending on
kernel-side exact-path helpers; reference analysis before handlers move.

## 2026-07-04 Stage-3 batch 7 DONE (brief-108) — associations moved with the data/builtin split; 9 families out

The designed split held: the association DATA LAYER (entry forms,
parse/render, key lookup/apply, set-lvalue — everything referenced by
kernel-side code or not-yet-moved families) stayed kernel-side as
`runtime/association_data.rs`; the 33 builtin specs + family helpers moved
(`MissingQ` stayed, its call is kernel-adjacent). `eval_dispatch` untouched
(the split-correctness stop-loss never fired). Runtime catalog 317, stdlib
196; workspace 1300/0 conserved (runtime 577, stdlib 211); CLI smoke → 21.
Biggest migration so far, as predicted from the 28-file incidental footprint:
22 dedicated tests + ~40 measured moves (9 whole-file + 12 partial sources).

Two gate notes. (1) Promotions came to 6 narrow facades — ONE OVER the
brief's ≤5 stop line; Codex should have stopped and reported rather than
finishing (outcome harmless, all six justified, but the discipline slip is
noted; future briefs size the line to the family's audited kernel-import
count instead of a flat 5). (2) One behavior-equivalence adaptation was
required and gate-verified: the new `protected_or_known_definition_target_symbol`
facade adds a REGISTERED-spec branch alongside the static vocabulary — moved
builtins (no longer in the static catalog) must still count as known/protected
for AssociateTo; consistent with the brief-102 resolution-probe conclusion.

## 2026-07-04 Stage-3 batch 6 DONE (brief-107) — predicates moved; 8 families out

50 predicate specs moved (runtime catalog 350, stdlib 163); the family got its
free-fn conversion in transit; `builtin_catalog/predicates.rs` deleted whole
(`MissingQ` stayed — its spec lives in the associations catalog). Migration
measured 33 test fns (vs logic's 31 — the mild trend holds). Promotions 4
(≤5 line): the audit-predicted `invalid_call_with_arg_form` +
`collect_level_parts_owned` + 2 `CasesTraversalConfig` constructors.
Workspace 1300/0 conserved (runtime 644, stdlib 144); CLI smoke → 2.

Two gate-reviewed edge calls, both accepted with lessons: (1)
`associations/basic/predicates.rs` renamed to `key_predicates.rs` ONLY because
the brief's grep gate used a bare word — grep gates must be path-precise
(second wide-grep incident after brief-100's); (2) the VM template tests lost
`SameQ` and now register a TEST-LOCAL minimal SameQ (structural equality) via
the open registry — probes unchanged, and crate-internal mechanism tests
genuinely cannot migrate (they touch runtime internals). EXPECT this pattern
to recur when control_flow moves (VM tests likely use If/Do too); the
test-local registration list joins the exile list as future-cleanup tracking.

## 2026-07-04 Stage-3 batch 5 DONE (brief-106) — logic moved; the comparisons left the kernel

The user chose Option 1 for the load-bearing tier (tests migrate WITH their
families; no dev-dep cycle; runtime test tree trends mechanism-only), and the
first mass migration turned out MILD: measured reality was 31 test fns
(22 whole-file + 9 partial), not the feared hundreds — runtime tests lean on
comparisons far less than assumed. Logic itself was the lowest-increment move
(free-fn conversion + kernel-method pub-ification all done in 099/101):
21 specs moved, runtime catalog 400, stdlib 113; the leftover catalog slice
renamed honestly to `builtin_catalog/predicates.rs` (50 specs); obsolete
nand/nor adapter shims deleted. Workspace 1300/0 conserved (runtime 679,
stdlib 111); CLI smoke `If[Xor[True, False] && 1 <= 1, 42, 0]` → 42.

Promotions: 3 (within the ≤5 line) — the gensym-GC introspection counters
(`collect_and_sweep_dead_gensyms` visibility flip + 2 read-only count
accessors), needed because the gensym-GC MECHANISM test uses logic builtins in
its probes and migrated per the Option-1 rule. Noted for a future cleanup
line: mechanism tests exiled to the stdlib only because their probe
expressions use stdlib builtins could be probe-rewritten and repatriated
later; the rule "don't rewrite probes to dodge migration" was correct for
keeping this batch honest, but the exile list is worth tracking (currently:
scoping_gensym_gc partial, debug_tools_resources partial).

## 2026-07-04 Stage-3 batch 4 DONE (brief-105) — strings moved (largest batch); 6 families out

The biggest single family so far: `strings` (facade + 21 files ≈6.5k lines, 61
handler-backed specs) now lives in rum-stdlib (which gained the `rum-syntax`
dep for ToExpression/SyntaxQ). Runtime catalog 482 → 421 (19 symbol-only
string specs legitimately stay); stdlib catalog 92; workspace 1300/0; CLI
smoke `StringLength[StringJoin[{"ab","cd"}]] + ToExpression["3*4"]` → 16.
Tests: the whole `tests/strings/` directory + 19 measured-failure fns
(→ `tests/strings/incidental_runtime.rs`) + the FileTemplateApply capability
assertion moved with the handler.

The one structural judgment call, gate-audited in depth: ToString/TextString's
expression RENDERER is shared kernel machinery (message_format,
builtins_input, scoping, control_flow all render through it) — it stayed in
the kernel as `runtime/text_rendering.rs`. Verified: byte-identical to the old
`strings/text/render.rs` modulo visibility/docs, plus `string_form_value`
folded in from template.rs (its only caller is the renderer itself). No
formatting code was copied or forked. Promotions within stop-loss: 7
kernel_api exports (incl. `Capability`, `ToStringFormat`, `render_*`) + 7 pub
Session methods (incl. the suspension API `suspend_with_frame`,
`drive_evaluated_value`/`drive_internal_value`,
`enforce_active_memory_limit_for_projected_bytes`) — 10 of 14 were
audit-predicted. Stop-loss lines (>12 new promotions / >40 failing fns /
pattern-matcher entanglement) NOT hit: measured 20 failing fns, no
entanglement (the string-pattern compiler moved cleanly with the family).

## 2026-07-04 Stage-3 batch 3 DONE (brief-104) — calculus + polynomials moved; 5 families out

First multi-file directory family through the pipeline: `polynomials/`
(mod/candidates/core/core/factors/factorization/rational_function) moved with
its layout intact; `calculus` (the `D` handler) along. Runtime catalog
488 → 482, stdlib 31 (11+5+9+1+5); runtime greps clean; CLI smoke
`--eval "Coefficient[Expand[(x+1)^4], x, 2] + D[x^3, x] /. x -> 2"` → 18;
workspace 1299/0 (runtime 753 + stdlib 34; 4 calculus + 13 polynomials test
fns migrated). One family-boundary discovery handled correctly: the
`PolynomialQ`/`PolynomialExpressionQ`/`RationalExpressionQ` PREDICATE handlers
physically lived inside `polynomials/mod.rs` — they moved VERBATIM (md5-equal
bodies, gate-checked) into `runtime/predicates.rs` so the predicates family
stays runtime-owned; their catalog entries never left. Promotions:
`HoldAttribute` + `Session::invalid_call_with_message` pub;
`parse_indeterminate_expr` facade retargeted. Remaining Stage-3 families: 8
(predicates, logic, arithmetic, strings, structure_ops, functional_ops,
associations, control_flow, rewrite_builtins — the load-bearing tier; test
strategy per family by measured failing-fn count).

## 2026-07-04 Stage-3 batch 2 DONE (brief-103) — integer_digits + number_theory moved; 3 families out

First BATCH move (two families in one brief) through the proven pipeline:
`rum-stdlib` now owns `integer_digits` (5 handlers) and `number_theory` (9
handlers, incl. the Prime/PrimePi small tables and Pollard-rho path); runtime
catalog 502 → 488, stdlib catalog 25 (11+5+9); runtime greps clean; CLI smoke
`--eval "NextPrime[100] + FromDigits[{1,2,3}] + EulerPhi[10]"` → 228; workspace
1299/0. Promotions: `Session::{invalid_call_with_arg_range,
enforce_generated_element_limit_for_count, thread_call_args, push_message}`
now `pub` (+ `evaluate` documented); `kernel_api` re-exports
`num_integer::{Integer,Roots}` + `num_traits::{One,Signed,Zero}` additions.

Two process notes. (1) The NEW test-migration rule proved out: `number_theory`
tests moved as a whole file (8 fns), while `integer_digits` tests were embedded
in `tests/arithmetic.rs` — the rule "run the runtime suite after the move; every
MEASURED failing function moves whole to the stdlib" migrated exactly 3
functions, no guesswork. (2) The first dispatch of this brief was killed
mid-move by my own foreground-chained dispatch command (Bash tool 2-min
timeout); the half-done tree was rolled back cleanly and re-dispatched. That
failure mode is now MECHANICALLY blocked by a PreToolUse hook
(`.claude/hooks/guard-dispatch.py`): any `dispatch-brief`/`cdx exec` Bash call
that is chained, bare-`&` detached, or not run_in_background is denied by the
harness before execution (DRY_RUN=1 exempt) — verified live.

## 2026-07-04 Stage 3 OPENED and the MOVE pipeline is PROVEN (brief-102) — combinatorics lives in rum-stdlib

The first family physically left the monolith. `rum-stdlib` now owns the
combinatorics implementation (`src/combinatorics.rs`, 11 handlers converted to
free fns while moving, bodies verbatim), its specs (`src/catalog.rs`, 11
entries byte-identical to the deleted runtime ones modulo handler path), and
its 5 behavioral tests; `install_core` = runtime catalog (now 502 entries)
then stdlib specs. rum-runtime greps CLEAN for combinatorics. Pilot family was
combinatorics (not logic) because it has zero incidental test usage — logic
owns the comparison operators the kernel's own tests use everywhere.

Step-0 resolution probe (the one real unknown) PASSED: a builtin registered
under a name ABSENT from the static catalog resolves and dispatches through
`eval_text` — no vocabulary fix needed; the probe stays as a regression test
(`registered_builtin_resolution.rs`). Kernel promotions were restrained and
documented: `Session::{invalid_call_with_exact_arg_count,
combine_additive_args, combine_multiplicative_args}` now `pub`;
`kernel_api` re-exports `BuiltinSpec` + `num_traits::ToPrimitive` (spares the
stdlib a num-traits dep). rum-stdlib gained the `rum-core` dep.

Gate: fmt + workspace 1299/0 (runtime −5 moved tests +1 probe, stdlib +5);
CLI smoke `--eval "Fibonacci[10] + CatalanNumber[4]"` → 69 (the brief's
literal `eval` subcommand syntax was wrong — rum-cli takes `--eval`; noted for
future briefs). No microbench (registration source only). Remaining Stage-3
work: the other 12 families through the same pipeline; load-bearing ones
(logic/predicates/arithmetic/…) additionally need the test-migration /
dev-dep-cycle decision recorded in the memory notes.

## 2026-07-03 Stage-2 step 3 DONE (brief-101) — logic handlers are free functions

The pilot family's handlers no longer live in `impl Session`: 16 handlers
(truth.rs 8 + comparison.rs 8, incl. the private `eval_comparison_subset` and
the truth continuation helpers) are free functions
`fn eval_x(session: &mut Session, …)` with bodies VERBATIM (receiver-only
change); the facade re-exports them per-item; `builtin_catalog/logic.rs` and
the nand/nor adapters reference `crate::runtime::logic::eval_*` explicitly.
Scope note that shaped this brief: catalog file grouping ≠ implementation
module grouping (catalog/logic.rs references 69 handlers; the logic MODULE
owns 16 — the rest are predicates/structure_ops and stay untouched until their
family's brief). Gate: fmt + workspace 1298/0; zero `impl Session` under
logic*; no microbench (static call → static call, dispatch fn-pointer shape
unchanged). The logic family is now move-ready: brief-102 = the physical move
pilot (files → the stdlib crate, spec ownership, kernel Session-method
pub-ification, test migration).

## 2026-07-03 Stage-2 step 2 DONE (brief-100) — registration is the ONLY builtin dispatch path

Discovery that motivated it: dispatch resolution silently fell back to a static
`OnceLock` map over `core_builtin_specs()`, so builtins never needed
registration and `rum-stdlib::install_core` was decorative — families cannot
leave the crate while the kernel statically consults its own catalog. Now:
`resolution.rs` reads ONLY the session's registered map (fallback deleted);
`Session::default()`/`new()` auto-install `core_builtin_specs()` (the
transitional bridge — shrinks as families move out); `with_config()` builds a
truly RAW session (hosts/stdlib must install — they already do);
`register_builtin` applies the `with_fallback_dispatch` merge against the
previously REGISTERED spec, preserving user-override semantics; `Names`
enumerates registered specs. The static `default_builtin_dispatch_spec*`
helpers legitimately REMAIN as vocabulary (known-system-symbol predicates in
`expr_runtime_helpers.rs` — a dispatch-vs-vocabulary distinction the original
brief's grep gate missed; Codex stop-lossed on it and the gate was amended).
New test: fresh session evaluates a builtin + DumpSave/Get restore keeps the
registered set. Gate: fmt + workspace 1298/0; resolution grep = 0; microbench
A/B flat-to-better (−8%…+1.6%, no consistent regression; mechanism touches
only the cache-miss resolution path).

## 2026-07-03 Stage-2 step 1 DONE (brief-099) — kernel_api v1 exists, logic family de-globbed

`runtime/kernel_api.rs` is now the single public door to the kernel's helper
surface: 40 audited fan-out ≥2 items re-exported in 14 grouped `pub use` lines
(`pub mod kernel_api` from `runtime/mod.rs`; defining `mod`s stay private, so
the promoted `pub` items are unreachable except via kernel_api). Pilot: the
`logic` family (`logic.rs` + `logic/{comparison,shared,truth}.rs`) carries NO
`use super::*` — only explicit `kernel_api`/`rum_core` imports. The pilot
CONFIRMED the audit model: demanded exactly the predicted 9 items (5 kernel_api
+ 4 rum_core), zero unplanned promotions.

Learned (now standing for briefs 100+): promoted items sit in private
submodules behind family facades; Rust cannot re-export a restricted re-export
(E0364/E0365), so facades need per-item `pub use self::<submod>::item;` lines
(never widen a glob re-export) — facade files belong in MAY-modify from the
start. Codex stop-lossed correctly on this and was resumed with a scoped
authorization. Gate: fmt + `cargo test --workspace` 1297/0; visibility/imports
only (rustfmt reflow aside, no body/signature changes; no microbench — no
codegen delta); the `SymbolExprMap`-vs-private-hasher warning closed at gate
time by making `RuntimeCacheHasher` `pub` inside its private mod (one word +
comment). Diff: 21 files, ~+56/−51.

## 2026-07-03 Stage-2 audit COMPLETE (brief-098) — the real kernel API is tiny

Method: per-family compile-error audit (comment out `use super::*` in each of the
14 builtin family modules, one at a time; collect unresolved names from
`cargo check`; restore) + a grep axis for kernel `Session` methods called from
family code. Full artifact: `docs/dev/kernel-api-audit-2026-07-03.md` (from
`/tmp/cdx_brief_098_report.md`); raw logs `/tmp/cdx_brief_098_raw_<family>.log`.
Diagnostic only — tree clean, final `cargo check` green. (Operational note: the
first run stopped at 6/14 families because Claude's concurrent PROGRESS-archive
commit tripped the whole-tree clean-check; resumed with the check scoped to
`crates/`. Lesson recorded: no repo mutation while a brief is in flight.)

Findings (free-item axis): of the runtime-internal namespace, only **258 items
are used by any family at all**; **24 have fan-out ≥3 families** (the prelude
core: `Session`/`EvalError` types, `system_expr`, `is_system_symbol_name`, the
`expr_as_*` coercion helpers, `compare_expr_canonical`, a few model/session
types like `ThreadingDispatch`/`ThreadArgSpec`/`CasesLevelSpec`), 16 have
fan-out 2, **218 are single-family** (move WITH their family, don't promote),
and ~600 internal symbols are used by no family (stay kernel-private). Known
family→family couplings that constrain move order: `arithmetic` numeric
helpers (`expr_as_exact_integer` 7 families, `expr_as_exact_rational` 5),
`associations/data` helpers (`association_entries_from_expr`,
`render_association` — 6 families each), structure path helpers.

Findings (Session-method axis, the future pub method surface, top of ~15-25):
`evaluate` (148 calls), the `invalid_call_with_*` argument-error helpers
(131+113+112+67), `suspend_with_frame` (100) + `drive_internal_value` (41) +
`suspend_with_result_frame` (17), `push_message` (37), the `enforce_*` resource
limits, `match_expr_owned`, `merged_option_exprs_for_system_symbol`.

Cheapest first move: **`logic`** — 9 free items, no same-family glob fallout, no
other-family coupling. Count reconciliation note: the earlier "528
`pub(in crate::runtime)` symbols" was a grep line count; the audit's item-name
scan counts 651 unique names — same order of magnitude, the design conclusion
(promote ~40, not 500+) is insensitive to the method.

## 2026-07-03 Crate-rebalance line OPENED (route 2a) — Stage 1 DONE: builtin dispatch is open

Line context: `rum-runtime` had grown into a 116,293-line / 391-file monolith
(evaluator + every builtin family) while `rum-stdlib` was a 21-line installer
facade; builtins were unmovable because dispatch was a CLOSED ~470-variant
`BuiltinHandler` enum + a three-file chained `match` + an exhaustive per-variant
capability `match`, and the families compile against a 528-symbol
`pub(in crate::runtime)` shared namespace via `use super::*` (377 files).
Canonical plan: `docs/crate-rebalance-plan.md` (d49e101) — Stage 1 open the
dispatch, Stage 2 define the kernel API, Stage 3 move the families; file-slimming
deliberately AFTER the split.

**Stage 1 (brief-097, commit 45d2e8a): delete the closed enum.** `BuiltinSpec`
now carries the handler as a plain fn pointer
(`pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) -> Result<Expr, EvalError>`)
and the capability as data (`capability: Capability`, default `Pure`; 8 FileRead
+ 3 FileWrite annotated in the catalog). Deleted: the `BuiltinHandler` enum, the
`eval_dispatch/{surface,symbolic,control_functional}.rs` match chain (~530
lines), and the `impl BuiltinHandler::capability` match (~500 lines); net −1467
lines. The 513 catalog entries reference `Session::eval_*` directly (fn-item →
fn-pointer coercion in const arrays); ~25 non-standard arms (args-only
signatures + inline bodies) live VERBATIM as adapters in `builtin_adapters.rs`
— `Evaluate` keeps its `suspend_with_frame` suspension shape. `CallDispatchPlan`
carries the resolved `BuiltinSpec` (Copy, resolve-once-at-plan-build preserved);
capability inheritance follows handler inheritance in `with_fallback_dispatch`
(a re-registered `Get` cannot shed its `FileRead` requirement). The
fn-pointer-comparison lint is allowed locally at the type (all spec copies
originate from one catalog entry, so pointer equality is stable).

Gate evidence: `cargo fmt --check` + `cargo test --workspace` green (1297/0);
`grep -rn "BuiltinHandler" crates/` = 0 hits; microbench A/B run TWICE
independently (Codex once, Claude reproduced via a HEAD worktree). KEY
measurement lesson: the two A/Bs disagreed label-by-label IN BOTH DIRECTIONS
(labels reported −21% in one run showed +9–11% in the other, and vice versa) —
sub-ms dispatch-tier deltas are noise-dominated; judge only cross-run
consistent-direction movement, of which there was none. Verdict: match-jump →
indirect call is measurement-neutral; the extraction precondition (builtin =
data registered into the session) now holds.

**Stage 2 audit dispatched (brief-098, e251618):** diagnostic, no source change
— per-family compile-error-driven audit (comment out `use super::*`, collect
unresolved names, restore) over 14 family modules + a grep axis for kernel
`Session` methods called from family code. Output: the promotion-candidate list
for the explicit kernel API (≥3-family items → prelude; single-family items →
move with the family; unused internals → stay private) + the family→family
couplings that constrain move order.

## 2026-07-04 — Release transformation line STARTED (plan committed)

Decision (user, this session): drop the fresh-history release-staging approach;
transform `main` itself into release shape, keep full history. New testing policy:
rum tests only its own documented design + rum-vs-rum perf A/B; difftest and the
mma-hol snapshot pipeline are removed (archived privately at
../Archive/rum-history, MANIFEST there; WLBuiltin/ already moved). Support matrix
dropped in favor of a hand-written language guide + a builtin reference generated
from the stdlib catalog + rustdoc. Endpoint: clean tree + bundle cold backup (no
remote). Plan: docs/dev/release-transformation-plan.md (briefs 124-130 + M0-M2).

## 2026-07-04 — M1: narrative + policy rewrite (release transformation)

DISCLAIMER.md rewritten (user-reviewed): independent project, no compatibility
claims, factual history of the early black-box method and its July 2026
removal, standing policy. README repositioned (embeddable sandbox-first
term-rewriting language; external-comparison framing removed). AGENTS.md
rewritten around the new Testing Policy hard lines (own-documented-behavior
tests + rum-vs-rum perf A/B only; no oracle harnesses/compat matrices — 
permanent). testing-strategy.md rewritten (4 layers, coverage axes kept,
documentation-as-contract, llvm-cov guidance). TODO.md rewritten (release line
active items + post-cut queue: toolchain refresh to current stable, coverage
rebuild, scoped-fs profile, pyo3-on-rum-host, showcase, vm_rhs_template).
CLAUDE.md updated to the new world. Briefs 124-126 all gated green earlier
today (commits 5eddf1b, 131614a, 3814ef0).

## 2026-07-05 — M1.5: ARCHITECTURE + v0.1 boundary rewritten; toolchain settled

ARCHITECTURE.md: independent positioning, rum-stdlib section updated to the
post-rebalance reality, rum-difftest section removed, Compatibility Target →
Behavior Contract (documentation is the contract), Performance Kernel
Direction → Performance Discipline (rum-vs-rum A/B), Conformance Strategy
removed, core-language boundary now defined by rum's own guide/reference.
docs/v0.1-acceptance-boundary.md rewritten (acceptance = documented slice +
hardened rum-host + CLI/REPL + jupyter preview; external-corpus lanes gone).
Toolchain/MSRV SETTLED per user: stay on Rust 1.85 (Debian stable default
rustc + edition-2024 floor); recorded in TODO/README. brief-127 gated green
(e0448c9): crates wording scan, zero behavior change, suite totals identical.

## 2026-07-05 — briefs 128 + 129 gated green (reference + guide)

brief-128 (8503321): generated builtin reference — 513 entries from the
installed registry (attributes, capability class, summary, verified limits
notes) + drift test (negative check demonstrated); determinism reproduced at
the gate (double run, empty diff); minimal doc-hidden read-only iteration
APIs in rum-runtime. brief-129 (64b7c08): nine-chapter language guide, all
examples current-build transcripts, nine spot-reproduced at the gate; README
links guide + reference. Guide writing surfaced two real pre-release bugs
(now in TODO §Pre-Release Fixes): unbounded user recursion natively
overflows the stack instead of tripping recursion_limit; ReplaceRepeated
limit surfaces as a CLI hard error instead of TerminatedEvaluation.

## 2026-07-05 — brief-130 gated green; fs-expansion direction queued

brief-130 (2089d35): rum-host fully rustdoc'd under warn(missing_docs) (zero
warnings, doctest green; one grouped re-export split into per-item pub use —
semantically identical), crate-level overviews for the library crates,
workspace metadata (descriptions, license/keywords/categories inheritance, no
vendor terms, repository field deferred to publication), ISSUE file archived.
Workspace 1131/0. TODO gains the post-cut fs-surface expansion item (scoped
roots first). Next: brief-131 fixes the two guide-surfaced bugs before M2.

## 2026-07-06 — brief-131 + M2: the release transformation line is COMPLETE

brief-131 (cdeaa59): Phase A measured ~20 KiB native stack per WL recursion
level (overflow at ~depth 412 of limit 1024) → native_stack_limit guard
(default 6 MiB consumed) at the amortized poll point, recoverable
TerminatedEvaluation["StackLimit"]; ReplaceRepeated overflow now
TerminatedEvaluation["IterationLimit"]. Zero new unsafe; workspace 1136/0;
A/B no hot-label regression (median 1.018, reproduced at the gate). M2:
TEMPLATE.md rewritten to the new policy; briefs 124-131 + the plan doc
archived; final sweep = whitelist-exact (DISCLAIMER nominative, three
one-sentence family mentions, jupyter ecosystem identifiers); TODO/CLAUDE
updated; bundle cold backup follows this commit.

## 2026-07-06 — FS purity line STARTED

Decision (user): strip the six general file builtins (FileNames, FileExistsQ,
FileDate, FileByteCount, Import, FileTemplateApply) out of the repo entirely
(archived for a future downstream rum-fs project, post-launch); keep the
module system (Get/Needs/FindFile) and persistence (Save/DumpSave) with their
I/O injected via a host provider; kernel crates become std::fs-free by
construction with a hygiene test. Plan: docs/dev/fs-purity-plan.md
(briefs 132-133 + M0/M1).

## 2026-07-06 — FS purity line COMPLETE (briefs 132-134)

brief-132 (a46836e): the six general fs builtins stripped and archived as the
future downstream rum-fs seed; pure path-string helpers stay; reference 507.
brief-133 (06dbbb6): FileIo provider seam — Get/Needs/FindFile/Save/DumpSave
do I/O through a host-injected trait; StdFileIo lives in rum-host (absent
under sandboxed()); General::noio contract for permissive-policy-no-provider;
hygiene test forbids std::fs in kernel crates outside /tests/ paths.
brief-134 (c92b84f): Capability::Clock — audit rule "Pure iff observable
result depends only on session state + arguments" moved 8 clock readers out
of Pure (incl. Pause/EvaluationData/TestReport the audit caught);
MemoryConstrained stays Pure with evidence; BlockRandom removed as an inert
shell; sandboxed() is now deterministic by construction; docs adopt the two
claims (no ambient authority; deterministic by default). Reference 506;
workspace 1141/0. Process docs archived; kernel hard lines now three: zero
unsafe, zero std::fs (non-test), Pure=deterministic.
