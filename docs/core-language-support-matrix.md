# Core Language Support Matrix

## Purpose

This document is the exhaustive working matrix for the current `rum`
core-language target.

The scope is defined by the union of the official Wolfram documentation pages
named in `ARCHITECTURE.md`:

- `Core Language & Structure`
  - `List`
  - `Expressions`
  - `Associations`
  - `Variables & Functions`
  - `Rules & Patterns`
  - `Functional Programming`
  - `Procedural Programming`
  - `String Manipulation`
  - `Package Development`
  - `Tuning & Debugging`
- `Language Overview`
  - `Symbolic Expressions`
  - `Lists & Expression Manipulation`
  - `Associations`
  - `Functional Operations`
  - `Pattern Matching`
  - `Rules & Transformations`
  - `Definitions & Assignments`
  - `Logic & Tests`
  - `Scoping & Modularity`
  - `Procedural Programming`
  - `Error & Exception Handling`
  - `String Manipulation`

This matrix tracks both:

- same-name functions and symbols explicitly pointed to by those pages
- named language mechanisms needed to realize that surface

## Status Legend

- `implemented`: landed with regression coverage and no major known scope note
  beyond ordinary follow-up polish
- `partial`: available now, but still intentionally subsetted or known to miss
  important Wolfram-compatible behavior
- `planned`: inside the scoped kernel target, but not yet implemented
- `excluded`: explicitly pointed to by the scoped pages, but intentionally out
  of scope for the current project phase
- `extension`: useful project-owned surface that is not itself part of the
  official matrix defined above

When a same-name item intentionally diverges from Wolfram behavior, note that
inline next to the item rather than silently leaving it in `implemented`.

## Ordering Model

The groups below are ordered by intended implementation sequence, not by the
official documentation menu hierarchy. `TODO.md` should reference these group
IDs instead of re-enumerating every function name.

- `CL-01`: expression, truth-value, and structural foundation
- `CL-02`: definitions, attributes, assignments, and scoping
- `CL-03`: rules, patterns, and tree walkers
- `CL-04`: functional and list combinators
- `CL-05`: associations and keyed data
- `CL-06`: procedural control, exceptions, and failure flow
- `CL-07`: string language
- `CL-08`: package, namespace, options, and protection tooling
- `CL-09`: tuning, debugging, and verification surface
- `CL-X`: explicit exclusions for this project phase

## Overlap Policy

Many functions appear on more than one official page. Each item is listed only
once in its primary implementation group. Other groups may mention it as
`cross-tracked elsewhere` when that helps explain page coverage.

## CL-01: Expression, Truth, and Structural Foundation

### Source Pages

- `Symbolic Expressions`
- `Expressions`
- `List`
- `Lists & Expression Manipulation`
- `Parts of Expressions`
- `Logic & Tests`

### Mechanisms

- `implemented`: symbolic `Expr` model, atom-vs-call representation, canonical
  heads, fixed-point evaluation core, list as a first-class expression head,
  standard structural traversal groundwork, exact path selectors with integer /
  `All` / `Span` / pattern-position subsets
- `partial`: full Wolfram-compatible part-spec parity, full `Heads` parity
  across every structural walker, general boolean/comparison evaluation
- `planned`: richer expression introspection, broader list-manipulation
  surface, fuller truth-value and predicate surface

### Functions and Symbols

  - `implemented`: `Identity`, `List`, `Head`, `Length`, `Depth`, `TrueQ`,
    `BooleanQ`, `AtomQ`, `ListQ`, `StringQ`, `IntegerQ`, `NumberQ`, `True`,
    `False`, `SameQ`, `UnsameQ`
  - `partial`: `Part`, `Extract`, `ReplacePart`, `Thread`, `First`, `Last`,
    `Rest`, `Most`, `Take`, `Drop`, `TakeDrop`, `Insert`, `Delete`,
    `TakeList`, `Append`, `Prepend`, `FullForm`, `Symbol`, `Construct`,
    `Equal`, `Unequal`, `Less`, `Greater`, `LessEqual`,
    `GreaterEqual`, `And`, `Or`, `Not`, `NumericQ`, `EvenQ`, `OddQ`,
    `Positive`, `Negative`, `NonPositive`, `NonNegative`, `PossibleZeroQ`,
    `MachineNumberQ`, `RealValuedNumberQ`, `RealValuedNumericQ`, `PrimeQ`,
    `AlgebraicIntegerQ`, `DuplicateFreeQ`, `IntersectingQ`, `DisjointQ`,
    `SubsetQ`, `Order`, `OrderedQ`, `ContainsAll`, `ContainsAny`,
    `ContainsExactly`, `ContainsNone`, `ContainsOnly`, `Between`, `VectorQ`,
    `MatrixQ`, `ArrayQ`, `AllTrue`, `AnyTrue`, `NoneTrue`, `CoprimeQ`
  - `planned`: `Heads`,
    `LexicographicOrder`, `NumericalOrder`, `AlphabeticOrder`,
    `GreaterThan`, `LessThan`, `GreaterEqualThan`, `LessEqualThan`,
    `EqualTo`, `UnequalTo`, `NearestTo`, `SameAs`, `PolynomialQ`,
    `PolynomialExpressionQ`, `RationalExpressionQ`, `DateObjectQ`,
    `TimeObjectQ`
