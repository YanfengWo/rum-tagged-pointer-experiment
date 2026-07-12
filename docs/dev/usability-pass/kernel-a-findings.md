# Kernel A Usability Audit Findings

## Count Reconciliation And Coverage

The authoritative subtraction reproduces exactly:

- installed reference headings: **612**
- distinct stdlib catalog names: **453**
- reference minus stdlib: **159 kernel names**
- unit 8: **55 names**, all present in the kernel set
- remaining for units 9-10: **104 names**

The catalog regex is anchored on a `BuiltinSpec::<constructor>("name", ...)`
call, and both inputs were checked for duplicates before set subtraction.

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
ref_set, std_set = set(reference), set(stdlib)
kernel = ref_set - std_set
print(f'stdlib catalog entries = {len(stdlib)}')
print(f'stdlib distinct names = {len(std_set)}')
print(f'installed reference names = {len(reference)}')
print(f'installed distinct names = {len(ref_set)}')
print(f'kernel diff = {len(kernel)}')
print(f'unit-8 names = {len(unit8)}')
print(f'unit-8 missing from kernel = {sorted(set(unit8) - kernel)}')
print(f'kernel names remaining = {len(kernel - set(unit8))}')
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
unit-8 missing from kernel = []
kernel names remaining = 104
stdlib duplicate names = 0
reference duplicate names = 0
stdlib names absent from reference = []
```

The findings-table coverage check is in Verification Outputs and prints
`COVERAGE-OK 55/55`.

## Protected Enforcement Surface (Whole-Registry)

The calibrated result is **129 protected / 483 unprotected** across all 612
installed names. The split is:

| Registry | Installed | Protected | Unprotected | Deliberately open | Should gain `Protected` |
|---|---:|---:|---:|---:|---:|
| stdlib catalog | 453 | 126 | 327 | 1 | 326 |
| kernel diff | 159 | 3 | 156 | 3 | 153 |
| **total** | **612** | **129** | **483** | **4** | **479** |

The only currently protected kernel names are `HoldForm`, `NameQ`, and
`SymbolName`. All 55 names in this unit are among the 153 kernel names that
should gain `Protected`.

### Classification

The 479 should-be-protected names are installed language heads, constants,
attribute symbols, option symbols, or result/status symbols. None is documented
as a user override point. Protection remains reversible through `Unprotect`, so
this is attribute hygiene and WL-default conformance, not immutability.

Four installed session-configuration values are deliberately open under the
current rum contract:

- `$AssertFunction` stores a user-supplied assertion callback; runtime tests
  assign it directly (`crates/rum-stdlib/src/tests/holding_and_control_part8.rs:78`).
- `$ContextPath`, `$Packages`, and `$Path` are writable session configuration.
  `Set` has explicit validated assignment branches for them before ordinary
  definition storage (`crates/rum-runtime/src/runtime/definitions/definitions.rs:198-243`).
  The context/package tests exercise those assignments
  (`crates/rum-runtime/src/runtime/tests/context_and_loader.rs:27`).

Those are writable values, not permission to redefine the surrounding builtin
surface, and open-by-default does not mean unprotectable: the three special
`Set` branches currently ignore an explicit `Protect` and are a separate
fail-closed gap (Evidence E14). If a later policy wants the attribute displayed
on them by default while keeping their setters, it needs an explicit
writable-system-variable exception contract; an attribute-only sweep must not
silently break `$AssertFunction = ...` or make the protection claim misleading.

`Protected` is a reversible mistake guard. A program can `Unprotect[Plus]` and
redefine it; this audit does not propose `Locked`, immutable definitions, or a
request rollback mode. The same registry weakness does increase the amount of
state an untrusted plugin can accidentally or intentionally disturb, but true
per-request/plugin isolation belongs to the host-embedding line. All 55 heads
correctly remain `Capability::Pure`: definitions, options, message switches,
and `$MessageList` are deterministic session state rather than outside-world
reads.

### Per-File Fix Map

This TSV is the complete machine-usable map in registration order. The fields
after each count are exact comma-separated names; `-` means empty. The 18 names
registered in both the runtime string vocabulary and stdlib are attributed to
their authoritative stdlib catalog, so every installed name appears once.

```text
source	installed	protected	deliberately_open	add_protected
crates/rum-stdlib/src/catalog/arithmetic.rs	31	12:Abs,Sign,Floor,Ceiling,IntegerPart,FractionalPart,Minus,GCD,LCM,Max,Min,MinMax	0:-	19:Identity,Plus,Times,Power,Subtract,Divide,Round,Factorial,Factorial2,Total,Mean,Median,Commonest,Quantile,Denominator,Numerator,Mod,Quotient,PowerMod
crates/rum-stdlib/src/catalog/associations.rs	37	0:-	0:-	37:Association,AssociationQ,AssociationKeyDeflatten,AssociationKeyFlatten,Lookup,Key,Keys,Values,KeyExistsQ,KeyTake,KeyDrop,KeyMap,KeyValueMap,KeySelect,KeyMemberQ,KeyFreeQ,KeyGroupBy,KeySort,KeySortBy,AssociationMap,AssociationThread,KeyUnion,KeyIntersection,KeyComplement,Normal,AssociateTo,KeyDropFrom,KeyValuePattern,Merge,MergeNested,Catenate,Counts,CountsBy,GroupBy,PositionIndex,Missing,DeleteMissing
crates/rum-stdlib/src/catalog/bytes.rs	11	11:Sha256,Sha512,Sha1,Md5,Crc32,ToHexString,FromHexString,ToBase64,FromBase64,ByteArrayQ,ByteArrayToString	0:-	0:-
crates/rum-stdlib/src/catalog/calculus.rs	1	0:-	0:-	1:D
crates/rum-stdlib/src/catalog/character_classes.rs	6	6:Alphanumeric,DigitCharacters,LowercaseLetters,PunctuationCharacters,UppercaseLetters,WhitespaceCharacters	0:-	0:-
crates/rum-stdlib/src/catalog/combinatorics.rs	11	11:Binomial,Fibonacci,PartitionsP,PartitionsQ,BernoulliB,Subfactorial,StirlingS1,StirlingS2,CatalanNumber,Multinomial,Pochhammer	0:-	0:-
crates/rum-stdlib/src/catalog/control.rs	35	2:Product,Sum	1:$AssertFunction	32:CompoundExpression,If,Which,Switch,Do,While,Until,For,Table,Abort,Exit,CheckAbort,AbortProtect,WithCleanup,Failsafe,Assert,PropagateAborts,Enclose,Confirm,ConfirmBy,ConfirmMatch,ConfirmQuiet,ConfirmAssert,Return,Break,Continue,Throw,Catch,Failure,FailureQ,Success,TerminatedEvaluation
crates/rum-stdlib/src/catalog/csv.rs	2	2:FromCsv,ToCsv	0:-	0:-
crates/rum-stdlib/src/catalog/datetime.rs	9	9:DateList,DateString,FromDateList,FromDateString,DatePlus,DateDifference,DayOfWeek,LeapYearQ,DaysInMonth	0:-	0:-
crates/rum-stdlib/src/catalog/functional.rs	42	0:-	0:-	42:BlockMap,SubsetMap,Comap,ComapApply,Accumulate,MapIndexed,Map,MapAll,MapAt,Scan,MapThread,TakeWhile,LengthWhile,Inner,Dot,Fold,FoldList,FoldPair,FoldWhile,SequenceFold,SequenceFoldList,Nest,NestWhile,NestList,NestWhileList,FixedPoint,FixedPointList,Apply,MapApply,OperatorApplied,CurryApplied,ReverseApplied,Composition,RightComposition,Through,Operate,Array,ArrayFilter,ArrayReduce,Outer,Select,SelectFirst
crates/rum-stdlib/src/catalog/ini.rs	2	2:FromIni,ToIni	0:-	0:-
crates/rum-stdlib/src/catalog/integer_digits.rs	5	5:IntegerLength,IntegerDigits,IntegerReverse,FromDigits,DigitCount	0:-	0:-
crates/rum-stdlib/src/catalog/json.rs	2	2:FromJson,ToJson	0:-	0:-
crates/rum-stdlib/src/catalog/logic.rs	21	0:-	0:-	21:And,Or,Nand,Nor,Xor,Equivalent,Implies,Not,Equal,Unequal,Less,Greater,LessEqual,GreaterEqual,LessThan,GreaterThan,LessEqualThan,GreaterEqualThan,EqualTo,UnequalTo,SameAs
crates/rum-stdlib/src/catalog/number_theory.rs	9	9:Divisors,EulerPhi,MoebiusMu,DivisorSigma,Prime,PrimePi,PrimeNu,PrimeOmega,NextPrime	0:-	0:-
crates/rum-stdlib/src/catalog/polynomials.rs	5	0:-	0:-	5:Expand,Coefficient,CoefficientList,Together,Factor
crates/rum-stdlib/src/catalog/predicates.rs	50	0:-	0:-	50:TrueQ,BooleanQ,Boole,AtomQ,ListQ,StringQ,IntegerQ,NumberQ,NumericQ,EvenQ,OddQ,Positive,Negative,NonPositive,NonNegative,PossibleZeroQ,MachineNumberQ,RealValuedNumberQ,RealValuedNumericQ,PrimeQ,AlgebraicIntegerQ,PolynomialQ,PolynomialExpressionQ,RationalExpressionQ,DateObjectQ,TimeObjectQ,DuplicateFreeQ,IntersectingQ,DisjointQ,SubsetQ,Order,OrderedQ,LexicographicOrder,NumericalOrder,AlphabeticOrder,ContainsAll,ContainsAny,ContainsExactly,ContainsNone,ContainsOnly,Between,VectorQ,MatrixQ,ArrayQ,AllTrue,AnyTrue,NoneTrue,CoprimeQ,SameQ,UnsameQ
crates/rum-stdlib/src/catalog/random.rs	11	11:SeedRandom,RandomInteger,RandomChoice,RandomSample,RandomString,BlockRandom,RandomUuid,UuidQ,RandomBytes,ReseedRandom,RandomPassword	0:-	0:-
crates/rum-stdlib/src/catalog/rewrite.rs	7	0:-	0:-	7:MatchQ,Cases,DeleteCases,Replace,ReplaceList,ReplaceAll,ReplaceRepeated
crates/rum-stdlib/src/catalog/schema.rs	2	2:ConformSchema,SchemaErrors	0:-	0:-
crates/rum-stdlib/src/catalog/strings.rs	97	25:Alternatives,Blank,BlankNullSequence,BlankSequence,DigitCharacter,EndOfLine,EndOfString,Except,LetterCharacter,Pattern,PunctuationCharacter,RegularExpression,StartOfLine,StartOfString,StringExpression,WhitespaceCharacter,WordBoundary,WordCharacter,StringReverse,ToLowerCase,ToUpperCase,Capitalize,Decapitalize,LowerCaseQ,UpperCaseQ	0:-	72:StringJoin,StringLength,StringForm,StringWrap,StringIndent,StringDedent,StringExpandTabs,AlignColumns,DecimalString,HtmlEscape,HtmlUnescape,ShellSplit,ShellQuote,FileExtension,FileBaseName,FileNameSplit,UrlEncode,UrlDecode,UrlParse,ParseArgs,Row,StringRiffle,StringPadLeft,StringPadRight,StringLines,StringTemplate,TemplateApply,TextWords,ToCharacterCode,ToExpression,StringToByteArray,Hash,Compress,Uncompress,Nearest,Pluralize,SyntaxQ,RumTokens,TokenizeWith,TextString,CharacterCounts,RemoveDiacritics,Alphabet,AlphabeticSort,EditDistance,HammingDistance,LongestCommonSubsequence,SequenceAlignment,Diff,Characters,CharacterRange,ToString,StringRepeat,LetterQ,DigitQ,StringContainsQ,StringStartsQ,StringEndsQ,StringPosition,StringSplit,StringTrim,StringReplace,StringCases,StringApply,StringDelete,StringTake,StringDrop,StringTakeDrop,StringPart,StringCount,StringMatchQ,StringFreeQ
crates/rum-stdlib/src/catalog/structure.rs	50	10:LeafCount,Differences,ConstantArray,PadLeft,PadRight,Subsets,Tuples,Partition,Union,Intersection	0:-	40:FullForm,Head,Length,Depth,First,Last,Rest,Most,Take,Drop,TakeDrop,Insert,Delete,TakeList,Range,Flatten,Join,Reverse,Transpose,DeleteDuplicates,DeleteDuplicatesBy,Duplicates,Tally,GatherBy,Sort,SortBy,BinarySearch,ReverseSort,ReverseSortBy,MaxBy,MinBy,TakeLargestBy,TakeSmallestBy,Riffle,SelectPositions,Complement,Append,Prepend,Symbol,Construct
crates/rum-stdlib/src/catalog/toml.rs	2	2:FromToml,ToToml	0:-	0:-
crates/rum-stdlib/src/catalog/xml.rs	3	3:FromXml,ToXml,XmlElement	0:-	0:-
crates/rum-stdlib/src/catalog/yaml.rs	2	2:FromYaml,ToYaml	0:-	0:-
crates/rum-runtime/src/runtime/builtin_catalog/associations.rs	1	0:-	0:-	1:MissingQ
crates/rum-runtime/src/runtime/builtin_catalog/debug.rs	26	0:-	0:-	26:Trace,TracePrint,Timing,AbsoluteTime,AbsoluteTiming,RepeatedTiming,Short,Echo,EchoFunction,EchoLabel,ByteCount,On,Off,Sow,Reap,TimeConstrained,MemoryConstrained,MemoryInUse,MaxMemoryUsed,Monitor,EvaluationData,RumProfileReset,RumProfileSnapshot,TestCreate,TestReport,Stack
crates/rum-runtime/src/runtime/builtin_catalog/functional.rs	2	1:HoldForm	0:-	1:Function
crates/rum-runtime/src/runtime/builtin_catalog/namespace.rs	63	2:SymbolName,NameQ	3:$ContextPath,$Packages,$Path	58:$Context,$Aborted,$Canceled,$Failed,$MessageList,$InputFileName,Nothing,FileNameJoin,DirectoryName,ParentDirectory,FileNameTake,Context,Contexts,Names,ResolveContextAliases,SpellingCorrection,File,FindFile,Get,Needs,Begin,End,BeginPackage,EndPackage,DeclarePackage,Save,DumpSave,Once,Message,MessageName,Messages,MessageDefinitions,MessageTags,Information,Definition,ValueQ,DefaultValues,OwnValues,UpValues,DownValues,SubValues,Attributes,SyntaxInformation,Options,OptionValue,ArgumentsOptions,CheckArguments,SetAttributes,ClearAttributes,SetOptions,Protect,Unprotect,Unset,Clear,ClearAll,Remove,Check,Quiet
crates/rum-runtime/src/runtime/builtin_catalog/rules.rs	30	0:-	0:-	30:Part,Extract,ReplacePart,ReplaceAt,Set,AddTo,SubtractFrom,TimesBy,DivideBy,ApplyTo,AppendTo,PrependTo,Increment,Decrement,PreIncrement,PreDecrement,SetDelayed,UpSet,UpSetDelayed,TagSet,TagSetDelayed,TagUnset,Rule,RuleDelayed,Count,MemberQ,FreeQ,Level,Position,NormalizeRules
crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs	21	0:-	0:-	21:With,Module,Unique,Block,Hold,HoldFirst,HoldRest,HoldAll,HoldAllComplete,HoldComplete,Unevaluated,Evaluate,HoldPattern,ReleaseHold,Sequence,Thread,Dialog,InputString,Input,Print,Pause
crates/rum-runtime/src/runtime/builtin_catalog/strings.rs	16	0:-	0:-	16:Whitespace,NumberString,IgnoreCase,Overlaps,Repeated,RepeatedNull,Shortest,Longest,TemplateObject,DiffObject,ByteArray,InputForm,OutputForm,StandardForm,TraditionalForm,Automatic
```

### Registration And Mirror Mechanism

`BuiltinSpec::new`, `special`, and `ordinary` hard-set
`AttributeSet::empty()` (`crates/rum-runtime/src/runtime/metadata.rs:297`,
`:334`, `:377`). Among the 483 unprotected entries, 350 use one of those
empty constructors. Another 100 use a hold constructor, which preserves a
hold attribute but has no `Protected`; the remaining 33 already pass a general
attribute set and need that set extended. The exact constructor split is:

```text
stdlib: new=2 ordinary=259 special=7 ordinary_with_hold=30 ordinary_with_attributes=29
kernel: new=29 ordinary=39 special=14 special_with_hold=2 ordinary_with_hold=68 ordinary_with_attributes=4
```

Do not change the global constructor defaults: downstream/plugin specs and the
four deliberate-open entries need an explicit choice. Change catalog entries
or file-local attribute constants instead. For stdlib entries, the same
attribute must also move through
`crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`; the
runtime uses that mirror when stdlib handlers are not registered
(`crates/rum-runtime/src/runtime/builtin_catalog.rs:95`).
`catalog_attributes_match_kernel_vocabulary`
(`crates/rum-stdlib/src/tests.rs:178`) checks catalog/mirror lockstep, and the
builtin-reference drift test (`crates/rum-stdlib/src/tests/builtin_reference_docgen.rs:5`)
requires regenerating `docs/reference/builtins.md` in the eventual fix.

The actual `Attribute` enum has 15 kinds: `HoldAllComplete`, `HoldAll`,
`HoldFirst`, `HoldRest`, `SequenceHold`, `Listable`, `NHoldAll`,
`NumericFunction`, `Orderless`, `Flat`, `OneIdentity`, `NonThreadable`,
`Protected`, `ReadProtected`, and `Temporary`
(`crates/rum-runtime/src/runtime/metadata.rs:32`). No new attribute kind is
proposed by this audit.

## Top Gaps By Script-User Value

1. **Safety:** only 129/612 installed names are protected; after four explicit
   writable-session exceptions, 479 heads/symbols should gain `Protected`.
2. **Safety:** three paths mutate an explicitly protected symbol with no
   `wrsym`-class diagnostic — (a) `ApplyTo[x, f]` where `f` protects `x` during
   the update, because the shared `store_held_update` does not recheck before
   writing (`x = 101`, `Attributes[x] = {Protected}`); (b) the special `Set`
   setters for `$ContextPath`/`$Packages`/`$Path` (E14); (c) `SetAttributes`
   (E5: `Protect[s]` then `SetAttributes[s, Listable]` succeeds → `{Listable,
   Protected}`). All three gate-reproduced. (`SetOptions` was claimed too but is
   NOT verified — the gate probe was deflected by an `optnf` unknown-option
   error; confirm during the fix.) The arithmetic/list compound assignments and
   the association-lvalue `Set` path are fail-closed (their update runs no user
   code in the window) — see the compound-assignment GATE block.
3. **Convenience/safety:** assigning an inspection table such as
   `DownValues[g] = DownValues[f]` does not install definitions on `g`; it
   silently defines the inspector head and creates a convincing false round-trip.
4. **Safety/flexibility:** wildcard strings are exact literal names;
   ``Clear["Global`*"]`` can silently leave all matching definitions intact,
   while ``Protect["Global`*"]`` protects a symbol literally named `*`.
