use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::predicates;

pub(crate) const PREDICATE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "TrueQ",
        "Tests whether an expression evaluates exactly to True.",
        predicates::eval_true_q,
    ),
    BuiltinSpec::ordinary(
        "BooleanQ",
        "Tests whether an expression evaluates exactly to True or False.",
        predicates::eval_boolean_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Boole",
        "Converts exact True and False to 1 and 0, leaving non-boolean conditions unevaluated.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_boole,
    ),
    BuiltinSpec::ordinary(
        "AtomQ",
        "Tests whether an expression is atomic rather than a normal expression.",
        predicates::eval_atom_q,
    ),
    BuiltinSpec::ordinary(
        "ListQ",
        "Tests whether an expression has head List.",
        predicates::eval_list_q,
    ),
    BuiltinSpec::ordinary(
        "StringQ",
        "Tests whether an expression is a string atom.",
        predicates::eval_string_q,
    ),
    BuiltinSpec::ordinary(
        "IntegerQ",
        "Tests whether an expression is an exact integer atom.",
        predicates::eval_integer_q,
    ),
    BuiltinSpec::ordinary(
        "NumberQ",
        "Tests whether an expression is an exact numeric atom in the current number subset.",
        predicates::eval_number_q,
    ),
    BuiltinSpec::ordinary(
        "NumericQ",
        "Tests whether an expression is currently numeric in the exact-number subset.",
        predicates::eval_numeric_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "EvenQ",
        "Tests whether an expression is currently an even exact integer.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_even_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "OddQ",
        "Tests whether an expression is currently an odd exact integer.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_odd_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Positive",
        "Tests positivity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_positive,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Negative",
        "Tests negativity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_negative,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "NonPositive",
        "Tests non-positivity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_non_positive,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "NonNegative",
        "Tests non-negativity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_non_negative,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PossibleZeroQ",
        "Tests exact zero in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_possible_zero_q,
    ),
    BuiltinSpec::ordinary(
        "MachineNumberQ",
        "Tests whether an expression is a machine number; currently always False because only exact numbers exist.",
        predicates::eval_machine_number_q,
    ),
    BuiltinSpec::ordinary(
        "RealValuedNumberQ",
        "Tests whether an expression is a real-valued number in the current exact-number subset.",
        predicates::eval_real_valued_number_q,
    ),
    BuiltinSpec::ordinary(
        "RealValuedNumericQ",
        "Tests whether an expression is real-valued and numeric in the current exact-number subset.",
        predicates::eval_real_valued_numeric_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeQ",
        "Tests primality in the current exact-integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_prime_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "AlgebraicIntegerQ",
        "Tests whether an expression is an exact integer in the current algebraic-integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_algebraic_integer_q,
    ),
    BuiltinSpec::ordinary(
        "PolynomialQ",
        "Tests whether an expression is polynomial in the current exact symbolic subset.",
        predicates::eval_polynomial_q,
    ),
    BuiltinSpec::ordinary(
        "PolynomialExpressionQ",
        "Tests whether an expression is polynomial in the requested variable subset.",
        predicates::eval_polynomial_expression_q,
    ),
    BuiltinSpec::ordinary(
        "RationalExpressionQ",
        "Tests whether an expression is a rational expression in the requested variable subset.",
        predicates::eval_rational_expression_q,
    ),
    BuiltinSpec::ordinary(
        "DateObjectQ",
        "Tests whether an expression has a DateObject head in the current structural subset.",
        predicates::eval_date_object_q,
    ),
    BuiltinSpec::ordinary(
        "TimeObjectQ",
        "Tests whether an expression has a TimeObject head in the current structural subset.",
        predicates::eval_time_object_q,
    ),
    BuiltinSpec::ordinary(
        "DuplicateFreeQ",
        "Tests whether a List has no repeated elements, optionally using SameTest.",
        predicates::eval_duplicate_free_q,
    ),
    BuiltinSpec::ordinary(
        "IntersectingQ",
        "Tests whether two Lists share any element, optionally using SameTest.",
        predicates::eval_intersecting_q,
    ),
    BuiltinSpec::ordinary(
        "DisjointQ",
        "Tests whether two Lists share no element, optionally using SameTest.",
        predicates::eval_disjoint_q,
    ),
    BuiltinSpec::ordinary(
        "SubsetQ",
        "Tests subset inclusion for Lists, optionally using SameTest.",
        predicates::eval_subset_q,
    ),
    BuiltinSpec::ordinary(
        "Order",
        "Orders two expressions using exact numeric order when possible, then the current canonical textual ordering subset.",
        predicates::eval_order,
    ),
    BuiltinSpec::ordinary(
        "OrderedQ",
        "Tests whether an expression's arguments are ordered under the default order or a supplied binary predicate.",
        predicates::eval_ordered_q,
    ),
    BuiltinSpec::ordinary(
        "LexicographicOrder",
        "Compares two expressions lexicographically under the default order or a supplied binary predicate.",
        predicates::eval_lexicographic_order,
    ),
    BuiltinSpec::ordinary(
        "NumericalOrder",
        "Compares two expressions using exact numeric order when possible, then canonical order.",
        predicates::eval_numerical_order,
    ),
    BuiltinSpec::ordinary(
        "AlphabeticOrder",
        "Compares two strings alphabetically in the current string-order subset.",
        predicates::eval_alphabetic_order,
    ),
    BuiltinSpec::ordinary(
        "ContainsAll",
        "Tests whether a List contains every requested element, optionally using SameTest.",
        predicates::eval_contains_all,
    ),
    BuiltinSpec::ordinary(
        "ContainsAny",
        "Tests whether a List contains any requested element, optionally using SameTest.",
        predicates::eval_contains_any,
    ),
    BuiltinSpec::ordinary(
        "ContainsExactly",
        "Tests List multiset equality, optionally using SameTest.",
        predicates::eval_contains_exactly,
    ),
    BuiltinSpec::ordinary(
        "ContainsNone",
        "Tests whether a List contains none of the requested elements, optionally using SameTest.",
        predicates::eval_contains_none,
    ),
    BuiltinSpec::ordinary(
        "ContainsOnly",
        "Tests whether every element of a List belongs to the allowed-element List, optionally using SameTest.",
        predicates::eval_contains_only,
    ),
    BuiltinSpec::ordinary(
        "Between",
        "Tests interval inclusion in the current exact numeric subset.",
        predicates::eval_between,
    ),
    BuiltinSpec::ordinary(
        "VectorQ",
        "Tests whether an expression is a one-dimensional List array in the current exact structural subset.",
        predicates::eval_vector_q,
    ),
    BuiltinSpec::ordinary(
        "MatrixQ",
        "Tests whether an expression is a rectangular two-dimensional List array in the current exact structural subset.",
        predicates::eval_matrix_q,
    ),
    BuiltinSpec::ordinary(
        "ArrayQ",
        "Tests whether an expression is a rectangular List array in the current exact structural subset.",
        predicates::eval_array_q,
    ),
    BuiltinSpec::ordinary(
        "AllTrue",
        "Tests whether all selected structural parts satisfy a criterion in the current boolean subset.",
        predicates::eval_all_true,
    ),
    BuiltinSpec::ordinary(
        "AnyTrue",
        "Tests whether any selected structural part satisfies a criterion in the current boolean subset.",
        predicates::eval_any_true,
    ),
    BuiltinSpec::ordinary(
        "NoneTrue",
        "Tests whether no selected structural parts satisfy a criterion in the current boolean subset.",
        predicates::eval_none_true,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "CoprimeQ",
        "Tests whether exact integers are pairwise relatively prime in the current integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_coprime_q,
    ),
    BuiltinSpec::ordinary(
        "SameQ",
        "Tests whether all arguments are exactly identical after evaluation.",
        predicates::eval_same_q,
    ),
    BuiltinSpec::ordinary(
        "UnsameQ",
        "Tests whether no two arguments are exactly identical after evaluation.",
        predicates::eval_unsame_q,
    ),
];
