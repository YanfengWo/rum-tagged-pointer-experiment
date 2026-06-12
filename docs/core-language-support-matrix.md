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
- `CL-Y`: `mma-hol` real-project pressure lane
- `CL-X`: explicit exclusions for this project phase

## Current Roadmap State

The `CL-01` through `CL-09` mainlines are closed for the current kernel-first
phase. Remaining `partial` / `planned` notes below describe known subset
boundaries, demand-driven parity tails, or future feature breadth. They should
not be read as active blockers unless `TODO.md` or a concrete downstream
project promotes one of them back into scheduled work.

`CL-Y` is different from the page-defined mainlines: it is a concrete
real-project acceptance lane. Items promoted by `CL-Y` should still be small,
evidence-backed kernel or harness tasks, not a broad promise to implement every
Wolfram feature that appears in a neighboring project.

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
  standard structural traversal groundwork, exact path selectors with integer
  position specs, `Part` / `Extract` `All` / `Span` subsets, including current
  `UpTo[...]` span-endpoint support plus current invalid `Span` / `UpTo`
  endpoint and argument-count diagnostics, and `ReplacePart`
  pattern-position subsets, including current atom `All` / positive empty-span
  and zero-endpoint boundary behavior in `Part` / `Extract`, current
  association-aware numeric/range `Part` / `Extract` value projection, current
  association-aware `First` / `Last` value selection plus `Rest` / `Most`
  association slicing and empty-input diagnostics, plus the current `Part`
  selector-list boundary and `Extract` nested position-list selector subset,
  including shaped integer / `Key[...]` selectors and nested `All` / `Span`
  rejection, empty-expression,
  empty-`Association[]`, and association-aware `Depth` parity, current unary
  `Extract[positionSpec]` operator-form support,
  `Level` scalar-zero, standard-`All`, empty-range, mixed positive/negative,
  and positive/negative `Infinity` levelspec parity plus current
  `Level::level` invalid-levelspec diagnostics and current
  traversal-family invalid-levelspec / match-limit `::level` / `::innf`
  diagnostics, and empty-expression negative-level depth alignment plus current
  oracle-locked `Heads` traversal edges, including string-named, empty-list,
  and nested-list `Heads` option forms and current `Depth` / `Level` /
  traversal-family `Heads` option diagnostics, plus
  compound-head negative-level handling across the current structural and
  cases/delete walkers, current
  `Not[Not[...]]` and relational-`Not` simplifications,
  flat nested `And` / `Or` residual simplification, first-pass `Xor` /
  `Equivalent` boolean/repeated-term reductions, first-pass `Implies`
  simplification including identical-argument tautology, current short-circuit
  `Nand` / `Nor`, current `Listable` `Boole` truth-to-integer conversion plus
  current `Listable` exact numeric predicate coverage,
  mixed-symbolic `Equal` / `Unequal` chain boundaries, adjacent mixed
  comparable `Equal` contradiction detection, current boolean-pair equality
  handling and `Unequal` known-prefix duplicate detection, plus current numeric
  order-chain endpoint contradiction detection while leaving nonnumeric order
  comparisons conservative, equality-like comparison operator forms for
  zero- and multi-argument applications, and current order-comparison
  operator-form arity diagnostics,
  nonatomic `OrderedQ` argument-order parity, including the current
  symbol-name, negative-power, and common system call-head ordering slice,
  `ReplacePart` atom-head non-replacement, current invalid and out-of-range
  `Part` / `Extract` selector diagnostics, and non-expanding `All` / `Span`
  position-spec boundaries, direct empty `Span[]` full-range selection,
  direct single-endpoint `Span[...]` prefix / suffix / empty-range selection,
  current de-duplicated invalid `UpTo[...]` span endpoint diagnostics, current
  invalid `Span` message rendering for infixable specs, current
  `Part::take` / `Extract::take` range rendering, current `Flatten::normal`
  atom / association input boundary, current `Take` / `Drop` / `TakeDrop` /
  `TakeList` atom-input diagnostics, current `Insert` / `Delete` atom-input
  and out-of-range integer-position diagnostics, current `Insert` / `Delete`
  integer path / list-of-paths subset including direct `Delete[..., 0]` head
  deletion, plus current `Symbol` string / arity diagnostics
- `partial`: full Wolfram-compatible part-spec parity, full `Heads` parity
  across every structural walker, `Flatten` beyond the current same-head /
  single-head-spec non-negative-level / `Infinity` plus level-list subset,
  `Join` beyond the current top-level same-head / association /
  positive-integer level subset, `Range` beyond the current exact-integer
  one-, two-, and three-argument subset, `Abs` / `Factorial` beyond the
  current exact numeric / non-negative exact-integer subset, general
  boolean/comparison evaluation beyond the current exact-numeric order-chain
  reductions
- `planned`: richer expression introspection, broader list-manipulation
  surface, fuller truth-value and predicate surface

### Functions and Symbols

  - `implemented`: `Identity`, `List`, `Head`, `Length`, `Depth`, `TrueQ`,
    `BooleanQ`, `Boole`, `AtomQ`, `ListQ`, `StringQ`, `IntegerQ`, `NumberQ`,
    `True`, `False`, `SameQ`, `UnsameQ`, `LessThan`, `GreaterThan`,
    `LessEqualThan`, `GreaterEqualThan`, `EqualTo`, `UnequalTo`, `SameAs`
  - `partial`: `Part`, `Extract`, `ReplacePart`, `Thread`, `First`, `Last`,
    `Rest`, `Most`, `Take`, `Drop`, `TakeDrop`, `Insert`, `Delete`,
    `TakeList`, `Flatten`, `Join`, `Reverse`, `Transpose`,
    `DeleteDuplicates`, `DeleteDuplicatesBy`, `Sort`, `SortBy`, `Union`,
    `Complement`, `Append`, `Prepend`, `Range`, `FullForm`, `Symbol`,
    `Construct`, `Abs`, `Factorial`,
    `Equal`, `Unequal`, `Less`, `Greater`, `LessEqual`,
    `GreaterEqual`, `And`, `Or`, `Nand`, `Nor`, `Xor`, `Equivalent`, `Implies`, `Not`, `NumericQ`, `EvenQ`, `OddQ`,
    `Positive`, `Negative`, `NonPositive`, `NonNegative`, `PossibleZeroQ`,
    `MachineNumberQ`, `RealValuedNumberQ`, `RealValuedNumericQ`, `PrimeQ`,
    `AlgebraicIntegerQ`, `DuplicateFreeQ`, `IntersectingQ`, `DisjointQ`,
    `SubsetQ`, `Order`, `OrderedQ`, `ContainsAll`, `ContainsAny`,
    `LexicographicOrder`, `NumericalOrder`, `AlphabeticOrder`,
    `ContainsExactly`, `ContainsNone`, `ContainsOnly`, `Between`, `VectorQ`,
    `MatrixQ`, `ArrayQ`, `AllTrue`, `AnyTrue`, `NoneTrue`, `CoprimeQ`,
    `NearestTo`, `PolynomialQ`, `PolynomialExpressionQ`,
    `RationalExpressionQ`, `DateObjectQ`, `TimeObjectQ`
  - `planned`: `Heads`
