# Builtin Reference

Generated from the installed `rum` builtin registry. Do not edit by hand.

Builtin count: 609

Regenerate with `cargo run -p rum-stdlib --bin generate-builtin-reference`.

### $Aborted

Represents a computation aborted by Abort or an external interrupt.

- Attributes: `None`
- Capability: `Pure`

### $AssertFunction

Stores the optional assertion failure callback used by Assert in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### $Canceled

Represents a canceled operation for confirmation-flow compatibility.

- Attributes: `None`
- Capability: `Pure`

### $Context

Evaluates to the current default context name as a string.

- Attributes: `None`
- Capability: `Pure`

### $ContextPath

Evaluates to the active context search path as a list of strings.

- Attributes: `None`
- Capability: `Pure`

### $Failed

Represents a failed operation.

- Attributes: `None`
- Capability: `Pure`

### $InputFileName

Evaluates to the path of the file currently being loaded, or the empty string at top level.

- Attributes: `None`
- Capability: `Pure`

### $MessageList

Lists the message names emitted by the most recent top-level evaluation.

- Attributes: `None`
- Capability: `Pure`

### $Packages

Lists package contexts loaded into the current session.

- Attributes: `None`
- Capability: `Pure`

### $Path

Lists search roots used by the package loader.

- Attributes: `None`
- Capability: `Pure`

### Abort

Generates an abort that terminates the current evaluation unless caught.

- Attributes: `None`
- Capability: `Pure`

### AbortProtect

Evaluates an expression while delaying any abort until the protected region exits in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### Abs

Returns the absolute value of exact integers and rationals.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### AbsoluteTime

Returns the current exact rational Unix timestamp, or converts a valid UTC date list to exact seconds since 1970-01-01T00:00:00Z.

- Attributes: `None`
- Capability: `Clock`
- Limits: the timestamp contract is exact rational seconds since the Unix epoch 1970-01-01T00:00:00Z; date-list conversion uses UTC, the proleptic Gregorian calendar, and no leap seconds.

### AbsoluteTiming

Evaluates an expression and returns {elapsed, value} using the current wall-clock exact-rational subset.

- Attributes: `HoldAll`
- Capability: `Clock`

### Accumulate

Returns running totals in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### AddTo

Adds an evaluated increment to a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`
- Capability: `Pure`

### AlgebraicIntegerQ

Tests whether an expression is an exact integer in the current algebraic-integer subset.

- Attributes: `Listable`
- Capability: `Pure`

### AlignColumns

Formats a list of string rows as left-aligned columns separated by "Separator" -> "  " by default; any string separator can be supplied.

- Attributes: `None`
- Capability: `Pure`

### AllTrue

Tests whether all selected structural parts satisfy a criterion in the current boolean subset.

- Attributes: `None`
- Capability: `Pure`

### Alphabet

Returns the current English lowercase alphabet subset.

- Attributes: `None`
- Capability: `Pure`

### AlphabeticOrder

Compares two strings alphabetically in the current string-order subset.

- Attributes: `None`
- Capability: `Pure`

### AlphabeticSort

Sorts strings using the current English alphabet subset with simple diacritic folding.

- Attributes: `None`
- Capability: `Pure`

### Alphanumeric

Evaluates to the uppercase, lowercase, and digit character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### Alternatives

Represents the current pattern alternative head.

- Attributes: `Protected`
- Capability: `Pure`

### And

Evaluates a conjunction with current short-circuit boolean semantics.

- Attributes: `HoldAll`
- Capability: `Pure`

### AnyTrue

Tests whether any selected structural part satisfies a criterion in the current boolean subset.

- Attributes: `None`
- Capability: `Pure`

### Append

Appends an element to the end of a normal expression, with current operator-form support.

- Attributes: `None`
- Capability: `Pure`

### AppendTo

Appends an evaluated item to a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Apply

Replaces the head of an expression in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### ApplyTo

Applies an evaluated function to a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`
- Capability: `Pure`

### ArgumentsOptions

Partitions a held call into positional arguments and trailing option rules.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Array

Constructs a dense list-like array by applying a function to generated indices in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### ArrayFilter

Applies a function to local list-array neighborhoods in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### ArrayQ

Tests whether an expression is a rectangular List array, optionally matching its depth against a pattern and requiring every leaf to satisfy a test.

- Attributes: `None`
- Capability: `Pure`

### ArrayReduce

Reduces rectangular list arrays across selected dimensions in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Assert

Represents an assertion site and supports the current session flag plus $AssertFunction hook.

- Attributes: `HoldAll`
- Capability: `Pure`

### AssociateTo

Adds or replaces association entries, including held symbol mutation in the current first-pass subset.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Association

Builds an association from Rule/RuleDelayed entries in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### AssociationKeyDeflatten

Builds nested associations from flat associations whose keys are path lists.

- Attributes: `None`
- Capability: `Pure`

### AssociationKeyFlatten

Flattens nested associations into associations whose keys are path lists.

- Attributes: `None`
- Capability: `Pure`

### AssociationMap

Builds an association by mapping a function over keys in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### AssociationQ

Tests whether an expression is an Association with rule entries.

- Attributes: `None`
- Capability: `Pure`

### AssociationThread

Threads rule or key/value inputs into an association in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### AtomQ

Tests whether an expression is atomic rather than a normal expression.

- Attributes: `None`
- Capability: `Pure`

### Attributes

Lists the currently active attributes for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Automatic

Represents the current default-specifier symbol shared by several first-pass string and tooling subsets.

- Attributes: `None`
- Capability: `Pure`

### Begin

Switches into a context and prepends it to the context path.

- Attributes: `None`
- Capability: `Pure`

### BeginPackage

Starts a package context with a package-oriented context path.

- Attributes: `None`
- Capability: `Pure`

### BernoulliB

Computes exact Bernoulli numbers for non-negative integer indices.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### Between

Tests interval inclusion in the current exact numeric subset.

- Attributes: `None`
- Capability: `Pure`

### BinarySearch

Binary-searches an ascending List under rum's canonical order without validating sortedness, returning a 1-based position or Missing["NotFound"].

- Attributes: `None`
- Capability: `Pure`
- Limits: assumes the input List is sorted ascending under rum's canonical order and does not validate sortedness; incomparable values under that order are treated as not found.

### Binomial

Computes exact binomial coefficients for integer arguments.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### Blank

Represents the current single-element pattern atom; in string patterns it matches one character.

- Attributes: `Protected`
- Capability: `Pure`

### BlankNullSequence

Represents the current zero-or-more sequence pattern atom; in string patterns it matches zero or more characters.

- Attributes: `Protected`
- Capability: `Pure`

### BlankSequence

Represents the current one-or-more sequence pattern atom; in string patterns it matches one or more characters.

- Attributes: `Protected`
- Capability: `Pure`

### Block

Temporarily rebinds symbols dynamically.

- Attributes: `HoldAll`
- Capability: `Pure`

### BlockMap

Maps a function over contiguous blocks in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### BlockRandom

Evaluates an expression while restoring the deterministic session PRNG state afterward.

- Attributes: `HoldAll`
- Capability: `Pure`

### Boole

Converts exact True and False to 1 and 0, leaving non-boolean conditions unevaluated.

- Attributes: `Listable`
- Capability: `Pure`

### BooleanQ

Tests whether an expression evaluates exactly to True or False.

- Attributes: `None`
- Capability: `Pure`

### Break

Exits the nearest enclosing loop.

- Attributes: `None`
- Capability: `Pure`

### ByteArray

Represents raw byte payloads using base64 text in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### ByteArrayQ

Tests whether an expression is a ByteArray with a valid base64 payload.

- Attributes: `Protected`
- Capability: `Pure`

### ByteArrayToString

Converts a ByteArray to a string when its payload is valid UTF-8, failing closed otherwise.

- Attributes: `Protected`
- Capability: `Pure`

### ByteCount

Returns the current canonical-text byte count for an expression in the first-pass resource subset.

- Attributes: `None`
- Capability: `Pure`

### Capitalize

Uppercases the first character of a string or each string in a list.

- Attributes: `Listable`, `Protected`, `ReadProtected`
- Capability: `Pure`

### Cases

Collects subexpressions matching a pattern or rule.

- Attributes: `None`
- Capability: `Pure`

### CatalanNumber

Computes exact Catalan numbers for non-negative integer indices.

- Attributes: `Listable`, `NumericFunction`, `Protected`, `ReadProtected`
- Capability: `Pure`

### Catch

Evaluates an expression and intercepts matching Throw values.

- Attributes: `HoldAll`
- Capability: `Pure`

### Catenate

Concatenates lists and association values into a list in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Ceiling

Returns the least integer not less than an exact number.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### CharacterCounts

Counts code-point occurrences in a string and returns an association in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### CharacterRange

Builds an inclusive range of single-character strings between two endpoints.

- Attributes: `None`
- Capability: `Pure`

### Characters

Splits a string into a list of single-code-point strings.

- Attributes: `None`
- Capability: `Pure`

### Check

Evaluates an expression and substitutes a fallback if matching messages are emitted.

- Attributes: `HoldAll`
- Capability: `Pure`

### CheckAbort

Evaluates an expression and substitutes a fallback if an Abort occurs.

- Attributes: `HoldAll`
- Capability: `Pure`

### CheckArguments

Checks whether a held call has an allowed positional argument count and valid trailing options.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Clear

Clears values and rules from one or more symbols.

- Attributes: `HoldAll`
- Capability: `Pure`

### ClearAll

Clears values, rules, messages, and user-defined attributes from one or more symbols.

- Attributes: `HoldAll`
- Capability: `Pure`

### ClearAttributes

Removes attributes from one or more symbols.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Coefficient

Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.

- Attributes: `None`
- Capability: `Pure`

### CoefficientList

Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.

- Attributes: `None`
- Capability: `Pure`

### Comap

Applies selected parts of a function collection to a common expression in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### ComapApply