- `cross-tracked elsewhere`: `Apply`, `Array`, `Cases`, `Count`,
  `DeleteCases`, `FreeQ`, `Join`, `Map`, `MapAll`, `MapAt`, `MemberQ`,
  `Outer`, `Position`, `Range`, `Select`, `Table`

## CL-02: Definitions, Attributes, Assignments, and Scoping

### Source Pages

- `Variables & Functions`
- `Definitions & Assignments`
- `Scoping & Modularity`

### Mechanisms

- `implemented`: ownvalues / upvalues / downvalues / subvalues, lexical
  substitution, fresh-symbol generation, dynamic rebinding, pure-function
  parameter binding, slot lowering for `#`, `##`, and `#name`, attribute
  storage and dispatch for the current attribute subset
- `partial`: public attribute surface is narrower than Wolfram's full
  attribute family, protection and option layers are still absent
- `planned`: fuller assignment/update family, protection framework, and
  broader definition-introspection parity

### Functions and Symbols

- `implemented`: `Set`, `SetDelayed`, `Unset`, `Clear`, `ClearAll`,
  `Attributes`, `OwnValues`, `UpValues`, `DownValues`, `SubValues`,
  `With`, `Module`, `Block`, `TagSet`, `Flat`, `HoldFirst`,
  `Listable`, `Orderless`
- `partial`: `Attributes`
- `planned`: `AddTo`, `AppendTo`, `ApplyTo`, `CompoundExpression`,
  `Decrement`, `PreDecrement`, `DefaultValues`, `DivideBy`, `Increment`,
  `PreIncrement`, `Information`, `PrependTo`, `Protected`, `Remove`, `Save`,
  `SubtractFrom`, `TimesBy`, `UpSet`, `ValueQ`
- `cross-tracked elsewhere`: `Alternatives`, `BlankSequence`, `Function`,
  `Optional`, `Check`, `Confirm`, `Enclose`, `Quiet`, `Rule`,
  `RuleDelayed`, `WithCleanup`

## CL-03: Rules, Patterns, and Tree Walkers

### Source Pages

- `Rules & Patterns`
- `Pattern Matching`
- `Pattern Matching Functions`
- `Rules & Transformations`

### Mechanisms

- `implemented`: named patterns, `Pattern`, `Blank`, `BlankSequence`,
  `BlankNullSequence`, head-constrained blanks, `Condition`, `PatternTest`,
  `Alternatives`, `Optional`, `Except`, `Repeated`, `RepeatedNull`, matcher
  IR, delayed-rule substitution, suspend/resume through matcher-driven flows
- `partial`: advanced pattern language breadth, full traversal parity, richer
  replacement-convenience family, pattern/string crossover
- `planned`: ordered-pattern controls and remaining parity cleanup

### Functions and Symbols

- `implemented`: `Rule`, `RuleDelayed`, `Condition`, `Except`,
  `Alternatives`, `Optional`, `Repeated`
- `partial`: `MatchQ`, `ReplaceAll`, `ReplaceRepeated`, `Cases`, `Count`,
  `MemberQ`, `FreeQ`, `Position`
- `planned`: `DeleteCases`, `Replace`, `ReplaceAt`, `ReplaceList`,
  `Longest`, `Shortest`, `PatternSequence`, `OrderlessPatternSequence`,
  `Verbatim`
- `cross-tracked elsewhere`: `Set`, `SetDelayed`, `Part`, `Extract`,
  `ReplacePart`, `StringExpression`

