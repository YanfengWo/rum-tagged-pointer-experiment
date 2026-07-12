# Kernel B Usability Audit Findings

## Count Reconciliation And Coverage

The standing derivation and the unit-9/10 split reproduce exactly:

- installed reference headings: **612**
- distinct stdlib catalog names: **453**
- reference minus stdlib: **159 kernel names**
- unit 8: **55 names**
- remaining after unit 8: **104 names**
- unit 9 (this audit): **47 names**, all in that remainder
- unit 10: the exact **57-name complement**, with no overlap

The catalog regex reads the first string argument after each
`BuiltinSpec::<constructor>(` across whitespace. Both inputs were checked for
duplicates before subtraction.

```text
$ grep -c '^### ' docs/reference/builtins.md
612
$ python3 - <<'PY'
from pathlib import Path
import re

spec_re = re.compile(r'BuiltinSpec::[A-Za-z_]+\(\s*"([^"]+)"')
heading_re = re.compile(r'^### (.+)$', re.M)
stdlib = [
    name
    for path in sorted(Path('crates/rum-stdlib/src/catalog').glob('*.rs'))
    for name in spec_re.findall(path.read_text())
]
reference = heading_re.findall(Path('docs/reference/builtins.md').read_text())
unit8 = [
    'Set', 'SetDelayed', 'TagSet', 'TagSetDelayed', 'TagUnset', 'UpSet',
    'UpSetDelayed', 'AddTo', 'SubtractFrom', 'TimesBy', 'DivideBy',
    'Increment', 'Decrement', 'PreIncrement', 'PreDecrement', 'AppendTo',
    'PrependTo', 'ApplyTo', 'OwnValues', 'DownValues', 'UpValues', 'SubValues',
    'DefaultValues', 'ValueQ', 'Definition', 'Information', 'Clear', 'ClearAll',
    'Remove', 'Unset', 'Attributes', 'SetAttributes', 'ClearAttributes',
    'Protect', 'Unprotect', 'HoldAll', 'HoldFirst', 'HoldRest',
    'HoldAllComplete', 'SyntaxInformation', 'Options', 'SetOptions',
    'OptionValue', 'ArgumentsOptions', 'Message', 'Messages', 'MessageName',
    'MessageTags', 'MessageDefinitions', '$MessageList', 'Quiet', 'Check',
    'CheckArguments', 'Off', 'On',
]
unit9 = [
    'Block', 'Module', 'With', 'Function', 'Unique',
    'Hold', 'HoldComplete', 'HoldForm', 'HoldPattern', 'ReleaseHold', 'Evaluate',
    'Unevaluated', 'Once', 'Sequence', 'Nothing', 'Sow', 'Reap',
    'Repeated', 'RepeatedNull', 'Longest', 'Shortest', 'IgnoreCase', 'Overlaps',
    'Whitespace', 'NumberString', 'SpellingCorrection', 'Rule', 'RuleDelayed',
    'NormalizeRules', 'Part', 'Extract', 'ReplacePart', 'ReplaceAt', 'Position',
    'Count', 'MemberQ', 'FreeQ', 'Level', 'Thread', 'MissingQ', 'InputForm',
    'OutputForm', 'StandardForm', 'TraditionalForm', 'Short', 'Automatic', '$Failed',
]
unit10 = [
    '$Aborted', '$Canceled', '$Context', '$ContextPath', '$InputFileName',
    '$Packages', '$Path', 'AbsoluteTime', 'AbsoluteTiming', 'Begin', 'BeginPackage',
    'ByteArray', 'ByteCount', 'Context', 'Contexts', 'DeclarePackage', 'Dialog',
    'DiffObject', 'DirectoryName', 'DumpSave', 'Echo', 'EchoFunction', 'EchoLabel',
    'End', 'EndPackage', 'EvaluationData', 'File', 'FileNameJoin', 'FileNameTake',
    'FindFile', 'Get', 'Input', 'InputString', 'MaxMemoryUsed', 'MemoryConstrained',
    'MemoryInUse', 'Monitor', 'NameQ', 'Names', 'Needs', 'ParentDirectory', 'Pause',
    'Print', 'RepeatedTiming', 'ResolveContextAliases', 'RumProfileReset',
    'RumProfileSnapshot', 'Save', 'Stack', 'SymbolName', 'TemplateObject',
    'TestCreate', 'TestReport', 'TimeConstrained', 'Timing', 'Trace', 'TracePrint',
]
ref_set, std_set = set(reference), set(stdlib)
kernel = ref_set - std_set
remaining = kernel - set(unit8)
print(f'stdlib catalog entries = {len(stdlib)}')
print(f'stdlib distinct names = {len(std_set)}')
print(f'installed reference names = {len(reference)}')
print(f'installed distinct names = {len(ref_set)}')
print(f'kernel diff = {len(kernel)}')
print(f'unit-8 names = {len(unit8)}')
print(f'after unit 8 = {len(remaining)}')
print(f'unit-9 names = {len(unit9)}')
print(f'unit-9 missing from kernel remainder = {sorted(set(unit9) - remaining)}')
print(f'after unit 9 = {len(remaining - set(unit9))}')
print(f'unit-10 names = {len(unit10)}')
print(f'unit-10 complement exact = {set(unit10) == remaining - set(unit9)}')
print(f'unit-10 missing from complement = {sorted(set(unit10) - (remaining - set(unit9)))}')
print(f'complement absent from unit 10 = {sorted((remaining - set(unit9)) - set(unit10))}')
print(f'unit-9/unit-10 overlap = {sorted(set(unit9) & set(unit10))}')
print(f'stdlib duplicate names = {len(stdlib) - len(std_set)}')
print(f'reference duplicate names = {len(reference) - len(ref_set)}')
print(f'stdlib names absent from reference = {sorted(std_set - ref_set)}')
PY
stdlib catalog entries = 453
stdlib distinct names = 453
installed reference names = 612
installed distinct names = 612
kernel diff = 159
unit-8 names = 55
after unit 8 = 104
unit-9 names = 47
unit-9 missing from kernel remainder = []
after unit 9 = 57
unit-10 names = 57
unit-10 complement exact = True
unit-10 missing from complement = []
complement absent from unit 10 = []
unit-9/unit-10 overlap = []
stdlib duplicate names = 0
reference duplicate names = 0
stdlib names absent from reference = []
```

The findings-table check in Verification Outputs prints, verbatim:

```text
COVERAGE-OK 47/47
```

## Headline: The `Unevaluated` No-Match Boundary

`Unevaluated` is honored while a call path consumes the prepared argument, but
its wrapper is not restored when no stage consumes the call. The boundary is:

| Path | Current result | Boundary |
|---|---|---|
| ordinary builtin | `Length[Unevaluated[1+1]] -> 2`; `Head[...] -> Plus` | Correct: the builtin receives the unwrapped, still-raw `Plus[1,1]`. |
| held head | `Hold[Unevaluated[1+1]] -> Hold[Plus[1,1]]` | Correct: the direct wrapper is stripped and the held argument remains raw. |
| matching downvalue / function / upvalue | `g[...] -> HoldComplete[Plus[1,1]]`; explicit `Function` does the same; the upvalue probe returns `caught` | Correct while the call stage consumes the prepared argument. |
| undefined head | `f[Unevaluated[1+1]] -> f[2]` | Divergence: the no-match reconstruction loses the wrapper, then the next fixed-point pass evaluates the content. |
| nonmatching definition | `q[x_Integer] := hit[x]; q[Unevaluated[1+1]] -> hit[2]` | Worse boundary: losing the wrapper can create a match that did not exist on the raw pass. |
| unconsumed sequence | `f[Unevaluated[Sequence[a,b]]] -> f[a,b]` | The stripped raw value is also spliced before the reconstructed call survives. |

