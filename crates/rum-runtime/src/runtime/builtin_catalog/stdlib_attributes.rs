use std::sync::OnceLock;

use super::*;

type Spec = BuiltinAttributeSpec;

const ATTR_FLAT_ONE_IDENTITY: AttributeSet = AttributeSet::empty()
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity);

const ATTR_FLAT_ONE_IDENTITY_PROTECTED_READ_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

const ATTR_HOLD_ALL: AttributeSet = AttributeSet::from_hold(HoldAttribute::HoldAll);

const ATTR_HOLD_ALL_COMPLETE_PROTECTED: AttributeSet =
    AttributeSet::from_attribute(Attribute::HoldAllComplete).with(Attribute::Protected);

const ATTR_HOLD_ALL_PROTECTED: AttributeSet =
    AttributeSet::from_hold(HoldAttribute::HoldAll).with(Attribute::Protected);

const ATTR_HOLD_ALL_PROTECTED_READ_PROTECTED: AttributeSet =
    AttributeSet::from_hold(HoldAttribute::HoldAll)
        .with(Attribute::Protected)
        .with(Attribute::ReadProtected);

const ATTR_HOLD_FIRST: AttributeSet = AttributeSet::from_hold(HoldAttribute::HoldFirst);

const ATTR_HOLD_REST: AttributeSet = AttributeSet::from_hold(HoldAttribute::HoldRest);

const ATTR_HOLD_REST_PROTECTED: AttributeSet =
    AttributeSet::from_hold(HoldAttribute::HoldRest).with(Attribute::Protected);

const ATTR_LISTABLE: AttributeSet = AttributeSet::empty().with(Attribute::Listable);

const ATTR_LISTABLE_N_HOLD_ALL_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::NHoldAll)
    .with(Attribute::Protected);

const ATTR_LISTABLE_NUMERIC_FUNCTION_ORDERLESS_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::NumericFunction)
    .with(Attribute::Orderless)
    .with(Attribute::Protected);

const ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::NumericFunction)
    .with(Attribute::Protected);

const ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED_READ_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::NumericFunction)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

const ATTR_LISTABLE_ORDERLESS_FLAT_ONE_IDENTITY: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::Orderless)
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity);

const ATTR_LISTABLE_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::Protected);

const ATTR_LISTABLE_PROTECTED_READ_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

const ATTR_NON_THREADABLE_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::NonThreadable)
    .with(Attribute::Protected);

const ATTR_NUMERIC_FUNCTION_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::NumericFunction)
    .with(Attribute::Protected);

const ATTR_ORDERLESS: AttributeSet = AttributeSet::empty().with(Attribute::Orderless);

const ATTR_ORDERLESS_FLAT_ONE_IDENTITY: AttributeSet = AttributeSet::empty()
    .with(Attribute::Orderless)
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity);

const ATTR_PROTECTED: AttributeSet = AttributeSet::empty().with(Attribute::Protected);

const ATTR_PROTECTED_READ_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