## CL-04: Functional and List Combinators

### Source Pages

- `Functional Programming`
- `Functional Operations`
- `Applying Functions to Lists`

### Mechanisms

- `implemented`: pure functions, operator-form lowering for `&`, application
  sugar `@`, `@@`, `@@@`, composition sugar `@*`, `/*`, and slot forms `#`,
  `##`, and `#name`
- `partial`: current threading surface through `Thread`, the existing
  `Listable` subset, first-pass runtime support for `BlockMap`, `SubsetMap`,
  `Comap`, `ComapApply`, `Accumulate`, `MapIndexed`, `Map`, `MapAll`,
  `MapAt`, `Scan`, `MapThread`, `TakeWhile`, `LengthWhile`, `Inner`, `Fold`,
  `FoldList`, `FoldPair`, `FoldWhile`, `SequenceFold`, `SequenceFoldList`,
  `Nest`, `NestWhile`, `NestList`, `NestWhileList`, `FixedPoint`,
  `FixedPointList`, `Apply`, `MapApply`, `OperatorApplied`, `CurryApplied`,
  `ReverseApplied`, `Composition`, `RightComposition`, `Through`, `Operate`,
  `ArrayFilter`, and `ArrayReduce`, plus current `levelspec` / `Heads`
  support for the `Apply`, `Map`, and `Comap` families, current
  callable-wrapper support for `OperatorApplied` / `CurryApplied` /
  `ReverseApplied`, operator-form `Heads` parity for the `Apply` family,
  current `Operate[..., 0]` and `Operate[p, n]` parity, current
  post-transform evaluation parity for `Through` / `Operate`, current
  rectangular-list radius/template support with `Padding -> None`,
  `Padding -> "Fixed"`, `Padding -> "Periodic"`, constant-padding forms,
  cyclic padding lists, matching per-axis asymmetric padding pairs, and
  explicit even-template anchoring for `ArrayFilter`, and current
  dimension-spec plus grouped-dimension support for `ArrayReduce`
- `planned`: higher-order traversal, repeated-application families, and
  remaining operator/combinator convenience layers

### Functions and Symbols

- `implemented`: `Function`
- `partial`: `Thread`, `BlockMap`, `SubsetMap`, `Comap`, `ComapApply`,
  `Accumulate`, `MapIndexed`, `Map`, `MapAll`, `MapAt`, `Scan`, `MapThread`,
  `TakeWhile`, `LengthWhile`, `Inner`, `Fold`, `FoldList`, `FoldPair`,
  `FoldWhile`, `SequenceFold`, `SequenceFoldList`, `Nest`, `NestWhile`,
  `NestList`, `NestWhileList`, `FixedPoint`, `FixedPointList`, `Apply`,
  `MapApply`, `OperatorApplied`, `CurryApplied`, `ReverseApplied`, `Through`,
  `Operate`, `ArrayFilter`, `ArrayReduce`, `Composition`,
  `RightComposition`
- `planned`: `ArrayFilter` / `ArrayReduce` sparse-array parity and any future
  broader ArrayPad-style padding or storage compatibility layers beyond the
  current dense-array subset
- `cross-tracked elsewhere`: `Append`, `Array`, `AssociationMap`, `Cases`,
  `CountsBy`, `GroupBy`, `Identity`, `KeySortBy`, `Outer`, `Position`,
  `Select`

## CL-05: Associations and Keyed Data

### Source Pages

- `Associations`

### Mechanisms

- `partial`: named-slot syntax `#name` exists generically, and the current
  first-pass association subset now includes association syntax/lowering,
  key-value canonicalization, keyed part specifications via `Key`,
  deterministic ordering for canonical association rendering, and key-based
  pattern matching through `KeyValuePattern`
- `planned`: held in-place mutation parity for `AssociateTo` / `KeyDropFrom`,
  fuller `Lookup` threading and multi-key parity, broader key-spec parity
  across key-manipulation helpers, richer `Missing` semantics, and `KeySortBy`
  option/comparer parity

### Functions and Symbols