- `cross-tracked elsewhere`: `Apply`, `Array`, `Cases`, `Count`,
  `DeleteCases`, `FreeQ`, `Map`, `MapAll`, `MapAt`, `MemberQ`, `Outer`,
  `Position`, `Select`, `Table`

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
  attribute family, and `Unique[...]` currently covers the zero-argument,
  string-prefix, symbol-prefix, list, and current attribute-setting forms;
  the held update family currently covers symbol-target `AddTo`,
  `SubtractFrom`, `TimesBy`, `DivideBy`, `ApplyTo`, `AppendTo`, `Increment`,
  `Decrement`, `PreIncrement`, `PreDecrement`, and `PrependTo`, including
  the current `+=` / `-=` / `*=` / `/=` / `//=` / postfix `++` /
  postfix `--` / prefix `++` / prefix `--` lowering and exact part-target
  updates including current bulk-selector distribution / broadcast assignment,
  empty bulk-selector no-op writes, and duplicate nested bulk selector leaf
  distribution, while nested selector lists that `Part` rejects are also
  rejected for held updates with current `Part::pkspec1` / `Set::pkspec1`
  diagnostics, and non-variable held-update targets use current `rvalue`
  diagnostics; `Remove` currently clears stored state for symbol
  and string-name targets, marks same-evaluation stale symbols as
  `Removed["name"]`, and allows later same-name reuse through runtime symbol
  generations, while `DefaultValues` includes the current `Default[...]` and
  explicit `SyntaxInformation[...]` entries, public definition lists render the
  current `HoldPattern[lhs] :> rhs` shape, and `Definition` returns a held
  first-pass definition expression; broader stale-symbol rendering /
  introspection corners and the protection / option / package-tooling layers
  are cross-tracked in `CL-08`
- `planned`: fuller assignment/update diagnostic parity and broader definition
  print-layer parity beyond the current transport-safe expression shapes

### Functions and Symbols

- `implemented`: `Set`, `SetDelayed`, `Unset`, `Clear`, `ClearAll`,
  `Attributes`, `OwnValues`, `UpValues`, `DownValues`, `SubValues`,
  `With`, `Module`, `Block`, `TagSet`, `Flat`, `HoldFirst`,
  `Listable`, `Orderless`
- `partial`: `Attributes`, `AddTo`, `SubtractFrom`, `TimesBy`, `DivideBy`,
  `ApplyTo`, `AppendTo`, `Increment`, `Decrement`, `PreIncrement`,
  `PreDecrement`, `PrependTo`, `DefaultValues`, `Definition`, `Information`,
  `Remove`, `Unique`, `ValueQ`
- `planned`: remaining update-family diagnostics, exact return-form /
  message-count parity for invalid nested held part-targets, deeper
  `Removed[...]` rendering / introspection corners, and definition-introspection
  print-layer parity beyond the current `HoldPattern` value-list, held
  `Definition`, and `InformationValueForm` summary shapes
- `cross-tracked elsewhere`: `Alternatives`, `BlankSequence`, `Function`,
  `Optional`, `Check`, `Confirm`, `CompoundExpression`, `Enclose`, `OptionValue`,
  `Options`, `OptionsPattern`, `Protect`, `Protected`, `Quiet`, `Rule`,
  `RuleDelayed`, `Save`, `SetOptions`, `Unprotect`, `UpSet`,
  `UpSetDelayed`, `WithCleanup`

## CL-03: Rules, Patterns, and Tree Walkers

### Source Pages

- `Rules & Patterns`
- `Pattern Matching`
- `Pattern Matching Functions`
- `Rules & Transformations`

### Mechanisms

- `implemented`: named patterns, `Pattern`, `Blank`, `BlankSequence`,
  `BlankNullSequence`, head-constrained blanks, `Condition`, `PatternTest`,
  `Alternatives`, `Optional`, `Except`, `Repeated`, `RepeatedNull`,
  `PatternSequence`, `OrderlessPatternSequence`, `Verbatim`, `Longest`,
  `Shortest`, matcher IR, delayed-rule substitution, suspend/resume through
  matcher-driven flows
- `partial`: advanced pattern language breadth, pattern/string crossover,
  and remaining long-tail beyond the current association-aware `Cases` /
  `DeleteCases` / `Level` / `Count` / `MemberQ` / `FreeQ` / `Position`
  tree-walker slice plus the current matcher-driven replacement convenience
  subset, including the current `Replace` / `ReplaceList` / `ReplaceAt`
  first passes, root backtracking for `ReplaceList`, and current
  `Flat` / `Orderless` regrouping parity for `MatchQ` / `ReplaceAll` /
  `ReplaceList`
- `planned`: ordered-pattern controls and remaining parity cleanup

### Functions and Symbols

- `implemented`: `Rule`, `RuleDelayed`, `Condition`, `Except`,
  `Alternatives`, `Optional`, `Repeated`, `PatternSequence`,
  `OrderlessPatternSequence`, `Verbatim`, `Longest`, `Shortest`
- `partial`: `MatchQ`, `Replace`, `ReplaceList`, `ReplaceAt`, `ReplaceAll`,
  `ReplaceRepeated`, `Cases`, `Count`, `MemberQ`, `FreeQ`, `Position`
- `partial`: `DeleteCases`
- `cross-tracked elsewhere`: `Set`, `SetDelayed`, `Part`, `Extract`,
  `ReplacePart`, `StringExpression`

## CL-04: Functional and List Combinators

### Source Pages

- `Functional Programming`
- `Functional Operations`
- `Applying Functions to Lists`

### Mechanisms

- `implemented`: pure functions, operator-form lowering for `&`, application
  sugar `@`, `/@`, `@@`, `@@@`, composition sugar `@*`, `/*`, and slot forms
  `#`, `##`, and `#name`