The missing-restore seam is code-visible. Ordinary-position planning strips the
wrapper and records `should_evaluate: false` at
`crates/rum-runtime/src/runtime/eval_call_prepare.rs:50-55`; if every later
stage misses, `resume_call_with_dispatch_plan` reconstructs the call solely
from the prepared arguments at
`crates/rum-runtime/src/runtime/eval_core/call.rs:97-101`. There is no retained
wrapper/provenance bit to restore. Any behavioral fix is in the evaluator core
and therefore requires the core-language microbench A/B gate.

## Top Gaps By Script-User Value

1. **Safety / trust boundary:** a caught error from `Module` leaves the fresh
   local's definition enumerable and readable (`Names[...] ->
   ``{"Global`auditLocal$1"}``, then `auditLocal$1 -> 1`). Per the brief, this is
   reported and STOPPED at the decision boundary: no renaming/scoping change is
   proposed.
2. **Safety:** `Unevaluated` loses its wrapper on an undefined or nonmatching
   call path; a later pass can evaluate it and even make a previously
   nonmatching downvalue fire.
3. **Safety:** `HoldAll` incorrectly suppresses ordinary `Sequence` splicing;
   `h[Sequence[a,b]]` stays nested after `SetAttributes[h,HoldAll]`, while
   `Evaluate` can force the splice. `HoldAllComplete` and `SequenceHold` do
   preserve it correctly.
4. **Safety:** delayed rule RHS evaluation depends on the replacement consumer.
   Inside a held `ReplaceAll` position the RHS remains an unevaluated
   `CompoundExpression`, while `ReplacePart` and `ReplaceAt` evaluate the same
   delayed RHS before insertion. Rule substitution also splices `Sequence`
   inside a `HoldAllComplete` head before the evaluator can block it.
5. **Safety:** `ReleaseHold` does not release `HoldForm`, despite `HoldForm`
   being the unit's supported transparent hold wrapper; it returns
   `HoldForm[Plus[1,1]]`.
6. **Safety / contract:** `ReplacePart`'s reference promises `Span` positions,
   but span rules silently leave the expression unchanged.
7. **Safety / flexibility:** the expression matcher accepts all documented
   `Repeated` count shapes, but the string matcher rejects numeric `n`; invalid
   negative/reversed bounds silently become false or inert rather than a
   diagnostic.
8. **Safety / discoverability:** `SpellingCorrection` is advertised and
    accepted as a `Names` option but is never read; `True` is a silent no-op.
9. **Safety:** `Once` advertises expiration options but only implements the
   `PersistenceTime -> 0` no-cache case; other known values silently cache for
   the whole session.
10. **Flexibility:** the documented operator forms for `Position`, `Count`,
    `MemberQ`, and `FreeQ` are absent. Each is call-planning work and needs the
    A/B gate.
11. **Convenience:** output-like string rendering hides `HoldForm`, but normal
    CLI/REPL value rendering prints the literal wrapper; the two user-facing
    output paths disagree.
12. **Flexibility / scoping boundary:** the staged three-argument
    `With[{...},{...},body]` form is missing. Current same-list bindings are
    intentionally sequential; no change to that existing semantics is proposed.

## Findings Table

### Scoping

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Block | WL | OK | boundary | Initializers are sequential under the current rum contract: the old state is cleared and a blank dynamic binding is pushed before each local initializer, so `x=10; Block[{x=x},HoldComplete[x]] -> HoldComplete[x]`; later initializers can see earlier local values (Evidence E3; `scoping/lexical/block.rs:59-70`; `rum-stdlib/src/tests/holding_and_control_part3.rs:297-320`). | Record the edge; propose no localization/initializer change in this audit. | |
| Block | WL | OK | safety | Dynamic state is restored on normal return, `Throw`, and `Abort`: the probe returns the local values/catch result and the outer `x` remains `10`; both initializer and body error paths restore definitions, attributes, options, syntax information, messages, and dynamic frames (`block.rs:312-399`, `:472-499`). | No change to restoration. | |
| Module | WL | gap | safety | **TRUST-BOUNDARY STOP:** after `Catch[Module[{auditLocal=1},Throw[fail]]]`, the fresh definition remains enumerable and readable as `auditLocal$1 -> 1` (Evidence E3). `finish_module_scope` only computes immediate reclamation for an `Ok` outcome, then retains error-path gensyms as GC candidates; the default sweep threshold is 200,000 (`module.rs:207-280`; `gensym_gc.rs:164-172`). | Escalate for the supervising decision. Per the hard rule, propose no renaming, reclamation, or scoping-semantic change in this audit. Any later evaluator-core fix needs the A/B gate. | L / decision |
| Module | WL | OK | boundary | Fresh renaming, initializer evaluation, closure capture, and normal-result reclamation are otherwise implemented; the head-start produces `x$1`, and escaping closures retain their locals. `$ModuleNumber` remains an honestly documented absent boundary. | Preserve the current renaming scheme; it is the downstream encapsulation trust boundary. | |
| With | WL | OK | boundary | Same-list bindings are intentionally sequential in rum: with outer `x=10`, `With[{x=1,y=x},HoldComplete[x,y]] -> HoldComplete[1,1]` (Evidence E3; `with.rs:31-46`; `rum-stdlib/src/tests/holding_and_control_part3.rs:32-49`). | Record the existing contract; propose no substitution-order change. | |
| With | WL | gap | flexibility | The staged `With[{...},{...},body]` declaration form is rejected as arity 3 and remains inert, with repeated fixed-point `With::argx` messages (Evidence E3; `with.rs:4-16`). | Record for the same supervising scoping decision; do not bolt a third argument onto the current sequential semantics. | M / decision |
| With | WL | OK | safety | Substitution reaches held positions, as required for lexical constants; `With[{x=1,...},HoldComplete[x,...]]` substitutes before the body evaluates (`with.rs:26-28`). | Preserve held-position substitution. | |
| Function | WL | OK | flexibility | Slot, named-symbol, named-list, two-argument attribute, and three-argument explicit-attribute forms all work. `Function[{x},HoldComplete[x],HoldAll][1+1]` retains raw `Plus[1,1]`; surplus application arguments follow the established variadic pure-function behavior and are ignored (Evidence E8; `function_and_subst_helpers.rs:91-139`; `scoping/callable.rs:9-28`; `runtime/tests/holding_and_control_part2.rs:53-80`). | No form or arity change. | |
| Unique | WL | OK | flexibility | Zero-arg, symbol/string prefix, nested list, `Unique[{"a","b"}]`, same-suffix symbol batches, and attribute forms work; generated names are registered and collision-checked (Evidence E8; `scoping/unique.rs:46-79`, `:90-189`). | No change; preserve the fresh-symbol trust property. | |