- `partial`: `Association`, `AssociationQ`, `Lookup`, `Key`, `Keys`, `Values`,
  `KeyExistsQ`, `KeyTake`, `KeyDrop`, `KeyMap`, `KeyValueMap`, `KeySelect`,
  `KeyMemberQ`, `KeyFreeQ`, `KeySort`, `KeySortBy`, `AssociationMap`,
  `AssociationThread`, `KeyUnion`, `KeyIntersection`, `KeyComplement`,
  `Normal`, `AssociateTo`, `KeyDropFrom`, `KeyValuePattern`, `Merge`,
  `Catenate`, `Counts`, `CountsBy`, `GroupBy`, `PositionIndex`, `Missing`
- `planned`: `KeySortBy` option parity, keyed part-spec parity, and remaining
  deep association semantics beyond the current first-pass subset
- `cross-tracked elsewhere`: `#name`, `DeleteDuplicates`, `Map`, `Plus`,
  `Select`, `Sort`

## CL-06: Procedural Control, Exceptions, and Failure Flow

### Source Pages

- `Procedural Programming`
- `Scoping & Modularity`
- `Error & Exception Handling`

### Mechanisms

- `implemented`: hold-aware branch selection, lexical and dynamic local state,
  message interception through the current `Check` / `Quiet` model
- `partial`: sequential evaluation plus first-pass `Switch` / `Do` /
  `While` / `Until` / `For` / `Table` / `Throw` / `Catch` / `Return` and
  `Enclose` / `Confirm*` / `Failure` now work, and the current cleanup/abort
  subset includes `WithCleanup`, `Abort`, `CheckAbort`, `PropagateAborts`,
  `AbortProtect`, and callable `Failsafe`; the current status-object and
  basic I/O subset also includes `MissingQ`, `Success`,
  `TerminatedEvaluation`, `Input`, `Print`, `Pause`, and `BlockRandom`.
  Recent follow-up work tightened protected-abort catching, session-config
  assertion enablement, `Success[...]` property extraction, and request-level
  print transport, but richer held/cleanup corners and deeper status parity
  are still pending
- `planned`: broader failure values, abort-protection parity, external
  interaction builtins, and deeper cleanup/exception compatibility

### Functions and Symbols

- `implemented`: `If`, `Which`, `Check`, `Quiet`, `CompoundExpression`,
  `Switch`
- `partial`: `Do`, `While`, `Until`, `For`, `Break`, `Continue`, `Return`,
  `Throw`, `Catch`, `Table`, `Enclose`, `Confirm`, `ConfirmBy`,
  `ConfirmMatch`, `ConfirmQuiet`, `ConfirmAssert`, `Failure`, `FailureQ`,
  `Success`, `TerminatedEvaluation`, `MissingQ`, `WithCleanup`, `Failsafe`,
  `Abort`, `CheckAbort`, `PropagateAborts`, `AbortProtect`, `Assert`,
  `BlockRandom`, `Input`, `Print`, `Pause` in the current first-pass subset
- `planned`: deeper `WithCleanup` / `Failsafe` / `CheckAbort` /
  `AbortProtect` parity, richer `Assert` controls beyond the current session
  flag, more complete `TerminatedEvaluation` behavior, and fuller host-I/O
  semantics
- `cross-tracked elsewhere`: `Block`, `Function`, `Module`, `Rule`,
  `RuleDelayed`, `With`

## CL-07: String Language

### Source Pages

- `String Manipulation`

### Mechanisms

- `implemented`: canonical string atoms, Unicode code-point semantics, parser
  escape support, exact-string construction and transformation core
- `partial`: `ToString` currently accepts the common `InputForm` /
  `OutputForm` / `StandardForm` / `FullForm` forms but still renders through
  the current canonical kernel textual subset, the string-pattern surface is
  still a first-pass subset, and the
  byte-array/hash/compress/encrypt/nearest/pluralization layer is still a
  current subset rather than full Wolfram parity
- `planned`: fuller symbolic string-pattern layer, conversions, templating,
  and nontrivial search/transformation semantics

### Functions and Symbols

- `implemented`: `Characters`, `StringJoin`, `StringLength`, `StringRepeat`,
  `StringTake`, `StringDrop`, `StringPart`
