use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Capability {
    Pure,
    FileRead,
    FileWrite,
}

impl Capability {
    const fn bit(self) -> u8 {
        match self {
            Self::Pure => 1 << 0,
            Self::FileRead => 1 << 1,
            Self::FileWrite => 1 << 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityPolicy {
    unrestricted: bool,
    allowed_bits: u8,
}

impl Default for CapabilityPolicy {
    fn default() -> Self {
        Self::unrestricted()
    }
}

impl CapabilityPolicy {
    pub const fn unrestricted() -> Self {
        Self {
            unrestricted: true,
            allowed_bits: 0,
        }
    }

    pub fn allow<I>(capabilities: I) -> Self
    where
        I: IntoIterator<Item = Capability>,
    {
        let mut policy = Self {
            unrestricted: false,
            allowed_bits: 0,
        };

        for capability in capabilities {
            policy.allowed_bits |= capability.bit();
        }

        policy
    }

    pub const fn builder() -> CapabilityPolicyBuilder {
        CapabilityPolicyBuilder {
            policy: Self {
                unrestricted: false,
                allowed_bits: 0,
            },
        }
    }

    #[inline]
    pub const fn is_unrestricted(self) -> bool {
        self.unrestricted
    }

    #[inline]
    pub const fn allows(self, capability: Capability) -> bool {
        self.unrestricted || self.allowed_bits & capability.bit() != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityPolicyBuilder {
    policy: CapabilityPolicy,
}

impl CapabilityPolicyBuilder {
    pub const fn new() -> Self {
        CapabilityPolicy::builder()
    }

    pub const fn allow(mut self, capability: Capability) -> Self {
        self.policy.allowed_bits |= capability.bit();
        self
    }

    pub const fn deny(mut self, capability: Capability) -> Self {
        self.policy.allowed_bits &= !capability.bit();
        self
    }

    pub const fn build(self) -> CapabilityPolicy {
        self.policy
    }
}

impl Default for CapabilityPolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinHandler {
    pub fn capability(self) -> Capability {
        match self {
            Self::FindFile
            | Self::FileNames
            | Self::FileExistsQ
            | Self::FileByteCount
            | Self::FileDate
            | Self::Import
            | Self::Get
            | Self::Needs => Capability::FileRead,
            Self::FileTemplateApply | Self::Save | Self::DumpSave => Capability::FileWrite,
            Self::Identity
            | Self::Plus
            | Self::Times
            | Self::Power
            | Self::Subtract
            | Self::Divide
            | Self::Abs
            | Self::Sign
            | Self::Floor
            | Self::Ceiling
            | Self::IntegerPart
            | Self::FractionalPart
            | Self::Minus
            | Self::Round
            | Self::Binomial
            | Self::Factorial
            | Self::Factorial2
            | Self::Fibonacci
            | Self::PartitionsP
            | Self::PartitionsQ
            | Self::BernoulliB
            | Self::Subfactorial
            | Self::StirlingS1
            | Self::StirlingS2
            | Self::CatalanNumber
            | Self::Multinomial
            | Self::Pochhammer
            | Self::Total
            | Self::Denominator
            | Self::Numerator
            | Self::IntegerLength
            | Self::IntegerDigits
            | Self::IntegerReverse
            | Self::FromDigits
            | Self::DigitCount
            | Self::Mod
            | Self::Quotient
            | Self::PowerMod
            | Self::GCD
            | Self::LCM
            | Self::Divisors
            | Self::EulerPhi
            | Self::MoebiusMu
            | Self::DivisorSigma
            | Self::Prime
            | Self::PrimePi
            | Self::PrimeNu
            | Self::PrimeOmega
            | Self::NextPrime
            | Self::Max
            | Self::Min
            | Self::MinMax
            | Self::Expand
            | Self::Coefficient
            | Self::CoefficientList
            | Self::Together
            | Self::Factor
            | Self::D
            | Self::StringJoin
            | Self::StringLength
            | Self::StringReverse
            | Self::ToLowerCase
            | Self::ToUpperCase
            | Self::LowerCaseQ
            | Self::UpperCaseQ
            | Self::Capitalize
            | Self::Decapitalize
            | Self::StringForm
            | Self::Row
            | Self::StringRiffle
            | Self::StringPadLeft
            | Self::StringTemplate
            | Self::TemplateApply
            | Self::TextWords
            | Self::ToCharacterCode
            | Self::ToExpression
            | Self::StringToByteArray
            | Self::Hash
            | Self::Compress
            | Self::Uncompress
            | Self::Encrypt
            | Self::Decrypt
            | Self::Nearest
            | Self::Pluralize
            | Self::SyntaxQ
            | Self::TextString
            | Self::CharacterCounts
            | Self::RemoveDiacritics
            | Self::Alphabet
            | Self::AlphabeticSort
            | Self::EditDistance
            | Self::HammingDistance
            | Self::LongestCommonSubsequence
            | Self::SequenceAlignment
            | Self::Diff
            | Self::Characters
            | Self::CharacterRange
            | Self::ToString
            | Self::StringRepeat
            | Self::LetterQ
            | Self::DigitQ
            | Self::StringContainsQ
            | Self::StringStartsQ
            | Self::StringEndsQ
            | Self::StringPosition
            | Self::StringSplit
            | Self::StringTrim
            | Self::StringReplace
            | Self::StringCases
            | Self::StringApply
            | Self::StringDelete
            | Self::StringTake
            | Self::StringDrop
            | Self::StringTakeDrop
            | Self::StringPart
            | Self::StringCount
            | Self::StringMatchQ
            | Self::StringFreeQ
            | Self::FullForm
            | Self::Head
            | Self::Length
            | Self::Depth
            | Self::LeafCount
            | Self::First
            | Self::Last
            | Self::Rest
            | Self::Most
            | Self::Take
            | Self::Drop
            | Self::TakeDrop
            | Self::Insert
            | Self::Delete
            | Self::TakeList
            | Self::Range
            | Self::Differences
            | Self::ConstantArray
            | Self::PadLeft
            | Self::PadRight
            | Self::Flatten
            | Self::Join
            | Self::Reverse
            | Self::Transpose
            | Self::DeleteDuplicates
            | Self::DeleteDuplicatesBy
            | Self::Sort
            | Self::SortBy
            | Self::Union
            | Self::Intersection
            | Self::Complement
            | Self::Subsets
            | Self::Tuples
            | Self::Partition
            | Self::Append
            | Self::Prepend
            | Self::Symbol
            | Self::Construct
            | Self::And
            | Self::Or
            | Self::Nand
            | Self::Nor
            | Self::Xor
            | Self::Equivalent
            | Self::Implies
            | Self::Not
            | Self::TrueQ
            | Self::BooleanQ
            | Self::Boole
            | Self::AtomQ
            | Self::ListQ
            | Self::StringQ
            | Self::IntegerQ
            | Self::NumberQ
            | Self::NumericQ
            | Self::EvenQ
            | Self::OddQ
            | Self::Positive
            | Self::Negative
            | Self::NonPositive
            | Self::NonNegative
            | Self::PossibleZeroQ
            | Self::MachineNumberQ
            | Self::RealValuedNumberQ
            | Self::RealValuedNumericQ
            | Self::PrimeQ
            | Self::AlgebraicIntegerQ
            | Self::PolynomialQ
            | Self::PolynomialExpressionQ
            | Self::RationalExpressionQ
            | Self::DateObjectQ
            | Self::TimeObjectQ
            | Self::DuplicateFreeQ
            | Self::IntersectingQ
            | Self::DisjointQ
            | Self::SubsetQ
            | Self::Order
            | Self::OrderedQ
            | Self::LexicographicOrder
            | Self::NumericalOrder
            | Self::AlphabeticOrder
            | Self::ContainsAll
            | Self::ContainsAny
            | Self::ContainsExactly
            | Self::ContainsNone
            | Self::ContainsOnly
            | Self::Between
            | Self::VectorQ
            | Self::MatrixQ
            | Self::ArrayQ
            | Self::AllTrue
            | Self::AnyTrue
            | Self::NoneTrue
            | Self::CoprimeQ
            | Self::Association
            | Self::AssociationQ
            | Self::Lookup
            | Self::Key
            | Self::Keys
            | Self::Values
            | Self::KeyExistsQ
            | Self::KeyTake
            | Self::KeyDrop
            | Self::KeyMap
            | Self::KeyValueMap
            | Self::KeySelect
            | Self::KeyMemberQ
            | Self::KeyFreeQ
            | Self::KeySort
            | Self::KeySortBy
            | Self::AssociationMap
            | Self::AssociationThread
            | Self::KeyUnion
            | Self::KeyIntersection
            | Self::KeyComplement
            | Self::Normal
            | Self::AssociateTo
            | Self::KeyDropFrom
            | Self::KeyValuePattern
            | Self::Merge
            | Self::Catenate
            | Self::Counts
            | Self::CountsBy
            | Self::GroupBy
            | Self::PositionIndex
            | Self::Missing
            | Self::DeleteMissing
            | Self::SameQ
            | Self::UnsameQ
            | Self::Equal
            | Self::Unequal
            | Self::Less
            | Self::Greater
            | Self::LessEqual
            | Self::GreaterEqual
            | Self::LessThan
            | Self::GreaterThan
            | Self::LessEqualThan
            | Self::GreaterEqualThan
            | Self::EqualTo
            | Self::UnequalTo
            | Self::SameAs
            | Self::Part
            | Self::Extract
            | Self::ReplacePart
            | Self::ReplaceAt
            | Self::Set
            | Self::AddTo
            | Self::SubtractFrom
            | Self::TimesBy
            | Self::DivideBy
            | Self::ApplyTo
            | Self::AppendTo
            | Self::PrependTo
            | Self::Increment
            | Self::Decrement
            | Self::PreIncrement
            | Self::PreDecrement
            | Self::SetDelayed
            | Self::UpSet
            | Self::UpSetDelayed
            | Self::TagSet
            | Self::TagSetDelayed
            | Self::TagUnset
            | Self::Rule
            | Self::RuleDelayed
            | Self::MatchQ
            | Self::Cases
            | Self::DeleteCases
            | Self::Count
            | Self::MemberQ
            | Self::FreeQ
            | Self::Level
            | Self::Position
            | Self::Replace
            | Self::ReplaceList
            | Self::Module
            | Self::Unique
            | Self::Block
            | Self::ReplaceAll
            | Self::ReplaceRepeated
            | Self::FileNameJoin
            | Self::DirectoryName
            | Self::ParentDirectory
            | Self::FileNameTake
            | Self::Context
            | Self::Contexts
            | Self::Names
            | Self::SymbolName
            | Self::NameQ
            | Self::Begin
            | Self::End
            | Self::BeginPackage
            | Self::EndPackage
            | Self::DeclarePackage
            | Self::Once
            | Self::Trace
            | Self::TracePrint
            | Self::Timing
            | Self::AbsoluteTime
            | Self::AbsoluteTiming
            | Self::RepeatedTiming
            | Self::Short
            | Self::Echo
            | Self::EchoFunction
            | Self::EchoLabel
            | Self::ByteCount
            | Self::On
            | Self::Off
            | Self::Sow
            | Self::Reap
            | Self::TimeConstrained
            | Self::MemoryConstrained
            | Self::MemoryInUse
            | Self::MaxMemoryUsed
            | Self::Monitor
            | Self::EvaluationData
            | Self::RumProfileReset
            | Self::RumProfileSnapshot
            | Self::TestCreate
            | Self::TestReport
            | Self::Stack
            | Self::Message
            | Self::MessageName
            | Self::Messages
            | Self::MessageDefinitions
            | Self::MessageTags
            | Self::Information
            | Self::Definition
            | Self::ValueQ
            | Self::DefaultValues
            | Self::OwnValues
            | Self::UpValues
            | Self::DownValues
            | Self::SubValues
            | Self::Attributes
            | Self::SyntaxInformation
            | Self::Options
            | Self::OptionValue
            | Self::ArgumentsOptions
            | Self::CheckArguments
            | Self::SetAttributes
            | Self::ClearAttributes
            | Self::SetOptions
            | Self::Protect
            | Self::Unprotect
            | Self::Unset
            | Self::Clear
            | Self::ClearAll
            | Self::Remove
            | Self::NormalizeRules
            | Self::Check
            | Self::Quiet
            | Self::CompoundExpression
            | Self::If
            | Self::Which
            | Self::Switch
            | Self::Do
            | Self::Product
            | Self::Sum
            | Self::While
            | Self::Until
            | Self::For
            | Self::Table
            | Self::Abort
            | Self::Exit
            | Self::CheckAbort
            | Self::AbortProtect
            | Self::WithCleanup
            | Self::Failsafe
            | Self::Assert
            | Self::Enclose
            | Self::Confirm
            | Self::ConfirmBy
            | Self::ConfirmMatch
            | Self::ConfirmQuiet
            | Self::ConfirmAssert
            | Self::Return
            | Self::Break
            | Self::Continue
            | Self::Throw
            | Self::Catch
            | Self::Failure
            | Self::FailureQ
            | Self::Success
            | Self::TerminatedEvaluation
            | Self::MissingQ
            | Self::With
            | Self::BlockMap
            | Self::SubsetMap
            | Self::Comap
            | Self::ComapApply
            | Self::Accumulate
            | Self::MapIndexed
            | Self::Map
            | Self::MapAll
            | Self::MapAt
            | Self::Scan
            | Self::MapThread
            | Self::TakeWhile
            | Self::LengthWhile
            | Self::Inner
            | Self::Dot
            | Self::Fold
            | Self::FoldList
            | Self::FoldPair
            | Self::FoldWhile
            | Self::SequenceFold
            | Self::SequenceFoldList
            | Self::Nest
            | Self::NestWhile
            | Self::NestList
            | Self::NestWhileList
            | Self::FixedPoint
            | Self::FixedPointList
            | Self::Through
            | Self::Operate
            | Self::Array
            | Self::ArrayFilter
            | Self::ArrayReduce
            | Self::Outer
            | Self::Select
            | Self::SelectFirst
            | Self::Apply
            | Self::MapApply
            | Self::OperatorApplied
            | Self::CurryApplied
            | Self::ReverseApplied
            | Self::Composition
            | Self::RightComposition
            | Self::HoldLiteral
            | Self::Unevaluated
            | Self::ReleaseHold
            | Self::Evaluate
            | Self::Sequence
            | Self::Thread
            | Self::Dialog
            | Self::Input
            | Self::InputString
            | Self::Print
            | Self::Pause
            | Self::BlockRandom => Capability::Pure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_policy_defaults_to_unrestricted() {
        let policy = CapabilityPolicy::default();

        assert!(policy.is_unrestricted());
        assert!(policy.allows(Capability::Pure));
        assert!(policy.allows(Capability::FileRead));
        assert!(policy.allows(Capability::FileWrite));
    }

    #[test]
    fn capability_policy_allowlist_is_default_deny() {
        let policy = CapabilityPolicy::allow([Capability::Pure, Capability::FileRead]);

        assert!(!policy.is_unrestricted());
        assert!(policy.allows(Capability::Pure));
        assert!(policy.allows(Capability::FileRead));
        assert!(!policy.allows(Capability::FileWrite));
    }

    #[test]
    fn handler_capabilities_classify_filesystem_builtins() {
        assert_eq!(BuiltinHandler::Get.capability(), Capability::FileRead);
        assert_eq!(BuiltinHandler::Needs.capability(), Capability::FileRead);
        assert_eq!(BuiltinHandler::FindFile.capability(), Capability::FileRead);
        assert_eq!(BuiltinHandler::Import.capability(), Capability::FileRead);
        assert_eq!(BuiltinHandler::Save.capability(), Capability::FileWrite);
        assert_eq!(BuiltinHandler::DumpSave.capability(), Capability::FileWrite);
        assert_eq!(
            BuiltinHandler::FileTemplateApply.capability(),
            Capability::FileWrite
        );
        assert_eq!(BuiltinHandler::FileNameJoin.capability(), Capability::Pure);
        assert_eq!(BuiltinHandler::DirectoryName.capability(), Capability::Pure);
        assert_eq!(
            BuiltinHandler::ParentDirectory.capability(),
            Capability::Pure
        );
        assert_eq!(BuiltinHandler::FileNameTake.capability(), Capability::Pure);
    }
}