- `partial`: current threading surface through `Thread`, the existing
  `Listable` subset, first-pass runtime support for `BlockMap`, `SubsetMap`,
  `Comap`, `ComapApply`, `Accumulate`, `MapIndexed`, `Map`, `MapAll`,
  `MapAt`, `Scan`, `MapThread`, `TakeWhile`, `LengthWhile`, `Inner`, `Fold`,
  `FoldList`, `FoldPair`, `FoldWhile`, `SequenceFold`, `SequenceFoldList`,
  `Nest`, `NestWhile`, `NestList`, `NestWhileList`, `FixedPoint`,
  `FixedPointList`, `Apply`, `MapApply`, `OperatorApplied`, `CurryApplied`,
  `ReverseApplied`, `Composition`, `RightComposition`, `Through`, `Operate`,
  `Array`, `Outer`, `Select`, `ArrayFilter`, and `ArrayReduce`, plus current
  `levelspec` / `Heads`
  support for the `Apply`, `Map`, and `Comap` families, current
  negative-level empty-expression depth alignment for the `Apply` / `Map`
  traversal families, current
  callable-wrapper support for `OperatorApplied` / `CurryApplied` /
  `ReverseApplied`, operator-form `Heads` parity for the `Apply` family,
  current `Operate[..., 0]` and `Operate[p, n]` parity, current
  post-transform evaluation parity for `Through` / `Operate`, current
  rectangular-list radius/template support with `Padding -> None`,
  `Padding -> "Fixed"`, `Padding -> "Periodic"`, constant-padding forms,
  cyclic padding lists, matching per-axis asymmetric padding pairs, and
  explicit even-template anchoring for `ArrayFilter`, and current
  dimension-spec plus grouped-dimension support for `ArrayReduce`, current
  dense same-head `Array` / `Outer` generation subset, and current `Select`
  expression / association / operator-form subset
- `planned`: sparse-array parity, broader ArrayPad-style padding / storage
  compatibility, and obscure higher-order traversal, threading diagnostic, or
  operator/combinator convenience corners beyond the current mainline subset

### Functions and Symbols

- `implemented`: `Function`
- `partial`: `Thread`, `BlockMap`, `SubsetMap`, `Comap`, `ComapApply`,
  `Accumulate`, `MapIndexed`, `Map`, `MapAll`, `MapAt`, `Scan`, `MapThread`,
  `TakeWhile`, `LengthWhile`, `Inner`, `Fold`, `FoldList`, `FoldPair`,
  `FoldWhile`, `SequenceFold`, `SequenceFoldList`, `Nest`, `NestWhile`,
  `NestList`, `NestWhileList`, `FixedPoint`, `FixedPointList`, `Apply`,
  `MapApply`, `OperatorApplied`, `CurryApplied`, `ReverseApplied`, `Through`,
  `Operate`, `Array`, `Outer`, `Select`, `ArrayFilter`, `ArrayReduce`,
  `Composition`, `RightComposition`
- `planned`: `ArrayFilter` / `ArrayReduce` sparse-array parity, any future
  broader ArrayPad-style padding or storage compatibility layers beyond the
  current dense-array subset, and lower-priority diagnostic / option-shape
  polish for rarely used functional-combinator corners
- `cross-tracked elsewhere`: `Append`, `AssociationMap`, `Cases`,
  `CountsBy`, `GroupBy`, `Identity`, `KeySortBy`, `Position`

## CL-05: Associations and Keyed Data

### Source Pages

- `Associations`

### Mechanisms

- `partial`: named-slot syntax `#name` exists generically, and the current
  first-pass association subset now includes association syntax/lowering,
  key-value canonicalization, current `AssociationQ` arity diagnostics,
  keyed part specifications via `Key`,
  deterministic ordering for canonical association rendering, key-based
  pattern matching through `KeyValuePattern` over associations and rule-lists
  including rule-vs-delayed-rule sensitivity and current non-unary inert
  boundaries, richer `Lookup`
  threading/multi-key behavior including `Lookup[..., default, h]` plus
  current Wolfram-style key-list threading, found-value-only wrapper
  application, single-rule/rule-list parity, current arity diagnostics plus
  singleton operator-form boundary, and keyed
  `Missing["KeyAbsent", key]` fallbacks, rule-list plus wrapper/threading
  support, delayed-entry `Values` wrapper boundary except the current
  `Unevaluated` wrapper escape, and arity plus invalid-input diagnostic parity
  for `Keys` / `Values`,
  rule-list-aware keyed predicates with current operator-form and
  arity-diagnostic parity through `KeyExistsQ` / `KeyMemberQ` / `KeyFreeQ`
  plus current `KeyMemberQ` / `KeyFreeQ` rule-list short-circuit invalid-rule
  boundaries,
  spec-ordered and threaded rule-list-normalizing `KeyTake` /
  `KeyDrop` with current arity, invalid-input diagnostic, and empty-rule-list
  parity, broader
  keyed-helper operator-form and arity-diagnostic parity across `KeyMap` /
  `KeyValueMap` / `KeySelect` / `KeySort` / `KeySortBy` /
  `AssociationMap` / `Merge` / `CountsBy` / `GroupBy`,
  current association-entry application and invalid mapped-entry diagnostics
  for `AssociationMap`,
  rule-list rejection parity for `KeyMap` / `KeyValueMap`,
  rule-list-normalizing `KeySelect` / `KeySort` / `KeySortBy`, mixed
  association/rule-list `Merge` including nested keyed collection groups plus
  more precise invalid-item diagnostics and current combiner-controlled
  delayed-value consumption,
  current list-form `KeyUnion` / `KeyIntersection` / `KeyComplement`
  semantics over association, rule, and rule-list keyed collections,
  including `KeyUnion[..., f]` fill behavior and current `KeyComplement`
  option-tail parity, current `AssociationThread` rule plus scalar/list
  broadcasting and singleton/list threading with delayed-rule preservation,
  deep `Association[...]` construction plus current recursive `Normal`
  association conversion for default, `Association`, `All`, and
  nonnegative-integer targets,
  ordinary-expression passthrough, and current `RuleDelayed` head preservation,
  `AssociateTo` update specs over nested rule
  lists / associations with current delayed-entry preservation plus current
  mutation arity diagnostics, callable
  `Key[key][assoc]` extraction plus current `Key` arity / invalid-application
  diagnostics, callable
  `Association[...][key, ...]` nested lookup, `Lookup[key]` operator form, current
  `Part[..., Key[...]]` plus
  numeric/range association projection including missing-key propagation,
  current `Extract::keyw` missing-key diagnostics, and no-mixed-key/index
  diagnostics, `Key[...]` key-spec unwrapping and current `RuleDelayed`
  preservation for the current `KeyTake` / `KeyDrop` / `KeyDropFrom` plus
  `KeyMap` / `KeySelect` / `KeySort` / `KeySortBy` keyed-transform subset
  plus `KeyUnion` / `KeyIntersection` / `KeyComplement`,
  custom `KeySort` comparers,
  `Catenate` over lists plus association values with current delayed
  association-value consumption, direct-association invalid-value subset
  parity, arity diagnostics, and invalid-value host-input resume coverage,
  current `Counts` over lists plus association values with explicit key-list
  count specs including duplicate, nested, rule-key, and empty-input edges plus
  delayed association-value consumption, current
  `GroupBy[list, f, g]` reducer support plus `key -> value` transform specs
  and nested spec-list grouping over lists and associations, association
  grouping, delayed association-value key evaluation with preserved grouped
  entries, and reducer invocation, current `PositionIndex` over lists plus
  association values and arity / invalid-input diagnostics,
  basic `Missing[]` / `Missing["reason"]` / `Missing["reason", expr]`
  forms plus current keyed-lookup propagation and current `DeleteMissing`
  over list / association / rule-list data including depth, third-argument
  scan-depth, `All`/`Infinity` depth aliases, eager delayed target-level deletion, delayed predicate
  suspension/resume, deeper target-level direct association-value deletion,
  invalid-argument diagnostics, and delayed nested-cleanup subsets, and held
  symbol / compound-lvalue plus exact part-target mutation first passes
  including scalar, list, `All`, span, deep `Key[...]`, mixed integer/key, and
  current `AssociateTo` bulk nested selectors, current `AssociateTo`
  missing-key and missing-part diagnostics, current non-symbol / invalid
  literal-target diagnostics, and current non-leaf / empty / bulk selector /
  missing-part `KeyDropFrom` part-target invalid boundaries for `AssociateTo` /
  `KeyDropFrom`
