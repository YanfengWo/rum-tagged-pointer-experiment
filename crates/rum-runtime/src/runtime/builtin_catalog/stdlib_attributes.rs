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

const ATTR_HOLD_ALL_PROTECTED_READ_PROTECTED: AttributeSet =
    AttributeSet::from_hold(HoldAttribute::HoldAll)
        .with(Attribute::Protected)
        .with(Attribute::ReadProtected);

const ATTR_HOLD_FIRST: AttributeSet = AttributeSet::from_hold(HoldAttribute::HoldFirst);

const ATTR_HOLD_REST: AttributeSet = AttributeSet::from_hold(HoldAttribute::HoldRest);

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

const ATTR_NON_THREADABLE_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::NonThreadable)
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
    Spec::new("AbortProtect", ATTR_HOLD_ALL),
    Spec::new("Abs", ATTR_LISTABLE),
    Spec::new("AlgebraicIntegerQ", ATTR_LISTABLE),
    Spec::new("Alphanumeric", ATTR_PROTECTED),
    Spec::new("And", ATTR_HOLD_ALL),
    Spec::new("Assert", ATTR_HOLD_ALL),
    Spec::new("AssociateTo", ATTR_HOLD_FIRST),
    Spec::new("BernoulliB", ATTR_LISTABLE_PROTECTED),
    Spec::new("Binomial", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("BlockRandom", ATTR_HOLD_ALL),
    Spec::new("Boole", ATTR_LISTABLE),
    Spec::new("Capitalize", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new(
        "CatalanNumber",
        ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED_READ_PROTECTED,
    ),
    Spec::new("Catch", ATTR_HOLD_ALL),
    Spec::new("Ceiling", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("CheckAbort", ATTR_HOLD_ALL),
    Spec::new("Composition", ATTR_FLAT_ONE_IDENTITY),
    Spec::new("CompoundExpression", ATTR_HOLD_ALL),
    Spec::new("Confirm", ATTR_HOLD_ALL),
    Spec::new("ConfirmAssert", ATTR_HOLD_ALL),
    Spec::new("ConfirmBy", ATTR_HOLD_ALL),
    Spec::new("ConfirmMatch", ATTR_HOLD_ALL),
    Spec::new("ConfirmQuiet", ATTR_HOLD_ALL),
    Spec::new("ConformSchema", ATTR_HOLD_REST),
    Spec::new("ConstantArray", ATTR_PROTECTED),
    Spec::new("CoprimeQ", ATTR_LISTABLE),
    Spec::new("D", ATTR_HOLD_REST),
    Spec::new("Decapitalize", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("Denominator", ATTR_LISTABLE),
    Spec::new("Differences", ATTR_NON_THREADABLE_PROTECTED),
    Spec::new("DigitCount", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("DigitCharacters", ATTR_PROTECTED),
    Spec::new("DivisorSigma", ATTR_LISTABLE_N_HOLD_ALL_PROTECTED),
    Spec::new("Divisors", ATTR_LISTABLE_PROTECTED),
    Spec::new("Do", ATTR_HOLD_ALL),
    Spec::new("Enclose", ATTR_HOLD_ALL),
    Spec::new("Equivalent", ATTR_ORDERLESS),
    Spec::new("EulerPhi", ATTR_LISTABLE_PROTECTED),
    Spec::new("EvenQ", ATTR_LISTABLE),
    Spec::new("Factorial", ATTR_LISTABLE),
    Spec::new("Factorial2", ATTR_LISTABLE),
    Spec::new(
        "Fibonacci",
        ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED_READ_PROTECTED,
    ),
    Spec::new("First", ATTR_HOLD_REST),
    Spec::new("Floor", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("For", ATTR_HOLD_ALL),
    Spec::new("FractionalPart", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("FromDigits", ATTR_PROTECTED),
    Spec::new("GCD", ATTR_LISTABLE),
    Spec::new("If", ATTR_HOLD_REST),
    Spec::new("IntegerDigits", ATTR_LISTABLE_PROTECTED),
    Spec::new("IntegerLength", ATTR_LISTABLE_PROTECTED),
    Spec::new("IntegerPart", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("IntegerReverse", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new(
        "Intersection",
        ATTR_FLAT_ONE_IDENTITY_PROTECTED_READ_PROTECTED,
    ),
    Spec::new("KeyDropFrom", ATTR_HOLD_FIRST),
    Spec::new("LCM", ATTR_LISTABLE),
    Spec::new("Last", ATTR_HOLD_REST),
    Spec::new("LeafCount", ATTR_PROTECTED),
    Spec::new("Lookup", ATTR_HOLD_ALL),
    Spec::new("LowerCaseQ", ATTR_PROTECTED),
    Spec::new("LowercaseLetters", ATTR_PROTECTED),
    Spec::new("MinMax", ATTR_PROTECTED),
    Spec::new("Minus", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("Mod", ATTR_LISTABLE),
    Spec::new("MoebiusMu", ATTR_LISTABLE_PROTECTED),
    Spec::new(
        "Multinomial",
        ATTR_LISTABLE_NUMERIC_FUNCTION_ORDERLESS_PROTECTED,
    ),
    Spec::new("Nand", ATTR_HOLD_ALL),
    Spec::new("Negative", ATTR_LISTABLE),
    Spec::new("NextPrime", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("NonNegative", ATTR_LISTABLE),
    Spec::new("NonPositive", ATTR_LISTABLE),
    Spec::new("Nor", ATTR_HOLD_ALL),
    Spec::new("Numerator", ATTR_LISTABLE),
    Spec::new("OddQ", ATTR_LISTABLE),
    Spec::new("Or", ATTR_HOLD_ALL),
    Spec::new("PadLeft", ATTR_PROTECTED),
    Spec::new("PadRight", ATTR_PROTECTED),
    Spec::new("Partition", ATTR_PROTECTED),
    Spec::new("PartitionsP", ATTR_LISTABLE_PROTECTED),
    Spec::new("PartitionsQ", ATTR_LISTABLE_PROTECTED),
    Spec::new("Plus", ATTR_LISTABLE_ORDERLESS_FLAT_ONE_IDENTITY),
    Spec::new("Pochhammer", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("Positive", ATTR_LISTABLE),
    Spec::new("PossibleZeroQ", ATTR_LISTABLE),
    Spec::new("Power", ATTR_LISTABLE),
    Spec::new("PowerMod", ATTR_LISTABLE),
    Spec::new("Prime", ATTR_LISTABLE_PROTECTED),
    Spec::new("PrimeNu", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("PrimeOmega", ATTR_PROTECTED_READ_PROTECTED),
    Spec::new("PrimePi", ATTR_LISTABLE_PROTECTED),
    Spec::new("PrimeQ", ATTR_LISTABLE),
    Spec::new("Product", ATTR_HOLD_ALL_PROTECTED_READ_PROTECTED),
    Spec::new("PunctuationCharacters", ATTR_PROTECTED),
    Spec::new("Quotient", ATTR_LISTABLE),
    Spec::new("RightComposition", ATTR_FLAT_ONE_IDENTITY),
    Spec::new("Round", ATTR_LISTABLE),
    Spec::new("SchemaErrors", ATTR_HOLD_REST),
    Spec::new("Sign", ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED),
    Spec::new("StirlingS1", ATTR_LISTABLE_PROTECTED),
    Spec::new("StirlingS2", ATTR_LISTABLE_PROTECTED),
    Spec::new("StringLength", ATTR_LISTABLE),
    Spec::new("StringReverse", ATTR_LISTABLE_PROTECTED),
    Spec::new(
        "Subfactorial",
        ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED_READ_PROTECTED,
    ),
    Spec::new("Subsets", ATTR_PROTECTED),
    Spec::new("Sum", ATTR_HOLD_ALL_PROTECTED_READ_PROTECTED),
    Spec::new("Switch", ATTR_HOLD_REST),
    Spec::new("Table", ATTR_HOLD_ALL),
    Spec::new("TextWords", ATTR_LISTABLE),
    Spec::new("Times", ATTR_LISTABLE_ORDERLESS_FLAT_ONE_IDENTITY),
    Spec::new("ToCharacterCode", ATTR_LISTABLE),
    Spec::new("ToLowerCase", ATTR_LISTABLE_PROTECTED),
    Spec::new("Tuples", ATTR_PROTECTED),
    Spec::new("Union", ATTR_FLAT_ONE_IDENTITY_PROTECTED_READ_PROTECTED),
    Spec::new("Until", ATTR_HOLD_ALL),
    Spec::new("UpperCaseQ", ATTR_PROTECTED),
    Spec::new("UppercaseLetters", ATTR_PROTECTED),
    Spec::new("Which", ATTR_HOLD_ALL),
    Spec::new("While", ATTR_HOLD_ALL),
    Spec::new("WithCleanup", ATTR_HOLD_ALL),
    Spec::new("Xor", ATTR_ORDERLESS_FLAT_ONE_IDENTITY),
];

const STDLIB_EMPTY_ATTRIBUTE_NAMES: &[&str] = &[
    "$AssertFunction",
    "Abort",
    "Accumulate",
    "AlignColumns",
    "AllTrue",
    "Alphabet",
    "AlphabeticOrder",
    "AlphabeticSort",
    "Alternatives",
    "AnyTrue",
    "Append",
    "Apply",
    "Array",
    "ArrayFilter",
    "ArrayQ",
    "ArrayReduce",
    "Association",
    "AssociationMap",
    "AssociationQ",
    "AssociationThread",
    "AtomQ",
    "Between",
    "Blank",
    "BlankNullSequence",
    "BlankSequence",
    "BlockMap",
    "BooleanQ",
    "Break",
    "ByteArrayQ",
    "ByteArrayToString",
    "Cases",
    "Catenate",
    "CharacterCounts",
    "CharacterRange",
    "Characters",
    "Coefficient",
    "CoefficientList",
    "Comap",
    "ComapApply",
    "Commonest",
    "Complement",
    "Compress",
    "Construct",
    "ContainsAll",
    "ContainsAny",
    "ContainsExactly",
    "ContainsNone",
    "ContainsOnly",
    "Continue",
    "Counts",
    "CountsBy",
    "Crc32",
    "CurryApplied",
    "DateDifference",
    "DateList",
    "DateObjectQ",
    "DatePlus",
    "DateString",
    "DayOfWeek",
    "DaysInMonth",
    "DecimalString",
    "Decrypt",
    "Delete",
    "DeleteCases",
    "DeleteDuplicates",
    "DeleteDuplicatesBy",
    "DeleteMissing",
    "Depth",
    "Diff",
    "DigitCharacter",
    "DigitQ",
    "DisjointQ",
    "Divide",
    "Dot",
    "Drop",
    "DuplicateFreeQ",
    "EditDistance",
    "Encrypt",
    "EndOfLine",
    "EndOfString",
    "Equal",
    "EqualTo",
    "Except",
    "Exit",
    "Expand",
    "Factor",
    "Failsafe",
    "Failure",
    "FailureQ",
    "FileBaseName",
    "FileExtension",
    "FileNameSplit",
    "FixedPoint",
    "FixedPointList",
    "Flatten",
    "Fold",
    "FoldList",
    "FoldPair",
    "FoldWhile",
    "FromBase64",
    "FromCsv",
    "FromDateString",
    "FromHexString",
    "FromIni",
    "FromJson",
    "FromToml",
    "FromXml",
    "FromYaml",
    "FullForm",
    "Greater",
    "GatherBy",
    "GreaterEqual",
    "GreaterEqualThan",
    "GreaterThan",
    "GroupBy",
    "HammingDistance",
    "Hash",
    "Head",
    "HtmlEscape",
    "HtmlUnescape",
    "Identity",
    "Implies",
    "Inner",
    "Insert",
    "IntegerQ",
    "IntersectingQ",
    "Join",
    "Key",
    "KeyComplement",
    "KeyDrop",
    "KeyExistsQ",
    "KeyFreeQ",
    "KeyIntersection",
    "KeyMap",
    "KeyMemberQ",
    "KeySelect",
    "KeySort",
    "KeySortBy",
    "KeyTake",
    "KeyUnion",
    "KeyValueMap",
    "KeyValuePattern",
    "Keys",
    "Length",
    "LengthWhile",
    "Less",
    "LessEqual",
    "LessEqualThan",
    "LessThan",
    "LetterCharacter",
    "LetterQ",
    "LeapYearQ",
    "LexicographicOrder",
    "ListQ",
    "LongestCommonSubsequence",
    "MachineNumberQ",
    "Map",
    "MapAll",
    "MapApply",
    "MapAt",
    "MapIndexed",
    "MapThread",
    "MatchQ",
    "MatrixQ",
    "Max",
    "MaxBy",
    "Md5",
    "Mean",
    "Median",
    "Merge",
    "Min",
    "MinBy",
    "Missing",
    "Most",
    "Nearest",
    "Nest",
    "NestList",
    "NestWhile",
    "NestWhileList",
    "NoneTrue",
    "Normal",
    "Not",
    "NumberQ",
    "NumericQ",
    "NumericalOrder",
    "Operate",
    "OperatorApplied",
    "Order",
    "OrderedQ",
    "Outer",
    "ParseArgs",
    "Pattern",
    "Pluralize",
    "PolynomialExpressionQ",
    "PolynomialQ",
    "PositionIndex",
    "Prepend",
    "PropagateAborts",
    "PunctuationCharacter",
    "Quantile",
    "Range",
    "RandomBytes",
    "RandomChoice",
    "RandomInteger",
    "RandomPassword",
    "RandomSample",
    "RandomString",
    "RandomUuid",
    "RationalExpressionQ",
    "RealValuedNumberQ",
    "RealValuedNumericQ",
    "RegularExpression",
    "RemoveDiacritics",
    "Replace",
    "ReplaceAll",
    "ReplaceList",
    "ReplaceRepeated",
    "Rest",
    "Return",
    "Reverse",
    "ReverseApplied",
    "ReverseSort",
    "ReverseSortBy",
    "Riffle",
    "Row",
    "RumTokens",
    "SameAs",
    "SameQ",
    "Scan",
    "SeedRandom",
    "Select",
    "SelectFirst",
    "SequenceAlignment",
    "SequenceFold",
    "SequenceFoldList",
    "Sha1",
    "Sha256",
    "Sha512",
    "ShellQuote",
    "ShellSplit",
    "Sort",
    "SortBy",
    "StartOfLine",
    "StartOfString",
    "StringApply",
    "StringCases",
    "StringContainsQ",
    "StringCount",
    "StringDedent",
    "StringDelete",
    "StringDrop",
    "StringEndsQ",
    "StringExpandTabs",
    "StringExpression",
    "StringForm",
    "StringFreeQ",
    "StringIndent",
    "StringJoin",
    "StringLines",
    "StringMatchQ",
    "StringPadLeft",
    "StringPadRight",
    "StringPart",
    "StringPosition",
    "StringQ",
    "StringRepeat",
    "StringReplace",
    "StringRiffle",
    "StringSplit",
    "StringStartsQ",
    "StringTake",
    "StringTakeDrop",
    "StringTemplate",
    "StringToByteArray",
    "StringTrim",
    "StringWrap",
    "SubsetMap",
    "SubsetQ",
    "Subtract",
    "Success",
    "Symbol",
    "SyntaxQ",
    "Take",
    "TakeDrop",
    "TakeLargestBy",
    "TakeList",
    "TakeSmallestBy",
    "TakeWhile",
    "Tally",
    "TemplateApply",
    "TerminatedEvaluation",
    "TextString",
    "Through",
    "Throw",
    "TimeObjectQ",
    "ToBase64",
    "ToCsv",
    "ToExpression",
    "ToHexString",
    "ToIni",
    "ToJson",
    "ToToml",
    "ToXml",
    "ToYaml",
    "TokenizeWith",
    "ToString",
    "ToUpperCase",
    "Together",
    "Total",
    "Transpose",
    "TrueQ",
    "Uncompress",
    "Unequal",
    "UnequalTo",
    "UnsameQ",
    "UrlDecode",
    "UrlEncode",
    "UrlParse",
    "UuidQ",
    "Values",
    "VectorQ",
    "WhitespaceCharacter",
    "WordBoundary",
    "WordCharacter",
    "XmlElement",
];

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