- `partial`: `ToString`, `StringContainsQ`, `StringExpression`,
  `StringReplace`, `StringCases`, `StringPosition`, `StringSplit`,
  `StringTakeDrop`, `StringCount`, `StringMatchQ`, `StringFreeQ`,
  `ToUpperCase`, `Whitespace`, `NumberString`, `LetterQ`, `DigitQ`,
  `StringApply`, `StringDelete`, `StringForm`, `StringPadLeft`,
  `StringRiffle`, `StringTemplate`, `TemplateApply`, `TextWords`,
  `ToCharacterCode`, `ToExpression`, `SyntaxQ`, `TextString`,
  `FileTemplateApply`, `CharacterCounts`, `RemoveDiacritics`,
  `Alphabet`, `AlphabeticSort`, `EditDistance`, `HammingDistance`,
  `LongestCommonSubsequence`, `SequenceAlignment`, `Diff`,
  `StringToByteArray`, `Hash`, `Compress`, `Encrypt`, `Nearest`,
  `Pluralize`
- `partial richer string-pattern parity`: `StartOfString`, `EndOfString`,
  `Repeated`, `RepeatedNull`, bounded repetition, `Shortest`, `Longest`,
  `Except`, `LetterCharacter`, `DigitCharacter`, `WordCharacter`,
  `WhitespaceCharacter`, `RegularExpression[...]` interop, and current
  zero-length/match-edge handling
- `partial richer nearest/pluralize parity`: `Nearest` now supports the
  current `DistanceFunction -> EditDistance|HammingDistance` subset over
  strings, and `Pluralize` now covers a current irregular-English subset in
  addition to the earlier rule-based forms
- `planned`: deeper byte-array/crypto compatibility, richer nearest-neighbor
  distance/radius semantics, fuller compression/encryption compatibility,
  and fuller linguistic-data-backed pluralization parity

## CL-08: Package, Namespace, Options, and Protection Tooling

### Source Pages

- `Package Development`

### Mechanisms

- `implemented`: contexts, context-path management, package load tracking,
  loader rollback on failure, message definition storage, message selection,
  definition/message inspection
- `partial`: explicit `Protect` / `Unprotect` now work on top of the shared
  attribute model, `Protected` participates in `Attributes[...]`, and current
  definition/message/clear write paths honor a first-pass protected-symbol
  barrier; symbol-level option storage and lookup now cover explicit
  `Options[...]`, `SetOptions[...]`, `OptionValue[...]`, `Options[f] = {...}`,
  and `Unset[Options[f]]`; current system default options now cover the
  first-pass subset already consumed by `ArrayFilter`, `CheckAbort`, the
  current string-option family, and the current `Heads`-aware tree-walker
  family, and `OptionsPattern[...]` now integrates with one-argument
  `OptionValue[...]` inside matching definitions; `SetOptions` now accepts
  symbol target lists, delayed option rules, and name-fallback option
  matching, and `OptionValue[...]` now accepts lists of option names while
  evaluating delayed option values at lookup time; `Names` now covers `Names[]`, string `*` wildcards, first-pass
  `RegularExpression[...]`, `StringExpression`, `Alternatives`, and
  pattern-list unions over current visible contexts or explicit contexts,
  including the current bounded / alternation / grouping /
  non-capturing-group regex subset; `Save` now rejects unknown options in
  the current package-tooling subset, and `Once` now accepts option rules in
  second position, reports unknown option names, and honors
  `PersistenceTime -> 0` as a no-cache subset;
  `SyntaxInformation` now covers direct query, assignment, `Unset`,
  system-default metadata for the current package/tooling subset plus a
  broader current slice of optioned string/traversal/package helpers,
  `Protected` barriers, and `Block` / loader snapshot participation;
  `DeclarePackage` now installs stub-like declared symbols, updates
  `$ContextPath`, and auto-`Needs` the declared context on first symbol use;
  `Save` / `DumpSave` now provide a textual first-pass serialization layer
  that can round-trip through `Get`, with current `IncludedContexts` /
  `ExcludedContexts` filtering and `DumpSave` `SymbolAttributes` handling,
  and `Once` now provides session-local memoization keyed by held expression
  plus location;
  `ArgumentsOptions` / `CheckArguments` now cover the current held-call and
  trailing-options subset, and direct builtin `UpSet` / `UpSetDelayed` now
  cover first-pass inferred upvalue tags
- `planned`: non-blocking long-tail package ergonomics, deeper
  `DeclarePackage` / `Save` / `DumpSave` / `Once` compatibility, richer
  `Names` regex parity beyond the current wildcard/grouping subset, and
  broader `SyntaxInformation` properties

### Functions and Symbols

- `implemented`: `$ContextPath`, `$Packages`, `Begin`, `BeginPackage`, `End`,
  `EndPackage`, `Needs`, `Message`, `MessageName`
