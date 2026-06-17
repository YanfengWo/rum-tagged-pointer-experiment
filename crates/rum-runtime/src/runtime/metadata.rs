use super::*;

pub(super) const DEFAULT_REPLACE_REPEATED_LIMIT: usize = 256;
pub(super) const DEFAULT_RECURSION_LIMIT: usize = 1024;
pub(super) const DEFAULT_EVALUATION_PASS_LIMIT: usize = 256;
pub(super) const DEFAULT_GENERATED_ELEMENT_LIMIT: usize = 100_000;
pub(super) const DEFAULT_INTEGER_WORK_LIMIT: usize = 100_000;
pub(super) const DEFAULT_ITERATION_WORK_LIMIT: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldAttribute {
    None,
    HoldFirst,
    HoldRest,
    HoldAll,
}

impl HoldAttribute {
    pub(super) fn holds_argument(self, index: usize) -> bool {
        match self {
            Self::None => false,
            Self::HoldFirst => index == 0,
            Self::HoldRest => index > 0,
            Self::HoldAll => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attribute {
    HoldAllComplete,
    HoldAll,
    HoldFirst,
    HoldRest,
    SequenceHold,
    Listable,
    Orderless,
    Flat,
    OneIdentity,
    Protected,
    Temporary,
}

impl Attribute {
    pub(super) const fn bit(self) -> u16 {
        match self {
            Self::HoldAllComplete => 1 << 0,
            Self::HoldAll => 1 << 1,
            Self::HoldFirst => 1 << 2,
            Self::HoldRest => 1 << 3,
            Self::SequenceHold => 1 << 4,
            Self::Listable => 1 << 5,
            Self::Orderless => 1 << 6,
            Self::Flat => 1 << 7,
            Self::OneIdentity => 1 << 8,
            Self::Protected => 1 << 9,
            Self::Temporary => 1 << 10,
        }
    }

    pub(super) fn symbol_name(self) -> &'static str {
        match self {
            Self::HoldAllComplete => "HoldAllComplete",
            Self::HoldAll => "HoldAll",
            Self::HoldFirst => "HoldFirst",
            Self::HoldRest => "HoldRest",
            Self::SequenceHold => "SequenceHold",
            Self::Listable => "Listable",
            Self::Orderless => "Orderless",
            Self::Flat => "Flat",
            Self::OneIdentity => "OneIdentity",
            Self::Protected => "Protected",
            Self::Temporary => "Temporary",
        }
    }

    pub(super) fn from_expr(expr: &Expr) -> Option<Self> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) if symbol.context() == SYSTEM_CONTEXT => {
                Self::from_name(symbol.name())
            }
            Expr::Atom(Atom::String(name)) => Self::from_name(name),
            _ => None,
        }
    }

    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "HoldAllComplete" => Some(Self::HoldAllComplete),
            "HoldAll" => Some(Self::HoldAll),
            "HoldFirst" => Some(Self::HoldFirst),
            "HoldRest" => Some(Self::HoldRest),
            "SequenceHold" => Some(Self::SequenceHold),
            "Listable" => Some(Self::Listable),
            "Orderless" => Some(Self::Orderless),
            "Flat" => Some(Self::Flat),
            "OneIdentity" => Some(Self::OneIdentity),
            "Protected" => Some(Self::Protected),
            "Temporary" => Some(Self::Temporary),
            _ => None,
        }
    }

    pub(super) fn expr(self) -> Expr {
        system_expr(self.symbol_name())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttributeSet {
    bits: u16,
}

impl AttributeSet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_attribute(attribute: Attribute) -> Self {
        Self {
            bits: Self::normalized_bits(attribute.bit()),
        }
    }

    pub const fn from_hold(hold: HoldAttribute) -> Self {
        match hold {
            HoldAttribute::None => Self::empty(),
            HoldAttribute::HoldFirst => Self::from_attribute(Attribute::HoldFirst),
            HoldAttribute::HoldRest => Self::from_attribute(Attribute::HoldRest),
            HoldAttribute::HoldAll => Self::from_attribute(Attribute::HoldAll),
        }
    }

    pub const fn with(self, attribute: Attribute) -> Self {
        Self {
            bits: Self::normalized_bits(self.bits | attribute.bit()),
        }
    }

    pub(super) fn contains(self, attribute: Attribute) -> bool {
        self.bits & attribute.bit() != 0
    }

    pub(super) fn insert(&mut self, attribute: Attribute) {
        self.bits = Self::normalized_bits(self.bits | attribute.bit());
    }

    pub(super) fn remove(&mut self, attribute: Attribute) {
        self.bits = Self::normalized_bits(self.bits & !attribute.bit());
    }

    const fn normalized_bits(bits: u16) -> u16 {
        let hold_all = Attribute::HoldAll.bit();
        let hold_first = Attribute::HoldFirst.bit();
        let hold_rest = Attribute::HoldRest.bit();
        let has_hold_all = bits & hold_all != 0;
        let has_hold_first = bits & hold_first != 0;
        let has_hold_rest = bits & hold_rest != 0;

        if has_hold_all || (has_hold_first && has_hold_rest) {
            (bits | hold_all) & !hold_first & !hold_rest
        } else {
            bits
        }
    }

    pub(super) fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub(super) fn len(self) -> usize {
        self.attributes().len()
    }

    pub(super) fn attributes(self) -> Vec<Attribute> {
        ALL_ATTRIBUTES
            .iter()
            .copied()
            .filter(|attribute| self.contains(*attribute))
            .collect()
    }

    pub(super) fn hold_attribute(self) -> HoldAttribute {
        let holds_first = self.contains(Attribute::HoldFirst);
        let holds_rest = self.contains(Attribute::HoldRest);

        if self.contains(Attribute::HoldAllComplete)
            || self.contains(Attribute::HoldAll)
            || (holds_first && holds_rest)
        {
            HoldAttribute::HoldAll
        } else if holds_first {
            HoldAttribute::HoldFirst
        } else if holds_rest {
            HoldAttribute::HoldRest
        } else {
            HoldAttribute::None
        }
    }

    pub(super) fn sequence_hold(self) -> bool {
        self.contains(Attribute::HoldAllComplete) || self.contains(Attribute::SequenceHold)
    }

    pub(super) fn blocks_evaluate_override(self) -> bool {
        self.contains(Attribute::HoldAllComplete)
    }

    pub(super) fn expr(self) -> Expr {
        Expr::call(
            system_expr("List"),
            self.attributes()
                .into_iter()
                .map(Attribute::expr)
                .collect::<Vec<_>>(),
        )
    }
}