- `planned`: deferred evaluator-model cleanup for the current
  `Values[..., Unevaluated]` delayed-value wrapper escape, any remaining
  keyed-helper rule-list breadth outside the current
  `KeyMap` / `KeyValueMap` rejection and `KeySelect` / `KeySort` /
  `KeySortBy` normalizing subset, deeper
  niche `Catenate` invalid-message corners, deeper `DeleteMissing` edge
  parity beyond the current invalid-argument diagnostics, and deeper
  association semantics beyond the current first-pass subset

### Functions and Symbols

- `partial`: `Association`, `AssociationQ`, `Lookup`, `Key`, `Keys`, `Values`,
  `KeyExistsQ`, `KeyTake`, `KeyDrop`, `KeyMap`, `KeyValueMap`, `KeySelect`,
  `KeyMemberQ`, `KeyFreeQ`, `KeySort`, `KeySortBy`, `AssociationMap`,
  `AssociationThread`, `KeyUnion`, `KeyIntersection`, `KeyComplement`,
  `Normal`, `AssociateTo`, `KeyDropFrom`, `KeyValuePattern`, `Merge`,
  `Catenate`, `Counts`, `CountsBy`, `GroupBy`, `PositionIndex`, `Missing`,
  `DeleteMissing`
- `planned`: deferred evaluator-model cleanup for the current
  `Values[..., Unevaluated]` delayed-value wrapper gap, any remaining
  keyed-helper rule-list compatibility, niche `Catenate`
  invalid-message corners, deeper `DeleteMissing` edge parity beyond the
  current invalid-argument diagnostics, and deeper association semantics beyond
  the current first-pass subset
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
  `TerminatedEvaluation`, `InputString`, `Input`, `Print`, `Pause`, and
  `BlockRandom`.
  Recent follow-up work tightened protected-abort catching, session-config
  assertion enablement, `On[Assert]` / `Off[Assert]` controls including
  Wolfram-style extra-argument global switch forms, inert disabled
  `Assert`, oracle-aligned enabled-`Assert` failure message tags/text,
  no-test and extra-argument `Assert` call handling, plus the current
  `$AssertFunction` callback hook including non-boolean-test diagnostics,
  `None` as the no-callback sentinel, and discarded callback return values,
  exact-key `Success[...]` / `Failure[...]` property
  extraction with the current `Missing` propagation subset, including current
  `Failure[...]` `All` / `"Properties"` / `"Message"` / `"StyledMessage"`
  parity for string-or-symbol tags plus string and `MessageName[...]`
  `MessageTemplate` / `MessageParameters` message rendering through the
  current message catalog, including named and delayed `MessageParameters`,
  delayed-parameter host-input resume, and delayed-template host-input resume,
  valid-template-only virtual
  `HeldMessageTemplate` / `MessageName`, plus direct virtual `"Tag"` projection
  and malformed status-object inert projection boundaries, silent malformed
  `Failure` / `Success` constructors, `FailureQ` recognition for malformed
  `Failure[...]`, immediate-rule plus single-`RuleDelayed` `Failure` details normalization,
  association-only `Success` details projection, request-level print transport
  with input-like non-string fragment rendering, non-string `InputString`
  prompts, `Input` / `InputString`
  `FieldMasked -> True` plus request-kind host request metadata, resumed
  `Input` parse diagnostics, `Pause::numnm`, and current option-error
  boundaries including option-list non-option offender shape,
  `Enclose` handler and `Confirm` information-expression host-input
  suspension/resume, broader `ConfirmBy` / `ConfirmMatch` /
  `ConfirmQuiet` / `ConfirmAssert` host-input resume coverage, and
  oracle-aligned generated `Confirm*` failure metadata for the current
  `MessageTemplate` / `MessageParameters` / `ConfirmationType` slice,
  including `$Canceled` and all `Missing[...]` confirmation handling while
  keeping `FailureQ[$Canceled]` false, plus `Confirm[Failure[...]]`
  wrapping into `ConfirmationFailed` with the source failure message,
  including delayed source `MessageTemplate` host-input resume,
  suspension-safe `WithCleanup` body cleanup plus current
  cleanup-time host-input resume, throw/abort cleanup boundaries,
  primary-abort precedence over cleanup-time `Throw`,
  oracle-aligned delayed `Failsafe` failure `MessageParameters` metadata,
  cleanup-`Return` preservation of the primary outcome, and nested
  `AbortProtect` pending-abort propagation, current `CheckAbort` option-list
  and multi-option `PropagateAborts` parity, including full option-list
  validation before body evaluation, `Rule` vs `RuleDelayed` option RHS
  handling with host-input resume, and fallback host-input resume, `Failsafe`
  true-only test gating plus inert non-wrapper arity,
  oracle-aligned Failsafe failure metadata, including multi-argument
  `HoldForm[...]` message-parameter grouping, generated Failsafe failure
  `Message` rendering for held arguments, function/fail-function host-input resume,
  inert direct `TerminatedEvaluation` constructor / application behavior plus
  generated top-level/request/evaluation-data `TerminatedEvaluation` for
  recursion, fixed-point iteration, and current host resource limits,
  current string-tag `Success`
  projection boundary, active-throw cleanup `Throw` bypass for the matching
  `Catch` frame plus request-level cleanup/primary throw reporting, top-level
  and cleanup-time `Throw::nocatch` diagnostics with cleanup-before-primary
  ordering plus explicit `Throw[..., None]` spelling, top-level
  `Break::nofwd` / `Continue::nofwd` diagnostics, and abort/failsafe resume
  through host input; treat richer held/cleanup corners and deeper status
  parity as non-blocking long-tail rather than active mainline blockers
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
  `BlockRandom`, `InputString`, `Input`, `Print`, `Pause`, `$Aborted`,
  `$Canceled`, and `$Failed` in the current first-pass subset