const STDLIB_NON_EMPTY_ATTRIBUTE_SPECS: &[Spec] = &[
    Spec::new("Abort", ATTR_PROTECTED),
    Spec::new("AbortProtect", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("Abs", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("Accumulate", ATTR_PROTECTED),
    Spec::new(
        "AlgebraicIntegerQ",
        ATTR_LISTABLE.with(Attribute::Protected),
    ),
    Spec::new("AlignColumns", ATTR_PROTECTED),
    Spec::new("AllTrue", ATTR_PROTECTED),
    Spec::new("Alphabet", ATTR_PROTECTED),
    Spec::new("AlphabeticOrder", ATTR_PROTECTED),
    Spec::new("AlphabeticSort", ATTR_PROTECTED),
    Spec::new("Alphanumeric", ATTR_PROTECTED),
    Spec::new("Alternatives", ATTR_PROTECTED),
    Spec::new("And", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("AnyTrue", ATTR_PROTECTED),
    Spec::new("Append", ATTR_PROTECTED),
    Spec::new("Apply", ATTR_PROTECTED),
    Spec::new("Array", ATTR_PROTECTED),
    Spec::new("ArrayFilter", ATTR_PROTECTED),
    Spec::new("ArrayQ", ATTR_PROTECTED),
    Spec::new("ArrayReduce", ATTR_PROTECTED),
    Spec::new("Assert", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new(
        "AssociateTo",
        ATTR_HOLD_FIRST
            .with(Attribute::SubValuesHoldAll)
            .with(Attribute::Protected),
    ),
    Spec::new("Association", ATTR_PROTECTED),
    Spec::new("AssociationKeyDeflatten", ATTR_PROTECTED),
    Spec::new("AssociationKeyFlatten", ATTR_PROTECTED),
    Spec::new("AssociationMap", ATTR_PROTECTED),
    Spec::new("AssociationQ", ATTR_PROTECTED),
    Spec::new("AssociationThread", ATTR_PROTECTED),
    Spec::new("AtomQ", ATTR_PROTECTED),
    Spec::new("BernoulliB", ATTR_LISTABLE_PROTECTED),
    Spec::new("Between", ATTR_PROTECTED),
    Spec::new("BinarySearch", ATTR_PROTECTED),
    Spec::new("Binomial", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("Blank", ATTR_PROTECTED),
    Spec::new("BlankNullSequence", ATTR_PROTECTED),
    Spec::new("BlankSequence", ATTR_PROTECTED),
    Spec::new("BlockMap", ATTR_PROTECTED),
    Spec::new("BlockRandom", ATTR_HOLD_ALL_PROTECTED),
    Spec::new("Boole", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("BooleanQ", ATTR_PROTECTED),
    Spec::new("Break", ATTR_PROTECTED),
    Spec::new("ByteArrayQ", ATTR_PROTECTED),
    Spec::new("ByteArrayToString", ATTR_PROTECTED),
    Spec::new("Capitalize", ATTR_LISTABLE_PROTECTED_READ_PROTECTED),
    Spec::new("Cases", ATTR_PROTECTED),
    Spec::new(
        "CatalanNumber",
        ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED_READ_PROTECTED,
    ),
    Spec::new("Catch", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("CatchExceptions", ATTR_HOLD_ALL_PROTECTED),
    Spec::new("Catenate", ATTR_PROTECTED),
    Spec::new("Ceiling", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("CharacterCounts", ATTR_PROTECTED),
    Spec::new("CharacterRange", ATTR_PROTECTED),
    Spec::new("Characters", ATTR_PROTECTED),
    Spec::new("CheckAbort", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("Coefficient", ATTR_PROTECTED),
    Spec::new("CoefficientList", ATTR_PROTECTED),
    Spec::new("Comap", ATTR_PROTECTED),
    Spec::new("ComapApply", ATTR_PROTECTED),
    Spec::new("Commonest", ATTR_PROTECTED),
    Spec::new("Complement", ATTR_PROTECTED),
    Spec::new(
        "Composition",
        ATTR_FLAT_ONE_IDENTITY.with(Attribute::Protected),
    ),
    Spec::new(
        "CompoundExpression",
        ATTR_HOLD_ALL.with(Attribute::Protected),
    ),
    Spec::new("Compress", ATTR_PROTECTED),
    Spec::new("Confirm", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("ConfirmAssert", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("ConfirmBy", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("ConfirmMatch", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("ConfirmQuiet", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("ConformSchema", ATTR_HOLD_REST_PROTECTED),
    Spec::new("ConstantArray", ATTR_PROTECTED),
    Spec::new("Construct", ATTR_PROTECTED),
    Spec::new("ContainsAll", ATTR_PROTECTED),
    Spec::new("ContainsAny", ATTR_PROTECTED),
    Spec::new("ContainsExactly", ATTR_PROTECTED),
    Spec::new("ContainsNone", ATTR_PROTECTED),
    Spec::new("ContainsOnly", ATTR_PROTECTED),
    Spec::new("Continue", ATTR_PROTECTED),
    Spec::new("CoprimeQ", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Counts", ATTR_PROTECTED),
    Spec::new("CountsBy", ATTR_PROTECTED),
    Spec::new("Crc32", ATTR_PROTECTED),
    Spec::new("CurryApplied", ATTR_PROTECTED),
    Spec::new("D", ATTR_HOLD_REST.with(Attribute::Protected)),
    Spec::new("DateDifference", ATTR_PROTECTED),
    Spec::new("DateList", ATTR_PROTECTED),
    Spec::new("DateObjectQ", ATTR_PROTECTED),
    Spec::new("DatePlus", ATTR_PROTECTED),
    Spec::new("DateString", ATTR_PROTECTED),
    Spec::new("DayOfWeek", ATTR_PROTECTED),
    Spec::new("DaysInMonth", ATTR_PROTECTED),
    Spec::new("Decapitalize", ATTR_LISTABLE_PROTECTED_READ_PROTECTED),
    Spec::new("DecimalString", ATTR_PROTECTED),
    Spec::new("Delete", ATTR_PROTECTED),
    Spec::new("DeleteCases", ATTR_PROTECTED),
    Spec::new("DeleteDuplicates", ATTR_PROTECTED),
    Spec::new("DeleteDuplicatesBy", ATTR_PROTECTED),
    Spec::new("DeleteMissing", ATTR_PROTECTED),
    Spec::new("Denominator", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Depth", ATTR_PROTECTED),
    Spec::new("Diff", ATTR_PROTECTED),
    Spec::new("Differences", ATTR_NON_THREADABLE_PROTECTED),
    Spec::new("DigitCharacter", ATTR_PROTECTED),
    Spec::new("DigitCharacters", ATTR_PROTECTED),
    Spec::new("DigitCount", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("DigitQ", ATTR_PROTECTED),
    Spec::new("DisjointQ", ATTR_PROTECTED),
    Spec::new("Divide", ATTR_PROTECTED),
    Spec::new("DivisorSigma", ATTR_LISTABLE_N_HOLD_ALL_PROTECTED),
    Spec::new("Divisors", ATTR_LISTABLE_PROTECTED),
    Spec::new("Do", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("Dot", ATTR_PROTECTED),
    Spec::new("Drop", ATTR_PROTECTED),
    Spec::new("DuplicateFreeQ", ATTR_PROTECTED),
    Spec::new("Duplicates", ATTR_PROTECTED),
    Spec::new("EditDistance", ATTR_PROTECTED),
    Spec::new("Enclose", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("EndOfLine", ATTR_PROTECTED),
    Spec::new("EndOfString", ATTR_PROTECTED),
    Spec::new("Equal", ATTR_PROTECTED),
    Spec::new("EqualTo", ATTR_PROTECTED),
    Spec::new("Equivalent", ATTR_ORDERLESS.with(Attribute::Protected)),
    Spec::new("EulerPhi", ATTR_LISTABLE_PROTECTED),
    Spec::new("EvenQ", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Except", ATTR_PROTECTED),
    Spec::new("Exception", ATTR_PROTECTED),
    Spec::new("ExceptionTypes", ATTR_PROTECTED),
    Spec::new("Exit", ATTR_PROTECTED),
    Spec::new("Expand", ATTR_PROTECTED),
    Spec::new("Factor", ATTR_PROTECTED),
    Spec::new("Factorial", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Factorial2", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Failsafe", ATTR_PROTECTED),
    Spec::new("Failure", ATTR_PROTECTED),
    Spec::new("FailureQ", ATTR_PROTECTED),
    Spec::new(
        "Fibonacci",
        ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED_READ_PROTECTED,
    ),
    Spec::new("FileBaseName", ATTR_PROTECTED),
    Spec::new("FileExtension", ATTR_PROTECTED),
    Spec::new("FileNameSplit", ATTR_PROTECTED),
    Spec::new("First", ATTR_HOLD_REST.with(Attribute::Protected)),
    Spec::new("FixedPoint", ATTR_PROTECTED),
    Spec::new("FixedPointList", ATTR_PROTECTED),
    Spec::new("Flatten", ATTR_PROTECTED),
    Spec::new("Floor", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("Fold", ATTR_PROTECTED),
    Spec::new("FoldList", ATTR_PROTECTED),
    Spec::new("FoldPair", ATTR_PROTECTED),
    Spec::new("FoldWhile", ATTR_PROTECTED),
    Spec::new("For", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("FractionalPart", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("FromBase64", ATTR_PROTECTED),
    Spec::new("FromCsv", ATTR_PROTECTED),
    Spec::new("FromDateList", ATTR_PROTECTED),
    Spec::new("FromDateString", ATTR_PROTECTED),
    Spec::new("FromDigits", ATTR_PROTECTED),
    Spec::new("FromHexString", ATTR_PROTECTED),
    Spec::new("FromIni", ATTR_PROTECTED),
    Spec::new("FromJson", ATTR_PROTECTED),
    Spec::new("FromToml", ATTR_PROTECTED),
    Spec::new("FromXml", ATTR_PROTECTED),
    Spec::new("FromYaml", ATTR_PROTECTED),
    Spec::new("FullForm", ATTR_PROTECTED),
    Spec::new("GCD", ATTR_LISTABLE_PROTECTED),
    Spec::new("GatherBy", ATTR_PROTECTED),
    Spec::new("Greater", ATTR_PROTECTED),
    Spec::new("GreaterEqual", ATTR_PROTECTED),
    Spec::new("GreaterEqualThan", ATTR_PROTECTED),
    Spec::new("GreaterThan", ATTR_PROTECTED),
    Spec::new("GroupBy", ATTR_PROTECTED),
    Spec::new("HammingDistance", ATTR_PROTECTED),
    Spec::new("Hash", ATTR_PROTECTED),
    Spec::new("Head", ATTR_PROTECTED),
    Spec::new("HtmlEscape", ATTR_PROTECTED),
    Spec::new("HtmlUnescape", ATTR_PROTECTED),
    Spec::new("Identity", ATTR_PROTECTED),
    Spec::new("If", ATTR_HOLD_REST.with(Attribute::Protected)),
    Spec::new("Implies", ATTR_PROTECTED),
    Spec::new("Inner", ATTR_PROTECTED),
    Spec::new("Insert", ATTR_PROTECTED),
    Spec::new("IntegerDigits", ATTR_LISTABLE_PROTECTED),
    Spec::new("IntegerLength", ATTR_LISTABLE_PROTECTED),
    Spec::new("IntegerPart", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("IntegerQ", ATTR_PROTECTED),
    Spec::new("IntegerReverse", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("IntersectingQ", ATTR_PROTECTED),
    Spec::new(
        "Intersection",
        ATTR_FLAT_ONE_IDENTITY_PROTECTED_READ_PROTECTED,
    ),
    Spec::new("Join", ATTR_PROTECTED),
    Spec::new("JoinAcross", ATTR_PROTECTED),
    Spec::new("Key", ATTR_PROTECTED),
    Spec::new("KeyComplement", ATTR_PROTECTED),
    Spec::new("KeyDrop", ATTR_PROTECTED),
    Spec::new(
        "KeyDropFrom",
        ATTR_HOLD_FIRST
            .with(Attribute::SubValuesHoldAll)
            .with(Attribute::Protected),
    ),
    Spec::new("KeyExistsQ", ATTR_PROTECTED),
    Spec::new("KeyFreeQ", ATTR_PROTECTED),
    Spec::new("KeyGroupBy", ATTR_PROTECTED),
    Spec::new("KeyIntersection", ATTR_PROTECTED),
    Spec::new("KeyMap", ATTR_PROTECTED),
    Spec::new("KeyMemberQ", ATTR_PROTECTED),
    Spec::new("KeySelect", ATTR_PROTECTED),
    Spec::new("KeySort", ATTR_PROTECTED),
    Spec::new("KeySortBy", ATTR_PROTECTED),
    Spec::new("KeyTake", ATTR_PROTECTED),
    Spec::new("KeyUnion", ATTR_PROTECTED),
    Spec::new("KeyValueMap", ATTR_PROTECTED),
    Spec::new("KeyValuePattern", ATTR_PROTECTED),
    Spec::new("Keys", ATTR_PROTECTED),
    Spec::new("LCM", ATTR_LISTABLE_PROTECTED),
    Spec::new("Last", ATTR_HOLD_REST.with(Attribute::Protected)),
    Spec::new("LazyDrop", ATTR_PROTECTED),
    Spec::new("LazyFoldList", ATTR_PROTECTED),
    Spec::new("LazyList", ATTR_HOLD_ALL_COMPLETE_PROTECTED),
    Spec::new("LazyMap", ATTR_PROTECTED),
    Spec::new("LazyNestList", ATTR_PROTECTED),
    Spec::new("LazyPermutations", ATTR_PROTECTED),
    Spec::new("LazyRange", ATTR_PROTECTED),
    Spec::new("LazySelect", ATTR_PROTECTED),
    Spec::new("LazySubsets", ATTR_PROTECTED),
    Spec::new("LazyTable", ATTR_HOLD_ALL_PROTECTED),
    Spec::new("LazyTake", ATTR_PROTECTED),
    Spec::new("LazyTuples", ATTR_PROTECTED),
    Spec::new("LeafCount", ATTR_PROTECTED),
    Spec::new("LeapYearQ", ATTR_PROTECTED),
    Spec::new("Length", ATTR_PROTECTED),
    Spec::new("LengthWhile", ATTR_PROTECTED),
    Spec::new("Less", ATTR_PROTECTED),
    Spec::new("LessEqual", ATTR_PROTECTED),
    Spec::new("LessEqualThan", ATTR_PROTECTED),
    Spec::new("LessThan", ATTR_PROTECTED),
    Spec::new("LetterCharacter", ATTR_PROTECTED),
    Spec::new("LetterQ", ATTR_PROTECTED),
    Spec::new("LexicographicOrder", ATTR_PROTECTED),
    Spec::new("ListQ", ATTR_PROTECTED),
    Spec::new("LongestCommonSubsequence", ATTR_PROTECTED),
    Spec::new("Lookup", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("LowerCaseQ", ATTR_PROTECTED),
    Spec::new("LowercaseLetters", ATTR_PROTECTED),
    Spec::new("MachineNumberQ", ATTR_PROTECTED),
    Spec::new("Map", ATTR_PROTECTED),
    Spec::new("MapAll", ATTR_PROTECTED),
    Spec::new("MapApply", ATTR_PROTECTED),
    Spec::new("MapAt", ATTR_PROTECTED),
    Spec::new("MapIndexed", ATTR_PROTECTED),
    Spec::new("MapThread", ATTR_PROTECTED),
    Spec::new("MatchQ", ATTR_PROTECTED),
    Spec::new("MatrixQ", ATTR_PROTECTED),
    Spec::new("Max", ATTR_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("MaxBy", ATTR_PROTECTED),
    Spec::new("MaximalBy", ATTR_PROTECTED),
    Spec::new("Md5", ATTR_PROTECTED),
    Spec::new("Mean", ATTR_PROTECTED),
    Spec::new("Median", ATTR_PROTECTED),
    Spec::new("Merge", ATTR_PROTECTED),
    Spec::new("MergeNested", ATTR_PROTECTED),
    Spec::new("Min", ATTR_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("MinBy", ATTR_PROTECTED),
    Spec::new("MinMax", ATTR_PROTECTED),
    Spec::new("MinimalBy", ATTR_PROTECTED),
    Spec::new("Minus", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("Missing", ATTR_PROTECTED),
    Spec::new("Mod", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("MoebiusMu", ATTR_LISTABLE_PROTECTED),
    Spec::new("Most", ATTR_PROTECTED),
    Spec::new(
        "Multinomial",
        ATTR_LISTABLE_NUMERIC_FUNCTION_ORDERLESS_PROTECTED,
    ),
    Spec::new("Nand", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("Nearest", ATTR_PROTECTED),
    Spec::new("Negative", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Nest", ATTR_PROTECTED),
    Spec::new("NestList", ATTR_PROTECTED),
    Spec::new("NestWhile", ATTR_PROTECTED),
    Spec::new("NestWhileList", ATTR_PROTECTED),
    Spec::new("NextPrime", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("NonNegative", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("NonPositive", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("NoneTrue", ATTR_PROTECTED),
    Spec::new("Nor", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("Normal", ATTR_PROTECTED),
    Spec::new("Not", ATTR_PROTECTED),
    Spec::new("NumberQ", ATTR_PROTECTED),
    Spec::new("Numerator", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("NumericQ", ATTR_PROTECTED),
    Spec::new("NumericalOrder", ATTR_PROTECTED),
    Spec::new("OddQ", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Operate", ATTR_PROTECTED),
    Spec::new("OperatorApplied", ATTR_PROTECTED),
    Spec::new("Or", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("Order", ATTR_PROTECTED),
    Spec::new("OrderedQ", ATTR_PROTECTED),
    Spec::new("Outer", ATTR_PROTECTED),
    Spec::new("PadLeft", ATTR_PROTECTED),
    Spec::new("PadRight", ATTR_PROTECTED),
    Spec::new("ParseArgs", ATTR_PROTECTED),
    Spec::new("Partition", ATTR_PROTECTED),
    Spec::new("PartitionsP", ATTR_LISTABLE_PROTECTED),
    Spec::new("PartitionsQ", ATTR_LISTABLE_PROTECTED),
    Spec::new("Pattern", ATTR_PROTECTED),
    Spec::new("Permutations", ATTR_PROTECTED),
    Spec::new("Pluralize", ATTR_PROTECTED),
    Spec::new(
        "Plus",
        ATTR_LISTABLE_ORDERLESS_FLAT_ONE_IDENTITY.with(Attribute::Protected),
    ),
    Spec::new("Pochhammer", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("PolynomialExpressionQ", ATTR_PROTECTED),
    Spec::new("PolynomialQ", ATTR_PROTECTED),
    Spec::new("PositionIndex", ATTR_PROTECTED),
    Spec::new("Positive", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("PossibleZeroQ", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Power", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("PowerMod", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Prepend", ATTR_PROTECTED),
    Spec::new("Prime", ATTR_LISTABLE_PROTECTED),
    Spec::new("PrimeNu", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("PrimeOmega", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("PrimePi", ATTR_LISTABLE_PROTECTED),
    Spec::new("PrimeQ", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Product", ATTR_HOLD_ALL_PROTECTED_READ_PROTECTED),
    Spec::new("PropagateAborts", ATTR_PROTECTED),
    Spec::new("PunctuationCharacter", ATTR_PROTECTED),
    Spec::new("PunctuationCharacters", ATTR_PROTECTED),
    Spec::new("Quantile", ATTR_PROTECTED),
    Spec::new("Quotient", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("RandomBytes", ATTR_PROTECTED),
    Spec::new("RandomChoice", ATTR_PROTECTED),
    Spec::new("RandomInteger", ATTR_PROTECTED),
    Spec::new("RandomPassword", ATTR_PROTECTED),
    Spec::new("RandomSample", ATTR_PROTECTED),
    Spec::new("RandomString", ATTR_PROTECTED),
    Spec::new("RandomUuid", ATTR_PROTECTED),
    Spec::new("Range", ATTR_PROTECTED),
    Spec::new("RationalExpressionQ", ATTR_PROTECTED),
    Spec::new("RealValuedNumberQ", ATTR_PROTECTED),
    Spec::new("RealValuedNumericQ", ATTR_PROTECTED),
    Spec::new("RegisterExceptionType", ATTR_PROTECTED),
    Spec::new("RegularExpression", ATTR_PROTECTED),
    Spec::new("RemoveDiacritics", ATTR_PROTECTED),
    Spec::new("Replace", ATTR_PROTECTED),
    Spec::new("ReplaceAll", ATTR_PROTECTED),
    Spec::new("ReplaceAllUnheld", ATTR_PROTECTED),
    Spec::new("ReplaceList", ATTR_PROTECTED),
    Spec::new("ReplaceRepeated", ATTR_PROTECTED),
    Spec::new("ReseedRandom", ATTR_PROTECTED),
    Spec::new("Rest", ATTR_PROTECTED),
    Spec::new("Return", ATTR_PROTECTED),
    Spec::new("Reverse", ATTR_PROTECTED),
    Spec::new("ReverseApplied", ATTR_PROTECTED),
    Spec::new("ReverseSort", ATTR_PROTECTED),
    Spec::new("ReverseSortBy", ATTR_PROTECTED),
    Spec::new("Riffle", ATTR_PROTECTED),
    Spec::new(
        "RightComposition",
        ATTR_FLAT_ONE_IDENTITY.with(Attribute::Protected),
    ),
    Spec::new("Round", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Row", ATTR_PROTECTED),
    Spec::new("RumTokens", ATTR_PROTECTED),
    Spec::new("SameAs", ATTR_PROTECTED),
    Spec::new("SameQ", ATTR_PROTECTED),
    Spec::new("Scan", ATTR_PROTECTED),
    Spec::new("SchemaErrors", ATTR_HOLD_REST_PROTECTED),
    Spec::new("SeedRandom", ATTR_PROTECTED),
    Spec::new("Select", ATTR_PROTECTED),
    Spec::new("SelectFirst", ATTR_PROTECTED),
    Spec::new("SelectPositions", ATTR_PROTECTED),
    Spec::new("SequenceAlignment", ATTR_PROTECTED),
    Spec::new("SequenceFold", ATTR_PROTECTED),
    Spec::new("SequenceFoldList", ATTR_PROTECTED),
    Spec::new("Sha1", ATTR_PROTECTED),
    Spec::new("Sha256", ATTR_PROTECTED),
    Spec::new("Sha512", ATTR_PROTECTED),
    Spec::new("ShellQuote", ATTR_PROTECTED),
    Spec::new("ShellSplit", ATTR_PROTECTED),
    Spec::new("Sign", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("Sort", ATTR_PROTECTED),
    Spec::new("SortBy", ATTR_PROTECTED),
    Spec::new("StartOfLine", ATTR_PROTECTED),
    Spec::new("StartOfString", ATTR_PROTECTED),
    Spec::new("StirlingS1", ATTR_LISTABLE_PROTECTED),
    Spec::new("StirlingS2", ATTR_LISTABLE_PROTECTED),
    Spec::new("StringApply", ATTR_PROTECTED),
    Spec::new("StringCases", ATTR_PROTECTED),
    Spec::new("StringContainsQ", ATTR_PROTECTED),
    Spec::new("StringCount", ATTR_PROTECTED),
    Spec::new("StringDedent", ATTR_PROTECTED),
    Spec::new("StringDelete", ATTR_PROTECTED),
    Spec::new("StringDrop", ATTR_PROTECTED),
    Spec::new("StringEndsQ", ATTR_PROTECTED),
    Spec::new("StringExpandTabs", ATTR_PROTECTED),
    Spec::new("StringExpression", ATTR_PROTECTED),
    Spec::new("StringForm", ATTR_PROTECTED),
    Spec::new("StringFreeQ", ATTR_PROTECTED),
    Spec::new("StringIndent", ATTR_PROTECTED),
    Spec::new("StringJoin", ATTR_PROTECTED),
    Spec::new("StringLength", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("StringLines", ATTR_PROTECTED),
    Spec::new("StringMatchQ", ATTR_PROTECTED),
    Spec::new("StringPadLeft", ATTR_PROTECTED),
    Spec::new("StringPadRight", ATTR_PROTECTED),
    Spec::new("StringPart", ATTR_PROTECTED),
    Spec::new("StringPosition", ATTR_PROTECTED),
    Spec::new("StringQ", ATTR_PROTECTED),
    Spec::new("StringRepeat", ATTR_PROTECTED),
    Spec::new("StringReplace", ATTR_PROTECTED),
    Spec::new("StringReverse", ATTR_LISTABLE_PROTECTED),
    Spec::new("StringRiffle", ATTR_PROTECTED),
    Spec::new("StringSplit", ATTR_PROTECTED),
    Spec::new("StringStartsQ", ATTR_PROTECTED),
    Spec::new("StringTake", ATTR_PROTECTED),
    Spec::new("StringTakeDrop", ATTR_PROTECTED),
    Spec::new("StringTemplate", ATTR_PROTECTED),
    Spec::new("StringToByteArray", ATTR_PROTECTED),
    Spec::new("StringTrim", ATTR_PROTECTED),
    Spec::new("StringWrap", ATTR_PROTECTED),
    Spec::new(
        "Subfactorial",
        ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED_READ_PROTECTED,
    ),
    Spec::new("SubsetMap", ATTR_PROTECTED),
    Spec::new("SubsetQ", ATTR_PROTECTED),
    Spec::new("Subsets", ATTR_PROTECTED),
    Spec::new("Subtract", ATTR_PROTECTED),
    Spec::new("Success", ATTR_PROTECTED),
    Spec::new("Sum", ATTR_HOLD_ALL_PROTECTED_READ_PROTECTED),
    Spec::new("Switch", ATTR_HOLD_REST.with(Attribute::Protected)),
    Spec::new("Symbol", ATTR_PROTECTED),
    Spec::new("SyntaxQ", ATTR_PROTECTED),
    Spec::new("Table", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("Take", ATTR_PROTECTED),
    Spec::new("TakeDrop", ATTR_PROTECTED),
    Spec::new("TakeLargestBy", ATTR_PROTECTED),
    Spec::new("TakeList", ATTR_PROTECTED),
    Spec::new("TakeSmallestBy", ATTR_PROTECTED),
    Spec::new("TakeWhile", ATTR_PROTECTED),
    Spec::new("Tally", ATTR_PROTECTED),
    Spec::new("TemplateApply", ATTR_PROTECTED),
    Spec::new("TerminatedEvaluation", ATTR_PROTECTED),
    Spec::new("TextString", ATTR_PROTECTED),
    Spec::new("TextWords", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("Through", ATTR_PROTECTED),
    Spec::new("Throw", ATTR_PROTECTED),
    Spec::new("ThrowException", ATTR_PROTECTED),
    Spec::new("TimeObjectQ", ATTR_PROTECTED),
    Spec::new(
        "Times",
        ATTR_LISTABLE_ORDERLESS_FLAT_ONE_IDENTITY.with(Attribute::Protected),
    ),
    Spec::new("ToBase64", ATTR_PROTECTED),
    Spec::new("ToCharacterCode", ATTR_LISTABLE.with(Attribute::Protected)),
    Spec::new("ToCsv", ATTR_PROTECTED),
    Spec::new("ToExpression", ATTR_PROTECTED),
    Spec::new("ToHexString", ATTR_PROTECTED),
    Spec::new("ToIni", ATTR_PROTECTED),
    Spec::new("ToJson", ATTR_PROTECTED),
    Spec::new("ToLowerCase", ATTR_LISTABLE_PROTECTED),
    Spec::new("ToString", ATTR_PROTECTED),
    Spec::new("ToToml", ATTR_PROTECTED),
    Spec::new("ToUpperCase", ATTR_LISTABLE_PROTECTED),
    Spec::new("ToXml", ATTR_PROTECTED),
    Spec::new("ToYaml", ATTR_PROTECTED),
    Spec::new("Together", ATTR_PROTECTED),
    Spec::new("TokenizeWith", ATTR_PROTECTED),
    Spec::new("Total", ATTR_PROTECTED),
    Spec::new("Transpose", ATTR_PROTECTED),
    Spec::new("TrueQ", ATTR_PROTECTED),
    Spec::new("Tuples", ATTR_PROTECTED),
    Spec::new("Uncompress", ATTR_PROTECTED),
    Spec::new("Unequal", ATTR_PROTECTED),
    Spec::new("UnequalTo", ATTR_PROTECTED),
    Spec::new("Union", ATTR_FLAT_ONE_IDENTITY_PROTECTED_READ_PROTECTED),
    Spec::new("UnsameQ", ATTR_PROTECTED),
    Spec::new("Until", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("UpperCaseQ", ATTR_PROTECTED),
    Spec::new("UppercaseLetters", ATTR_PROTECTED),
    Spec::new("UrlDecode", ATTR_PROTECTED),
    Spec::new("UrlEncode", ATTR_PROTECTED),
    Spec::new("UrlParse", ATTR_PROTECTED),
    Spec::new("UuidQ", ATTR_PROTECTED),
    Spec::new("Values", ATTR_PROTECTED),
    Spec::new("VectorQ", ATTR_PROTECTED),
    Spec::new("Which", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("While", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("WhitespaceCharacter", ATTR_PROTECTED),
    Spec::new("WhitespaceCharacters", ATTR_PROTECTED),
    Spec::new("WithCleanup", ATTR_HOLD_ALL.with(Attribute::Protected)),
    Spec::new("WordBoundary", ATTR_PROTECTED),
    Spec::new("WordCharacter", ATTR_PROTECTED),
    Spec::new("XmlElement", ATTR_PROTECTED),
    Spec::new(
        "Xor",
        ATTR_ORDERLESS_FLAT_ONE_IDENTITY.with(Attribute::Protected),
    ),
];
const STDLIB_EMPTY_ATTRIBUTE_NAMES: &[&str] = &["$AssertFunction", "Decrypt", "Encrypt"];

// Both tables above are binary-searched by ASCII byte order (str::cmp), so an
// out-of-place insert silently breaks lookups near it. Enforce strict ASCII
// sortedness (which also forbids duplicates) at compile time. The allows are
// needed because dead-code analysis does not root usage from the anonymous
// `const _` assertions below.
#[allow(dead_code)]
const fn str_ascii_lt(lhs: &str, rhs: &str) -> bool {
    let lhs = lhs.as_bytes();
    let rhs = rhs.as_bytes();
    let shorter = if lhs.len() < rhs.len() {
        lhs.len()
    } else {
        rhs.len()
    };
    let mut index = 0;
    while index < shorter {
        if lhs[index] != rhs[index] {
            return lhs[index] < rhs[index];
        }
        index += 1;
    }
    lhs.len() < rhs.len()
}

#[allow(dead_code)]
const fn attribute_specs_strictly_sorted(specs: &[Spec]) -> bool {
    let mut index = 1;
    while index < specs.len() {
        if !str_ascii_lt(specs[index - 1].name, specs[index].name) {
            return false;
        }
        index += 1;
    }
    true
}

#[allow(dead_code)]
const fn attribute_names_strictly_sorted(names: &[&str]) -> bool {
    let mut index = 1;
    while index < names.len() {
        if !str_ascii_lt(names[index - 1], names[index]) {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(
    attribute_specs_strictly_sorted(STDLIB_NON_EMPTY_ATTRIBUTE_SPECS),
    "STDLIB_NON_EMPTY_ATTRIBUTE_SPECS must stay strictly ASCII-sorted (binary-searched)"
);
const _: () = assert!(
    attribute_names_strictly_sorted(STDLIB_EMPTY_ATTRIBUTE_NAMES),
    "STDLIB_EMPTY_ATTRIBUTE_NAMES must stay strictly ASCII-sorted (binary-searched)"
);

pub(super) fn stdlib_builtin_attribute_specs() -> &'static [BuiltinAttributeSpec] {
    static SPECS: OnceLock<Vec<BuiltinAttributeSpec>> = OnceLock::new();

    SPECS
        .get_or_init(|| {
            let mut specs = Vec::with_capacity(
                STDLIB_NON_EMPTY_ATTRIBUTE_SPECS.len() + STDLIB_EMPTY_ATTRIBUTE_NAMES.len(),
            );
            specs.extend_from_slice(STDLIB_NON_EMPTY_ATTRIBUTE_SPECS);
            specs.extend(
                STDLIB_EMPTY_ATTRIBUTE_NAMES
                    .iter()
                    .copied()
                    .map(|name| Spec::new(name, AttributeSet::empty())),
            );
            specs.sort_unstable_by(|lhs, rhs| lhs.name.cmp(rhs.name));
            specs
        })
        .as_slice()
}

pub(super) fn default_stdlib_attribute_set_for_symbol(symbol: &Symbol) -> Option<AttributeSet> {
    if !symbol.is_system() {
        return None;
    }

    let name = symbol.name();
    if let Ok(index) = STDLIB_NON_EMPTY_ATTRIBUTE_SPECS.binary_search_by(|spec| spec.name.cmp(name))
    {
        return Some(STDLIB_NON_EMPTY_ATTRIBUTE_SPECS[index].attributes);
    }
    if STDLIB_EMPTY_ATTRIBUTE_NAMES
        .binary_search_by(|candidate| candidate.cmp(&name))
        .is_ok()
    {
        return Some(AttributeSet::empty());
    }

    None
}