### Eval Control / Hold Family

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Hold | WL | gap | safety | `HoldAll` currently sets `allow_sequence_splice: false` for every held argument, so `Hold[Sequence[a,b]]` and a user `HoldAll` head retain the literal `Sequence`; ordinary `HoldAll` should hold evaluation but still allow the Sequence escape (Evidence E5; `eval_call_prepare.rs:18-47`). | Separate evaluation holding from sequence-splice permission; keep `HoldAllComplete`/`SequenceHold` blocking. Touches evaluator core → microbench A/B gate. | M/L |
| Hold | WL | OK | safety | Arithmetic stays raw and a direct `Evaluate` pierces the hold (`Hold[1+1]`, `Hold[Evaluate[1+1]] -> Hold[2]`). | Preserve the direct `Evaluate` escape while fixing Sequence independently. | |
| HoldComplete | WL | OK | safety | `HoldAllComplete` blocks both direct `Evaluate` and evaluator-time Sequence splicing (`HoldComplete[Evaluate[Plus[1,1]]]`, and a user complete-hold head retains `Sequence[a,b]`; Evidence E2, E5). | Preserve this complete boundary. | |
| HoldComplete | WL | gap | safety | Rule-binding substitution ignores the destination head's attributes and syntactically splices Sequence first: `a /. (a :> hc[Sequence[1,2]]) -> hc[1,2]`, while direct `hc[Sequence[1,2]]` remains nested under `HoldAllComplete` (Evidence E7; `substitution/binding_substitution.rs:281-323`). | Correct substitution/rebuild semantics without adding an attribute kind. Touches matcher/evaluator hot paths → microbench A/B gate. | L |
| HoldForm | WL | gap | convenience | The output-like `ToString` path hides the wrapper, but CLI/REPL response rendering uses canonical `Expr::Display` and prints `HoldForm[Plus[1,1]]` literally (Evidence E15; `kernel_data/text_rendering.rs:98-103`; `rum-terminal/src/lib.rs:38-47`). | Route terminal value presentation through the existing transparent output-like arm, without a typesetting expansion. A renderer-only fix needs no evaluator A/B; any evaluator behavior change would touch evaluator core → A/B gate. | S/M |
| HoldForm | WL | OK | boundary | Input-like rendering deliberately preserves the wrapper (`ToString[...,InputForm]`), while output/standard/traditional string forms hide it. That split is useful for round-tripping and is honestly described by the reference. | Preserve the input-like wrapper. | |
| HoldPattern | WL | OK | safety | Matcher compilation strips `HoldPattern` without evaluating its pattern, and the head-start match succeeds (`function_and_subst_helpers.rs:23-31`; Evidence E6). | No change. | |
| ReleaseHold | WL | gap | safety | `ReleaseHold[HoldForm[1+1]]` returns the still-held `HoldForm[Plus[1,1]]`. The supported-wrapper list contains `Hold`, the Hold attribute heads, `HoldComplete`, and `HoldPattern`, but omits `HoldForm` (`hold_thread.rs:17-30`; `function_and_subst_helpers.rs:34-60`; Evidence E5). | Include the supported `HoldForm` wrapper and pin the exact layer contract. Touches hold-family evaluator behavior → microbench A/B gate. | S/M |
| Evaluate | WL | OK | boundary | Piercing is hard-coded in argument planning, not caused by `Evaluate`'s own `HoldAll`: a held parent recognizes the wrapper at `eval_call_prepare.rs:35-40`. The attribute is still load-bearing for standalone `Evaluate`, because it prevents generic argument/Sequence preparation before `builtin_evaluate` explicitly evaluates its one raw argument (`builtin_adapters.rs:771-781`). | Do not align attributes cosmetically. Removing `HoldAll` would change behavior and touch evaluator core → A/B gate. | |
| Unevaluated | WL | gap | safety | Undefined and nonmatching calls lose the wrapper during no-match reconstruction; the latter can subsequently fire a definition (`q[Unevaluated[1+1]] -> hit[2]`). Builtins, held heads, matching definitions, functions, and upvalues consume the raw value correctly (Headline; Evidence E4). | Retain enough preparation provenance to restore the wrapper only on the no-consumer path. Touches evaluator core → microbench A/B gate. | L |
| Once | WL | OK | boundary | The deterministic cache is session-owned (`Session::once_values`), included in rollback snapshots, cleared by host `reset()`, and not serialized by definition snapshots; restore resets before replay (`session_state.rs:141`, `session_helpers.rs:38-84`; `rum-host/src/session.rs:161-200`). This is correctly `Capability::Pure`. | Preserve session-local deterministic state and reset/restore behavior. | |
| Once | WL | gap | safety | `Options[Once]` advertises `ExpirationDate` and `PersistenceTime`, but runtime logic only distinguishes integer `PersistenceTime -> 0`; every other known value, including `ExpirationDate`, silently means cache forever (`options/defaults.rs:72-75`; `package_tools.rs:599-607`). | While keeping `Pure`, fail closed on unsupported known option values (or document/implement only deterministic policies); do not introduce a clock read under this symbol. | S/M |
| Sequence | WL | gap | safety | It splices in ordinary calls and is blocked by `HoldAllComplete`/`SequenceHold`, but is also incorrectly blocked by ordinary `HoldAll` (Evidence E5). | Same evaluator-core correction as `Hold`; microbench A/B gate required. | M/L |
| Nothing | WL | OK | safety | `Nothing` is removed only from evaluated `List` arguments; it remains in `f[a,Nothing,b]` and inside a held list, then disappears after `ReleaseHold` (Evidence E5; `eval_call_prepare.rs:78-135`). | No change; the list-only boundary is intended. | |
| Sow | WL | OK | boundary | Outside any `Reap`, `Sow[5]` returns `5` and records nowhere. Inside scopes it groups the already evaluated value/tag in session state (`debug_tools/reap.rs:17-50`; Evidence E14). `Pure` is correct. | No capability change. | |
| Reap | WL | OK | flexibility | Body holding, default grouping, tag patterns, handlers, nested session scopes, and cleanup on error/suspension are implemented; `Reap[Sow[6]] -> {6,{{6}}}` (Evidence E14; `debug_tools/reap.rs:52-97`). | No change in this first-pass grouping subset. | |

### Pattern Atoms + Rule Heads

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Repeated | WL | gap | flexibility | Expression patterns accept `n`, `{n}`, and `{min,max}` (plus `Infinity`), but string patterns accept only list bounds: `StringCases["aaa",Repeated["a",2]]` stays inert while `{2}` works (Evidence E9; `matcher_and_symbols.rs:362-424`; `rum-stdlib/src/strings/patterns.rs:306-316`, `:711-728`). | Share the documented count-shape parser across expression and string repetition. Matcher changes require the microbench A/B gate. | M |
| Repeated | WL | gap | safety | Negative or reversed bounds silently become a nonmatch in expression patterns and an inert string call, with no diagnostic (Evidence E9). | Validate malformed repetition bounds at pattern normalization and fail closed rather than report a false data mismatch. Touches matcher core → microbench A/B gate. | M |
| RepeatedNull | WL | gap | safety | The same split parser and silent-invalid boundary applies to `RepeatedNull`; its unbounded zero-or-more default works, but numeric bounded string use and malformed bounds are not coherently handled (`matcher_and_symbols.rs:352-375`; `strings/patterns.rs:319-329`). | Share validation with `Repeated`; matcher-core A/B gate. | M |
| Longest | WL | OK | flexibility | It is consumed by both the string matcher and expression sequence matcher; expression binding chooses `{a,b}` before `{c}` in the probe (Evidence E6, E9; `matcher_and_symbols.rs:301-309`). | No string-only restriction. | |
| Shortest | WL | OK | flexibility | It is likewise consumed in both domains and chooses `{a}` before `{b,c}` for the expression sequence probe (Evidence E6, E9). | No change. | |
| IgnoreCase | WL | OK | flexibility | It is read by `Names` and the current string matching/search consumers; live case-sensitive/case-insensitive `Names` results differ (Evidence E10; `namespace_tools/names.rs:225-229`; `strings/patterns.rs:507-680`). Unsupported option syntax is rejected by each consumer parser rather than treated as a pattern atom. | No change. | |
| Overlaps | WL | OK | flexibility | `StringCases`, `StringPosition`, and `StringCount` consume it with their documented defaults; the probe distinguishes one non-overlapping from two overlapping `"aa"` matches (Evidence E10; `strings/patterns.rs:582-680`). | No change. | |
| Whitespace | WL | OK | boundary | It is an inert protected marker consumed as the string-pattern atom for one-or-more whitespace; the probe extracts the whitespace run (Evidence E10; `strings/patterns.rs:197-199`). | Keep it string-pattern-only. | |
| NumberString | WL | OK | boundary | It is an inert protected decimal-style string-pattern atom; the probe extracts `"-1.5"` (`strings/patterns.rs:221-223`; Evidence E10). | Keep the documented folk decimal subset. | |
| SpellingCorrection | WL | gap | safety | `Names` registers and accepts the option but reads only `IgnoreCase`; `SpellingCorrection -> True` produces the same empty typo result as `False`, without a message (Evidence E10; `options/defaults.rs:76-80`; `namespace_tools/names.rs:200-229`, `:322-327`). | Until spelling correction exists, reject the non-default `True` value explicitly; default `False` remains byte-identical. | S |
| Rule | WL | OK | boundary | Immediate RHS evaluation and non-splicing construction are hard-coded: the special form receives raw args, evaluates lhs/RHS explicitly, and returns `Rule[a,Sequence[1,2]]` (Evidence E6-E7; `definitions/definitions.rs:67-80`). No hold attribute is needed. | Do not add `HoldRest`/`SequenceHold` as metadata hygiene; direct-call behavior would be unchanged. | |
| RuleDelayed | WL | OK | boundary | Construction evaluates the pattern-aware lhs but stores the raw RHS (`definitions/definitions.rs:82-95`); the side-effect probe leaves the counter at `1` after constructing one immediate and one delayed rule (Evidence E7). | Preserve delayed construction. | |
| RuleDelayed | WL | gap | safety | Consumer behavior diverges: inside a held `ReplaceAll` position, `a :> (c=c+1;b)` is inserted as an unevaluated `CompoundExpression`, while `ReplacePart` and `ReplaceAt` evaluate the delayed RHS before insertion (Evidence E7; `definitions/application/replace_all.rs:93-166`; `structure_data/replace/replace_part.rs:199-215`; `replace_at.rs:111-128`). | Separate “RHS fires when matched” from subsequent held-position evaluation without designing `ReplaceAllUnheld` here. Touches rewrite/matcher/evaluator hot paths → microbench A/B gate. | L |
| NormalizeRules | rum | OK | convenience | The rum-native debugger accepts a rule, rule list, or symbol resolving to rules, preserves immediate/delayed heads, returns one explicit canonical list, and diagnoses invalid input (`definitions/definitions.rs:4-12`, `:97-113`; Evidence E14). | No change; the contract is composable and discoverable in the reference. | |

