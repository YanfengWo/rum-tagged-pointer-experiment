use super::*;

const CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Identity",
        "Returns its argument unchanged.",
        BuiltinHandler::Identity,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Plus",
        "Adds exact numbers and canonically combines additive expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        BuiltinHandler::Plus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Times",
        "Multiplies exact numbers and canonically combines multiplicative expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        BuiltinHandler::Times,
    ),
    BuiltinSpec::ordinary(
        "Power",
        "Raises an exact number to an integer power when possible.",
        BuiltinHandler::Power,
    ),
    BuiltinSpec::ordinary(
        "Subtract",
        "Subtracts one exact number from another.",
        BuiltinHandler::Subtract,
    ),
    BuiltinSpec::ordinary(
        "Divide",
        "Divides exact numbers, producing exact rationals when possible.",
        BuiltinHandler::Divide,
    ),
    BuiltinSpec::ordinary(
        "Mod",
        "Returns the modulus of one integer with respect to another.",
        BuiltinHandler::Mod,
    ),
    BuiltinSpec::ordinary(
        "Quotient",
        "Returns the floor-division quotient of two integers.",
        BuiltinHandler::Quotient,
    ),
    BuiltinSpec::ordinary(
        "PowerMod",
        "Computes modular exponentiation for exact integers.",
        BuiltinHandler::PowerMod,
    ),
    BuiltinSpec::ordinary(
        "GCD",
        "Computes the greatest common divisor of exact integers.",
        BuiltinHandler::GCD,
    ),
    BuiltinSpec::ordinary(
        "LCM",
        "Computes the least common multiple of exact integers.",
        BuiltinHandler::LCM,
    ),
    BuiltinSpec::ordinary(
        "Expand",
        "Expands rational-coefficient polynomial expressions over pattern-free indeterminates.",
        BuiltinHandler::Expand,
    ),
    BuiltinSpec::ordinary(
        "Coefficient",
        "Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.",
        BuiltinHandler::Coefficient,
    ),
    BuiltinSpec::ordinary(
        "CoefficientList",
        "Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.",
        BuiltinHandler::CoefficientList,
    ),
    BuiltinSpec::ordinary(
        "Together",
        "Combines rational-function terms over a common polynomial denominator.",
        BuiltinHandler::Together,
    ),
    BuiltinSpec::ordinary(
        "Factor",
        "Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.",
        BuiltinHandler::Factor,
    ),
    BuiltinSpec::ordinary_with_hold(
        "D",
        "Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.",
        HoldAttribute::HoldRest,
        BuiltinHandler::D,
    ),
    BuiltinSpec::ordinary(
        "StringJoin",
        "Concatenates strings or a list of strings into a single string.",
        BuiltinHandler::StringJoin,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StringLength",
        "Returns the Unicode code-point length of a string.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::StringLength,
    ),
    BuiltinSpec::ordinary(
        "Characters",
        "Splits a string into a list of single-code-point strings.",
        BuiltinHandler::Characters,
    ),
    BuiltinSpec::ordinary(
        "CharacterRange",
        "Builds an inclusive range of single-character strings between two endpoints.",
        BuiltinHandler::CharacterRange,
    ),
    BuiltinSpec::ordinary(
        "ToString",
        "Renders an evaluated expression into the current canonical kernel text form.",
        BuiltinHandler::ToString,
    ),
    BuiltinSpec::ordinary(
        "StringRepeat",
        "Repeats a string a non-negative number of times.",
        BuiltinHandler::StringRepeat,
    ),
    BuiltinSpec::ordinary(
        "StringContainsQ",
        "Tests whether a string or list of strings contains an exact substring in the current first-pass string subset.",
        BuiltinHandler::StringContainsQ,
    ),
    BuiltinSpec::ordinary(
        "StringStartsQ",
        "Tests whether a string or list of strings starts with an exact substring in the current first-pass string subset.",
        BuiltinHandler::StringStartsQ,
    ),
    BuiltinSpec::ordinary(
        "StringEndsQ",
        "Tests whether a string or list of strings ends with an exact substring in the current first-pass string subset.",
        BuiltinHandler::StringEndsQ,
    ),
    BuiltinSpec::ordinary(
        "StringPosition",
        "Returns 1-based inclusive positions of exact substring matches, including overlaps, in the current first-pass string subset.",
        BuiltinHandler::StringPosition,
    ),
    BuiltinSpec::ordinary(
        "StringSplit",
        "Splits strings using whitespace or exact delimiters in the current first-pass string subset.",
        BuiltinHandler::StringSplit,
    ),
    BuiltinSpec::ordinary(
        "StringTrim",
        "Trims whitespace or exact boundary substrings in the current first-pass string subset.",
        BuiltinHandler::StringTrim,
    ),
    BuiltinSpec::ordinary(
        "StringReplace",
        "Replaces exact string needles using immediate string rules in the current first-pass string subset.",
        BuiltinHandler::StringReplace,
    ),
    BuiltinSpec::ordinary(
        "StringCases",
        "Extracts non-overlapping exact string matches or immediate exact-string rule outputs in the current first-pass string subset.",
        BuiltinHandler::StringCases,
    ),
    BuiltinSpec::ordinary(
        "StringTake",
        "Extracts a substring using integer counts or inclusive 1-based ranges.",
        BuiltinHandler::StringTake,
    ),
    BuiltinSpec::ordinary(
        "StringDrop",
        "Drops a substring using integer counts or inclusive 1-based ranges.",
        BuiltinHandler::StringDrop,
    ),
    BuiltinSpec::ordinary(
        "StringPart",
        "Extracts individual characters by 1-based positions.",
        BuiltinHandler::StringPart,
    ),
    BuiltinSpec::ordinary_with_hold(
        "FullForm",
        "Renders the held argument as a canonical structural-form string in the current core expression subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::FullForm,
    ),
    BuiltinSpec::ordinary(
        "Head",
        "Returns the structural head of an expression, optionally wrapped in a second head.",
        BuiltinHandler::Head,
    ),
    BuiltinSpec::ordinary(
        "Length",
        "Returns the number of structural elements in an expression.",
        BuiltinHandler::Length,
    ),
    BuiltinSpec::ordinary(
        "Depth",
        "Returns the structural depth of an expression, optionally including heads.",
        BuiltinHandler::Depth,
    ),
    BuiltinSpec::ordinary_with_hold(
        "First",
        "Returns the first structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        BuiltinHandler::First,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Last",
        "Returns the last structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        BuiltinHandler::Last,
    ),
    BuiltinSpec::ordinary(
        "Rest",
        "Returns an expression with its first structural element removed.",
        BuiltinHandler::Rest,
    ),
    BuiltinSpec::ordinary(
        "Most",
        "Returns an expression with its last structural element removed.",
        BuiltinHandler::Most,
    ),
    BuiltinSpec::ordinary(
        "Take",
        "Takes structural elements using the current standard sequence-spec subset.",
        BuiltinHandler::Take,
    ),
    BuiltinSpec::ordinary(
        "Drop",
        "Drops structural elements using the current standard sequence-spec subset.",
        BuiltinHandler::Drop,
    ),
    BuiltinSpec::ordinary(
        "TakeDrop",
        "Returns both the taken and dropped structural elements using the current standard sequence-spec subset.",
        BuiltinHandler::TakeDrop,
    ),
    BuiltinSpec::ordinary(
        "Insert",
        "Inserts an element at a top-level structural position in the current first-pass subset.",
        BuiltinHandler::Insert,
    ),
    BuiltinSpec::ordinary(
        "Delete",
        "Deletes one or more top-level structural positions in the current first-pass subset.",
        BuiltinHandler::Delete,
    ),
    BuiltinSpec::ordinary(
        "TakeList",
        "Splits a normal expression into sequential chunks by non-negative counts.",
        BuiltinHandler::TakeList,
    ),
    BuiltinSpec::ordinary(
        "Append",
        "Appends an element to the end of a normal expression, with current operator-form support.",
        BuiltinHandler::Append,
    ),
    BuiltinSpec::ordinary(
        "Prepend",
        "Prepends an element to the front of a normal expression, with current operator-form support.",
        BuiltinHandler::Prepend,
    ),
    BuiltinSpec::ordinary(
        "Symbol",
        "Resolves a symbol from a string name using the current context path.",
        BuiltinHandler::Symbol,
    ),
    BuiltinSpec::ordinary(
        "Construct",
        "Constructs an expression from a head and zero or more arguments.",
        BuiltinHandler::Construct,
    ),
    BuiltinSpec::ordinary_with_hold(
        "And",
        "Evaluates a conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        BuiltinHandler::And,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Or",
        "Evaluates a disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Or,
    ),
    BuiltinSpec::ordinary(
        "Not",
        "Negates a boolean value in the current exact boolean subset.",
        BuiltinHandler::Not,
    ),
    BuiltinSpec::ordinary(
        "TrueQ",
        "Tests whether an expression evaluates exactly to True.",
        BuiltinHandler::TrueQ,
    ),
    BuiltinSpec::ordinary(
        "BooleanQ",
        "Tests whether an expression evaluates exactly to True or False.",
        BuiltinHandler::BooleanQ,
    ),
    BuiltinSpec::ordinary(
        "AtomQ",
        "Tests whether an expression is atomic rather than a normal expression.",
        BuiltinHandler::AtomQ,
    ),
    BuiltinSpec::ordinary(
        "ListQ",
        "Tests whether an expression has head List.",
        BuiltinHandler::ListQ,
    ),
    BuiltinSpec::ordinary(
        "StringQ",
        "Tests whether an expression is a string atom.",
        BuiltinHandler::StringQ,
    ),
    BuiltinSpec::ordinary(
        "IntegerQ",
        "Tests whether an expression is an exact integer atom.",
        BuiltinHandler::IntegerQ,
    ),
    BuiltinSpec::ordinary(
        "NumberQ",
        "Tests whether an expression is an exact numeric atom in the current number subset.",
        BuiltinHandler::NumberQ,
    ),
    BuiltinSpec::ordinary(
        "NumericQ",
        "Tests whether an expression is currently numeric in the exact-number subset.",
        BuiltinHandler::NumericQ,
    ),
    BuiltinSpec::ordinary(
        "EvenQ",
        "Tests whether an expression is currently an even exact integer.",
        BuiltinHandler::EvenQ,
    ),
    BuiltinSpec::ordinary(
        "OddQ",
        "Tests whether an expression is currently an odd exact integer.",
        BuiltinHandler::OddQ,
    ),
    BuiltinSpec::ordinary(
        "Positive",
        "Tests positivity in the current exact integer/rational subset.",
        BuiltinHandler::Positive,
    ),
    BuiltinSpec::ordinary(
        "Negative",
        "Tests negativity in the current exact integer/rational subset.",
        BuiltinHandler::Negative,
    ),
    BuiltinSpec::ordinary(
        "NonPositive",
        "Tests non-positivity in the current exact integer/rational subset.",
        BuiltinHandler::NonPositive,
    ),
    BuiltinSpec::ordinary(
        "NonNegative",
        "Tests non-negativity in the current exact integer/rational subset.",
        BuiltinHandler::NonNegative,
    ),
    BuiltinSpec::ordinary(
        "PossibleZeroQ",
        "Tests exact zero in the current exact integer/rational subset.",
        BuiltinHandler::PossibleZeroQ,
    ),
    BuiltinSpec::ordinary(
        "MachineNumberQ",
        "Tests whether an expression is a machine number; currently always False because only exact numbers exist.",
        BuiltinHandler::MachineNumberQ,
    ),
    BuiltinSpec::ordinary(
        "RealValuedNumberQ",
        "Tests whether an expression is a real-valued number in the current exact-number subset.",
        BuiltinHandler::RealValuedNumberQ,
    ),
    BuiltinSpec::ordinary(
        "RealValuedNumericQ",
        "Tests whether an expression is real-valued and numeric in the current exact-number subset.",
        BuiltinHandler::RealValuedNumericQ,
    ),
    BuiltinSpec::ordinary(
        "PrimeQ",
        "Tests primality in the current exact-integer subset.",
        BuiltinHandler::PrimeQ,
    ),
    BuiltinSpec::ordinary(
        "AlgebraicIntegerQ",
        "Tests whether an expression is an exact integer in the current algebraic-integer subset.",
        BuiltinHandler::AlgebraicIntegerQ,
    ),
    BuiltinSpec::ordinary(
        "DuplicateFreeQ",
        "Tests whether a List has no repeated elements in the current exact structural subset.",
        BuiltinHandler::DuplicateFreeQ,
    ),
    BuiltinSpec::ordinary(
        "IntersectingQ",
        "Tests whether two Lists share any element in the current exact structural subset.",
        BuiltinHandler::IntersectingQ,
    ),
    BuiltinSpec::ordinary(
        "DisjointQ",
        "Tests whether two Lists share no element in the current exact structural subset.",
        BuiltinHandler::DisjointQ,
    ),
    BuiltinSpec::ordinary(
        "SubsetQ",
        "Tests subset inclusion for Lists in the current exact structural subset.",
        BuiltinHandler::SubsetQ,
    ),
    BuiltinSpec::ordinary(
        "Order",
        "Orders two expressions using the current canonical textual ordering subset.",
        BuiltinHandler::Order,
    ),
    BuiltinSpec::ordinary(
        "OrderedQ",
        "Tests whether a List is nondecreasing under the current canonical textual ordering subset.",
        BuiltinHandler::OrderedQ,
    ),
    BuiltinSpec::ordinary(
        "ContainsAll",
        "Tests whether a List contains every requested element in the current exact structural subset.",
        BuiltinHandler::ContainsAll,
    ),
    BuiltinSpec::ordinary(
        "ContainsAny",
        "Tests whether a List contains any requested element in the current exact structural subset.",
        BuiltinHandler::ContainsAny,
    ),
    BuiltinSpec::ordinary(
        "ContainsExactly",
        "Tests List multiset equality in the current exact structural subset.",
        BuiltinHandler::ContainsExactly,
    ),
    BuiltinSpec::ordinary(
        "ContainsNone",
        "Tests whether a List contains none of the requested elements in the current exact structural subset.",
        BuiltinHandler::ContainsNone,
    ),
    BuiltinSpec::ordinary(
        "ContainsOnly",
        "Tests whether every element of a List belongs to the allowed-element List in the current exact structural subset.",
        BuiltinHandler::ContainsOnly,
    ),
    BuiltinSpec::ordinary(
        "Between",
        "Tests interval inclusion in the current exact numeric subset.",
        BuiltinHandler::Between,
    ),
    BuiltinSpec::ordinary(
        "VectorQ",
        "Tests whether an expression is a one-dimensional List array in the current exact structural subset.",
        BuiltinHandler::VectorQ,
    ),
    BuiltinSpec::ordinary(
        "MatrixQ",
        "Tests whether an expression is a rectangular two-dimensional List array in the current exact structural subset.",
        BuiltinHandler::MatrixQ,
    ),
    BuiltinSpec::ordinary(
        "ArrayQ",
        "Tests whether an expression is a rectangular List array in the current exact structural subset.",
        BuiltinHandler::ArrayQ,
    ),
    BuiltinSpec::ordinary(
        "AllTrue",
        "Tests whether all selected structural parts satisfy a criterion in the current boolean subset.",
        BuiltinHandler::AllTrue,
    ),
    BuiltinSpec::ordinary(
        "AnyTrue",
        "Tests whether any selected structural part satisfies a criterion in the current boolean subset.",
        BuiltinHandler::AnyTrue,
    ),
    BuiltinSpec::ordinary(
        "NoneTrue",
        "Tests whether no selected structural parts satisfy a criterion in the current boolean subset.",
        BuiltinHandler::NoneTrue,
    ),
    BuiltinSpec::ordinary(
        "CoprimeQ",
        "Tests whether exact integers are pairwise relatively prime in the current integer subset.",
        BuiltinHandler::CoprimeQ,
    ),
    BuiltinSpec::ordinary(
        "Association",
        "Builds an association from Rule/RuleDelayed entries in the current first-pass subset.",
        BuiltinHandler::Association,
    ),
    BuiltinSpec::ordinary(
        "AssociationQ",
        "Tests whether an expression is an Association with rule entries.",
        BuiltinHandler::AssociationQ,
    ),
    BuiltinSpec::ordinary(
        "Lookup",
        "Looks up a key in an association in the current first-pass subset.",
        BuiltinHandler::Lookup,
    ),
    BuiltinSpec::ordinary(
        "Key",
        "Represents a keyed selector in the current first-pass association subset.",
        BuiltinHandler::Key,
    ),
    BuiltinSpec::ordinary(
        "Keys",
        "Returns association keys in structural order.",
        BuiltinHandler::Keys,
    ),
    BuiltinSpec::ordinary(
        "Values",
        "Returns association values in structural order.",
        BuiltinHandler::Values,
    ),
    BuiltinSpec::ordinary(
        "KeyExistsQ",
        "Tests whether an association contains a given key in the current first-pass subset.",
        BuiltinHandler::KeyExistsQ,
    ),
    BuiltinSpec::ordinary(
        "KeyTake",
        "Keeps selected keys from an association in the current first-pass subset.",
        BuiltinHandler::KeyTake,
    ),
    BuiltinSpec::ordinary(
        "KeyDrop",
        "Drops selected keys from an association in the current first-pass subset.",
        BuiltinHandler::KeyDrop,
    ),
    BuiltinSpec::ordinary(
        "KeyMap",
        "Maps a function over association keys in the current first-pass subset.",
        BuiltinHandler::KeyMap,
    ),
    BuiltinSpec::ordinary(
        "KeyValueMap",
        "Maps a binary function over association key-value pairs in the current first-pass subset.",
        BuiltinHandler::KeyValueMap,
    ),
    BuiltinSpec::ordinary(
        "KeySelect",
        "Selects association entries whose keys satisfy a criterion in the current first-pass subset.",
        BuiltinHandler::KeySelect,
    ),
    BuiltinSpec::ordinary(
        "KeyMemberQ",
        "Tests whether an association contains a key in the current first-pass subset.",
        BuiltinHandler::KeyMemberQ,
    ),
    BuiltinSpec::ordinary(
        "KeyFreeQ",
        "Tests whether an association does not contain a key in the current first-pass subset.",
        BuiltinHandler::KeyFreeQ,
    ),
    BuiltinSpec::ordinary(
        "KeySort",
        "Sorts association entries by key in the current canonical ordering subset.",
        BuiltinHandler::KeySort,
    ),
    BuiltinSpec::ordinary(
        "KeySortBy",
        "Sorts association entries by transformed keys in the current first-pass subset.",
        BuiltinHandler::KeySortBy,
    ),
    BuiltinSpec::ordinary(
        "AssociationMap",
        "Builds an association by mapping a function over keys in the current first-pass subset.",
        BuiltinHandler::AssociationMap,
    ),
    BuiltinSpec::ordinary(
        "AssociationThread",
        "Threads key and value lists into an association in the current first-pass subset.",
        BuiltinHandler::AssociationThread,
    ),
    BuiltinSpec::ordinary(
        "KeyUnion",
        "Builds a key-union association in the current first-pass subset.",
        BuiltinHandler::KeyUnion,
    ),
    BuiltinSpec::ordinary(
        "KeyIntersection",
        "Builds a key-intersection association in the current first-pass subset.",
        BuiltinHandler::KeyIntersection,
    ),
    BuiltinSpec::ordinary(
        "KeyComplement",
        "Builds a key-complement association in the current first-pass subset.",
        BuiltinHandler::KeyComplement,
    ),
    BuiltinSpec::ordinary(
        "Normal",
        "Converts an association into a list of rules in the current first-pass subset.",
        BuiltinHandler::Normal,
    ),
    BuiltinSpec::ordinary(
        "AssociateTo",
        "Adds or replaces association entries in the current first-pass subset.",
        BuiltinHandler::AssociateTo,
    ),
    BuiltinSpec::ordinary(
        "KeyDropFrom",
        "Drops keys from an association in the current first-pass subset.",
        BuiltinHandler::KeyDropFrom,
    ),
    BuiltinSpec::ordinary(
        "KeyValuePattern",
        "Represents an association key/value pattern specification in the current first-pass subset.",
        BuiltinHandler::KeyValuePattern,
    ),
    BuiltinSpec::ordinary(
        "Merge",
        "Merges associations by key using a combiner in the current first-pass subset.",
        BuiltinHandler::Merge,
    ),
    BuiltinSpec::ordinary(
        "Catenate",
        "Concatenates a list of associations into a single association in the current first-pass subset.",
        BuiltinHandler::Catenate,
    ),
    BuiltinSpec::ordinary(
        "Counts",
        "Counts repeated elements and returns an association in the current first-pass subset.",
        BuiltinHandler::Counts,
    ),
    BuiltinSpec::ordinary(
        "CountsBy",
        "Counts transformed elements and returns an association in the current first-pass subset.",
        BuiltinHandler::CountsBy,
    ),
    BuiltinSpec::ordinary(
        "GroupBy",
        "Groups list elements by transformed keys in the current first-pass subset.",
        BuiltinHandler::GroupBy,
    ),
    BuiltinSpec::ordinary(
        "PositionIndex",
        "Builds an association from elements to 1-based position lists in the current first-pass subset.",
        BuiltinHandler::PositionIndex,
    ),
    BuiltinSpec::ordinary(
        "Missing",
        "Represents missing-data markers in the current first-pass subset.",
        BuiltinHandler::Missing,
    ),
    BuiltinSpec::ordinary(
        "SameQ",
        "Tests whether all arguments are exactly identical after evaluation.",
        BuiltinHandler::SameQ,
    ),
    BuiltinSpec::ordinary(
        "UnsameQ",
        "Tests whether no two arguments are exactly identical after evaluation.",
        BuiltinHandler::UnsameQ,
    ),
    BuiltinSpec::ordinary(
        "Equal",
        "Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.",
        BuiltinHandler::Equal,
    ),
    BuiltinSpec::ordinary(
        "Unequal",
        "Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.",
        BuiltinHandler::Unequal,
    ),
    BuiltinSpec::ordinary(
        "Less",
        "Tests strict increasing order in the current exact comparable subset.",
        BuiltinHandler::Less,
    ),
    BuiltinSpec::ordinary(
        "Greater",
        "Tests strict decreasing order in the current exact comparable subset.",
        BuiltinHandler::Greater,
    ),
    BuiltinSpec::ordinary(
        "LessEqual",
        "Tests nondecreasing order in the current exact comparable subset.",
        BuiltinHandler::LessEqual,
    ),
    BuiltinSpec::ordinary(
        "GreaterEqual",
        "Tests nonincreasing order in the current exact comparable subset.",
        BuiltinHandler::GreaterEqual,
    ),
    BuiltinSpec::ordinary(
        "Part",
        "Extracts structural parts using integer positions, All, Span, and exact nested part specifications.",
        BuiltinHandler::Part,
    ),
    BuiltinSpec::ordinary(
        "Extract",
        "Extracts parts at structural positions, optionally wrapping each result before evaluation.",
        BuiltinHandler::Extract,
    ),
    BuiltinSpec::ordinary(
        "ReplacePart",
        "Replaces structural parts via immediate or delayed rules over exact, Span, and pattern-based position specifications, with Heads-aware matching.",
        BuiltinHandler::ReplacePart,
    ),
    BuiltinSpec::special(
        "Set",
        "Assigns an immediate value to a symbol or pattern.",
        BuiltinHandler::Set,
    ),
    BuiltinSpec::special(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
        BuiltinHandler::SetDelayed,
    ),
    BuiltinSpec::special(
        "TagSet",
        "Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.",
        BuiltinHandler::TagSet,
    ),
    BuiltinSpec::special(
        "TagSetDelayed",
        "Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.",
        BuiltinHandler::TagSetDelayed,
    ),
    BuiltinSpec::special(
        "TagUnset",
        "Removes a tagged definition using an explicit tag and lhs.",
        BuiltinHandler::TagUnset,
    ),
    BuiltinSpec::special(
        "Rule",
        "Represents an immediate rewrite rule.",
        BuiltinHandler::Rule,
    ),
    BuiltinSpec::special(
        "RuleDelayed",
        "Represents a delayed rewrite rule.",
        BuiltinHandler::RuleDelayed,
    ),
    BuiltinSpec::special(
        "MatchQ",
        "Tests whether an expression matches a pattern.",
        BuiltinHandler::MatchQ,
    ),
    BuiltinSpec::special(
        "Cases",
        "Collects subexpressions matching a pattern or rule.",
        BuiltinHandler::Cases,
    ),
    BuiltinSpec::special(
        "Count",
        "Counts subexpressions matching a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::Count,
    ),
    BuiltinSpec::special(
        "MemberQ",
        "Tests whether any subexpression matches a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::MemberQ,
    ),
    BuiltinSpec::special(
        "FreeQ",
        "Tests whether no subexpression matches a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::FreeQ,
    ),
    BuiltinSpec::ordinary(
        "Level",
        "Collects subexpressions found on levels specified by a standard levelspec.",
        BuiltinHandler::Level,
    ),
    BuiltinSpec::special(
        "Position",
        "Finds positions of subexpressions matching a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::Position,
    ),
    BuiltinSpec::special(
        "ReplaceAll",
        "Applies rules everywhere in an expression.",
        BuiltinHandler::ReplaceAll,
    ),
    BuiltinSpec::special(
        "ReplaceRepeated",
        "Repeatedly applies rules until an expression reaches a fixed point.",
        BuiltinHandler::ReplaceRepeated,
    ),
    BuiltinSpec::ordinary_with_hold(
        "NormalizeRules",
        "Normalizes a rule or rule list into an explicit canonical list form for debugging.",
        HoldAttribute::HoldAll,
        BuiltinHandler::NormalizeRules,
    ),
    BuiltinSpec::new(
        "$Context",
        "Evaluates to the current default context name as a string.",
    ),
    BuiltinSpec::new(
        "$ContextPath",
        "Evaluates to the active context search path as a list of strings.",
    ),
    BuiltinSpec::new("$Failed", "Represents a failed operation."),
    BuiltinSpec::new(
        "$MessageList",
        "Lists the message names emitted by the most recent top-level evaluation.",
    ),
    BuiltinSpec::new(
        "$Packages",
        "Lists package contexts loaded into the current session.",
    ),
    BuiltinSpec::new("$Path", "Lists search roots used by the package loader."),
    BuiltinSpec::ordinary_with_hold(
        "Context",
        "Returns the context of the current session or of a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Context,
    ),
    BuiltinSpec::new("File", "Represents an explicit file specification."),
    BuiltinSpec::ordinary(
        "FindFile",
        "Finds the full path of a file or package using the current search path.",
        BuiltinHandler::FindFile,
    ),
    BuiltinSpec::ordinary(
        "Get",
        "Reads a file or package and evaluates the expressions it contains.",
        BuiltinHandler::Get,
    ),
    BuiltinSpec::ordinary(
        "Needs",
        "Loads a package only if its context has not already been loaded.",
        BuiltinHandler::Needs,
    ),
    BuiltinSpec::ordinary(
        "Begin",
        "Switches into a context and prepends it to the context path.",
        BuiltinHandler::Begin,
    ),
    BuiltinSpec::ordinary(
        "End",
        "Restores the previous context after a matching Begin.",
        BuiltinHandler::End,
    ),
    BuiltinSpec::ordinary(
        "BeginPackage",
        "Starts a package context with a package-oriented context path.",
        BuiltinHandler::BeginPackage,
    ),
    BuiltinSpec::ordinary(
        "EndPackage",
        "Ends a package context and leaves it on the restored context path.",
        BuiltinHandler::EndPackage,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Message",
        "Emits a message described by a MessageName expression.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Message,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageName",
        "Represents a symbolic message name such as f::tag.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageName,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Messages",
        "Lists message names defined for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Messages,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageDefinitions",
        "Lists stored message template definitions for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageDefinitions,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageTags",
        "Lists message tags defined globally, for a symbol, or for one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageTags,
    ),
    BuiltinSpec::ordinary_with_hold(
        "OwnValues",
        "Lists stored immediate or delayed ownvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::OwnValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "UpValues",
        "Lists stored tagged upvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::UpValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "DownValues",
        "Lists stored downvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::DownValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SubValues",
        "Lists stored subvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SubValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Attributes",
        "Lists the currently active attributes for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Attributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SetAttributes",
        "Adds attributes to one or more symbols.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SetAttributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ClearAttributes",
        "Removes attributes from one or more symbols.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::ClearAttributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Unset",
        "Removes a specific stored definition or message template.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Unset,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Clear",
        "Clears values and rules from one or more symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Clear,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ClearAll",
        "Clears values, rules, messages, and user-defined attributes from one or more symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ClearAll,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Check",
        "Evaluates an expression and substitutes a fallback if matching messages are emitted.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Check,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Quiet",
        "Evaluates an expression while suppressing matching runtime messages it emits.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Quiet,
    ),
    BuiltinSpec::ordinary_with_hold(
        "CompoundExpression",
        "Evaluates expressions sequentially and returns the last result.",
        HoldAttribute::HoldAll,
        BuiltinHandler::CompoundExpression,
    ),
    BuiltinSpec::ordinary_with_hold(
        "If",
        "Evaluates one of several branches conditionally.",
        HoldAttribute::HoldRest,
        BuiltinHandler::If,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Which",
        "Evaluates conditions in order and returns the first matching branch.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Which,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Switch",
        "Matches one evaluated expression against held form/value pairs.",
        HoldAttribute::HoldRest,
        BuiltinHandler::Switch,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Do",
        "Evaluates an expression repeatedly in the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Do,
    ),
    BuiltinSpec::ordinary_with_hold(
        "While",
        "Evaluates a body while its test yields True.",
        HoldAttribute::HoldAll,
        BuiltinHandler::While,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Until",
        "Evaluates a body until its test yields True.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Until,
    ),
    BuiltinSpec::ordinary_with_hold(
        "For",
        "Evaluates start, test, increment, and body in the current first-pass procedural subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::For,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Table",
        "Generates lists using the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Table,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Enclose",
        "Evaluates an expression and catches confirmation failures in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Enclose,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Confirm",
        "Confirms that an expression does not evaluate to a failure-like result in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Confirm,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmBy",
        "Confirms that a test function returns True for an expression in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ConfirmBy,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmMatch",
        "Confirms that an expression matches a held pattern in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ConfirmMatch,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmQuiet",
        "Confirms that an evaluation emits no matching messages in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ConfirmQuiet,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmAssert",
        "Confirms that a test evaluates to True in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ConfirmAssert,
    ),
    BuiltinSpec::ordinary(
        "Return",
        "Exits the current function-style evaluation with an optional result.",
        BuiltinHandler::Return,
    ),
    BuiltinSpec::ordinary(
        "Break",
        "Exits the nearest enclosing loop.",
        BuiltinHandler::Break,
    ),
    BuiltinSpec::ordinary(
        "Continue",
        "Skips to the next iteration of the nearest enclosing loop.",
        BuiltinHandler::Continue,
    ),
    BuiltinSpec::ordinary(
        "Throw",
        "Propagates a nonlocal value, optionally tagged for Catch matching.",
        BuiltinHandler::Throw,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Catch",
        "Evaluates an expression and intercepts matching Throw values.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Catch,
    ),
    BuiltinSpec::ordinary(
        "Failure",
        "Represents a failure object with associated details in the current first-pass subset.",
        BuiltinHandler::Failure,
    ),
    BuiltinSpec::ordinary(
        "FailureQ",
        "Tests whether an expression is a Failure object.",
        BuiltinHandler::FailureQ,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Function",
        "Creates a pure function.",
        HoldAttribute::HoldAll,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary(
        "BlockMap",
        "Maps a function over contiguous blocks in the current first-pass subset.",
        BuiltinHandler::BlockMap,
    ),
    BuiltinSpec::ordinary(
        "SubsetMap",
        "Replaces selected exact parts using a mapped subset result in the current first-pass subset.",
        BuiltinHandler::SubsetMap,
    ),
    BuiltinSpec::ordinary(
        "Comap",
        "Applies selected parts of a function collection to a common expression in the current first-pass subset.",
        BuiltinHandler::Comap,
    ),
    BuiltinSpec::ordinary(
        "ComapApply",
        "Applies selected parts of a function collection as heads of a common expression in the current first-pass subset.",
        BuiltinHandler::ComapApply,
    ),
    BuiltinSpec::ordinary(
        "Accumulate",
        "Returns running totals in the current first-pass subset.",
        BuiltinHandler::Accumulate,
    ),
    BuiltinSpec::ordinary(
        "MapIndexed",
        "Applies a function to parts together with their positions in the current first-pass subset.",
        BuiltinHandler::MapIndexed,
    ),
    BuiltinSpec::ordinary(
        "Map",
        "Applies a function over selected levels of an expression in the current first-pass subset.",
        BuiltinHandler::Map,
    ),
    BuiltinSpec::ordinary(
        "MapAll",
        "Applies a function to every subexpression in the current first-pass subset.",
        BuiltinHandler::MapAll,
    ),
    BuiltinSpec::ordinary(
        "MapAt",
        "Applies a function at exact part specifications in the current first-pass subset.",
        BuiltinHandler::MapAt,
    ),
    BuiltinSpec::ordinary(
        "Scan",
        "Evaluates a function across selected levels for side effects in the current first-pass subset.",
        BuiltinHandler::Scan,
    ),
    BuiltinSpec::ordinary(
        "MapThread",
        "Threads corresponding elements from multiple expressions in the current first-pass subset.",
        BuiltinHandler::MapThread,
    ),
    BuiltinSpec::ordinary(
        "TakeWhile",
        "Takes the longest prefix satisfying a criterion in the current first-pass subset.",
        BuiltinHandler::TakeWhile,
    ),
    BuiltinSpec::ordinary(
        "LengthWhile",
        "Counts the longest prefix satisfying a criterion in the current first-pass subset.",
        BuiltinHandler::LengthWhile,
    ),
    BuiltinSpec::ordinary(
        "Inner",
        "Combines matching tensor dimensions in the current first-pass subset.",
        BuiltinHandler::Inner,
    ),
    BuiltinSpec::ordinary(
        "Fold",
        "Accumulates over elements in the current first-pass subset.",
        BuiltinHandler::Fold,
    ),
    BuiltinSpec::ordinary(
        "FoldList",
        "Returns successive accumulator states in the current first-pass subset.",
        BuiltinHandler::FoldList,
    ),
    BuiltinSpec::ordinary(
        "FoldPair",
        "Accumulates pair-valued state transitions in the current first-pass subset.",
        BuiltinHandler::FoldPair,
    ),
    BuiltinSpec::ordinary(
        "FoldWhile",
        "Accumulates while a test stays True in the current first-pass subset.",
        BuiltinHandler::FoldWhile,
    ),
    BuiltinSpec::ordinary(
        "SequenceFold",
        "Folds a sequence with a sliding history in the current first-pass subset.",
        BuiltinHandler::SequenceFold,
    ),
    BuiltinSpec::ordinary(
        "SequenceFoldList",
        "Returns the running history of a sequence fold in the current first-pass subset.",
        BuiltinHandler::SequenceFoldList,
    ),
    BuiltinSpec::ordinary(
        "Nest",
        "Applies a function repeatedly a fixed number of times.",
        BuiltinHandler::Nest,
    ),
    BuiltinSpec::ordinary(
        "NestWhile",
        "Repeatedly applies a function while a test stays True in the current first-pass subset.",
        BuiltinHandler::NestWhile,
    ),
    BuiltinSpec::ordinary(
        "NestList",
        "Returns repeated applications of a function in the current first-pass subset.",
        BuiltinHandler::NestList,
    ),
    BuiltinSpec::ordinary(
        "NestWhileList",
        "Returns repeated applications while a test stays True in the current first-pass subset.",
        BuiltinHandler::NestWhileList,
    ),
    BuiltinSpec::ordinary(
        "FixedPoint",
        "Repeatedly applies a function until the result stops changing in the current first-pass subset.",
        BuiltinHandler::FixedPoint,
    ),
    BuiltinSpec::ordinary(
        "FixedPointList",
        "Returns successive repeated applications until a fixed point in the current first-pass subset.",
        BuiltinHandler::FixedPointList,
    ),
    BuiltinSpec::ordinary(
        "Apply",
        "Replaces the head of an expression in the current first-pass subset.",
        BuiltinHandler::Apply,
    ),
    BuiltinSpec::ordinary(
        "MapApply",
        "Applies head replacement one level down in the current first-pass subset.",
        BuiltinHandler::MapApply,
    ),
    BuiltinSpec::ordinary(
        "OperatorApplied",
        "Represents a curried operator wrapper in the current first-pass subset.",
        BuiltinHandler::OperatorApplied,
    ),
    BuiltinSpec::ordinary(
        "CurryApplied",
        "Represents a left-to-right curried operator wrapper in the current first-pass subset.",
        BuiltinHandler::CurryApplied,
    ),
    BuiltinSpec::ordinary(
        "ReverseApplied",
        "Represents a wrapper that reverses current call arguments before application in the current first-pass subset.",
        BuiltinHandler::ReverseApplied,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Composition",
        "Represents a left composition of functions in the current first-pass subset.",
        AttributeSet::empty()
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        BuiltinHandler::Composition,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "RightComposition",
        "Represents a right composition of functions in the current first-pass subset.",
        AttributeSet::empty()
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        BuiltinHandler::RightComposition,
    ),
    BuiltinSpec::ordinary(
        "Through",
        "Threads arguments through a functional head in the current first-pass subset.",
        BuiltinHandler::Through,
    ),
    BuiltinSpec::ordinary(
        "Operate",
        "Applies an operator to heads in the head chain of an expression in the current first-pass subset.",
        BuiltinHandler::Operate,
    ),
    BuiltinSpec::ordinary(
        "ArrayFilter",
        "Applies a function to local list-array neighborhoods in the current first-pass subset.",
        BuiltinHandler::ArrayFilter,
    ),
    BuiltinSpec::ordinary(
        "ArrayReduce",
        "Reduces rectangular list arrays across selected dimensions in the current first-pass subset.",
        BuiltinHandler::ArrayReduce,
    ),
    BuiltinSpec::ordinary_with_hold(
        "With",
        "Lexically substitutes local constants into a body.",
        HoldAttribute::HoldAll,
        BuiltinHandler::With,
    ),
    BuiltinSpec::special_with_hold(
        "Module",
        "Creates lexical local symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Module,
    ),
    BuiltinSpec::special_with_hold(
        "Block",
        "Temporarily rebinds symbols dynamically.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Block,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Hold",
        "Prevents evaluation of its arguments.",
        HoldAttribute::HoldAll,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldFirst",
        "Prevents evaluation of the first argument.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldRest",
        "Prevents evaluation of all but the first argument.",
        HoldAttribute::HoldRest,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldAll",
        "Prevents evaluation of all arguments.",
        HoldAttribute::HoldAll,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "HoldAllComplete",
        "Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Unevaluated",
        "Temporarily leaves an argument unevaluated as a direct function argument.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        BuiltinHandler::Unevaluated,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Evaluate",
        "Forces evaluation in held contexts.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Evaluate,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldPattern",
        "Marks a pattern expression so it is matched without prior evaluation.",
        HoldAttribute::HoldAll,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ReleaseHold",
        "Removes supported hold wrappers and resumes evaluation.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ReleaseHold,
    ),
    BuiltinSpec::ordinary(
        "Sequence",
        "Splices a sequence of arguments into a call.",
        BuiltinHandler::Sequence,
    ),
    BuiltinSpec::ordinary(
        "Thread",
        "Threads an expression over arguments that share a common head.",
        BuiltinHandler::Thread,
    ),
    BuiltinSpec::ordinary(
        "InputString",
        "Requests one line of host-provided text and returns it as a string.",
        BuiltinHandler::InputString,
    ),
];