Applies selected parts of a function collection as heads of a common expression in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Commonest

Returns the most frequent elements of a list in first-occurrence order, including all ties.

- Attributes: `None`
- Capability: `Pure`

### Complement

Returns sorted unique elements from the first expression that are absent from later same-head expressions; SameTest keeps the first representative in structural scan order.

- Attributes: `None`
- Capability: `Pure`

### Composition

Represents a left composition of functions in the current first-pass subset.

- Attributes: `Flat`, `OneIdentity`
- Capability: `Pure`

### CompoundExpression

Evaluates expressions sequentially and returns the last result.

- Attributes: `HoldAll`
- Capability: `Pure`

### Compress

Encodes expressions into the current project-local compressed string envelope.

- Attributes: `None`
- Capability: `Pure`

### Confirm

Confirms that an expression does not evaluate to a failure-like result in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### ConfirmAssert

Confirms that a test evaluates to True in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### ConfirmBy

Confirms that a test function returns True for an expression in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### ConfirmMatch

Confirms that an expression matches a held pattern in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### ConfirmQuiet

Confirms that an evaluation emits no matching messages in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### ConformSchema

Validates a value against a symbolic schema and returns the normalized value with defaults filled.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`

### ConstantArray

Constructs a constant plain list for non-negative integer scalar and list dimensions.

- Attributes: `Protected`
- Capability: `Pure`

### Construct

Constructs an expression from a head and zero or more arguments.

- Attributes: `None`
- Capability: `Pure`

### ContainsAll

Tests whether a List contains every requested element, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### ContainsAny

Tests whether a List contains any requested element, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### ContainsExactly

Tests List multiset equality, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### ContainsNone

Tests whether a List contains none of the requested elements, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### ContainsOnly

Tests whether every element of a List belongs to the allowed-element List, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### Context

Returns the context of the current session or of a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Contexts

Lists known contexts in the current session.

- Attributes: `None`
- Capability: `Pure`

### Continue

Skips to the next iteration of the nearest enclosing loop.

- Attributes: `None`
- Capability: `Pure`

### CoprimeQ

Tests whether exact integers are pairwise relatively prime in the current integer subset.

- Attributes: `Listable`
- Capability: `Pure`

### Count

Counts subexpressions matching a pattern on levels specified by a standard levelspec.

- Attributes: `None`
- Capability: `Pure`

### Counts

Counts repeated elements and returns an association in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### CountsBy

Counts transformed elements and returns an association in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Crc32

Computes the IEEE CRC-32 checksum over a string's UTF-8 bytes or a ByteArray.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: computes the IEEE CRC-32 checksum for integrity and interoperability, not security; it is not collision-resistant.

### CurryApplied

Represents a left-to-right curried operator wrapper in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### D

Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.

- Attributes: `HoldRest`
- Capability: `Pure`

### DateDifference

Returns the exact fixed-duration difference between two timestamps or date lists.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports only fixed-duration units Seconds, Minutes, Hours, Days, and Weeks; calendar Months and Years are outside the folk date/time subset.

### DateList

Converts an exact Unix timestamp to a UTC date list, or reads the current UTC date list in the zero-argument form.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: uses UTC, the proleptic Gregorian calendar, and no leap seconds; the zero-argument current-time form requires Capability::Clock.

### DateObjectQ

Tests whether an expression has a DateObject head in the current structural subset.

- Attributes: `None`
- Capability: `Pure`

### DatePlus

Adds an exact fixed-duration offset to an exact timestamp or date list.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports only fixed-duration units Seconds, Minutes, Hours, Days, and Weeks; calendar Months and Years are outside the folk date/time subset.

### DateString

Formats an exact timestamp or date list as an ISO 8601 UTC string.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders ISO 8601 UTC only; non-integer seconds are rendered only when their exact rational value has a terminating decimal expansion, and the zero-argument current-time form requires Capability::Clock. The two-argument form DateString[date, fmt] accepts fmt "ISODateTime" (default, full UTC timestamp), "ISODate" (date only), or "ISOTime" (time only).

### DayOfWeek

Returns the ISO weekday symbol for an exact timestamp or date list.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: the two-argument form DayOfWeek[date, sel] accepts sel "Name" (default weekday symbol) or "ISOInteger" (Monday=1 through Sunday=7).

### DaysInMonth

Returns the number of days in a proleptic Gregorian month.

- Attributes: `Protected`
- Capability: `Pure`

### Decapitalize

Lowercases the first character of a string or each string in a list.

- Attributes: `Listable`, `Protected`, `ReadProtected`
- Capability: `Pure`

### DecimalString

Renders an exact number as a plain decimal string with exactly n fractional digits.

- Attributes: `None`
- Capability: `Pure`
- Limits: uses exact integer/rational arithmetic only, rounds half away from zero, and always emits exactly n fractional digits.

### DeclarePackage

Declares stub names for a package context and prepends that context to the search path.

- Attributes: `None`
- Capability: `Pure`

### Decrement

Decrements a held symbol or exact Part target and returns its previous value.

- Attributes: `HoldFirst`
- Capability: `Pure`

### DefaultValues

Lists stored Default[...] definitions for a symbol in the current optional-pattern subset.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Definition

Returns a held first-pass expression containing visible definitions for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Delete

Deletes one or more top-level structural positions in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### DeleteCases

Deletes subexpressions matching a pattern on levels specified by a standard levelspec.

- Attributes: `None`
- Capability: `Pure`

### DeleteDuplicates

Keeps the first occurrence of structurally equal top-level elements, or of elements equivalent under a custom binary test.

- Attributes: `None`
- Capability: `Pure`
- Limits: the custom binary-test form performs pairwise O(n^2) representative comparisons and keeps the first representative.

### DeleteDuplicatesBy

Keeps the first top-level element for each evaluated key in the current unary-function subset.

- Attributes: `None`
- Capability: `Pure`

### DeleteMissing

Deletes Missing entries from lists, associations, and rule-list data in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Denominator

Returns the denominator of exact integers and rationals.

- Attributes: `Listable`
- Capability: `Pure`

### Depth

Returns the structural depth of an expression, optionally including heads.

- Attributes: `None`
- Capability: `Pure`

### Dialog

Runs the current first-pass dialog subset with Return-sensitive evaluation and host-input suspension.

- Attributes: `HoldAll`
- Capability: `Pure`

### Diff

Builds a simple DiffObject for strings or lists in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### DiffObject

Represents the current diff result object returned by Diff.

- Attributes: `None`
- Capability: `Pure`

### Differences

Returns successive differences for plain lists, including the current non-negative order subset.

- Attributes: `NonThreadable`, `Protected`
- Capability: `Pure`

### DigitCharacter

Represents the current string-pattern atom for one ASCII decimal digit.

- Attributes: `Protected`
- Capability: `Pure`

### DigitCharacters

Evaluates to the ASCII digit character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### DigitCount

Counts digits in the absolute value of an exact integer.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`

### DigitQ

Tests whether all characters in a string belong to the current digit subset.

- Attributes: `None`
- Capability: `Pure`

### DirectoryName

Returns a parent directory path string for a file name.

- Attributes: `None`
- Capability: `Pure`

### DisjointQ

Tests whether two Lists share no element, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### Divide

Divides exact numbers, producing exact rationals when possible.

- Attributes: `None`
- Capability: `Pure`

### DivideBy

Divides a held symbol or exact Part target by an evaluated divisor and stores the result.

- Attributes: `HoldFirst`
- Capability: `Pure`

### DivisorSigma

Computes exact divisor power sums for exact integer arguments.

- Attributes: `Listable`, `NHoldAll`, `Protected`
- Capability: `Pure`

### Divisors

Returns the sorted positive divisors of an exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### Do

Evaluates an expression repeatedly in the current first-pass iterator subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### Dot

Contracts list tensors over the last axis of each left operand and the first axis of each right operand.

- Attributes: `None`
- Capability: `Pure`

### DownValues

Lists stored downvalue definitions for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Drop

Drops structural elements using the current standard sequence-spec subset.

- Attributes: `None`
- Capability: `Pure`

### DumpSave

Writes a textual first-pass dump of selected symbols to a file and returns the saved symbol list.

- Attributes: `HoldRest`
- Capability: `FileWrite`
- Limits: writes a textual dump format; it is not a binary image.

### DuplicateFreeQ

Tests whether a List has no repeated elements, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### Duplicates

Returns structurally equal top-level List elements that occur two or more times, once each in first-appearance order.

- Attributes: `None`
- Capability: `Pure`

### Echo

Prints a value through the runtime print stream and returns it.

- Attributes: `None`
- Capability: `Pure`

### EchoFunction

Returns a pure function that echoes its argument through the runtime print stream.

- Attributes: `None`
- Capability: `Pure`

### EchoLabel

Returns a pure function that echoes its argument with a label through the runtime print stream.

- Attributes: `None`
- Capability: `Pure`

### EditDistance

Computes unit-cost Levenshtein distance for strings or lists in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Enclose

Evaluates an expression and catches confirmation failures in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### End

Restores the previous context after a matching Begin.

- Attributes: `None`
- Capability: `Pure`

### EndOfLine

Represents the current string-pattern anchor for the end of a string or a position before LF.

- Attributes: `Protected`
- Capability: `Pure`

### EndOfString

Represents the current string-pattern anchor for the end of a string.

- Attributes: `Protected`
- Capability: `Pure`

### EndPackage

Ends a package context and leaves it on the restored context path.

- Attributes: `None`
- Capability: `Pure`

### Equal

Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.

- Attributes: `None`
- Capability: `Pure`

### EqualTo