### Structural / Pattern-Query

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Part | WL | OK | flexibility | Negative indices, `All`, positive/negative/step spans, multi-selection, and multidimensional selectors work; the probe covers all requested shapes (Evidence E11; `structure_data/selectors/parse.rs:23-205`). | No additional selector form. | |
| Extract | WL | OK | flexibility | Single/multiple positions, nested selectors, the three-argument wrapper, and `Extract[pos]` operator form work. `Extract[Hold[1+1],{1},Hold]` returns held raw `Plus[1,1]` (Evidence E11; `selectors.rs:136-204`; `selectors/extract.rs:6-113`). | No change. | |
| ReplacePart | WL | gap | safety | The reference says exact, Span, and pattern-based positions, but `(2;;4;;2)->x` silently leaves the list unchanged (Evidence E12). The exact parser recognizes only integer/key selectors; a Span falls through as a literal pattern that cannot match integer position paths (`replace/parsing.rs:48-121`). | Implement the already-documented span target selection; do not weaken the reference contract. Keep default rule ordering. | M |
| ReplacePart | WL | OK | flexibility | Multiple exact positions, multiple rules, pattern positions, heads control, and delayed RHS evaluation work; later same-path rules win and prefix conflicts are deterministic (Evidence E12; `replace/plan.rs:60-112`). | Preserve these forms. | |
| ReplaceAt | WL | OK | flexibility | Modern direct and operator forms, rule lists, and multiple exact positions work; delayed RHS values are evaluated before insertion (Evidence E6, E12; `replace/replace_at.rs:4-144`). | No change in the documented exact-position subset. | |
| Position | WL | OK | flexibility | Standard levelspec, first-`n`, and `Heads` controls work; the probe limits the result to `{{1}}` (`tree_walkers/position.rs:4-86`; Evidence E13). | Preserve data forms. | |
| Position | WL | gap | flexibility | The documented `Position[patt]` operator form is absent; unary input is treated as an invalid data call (Evidence E13; `position.rs:4-6`). | Add the unambiguous operator wrapper. Touches call planning → microbench A/B gate. | M |
| Count | WL | OK | flexibility | Standard levelspec and `Heads` control work (`tree_walkers/count_like.rs:4-21`, `:81-100`; Evidence E13). | Preserve data forms. | |
| Count | WL | gap | flexibility | The documented `Count[patt]` operator form is absent and emits `Count::argtu` (Evidence E13). | Add the unambiguous operator wrapper. Touches call planning → microbench A/B gate. | M |
| MemberQ | WL | OK | flexibility | Standard levelspec and `Heads` control work; the explicit `Infinity` probe reaches a nested member (`count_like.rs:23-40`, `:123-143`; Evidence E13). | Preserve data forms. | |
| MemberQ | WL | gap | flexibility | The documented `MemberQ[patt]` operator form is absent and emits `MemberQ::argtu` (Evidence E13). | Add the unambiguous operator wrapper. Touches call planning → microbench A/B gate. | M |
| FreeQ | WL | OK | flexibility | Standard levelspec and `Heads` control work, including the current default-head policy (`count_like.rs:42-59`, `:165-185`; Evidence E13). | Preserve data forms. | |
| FreeQ | WL | gap | flexibility | The documented `FreeQ[patt]` operator form is absent and emits `FreeQ::argtu` (Evidence E13). | Add the unambiguous operator wrapper. Touches call planning → microbench A/B gate. | M |
| Level | WL | OK | flexibility | Standard positive/negative levels, third-argument apply head, and `Heads` option (including the four-argument combination) work (`tree_walkers/level.rs:4-68`; Evidence E6, E13). | No change. | |
| Thread | WL | OK | flexibility | One-argument default List threading, `Thread[expr,h]`, and the documented current third-argument sequence-spec subset work (`hold_thread.rs:32-105`; Evidence E6, E13). | No change. | |
| MissingQ | WL | OK | convenience | It coheres with the installed `Missing[...]` family: any `Missing` call is `True`, while the bare marker and ordinary values are `False` (`runtime/mod.rs:75-93`; Evidence E13). | No change. | |

### Forms + Marker Symbols

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| InputForm | WL | OK | boundary | As a direct head it is inert; as a marker it selects the current parser-compatible `ToExpression` subset and input-like `ToString` rendering. The direct-head inertness is consistent with the reference calling it a formatting symbol, not a renderer (`strings/text/builtins.rs:407-439`, `:576-595`; Evidence E15). | No wrapper semantics or typesetting expansion. | |
| InputForm | WL | gap | convenience | The reference mentions only the `ToExpression` consumer, although `ToString[...,InputForm]` is also implemented and observably distinct for `HoldForm` (Evidence E15; `rum-stdlib/src/strings/text/builtins.rs:585-595`). | Update the generated summary in a later documentation batch; no behavior change. | S |
| OutputForm | WL | OK | boundary | It is an inert protected marker consumed by `ToString`'s output-like renderer; direct `OutputForm[Hold[...]]` remains explicit (Evidence E15; `strings/text/builtins.rs:585-595`). | No front-end formatter expansion. | |
| StandardForm | WL | OK | boundary | It is accepted by both `ToExpression` and output-like `ToString`; direct wrapper use is inert (Evidence E15). | Preserve the shared current subset. | |
| StandardForm | WL | gap | convenience | Its reference summary mentions only `ToExpression`, omitting the implemented `ToString` consumer (Evidence E15; `rum-stdlib/src/strings/text/builtins.rs:585-595`). | Update the generated summary; no behavior change. | S |
| TraditionalForm | WL | OK | boundary | It is an inert marker selecting the same current output-like `ToString` renderer. The absence of distinct traditional typesetting is an honest folk-scope boundary (Evidence E15). | No typesetting arms race. | |
| Short | WL | OK | convenience | Both `Short[expr]` and positive-integer `Short[expr,n]` return an honest canonical-text string with middle ellipsis; malformed limits fail with `Short::argform` (`debug_tools/basic.rs:3-28`, `:133-150`; Evidence E15). | Keep the documented string-returning folk subset. | |
| Automatic | WL | OK | boundary | It is a protected inert marker shared by option/spec parsers; it has no standalone evaluation promise. | No change. | |
| $Failed | WL | OK | safety | It is a protected inert failure marker returned by fail-closed builtin paths; no ambient state or capability is attached. | No change. | |