- `partial`: `Protect`, `Unprotect`, `Protected`, `Options`, `SetOptions`,
  `OptionValue`, `OptionsPattern`, `Names`, `SyntaxInformation`,
  `ArgumentsOptions`, `CheckArguments`, `UpSet`, `UpSetDelayed`,
  `DeclarePackage`, `DumpSave`, `Once`, `Save`
- `planned`: long-tail `DeclarePackage` / `Save` / `DumpSave` / `Once`
  parity, richer `Names` regex coverage beyond the current wildcard /
  bounded / alternation subset, and broader `SyntaxInformation` properties
- `cross-tracked elsewhere`: `AbortProtect`, `Assert`, `Block`,
  `BlockRandom`, `Check`, `Confirm`, `Enclose`, `Failure`, `FailureQ`,
  `Module`, `MissingQ`, `Quiet`, `Success`, `TagSet`, `WithCleanup`

## CL-09: Tuning, Debugging, and Verification Surface

### Source Pages

- `Tuning & Debugging`
- `Error & Exception Handling`

### Mechanisms

- `implemented`: structured runtime messages, top-level message list, internal
  structured tracing, request-level trace payloads, recursion / iteration /
  evaluation-pass limits
- `partial`: a first language-surface debugging layer now exists through a
  structured `Trace[...]` tree subset, including current filtered
  `Trace[..., patt]` forms, current list-pattern filtering, current
  structured levelspec filtering, and current `Heads -> True|False`
  support, now with depth-aware nested branches and more stable compact
  filtered shells, plus an indented structured `TracePrint[...]` subset;
  exact-rational wall-clock
  `Timing[...]` / `AbsoluteTiming[...]`, fixed-sample
  `RepeatedTiming[...]`, echo-style debugging output, session-level `On` /
  `Off` message gating, `Sow` / `Reap` tag collection, a current
  canonical-text `Short[...]` display subset, a current
  layout-aware structural `ByteCount[...]` subset with payload-aware
  `ByteArray[...]` accounting, cooperative
  `TimeConstrained[...]` / `MemoryConstrained[...]` wrappers, a session
  layout-aware structural memory-proxy `MemoryInUse[]` subset, `Monitor[...]` hooks with
  current `EvaluationMonitor` / `StepMonitor` option symbols plus stable
  initial/final display refreshes, broader default-option coverage, and
  explicit `None` callback overrides, a first-pass `Dialog[...]` subset with
  `Return`-sensitive evaluation boundaries plus current `DialogProlog` /
  `DialogSymbols` handling and host-input resume support,
  `EvaluationData[...]` property projection through current string-or-symbol,
  list, and `All` forms plus `"Properties"` with matching structured `"Trace"`,
  held `"Input"`, and held `"MessagesExpressions"` payloads, including current
  input-resume coverage, and
  first-pass `TestCreate[...]` / `TestReport[...]`
  metadata through
  `TestID`, `SameTest`, and expected-message matching with current
  `TestReport[..., prop]` string-or-symbol projection plus report-level
  `"Success"`, held per-test `"Input"`, current merging of
  `{report_1, report_2, ...}`, broader per-test `ActualMessagesText` /
  `OutputLog` / `Timing` payloads, and current report-level
  `SameTest` / `TimeConstraint` / `MemoryConstraint` options, alongside
  `Stack[]`; richer
  verification, monitoring, and resource-accounting parity is still
  pending even though the internal tracing/message
  foundations are already in place
- `planned`: user-facing trace/timing/resource-limit family, verification
  surface, and debugging output controls

### Functions and Symbols