Represents a unary comparison operator equivalent to Equal[#, bound] when applied.

- Attributes: `None`
- Capability: `Pure`

### Equivalent

Evaluates logical equivalence over exact boolean arguments and reduces duplicate symbolic terms in the current subset.

- Attributes: `Orderless`
- Capability: `Pure`

### EulerPhi

Computes Euler's totient function for an exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### Evaluate

Forces evaluation in held contexts.

- Attributes: `HoldAll`
- Capability: `Pure`

### EvaluationData

Evaluates an expression and returns a first-pass association of result and diagnostics.

- Attributes: `HoldFirst`
- Capability: `Clock`

### EvenQ

Tests whether an expression is currently an even exact integer.

- Attributes: `Listable`
- Capability: `Pure`

### Except

Represents the current pattern exclusion head.

- Attributes: `Protected`
- Capability: `Pure`

### Exit

Requests host-level termination with an optional integer exit code.

- Attributes: `None`
- Capability: `Process`

### Expand

Expands rational-coefficient polynomial expressions over pattern-free indeterminates.

- Attributes: `None`
- Capability: `Pure`

### Extract

Extracts parts at structural positions, optionally wrapping each result before evaluation.

- Attributes: `None`
- Capability: `Pure`

### Factor

Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.

- Attributes: `None`
- Capability: `Pure`
- Limits: handles rational-coefficient polynomial expressions; current factorization extracts monomial factors and recursively splits discovered linear factors.

### Factorial

Computes factorials for non-negative exact integers.

- Attributes: `Listable`
- Capability: `Pure`

### Factorial2

Computes double factorials for non-negative exact integers.

- Attributes: `Listable`
- Capability: `Pure`

### Failsafe

Creates a callable wrapper that preserves failure-like inputs in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Failure

Represents a failure object with associated details in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### FailureQ

Tests whether an expression is a Failure object.

- Attributes: `None`
- Capability: `Pure`

### Fibonacci

Computes exact Fibonacci numbers for integer indices.

- Attributes: `Listable`, `NumericFunction`, `Protected`, `ReadProtected`
- Capability: `Pure`

### File

Represents an explicit file specification.

- Attributes: `None`
- Capability: `Pure`

### FileBaseName

Returns the final path component without its extension.

- Attributes: `None`
- Capability: `Pure`

### FileExtension

Returns the final path component extension without the dot, or an empty string when no extension is present.

- Attributes: `None`
- Capability: `Pure`
- Limits: dotfiles such as .bashrc have no extension; only the final slash-separated path component is inspected.

### FileNameJoin

Joins path components into a platform path string.

- Attributes: `None`
- Capability: `Pure`

### FileNameSplit

Splits a slash-separated path string into components, preserving an absolute leading slash as its own component.

- Attributes: `None`
- Capability: `Pure`
- Limits: treats / as the documented separator, emits an absolute leading / as its own component, and skips repeated or trailing separators.

### FileNameTake

Takes file name components from a path string.

- Attributes: `None`
- Capability: `Pure`

### FindFile

Finds the full path of a file or package using the current search path.

- Attributes: `None`
- Capability: `FileRead`

### First

Returns the first structural element, with an optional held default.

- Attributes: `HoldRest`
- Capability: `Pure`

### FixedPoint

Repeatedly applies a function until the result stops changing in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### FixedPointList

Returns successive repeated applications until a fixed point in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Flatten

Flattens nested expressions through the current non-negative level, single-head-spec, and level-list subset.

- Attributes: `None`
- Capability: `Pure`

### Floor

Returns the greatest integer not greater than an exact number.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### Fold

Accumulates over elements in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### FoldList

Returns successive accumulator states in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### FoldPair

Accumulates pair-valued state transitions in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### FoldWhile

Accumulates while a test stays True in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### For

Evaluates start, test, increment, and body in the current first-pass procedural subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### FractionalPart

Returns the exact fractional part after truncating toward zero.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### FreeQ

Tests whether no subexpression matches a pattern on levels specified by a standard levelspec.

- Attributes: `None`
- Capability: `Pure`

### FromBase64

Decodes standard RFC 4648 base64 with padding into a ByteArray, failing closed on malformed input.

- Attributes: `Protected`
- Capability: `Pure`

### FromCsv

Parses CSV text into Lists of Strings, optionally using the first row as Association headers.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: parses RFC 4180-style text into String fields only; plain rows may be ragged, while Headers -> True rejects duplicate headers and ragged data rows. Delimiter must be one non-quote, non-newline character.

### FromDateString

Parses the documented ISO 8601 UTC/fixed-offset subset into an exact Unix timestamp.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: accepts only the documented ISO 8601 subset with optional Z or fixed +/-HH:MM offsets; there is no time-zone database and no leap-second support.

### FromDigits

Constructs an exact integer from a list or string of digits.

- Attributes: `Protected`
- Capability: `Pure`

### FromHexString

Decodes even-length hexadecimal text into a ByteArray, failing closed on malformed input.

- Attributes: `Protected`
- Capability: `Pure`

### FromIni

Parses rum's fail-closed INI dialect into an Association of section Associations.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: implements rum's documented INI dialect only: whole-line ;/# comments, [section] headers, key = value lines, trimmed keys/values, verbatim values, no nesting, no multiline values, and no inline comments.

### FromJson

Parses an RFC 8259 JSON text string into exact rum values, failing closed on invalid JSON.

- Attributes: `Protected`
- Capability: `Pure`

### FromToml

Parses rum's documented fail-closed TOML subset into exact rum values.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: implements rum's documented TOML 1.0 folk subset only: tables, arrays-of-tables, inline tables, strings, exact integers/decimals, booleans, and date/time tokens; inf/nan and unsupported TOML surface fail closed. Offset date-times decode as exact Unix-second instants, while local dates/times decode as Lists.

### FromXml

Parses XML text into a symbolic XmlElement tree, rejecting DTDs and unsupported XML surface.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: parses a value-oriented XML element tree only: declaration syntax is validated but not represented, comments are dropped, CDATA becomes text, namespaces remain verbatim, whitespace-only runs around element content are dropped, and DTD/DOCTYPE declarations are deliberately rejected.

### FromYaml

Parses rum's documented fail-closed YAML block subset into exact rum values.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: implements rum's documented YAML sane block subset only: block mappings and sequences, plain and quoted scalars, literal | and |- blocks, comments, and empty flow {} / []; anchors, aliases, tags, directives, document markers, merge keys, folded blocks, non-empty flow collections, duplicate keys, tabs in indentation, and ambiguous same-indent collection mixes fail closed.

### FullForm

Renders the held argument as a canonical structural-form string in the current core expression subset.

- Attributes: `None`
- Capability: `Pure`

### Function

Creates a pure function.

- Attributes: `HoldAll`
- Capability: `Pure`

### GCD

Computes the greatest common divisor of exact integers.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### GatherBy

Groups top-level elements by evaluated keys in first-occurrence key order, including hierarchical grouping for a list of functions.

- Attributes: `None`
- Capability: `Pure`

### Get

Reads a file or package and evaluates the expressions it contains.

- Attributes: `None`
- Capability: `FileRead`

### Greater

Tests strict decreasing order in the current exact comparable subset.

- Attributes: `None`
- Capability: `Pure`

### GreaterEqual

Tests nonincreasing order in the current exact comparable subset.

- Attributes: `None`
- Capability: `Pure`

### GreaterEqualThan

Represents a unary comparison operator equivalent to GreaterEqual[#, bound] when applied.

- Attributes: `None`
- Capability: `Pure`

### GreaterThan

Represents a unary comparison operator equivalent to Greater[#, bound] when applied.

- Attributes: `None`
- Capability: `Pure`

### GroupBy

Groups list elements by transformed keys, including the current reducer form.

- Attributes: `None`
- Capability: `Pure`

### HammingDistance

Computes mismatch counts for equal-length strings or lists in the current first-pass subset; unequal lengths emit a message and leave the call inert.

- Attributes: `None`
- Capability: `Pure`

### Hash

Computes SHA256, SHA1, SHA512, MD5, or CRC32 hashes over the current string/expression subset.

- Attributes: `None`
- Capability: `Pure`

### Head

Returns the structural head of an expression, optionally wrapped in a second head.

- Attributes: `None`
- Capability: `Pure`

### Hold

Prevents evaluation of its arguments.

- Attributes: `HoldAll`
- Capability: `Pure`

### HoldAll

Prevents evaluation of all arguments.

- Attributes: `HoldAll`
- Capability: `Pure`

### HoldAllComplete

Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.

- Attributes: `HoldAllComplete`
- Capability: `Pure`

### HoldComplete

Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.

- Attributes: `HoldAllComplete`
- Capability: `Pure`

### HoldFirst

Prevents evaluation of the first argument.

- Attributes: `HoldFirst`
- Capability: `Pure`

### HoldForm

Holds an expression while rendering transparently in output-like forms.

- Attributes: `HoldAll`, `Protected`
- Capability: `Pure`

### HoldPattern

Marks a pattern expression so it is matched without prior evaluation.

- Attributes: `HoldAll`
- Capability: `Pure`

### HoldRest

Prevents evaluation of all but the first argument.

- Attributes: `HoldRest`
- Capability: `Pure`

### HtmlEscape

Escapes the five HTML text characters &, <, >, double quote, and single quote.

- Attributes: `None`
- Capability: `Pure`

### HtmlUnescape

Unescapes the five named HTML entities and decimal or hexadecimal numeric character references, leaving unknown and malformed entities verbatim by default; with "Strict" -> True, malformed entities fail closed with $Failed.

- Attributes: `None`
- Capability: `Pure`

### Identity

Returns its argument unchanged.

- Attributes: `None`
- Capability: `Pure`

### If

Evaluates one of several branches conditionally.

- Attributes: `HoldRest`
- Capability: `Pure`

### IgnoreCase

Represents the current option symbol for case-insensitive string matching.

- Attributes: `None`
- Capability: `Pure`

### Implies

Evaluates binary logical implication over exact boolean arguments in the current subset.

- Attributes: `None`
- Capability: `Pure`

### Increment

Increments a held symbol or exact Part target and returns its previous value.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Information

Returns a first-pass InformationData summary for a symbol or symbol-name string.

- Attributes: `None`
- Capability: `Pure`

### Inner

Combines matching tensor dimensions in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Input

Requests one line of host-provided text, parses it as rum source, and evaluates the resulting expression in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### InputForm

Represents the current formatting symbol accepted by the current ToExpression subset.

- Attributes: `None`
- Capability: `Pure`

### InputString

Requests one line of host-provided text and returns it as a string.

- Attributes: `None`
- Capability: `Pure`

### Insert

Inserts an element at a top-level structural position in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### IntegerDigits

Returns the digits of an exact integer in a given base.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### IntegerLength

Returns the number of digits in an exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### IntegerPart

Returns the integer part of an exact number by truncating toward zero.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### IntegerQ

Tests whether an expression is an exact integer atom.

- Attributes: `None`
- Capability: `Pure`

### IntegerReverse

Reverses the digits of an exact integer in a given base.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`

### IntersectingQ

Tests whether two Lists share any element, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### Intersection

Returns sorted unique elements common to all same-head normal expressions; SameTest keeps the first representative in structural scan order.

- Attributes: `Flat`, `OneIdentity`, `Protected`, `ReadProtected`
- Capability: `Pure`

### Join

Concatenates same-head normal expressions and associations in the current top-level subset.

- Attributes: `None`
- Capability: `Pure`

### Key

Represents a keyed selector in the current first-pass association subset.

- Attributes: `None`
- Capability: `Pure`

### KeyComplement

Builds a key-complement association in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeyDrop

Drops selected keys from an association in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeyDropFrom

Drops keys from an association, including held symbol mutation in the current first-pass subset.

- Attributes: `HoldFirst`
- Capability: `Pure`

### KeyExistsQ

Tests whether an association contains a given key in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeyFreeQ

Tests whether an association does not contain a key in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeyGroupBy

Groups association entries by transformed keys, including the current reducer form.

- Attributes: `None`
- Capability: `Pure`

### KeyIntersection

Builds a key-intersection association in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeyMap

Maps a function over association keys in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeyMemberQ

Tests whether an association contains a key in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeySelect

Selects association entries whose keys satisfy a criterion in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeySort

Sorts association entries by key in the current canonical ordering subset.

- Attributes: `None`
- Capability: `Pure`

### KeySortBy

Sorts association entries by transformed keys, optionally using a supplied binary ordering predicate.

- Attributes: `None`
- Capability: `Pure`

### KeyTake

Keeps selected keys from an association in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeyUnion

Builds keyed unions, including the current missing-key fill-function form.

- Attributes: `None`
- Capability: `Pure`

### KeyValueMap

Maps a binary function over association key-value pairs in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### KeyValuePattern

Represents an association key/value pattern specification in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Keys

Returns association keys in structural order.

- Attributes: `None`
- Capability: `Pure`

### LCM

Computes the least common multiple of exact integers.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### Last

Returns the last structural element, with an optional held default.

- Attributes: `HoldRest`
- Capability: `Pure`

### LeafCount

Counts atomic leaves in an expression's full structural tree, including heads.

- Attributes: `Protected`
- Capability: `Pure`

### LeapYearQ

Tests whether an exact integer year is a leap year in the proleptic Gregorian calendar.

- Attributes: `Protected`
- Capability: `Pure`

### Length

Returns the number of structural elements in an expression.

- Attributes: `None`
- Capability: `Pure`

### LengthWhile

Counts the longest prefix satisfying a criterion in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Less

Tests strict increasing order in the current exact comparable subset.

- Attributes: `None`
- Capability: `Pure`

### LessEqual

Tests nondecreasing order in the current exact comparable subset.

- Attributes: `None`
- Capability: `Pure`

### LessEqualThan

Represents a unary comparison operator equivalent to LessEqual[#, bound] when applied.

- Attributes: `None`
- Capability: `Pure`

### LessThan

Represents a unary comparison operator equivalent to Less[#, bound] when applied.

- Attributes: `None`
- Capability: `Pure`

### LetterCharacter

Represents the current string-pattern atom for one alphabetic character.

- Attributes: `Protected`
- Capability: `Pure`

### LetterQ

Tests whether all characters in a string belong to the current letter subset.

- Attributes: `None`
- Capability: `Pure`

### Level

Collects subexpressions found on levels specified by a standard levelspec.

- Attributes: `None`
- Capability: `Pure`

### LexicographicOrder

Compares two expressions lexicographically under the default order or a supplied binary predicate.

- Attributes: `None`
- Capability: `Pure`

### ListQ

Tests whether an expression has head List.

- Attributes: `None`
- Capability: `Pure`

### Longest

Represents the current string-pattern wrapper that prefers the longest match.

- Attributes: `None`
- Capability: `Pure`

### LongestCommonSubsequence

Finds the longest common contiguous substring or sublist in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Lookup

Looks up a key in an association in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### LowerCaseQ

Tests whether every character in a string is lowercase.

- Attributes: `Protected`
- Capability: `Pure`

### LowercaseLetters

Evaluates to the lowercase ASCII letter character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### MachineNumberQ

Tests whether an expression is a machine number; currently always False because only exact numbers exist.

- Attributes: `None`
- Capability: `Pure`

### Map

Applies a function over selected levels of an expression in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### MapAll

Applies a function to every subexpression in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### MapApply

Applies head replacement one level down in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### MapAt

Applies a function at exact part specifications in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### MapIndexed

Applies a function to parts together with their positions in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### MapThread

Threads corresponding elements from multiple expressions in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### MatchQ

Tests whether an expression matches a pattern.

- Attributes: `None`
- Capability: `Pure`

### MatrixQ

Tests whether an expression is a rectangular two-dimensional List array, optionally requiring every leaf to satisfy a test.

- Attributes: `None`
- Capability: `Pure`

### Max

Returns the maximum exact integer or rational operand.

- Attributes: `NumericFunction`, `Protected`
- Capability: `Pure`

### MaxBy

Returns the first top-level element attaining the maximum evaluated key, or the top n in decreasing key order with input-order ties.

- Attributes: `None`
- Capability: `Pure`

### MaxMemoryUsed

Returns the peak session memory-proxy byte count high-water mark subset.

- Attributes: `None`
- Capability: `Diagnostic`

### Md5

Computes an MD5 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: intended only for interoperability and checksums; it is not collision-resistant.

### Mean

Aggregates a non-empty list as Total[list] divided by Length[list] through exact arithmetic.

- Attributes: `None`
- Capability: `Pure`

### Median

Returns the middle element of a canonically sorted non-empty list, averaging the two middle values exactly for even lengths.

- Attributes: `None`
- Capability: `Pure`

### MemberQ

Tests whether any subexpression matches a pattern on levels specified by a standard levelspec.

- Attributes: `None`
- Capability: `Pure`

### MemoryConstrained

Evaluates an expression under the current cooperative memory-proxy limit subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### MemoryInUse

Returns the current session memory-proxy byte count subset.

- Attributes: `None`
- Capability: `Diagnostic`

### Merge

Merges associations by key using a combiner in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### MergeNested

Recursively merges nested associations by key using a combiner at leaves.

- Attributes: `None`
- Capability: `Pure`

### Message

Emits a message described by a MessageName expression.

- Attributes: `HoldFirst`
- Capability: `Pure`

### MessageDefinitions

Lists stored message template definitions for the whole session, a symbol, or one message query.

- Attributes: `HoldAll`
- Capability: `Pure`

### MessageName

Represents a symbolic message name such as f::tag.

- Attributes: `HoldAll`
- Capability: `Pure`

### MessageTags

Lists message tags defined globally, for a symbol, or for one message query.

- Attributes: `HoldAll`
- Capability: `Pure`

### Messages

Lists message names defined for the whole session, a symbol, or one message query.

- Attributes: `HoldAll`
- Capability: `Pure`

### Min

Returns the minimum exact integer or rational operand.

- Attributes: `NumericFunction`, `Protected`
- Capability: `Pure`

### MinBy

Returns the first top-level element attaining the minimum evaluated key, or the bottom n in increasing key order with input-order ties.

- Attributes: `None`
- Capability: `Pure`

### MinMax

Returns the exact minimum and maximum for one scalar or list argument.

- Attributes: `Protected`
- Capability: `Pure`

### Minus

Negates one exact or symbolic argument.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### Missing

Represents missing-data markers in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### MissingQ

Tests whether an expression is a Missing object.

- Attributes: `None`
- Capability: `Pure`

### Mod

Returns the modulus of one integer with respect to another.

- Attributes: `Listable`
- Capability: `Pure`

### Module

Creates lexical local symbols.

- Attributes: `HoldAll`
- Capability: `Pure`

### MoebiusMu

Computes the Moebius mu function for an exact integer.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### Monitor

Evaluates an expression while exposing the current first-pass monitor display and callback subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### Most

Returns an expression with its last structural element removed.

- Attributes: `None`
- Capability: `Pure`

### Multinomial

Computes exact multinomial coefficients for non-negative integer arguments.

- Attributes: `Listable`, `NumericFunction`, `Orderless`, `Protected`
- Capability: `Pure`

### NameQ

Tests whether a string is the exact name of an existing symbol.

- Attributes: `Protected`
- Capability: `Pure`

### Names

Lists symbol names matching a current first-pass name pattern subset.

- Attributes: `None`
- Capability: `Pure`

### Nand

Evaluates a negated conjunction with current short-circuit boolean semantics.

- Attributes: `HoldAll`
- Capability: `Pure`

### Nearest

Finds nearest strings or exact numeric values in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Needs

Loads a package only if its context has not already been loaded.

- Attributes: `None`
- Capability: `FileRead`

### Negative

Tests negativity in the current exact integer/rational subset.

- Attributes: `Listable`
- Capability: `Pure`

### Nest

Applies a function repeatedly a fixed number of times.

- Attributes: `None`
- Capability: `Pure`

### NestList

Returns repeated applications of a function in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### NestWhile

Repeatedly applies a function while a test stays True in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### NestWhileList

Returns repeated applications while a test stays True in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### NextPrime

Returns the next or previous prime in rum's signed-prime order.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`

### NonNegative

Tests non-negativity in the current exact integer/rational subset.

- Attributes: `Listable`
- Capability: `Pure`

### NonPositive

Tests non-positivity in the current exact integer/rational subset.

- Attributes: `Listable`
- Capability: `Pure`

### NoneTrue

Tests whether no selected structural parts satisfy a criterion in the current boolean subset.

- Attributes: `None`
- Capability: `Pure`

### Nor

Evaluates a negated disjunction with current short-circuit boolean semantics.

- Attributes: `HoldAll`
- Capability: `Pure`

### Normal

Converts an association into a list of rules in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### NormalizeRules

Normalizes a rule or rule list into an explicit canonical list form for debugging.

- Attributes: `HoldAll`
- Capability: `Pure`

### Not

Negates a boolean value in the current exact boolean subset.

- Attributes: `None`
- Capability: `Pure`

### Nothing

Represents a list element that is removed from evaluated Lists.

- Attributes: `None`
- Capability: `Pure`

### NumberQ

Tests whether an expression is an exact numeric atom in the current number subset.

- Attributes: `None`
- Capability: `Pure`

### NumberString

Represents the current string-pattern atom for decimal-style numeric text.

- Attributes: `None`
- Capability: `Pure`

### Numerator

Returns the numerator of exact integers and rationals.

- Attributes: `Listable`
- Capability: `Pure`

### NumericQ

Tests whether an expression is currently numeric in the exact-number subset.

- Attributes: `None`
- Capability: `Pure`

### NumericalOrder

Compares two expressions using exact numeric order when possible, then canonical order.

- Attributes: `None`
- Capability: `Pure`

### OddQ

Tests whether an expression is currently an odd exact integer.

- Attributes: `Listable`
- Capability: `Pure`

### Off

Disables session-level assertion checking or emission of matching messages in the current selector subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### On

Enables session-level assertion checking or emission of matching messages in the current selector subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### Once

Evaluates an expression at most once per session-local cache key in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### Operate

Applies an operator to heads in the head chain of an expression in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### OperatorApplied

Represents a curried operator wrapper in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### OptionValue

Looks up an option value from explicit rules or a symbol's defaults.

- Attributes: `HoldAll`
- Capability: `Pure`

### Options

Lists the current default option rules for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Or

Evaluates a disjunction with current short-circuit boolean semantics.

- Attributes: `HoldAll`
- Capability: `Pure`

### Order

Orders two expressions using exact numeric order when possible, then the current canonical textual ordering subset.

- Attributes: `None`
- Capability: `Pure`

### OrderedQ

Tests whether an expression's arguments are ordered under the default order or a supplied binary predicate.

- Attributes: `None`
- Capability: `Pure`

### Outer

Forms outer combinations of same-head expressions in the current dense-array subset.

- Attributes: `None`
- Capability: `Pure`

### OutputForm

Represents the current formatting symbol accepted by the current ToString subset.

- Attributes: `None`
- Capability: `Pure`

### Overlaps

Represents the current option symbol for overlapping string-match counting.

- Attributes: `None`
- Capability: `Pure`

### OwnValues

Lists stored immediate or delayed ownvalue definitions for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### PadLeft

Pads or truncates a normal expression on the left using scalar lengths or rectangular list dimensions.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: does not support the margin argument; list dimensions apply to rectangular nested lists only.

### PadRight

Pads or truncates a normal expression on the right using scalar lengths or rectangular list dimensions.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: does not support the margin argument; list dimensions apply to rectangular nested lists only.

### ParentDirectory

Returns the parent directory for an existing directory path.

- Attributes: `None`
- Capability: `Pure`

### ParseArgs

Parses script arguments into positional arguments, --key=value options, and --flag names; with "ShortOptions" -> True, single-dash clusters become single-letter flags, without attached-value syntax.

- Attributes: `None`
- Capability: `Pure`
- Limits: recognizes --key=value options, bare --flag flags, and -- as the end of option parsing; "ShortOptions" -> True recognizes single-dash flag clusters but does not support attached values.

### Part

Extracts structural parts using integer positions, All, Span, and exact nested part specifications.

- Attributes: `None`
- Capability: `Pure`

### Partition

Partitions a flat normal expression into fixed-size blocks; aligned forms anchor the first element at kL and stop once the last element appears at or beyond kR.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: supports flat positive scalar size/offset forms; aligned forms use positive {kL, kR}, prepend kL - 1 fill positions, stop at the first d-stepped block where the final item appears at position >= kR, use cyclic list padding when padding is omitted, and use ragged no-padding output for {}.

### PartitionsP

Computes the number of integer partitions by Euler's pentagonal recurrence.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### PartitionsQ

Computes the number of partitions into distinct parts.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### Pattern

Represents the current named pattern-binding head.

- Attributes: `Protected`
- Capability: `Pure`

### Pause

Pauses for at least a nonnegative exact number of seconds in the current first-pass subset.

- Attributes: `None`
- Capability: `Clock`

### Pluralize

Applies a simple English pluralization subset.

- Attributes: `None`
- Capability: `Pure`

### Plus

Adds exact numbers and canonically combines additive expressions.

- Attributes: `Listable`, `Orderless`, `Flat`, `OneIdentity`
- Capability: `Pure`

### Pochhammer

Computes finite rising factorial products for non-negative integer counts.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### PolynomialExpressionQ

Tests whether an expression is polynomial in the requested variable subset.

- Attributes: `None`
- Capability: `Pure`

### PolynomialQ

Tests whether an expression is polynomial in the current exact symbolic subset.

- Attributes: `None`
- Capability: `Pure`

### Position

Finds positions of subexpressions matching a pattern on levels specified by a standard levelspec.

- Attributes: `None`
- Capability: `Pure`

### PositionIndex

Builds an association from elements to 1-based position lists in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Positive

Tests positivity in the current exact integer/rational subset.

- Attributes: `Listable`
- Capability: `Pure`

### PossibleZeroQ

Tests exact zero in the current exact integer/rational subset.

- Attributes: `Listable`
- Capability: `Pure`

### Power

Raises an exact number to an integer power when possible.

- Attributes: `Listable`
- Capability: `Pure`

### PowerMod

Computes modular exponentiation for exact integers.

- Attributes: `Listable`
- Capability: `Pure`

### PreDecrement

Decrements a held symbol or exact Part target and returns the updated value.

- Attributes: `HoldFirst`
- Capability: `Pure`

### PreIncrement

Increments a held symbol or exact Part target and returns the updated value.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Prepend

Prepends an element to the front of a normal expression, with current operator-form support.

- Attributes: `None`
- Capability: `Pure`

### PrependTo

Prepends an evaluated item to a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Prime

Returns the n-th positive prime for a positive integer n.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Limits: uses a bounded integer-work budget and sieve-based prime generation for supported exact positive integer indices.

### PrimeNu

Counts distinct prime factors of an exact integer.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`

### PrimeOmega

Counts prime factors with multiplicity for an exact integer.

- Attributes: `Protected`, `ReadProtected`
- Capability: `Pure`

### PrimePi

Counts primes not greater than an exact integer or rational argument.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`
- Limits: uses a bounded integer-work budget and sieve-based counting after flooring exact rational inputs.

### PrimeQ

Tests primality in the current exact-integer subset.

- Attributes: `Listable`
- Capability: `Pure`
- Limits: uses direct trial division over exact integers.

### Print

Emits a line of plain print output through the runtime print stream.

- Attributes: `None`
- Capability: `Pure`

### Product

Multiplies expression values over concrete iterator specifications.

- Attributes: `HoldAll`, `Protected`, `ReadProtected`
- Capability: `Pure`

### PropagateAborts

Represents the current first-pass CheckAbort option for rethrowing caught aborts.

- Attributes: `None`
- Capability: `Pure`

### Protect

Adds the Protected attribute to one or more symbols.

- Attributes: `HoldAll`
- Capability: `Pure`

### PunctuationCharacter

Represents the current string-pattern atom for one ASCII punctuation character.

- Attributes: `Protected`
- Capability: `Pure`

### PunctuationCharacters

Evaluates to the ASCII punctuation character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### Quantile

Returns the nearest-rank quantile of a non-empty list using Ceiling[q * n] clamped to 1..n, with q exact in [0, 1].

- Attributes: `None`
- Capability: `Pure`

### Quiet

Evaluates an expression while suppressing matching runtime messages it emits.

- Attributes: `HoldAll`
- Capability: `Pure`

### Quotient

Returns the floor-division quotient of two integers.

- Attributes: `Listable`
- Capability: `Pure`

### RandomBytes

Returns entropy bytes as a ByteArray using the host entropy provider.

- Attributes: `None`
- Capability: `Random`
- Limits: draws directly from the host entropy provider, not from the deterministic session PRNG.

### RandomChoice

Chooses elements from a list using the deterministic session PRNG.

- Attributes: `None`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG; it is not cryptographic.

### RandomInteger

Draws exact integers uniformly from the deterministic session PRNG.

- Attributes: `None`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG with rejection sampling for unbiased exact integers; it is not cryptographic. There is no RandomReal in the current exact-number boundary.

### RandomPassword

Draws an entropy-backed password string from an alphabet, optionally enforcing policy constraints.

- Attributes: `None`
- Capability: `Random`
- Limits: uses the host entropy provider directly with rejection sampling over the alphabet size to avoid modulo bias; it is not affected by SeedRandom or BlockRandom. The default alphabet is the 62-character A-Za-z0-9 set. Alphabet strings are read as code points, and duplicate code points weight the draw. RandomPassword[n, opts...] and RandomPassword[alphabet, n, opts...] accept string-keyed policy rules "Require" -> {group, ...}, "NoAdjacentRepeats" -> True|False, "Exclude" -> {substr, ...}, and "IgnoreCase" -> True|False. Require groups are exact character sets and must be non-empty subsets of the alphabet; IgnoreCase affects only Exclude matching. NoAdjacentRepeats forbids identical adjacent characters only, not sequential runs. Policy sampling draws whole candidate passwords uniformly from alphabet^n and accepts only candidates satisfying every constraint, so accepted outputs are uniform over the satisfying set. Unsatisfied policies fail closed after 10000 attempts.

### RandomSample

Samples distinct positions from a list using the deterministic session PRNG.

- Attributes: `None`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG; it is not cryptographic.

### RandomString

Draws a string of characters from an alphabet using the deterministic session PRNG.

- Attributes: `None`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG with rejection sampling over the alphabet size; it is not cryptographic. The default alphabet is the 62-character A-Za-z0-9 set. Alphabet strings are read as code points, and duplicate code points weight the draw.

### RandomUuid

Returns a reproducible version-4-format UUID string from the deterministic session PRNG.

- Attributes: `None`
- Capability: `Pure`
- Limits: uses the deterministic session PRNG and is reproducible under the same seed; use an entropy reseed first when global uniqueness matters.

### Range

Constructs a plain list of exact integers from the current one-, two-, and three-argument subset.

- Attributes: `None`
- Capability: `Pure`

### RationalExpressionQ

Tests whether an expression is a rational expression in the requested variable subset.

- Attributes: `None`
- Capability: `Pure`

### RealValuedNumberQ

Tests whether an expression is a real-valued number in the current exact-number subset.

- Attributes: `None`
- Capability: `Pure`

### RealValuedNumericQ

Tests whether an expression is real-valued and numeric in the current exact-number subset.

- Attributes: `None`
- Capability: `Pure`

### Reap

Evaluates an expression and collects values emitted by Sow in the current first-pass grouping subset.

- Attributes: `HoldFirst`
- Capability: `Pure`

### RegularExpression

Represents the current regular-expression string-pattern wrapper.

- Attributes: `Protected`
- Capability: `Pure`

### ReleaseHold

Removes supported hold wrappers and resumes evaluation.

- Attributes: `HoldAll`
- Capability: `Pure`

### Remove

Removes the current stored state for one or more symbols in the first-pass namespace subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### RemoveDiacritics

Removes common Latin diacritics from strings in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Repeated

Represents the current string-pattern repetition head for one or more matches.

- Attributes: `None`
- Capability: `Pure`

### RepeatedNull

Represents the current string-pattern repetition head for zero or more matches.

- Attributes: `None`
- Capability: `Pure`

### RepeatedTiming

Evaluates an expression multiple times and returns {elapsed, value} in the current fixed-sample wall-clock subset.

- Attributes: `HoldFirst`, `SequenceHold`
- Capability: `Clock`

### Replace

Applies rules at the current first-pass Replace levelspec subset.

- Attributes: `None`
- Capability: `Pure`

### ReplaceAll

Applies rules everywhere in an expression.

- Attributes: `None`
- Capability: `Pure`

### ReplaceAt

Applies rules at exact structural positions, with the current operator-form and exact-position first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### ReplaceList

Returns the current first-pass root replacement results across a rule list, with optional max-result limiting.

- Attributes: `None`
- Capability: `Pure`

### ReplacePart

Replaces structural parts via immediate or delayed rules over exact, Span, and pattern-based position specifications, with Heads-aware matching.

- Attributes: `None`
- Capability: `Pure`

### ReplaceRepeated

Repeatedly applies rules until an expression reaches a fixed point.

- Attributes: `None`
- Capability: `Pure`

### ResolveContextAliases

Represents the current Names option symbol for context-alias resolution.

- Attributes: `None`
- Capability: `Pure`

### Rest

Returns an expression with its first structural element removed.

- Attributes: `None`
- Capability: `Pure`

### Return

Exits the current function-style evaluation with an optional result.

- Attributes: `None`
- Capability: `Pure`

### Reverse

Reverses normal-expression arguments at positive levels or at every level with All.

- Attributes: `None`
- Capability: `Pure`

### ReverseApplied

Represents a wrapper that reverses current call arguments before application in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### ReverseSort

Sorts top-level elements using structural ordering or a binary predicate, then reverses the result.

- Attributes: `None`
- Capability: `Pure`

### ReverseSortBy

Sorts top-level elements by an evaluated key, then reverses the result.

- Attributes: `None`
- Capability: `Pure`

### Riffle

Inserts one separator between top-level elements of a normal expression, with optional interval or position specs.

- Attributes: `None`
- Capability: `Pure`
- Limits: supports one separator with optional positive interval or {imin, imax, di} insertion-position specs; list-separator forms are outside the current contract.

### RightComposition

Represents a right composition of functions in the current first-pass subset.

- Attributes: `Flat`, `OneIdentity`
- Capability: `Pure`

### Round

Rounds exact numbers to the nearest integer or exact multiple in the current snapshot-helper subset.

- Attributes: `Listable`
- Capability: `Pure`

### Row

Renders a list of expressions as a single text string in the current harness subset.

- Attributes: `None`
- Capability: `Pure`

### Rule

Represents an immediate rewrite rule.

- Attributes: `None`
- Capability: `Pure`

### RuleDelayed

Represents a delayed rewrite rule.

- Attributes: `None`
- Capability: `Pure`

### RumProfileReset

Internal diagnostic hook: resets rum runtime profile counters when profiling is enabled.

- Attributes: `None`
- Capability: `Diagnostic`

### RumProfileSnapshot

Internal diagnostic hook: prints rum runtime profile counters when profiling is enabled.

- Attributes: `None`
- Capability: `Diagnostic`

### RumTokens

Tokenizes rum source text into total, trivia-preserving lexer spans.

- Attributes: `None`
- Capability: `Pure`

### SameAs

Represents a unary comparison operator equivalent to SameQ[#, bound] when applied.

- Attributes: `None`
- Capability: `Pure`

### SameQ

Tests whether all arguments are exactly identical after evaluation.

- Attributes: `None`
- Capability: `Pure`

### Save

Appends a textual first-pass serialization of selected symbols to a file.

- Attributes: `HoldRest`
- Capability: `FileWrite`

### Scan

Evaluates a function across selected levels for side effects in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### SchemaErrors

Returns path-keyed validation errors for a value checked against a symbolic schema.

- Attributes: `HoldRest`, `Protected`
- Capability: `Pure`

### SeedRandom

Reseeds the deterministic session PRNG from an exact integer, or from host entropy in the zero-argument form.

- Attributes: `None`
- Capability: `Pure`
- Limits: uses xoshiro256** with splitmix64 seed expansion; exact integer seeds fold magnitude bytes through splitmix64, with a sign tag for negative seeds. The zero-argument entropy form requires Capability::Random.

### Select

Selects elements satisfying a criterion in the current sequence subset.

- Attributes: `None`
- Capability: `Pure`

### SelectFirst

Returns the first element satisfying a criterion in the current sequence subset.

- Attributes: `None`
- Capability: `Pure`

### SelectPositions

Returns one-element 1-based position paths for top-level List elements whose criterion evaluates to True, matching Position's path convention, with an optional first-n limit.

- Attributes: `None`
- Capability: `Pure`

### Sequence

Splices a sequence of arguments into a call.

- Attributes: `None`
- Capability: `Pure`

### SequenceAlignment

Builds a simple global alignment summary for strings or lists in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### SequenceFold

Folds a sequence with a sliding history in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### SequenceFoldList

Returns the running history of a sequence fold in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Set

Assigns an immediate value to a symbol or pattern.

- Attributes: `None`
- Capability: `Pure`

### SetAttributes

Adds attributes to one or more symbols.

- Attributes: `HoldFirst`
- Capability: `Pure`

### SetDelayed

Assigns a delayed definition evaluated on use.

- Attributes: `None`
- Capability: `Pure`

### SetOptions

Updates known default options for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Sha1

Computes a SHA-1 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: intended only for interoperability and checksums; it is not collision-resistant.

### Sha256

Computes a SHA-256 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.

- Attributes: `Protected`
- Capability: `Pure`

### Sha512

Computes a SHA-512 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.

- Attributes: `Protected`
- Capability: `Pure`

### ShellQuote

Single-quotes a string for POSIX-style shell input, escaping embedded single quotes.

- Attributes: `None`
- Capability: `Pure`

### ShellSplit

Splits a POSIX-style shell word string with whitespace, quotes, and backslash escapes, returning $Failed on malformed quoting.

- Attributes: `None`
- Capability: `Pure`
- Limits: implements only whitespace splitting, single and double quotes, and backslash escapes outside single quotes; expansion, globbing, command substitution, and short-option parsing are outside this text utility.

### Short

Returns the current canonical-text shortened display subset for an expression.

- Attributes: `None`
- Capability: `Pure`

### Shortest

Represents the current string-pattern wrapper that prefers the shortest match.

- Attributes: `None`
- Capability: `Pure`

### Sign

Returns the sign of exact integers and rationals.

- Attributes: `Listable`, `NumericFunction`, `Protected`
- Capability: `Pure`

### Sort

Sorts top-level elements using the current structural ordering subset.

- Attributes: `None`
- Capability: `Pure`

### SortBy

Sorts top-level elements by an evaluated key, optionally ordering keys with a binary predicate and preserving input order for predicate ties.

- Attributes: `None`
- Capability: `Pure`

### Sow

Collects a value for enclosing Reap in the current first-pass tag-and-group subset.

- Attributes: `None`
- Capability: `Pure`

### SpellingCorrection

Represents the current Names option symbol for spelling-correction control.

- Attributes: `None`
- Capability: `Pure`

### Stack

Returns the current evaluation stack in the first-pass held-expression subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### StandardForm

Represents the current formatting symbol accepted by the current ToExpression subset.

- Attributes: `None`
- Capability: `Pure`

### StartOfLine

Represents the current string-pattern anchor for the start of a string or a position after LF.

- Attributes: `Protected`
- Capability: `Pure`

### StartOfString

Represents the current string-pattern anchor for the start of a string.

- Attributes: `Protected`
- Capability: `Pure`

### StirlingS1

Computes signed Stirling numbers of the first kind.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### StirlingS2

Computes Stirling numbers of the second kind.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### StringApply

Applies a list-oriented function to string code points and reassembles string output in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### StringCases

Extracts non-overlapping exact string matches or immediate exact-string rule outputs in the current first-pass string subset.

- Attributes: `None`
- Capability: `Pure`

### StringContainsQ

Tests whether a string or list of strings contains an exact substring in the current first-pass string subset.

- Attributes: `None`
- Capability: `Pure`

### StringCount

Counts exact or current string-pattern matches, including the current IgnoreCase and Overlaps option subset.

- Attributes: `None`
- Capability: `Pure`

### StringDedent

Removes the longest common leading whitespace from the non-blank lines of a string.

- Attributes: `None`
- Capability: `Pure`
- Limits: computes the common leading whitespace from non-blank lines; blank and whitespace-only lines do not determine the prefix.

### StringDelete

Deletes exact or current string-pattern matches from strings in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### StringDrop

Drops a substring using integer counts or inclusive 1-based ranges.

- Attributes: `None`
- Capability: `Pure`

### StringEndsQ

Tests whether a string or list of strings ends with an exact substring in the current first-pass string subset.

- Attributes: `None`
- Capability: `Pure`

### StringExpandTabs

Expands tabs in a string to the next fixed-width column stop, defaulting to width 8.

- Attributes: `None`
- Capability: `Pure`

### StringExpression

Represents the current string-pattern sequence head produced by `~~` syntax.

- Attributes: `Protected`
- Capability: `Pure`

### StringForm

Formats a template string eagerly, replacing `` and `n` placeholders with ToString-rendered arguments.

- Attributes: `None`
- Capability: `Pure`
- Limits: returns a String eagerly rather than a lazy display wrapper; a backslash before a backtick emits a literal backtick, and missing or malformed placeholders emit StringForm::slot and leave the call inert.

### StringFreeQ

Tests whether a string is free of exact or current string-pattern matches.

- Attributes: `None`
- Capability: `Pure`

### StringIndent

Prefixes each non-blank line of a string with a string prefix or an exact number of spaces. With "BlankLines" -> True, blank and whitespace-only lines are prefixed too; the default False leaves them unchanged.

- Attributes: `None`
- Capability: `Pure`
- Limits: does not prefix empty or whitespace-only lines.

### StringJoin

Concatenates strings or a list of strings into a single string.

- Attributes: `None`
- Capability: `Pure`

### StringLength

Returns the Unicode code-point length of a string.

- Attributes: `Listable`
- Capability: `Pure`

### StringLines

Splits a string on LF or CRLF newlines, preserving interior empty lines.

- Attributes: `None`
- Capability: `Pure`
- Limits: splits only on LF and CRLF; one trailing newline is discarded while interior empty lines are preserved.

### StringMatchQ

Tests whether a whole string matches an exact or current string-pattern spec.

- Attributes: `None`
- Capability: `Pure`

### StringPadLeft

Pads strings on the left to a target width using a string pad or a non-empty list of non-empty string pads; list pads cycle from the first emitted pad character and the final pad piece is truncated to the target character width.

- Attributes: `None`
- Capability: `Pure`

### StringPadRight

Pads strings on the right to a target width using a string pad or a non-empty list of non-empty string pads; list pads cycle from the first emitted pad character and the final pad piece is truncated to the target character width. Values at or above width are unchanged.

- Attributes: `None`
- Capability: `Pure`

### StringPart

Extracts individual characters by 1-based positions.

- Attributes: `None`
- Capability: `Pure`

### StringPosition

Returns 1-based inclusive positions of exact substring matches, including overlaps, in the current first-pass string subset.

- Attributes: `None`
- Capability: `Pure`

### StringQ

Tests whether an expression is a string atom.

- Attributes: `None`
- Capability: `Pure`

### StringRepeat

Repeats a string a non-negative number of times.

- Attributes: `None`
- Capability: `Pure`

### StringReplace

Replaces exact string needles using immediate string rules in the current first-pass string subset.

- Attributes: `None`
- Capability: `Pure`

### StringReverse

Reverses the Unicode code-point characters in a string.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### StringRiffle

Riffles a flat string list with a string separator or {left, separator, right} wrapper, and riffles nested string rows with row and item separators.

- Attributes: `None`
- Capability: `Pure`

### StringSplit

Splits strings using whitespace or exact delimiters in the current first-pass string subset.

- Attributes: `None`
- Capability: `Pure`

### StringStartsQ

Tests whether a string or list of strings starts with an exact substring in the current first-pass string subset.

- Attributes: `None`
- Capability: `Pure`

### StringTake

Extracts a substring using integer counts or inclusive 1-based ranges.

- Attributes: `None`
- Capability: `Pure`

### StringTakeDrop

Returns both the taken and dropped substrings using the current StringTake/StringDrop spec subset.

- Attributes: `None`
- Capability: `Pure`

### StringTemplate

Builds a simple TemplateObject wrapper from a template string in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### StringToByteArray

Converts strings to ByteArray objects using UTF8/UTF-8 encoding.

- Attributes: `None`
- Capability: `Pure`

### StringTrim

Trims whitespace or exact boundary substrings in the current first-pass string subset.

- Attributes: `None`
- Capability: `Pure`

### StringWrap

Greedily word-wraps a string to a positive character width, separating output lines with LF. With "PreserveNewlines" -> True, wraps each LF-delimited line independently and preserves blank or whitespace-only lines verbatim; the default False collapses all whitespace as word separators.

- Attributes: `None`
- Capability: `Pure`
- Limits: normalizes input whitespace runs to single spaces while wrapping; it does not preserve original spacing inside wrapped text.

### SubValues

Lists stored subvalue definitions for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Subfactorial

Computes derangement numbers for non-negative integer indices.

- Attributes: `Listable`, `NumericFunction`, `Protected`, `ReadProtected`
- Capability: `Pure`

### SubsetMap

Replaces selected exact parts using a mapped subset result in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### SubsetQ

Tests subset inclusion for Lists, optionally using SameTest.

- Attributes: `None`
- Capability: `Pure`

### Subsets

Returns subsets of a normal expression in rum's size-major order for scalar, All, Infinity, and range size specs.

- Attributes: `Protected`
- Capability: `Pure`

### Subtract

Subtracts one exact number from another.

- Attributes: `None`
- Capability: `Pure`

### SubtractFrom

Subtracts an evaluated decrement from a held symbol or exact Part target and stores the result.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Success

Represents a success object with associated details in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Sum

Adds expression values over concrete iterator specifications.

- Attributes: `HoldAll`, `Protected`, `ReadProtected`
- Capability: `Pure`

### Switch

Matches one evaluated expression against held form/value pairs.

- Attributes: `HoldRest`
- Capability: `Pure`

### Symbol

Resolves a symbol from a string name using the current context path.

- Attributes: `None`
- Capability: `Pure`

### SymbolName

Returns the short name of a symbol without its context.

- Attributes: `Protected`
- Capability: `Pure`

### SyntaxInformation

Lists the current syntax-information metadata rules for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### SyntaxQ

Tests whether a string parses successfully in the current syntax subset.

- Attributes: `None`
- Capability: `Pure`

### Table

Generates lists using the current first-pass iterator subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### TagSet

Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.

- Attributes: `None`
- Capability: `Pure`

### TagSetDelayed

Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.

- Attributes: `None`
- Capability: `Pure`

### TagUnset

Removes a tagged definition using an explicit tag and lhs.

- Attributes: `None`
- Capability: `Pure`

### Take

Takes structural elements using the current standard sequence-spec subset.

- Attributes: `None`
- Capability: `Pure`

### TakeDrop

Returns both the taken and dropped structural elements using the current standard sequence-spec subset.

- Attributes: `None`
- Capability: `Pure`

### TakeLargestBy

Takes top-level elements with largest evaluated keys in descending key order, preserving tie order.

- Attributes: `None`
- Capability: `Pure`

### TakeList

Splits a normal expression into sequential chunks by non-negative counts.

- Attributes: `None`
- Capability: `Pure`

### TakeSmallestBy

Takes top-level elements with smallest evaluated keys in ascending key order, preserving tie order.

- Attributes: `None`
- Capability: `Pure`

### TakeWhile

Takes the longest prefix satisfying a criterion in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Tally

Tallies structurally same top-level elements, or elements equivalent under a custom binary test, with the first element as representative.

- Attributes: `None`
- Capability: `Pure`
- Limits: the custom binary-test form performs pairwise O(n^2) representative comparisons and uses the first element of each class as representative.

### TemplateApply

Applies a simple named or positional string template in the current first-pass subset; unfilled slots leave the call inert by default, while "Strict" -> True reports them and returns $Failed.

- Attributes: `None`
- Capability: `Pure`

### TemplateObject

Represents the current simple template object wrapper returned by StringTemplate.

- Attributes: `None`
- Capability: `Pure`

### TerminatedEvaluation

Represents a terminated evaluation object in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### TestCreate

Builds a first-pass symbolic test specification.

- Attributes: `HoldAll`
- Capability: `Pure`

### TestReport

Runs one or more first-pass symbolic tests and returns an association summary.

- Attributes: `None`
- Capability: `Clock`

### TextString

Converts expressions to the current canonical text subset, leaving raw strings unquoted.

- Attributes: `None`
- Capability: `Pure`

### TextWords

Splits text into alphanumeric word runs in the current first-pass subset.

- Attributes: `Listable`
- Capability: `Pure`

### Thread

Threads an expression over arguments that share a common head.

- Attributes: `None`
- Capability: `Pure`

### Through

Threads arguments through a functional head in the current first-pass subset.

- Attributes: `None`
- Capability: `Pure`

### Throw

Propagates a nonlocal value, optionally tagged for Catch matching.

- Attributes: `None`
- Capability: `Pure`

### TimeConstrained

Evaluates an expression under the current cooperative exact-numeric timeout subset.

- Attributes: `HoldAll`
- Capability: `Clock`

### TimeObjectQ

Tests whether an expression has a TimeObject head in the current structural subset.

- Attributes: `None`
- Capability: `Pure`

### Times

Multiplies exact numbers and canonically combines multiplicative expressions.

- Attributes: `Listable`, `Orderless`, `Flat`, `OneIdentity`
- Capability: `Pure`

### TimesBy

Multiplies a held symbol or exact Part target by an evaluated factor and stores the result.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Timing

Evaluates an expression and returns {elapsed, value} using the current wall-clock exact-rational subset.

- Attributes: `HoldAll`
- Capability: `Clock`

### ToBase64

Encodes a string's UTF-8 bytes or a ByteArray as standard RFC 4648 base64 with padding.

- Attributes: `Protected`
- Capability: `Pure`

### ToCharacterCode

Returns Unicode scalar values for strings, or UTF-8 byte values when given the "UTF8" or "UTF-8" encoding argument.

- Attributes: `Listable`
- Capability: `Pure`

### ToCsv

Renders Lists of String rows or String-keyed Association rows as CSV text.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only String fields from list rows or String-keyed Association rows; Association rows imply a header row and must share the same keys. Delimiter must be one non-quote, non-newline character.

### ToExpression

Parses and evaluates input strings using the current InputForm/StandardForm-compatible subset.

- Attributes: `None`
- Capability: `Pure`

### ToHexString

Encodes a string's UTF-8 bytes or a ByteArray as lowercase hexadecimal text.

- Attributes: `Protected`
- Capability: `Pure`

### ToIni

Renders an Association of section Associations as rum's INI dialect.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only Association-of-Associations with String section names, keys, and values; unrepresentable names, multiline values, whitespace-trimming value loss, and an empty default section fail closed.

### ToJson

Renders the JSON-representable subset of rum values as a JSON text string.

- Attributes: `Protected`
- Capability: `Pure`

### ToLowerCase

Converts a string to lowercase using the current Unicode-aware subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### ToString

Renders an evaluated expression into the current canonical kernel text form.

- Attributes: `None`
- Capability: `Pure`

### ToToml

Renders the TOML-representable subset of rum Associations as TOML text.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only top-level Associations with String keys, exact integers, terminating rationals, booleans, strings, Lists, nested Associations, and Lists of Associations; nested Associations use sub-table headers unless inline form is required to preserve same-level key order. It never synthesizes TOML datetime syntax, so date Lists and timestamps encode as arrays or numbers.

### ToUpperCase

Converts a string or list of strings to uppercase using the current Unicode-aware subset.

- Attributes: `Listable`, `Protected`
- Capability: `Pure`

### ToXml

Renders a well-formed XmlElement tree as XML text.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only well-formed XmlElement[tag_String, attrs_Association, children_List] trees with String attributes and String/XmlElement children; names use the documented ASCII XML-name subset, compact output is default, and Indent pretty-prints only element-only content.

### ToYaml

Renders the YAML-representable subset of rum values as YAML text.

- Attributes: `Protected`
- Capability: `Pure`
- Limits: renders only Associations with String keys, Lists, exact integers, terminating rationals, True, False, Null, and Strings; safe plain strings use the documented Norway-safe rule, other strings are double-quoted or literal blocks, and unsupported YAML surface is never synthesized.

### Together

Combines rational-function terms over a common polynomial denominator.

- Attributes: `None`
- Capability: `Pure`

### TokenizeWith

Scans a string with string-pattern rules and returns total 1-based token spans.

- Attributes: `None`
- Capability: `Pure`

### Total

Totals list elements through the current exact Plus semantics.

- Attributes: `None`
- Capability: `Pure`

### Trace

Evaluates an expression and returns a first-pass structured trace of current evaluation events.

- Attributes: `HoldAll`
- Capability: `Pure`

### TracePrint

Evaluates an expression, emits its current first-pass structured trace through the print stream, and returns the result.

- Attributes: `HoldAll`
- Capability: `Pure`

### TraditionalForm

Represents the current formatting symbol accepted by the current ToString subset.

- Attributes: `None`
- Capability: `Pure`

### Transpose

Transposes rectangular plain-list matrices and tensors using identity, first-two-swap, or rank-matching positive permutations.

- Attributes: `None`
- Capability: `Pure`

### TrueQ

Tests whether an expression evaluates exactly to True.

- Attributes: `None`
- Capability: `Pure`

### Tuples

Returns repeated tuples or cartesian products for normal expressions in the current subset.

- Attributes: `Protected`
- Capability: `Pure`

### Uncompress

Recovers values from the current project-local compressed string envelope; malformed envelopes stay inert by default, while "Strict" -> True reports them and returns $Failed.

- Attributes: `None`
- Capability: `Pure`

### Unequal

Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.

- Attributes: `None`
- Capability: `Pure`

### UnequalTo

Represents a unary comparison operator equivalent to Unequal[#, bound] when applied.

- Attributes: `None`
- Capability: `Pure`

### Unevaluated

Temporarily leaves an argument unevaluated as a direct function argument.

- Attributes: `HoldAllComplete`
- Capability: `Pure`

### Union

Returns sorted unique elements across same-head normal expressions; SameTest keeps the first representative in structural scan order.

- Attributes: `Flat`, `OneIdentity`, `Protected`, `ReadProtected`
- Capability: `Pure`

### Unique

Creates fresh symbols using prefix, list, and attribute forms.

- Attributes: `None`
- Capability: `Pure`

### Unprotect

Removes the Protected attribute from one or more symbols.

- Attributes: `HoldAll`
- Capability: `Pure`

### UnsameQ

Tests whether no two arguments are exactly identical after evaluation.

- Attributes: `None`
- Capability: `Pure`

### Unset

Removes a specific stored definition or message template.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Until

Evaluates a body until its test yields True.

- Attributes: `HoldAll`
- Capability: `Pure`

### UpSet

Stores an immediate upvalue definition on one or more tag symbols inferred from the lhs arguments.

- Attributes: `None`
- Capability: `Pure`

### UpSetDelayed

Stores a delayed upvalue definition on one or more tag symbols inferred from the lhs arguments.

- Attributes: `None`
- Capability: `Pure`

### UpValues

Lists stored tagged upvalue definitions for a symbol.

- Attributes: `HoldFirst`
- Capability: `Pure`

### UpperCaseQ

Tests whether every character in a string is uppercase.

- Attributes: `Protected`
- Capability: `Pure`

### UppercaseLetters

Evaluates to the uppercase ASCII letter character-class string.

- Attributes: `Protected`
- Capability: `Pure`

### UrlDecode

Decodes percent escapes in a UTF-8 URL string, returning $Failed for invalid escapes or invalid UTF-8; with "FormEncoding" -> True, + decodes to a space.

- Attributes: `None`
- Capability: `Pure`

### UrlEncode

Percent-encodes a string using UTF-8 bytes and the RFC 3986 unreserved character set; with "FormEncoding" -> True, spaces are encoded as + for form query data.

- Attributes: `None`
- Capability: `Pure`

### UrlParse

Parses scheme://host[:port][/path][?query][#fragment] into an association of string fields; "DefaultScheme" can supply a scheme for schemeless inputs, and "DecodeQuery" decodes query text using form encoding.

- Attributes: `None`
- Capability: `Pure`
- Limits: without "DefaultScheme", accepts only a strict scheme://host[:port][/path][?query][#fragment] shape with an ASCII scheme and decimal port; with "DecodeQuery" -> True, query text is decoded as application/x-www-form-urlencoded data.

### UuidQ

Tests whether a string has version-4 UUID format and variant bits.

- Attributes: `None`
- Capability: `Pure`

### ValueQ

Tests whether an expression has a value by trial evaluation without committing side effects.

- Attributes: `HoldFirst`
- Capability: `Pure`

### Values

Returns association values in structural order.

- Attributes: `None`
- Capability: `Pure`

### VectorQ

Tests whether an expression is a one-dimensional List array, optionally requiring every leaf to satisfy a test.

- Attributes: `None`
- Capability: `Pure`

### Which

Evaluates conditions in order and returns the first matching branch.

- Attributes: `HoldAll`
- Capability: `Pure`

### While

Evaluates a body while its test yields True.

- Attributes: `HoldAll`
- Capability: `Pure`

### Whitespace

Represents the current string-pattern atom for one or more whitespace characters.

- Attributes: `None`
- Capability: `Pure`

### WhitespaceCharacter

Represents the current string-pattern atom for one whitespace character.

- Attributes: `Protected`
- Capability: `Pure`

### With

Lexically substitutes local constants into a body.

- Attributes: `HoldAll`
- Capability: `Pure`

### WithCleanup

Evaluates cleanup code even when the main expression exits abnormally in the current first-pass subset.

- Attributes: `HoldAll`
- Capability: `Pure`

### WordBoundary

Represents the current string-pattern anchor at a transition between word and non-word text.

- Attributes: `Protected`
- Capability: `Pure`

### WordCharacter

Represents the current string-pattern atom for one alphabetic, ASCII digit, or underscore character.

- Attributes: `Protected`
- Capability: `Pure`

### XmlElement

Represents an inert XML element as XmlElement[tag, attrs, children].

- Attributes: `Protected`
- Capability: `Pure`

### Xor

Evaluates exclusive-or over exact boolean arguments and reduces duplicate symbolic terms in the current subset.

- Attributes: `Orderless`, `Flat`, `OneIdentity`
- Capability: `Pure`