## Hold-semantics design input (units 8-10 parked item)

This section is factual input only. It proposes neither a new attribute kind
nor a design for `SubValuesHoldAll` or `ReplaceAllUnheld`.

### Hard-Coded-Hold Trace

One premise in the brief is stale against the current tree: `Attribute` **does
contain** `SequenceHold` (and `NHoldAll`) at
`crates/rum-runtime/src/runtime/metadata.rs:32-48`; `sequence_hold()` actively
checks `SequenceHold` and `HoldAllComplete` at `:223-229`. No new attribute kind
is proposed. The important fact is still that the heads below do not derive
their behavior from those attributes.

| Head / mechanism | Where behavior lives | Attribute-driven? | What an attribute change would do today |
|---|---|---|---|
| `Rule` | Registered as `BuiltinSpec::special` (`builtin_catalog/rules.rs:125-129`); special dispatch passes `raw_args` (`eval_dispatch/mod.rs:61-79`); `eval_rule` evaluates the pattern-aware lhs and RHS explicitly (`definitions/definitions.rs:67-80`). | No. | Adding existing `HoldRest` or `SequenceHold` would change introspection only for direct calls; special forms never enter generic preparation. It would be dead-weight metadata. |
| `RuleDelayed` | Same special dispatch; `eval_rule_delayed` evaluates only the pattern-aware lhs and clones the raw RHS (`definitions.rs:82-95`). | No. | Adding `HoldRest`/`SequenceHold` is likewise a direct-call no-op and duplicates the special path. |
| `Evaluate` pierce | A held parent's planner recognizes the literal wrapper and sets `should_evaluate=true`, `allow_sequence_splice=true` (`eval_call_prepare.rs:35-40`). | The pierce itself is not. | Removing `Evaluate`'s own `HoldAll` would not remove the parent pierce, but would let standalone arguments/Sequence be generically prepared before `builtin_evaluate`; it is therefore behavior-changing, not hygiene. |
| standalone `Evaluate` | `builtin_evaluate` explicitly evaluates its one raw argument (`builtin_adapters.rs:771-781`). | Its raw-input guarantee uses `HoldAll`. | `HoldAll` is load-bearing for handler ownership of standalone evaluation; keep it unless separately redesigned and benchmarked. |
| `Unevaluated` | The planner strips the direct wrapper in both held and ordinary positions (`eval_call_prepare.rs:27-32`, `:50-55`); the builtin itself also unwraps standalone unary calls (`builtin_adapters.rs:751-760`). | Wrapper recognition is hard-coded. | No existing attribute can encode “restore if no call stage consumes me”; metadata addition cannot fix the headline gap. |
| `Sequence` splicing | `append_prepared_argument` splices prepared `Sequence` values unless the planner disallows it or `sequence_hold()` is true (`eval_call_prepare.rs:65-75`). List has a separate `Nothing`-aware append path (`:78-105`). | Partly: `SequenceHold`/`HoldAllComplete` block it, but ordinary `HoldAll` currently also blocks via the hard-coded `allow_sequence_splice=false`. | Adding `SequenceHold` to a normal head blocks splicing. Adding it to special `Rule` heads is a no-op. Fixing ordinary `HoldAll` needs planner logic, not a new kind. |
| `Nothing` removal | Only the List-specific preparation helpers drop the symbol (`eval_call_prepare.rs:78-135`). | Head-name special case, not an attribute. | An attribute addition is unrelated and would be dead weight. |
| `ReleaseHold` | The builtin loops `release_hold_once` over consecutive recognized wrappers, then evaluates (`hold_thread.rs:17-30`); the wrapper-name match is explicit (`function_and_subst_helpers.rs:34-60`). | No. | An attribute cannot add the missing `HoldForm` case or define the layer policy. |
| rule RHS Sequence substitution | `substitute_rule_call_changed_with_arg_alphas` splices any substituted or literal Sequence argument while rebuilding (`binding_substitution.rs:281-323`). | No destination-head attribute lookup. | Adding `SequenceHold` to the destination head works only during later evaluator preparation; it is too late to undo this syntactic splice. |

### Held Arguments Through Matching And Replacement Today

1. Pattern-bearing APIs resolve the pattern under their special/raw path,
   normalize it, and compile a matcher
   (`definition_catalog/rewrite_rules.rs:4-35`, `:119-145`). `HoldPattern` is
   stripped structurally; it is not evaluated.
2. A `Rule` RHS is evaluated when the rule object is constructed. A
   `RuleDelayed` RHS is stored raw. Both compile into `RewriteRule` with the
   same matcher and a `delayed` flag (`definition_analysis_helpers.rs:546-579`).
3. On a match, binding substitution rebuilds the RHS. It is scope-aware for
   `Module`, `With`, and explicit `Function`, but Sequence splicing during that
   rebuild does not consult hold attributes (`binding_substitution.rs:126-333`).
4. `ReplaceAll` traverses every head and argument after a root miss, regardless
   of the evaluated head's hold attributes
   (`definitions/application/replace_all.rs:44-90`). Thus it rewrites inside
   `Hold`, `HoldForm`, and `HoldComplete` today; it does not implement “unheld
   parts only.”
5. After a match, the shared rewrite applicator returns the substituted RHS
   without evaluating it (`replace_all.rs:93-166`). `ReplaceAll` evaluates only
   the fully rebuilt result (`kernel_data/rewrite_data/replace/all.rs:49-54`).
   Consequently a delayed RHS inserted beneath `Hold` does **not** fire there.
   By contrast, `ReplacePart` explicitly evaluates each delayed RHS before
   insertion (`replace_part.rs:199-215`) and `ReplaceAt` evaluates the matched
   replacement before writing the path (`replace_at.rs:111-128`).
6. Therefore `ReplaceAllUnheld` would need, at minimum, a traversal decision
   based on the same evaluated-head hold plan; this audit deliberately does not
   specify that design. The separate existing defect is that delayed RHS
   firing is consumer-dependent even before an “unheld-only” traversal exists.

### The Unit-9 Half Of The SubValues Hold Question

For `f[a][b]`, the evaluator first evaluates the nested head `f[a]`
(`eval_core/call.rs:4-18`). It then builds the outer call plan. Attribute lookup
returns attributes only for a literal symbol or a parsed `Function`; a general
nested head gets the empty set (`eval_call_prepare.rs:426-447`). The base symbol
`f` is consulted only to decide whether a `SubValues` stage exists
(`eval_dispatch/mod.rs:15-21`). Therefore:

- `a` is evaluated while the inner call/head stabilizes;
- `b` is evaluated by ordinary empty-attribute outer argument preparation;
- only then is the candidate `f[evaluated-a][evaluated-b]` built and matched by
  `try_apply_sub_values` (`definitions/application/sub_values.rs:4-43`);
- after a match, its RHS is substituted and evaluated (`sub_values.rs:117-166`,
  `:209-255`).

This is exactly the current pre-hold-extension behavior reproduced by unit 8:
both side-effecting arguments increment before the subvalue returns
`HoldComplete[1,2]`. No base-head hold metadata is inherited by the outer call,
and no design change is proposed here.

## Evidence Appendix

Twenty-one bounded CLI invocations were used, below the ~40 probe budget. Every
CLI invocation fed empty stdin. No external engine, network, dependency, code,
or test change was used.

### E1: Calibrated Attribute Survey