- `implemented`: none beyond the internal mechanisms listed above
- `partial`: `Trace`, `TracePrint`, `Timing`, `AbsoluteTiming`,
  `RepeatedTiming`, `ByteCount`, `Short`, `Echo`, `EchoFunction`,
  `EchoLabel`, `On`, `Off`, `Sow`, `Reap`, `TimeConstrained`,
  `MemoryConstrained`, `MemoryInUse`, `Monitor`, `Dialog`,
  `EvaluationMonitor`, `StepMonitor`, `EvaluationData`, `TestCreate`,
  `TestReport`, `Stack` in
  the current first-pass subset, including a structured default
  `Trace[expr]` model plus structured filtered trace forms with the current
  levelspec and `Heads` filtering, current default `Heads` option support
  for filtered `Trace[...]` / `TracePrint[...]`, depth-aware nested branch
  shaping, an indented structured `TracePrint[...]` subset, current
  layout-aware structural byte/memory accounting with payload-aware
  `ByteArray[...]` handling, current symbol-or-string report/property projection,
  held message-expression payloads, current report merging plus
  current report-level `SameTest` / `TimeConstraint` / `MemoryConstraint`
  options, broader per-test `ActualMessagesText` / `OutputLog` / `Timing`
  payloads, current input-resume coverage for `EvaluationData` / `Monitor`,
  and a first-pass `Dialog[...]` / `Dialog[]` one-shot subset with current
  `DialogProlog` / `DialogSymbols` handling; treat any remaining deeper
  real-memory accounting or trace/report edge-case parity as long-tail polish
  rather than active mainline blockers
- `planned`: deeper real-memory accounting and trace/report edge-case polish
- `cross-tracked elsewhere`: `AbortProtect`, `Check`, `CheckAbort`,
  `Failure`, `Message`, `Missing`, `PropagateAborts`, `Quiet`

## CL-X: Explicit Exclusions For This Project Phase

These names are explicitly pointed to by the scoped pages, but are currently
kept outside the kernel-first project phase.

### Notebook, Front-End, and UI-Specific

- `TreeForm`
- `DynamicModule`
- `CellContext`
- `File > New > Package (.wl)`
- `Initialization Cell`
- `Notebook History`
- `CurrentNotebookImage`
- `CurrentScreenImage`
- `PrintTemporary`
- `TrackedSymbols`
- `SynchronousUpdating`
- `UpdateInterval`

### Graphics, Visualization, and Dataset UI

- `ArrayPlot`
- `Column`
- `Grid`
- `ListPlot`
- `GraphPlot`
- `WordCloud`
- `Plot`
- `Plot3D`
- `NestGraph`
- `Dataset`
- `JoinAcross`

### External Operations, Files, Web, and Deployment

- `Import`
- `Export`
- `ImportString`
- `ExportString`
- `ExpandFileName`
- `FileBaseName`
- `FileNameSplit`
- `FileNameTake`
- `URLBuild`
- `URLDecode`
- `URLEncode`
- `URLParse`
- `Encode`
- `DefineResourceFunction`
- `ResourceFunction`
- `ResourceSubmit`
- `wolframscript`
- `HostLookup`
- `PingTime`
- `StartWebSession`
- `WebExecute`
- `WebImage`
- `NetworkPacketTrace`

### AI, Semantic Services, and Knowledge-System Hooks

- `Interpreter`
- `SemanticInterpretation`
- `DictionaryLookup`
- `LLMResourceFunction`
- `LLMFunction`
- `LLMExampleFunction`
- `LLMPrompt`
- `LLMTool`
- `LLMSynthesize`
- `ExampleData`

### Compiler, Parallel, and System-Scale Tooling

- `Compile`
- `FunctionCompile`
- `Parallelize`
- `Dispatch`
- `DataStructure`
- `SystemInformation`
- `SystemOptions`
- `RandomGeneratorState`
- `RandomSeeding`

### Mathematics Outside The Current Non-Math Core Target

- `N`
- `Sum`
- `NDSolve`

## Project-Owned Extensions Outside The Official Matrix

These are useful surfaces that exist today, but they are not themselves part
of the page-defined official matrix above.

- `implemented`: `CharacterRange`, `Context`, `FindFile`, `Get`,
  `InputString`, `Level`, `MessageDefinitions`, `MessageTags`, `Messages`,
  `NormalizeRules`, `ReleaseHold`, `SetAttributes`, `ClearAttributes`,
  `StringEndsQ`, `StringStartsQ`, `StringTrim`, `TagSetDelayed`, `TagUnset`,
  `$Context`, `$Failed`, `$MessageList`, `$Path`, `File`

## Maintenance Notes

- Update this matrix before or alongside the implementation PR that changes a
  same-name item in the scoped surface.
- Keep extra helper builtins in `extension` unless they are explicitly named on
  the scoped official pages.
- When an item is intentionally subsetted, keep it in `partial` until the
  remaining gap is either implemented or documented as a deliberate divergence.
- Keep `TODO.md` aligned to the `CL-*` group IDs instead of re-listing every
  function name there.