5. **Convenience:** `Information[x]` evaluates a defined `x` before inspection;
   the string workaround works, but builtin usage falls back to only
   ``"System`Name"`` and emitted builtin message tags are absent.
6. **Discoverability:** `Messages[Set]`, `MessageTags[Set]`, and
   `MessageDefinitions[Set]` are empty even immediately after `Set::argx`.
7. **Flexibility:** `Attributes[{a,b}]` and `Options[f, name]` are missing
   documented data forms; `Off[]`/`On[]` do not provide the all-message switch.

Operator-form disambiguation produces no finding in this unit. Unary and
low-arity calls are meaningful data, mutation, inspection, or selector forms;
none is an unambiguous missing curried operator. The list, wildcard, lookup,
and selector gaps below are data forms, not operator forms.

## Findings Table

### Assignment / Definition

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Set | WL | gap | safety | The definition-write guard works, but the `Set` head itself has no `Protected`; a user can redefine the language's assignment head (Protected surface; Evidence E1-E3). | Add `Protected` to the catalog entry. This is mutation-path-only metadata and needs no microbench A/B. | S |
| Set | WL | gap | safety | One side path bypasses protection: the dedicated setters for `$ContextPath`, `$Packages`, and `$Path` return from `finish_definition_assignment` BEFORE the ordinary guard is reached, so they mutate a symbol whose attribute is visibly `Protected` with no message (Evidence E14, gate-reproduced; `definitions/definitions.rs:198-255`). | Keep these configuration values open by DEFAULT, but make their special setters honor an explicit `Protect`. | S |
| Set | WL | OK | safety | The association-lvalue path is fail-closed, contrary to the submitted draft: `a = <|"x" -> 1|>; Protect[a]; a["x"] = 99` emits `Set::wrsym` and leaves `a` unchanged (gate-reproduced; the draft's "Evidence E16" does not exist). | No change. | |
| Set | WL | OK | boundary | `Set` is a special form: lowering preserves raw lhs/rhs, dispatch sends raw arguments directly, and `eval_set` evaluates only the rhs before storage. Its missing `HoldFirst`/`SequenceHold` metadata is not a behavior hole; adding either merely because WL lists it would duplicate the hard-coded path and would touch attribute-driven planning (Hard-Coded-Hold Trace). | Keep the special-form hold design; do not add non-`Protected` attributes without a separate semantic justification and A/B gate. | |
| SetDelayed | WL | gap | safety | Its guarded storage rejects a protected target with `SetDelayed::wrsym`, but the `SetDelayed` head is unprotected (Protected surface; Evidence E2-E3). | Add `Protected`; preserve special-form/raw-rhs behavior. | S |
| TagSet | WL | gap | safety | Tagged immediate writes honor protection, but the mutation head itself is redefinable (Evidence E2-E3; `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:28`). | Add `Protected`; preserve the current tag-position validation. | S |
| TagSetDelayed | WL | gap | safety | Tagged delayed writes honor protection, but the mutation head itself is redefinable (Evidence E2-E3; `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:52`). | Add `Protected`; preserve raw delayed rhs semantics. | S |
| TagUnset | WL | gap | safety | Removal from a protected tag is rejected with `TagUnset::wrsym`, but `TagUnset` itself is redefinable (Evidence E2-E3; `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:75`). | Add `Protected`. | S |
| UpSet | WL | gap | safety | Every inferred target is checked before storing; protected targets remain unchanged, but `UpSet` itself is unprotected (Evidence E2-E3; `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:129`). | Add `Protected`; retain per-target fail-closed behavior. | S |
| UpSetDelayed | WL | gap | safety | Delayed inferred upvalue writes honor protection, but the head is unprotected (Evidence E2-E3). | Add `Protected`; preserve special-form/raw-rhs behavior. | S |

### Compound Assignment

All 11 forms have unambiguous mutation/data arities, not operator-form gaps.
Their shared target normalization rejects a target that is already protected
(`definition_updates.rs:244-304`), and the batched probe shows all 11 leave such
values unchanged (Evidence E4).

> **GATE — CORRECTED TWICE (Claude, 2026-07-12).** The submitted draft claimed
> a shared "check-to-store window": the guard runs on the initial target, the
> update is evaluated afterwards (`finish_held_update`, `definition_updates.rs:361-375`),
> and `store_held_update` (`:377-418`) writes without rechecking — citing
> "Evidence E15/E16", which the truncated run never wrote into the file.
>
> My first gate pass judged this "false, final store DOES recheck", from a probe
> (`AddTo[t, g[0]]` with `g` protecting `t`) that returned `AddTo::wrsym`. **That
> pass was wrong** — the probe could not reach the window (the second argument is
> not held, so `g[0]` evaluates and `Protect[t]` fires BEFORE the guard). Reading
> the code settles it: the window is real. `finish_held_update` evaluates `update`
> and only then calls `store_held_update`, which stores with no protection
> recheck.
>
> **The window IS reachable — via `ApplyTo`**, the one compound assignment whose
> update runs user code. `ApplyTo[x, f]` builds `update = f[x]` (`:102`), so `f`'s
> body executes inside the window. Gate-reproduced:
>
> ```
> $ printf '' | cargo run -q -p rum-cli -- --eval 'ClearAll[x, f]; x = 1; f = Function[v, (Protect[x]; v + 100)]; ApplyTo[x, f]; Print[{x, Attributes[x]}]'
> {101, {Protected}}
> ```
>
> The write landed (`x = 101`) on a visibly `Protected` symbol, with **no
> message**. This is a genuine safety bug — the fix is a recheck in
> `store_held_update` before the write (queued as triage batch 1a below).
>
> **But the draft over-generalized it to all 11 forms; that half is wrong.** The
> other ten build `update = Plus[current, arg]` / `Append[current, arg]` (`:41`,
> `:59`, `:117`) where `arg` is already evaluated, so `evaluate(update)` runs no
> user code and the window is not reachable: `AddTo[t, (Protect[t]; 5)]` returns
> `AddTo::wrsym` and leaves `t` unchanged. So the corrected finding is narrow and
> exact: **`store_held_update` lacks a recheck (root cause, all forms share it),
> but only `ApplyTo` can actually trigger it today.** The rows below reflect that.

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| AddTo | WL | gap | safety | The head lacks `Protected`. The `store_held_update` recheck gap (see the block above) is SHARED but not reachable through `AddTo`: its update is `Plus[current, arg]` with `arg` already evaluated, so `AddTo[t, (Protect[t]; 5)]` fails closed with `AddTo::wrsym` (gate-verified) (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst`. Mutation-path-only metadata; no A/B gate. The store recheck lands once (batch 1a) and covers all forms. | S |
| SubtractFrom | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst`. | S |
| TimesBy | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst`. | S |
| DivideBy | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst`. | S |
| Increment | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst` and postfix return semantics. | S |
| Decrement | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst`. | S |
| PreIncrement | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst` and prefix return semantics. | S |
| PreDecrement | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst`. | S |
| AppendTo | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed, including exact `Part` owners and updates that protect mid-evaluation (gate-verified) (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst`. | S |
| PrependTo | WL | gap | safety | The head itself lacks `Protected`; the mutation guard is fail-closed (Evidence E2-E4). | Add `Protected`, preserve `HoldFirst`. | S |
| ApplyTo | rum | gap | safety | **REAL PROTECTION-BYPASS (gate-reproduced).** The rum-native head lacks `Protected`, AND it is the one compound assignment that reaches the shared `store_held_update` recheck gap: its update is `f[current]` (`definition_updates.rs:102`), so a function that protects the target mid-update still lands the write. `x = 1; f = Function[v, (Protect[x]; v+100)]; ApplyTo[x, f]` → `x = 101` with `Attributes[x] = {Protected}`, no message. | Add `Protected` (S). Separately, recheck the target in `store_held_update` before the write (batch 1a, M) — this is the load-bearing fix; the attribute alone does not close it. | M |

### Value Tables / Introspection / Removal

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| OwnValues | WL | gap | safety | The inspector head is unprotected (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| OwnValues | WL | gap | convenience | `OwnValues[b] = OwnValues[a]` defines a downvalue on `OwnValues`, makes later inspection look copied, but leaves `b` unassigned (Evidence E7; `definition_analysis_helpers.rs:183-201`). | Add real value-table assignment normalization or fail closed; do not let a setter-looking call redefine the inspector head. A setter path through `Set` is hot-path work and needs the rum-vs-rum A/B gate. | M |
| DownValues | WL | gap | safety | The inspector head is unprotected (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| DownValues | WL | gap | convenience | `DownValues[g] = DownValues[f]` returns the copied-looking rules from `DownValues[g]`, but `g[2]` remains inert because the assignment attached to `DownValues` itself (Evidence E7). | Support round-trippable table assignment with validation and atomic replacement, or reject it explicitly. Any `Set`-path change needs an A/B gate. | M |
| UpValues | WL | gap | safety | The inspector head is unprotected (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| UpValues | WL | gap | convenience | Assigning `UpValues[v]` copies display data only by redefining `UpValues`; it does not attach an upvalue to `v` (Evidence E7). | Add validated upvalue-table assignment or fail closed; never preserve the false-success shape. A `Set`-path change needs A/B. | M |
| SubValues | WL | gap | safety | The inspector head is unprotected (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| SubValues | WL | gap | convenience | Assigning `SubValues[t]` redefines the inspector and does not make `t[1][2]` callable (Evidence E7). | Add validated subvalue-table assignment or fail closed; a `Set`-path change needs A/B. | M |
| SubValues | WL | OK | boundary | The inspector itself has `HoldFirst`. Actual subvalue application is different: nested heads have no inherited base-symbol attributes, so both captured inner arguments and outer application arguments evaluate before matching; the probe yields `HoldComplete[1,2]` and counter `2` (Evidence E9; `eval_call_prepare.rs:426`; `definitions/application/sub_values.rs:8`). | Record only as input to the parked `SubValuesHoldAll`-equivalent design; do not design or add an attribute in this unit. | |
| DefaultValues | WL | gap | safety | The inspector head is unprotected (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| DefaultValues | WL | gap | convenience | `DefaultValues[g] = DefaultValues[f]` creates a definition on `DefaultValues`; `Default[g]` stays inert (Evidence E7). | Add a validated default-table setter or fail closed. A `Set`-path change needs A/B. | M |
| ValueQ | WL | gap | safety | `ValueQ` itself is unprotected (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| ValueQ | WL | OK | safety | It accepts non-symbol expressions and evaluates against a full rollback snapshot: `ValueQ[1]` is `False`, `ValueQ[1+1]` and `ValueQ[x=x+1]` are `True`, while `x` remains `0` (Evidence E9; `builtins_messages_inspect.rs:32`; `session_helpers.rs:92`). | No change; rollback rather than resumable continuation is the documented boundary. | |
| Definition | WL | gap | safety | The introspector head is unprotected (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| Definition | WL | OK | convenience | The held first-pass output includes attributes, stored messages, options, own/default definitions, and syntax information, without attempting a full front-end pretty-printer (Evidence E8; `builtins_messages_inspect.rs:152-201`). | Keep the honest held-expression format; no formatting arms race. | |
| Information | WL | gap | safety | The introspector head is unprotected (Protected surface; Evidence E2). | Add `Protected`. | S |
| Information | WL | gap | convenience | `Information[x]` first evaluates defined `x` and becomes inert `Information[5]`; `Information["x"]` works. For builtins, `Usage` falls back to only ``"System`Set"``, and the data has no message-tags field (Evidence E8; `builtins_messages_inspect.rs:16-29`, `:245-293`). | Make direct-symbol inspection hold its target and use the builtin catalog summary as the usage fallback; add compact message-tag discovery, not full WL formatting. A hold-attribute/call-planning change requires the microbench A/B gate. | M |
| Clear | WL | gap | safety | Protected exact targets emit `Clear::wrsym`, but the `Clear` head itself is unprotected (Evidence E2, E5). | Add `Protected` while preserving `HoldAll`. | S |
| Clear | WL | gap | flexibility | Exact symbols, strings, and nested lists work, but ``Clear["Global`*"]`` treats `*` literally and silently leaves matching definitions intact (Evidence E6; `definition_analysis_helpers.rs:117-134`). | Add the documented name-pattern form using the existing namespace matcher; enumerate matches first and apply protection checks per target. | M |
| ClearAll | WL | gap | safety | It correctly refuses a protected target before clearing definitions, messages, options, syntax information, defaults, or attributes, but its own head is unprotected (Evidence E2, E5; `definition_clear.rs:23-25`). | Add `Protected` while preserving `HoldAll` and the current all-state clear boundary. | S |
| Remove | WL | gap | safety | Exact protected targets emit `Remove::rmptc` and retain state, but the `Remove` head is unprotected (Evidence E2, E5; `definition_clear.rs:34-52`). | Add `Protected` while preserving `HoldAll`. | S |
| Remove | WL | gap | flexibility | ``Remove["Global`*"]`` is the same silent literal-name no-op as `Clear`; exact removal otherwise clears all current state, marks same-evaluation identities stale, and permits a fresh later identity (Evidence E6; `incidental_definitions_and_rules_lifecycle.rs:64-197`). | Add documented name-pattern expansion with a precomputed target set and per-symbol protection checks; do not allow a silent kernel-wide wipe. | M |
| Unset | WL | gap | safety | Specific definition/message/options/syntax-information removal checks its owner and emits `Unset::wrsym` for protected targets, but `Unset` itself is unprotected (Evidence E2, E5; `definition_clear.rs:10-24`). | Add `Protected` while preserving `HoldFirst`. | S |

### Attributes / Protection

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Attributes | WL | gap | safety | The attribute inspector itself lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| Attributes | WL | gap | flexibility | `Attributes[{a,b}]` emits `Attributes::argform`, although list targets already work for the adjacent mutation forms (Evidence E6; `attributes.rs:28-43`). | Add the documented list query and return one attribute list per input symbol; this is builtin-local, not an operator form. | S |
| SetAttributes | WL | gap | safety | The head lacks `Protected`, and more seriously it changes a protected target without checking protection: after `Protect[Plus]`, `SetAttributes[Plus,HoldAll]` succeeds with no message (Evidence E5; `attributes.rs:104-138`). List targets themselves already work. | Add `Protected` to the head and enforce `wrsym` before non-`Protected` attribute changes. Keep explicit addition of `Protected` valid. No call-planning change is needed for the guard itself. | S |
| ClearAttributes | WL | gap | safety | The head lacks `Protected`; `ClearAttributes[p,HoldAll]` removes a non-protection attribute while `p` remains protected, with no write message (Evidence E5; `attributes.rs:104-138`). | Add `Protected` to the head and guard changes to other attributes on protected targets. Preserve the explicit `ClearAttributes[p,Protected]`/`Unprotect[p]` escape rather than creating immutability. | S |
| Protect | WL | gap | safety | The protection mechanism itself has only `HoldAll`; because the head is unprotected, `Unprotect[q_] := 7`-style redefinition also applies to `Protect` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll` and reversible unprotection. | S |
| Protect | WL | gap | flexibility | Multiple symbols and nested lists work, but a string pattern is treated as one exact symbol: ``Protect["Global`*"]`` returns `{"*"}` and protects the literal `*`. The returned short names also lose context and include already-unchanged targets (`attributes.rs:140-168`; Evidence E6). | Add documented name-pattern expansion and return an unambiguous list of names actually selected/changed, preserving the WL default return shape after confirming its exact symbol/string representation. | M |
| Unprotect | WL | gap | safety | The head that removes protection is itself unprotected and can be redefined without first opting out (Evidence E2). | Add `Protected` while preserving `HoldAll`; users can still explicitly `Unprotect[Unprotect]`. | S |
| Unprotect | WL | gap | flexibility | Lists and exact name strings work, but string patterns do not expand and the short-name return is context-ambiguous (Evidence E6; `attributes.rs:140-168`). | Share the safe pattern-target normalization and qualified/contract-compatible return path with `Protect`. | M |
| HoldAll | WL | gap | safety | The attribute symbol/head has `HoldAll` but lacks `Protected`, so the language's hold vocabulary is redefinable (Protected surface; Evidence E1-E2). | Add `Protected` while preserving `HoldAll`; this does not add a new attribute kind. | S |
| HoldFirst | WL | gap | safety | The attribute symbol/head has `HoldFirst` but lacks `Protected` (Protected surface; Evidence E1-E2). | Add `Protected` while preserving `HoldFirst`. | S |
| HoldRest | WL | gap | safety | The attribute symbol/head has `HoldRest` but lacks `Protected` (Protected surface; Evidence E1-E2). | Add `Protected` while preserving `HoldRest`. | S |
| HoldAllComplete | WL | gap | safety | The attribute symbol/head has `HoldAllComplete` but lacks `Protected` (Protected surface; Evidence E1-E2). | Extend its existing general attribute set with `Protected`; preserve `HoldAllComplete`. | S |
| SyntaxInformation | WL name; rum-specific shape | gap | safety | The metadata inspector/setter head lacks `Protected`; its assignment path does correctly check the target owner (Protected surface; `definitions.rs:276-291`). | Add `Protected` while preserving `HoldFirst` and the rum-specific rule-list shape. | S |
| SyntaxInformation | WL name; rum-specific shape | OK | convenience | Read, immediate assignment, `Unset`, `ClearAll`, `Block` localization, `Definition`, and save/restore integration are covered in the current subset (`syntax_information_lifecycle.rs:5-158`; Evidence E8). | No shape expansion in this audit. | |

### Options

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Options | WL | gap | safety | The options inspector/setter head is unprotected (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`; the existing `Options[f] = rules` target normalization already checks protection on `f`. | S |
| Options | WL | gap | flexibility | Only the one-argument listing form exists; `Options[f,a]` remains inert and emits repeated arity messages at fixed-point evaluation (Evidence E10; `options/builtins.rs:10-24`). | Add the documented single-option lookup `Options[f,name]`, returning the matching rule(s) without changing `Options[f]`. | S |
| SetOptions | WL | gap | safety | The head lacks `Protected`, and `SetOptions` never checks the target: it changes protected `RandomPassword`'s `IgnoreCase` default to `True` with no message (Evidence E5; `options/builtins.rs:153-218`). | Add `Protected` to the head and a per-target `wrsym` guard before validating/applying updates. Keep list-target handling atomic per target. | S |
| SetOptions | WL | OK | flexibility | Its return is already useful and WL-shaped for the supported subset: it returns the complete updated option-rule list, and a list target preserves a list-of-results shape (`options/builtins.rs:171-183`; Evidence E10). | Preserve the return while adding protection enforcement. | |
| OptionValue | WL | gap | safety | The option lookup head lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`. | S |
| OptionValue | WL | OK | flexibility | Scoped one-argument lookup, two-argument symbol lookup, explicit three-argument rules, and list-valued name queries are implemented; unknown names message rather than fabricate a value (`options/builtins.rs:69-149`, `:241-338`; Evidence E10). | No new call form needed. | |
| ArgumentsOptions | rum | gap | safety | The rum-native held argument/option partitioner lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| ArgumentsOptions | rum | OK | flexibility | It accepts exact or `{min,max}` positional counts, validates known trailing options, returns `{positional,options}`, and produces structured `Failure` data on bad counts/options (`options/arguments.rs:257-312`). | No change. | |

### Messages

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Message | WL | gap | safety | The emitter head lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| Message | WL | OK | flexibility | It accepts a canonical `MessageName` plus template arguments, evaluates delayed templates, and records structured runtime messages (Evidence E11; `builtins_messages.rs:4-29`). | No new selector or operator form. | |
| Messages | WL | gap | safety | The message-name inspector lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`. | S |
| Messages | WL | gap | convenience | It only reports stored user templates. Immediately after `Set[]` emits `Set::argx`, `Messages[Set]` is `{}`; builtin diagnostics are therefore undiscoverable through this surface (Evidence E12; `message_catalog.rs:72-91`, `builtins_messages.rs:50-75`). | Add a compact builtin diagnostic-tag catalog or merge registered builtin tags into inspection; do not build a full front-end message formatter. | M |
| MessageName | WL | gap | safety | The symbolic message-name head lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`. | S |
| MessageName | WL | OK | flexibility | Surface `f::tag`, direct symbol/string arguments, and string/symbol tags normalize to one held canonical form; the selector parser accepts that form (Evidence E11; `message_selector_parsing.rs:42-104`). | No change. | |
| MessageTags | rum | gap | safety | The rum-native tag inspector lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`. | S |
| MessageTags | rum | gap | convenience | It reports stored user-template tags but not tags for emitted builtin diagnostics; `MessageTags[Set]` is empty after `Set::argx` (Evidence E12). | Share the compact builtin-tag discovery source proposed for `Messages`. | M |
| MessageDefinitions | rum | gap | safety | The rum-native template inspector lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`. | S |
| MessageDefinitions | rum | gap | convenience | It round-trips stored user templates, but there is no discoverable definition/description for builtin diagnostics (Evidence E11-E12). | Merge builtin diagnostic metadata where available; an honest tag plus concise template/description is sufficient. | M |
| $MessageList | WL | gap | safety | The session message-list symbol lacks `Protected`, although it is a read-only computed value (Protected surface; Evidence E2). | Add `Protected`; keep it `Capability::Pure`. | S |
| $MessageList | WL | OK | boundary | It reflects visible messages emitted so far in the current top-level input, excludes `Quiet`-suppressed messages, and is replaced at top-level completion (`session_eval.rs:177-218`; `tests/messages.rs:332-376`; Evidence E11). `reset()` clears messages; text snapshots intentionally exclude messages and `$` variables. | No capability or snapshot expansion. | |
| Quiet | WL | gap | safety | The message-suppression head lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`. | S |
| Quiet | WL | OK | flexibility | `Quiet[expr]` and `Quiet[expr,{f::tag,...}]` work; malformed selectors fail closed, and suppressed messages do not trigger `Check` (Evidence E11; `check_quiet.rs:51-161`). | No change. | |
| Check | WL | gap | safety | The fallback/control head lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`. | S |
| Check | WL | OK | flexibility | `Check[expr,fail]` and selective `Check[expr,fail,{tags}]` work, including recoverable evaluator errors; fallback evaluation stays held until selected (Evidence E11; `check_quiet.rs:4-49`). | No change. | |
| CheckArguments | rum | gap | safety | The rum-native validator head lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldFirst`. | S |
| CheckArguments | rum | OK | flexibility | It shares exact/range count and option-suffix validation with `ArgumentsOptions`, returns `True`/`False`, and emits the target head's diagnostic (`options/arguments.rs:314-359`). | No change. | |
| Off | WL | gap | safety | The persistent message-switch head lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`; keep the state session-local and `Pure`. | S |
| Off | WL | gap | flexibility | `Off[f::tag]`, symbol selectors, strings, lists, and multiple selectors work, but `Off[]` emits `Off::argx` instead of the documented all-message switch (Evidence E11; `message_switch.rs:26-71`). | Add the no-argument all-message form without changing selector defaults. | S |
| On | WL | gap | safety | The persistent message-switch head lacks `Protected` (Protected surface; Evidence E2). | Add `Protected` while preserving `HoldAll`; keep the state session-local and `Pure`. | S |
| On | WL | gap | flexibility | `On[f::tag]` and the current selector forms work, but `On[]` cannot restore all-message emission after a global switch (Evidence E11; `message_switch.rs:4-24`). | Add the paired no-argument all-message form. | S |

## Hard-Coded-Hold Trace For `Set`

The hold behavior does not come from `Attributes[Set]`:

1. Syntax lowering turns `lhs = rhs` into the canonical call `Set[lhs,rhs]`
   without evaluation (`crates/rum-runtime/src/runtime/lowering.rs:286-312`).
2. The catalog registers `Set` with `BuiltinSpec::special`, whose constructor
   stores `AttributeSet::empty()` (`builtin_catalog/rules.rs:24-28`;
   `metadata.rs:334-342`).
3. `build_call_dispatch_plan` gives every special form a one-stage plan and
   `SpecialForm` dispatch passes `state.raw_args` directly, bypassing
   `PrepareArguments` and all attribute-driven argument planning
   (`eval_dispatch/mod.rs:4-13`, `:61-79`).
4. `eval_set` clones the raw lhs, explicitly evaluates only the rhs, then sends
   both to definition assignment (`definitions/definitions.rs:14-51`).
   `SetDelayed` clones both raw arguments and stores the rhs delayed (`:54-65`).

Therefore adding `HoldFirst` to `Set` would be a duplicate/no-op for direct
`Set` calls, apart from changing introspection metadata. `SequenceHold` would
likewise duplicate the fact that the generic sequence-preparation stage never
runs. This audit proposes neither: a blind WL attribute transcription has no
independent usability benefit, and any non-`Protected` attribute addition would
touch call planning and require the microbench A/B gate. Adding only
`Protected` is independently justified and has no default-call planning cost.

## Evidence Appendix

Twenty bounded CLI invocations were used, below the ~40 probe budget. Every
CLI probe fed empty stdin. No external engine or network was used.

> **GATE RECORD (Claude, 2026-07-12).** This appendix ends at **E14**. The
> findings table cited "Evidence E15/E16" in twelve rows; those sections were
> never written into the file — the run was truncated at the end (the report
> cut off mid-sentence, and the tail of the dispatch log shows an upstream
> `cybersecurity risk` content filter firing). The most likely reading is that
> E15/E16 described a protection-bypass and the write was cut off there, leaving
> the table's forward-references dangling. **The claim they backed is
> substantially TRUE.**
>
> My first gate pass wrongly judged it false, using a probe that could not reach
> the code window. Reading `definition_updates.rs` settles it: `finish_held_update`
> (`:361-375`) evaluates the update and then `store_held_update` (`:377-418`)
> writes with no protection recheck. The window is reachable through `ApplyTo`
> (update = `f[current]`, `:102`): `x = 1; f = Function[v, (Protect[x]; v+100)];
> ApplyTo[x, f]` → `x = 101` with `Attributes[x] = {Protected}`, no message
> (reproduced twice). The draft's error was scope — it attributed the hole to all
> eleven compound assignments, but the other ten evaluate their update as pure
> arithmetic/list ops (no user code in the window) and fail closed. Corrected
> finding: the shared `store_held_update` lacks a recheck (root cause), and
> `ApplyTo` is the reachable trigger today. Rows corrected in place; the fix is
> triage batch 1a.
>
> Independently gate-reproduced and standing: the 129/612 `Protected` count (E1),
> the unit-8 attribute sweep (E2), the `$ContextPath`/`$Packages`/`$Path` setter
> bypass (E14), `SetAttributes` on a protected symbol (E5), the value-table false
> round-trip (E7), and the silent literal-wildcard `Clear` (E6). The
> association-lvalue `Set` half of the draft's claim is genuinely fail-closed
> (`a["k"] = (Protect[a]; 99)` → `Set::wrsym`), so that OK row stands. One E5
> claim — `SetOptions` on a protected symbol — is NOT verified: the gate probe
> hit an `optnf` unknown-option error before reaching the protection path. Treat
> it as unconfirmed until the fix re-probes it.

### E1: Calibrated Protected Parser And Live Cross-Check

The parser splits each reference entry, requires exactly one Attributes line,
and extracts **every** backticked token from that line. It does not capture only
the first attribute.

```text
$ python3 - <<'PY'
from pathlib import Path
import re

ref_text = Path('docs/reference/builtins.md').read_text()
entry_re = re.compile(r'^### (?P<name>.+?)\n(?P<body>.*?)(?=^### |\Z)', re.M | re.S)
attr_re = re.compile(r'^- Attributes: (?P<attrs>.+)$', re.M)
entries = {}
for match in entry_re.finditer(ref_text):
    body = match.group('body')
    raw = attr_re.search(body).group('attrs')
    attrs = [] if raw == '`None`' else re.findall(r'`([^`]+)`', raw)
    entries[match.group('name')] = attrs

spec_re = re.compile(r'BuiltinSpec::[A-Za-z_]+\(\s*"([^"]+)"')
stdlib = {
    name
    for path in Path('crates/rum-stdlib/src/catalog').glob('*.rs')
    for name in spec_re.findall(path.read_text())
}
kernel = set(entries) - stdlib
protected = {name for name, attrs in entries.items() if 'Protected' in attrs}
print(f'parsed entries = {len(entries)}')
print(f'entries with one Attributes line = {sum(1 for m in entry_re.finditer(ref_text) if len(attr_re.findall(m.group("body"))) == 1)}')
print(f'protected installed = {len(protected)}')
print(f'unprotected installed = {len(entries) - len(protected)}')
print(f'protected stdlib = {len(protected & stdlib)}')
print(f'unprotected stdlib = {len(stdlib - protected)}')
print(f'protected kernel = {len(protected & kernel)}')
print(f'unprotected kernel = {len(kernel - protected)}')
for name in ['Abs', 'BlockRandom', 'Capitalize', 'Product', 'RandomInteger', 'ConformSchema']:
    print(f'{name}: {entries[name]}')
PY
parsed entries = 612
entries with one Attributes line = 612
protected installed = 129
unprotected installed = 483
protected stdlib = 126
unprotected stdlib = 327
protected kernel = 3
unprotected kernel = 156
Abs: ['Listable', 'NumericFunction', 'Protected']
BlockRandom: ['HoldAll', 'Protected']
Capitalize: ['Listable', 'Protected', 'ReadProtected']
Product: ['HoldAll', 'Protected', 'ReadProtected']
RandomInteger: ['Protected']
ConformSchema: ['HoldRest', 'Protected']
```

Live calibration matched the parsed multi-attribute and kernel entries exactly:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Attributes /@ {"Abs", "BlockRandom", "Capitalize", "Product", "RandomInteger", "ConformSchema", "HoldForm", "NameQ", "SymbolName", "Plus"}'
List[List[Listable, NumericFunction, Protected], List[HoldAll, Protected], List[Listable, Protected, ReadProtected], List[HoldAll, Protected, ReadProtected], List[Protected], List[HoldRest, Protected], List[HoldAll, Protected], List[Protected], List[Protected], List[Listable, Orderless, Flat, OneIdentity]]
```

### E2: Unit-8 Attribute Sweep And Head-Start Reproduction

All 55 results omit `Protected`, in the authoritative group/order:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Attributes /@ {"Set", "SetDelayed", "TagSet", "TagSetDelayed", "TagUnset", "UpSet", "UpSetDelayed", "AddTo", "SubtractFrom", "TimesBy", "DivideBy", "Increment", "Decrement", "PreIncrement", "PreDecrement", "AppendTo", "PrependTo", "ApplyTo", "OwnValues", "DownValues", "UpValues", "SubValues", "DefaultValues", "ValueQ", "Definition", "Information", "Clear", "ClearAll", "Remove", "Unset", "Attributes", "SetAttributes", "ClearAttributes", "Protect", "Unprotect", "HoldAll", "HoldFirst", "HoldRest", "HoldAllComplete", "SyntaxInformation", "Options", "SetOptions", "OptionValue", "ArgumentsOptions", "Message", "Messages", "MessageName", "MessageTags", "MessageDefinitions", "$MessageList", "Quiet", "Check", "CheckArguments", "Off", "On"}'
List[List[], List[], List[], List[], List[], List[], List[], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldFirst], List[HoldRest], List[HoldAllComplete], List[HoldFirst], List[HoldFirst], List[HoldFirst], List[HoldAll], List[HoldFirst], List[HoldFirst], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[], List[HoldAll], List[HoldAll], List[HoldFirst], List[HoldAll], List[HoldAll]]
```

The existing enforcement point rejects the protected random head and stores no
downvalue:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'SeedRandom[123]; RandomInteger[x_] := 99; {RandomInteger[5], DownValues[RandomInteger]}'
List[5, List[]]
! SetDelayed::wrsym: Symbol RandomInteger is Protected.
```

Representative core heads have their planning attributes but no protection;
redefining `Plus` succeeds and changes infix arithmetic:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'before = Attributes /@ {"Plus", "Times", "Power", "Equal", "If", "Map", "Part", "And", "Or", "StringLength"}; Plus[a_, b_] := 42; {before, 1 + 2, DownValues[Plus]}'
List[List[List[Listable, Orderless, Flat, OneIdentity], List[Listable, Orderless, Flat, OneIdentity], List[Listable], List[], List[HoldRest], List[], List[], List[HoldAll], List[HoldAll], List[Listable]], 42, List[RuleDelayed[HoldPattern[Plus[Pattern[a, Blank[]], Pattern[b, Blank[]]]], 42]]]
```

The guard's own head is redefinable:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Unprotect[q_] := 7; {Unprotect[z], DownValues[Unprotect], Attributes[Protect], Attributes[Set]}'
List[7, List[RuleDelayed[HoldPattern[Unprotect[Pattern[q, Blank[]]]], 7]], List[HoldAll], List[]]
```

### E3: Assignment And Tagged Mutation Fail Closed

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'a = 1; e /: h[e] := 9; Protect[a, b, c, d, e, f, g]; r1 = Set[a, 2]; r2 = SetDelayed[b, 2]; r3 = TagSet[c, h[c], 3]; r4 = TagSetDelayed[d, h[d], 4]; r5 = TagUnset[e, h[e]]; r6 = UpSet[h[f], 6]; r7 = UpSetDelayed[h[g], 7]; {r1, a, r2, OwnValues[b], r3, UpValues[c], r4, UpValues[d], r5, h[e], r6, UpValues[f], r7, UpValues[g]}'
List[2, 1, Null, List[], 3, List[], Null, List[], Null, 9, 6, List[], Null, List[]]
! Set::wrsym: Symbol a is Protected.
! SetDelayed::wrsym: Symbol b is Protected.
! TagSet::wrsym: Symbol c is Protected.
! TagSetDelayed::wrsym: Symbol d is Protected.
! TagUnset::wrsym: Symbol e is Protected.
! UpSet::wrsym: Symbol f is Protected.
! UpSetDelayed::wrsym: Symbol g is Protected.
```

### E4: All Compound Assignments Fail Closed

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'p1 = 10; p2 = 10; p3 = 10; p4 = 10; p5 = 10; p6 = 10; p7 = 10; p8 = 10; p9 = {a}; p10 = {a}; p11 = 10; Protect[p1, p2, p3, p4, p5, p6, p7, p8, p9, p10, p11]; AddTo[p1, 2]; SubtractFrom[p2, 2]; TimesBy[p3, 2]; DivideBy[p4, 2]; Increment[p5]; Decrement[p6]; PreIncrement[p7]; PreDecrement[p8]; AppendTo[p9, b]; PrependTo[p10, b]; ApplyTo[p11, Function[z, z + 2]]; {p1, p2, p3, p4, p5, p6, p7, p8, p9, p10, p11}'
! AddTo::wrsym: Symbol p1 is Protected.
! SubtractFrom::wrsym: Symbol p2 is Protected.
! TimesBy::wrsym: Symbol p3 is Protected.
! DivideBy::wrsym: Symbol p4 is Protected.
! Increment::wrsym: Symbol p5 is Protected.
! Decrement::wrsym: Symbol p6 is Protected.
! PreIncrement::wrsym: Symbol p7 is Protected.
! PreDecrement::wrsym: Symbol p8 is Protected.
! AppendTo::wrsym: Symbol p9 is Protected.
! PrependTo::wrsym: Symbol p10 is Protected.
! ApplyTo::wrsym: Symbol p11 is Protected.
List[10, 10, 10, 10, 10, 10, 10, 10, List[a], List[a], 10]
```

### E5: Destructive Guards Hold; Attribute And Option Guards Leak

`Unset`, `Clear`, `ClearAll`, and `Remove` retain protected state. The attribute
and option mutations after them succeed silently: `HoldAll` disappears,
`Listable` appears, and option `a` changes to `1`.

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'p = 1; p[x_] := x; p::usage = "p"; Options[p] = {a -> 0}; SyntaxInformation[p] = {"ArgumentsPattern" -> {_}}; SetAttributes[p, HoldAll]; Protect[p]; Unset[p]; Clear[p]; ClearAll[p]; Remove[p]; SetAttributes[p, Listable]; ClearAttributes[p, HoldAll]; SetOptions[p, a -> 1]; {p, p[3], Attributes[p], Options[p], SyntaxInformation[p], Messages[p]}'
List[1, 1[3], List[Listable, Protected], List[Rule[a, 1]], List[Rule["ArgumentsPattern", List[Blank[]]]], List[p::usage]]
! Unset::wrsym: Symbol p is Protected.
! Clear::wrsym: Symbol p is Protected.
! ClearAll::wrsym: Symbol p is Protected.
! Remove::rmptc: Symbol p is Protected and cannot be removed.
```

The same leaks affect installed heads. `Plus` is first explicitly protected;
`RandomPassword` was already protected by the unit-7 sweep.

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Protect[Plus]; SetAttributes[Plus, HoldAll]; SetOptions[RandomPassword, IgnoreCase -> True]; {Attributes[Plus], Options[RandomPassword]}'
List[List[HoldAll, Listable, Orderless, Flat, OneIdentity, Protected], List[Rule[Require, List[]], Rule[NoAdjacentRepeats, False], Rule[Exclude, List[]], Rule[IgnoreCase, True]]]
```

The source explains the split. Definition writes converge on
`ensure_symbol_not_protected_for_write` (`attributes.rs:209-220`), and clear/
remove paths check before state deletion (`definition_clear.rs:10-85`). By
contrast, attribute updates write at `attributes.rs:124-135` with no guard,
and option updates write at `options/builtins.rs:186-217` with no guard.

### E6: List Forms And Literal Wildcard Strings

`SetAttributes`, `Protect`, and `Unprotect` accept lists. `Attributes` does not,
and the wildcard string is resolved as the exact short name `*`.

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'a = 1; b = 2; Attributes[{a, b}]; SetAttributes[{a, b}, {HoldFirst}]; protected = Protect[{a, b}]; unprotected = Unprotect[{a, b}]; patternProtected = Protect["Global`*"]; {Attributes[a], Attributes[b], protected, unprotected, patternProtected, Attributes["Global`*"]}'
List[List[HoldFirst], List[HoldFirst], List["a", "b"], List["a", "b"], List["*"], List[Protected]]
! Attributes::argform: expected a symbol or symbol-name string argument
```

`Clear` and `Remove` silently leave the matching definitions intact:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'a = 1; b[x_] := x + 1; Clear["Global`*"]; afterClear = {a, b[2]}; Remove["Global`*"]; {afterClear, a, b[2], Names["Global`*"]}'
List[List[1, 3], 1, 3, List["Global`$ScriptInput", "Global`a", "Global`afterClear", "Global`b"]]
```

### E7: Value Tables Are Not Round-Trippable

Each apparent table assignment creates a downvalue on the inspector. The
queried table then echoes the copied rules, but none of the target symbols gains
the corresponding behavior. The final five lists expose the definitions stored
on the inspector heads.

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'a = 7; ov = OwnValues[a]; OwnValues[b] = ov; f[x_] := x + 1; dv = DownValues[f]; DownValues[g] = dv; u /: h[u] := 1; uv = UpValues[u]; UpValues[v] = uv; s[x_][y_] := x + y; sv = SubValues[s]; SubValues[t] = sv; Default[f] = 9; dfv = DefaultValues[f]; DefaultValues[g] = dfv; {b, g[2], h[v], t[1][2], Default[g], OwnValues[b], DownValues[g], UpValues[v], SubValues[t], DefaultValues[g], DownValues /@ {"OwnValues", "DownValues", "UpValues", "SubValues", "DefaultValues"}}'
List[b, g[2], h[v], t[1][2], Default[g], List[RuleDelayed[HoldPattern[a], 7]], List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], Plus[x, 1]]], List[RuleDelayed[HoldPattern[h[u]], 1]], List[RuleDelayed[HoldPattern[s[Pattern[x, Blank[]]][Pattern[y, Blank[]]]], Plus[x, y]]], List[RuleDelayed[HoldPattern[Default[f]], 9]], List[List[RuleDelayed[HoldPattern[OwnValues[b]], List[RuleDelayed[HoldPattern[a], 7]]]], List[RuleDelayed[HoldPattern[DownValues[g]], List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], Plus[x, 1]]]]], List[RuleDelayed[HoldPattern[UpValues[v]], List[RuleDelayed[HoldPattern[h[u]], 1]]]], List[RuleDelayed[HoldPattern[SubValues[t]], List[RuleDelayed[HoldPattern[s[Pattern[x, Blank[]]][Pattern[y, Blank[]]]], Plus[x, y]]]]], List[RuleDelayed[HoldPattern[DefaultValues[g]], List[RuleDelayed[HoldPattern[Default[f]], 9]]]]]]
```

`normalize_definition_assignment_target` recognizes messages,
`SyntaxInformation`, symbols, and ordinary definition lhs forms, but has no
value-table setter branch (`definition_analysis_helpers.rs:183-201`).

### E8: `Information` And `Definition`

Direct `Information[x]` loses a defined symbol to ordinary argument evaluation;
the string form works. Builtin usage falls back to the qualified name rather
than the catalog summary.

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'x = 5; {Information[x], Information["x"], Information["Set"]}'
List[Information[5], InformationData[Association[Rule["ObjectType", "Symbol"], Rule["Usage", "Global`x"], Rule["Documentation", None], Rule["OwnValues", Information`InformationValueForm[OwnValues, x, List[RuleDelayed[HoldPattern[x], 5]]]], Rule["UpValues", None], Rule["DownValues", None], Rule["SubValues", None], Rule["DefaultValues", None], Rule["NValues", None], Rule["FormatValues", None], Rule["Options", None], Rule["Attributes", List[]], Rule["FullName", "Global`x"]]], InformationData[Association[Rule["ObjectType", "Symbol"], Rule["Usage", "System`Set"], Rule["Documentation", None], Rule["OwnValues", None], Rule["UpValues", None], Rule["DownValues", None], Rule["SubValues", None], Rule["DefaultValues", None], Rule["NValues", None], Rule["FormatValues", None], Rule["Options", None], Rule["Attributes", List[]], Rule["FullName", "System`Set"]]]]
```

`Definition` produces an honest held state form rather than a display-only
pretty print:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'f::usage = "desc"; Options[f] = {a -> 0}; SetAttributes[f, HoldFirst]; f = 1; f[x_Integer] := x; Default[f] = 7; SyntaxInformation[f] = {"ArgumentsPattern" -> {_}}; Definition[f]'
HoldComplete[Set[Attributes[f], List[HoldFirst]], Set[f::usage, "desc"], Set[Options[f], List[Rule[a, 0]]], Set[f, 1], Set[Default[f], 7], Set[SyntaxInformation[f], List[Rule["ArgumentsPattern", List[Blank[]]]]]]
```

### E9: `ValueQ` Rollback And Current Subvalue Evaluation

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'x = 0; values = {ValueQ[1], ValueQ[1 + 1], ValueQ[x = x + 1], x}; c = 0; op[a_][b_] := HoldComplete[a, b]; applied = op[c = c + 1][c = c + 1]; {values, applied, c, SubValues[op], Attributes[SubValues]}'
List[List[False, True, True, 0], HoldComplete[1, 2], 2, List[RuleDelayed[HoldPattern[op[Pattern[a, Blank[]]][Pattern[b, Blank[]]]], HoldComplete[a, b]]], List[HoldFirst]]
```

The result is the requested forward-pointer fact: `SubValues` inspection holds
its target, but subvalue *application* evaluates both the inner captured
argument and the outer argument before the subvalue matcher. No
`SubValuesHoldAll`-equivalent is designed here.

### E10: Options Forms And Return Values

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Options[f] = {a -> 0, b -> 2}; updated = SetOptions[f, a -> 1]; {updated, Options[f], OptionValue[f, a], OptionValue[f, {a, b}], Options[f, a]}'
List[List[Rule[a, 1], Rule[b, 2]], List[Rule[a, 1], Rule[b, 2]], 1, List[1, 2], Options[f, a]]
! Options::argx: expected exactly 1 argument(s), got 2
! Options::argx: expected exactly 1 argument(s), got 2
! Options::argx: expected exactly 1 argument(s), got 2
```

The repeated arity output is fixed-point re-evaluation of the inert unsupported
call, not three separate probe invocations.

### E11: Message Selectors, `$MessageList`, And Missing Global Switches

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'f::tag = "boom"; disabled = Off[f::tag]; silent = Message[f::tag]; enabled = On[f::tag]; checked = Check[Message[f::tag]; 1, fallback, {f::tag}]; quiet = Quiet[Message[f::tag]; 2, {f::tag}]; {MessageName[f, tag], Messages[f], MessageTags[f], MessageDefinitions[f], disabled, silent, enabled, checked, quiet, $MessageList, Off[], On[]}'
List[f::tag, List[f::tag], List["tag"], List[Rule[f::tag, "boom"]], Null, Null, Null, fallback, 2, List[HoldForm[f::tag]], Off[], On[]]
! f::tag: boom
! Off::argx: expected at least 1 argument
! On::argx: expected at least 1 argument
! Off::argx: expected at least 1 argument
! On::argx: expected at least 1 argument
! Off::argx: expected at least 1 argument
! On::argx: expected at least 1 argument
```

### E12: Builtin Diagnostics Are Not Inspectable

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Set[]; {Messages[Set], MessageTags[Set], MessageDefinitions[Set]}'
List[List[], List[], List[]]
! Set::argx: expected exactly 2 argument(s), got 0
```

### E13: Session-State Lifecycle Boundary

`HostSession::reset()` rebuilds the original configured session and drops user
definitions, attributes, contexts, loaded packages, and messages
(`crates/rum-host/src/session.rs:161-173`). Text snapshots deliberately capture
definitions plus attributes of defined symbols, but not attribute-only symbols,
`$` variables, options, messages, PRNG state, or message-switch state
(`crates/rum-host/src/session.rs:175-195`). Runtime rollback snapshots used by
`ValueQ` and loader transactions do include options, messages, disabled message
selectors, and `$MessageList` state (`session_helpers.rs:38-89`). This is
coherent with the documented lifecycle and does not justify a capability label
or broader snapshot in this audit.

### E14: Writable Session Variables Ignore Explicit Protection

The variables are intentionally writable by default, but their special setters
also bypass an attribute the user explicitly added. All three remain visibly
`Protected` after the writes:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- cargo run -q -p rum-cli -- --eval 'Protect[System`$ContextPath, System`$Packages, System`$Path]; System`$ContextPath = {"System`", "Leak`"}; System`$Packages = {"Leak`"}; System`$Path = {"/tmp/rum-protected-path"}; {System`$ContextPath, System`$Packages, System`$Path, Attributes[System`$ContextPath], Attributes[System`$Packages], Attributes[System`$Path]}'
List[List["System`", "Leak`"], List["Leak`"], List["/tmp/rum-protected-path"], List[Protected], List[Protected], List[Protected]]
```

The code-visible cause is ordering: all three branches return from
`finish_definition_assignment` before `try_set_symbol_own_value_lvalue` reaches
`ensure_symbol_not_protected_for_write` (`definitions/definitions.rs:198-255`).

## Triage Sketch

1. **Protected-state enforcement holes** (safety, S): make the three special
   writable-session `Set` branches (`$ContextPath`/`$Packages`/`$Path`) honor
   explicit protection, and add per-target checks to `SetAttributes`,
   non-`Protected` `ClearAttributes` changes, and `SetOptions` (verify the
   `SetOptions` path first — it is the one claim the gate could not reproduce);
   retain the explicit `Unprotect` / `ClearAttributes[..., Protected]` escape.
   These are cold mutation paths, not call planning; no A/B gate.

1a. **`store_held_update` protection recheck** (safety, M — REAL BUG, gate-
   reproduced): before writing in `store_held_update`
   (`definition_updates.rs:377-418`), recheck `ensure_symbol_not_protected_for_write`
   on the target (both the `Symbol` and `Part` owner cases). Today `ApplyTo[x, f]`
   with an `f` that protects `x` mid-update lands the write on a visibly
   protected symbol with no message. The other ten compound assignments share
   the code path but cannot trigger it (pure-arithmetic/list update, no user
   code in the window); the single recheck closes all of them. This touches the
   held-update store path, not call planning — no microbench A/B, but add an
   `ApplyTo` regression test. Pair with the `Protected` attribute on the eleven
   heads (batch 2), which is necessary but NOT sufficient here.
2. **Whole-registry `Protected` sweep** (safety, L but mechanical): add the
   attribute to the 479 names in the file map, preserve the four deliberate-open
   session values, sync `stdlib_attributes.rs`, regenerate the reference, and
   add coverage assertions. `Protected` is checked only on mutation, so no A/B
   gate. This can be reviewed as kernel and stdlib sub-batches without changing
   the classification.
3. **Truthful, round-trippable value tables** (convenience/safety, M/L): add
   validated assignment routing for `OwnValues`/`DownValues`/`UpValues`/
   `SubValues`/`DefaultValues`, with atomic validation and a fail-closed result
   on malformed rules. This changes the hot `Set` target-classification path,
   so run the core-language microbench A/B gate even though it is not
   attribute-driven planning.
4. **Safe bulk-name and lookup forms** (flexibility/safety, M): implement
   name-pattern targets for `Clear`/`Remove`/`Protect`/`Unprotect`, add
   `Attributes[{...}]` and `Options[f,name]`, and precompute target sets before
   mutation. These are builtin-local normalization paths; no call-planning A/B
   is expected.
5. **REPL inspection and message discovery** (convenience, M): make
   `Information[symbol]` hold a direct symbol, use catalog summaries, expose
   compact builtin diagnostic tags/templates, and add paired `Off[]`/`On[]`.
   The `Information` hold change touches attribute-driven call planning and
   therefore requires the microbench A/B gate; the catalog/global-switch pieces
   do not.

No batch proposes a new attribute kind, a non-reversible protection mode, a
capability change, or a changed default for a WL-inherited name.

## Verification Outputs

Count reconciliation and its script/output are pasted verbatim at the start of
this document. The calibrated Protected parser, counts, and live cross-check are
pasted verbatim in Evidence E1.

The coverage check scopes itself to the findings tables so the separate
whole-registry summary table cannot create a false positive:

```text
$ python3 - <<'PY'
from pathlib import Path
import re
expected = [
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
body = Path('docs/dev/usability-pass/kernel-a-findings.md').read_text()
findings = body.split('## Findings Table', 1)[1].split('## Hard-Coded-Hold Trace', 1)[0]
row_names = {
    match.group(1).strip()
    for match in re.finditer(r'^\| ([^|]+) \|', findings, re.M)
}
row_names.discard('Builtin')
missing = [name for name in expected if name not in row_names]
extra = sorted(row_names - set(expected))
if missing or extra or len(set(expected)) != 55:
    print(f'COVERAGE-FAIL missing={missing} extra={extra} expected={len(set(expected))}')
else:
    print(f'COVERAGE-OK {len(set(expected))}/{len(set(expected))}')
PY
COVERAGE-OK 55/55
```

```text
$ git status --short
?? docs/dev/usability-pass/kernel-a-findings.md
```

```text
$ scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

## Stop-Loss / Open Questions

None. No runtime probe repeated a failure. The count, attribute, and coverage
instruments were calibrated before their results were trusted.