- `planned`: non-blocking long-tail items include remaining deeper
  `WithCleanup` / `Failsafe` / `CheckAbort` / `AbortProtect` parity, richer
  `Assert` controls beyond the current session
  flag plus `$AssertFunction` hook, broader generated-resource
  `TerminatedEvaluation` parity beyond the current recursion/fixed-point,
  generated-element, integer-work, and iteration-work limit subset, and fuller
  host-I/O semantics
- `cross-tracked elsewhere`: `Block`, `Function`, `Module`, `Rule`,
  `RuleDelayed`, `With`

## CL-07: String Language

### Source Pages

- `String Manipulation`

### Mechanisms

- `implemented`: canonical string atoms, Unicode code-point semantics, parser
  escape support, exact-string construction and transformation core
- `implemented`: `ToString` and `TextString` now share a current input-like
  textual subset for common arithmetic and structural expressions; the
  recognized form-family includes `InputForm` / `OutputForm` /
  `StandardForm` / `TraditionalForm` / `FullForm`, and the renderer now
  covers slots, one-argument pure functions, assignment/replacement syntax,
  current symbolic pattern notation including `PatternTest` / `Optional` and
  one-argument `Repeated` / `RepeatedNull`, infix `StringExpression` (`~~`),
  plus `Part` / `Map` / `Apply` / `MapApply` syntax; the current
  byte-array/hash/compress/uncompress/encrypt/decrypt/nearest/pluralization
  layer is also mainline-complete for the project-owned subset
- `partial`: fuller language-data-backed pluralization, broader crypto
  envelope interop, and unicode-heavy golden coverage remain non-blocking
  polish rather than active mainline blockers

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
  `StringToByteArray`, `Hash`, `Compress`, `Uncompress`, `Encrypt`,
  `Decrypt`, `Nearest`, `Pluralize`
- `partial richer string-pattern parity`: `StartOfString`, `EndOfString`,
  `Repeated`, `RepeatedNull`, bounded repetition, `Shortest`, `Longest`,
  `Except`, `LetterCharacter`, `DigitCharacter`, `WordCharacter`,
  `WhitespaceCharacter`, `RegularExpression[...]` interop, `_` / `__` /
  `___`, named `Pattern[x, ...]` capture bindings, current `Rule` /
  `RuleDelayed` substitution for string replace/case rules, repeated named
  capture consistency across predicate/query paths, and current
  zero-length/match-edge handling
- `partial richer nearest/pluralize parity`: `Nearest` now supports the
  current `DistanceFunction -> EditDistance|HammingDistance` subset over
  strings plus a current `{resultSpec, radius}` / `{resultSpec, Infinity}`
  form, singleton `Rule[key, value]` datasets, and `Pluralize` now covers a
  wider case-preserving irregular/common-ending English subset plus current
  phrase-level pluralization of the final English word in simple multiword
  strings
- `partial deeper byte-array/crypto compatibility`: `Hash`, `Compress`, and
  `Encrypt` now consume `ByteArray[...]` payloads through the current base64
  transport subset, `Encrypt` also accepts `ByteArray[...]` keys rather than
  only textual expression forms, and `Decrypt` now round-trips the current
  encrypted-object subset for strings, byte arrays, and current parseable
  input-form expression payloads
- `partial association-aware nearest parity`: `Nearest` now also accepts
  `Association[...]` datasets in addition to rule lists for the current
  numeric and string-distance subsets
- `planned`: deeper nearest-neighbor semantics, fuller
  compression/encryption compatibility, and fuller linguistic-data-backed
  pluralization parity

## CL-08: Package, Namespace, Options, and Protection Tooling

### Source Pages

- `Package Development`

### Mechanisms

- `implemented`: contexts, context-path management, package load tracking,
  loader rollback on parse/evaluation/resource failure, message definition
  storage, message selection, definition/message inspection
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
  non-capturing-group regex subset, plus the current default option surface
  and `IgnoreCase -> True` matching for wildcard and supported regex
  patterns; `Save` now rejects unknown options in
  the current package-tooling subset, and `Once` now accepts option rules in
  second position, reports unknown option names, and honors
  `PersistenceTime -> 0` as a no-cache subset;
  `SyntaxInformation` now covers direct query, assignment, `Unset`,
  system-default metadata for the current package/tooling subset plus a
  broader current slice of optioned string/traversal/package helpers,
  `Names` option metadata, `Protect` / `Unprotect` argument shapes,
  `Protected` barriers, and `Block` / loader snapshot participation;
  `DeclarePackage` now installs stub-like declared symbols, updates
  `$ContextPath`, and auto-`Needs` the declared context on first symbol use;
  `Save` / `DumpSave` now provide a textual first-pass serialization layer
  that can round-trip through `Get`, with current `IncludedContexts` /
  `ExcludedContexts` filtering and `DumpSave` `SymbolAttributes` handling,
  and `Once` now provides session-local memoization keyed by held expression
  plus location; package-tooling regressions cover `Save` / `DumpSave`
  object selection through string and current regex names patterns;
  `ArgumentsOptions` / `CheckArguments` now cover the current held-call and
  trailing-options subset, and direct builtin `UpSet` / `UpSetDelayed` now
  cover first-pass inferred upvalue tags