pub fn core_builtin_specs() -> &'static [BuiltinSpec] {
    CORE_BUILTINS
}

pub(super) fn default_builtin_dispatch_spec(name: &str) -> Option<BuiltinSpec> {
    CORE_BUILTINS.iter().find(|spec| spec.name == name).copied()
}

pub(super) fn default_system_attribute_set(symbol: &Symbol) -> AttributeSet {
    if symbol.context() != SYSTEM_CONTEXT {
        return AttributeSet::empty();
    }

    default_builtin_dispatch_spec(symbol.name())
        .map(|spec| spec.attributes)
        .unwrap_or_else(AttributeSet::empty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_catalog_drives_both_dispatch_and_default_attributes() {
        let module = Symbol::new(SYSTEM_CONTEXT, "Module");
        let spec = default_builtin_dispatch_spec("Module").expect("Module should be cataloged");

        assert_eq!(spec.dispatch, BuiltinDispatch::SpecialForm);
        assert_eq!(
            spec.attributes,
            AttributeSet::from_hold(HoldAttribute::HoldAll)
        );
        assert_eq!(default_system_attribute_set(&module), spec.attributes);
    }

    #[test]
    fn non_catalog_system_attributes_can_still_use_manual_fallbacks() {
        let plus = Symbol::new(SYSTEM_CONTEXT, "Plus");
        let attributes = default_system_attribute_set(&plus);

        assert!(attributes.contains(Attribute::Listable));
        assert!(attributes.contains(Attribute::Orderless));
        assert!(attributes.contains(Attribute::Flat));
        assert!(attributes.contains(Attribute::OneIdentity));
    }
}
