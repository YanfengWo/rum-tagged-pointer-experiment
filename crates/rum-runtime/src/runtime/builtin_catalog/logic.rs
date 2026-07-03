use super::*;

pub(super) const LOGIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "And",
        "Evaluates a conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        crate::runtime::logic::eval_and,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Or",
        "Evaluates a disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        crate::runtime::logic::eval_or,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Nand",
        "Evaluates a negated conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        builtin_nand,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Nor",
        "Evaluates a negated disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        builtin_nor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Xor",
        "Evaluates exclusive-or over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty()
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        crate::runtime::logic::eval_xor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Equivalent",
        "Evaluates logical equivalence over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty().with(Attribute::Orderless),
        crate::runtime::logic::eval_equivalent,
    ),
    BuiltinSpec::ordinary(
        "Implies",
        "Evaluates binary logical implication over exact boolean arguments in the current subset.",
        crate::runtime::logic::eval_implies,
    ),
    BuiltinSpec::ordinary(
        "Not",
        "Negates a boolean value in the current exact boolean subset.",
        crate::runtime::logic::eval_not,
    ),
    BuiltinSpec::ordinary(
        "TrueQ",
        "Tests whether an expression evaluates exactly to True.",
        Session::eval_true_q,
    ),
    BuiltinSpec::ordinary(
        "BooleanQ",
        "Tests whether an expression evaluates exactly to True or False.",
        Session::eval_boolean_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Boole",
        "Converts exact True and False to 1 and 0, leaving non-boolean conditions unevaluated.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_boole,
    ),
    BuiltinSpec::ordinary(
        "AtomQ",
        "Tests whether an expression is atomic rather than a normal expression.",
        Session::eval_atom_q,
    ),
    BuiltinSpec::ordinary(
        "ListQ",
        "Tests whether an expression has head List.",
        Session::eval_list_q,
    ),
    BuiltinSpec::ordinary(
        "StringQ",
        "Tests whether an expression is a string atom.",
        Session::eval_string_q,
    ),
    BuiltinSpec::ordinary(
        "IntegerQ",
        "Tests whether an expression is an exact integer atom.",
        Session::eval_integer_q,
    ),
    BuiltinSpec::ordinary(
        "NumberQ",
        "Tests whether an expression is an exact numeric atom in the current number subset.",
        Session::eval_number_q,
    ),
    BuiltinSpec::ordinary(
        "NumericQ",
        "Tests whether an expression is currently numeric in the exact-number subset.",
        Session::eval_numeric_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "EvenQ",
        "Tests whether an expression is currently an even exact integer.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_even_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "OddQ",
        "Tests whether an expression is currently an odd exact integer.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_odd_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Positive",
        "Tests positivity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_positive,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Negative",
        "Tests negativity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_negative,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "NonPositive",
        "Tests non-positivity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_non_positive,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "NonNegative",
        "Tests non-negativity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_non_negative,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PossibleZeroQ",
        "Tests exact zero in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_possible_zero_q,
    ),
    BuiltinSpec::ordinary(
        "MachineNumberQ",
        "Tests whether an expression is a machine number; currently always False because only exact numbers exist.",
        Session::eval_machine_number_q,
    ),
    BuiltinSpec::ordinary(
        "RealValuedNumberQ",
        "Tests whether an expression is a real-valued number in the current exact-number subset.",
        Session::eval_real_valued_number_q,
    ),
    BuiltinSpec::ordinary(
        "RealValuedNumericQ",
        "Tests whether an expression is real-valued and numeric in the current exact-number subset.",
        Session::eval_real_valued_numeric_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeQ",
        "Tests primality in the current exact-integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_prime_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "AlgebraicIntegerQ",
        "Tests whether an expression is an exact integer in the current algebraic-integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_algebraic_integer_q,
    ),
    BuiltinSpec::ordinary(
        "PolynomialQ",
        "Tests whether an expression is polynomial in the current exact symbolic subset.",
        Session::eval_polynomial_q,
    ),
    BuiltinSpec::ordinary(
        "PolynomialExpressionQ",
        "Tests whether an expression is polynomial in the requested variable subset.",
        Session::eval_polynomial_expression_q,
    ),
    BuiltinSpec::ordinary(
        "RationalExpressionQ",
        "Tests whether an expression is a rational expression in the requested variable subset.",
        Session::eval_rational_expression_q,
    ),
    BuiltinSpec::ordinary(
        "DateObjectQ",
        "Tests whether an expression has a DateObject head in the current structural subset.",
        Session::eval_date_object_q,
    ),
    BuiltinSpec::ordinary(
        "TimeObjectQ",
        "Tests whether an expression has a TimeObject head in the current structural subset.",
        Session::eval_time_object_q,
    ),
    BuiltinSpec::ordinary(
        "DuplicateFreeQ",
        "Tests whether a List has no repeated elements in the current exact structural subset.",
        Session::eval_duplicate_free_q,
    ),
    BuiltinSpec::ordinary(
        "IntersectingQ",
        "Tests whether two Lists share any element in the current exact structural subset.",
        Session::eval_intersecting_q,
    ),
    BuiltinSpec::ordinary(
        "DisjointQ",
        "Tests whether two Lists share no element in the current exact structural subset.",
        Session::eval_disjoint_q,
    ),
    BuiltinSpec::ordinary(
        "SubsetQ",
        "Tests subset inclusion for Lists in the current exact structural subset.",
        Session::eval_subset_q,
    ),
    BuiltinSpec::ordinary(
        "Order",
        "Orders two expressions using exact numeric order when possible, then the current canonical textual ordering subset.",
        Session::eval_order,
    ),
    BuiltinSpec::ordinary(
        "OrderedQ",
        "Tests whether a List is nondecreasing under the current exact order subset.",
        Session::eval_ordered_q,
    ),
    BuiltinSpec::ordinary(
        "LexicographicOrder",
        "Compares two expressions lexicographically in the current exact order subset.",
        Session::eval_lexicographic_order,
    ),
    BuiltinSpec::ordinary(
        "NumericalOrder",
        "Compares two expressions using exact numeric order when possible, then canonical order.",
        Session::eval_numerical_order,
    ),
    BuiltinSpec::ordinary(
        "AlphabeticOrder",
        "Compares two strings alphabetically in the current string-order subset.",
        Session::eval_alphabetic_order,
    ),
    BuiltinSpec::ordinary(
        "ContainsAll",
        "Tests whether a List contains every requested element in the current exact structural subset.",
        Session::eval_contains_all,
    ),
    BuiltinSpec::ordinary(
        "ContainsAny",
        "Tests whether a List contains any requested element in the current exact structural subset.",
        Session::eval_contains_any,
    ),
    BuiltinSpec::ordinary(
        "ContainsExactly",
        "Tests List multiset equality in the current exact structural subset.",
        Session::eval_contains_exactly,
    ),
    BuiltinSpec::ordinary(
        "ContainsNone",
        "Tests whether a List contains none of the requested elements in the current exact structural subset.",
        Session::eval_contains_none,
    ),
    BuiltinSpec::ordinary(
        "ContainsOnly",
        "Tests whether every element of a List belongs to the allowed-element List in the current exact structural subset.",
        Session::eval_contains_only,
    ),
    BuiltinSpec::ordinary(
        "Between",
        "Tests interval inclusion in the current exact numeric subset.",
        Session::eval_between,
    ),
    BuiltinSpec::ordinary(
        "VectorQ",
        "Tests whether an expression is a one-dimensional List array in the current exact structural subset.",
        Session::eval_vector_q,
    ),
    BuiltinSpec::ordinary(
        "MatrixQ",
        "Tests whether an expression is a rectangular two-dimensional List array in the current exact structural subset.",
        Session::eval_matrix_q,
    ),
    BuiltinSpec::ordinary(
        "ArrayQ",
        "Tests whether an expression is a rectangular List array in the current exact structural subset.",
        Session::eval_array_q,
    ),
    BuiltinSpec::ordinary(
        "AllTrue",
        "Tests whether all selected structural parts satisfy a criterion in the current boolean subset.",
        Session::eval_all_true,
    ),
    BuiltinSpec::ordinary(
        "AnyTrue",
        "Tests whether any selected structural part satisfies a criterion in the current boolean subset.",
        Session::eval_any_true,
    ),
    BuiltinSpec::ordinary(
        "NoneTrue",
        "Tests whether no selected structural parts satisfy a criterion in the current boolean subset.",
        Session::eval_none_true,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "CoprimeQ",
        "Tests whether exact integers are pairwise relatively prime in the current integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_coprime_q,
    ),
    BuiltinSpec::ordinary(
        "SameQ",
        "Tests whether all arguments are exactly identical after evaluation.",
        Session::eval_same_q,
    ),
    BuiltinSpec::ordinary(
        "UnsameQ",
        "Tests whether no two arguments are exactly identical after evaluation.",
        Session::eval_unsame_q,
    ),
    BuiltinSpec::ordinary(
        "Equal",
        "Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.",
        crate::runtime::logic::eval_equal,
    ),
    BuiltinSpec::ordinary(
        "Unequal",
        "Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.",
        crate::runtime::logic::eval_unequal,
    ),
    BuiltinSpec::ordinary(
        "Less",
        "Tests strict increasing order in the current exact comparable subset.",
        crate::runtime::logic::eval_less,
    ),
    BuiltinSpec::ordinary(
        "Greater",
        "Tests strict decreasing order in the current exact comparable subset.",
        crate::runtime::logic::eval_greater,
    ),
    BuiltinSpec::ordinary(
        "LessEqual",
        "Tests nondecreasing order in the current exact comparable subset.",
        crate::runtime::logic::eval_less_equal,
    ),
    BuiltinSpec::ordinary(
        "GreaterEqual",
        "Tests nonincreasing order in the current exact comparable subset.",
        crate::runtime::logic::eval_greater_equal,
    ),
    BuiltinSpec::ordinary(
        "LessThan",
        "Represents a unary comparison operator equivalent to Less[#, bound] when applied.",
        crate::runtime::logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "GreaterThan",
        "Represents a unary comparison operator equivalent to Greater[#, bound] when applied.",
        crate::runtime::logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "LessEqualThan",
        "Represents a unary comparison operator equivalent to LessEqual[#, bound] when applied.",
        crate::runtime::logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "GreaterEqualThan",
        "Represents a unary comparison operator equivalent to GreaterEqual[#, bound] when applied.",
        crate::runtime::logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "EqualTo",
        "Represents a unary comparison operator equivalent to Equal[#, bound] when applied.",
        crate::runtime::logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "UnequalTo",
        "Represents a unary comparison operator equivalent to Unequal[#, bound] when applied.",
        crate::runtime::logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "SameAs",
        "Represents a unary comparison operator equivalent to SameQ[#, bound] when applied.",
        crate::runtime::logic::eval_comparison_operator_wrapper,
    ),
];