- `planned`: non-blocking long-tail package ergonomics, deeper
  `DeclarePackage` / `Save` / `DumpSave` / `Once` compatibility, richer
  `Names` regex parity beyond the current wildcard/grouping/ignore-case
  subset, and broader `SyntaxInformation` properties

### Functions and Symbols

- `implemented`: `$ContextPath`, `$Packages`, `Begin`, `BeginPackage`, `End`,
  `EndPackage`, `Needs`, `Message`, `MessageName`
- `partial`: `Protect`, `Unprotect`, `Protected`, `Options`, `SetOptions`,
  `OptionValue`, `OptionsPattern`, `Names`, `SyntaxInformation`,
  `ArgumentsOptions`, `CheckArguments`, `UpSet`, `UpSetDelayed`,
  `DeclarePackage`, `DumpSave`, `Once`, `Save`
- `planned`: long-tail `DeclarePackage` / `Save` / `DumpSave` / `Once`
  parity, richer `Names` regex coverage beyond the current wildcard /
  bounded / alternation / ignore-case subset, and broader
  `SyntaxInformation` properties
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
  layout-aware structural memory-proxy `MemoryInUse[]` subset, including
  removed-symbol identity / generation state in the current session-state
  proxy model, `Monitor[...]` hooks with current `EvaluationMonitor` /
  `StepMonitor` option symbols plus stable initial/final display refreshes,
  broader default-option coverage, and
  explicit `None` callback overrides, a first-pass `Dialog[...]` subset with
  `Return`-sensitive evaluation boundaries plus current `DialogProlog` /
  `DialogSymbols` handling and host-input resume support,
  `EvaluationData[...]` property projection through current string-or-symbol,
  list, and `All` forms plus `"Properties"` with matching structured `"Trace"`,
  held `"Input"`, and held `"MessagesExpressions"` payloads, including
  list-projection coverage for held message-expression / message-text payloads,
  request-shaped `Missing["KeyAbsent", ...]` payloads, and current
  input-resume coverage, and
  first-pass `TestCreate[...]` / `TestReport[...]`
  metadata through
  `TestID`, `SameTest`, and expected-message matching with current
  `TestReport[..., prop]` string-or-symbol projection plus report-level
  `"Success"`, held per-test `"Input"`, current merging of
  `{report_1, report_2, ...}`, broader per-test `ActualMessagesText` /
  `ActualMessagesExpressions` / `OutputLog` / `Timing` payloads, current
  merged-report property projection, and current report-level `SameTest` /
  `TimeConstraint` / `MemoryConstraint` options, alongside `Stack[]`; richer
  verification, monitoring, and resource-accounting parity is still
  pending even though the internal tracing/message
  foundations are already in place
- `planned`: optional allocator-level real-memory accounting and obscure
  trace/report edge-case parity if concrete downstream demand appears

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
  for filtered `Trace[...]` / `TracePrint[...]` plus explicit override
  coverage, depth-aware nested branch shaping, an indented structured
  `TracePrint[...]` subset, current
  layout-aware structural byte/memory accounting with payload-aware
  `ByteArray[...]` handling and removed-symbol identity / generation-state
  session accounting, current symbol-or-string report/property projection
  with request-shaped `Missing["KeyAbsent", ...]` payloads, held
  message-expression payloads, current report merging plus
  current report-level `SameTest` / `TimeConstraint` / `MemoryConstraint`
  options, broader per-test `ActualMessagesText` /
  `ActualMessagesExpressions` / `OutputLog` / `Timing` payloads, current
  merged-report property projection, current input-resume coverage for
  `EvaluationData` / `Monitor`, and a first-pass `Dialog[...]` / `Dialog[]`
  one-shot subset with current
  `DialogProlog` / `DialogSymbols` handling; treat any remaining deeper
  allocator-level real-memory accounting or obscure trace/report edge-case
  parity as future demand-driven work rather than active mainline blockers
- `planned`: optional allocator-level real-memory accounting and obscure
  trace/report edge-case parity if concrete downstream demand appears
- `cross-tracked elsewhere`: `AbortProtect`, `Check`, `CheckAbort`,
  `Failure`, `Message`, `Missing`, `PropagateAborts`, `Quiet`

## CL-Y: `mma-hol` Real-Project Pressure Lane

### Audit Snapshot

Last scanned: 2026-06-11.

Local source snapshot:

- repository: `/Users/fft/Developments/mma-hol`
- fixed probe clone: `/private/tmp/rum-mma-hol-latest.FtARYw/mma-hol`
- clean probe clone: `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol`
- HEAD: `698e3d0` (`M7-7 (Real) Complete.wl: Dedekind completeness +
  Archimedean + Q-density`)
- worktree note: the live sibling repository was clean at clone time and has
  no configured remote; the clean `/private/tmp` clone is the current semantic
  smoke target, while the earlier `latest` clone only records historical
  probe-only normalizations
- scanned scope: tracked `*.wl` / `*.wls` source under the project; the current
  run order includes Real `Cut`, `RatAux`, `Field`, `Mul`, `Inv`, and
  `Complete`

Primary acceptance candidate:

- `tests/run_all.wls` should become the first cold real-project smoke lane.
  It loads core packages, automation packages, stdlib packages, and the current
  Real frontier files `Cut.wl`, `RatAux.wl`, `Field.wl`, `Mul.wl`, `Inv.wl`,
  and `Complete.wl`.
- `scripts/mma-hol-bounded-smoke.sh` is the `rum`-side bounded diagnostic
  driver for this lane. It runs package-load and test-file probes with
  per-stage timeouts and captured logs, but its elapsed times are not
  acceptance benchmarks because load probes replay prefixes in fresh sessions.

Secondary harness candidates:

- `tests/build_snapshot.wls`, `tests/run_fast.wls`, and
  `tests/extend_snapshot.wls` exercise snapshot-style development flow via
  `DumpSave`, `Get`, `Contexts[]`, file timestamps, and context-path restore.
  Treat these as a required developer-workflow lane for an eventual
  `mma-hol`-to-`rum-hol` migration, even though the cold `run_all.wls` lane
  remains the semantic baseline.
- `tests/lint_private.wls` is useful but not a kernel acceptance blocker; it
  adds `Import[..., "Text"]`, `Reap`, and `Sow` pressure.