pub(super) const ALL_ATTRIBUTES: [Attribute; 11] = [
    Attribute::HoldAllComplete,
    Attribute::HoldAll,
    Attribute::HoldFirst,
    Attribute::HoldRest,
    Attribute::SequenceHold,
    Attribute::Listable,
    Attribute::Orderless,
    Attribute::Flat,
    Attribute::OneIdentity,
    Attribute::Protected,
    Attribute::Temporary,
];

pub(super) fn parse_attribute_set_expr(expr: &Expr) -> Option<AttributeSet> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut attributes = AttributeSet::empty();

            for item in args {
                // Preserve the user-written spec for ClearAttributes; storage normalizes later.
                attributes.bits |= Attribute::from_expr(item)?.bit();
            }

            Some(attributes)
        }
        _ => Some(AttributeSet::from_attribute(Attribute::from_expr(expr)?)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinDispatch {
    None,
    SpecialForm,
    Ordinary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinHandler {
    Identity,
    Plus,
    Times,
    Power,
    Subtract,
    Divide,
    Abs,
    Round,
    Factorial,
    Total,
    Denominator,
    Mod,
    Quotient,
    PowerMod,
    GCD,
    LCM,
    Expand,
    Coefficient,
    CoefficientList,
    Together,
    Factor,
    D,
    StringJoin,
    StringLength,
    ToUpperCase,
    StringForm,
    Row,
    StringRiffle,
    StringPadLeft,
    StringTemplate,
    TemplateApply,
    TextWords,
    ToCharacterCode,
    ToExpression,
    StringToByteArray,
    Hash,
    Compress,
    Uncompress,
    Encrypt,
    Decrypt,
    Nearest,
    Pluralize,
    SyntaxQ,
    TextString,
    FileTemplateApply,
    CharacterCounts,
    RemoveDiacritics,
    Alphabet,
    AlphabeticSort,
    EditDistance,
    HammingDistance,
    LongestCommonSubsequence,
    SequenceAlignment,
    Diff,
    Characters,
    CharacterRange,
    ToString,
    StringRepeat,
    LetterQ,
    DigitQ,
    StringContainsQ,
    StringStartsQ,
    StringEndsQ,
    StringPosition,
    StringSplit,
    StringTrim,
    StringReplace,
    StringCases,
    StringApply,
    StringDelete,
    StringTake,
    StringDrop,
    StringTakeDrop,
    StringPart,
    StringCount,
    StringMatchQ,
    StringFreeQ,
    FullForm,
    Head,
    Length,
    Depth,
    First,
    Last,
    Rest,
    Most,
    Take,
    Drop,
    TakeDrop,
    Insert,
    Delete,
    TakeList,
    Range,
    Flatten,
    Join,
    Reverse,
    Transpose,
    DeleteDuplicates,
    DeleteDuplicatesBy,
    Sort,
    SortBy,
    Union,
    Complement,
    Append,
    Prepend,
    Symbol,
    Construct,
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Equivalent,
    Implies,
    Not,
    TrueQ,
    BooleanQ,
    Boole,
    AtomQ,
    ListQ,
    StringQ,
    IntegerQ,
    NumberQ,
    NumericQ,
    EvenQ,
    OddQ,
    Positive,
    Negative,
    NonPositive,
    NonNegative,
    PossibleZeroQ,
    MachineNumberQ,
    RealValuedNumberQ,
    RealValuedNumericQ,
    PrimeQ,
    AlgebraicIntegerQ,
    PolynomialQ,
    PolynomialExpressionQ,
    RationalExpressionQ,
    DateObjectQ,
    TimeObjectQ,
    DuplicateFreeQ,
    IntersectingQ,
    DisjointQ,
    SubsetQ,
    Order,
    OrderedQ,
    LexicographicOrder,
    NumericalOrder,
    AlphabeticOrder,
    ContainsAll,
    ContainsAny,
    ContainsExactly,
    ContainsNone,
    ContainsOnly,
    Between,
    VectorQ,
    MatrixQ,
    ArrayQ,
    AllTrue,
    AnyTrue,
    NoneTrue,
    CoprimeQ,
    Association,
    AssociationQ,
    Lookup,
    Key,
    Keys,
    Values,
    KeyExistsQ,
    KeyTake,
    KeyDrop,
    KeyMap,
    KeyValueMap,
    KeySelect,
    KeyMemberQ,
    KeyFreeQ,
    KeySort,
    KeySortBy,
    AssociationMap,
    AssociationThread,
    KeyUnion,
    KeyIntersection,
    KeyComplement,
    Normal,
    AssociateTo,
    KeyDropFrom,
    KeyValuePattern,
    Merge,
    Catenate,
    Counts,
    CountsBy,
    GroupBy,
    PositionIndex,
    Missing,
    DeleteMissing,
    SameQ,
    UnsameQ,
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    LessThan,
    GreaterThan,
    LessEqualThan,
    GreaterEqualThan,
    EqualTo,
    UnequalTo,
    SameAs,
    Part,
    Extract,
    ReplacePart,
    ReplaceAt,
    Set,
    AddTo,
    SubtractFrom,
    TimesBy,
    DivideBy,
    ApplyTo,
    AppendTo,
    PrependTo,
    Increment,
    Decrement,
    PreIncrement,
    PreDecrement,
    SetDelayed,
    UpSet,
    UpSetDelayed,
    TagSet,
    TagSetDelayed,
    TagUnset,
    Rule,
    RuleDelayed,
    MatchQ,
    Cases,
    DeleteCases,
    Count,
    MemberQ,
    FreeQ,
    Level,
    Position,
    Replace,
    ReplaceList,
    Module,
    Unique,
    Block,
    ReplaceAll,
    ReplaceRepeated,
    FindFile,
    FileNameJoin,
    DirectoryName,
    ParentDirectory,
    FileNameTake,
    FileNames,
    FileExistsQ,
    FileByteCount,
    FileDate,
    Import,
    Get,
    Needs,
    Context,
    Contexts,
    Names,
    Begin,
    End,
    BeginPackage,
    EndPackage,
    DeclarePackage,
    Save,
    DumpSave,
    Once,
    Trace,
    TracePrint,
    Timing,
    AbsoluteTime,
    AbsoluteTiming,
    RepeatedTiming,
    Short,
    Echo,
    EchoFunction,
    EchoLabel,
    ByteCount,
    On,
    Off,
    Sow,
    Reap,
    TimeConstrained,
    MemoryConstrained,
    MemoryInUse,
    Monitor,
    EvaluationData,
    RumProfileReset,
    RumProfileSnapshot,
    TestCreate,
    TestReport,
    Stack,
    Message,
    MessageName,
    Messages,
    MessageDefinitions,
    MessageTags,
    Information,
    Definition,
    ValueQ,
    DefaultValues,
    OwnValues,
    UpValues,
    DownValues,
    SubValues,
    Attributes,
    SyntaxInformation,
    Options,
    OptionValue,
    ArgumentsOptions,
    CheckArguments,
    SetAttributes,
    ClearAttributes,
    SetOptions,
    Protect,
    Unprotect,
    Unset,
    Clear,
    ClearAll,
    Remove,
    NormalizeRules,
    Check,
    Quiet,
    CompoundExpression,
    If,
    Which,
    Switch,
    Do,
    While,
    Until,
    For,
    Table,
    Abort,
    Exit,
    CheckAbort,
    AbortProtect,
    WithCleanup,
    Failsafe,
    Assert,
    Enclose,
    Confirm,
    ConfirmBy,
    ConfirmMatch,
    ConfirmQuiet,
    ConfirmAssert,
    Return,
    Break,
    Continue,
    Throw,
    Catch,
    Failure,
    FailureQ,
    Success,
    TerminatedEvaluation,
    MissingQ,
    With,
    BlockMap,
    SubsetMap,
    Comap,
    ComapApply,
    Accumulate,
    MapIndexed,
    Map,
    MapAll,
    MapAt,
    Scan,
    MapThread,
    TakeWhile,
    LengthWhile,
    Inner,
    Fold,
    FoldList,
    FoldPair,
    FoldWhile,
    SequenceFold,
    SequenceFoldList,
    Nest,
    NestWhile,
    NestList,
    NestWhileList,
    FixedPoint,
    FixedPointList,
    Through,
    Operate,
    Array,
    ArrayFilter,
    ArrayReduce,
    Outer,
    Select,
    SelectFirst,
    Apply,
    MapApply,
    OperatorApplied,
    CurryApplied,
    ReverseApplied,
    Composition,
    RightComposition,
    HoldLiteral,
    Unevaluated,
    ReleaseHold,
    Evaluate,
    Sequence,
    Thread,
    Dialog,
    Input,
    InputString,
    Print,
    Pause,
    BlockRandom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub attributes: AttributeSet,
    pub dispatch: BuiltinDispatch,
    pub handler: Option<BuiltinHandler>,
}

impl BuiltinSpec {
    pub const fn new(name: &'static str, summary: &'static str) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::empty(),
            dispatch: BuiltinDispatch::None,
            handler: None,
        }
    }

    pub const fn with_hold(name: &'static str, summary: &'static str, hold: HoldAttribute) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::from_hold(hold),
            dispatch: BuiltinDispatch::None,
            handler: None,
        }
    }

    pub const fn with_attributes(
        name: &'static str,
        summary: &'static str,
        attributes: AttributeSet,
    ) -> Self {
        Self {
            name,
            summary,
            attributes,
            dispatch: BuiltinDispatch::None,
            handler: None,
        }
    }

    pub const fn special(
        name: &'static str,
        summary: &'static str,
        handler: BuiltinHandler,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::empty(),
            dispatch: BuiltinDispatch::SpecialForm,
            handler: Some(handler),
        }
    }

    pub const fn special_with_hold(
        name: &'static str,
        summary: &'static str,
        hold: HoldAttribute,
        handler: BuiltinHandler,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::from_hold(hold),
            dispatch: BuiltinDispatch::SpecialForm,
            handler: Some(handler),
        }
    }

    pub const fn special_with_attributes(
        name: &'static str,
        summary: &'static str,
        attributes: AttributeSet,
        handler: BuiltinHandler,
    ) -> Self {
        Self {
            name,
            summary,
            attributes,
            dispatch: BuiltinDispatch::SpecialForm,
            handler: Some(handler),
        }
    }

    pub const fn ordinary(
        name: &'static str,
        summary: &'static str,
        handler: BuiltinHandler,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::empty(),
            dispatch: BuiltinDispatch::Ordinary,
            handler: Some(handler),
        }
    }

    pub const fn ordinary_with_hold(
        name: &'static str,
        summary: &'static str,
        hold: HoldAttribute,
        handler: BuiltinHandler,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::from_hold(hold),
            dispatch: BuiltinDispatch::Ordinary,
            handler: Some(handler),
        }
    }

    pub const fn ordinary_with_attributes(
        name: &'static str,
        summary: &'static str,
        attributes: AttributeSet,
        handler: BuiltinHandler,
    ) -> Self {
        Self {
            name,
            summary,
            attributes,
            dispatch: BuiltinDispatch::Ordinary,
            handler: Some(handler),
        }
    }

    pub(super) fn with_fallback_dispatch(mut self, fallback: Self) -> Self {
        if self.dispatch == BuiltinDispatch::None {
            self.dispatch = fallback.dispatch;
        }

        if self.handler.is_none() {
            self.handler = fallback.handler;
        }

        self
    }
}