The reference parser requires exactly one Attributes line per entry and
extracts **every** backticked token. Its first draft used `.+` while DOTALL was
active, greedily consumed headings, and failed once with `KeyError: 'Block'`;
the corrected heading group is `[^\n]+`. It did not fail a second time.

```text
$ python3 - <<'PY'
from pathlib import Path
import re

names = [
    'Block', 'Module', 'With', 'Function', 'Unique', 'Hold', 'HoldComplete',
    'HoldForm', 'HoldPattern', 'ReleaseHold', 'Evaluate', 'Unevaluated', 'Once',
    'Sequence', 'Nothing', 'Sow', 'Reap', 'Repeated', 'RepeatedNull', 'Longest',
    'Shortest', 'IgnoreCase', 'Overlaps', 'Whitespace', 'NumberString',
    'SpellingCorrection', 'Rule', 'RuleDelayed', 'NormalizeRules', 'Part',
    'Extract', 'ReplacePart', 'ReplaceAt', 'Position', 'Count', 'MemberQ',
    'FreeQ', 'Level', 'Thread', 'MissingQ', 'InputForm', 'OutputForm',
    'StandardForm', 'TraditionalForm', 'Short', 'Automatic', '$Failed',
]
text = Path('docs/reference/builtins.md').read_text()
blocks = {
    match.group(1): match.group(2)
    for match in re.finditer(r'^### ([^\n]+)\n\n(.*?)(?=^### |\Z)', text, re.M | re.S)
}
parsed = {}
for name in names:
    lines = [line for line in blocks[name].splitlines() if line.startswith('- Attributes:')]
    if len(lines) != 1:
        raise SystemExit(f'{name}: expected one Attributes line, got {len(lines)}')
    parsed[name] = re.findall(r'`([^`]+)`', lines[0])
print(f'unit-9 entries = {len(names)}')
print(f'protected = {sum("Protected" in attrs for attrs in parsed.values())}')
print(f'unprotected = {[name for name, attrs in parsed.items() if "Protected" not in attrs]}')
print('hold-class =')
for name, attrs in parsed.items():
    holds = [attr for attr in attrs if attr in {'HoldAll', 'HoldAllComplete', 'HoldFirst', 'HoldRest'}]
    if holds:
        print(f'  {name}: {holds[0]}')
PY
unit-9 entries = 47
protected = 47
unprotected = []
hold-class =
  Block: HoldAll
  Module: HoldAll
  With: HoldAll
  Function: HoldAll
  Hold: HoldAll
  HoldComplete: HoldAllComplete
  HoldForm: HoldAll
  HoldPattern: HoldAll
  ReleaseHold: HoldAll
  Evaluate: HoldAll
  Unevaluated: HoldAllComplete
  Once: HoldAll
  Reap: HoldFirst
  NormalizeRules: HoldAll
```

The first live list-map cross-check returned 46 entries because the evaluated
list correctly dropped its `Nothing` element. That calibrated a second audit
instrument rather than being counted as 47/47 evidence; the standalone check
then supplied the missing row.

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Attributes /@ {Block, Module, With, Function, Unique, Hold, HoldComplete, HoldForm, HoldPattern, ReleaseHold, Evaluate, Unevaluated, Once, Sequence, Nothing, Sow, Reap, Repeated, RepeatedNull, Longest, Shortest, IgnoreCase, Overlaps, Whitespace, NumberString, SpellingCorrection, Rule, RuleDelayed, NormalizeRules, Part, Extract, ReplacePart, ReplaceAt, Position, Count, MemberQ, FreeQ, Level, Thread, MissingQ, InputForm, OutputForm, StandardForm, TraditionalForm, Short, Automatic, $Failed}'
List[List[HoldAll, Protected], List[HoldAll, Protected], List[HoldAll, Protected], List[HoldAll, Protected], List[Protected], List[HoldAll, Protected], List[HoldAllComplete, Protected], List[HoldAll, Protected], List[HoldAll, Protected], List[HoldAll, Protected], List[HoldAll, Protected], List[HoldAllComplete, Protected], List[HoldAll, Protected], List[Protected], List[Protected], List[HoldFirst, Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[HoldAll, Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[Protected]]
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Attributes[Nothing]'
List[Protected]
```

This live result and spot probes of actual behaviors cross-check the
script-derived reference metadata; neither the first-backtick regex trap nor
the evaluated-`Nothing` list trap is trusted.

### E2: Head-Start Scoping And Hold Reproduction

```text
$ printf '' | scripts/run-bounded.py --timeout 180 -- cargo run -q -p rum-cli -- --eval '{Module[{x}, x], Module[{x = 2}, x + 1], Block[{y = 5}, y * 2], With[{z = 3}, z^2], Unique[], Unique["t"], Hold[1 + 1], ReleaseHold[Hold[1 + 1]], Hold[Evaluate[1 + 1]], HoldComplete[Evaluate[1 + 1]], f[Unevaluated[1 + 1]], Once[1 + 1]}'
List[x$1, 3, 10, 9, $3, t4, Hold[Plus[1, 1]], 2, Hold[2], HoldComplete[Evaluate[Plus[1, 1]]], f[2], 2]
```

### E3: Scoping Error Paths And Initializers

The trust-boundary leak is directly enumerable and readable even though the
thrown value is the unrelated symbol `fail`:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{Names["Global`auditLocal$*"], Catch[Module[{auditLocal = 1}, Throw[fail]]], Names["Global`auditLocal$*"], Global`auditLocal$1}'
List[List[], fail, List["Global`auditLocal$1"], 1]
```

`Block` restoration is green, and the same probe records the current sequential
initializer/binding semantics plus the missing staged `With` form:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'x = 10; {Block[{x = x}, HoldComplete[x]], x, Catch[Block[{x = 2}, Throw[x]]], x, CheckAbort[Block[{x = 3}, Abort[]], caught], x, With[{x = 1, y = x}, HoldComplete[x, y]], With[{x = 1}, {y = x}, HoldComplete[x, y]]}'
List[HoldComplete[x], 10, 2, 10, caught, 10, HoldComplete[1, 1], With[List[Set[x, 1]], List[Set[y, x]], HoldComplete[x, y]]]
! With::argx: expected exactly 2 argument(s), got 3
! With::argx: expected exactly 2 argument(s), got 3
! With::argx: expected exactly 2 argument(s), got 3
```

Repeated arity messages are fixed-point re-evaluation of one inert call, not
three probe invocations.

### E4: `Unevaluated` Consumer / No-Consumer Boundary

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'g[x_] := HoldComplete[x]; q[x_Integer] := hit[x]; u /: h[u] := caught; {f[Unevaluated[1 + 1]], g[Unevaluated[1 + 1]], q[Unevaluated[1 + 1]], Function[x, HoldComplete[x]][Unevaluated[1 + 1]], h[Unevaluated[u]], Hold[Unevaluated[1 + 1]], f[Unevaluated[Sequence[a, b]]], Hold[Unevaluated[Sequence[a, b]]], Length[Unevaluated[1 + 1]], Head[Unevaluated[1 + 1]]}'
List[f[2], HoldComplete[Plus[1, 1]], hit[2], HoldComplete[Plus[1, 1]], caught, Hold[Plus[1, 1]], f[a, b], Hold[Sequence[a, b]], 2, Plus]
```

### E5: Sequence, Nothing, And Release Boundaries

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'SetAttributes[h, HoldAll]; SetAttributes[hc, HoldAllComplete]; SetAttributes[sh, SequenceHold]; {f[Sequence[a, b]], h[Sequence[a, b]], h[Evaluate[Sequence[a, b]]], hc[Sequence[a, b]], sh[Sequence[a, b]], Rule[a, Sequence[1, 2]], RuleDelayed[a, Sequence[1, 2]], {a, Nothing, b}, f[a, Nothing, b], Hold[{a, Nothing, b}], ReleaseHold[Hold[{a, Nothing, b}]], ReleaseHold[Hold[Hold[1 + 1]]], ReleaseHold[HoldForm[1 + 1]], ReleaseHold[HoldPattern[1 + 1]], ReleaseHold[HoldAll[1 + 1]]}'
List[f[a, b], h[Sequence[a, b]], h[a, b], hc[Sequence[a, b]], sh[Sequence[a, b]], Rule[a, Sequence[1, 2]], RuleDelayed[a, Sequence[1, 2]], List[a, b], f[a, Nothing, b], Hold[List[a, Nothing, b]], List[a, b], 2, HoldForm[Plus[1, 1]], 2, 2]
```

### E6: Head-Start Pattern And Structure Reproduction

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{Length[Unevaluated[1 + 1]], Head[Unevaluated[1 + 1]], MatchQ[{1, 1, 2}, {Repeated[1], 2}], MatchQ[{1, 1, 1}, {Repeated[1, {2}]}], StringCases["aaa", Longest[RepeatedNull["a"]]], StringCases["aaa", Shortest[Repeated["a"]]], MatchQ[Hold[1 + 1], Hold[HoldPattern[1 + 1]]], ReplaceAt[{1, 2, 3}, x_ :> x + 10, 2], ReplacePart[{1, 2, 3}, 2 -> b], Part[{1, 2, 3, 4, 5}, 2 ;; 4], Thread[f[{1, 2}, {a, b}]], Position[{1, {1, 2}}, 1], Level[{1, {2, {3}}}, {-1}], Count[{1, 2, 1, 3}, 1], Rule[a, Sequence[1, 2]]}'
List[2, Plus, True, False, List["aaa", ""], List["a", "a", "a"], True, List[1, 12, 3], List[1, b, 3], List[2, 3, 4], List[f[1, a], f[2, b]], List[List[1], List[2, 1]], List[1, 2, 3], 2, Rule[a, Sequence[1, 2]]]
```

### E7: Hard-Coded Rules And Held Replacement

Construction evaluates only the immediate RHS:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'c = 0; ri = Rule[a, (c = c + 1; Sequence[1, 2])]; rd = RuleDelayed[a, (c = c + 10; Sequence[3, 4])]; {ri, rd, c, Attributes[Evaluate], Attributes[Rule], Attributes[RuleDelayed], Attributes[SequenceHold]}'
List[Rule[a, Sequence[1, 2]], RuleDelayed[a, CompoundExpression[Set[c, Plus[c, 10]], Sequence[3, 4]]], 1, List[HoldAll, Protected], List[Protected], List[Protected], List[]]
```

Replacement consumers and pre-evaluator substitution then diverge:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'c = 0; ra = Hold[a] /. (a :> (c = c + 1; b)); rp = ReplacePart[Hold[a], {1} :> (c = c + 10; b)]; rt = ReplaceAt[Hold[a], a :> (c = c + 100; b), {1}]; SetAttributes[hc, HoldAllComplete]; rs = a /. (a :> hc[Sequence[1, 2]]); {ra, rp, rt, c, rs, hc[Sequence[1, 2]]}'
List[Hold[CompoundExpression[Set[c, Plus[c, 1]], b]], Hold[b], Hold[b], 110, hc[1, 2], hc[Sequence[1, 2]]]
```

### E8: Function And Unique Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{Function[{x, y}, HoldComplete[x, y]][1 + 1, 2 + 2], Function[x, HoldComplete[x]][1 + 1], Function[{x}, HoldComplete[x], HoldAll][1 + 1], Function[HoldComplete[#], HoldAll][1 + 1], Function[{x}, x][1, 2], Function[{}, done][1]}'
List[HoldComplete[2, 4], HoldComplete[2], HoldComplete[Plus[1, 1]], HoldComplete[Plus[1, 1]], 1, done]
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{Unique[{"a", "b"}], Unique[{u, v}], Attributes /@ Unique[{p, q}, HoldAll]}'
List[List[a1, b2], List[u$3, v$3], List[List[HoldAll], List[HoldAll]]]
```

The final two Function results confirm the repository's established
surplus-argument behavior; they are not filed as a gap.

### E9: Repetition Counts And Greediness

Expression patterns cover the positive shapes and both greediness directions;
invalid shapes silently return `False`:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{MatchQ[f[1, 1, 2], f[Repeated[1, 2], 2]], MatchQ[f[1, 2], f[Repeated[1, {1, 2}], 2]], MatchQ[f[1, 1, 1], f[Repeated[1, {2}], 1]], MatchQ[f[2], f[RepeatedNull[1], 2]], MatchQ[f[1], f[Repeated[1, -1]]], MatchQ[f[1], f[Repeated[1, {2, 1}]]], f[a, b, c] /. f[Longest[x__], y__] :> HoldComplete[{x}, {y}], f[a, b, c] /. f[Shortest[x__], y__] :> HoldComplete[{x}, {y}]}'
List[True, True, True, True, False, False, HoldComplete[List[a, b], List[c]], HoldComplete[List[a], List[b, c]]]
```

The string parser accepts `{n}` but not the equivalent numeric `n`, and leaves
invalid bounds inert without a message:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{StringCases["aaa", Repeated["a", 2]], StringCases["aaa", Repeated["a", {2}]], StringCases["a", Repeated["a", -1]], StringCases["a", Repeated["a", {2, 1}]]}'
List[StringCases["aaa", Repeated["a", 2]], List["aa"], StringCases["a", Repeated["a", -1]], StringCases["a", Repeated["a", List[2, 1]]]]
```

### E10: String / Names Marker Consumers

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'spellingAuditTarget = 1; {Names["Global`spellingaudittarget", IgnoreCase -> False], Names["Global`spellingaudittarget", IgnoreCase -> True], Names["Global`spelingAuditTarget", SpellingCorrection -> False], Names["Global`spelingAuditTarget", SpellingCorrection -> True], StringCases["aaa", "aa", Overlaps -> False], StringCases["aaa", "aa", Overlaps -> True], StringCases[" \t-1.5", {Whitespace, NumberString}]}'
List[List[], List["Global`spellingAuditTarget"], List[], List[], List["aa"], List["aa", "aa"], List[" \t", "-1.5"]]
```

### E11: Part And Extract Surface

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{Part[{a, b, c, d, e}, -2], Part[{a, b, c, d, e}, 1 ;; 5 ;; 2], Part[{{a, b}, {c, d}}, All, -1], Part[{a, b, c}, {1, 3}], Extract[{{a, b}, {c, d}}, {{1, 2}, {2, 1}}], Extract[Hold[1 + 1], {1}, Hold], Extract[{2, 1}][{{a, b}, {c, d}}]}'
List[d, List[a, c, e], List[b, d], List[a, c], List[b, c], Hold[Plus[1, 1]], c]
```

### E12: ReplacePart / ReplaceAt Surface

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{ReplacePart[{a, b, c, d}, {{1}, {3}} -> x], ReplacePart[{a, b, c}, {1 -> x, 3 -> y}], ReplacePart[{a, b, c, d}, (2 ;; 4 ;; 2) -> x], ReplaceAt[{1, 2, 3}, x_Integer :> x + 10, {{1}, {3}}], ReplaceAt[x_Integer :> x + 1, {2}][{1, 2, 3}]}'
List[List[x, b, x, d], List[x, b, y], List[a, b, c, d], List[11, 2, 13], List[1, 3, 3]]
```

### E13: Pattern Queries And Operator Forms

The data forms are broad:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{Position[f[a, f[a]], a, Infinity, 1, Heads -> False], Count[f[a, f[a]], a, Infinity], MemberQ[f[g[a]], a, Infinity], FreeQ[f[g[a]], a, {1}], Level[f[a, g[b]], {1}, h], Level[f[a], 1, List, Heads -> True], Thread[f[h[1, 2], h[a, b]], h], MissingQ[Missing["KeyAbsent"]], MissingQ[Missing], MissingQ[1]}'
List[List[List[1]], 2, True, True, h[a, g[b]], List[f, a], h[f[1, a], f[2, b]], True, False, False]
```

Only `Extract[pos]` supplies its documented operator wrapper:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{Extract[{1}], Position[_Integer], Count[_Integer], MemberQ[_Integer], FreeQ[_Integer]}'
List[Function[Extract[Slot[1], List[1]]], Position[Blank[Integer]], Count[Blank[Integer]], MemberQ[Blank[Integer]], FreeQ[Blank[Integer]]]
! Position::argtu: expected between 2 and 5 argument(s), got 1
! Count::argtu: expected between 2 and 4 argument(s), got 1
! MemberQ::argtu: expected between 2 and 4 argument(s), got 1
! FreeQ::argtu: expected between 2 and 4 argument(s), got 1
! Position::argtu: expected between 2 and 5 argument(s), got 1
! Count::argtu: expected between 2 and 4 argument(s), got 1
! MemberQ::argtu: expected between 2 and 4 argument(s), got 1
! FreeQ::argtu: expected between 2 and 4 argument(s), got 1
```

### E14: Once, Sow/Reap, And NormalizeRules

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'n = 0; rules = {a -> 1, b :> 2}; {Once[n = n + 1], Once[n = n + 1], Once[n = n + 1, "alt"], n, Sow[5], Reap[Sow[6]], NormalizeRules[rules], NormalizeRules[a -> Sequence[1, 2]]}'
List[1, 1, 2, 2, 5, List[6, List[List[6]]], List[Rule[a, 1], RuleDelayed[b, 2]], List[Rule[a, Sequence[1, 2]]]]
```

`Once` cache lifecycle is also code-visible: a fresh `Session` initializes an
empty map (`session/session_state.rs:358-380`), host reset rebuilds the session
(`rum-host/src/session.rs:161-173`), and snapshot rustdoc explicitly excludes
non-definition state before restore resets (`:175-200`).

### E15: Forms And Short

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval '{HoldForm[1 + 1], ToString[HoldForm[1 + 1], OutputForm], ToString[HoldForm[1 + 1], StandardForm], ToString[HoldForm[1 + 1], InputForm], ToString[HoldForm[1 + 1], TraditionalForm], ToExpression["1 + 1", InputForm], ToExpression["1 + 1", StandardForm], InputForm[Hold[1 + 1]], OutputForm[Hold[1 + 1]], StandardForm[Hold[1 + 1]], TraditionalForm[Hold[1 + 1]], Short[Range[20], 18], Automatic, $Failed}'
List[HoldForm[Plus[1, 1]], "1 + 1", "1 + 1", "HoldForm[1 + 1]", "1 + 1", 2, 2, InputForm[Hold[Plus[1, 1]]], OutputForm[Hold[Plus[1, 1]]], StandardForm[Hold[Plus[1, 1]]], TraditionalForm[Hold[Plus[1, 1]]], "List[1, 2...9, 20]", Automatic, $Failed]
```

## Triage Sketch

The Module result is a **decision gate, not a proposed fix batch**: the fresh
symbol remains observable after a caught error, and the brief orders an
immediate STOP on scoping design. The staged `With` form also remains a scoping
decision rather than a batch below. Current sequential `Block`/`With` semantics
are recorded, not proposed for change. No audit batch below changes Module
renaming/scoping.

1. **Evaluator hold correctness** (safety, L): restore `Unevaluated` only on a
   true no-consumer path, separate ordinary `HoldAll` from Sequence blocking,
   and make `ReleaseHold` cover `HoldForm` with a pinned layer contract. This
   touches evaluator core → **mandatory core-language microbench A/B gate**.
2. **Rule firing in held replacement** (safety, L): make delayed RHS firing
   consistent across `ReplaceAll`/`ReplacePart`/`ReplaceAt`, and stop
   substitution-time Sequence splicing from bypassing `HoldAllComplete`.
   Record—but do not design—`ReplaceAllUnheld` separately. This touches
   matcher/rewrite/evaluator hot paths → **mandatory A/B gate**.
3. **Pattern/spec fail-closed and truthful contracts** (safety/flexibility, M):
   unify expression/string repetition bounds with diagnostics, implement the
   already-documented `ReplacePart` Span surface, reject non-default ignored
   `SpellingCorrection`, and reject unsupported known `Once` expiration
   policies while preserving `Pure`. The matcher portion needs the **A/B gate**;
   the builtin-local validation/Span/option portions do not touch call planning.
4. **Pattern-query operator forms** (flexibility, M): `Position[patt]`,
   `Count[patt]`, `MemberQ[patt]`, and `FreeQ[patt]`. This touches call planning
   → **mandatory A/B gate**.
5. **Folk presentation truthfulness** (convenience, S/M): make terminal value
   output use the existing transparent `HoldForm` rendering and update the
   `InputForm`/`StandardForm` summaries to name their implemented `ToString`
   consumer. Renderer/docs-only; no evaluator A/B if kept out of evaluation.

All batches preserve WL-name defaults, add no attribute kind, and make no
capability change. `Sow`, `Reap`, and `Once` remain deterministic session state
under `Capability::Pure`.

## Verification Outputs

Count reconciliation and its output are pasted verbatim at the start of this
document. The coverage check scopes itself to the five findings tables, so
trace/evidence tables cannot create a false positive.

```text
$ python3 - <<'PY'
from pathlib import Path
import re
expected = [
    'Block', 'Module', 'With', 'Function', 'Unique',
    'Hold', 'HoldComplete', 'HoldForm', 'HoldPattern', 'ReleaseHold', 'Evaluate',
    'Unevaluated', 'Once', 'Sequence', 'Nothing', 'Sow', 'Reap',
    'Repeated', 'RepeatedNull', 'Longest', 'Shortest', 'IgnoreCase', 'Overlaps',
    'Whitespace', 'NumberString', 'SpellingCorrection', 'Rule', 'RuleDelayed',
    'NormalizeRules', 'Part', 'Extract', 'ReplacePart', 'ReplaceAt', 'Position',
    'Count', 'MemberQ', 'FreeQ', 'Level', 'Thread', 'MissingQ', 'InputForm',
    'OutputForm', 'StandardForm', 'TraditionalForm', 'Short', 'Automatic', '$Failed',
]
body = Path('docs/dev/usability-pass/kernel-b-findings.md').read_text()
findings = body.split('## Findings Table', 1)[1].split('## Hold-semantics design input', 1)[0]
row_names = {
    match.group(1).strip()
    for match in re.finditer(r'^\| ([^|]+) \|', findings, re.M)
}
row_names.discard('Builtin')
missing = [name for name in expected if name not in row_names]
extra = sorted(row_names - set(expected))
if missing or extra or len(set(expected)) != 47:
    print(f'COVERAGE-FAIL missing={missing} extra={extra} expected={len(set(expected))}')
else:
    print(f'COVERAGE-OK {len(set(expected))}/{len(set(expected))}')
PY
COVERAGE-OK 47/47
```

```text
$ git status --short
?? docs/dev/usability-pass/kernel-b-findings.md
```

```text
$ scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

## Stop-Loss / Open Questions

- **Scoping STOP:** the Module error-path leak is confirmed and is above this
  audit's authority. No renaming, cleanup, or scoping proposal was made.
- The current reference says only “supported hold wrappers” and does not pin
  whether nested `ReleaseHold` removes one or all consecutive layers. The audit
  files the definite missing-`HoldForm` gap and leaves the exact nested-layer
  question open rather than guessing.
- No runtime probe misbehaved twice. The attribute parser failed once due to a
  greedy DOTALL heading regex and was corrected; the live attribute list then
  exposed and corrected the separate evaluated-`Nothing` counting trap.