Strict/stable snapshot boundary:

- `mma-hol` now distinguishes strict and stable kernel encapsulation. Strict
  mode keeps kernel state behind `Module` / `Unique` generated symbols and is
  deliberately not a cross-session `DumpSave` restore target.
- Stable mode uses fixed `HOL` private-context symbols for kernel state. That
  is the only mode `rum` should target for snapshot-backed `mma-hol` runs.
- `rum` already has a first-pass textual `Save` / `DumpSave` / `Get` layer that
  can round-trip simple definitions, attributes, options, syntax information,
  messages, and name-pattern/context selections. It is not Wolfram `.mx` binary
  compatibility and should be treated as a `rum` snapshot format.
- The stable snapshot helper surface now has first-pass support: `Contexts[]`,
  file existence/date/size helpers, `AbsoluteTime`, `Round`, recursive
  `FileNames[..., ..., Infinity]`, string wildcard filters such as `"HOL`*"`,
  context-string `DumpSave`, and restore-time `$ContextPath` / `$Packages`
  session-field assignment. The full `tests/build_snapshot.wls` /
  `tests/run_fast.wls` scripts still need validation against the current
  `mma-hol` tree before they can be treated as the practical development loop
  for `rum-hol`.
- The current stable snapshot gap audit lives in
  [docs/mma-hol-smoke-and-snapshot.md](./mma-hol-smoke-and-snapshot.md).

### Required Language Shape

The current `mma-hol` tree needs the following host-language clusters:

- package/script loading: `BeginPackage`, `Begin`, `End`, `EndPackage`,
  `Needs`, `Get`, `$Path`, `$InputFileName`, `FileNameJoin`,
  `DirectoryName`, `ParentDirectory`, `FileNames`, `FileNameTake`, and `Exit`
- lexical and dynamic state: heavy `Module`, closure capture through private
  gensyms, `Unique`, `With`, `Block`, `SetAttributes`, and `ValueQ`
- rules and patterns: named blanks, typed blanks, blank sequences,
  `Condition`, rule and delayed-rule replacement, `MatchQ`, `Cases` with
  levelspecs, and list-of-rule substitution flows
- association-backed bookkeeping: `Association`, `AssociationQ`, `Lookup`,
  `KeyExistsQ`, `KeyDrop`, `KeySelect`, `KeySort`, `KeyValueMap`, `Keys`,
  `Values`, `Merge`, `AssociateTo`, `Normal`, `Missing`, and `MissingQ`
- procedural and functional control: `If`, `Which`, `Switch`, `Do`, `Table`,
  `While`, `Break`, `Return`, `Catch`, `Throw`, `Map`, `Apply`, `MapThread`,
  `Fold`, `Nest`, `Scan`, `Select`, `SelectFirst`, `AllTrue`, `AnyTrue`, and
  `Outer`. The latest probe specifically requires `Table` to substitute
  iterator values into held subexpressions such as the lhs of `Rule`.
- exact host-level arithmetic for automation: exact integers/rationals,
  `Plus`, `Times`, `Subtract`, `Divide`, `Power`, `Mod`, `Quotient`, `GCD`,
  `LCM`, and the small aggregation helpers listed below
- list/set plumbing: `Append`, `AppendTo`, `Prepend`, `Join`, `Flatten`,
  `Reverse`, `Transpose`, `Union`, `Complement`, `Delete`, `DeleteCases`,
  `DeleteDuplicates`, `DeleteDuplicatesBy`, `Sort`, `SortBy`, `Range`,
  `Take`, `Drop`, `Most`, `Rest`, `First`, `Last`, and `Length`
- string/tokenizer helpers: `Characters`, `StringLength`, `StringTake`,
  `StringJoin`, `StringContainsQ`, `StringMatchQ`, `StringPosition`,
  `StringRiffle`, `StringSplit`, `StringStartsQ`, `DigitQ`, `LetterQ`,
  `Whitespace`, and `ToString`
- diagnostics and test output: `Failure`, `FailureQ`, `Stack[]`, `Print`,
  text-only `Row`, `AbsoluteTime`, and `Round`
- source compatibility: ordinary source identifiers may use Wolfram-style
  Unicode letters and HOL glyph-like names; this is parser/source compatibility
  pressure, not a request for notebook front-end rendering

### Newly Promoted Or Reconfirmed Blockers

These are the concrete `CL-Y` items that should be checked before declaring
the `mma-hol` lane healthy:

- `Total`: first-pass exact/list support is implemented and now lets
  `Merge[..., Total]` reduce grouped exact numeric coefficients instead of
  preserving `Total[List[...]]` as an inert head.
- `Denominator`: first-pass exact integer/rational support is implemented for
  the `Denominator /@ Values[prov]` shape used before Farkas-certificate
  `LCM`.
- `Apply[LCM|GCD, list]`: covered by runtime regression in the actual
  `mma-hol` usage shape.
- `/@` map syntax and Unicode letter identifiers: first-pass source support is
  implemented for ordinary map forms and ordinary Unicode letter/digit symbols
  such as Greek local names used by `mma-hol`; escaped Wolfram box/name syntax
  is still outside the current source subset. Assignment RHS forms such as
  `ks = #[[1]] & /@ args` and rule-function bodies such as
  `Map[f[#[[1]]] -> #[[2]] &, pairs]` are covered.
- pattern syntax sugar: explicit `Alternatives[...]` and `PatternTest[...]`
  are supported by the matcher, and the source shorthands `p1 | p2` and
  `_?pred` used by latest `mma-hol` sources now lower to those forms.
- `Contexts[]`, `DumpSave`, `$ContextPath`, and `$Packages`: these are not
  needed for the first cold `run_all.wls` semantic smoke, but are required for
  the stable-mode snapshot-backed `run_fast.wls` workflow and for practical
  `rum-hol` development. `Contexts[]` now enumerates first-pass session
  contexts, `DumpSave[file, Evaluate[contexts]]` accepts context strings and
  round-trips a tiny public/private context tree, and `$ContextPath` /
  `$Packages` assignment mutates session fields. The remaining work is full
  stable-script validation against the current `mma-hol` tree.
- script/package source compatibility: initial shebang lines and multiline
  comments are handled by the loader splitter, infix logical/comparison syntax
  lowers to the existing boolean/comparison builtins, `:` named-pattern sugar,
  `Function`-postfix map forms such as `f[#] & /@ xs`, and string-join `<>`
  are implemented for the current `mma-hol` sources.
- package public-symbol visibility: usage messages now make package public
  symbols visible while evaluating private context definitions, so private
  implementation code binds exported definitions to the intended public names.
- association key selectors: string keys can be used as `Part` selectors for
  association-backed payloads, including `Failure[..., assoc][[2, "key"]]`
  harness shapes.
- association and list assignment shapes: evaluated association keys,
  association key lvalues such as `a[k] = v` / `a[[Key[k]]] = v`, and list
  destructuring assignment such as `{x, y} = rhs` are implemented for the
  current `Types.wl` / `Terms.wl` usage.
- definition lhs and `Return` boundaries: definition lhs call heads are
  normalized through own values before rule storage, which covers the strict
  kernel's private theorem-tag patterns, and `Return[...]` is consumed at
  definition application boundaries instead of leaking to callers.
- `FileExistsQ`, `FileDate`, `FileByteCount`, `AbsoluteTime`, and `Round`:
  first-pass harness utilities for snapshot freshness and timing reports;
  simple decimal timing literals lower to exact rationals. Keep this scoped to
  script/harness support rather than general system monitoring or full
  machine-real semantics.
- `Row`, `Import[..., "Text"]`, `Reap`, and `Sow`: first-pass lint and harness
  glue exists, but the private-context lint script remains below the cold run
  and snapshot lanes in priority.
- clean-HEAD source smoke: using a clean archive of `mma-hol` HEAD `83d768f`,
  the current runtime loads `ErrorUtil.wl`, `Types.wl`, `Terms.wl`,
  `Kernel.wl`, and `tests/harness.wl` in sequence. `errorutil_tests.wl`
  reports `List[10, 0]`; `types_tests.wl` reports `List[36, 0]`;
  `terms_tests.wl` reports `List[67, 0]`; `kernel_tests.wl` reports
  `List[71, 0]`; `bootstrap_tests.wl` reports `List[45, 0]`; and
  `equal_tests.wl` reports `List[17, 0]`. The next observed blocker is the Bool
  layer: after fixing definition-boundary `Return`, `GEN[x, REFL[x]]` reaches
  the complex `BETACONV` / kernel `BETA` path and still needs bounded profiling
  around deep term traversal. The full `run_all.wls` path still needs a bounded
  smoke driver before it can be a routine acceptance check.
- latest clean-clone smoke: using `mma-hol` HEAD `698e3d0`, the core through
  parser/printer lane is green without source normalizations:
  `kernel_tests.wl` reports `List[71, 0]`, dependency-preloaded
  `bool_tests.wl` reports `List[73, 0]` with cold release external wall time
  about `22.95s`, `drule_tests.wl` reports `List[43, 0]` in about `4.18s`,
  `tactics_tests.wl` reports `List[49, 0]` in about `5.18s`, and
  `printer_tests.wl` plus `parser_tests.wl` report `List[126, 0]` in about
  `4.62s`.
- evaluator compatibility promoted by the latest probe and now covered in
  `rum`: definition lhs non-pattern constants evaluate through own values
  before rule storage (`p = a; f[p] := ...` defines `f[a]`, as in Wolfram);
  `Table` iterator substitution reaches held `Rule` lhs positions; `Rule` and
  `RuleDelayed` evaluate their lhs enough for substitution workflows without
  evaluating pattern names; the cycle guard allows state-progressing recursive
  definitions such as `parseTypeExpr[]` and `parseAtom[]`; and ordinary
  expression pattern arguments such as `spec["kind"]` are evaluated before
  `MemberQ` / `MatchQ` compile them as exact patterns.
- next CL-Y frontier: the bounded cold `run_all.wls` probe has moved the lane
  past generic driver construction and into the `PropTaut` / `Simp`-backed
  stdlib packages. After the 2026-06-12 external representation pass and the
  same-bit stability-invalidation follow-up, `proptaut_tests.wl` is a green
  smoke signal at about `239s`, and the focused PropTaut distribution probe is
  about `12.73-12.88s` versus the local WolframKernel internal `3.80s`. The
  old `stdlib/Pair.wl` non-theorem failure was traced to rum's too-low default
  recursion limit, and the default now matches the local Wolfram baseline of
  `1024`. `Pair` still exceeds a `600s` load budget, so later stdlib package
  statuses cannot be trusted until that gate is cleared. Treat the current
  performance notes as evidence-backed next steps, not as permission for broad
  speculative evaluator rewrites.

### Non-Goals

`CL-Y` does not promote these areas by itself:

- full CAS or mathematical-analysis coverage in `rum`; `mma-hol` owns the HOL
  mathematics, while `rum` only needs to execute the Wolfram-style host code
- notebook UI, boxes, graphics, dynamic front-end behavior, or rich theorem
  rendering
- compatibility with temporary probe files, debug transcripts, or dirty
  one-off local experiments in the neighboring checkout
- broad file/network/system APIs beyond the small script and snapshot helpers
  explicitly used by the smoke lanes

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
  `Contexts`, `InputString`, `Level`, `MessageDefinitions`, `MessageTags`,
  `Messages`, `NormalizeRules`, `ReleaseHold`, `Row`, `SetAttributes`,
  `ClearAttributes`, `StringEndsQ`, `StringStartsQ`, `StringTrim`,
  `TagSetDelayed`, `TagUnset`, `$Context`, `$Failed`, `$InputFileName`,
  `$MessageList`, `$Path`, `File`, `FileNameJoin`, `DirectoryName`,
  `ParentDirectory`, `FileNames`, `FileNameTake`, `FileExistsQ`, `FileDate`,
  `FileByteCount`, `Import`, `AbsoluteTime`, `Round`, `Exit`, `Total`,
  `Denominator`, `Reap`, `Sow`
- `planned`: remaining real-project script/package glue promoted by `CL-Y`,
  currently including full stable snapshot script validation, private-context
  lint validation, and the Bool-layer
  `GEN` / `BETACONV` deep term traversal blocker exposed after the current
  core `types` / `terms` / `kernel` assertions were cleared

## Maintenance Notes

- Update this matrix before or alongside the implementation PR that changes a
  same-name item in the scoped surface.
- Keep extra helper builtins in `extension` unless they are explicitly named on
  the scoped official pages.
- When an item is intentionally subsetted, keep it in `partial` until the
  remaining gap is either implemented or documented as a deliberate divergence.
- Keep `TODO.md` aligned to the `CL-*` group IDs instead of re-listing every
  function name there.
