use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec, Capability, HoldAttribute};

use crate::{
    associations, calculus, combinatorics, control_flow, functional_ops, integer_digits, logic,
    number_theory, polynomials, predicates, rewrite_builtins, strings, structure_ops,
};

pub(crate) const LOGIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "And",
        "Evaluates a conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_and,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Or",
        "Evaluates a disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_or,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Nand",
        "Evaluates a negated conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_nand,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Nor",
        "Evaluates a negated disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_nor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Xor",
        "Evaluates exclusive-or over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty()
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        logic::eval_xor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Equivalent",
        "Evaluates logical equivalence over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty().with(Attribute::Orderless),
        logic::eval_equivalent,
    ),
    BuiltinSpec::ordinary(
        "Implies",
        "Evaluates binary logical implication over exact boolean arguments in the current subset.",
        logic::eval_implies,
    ),
    BuiltinSpec::ordinary(
        "Not",
        "Negates a boolean value in the current exact boolean subset.",
        logic::eval_not,
    ),
    BuiltinSpec::ordinary(
        "Equal",
        "Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.",
        logic::eval_equal,
    ),
    BuiltinSpec::ordinary(
        "Unequal",
        "Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.",
        logic::eval_unequal,
    ),
    BuiltinSpec::ordinary(
        "Less",
        "Tests strict increasing order in the current exact comparable subset.",
        logic::eval_less,
    ),
    BuiltinSpec::ordinary(
        "Greater",
        "Tests strict decreasing order in the current exact comparable subset.",
        logic::eval_greater,
    ),
    BuiltinSpec::ordinary(
        "LessEqual",
        "Tests nondecreasing order in the current exact comparable subset.",
        logic::eval_less_equal,
    ),
    BuiltinSpec::ordinary(
        "GreaterEqual",
        "Tests nonincreasing order in the current exact comparable subset.",
        logic::eval_greater_equal,
    ),
    BuiltinSpec::ordinary(
        "LessThan",
        "Represents a unary comparison operator equivalent to Less[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "GreaterThan",
        "Represents a unary comparison operator equivalent to Greater[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "LessEqualThan",
        "Represents a unary comparison operator equivalent to LessEqual[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "GreaterEqualThan",
        "Represents a unary comparison operator equivalent to GreaterEqual[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "EqualTo",
        "Represents a unary comparison operator equivalent to Equal[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "UnequalTo",
        "Represents a unary comparison operator equivalent to Unequal[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "SameAs",
        "Represents a unary comparison operator equivalent to SameQ[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
];

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
        "Tests whether a List has no repeated elements in the current exact structural subset.",
        predicates::eval_duplicate_free_q,
    ),
    BuiltinSpec::ordinary(
        "IntersectingQ",
        "Tests whether two Lists share any element in the current exact structural subset.",
        predicates::eval_intersecting_q,
    ),
    BuiltinSpec::ordinary(
        "DisjointQ",
        "Tests whether two Lists share no element in the current exact structural subset.",
        predicates::eval_disjoint_q,
    ),
    BuiltinSpec::ordinary(
        "SubsetQ",
        "Tests subset inclusion for Lists in the current exact structural subset.",
        predicates::eval_subset_q,
    ),
    BuiltinSpec::ordinary(
        "Order",
        "Orders two expressions using exact numeric order when possible, then the current canonical textual ordering subset.",
        predicates::eval_order,
    ),
    BuiltinSpec::ordinary(
        "OrderedQ",
        "Tests whether a List is nondecreasing under the current exact order subset.",
        predicates::eval_ordered_q,
    ),
    BuiltinSpec::ordinary(
        "LexicographicOrder",
        "Compares two expressions lexicographically in the current exact order subset.",
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
        "Tests whether a List contains every requested element in the current exact structural subset.",
        predicates::eval_contains_all,
    ),
    BuiltinSpec::ordinary(
        "ContainsAny",
        "Tests whether a List contains any requested element in the current exact structural subset.",
        predicates::eval_contains_any,
    ),
    BuiltinSpec::ordinary(
        "ContainsExactly",
        "Tests List multiset equality in the current exact structural subset.",
        predicates::eval_contains_exactly,
    ),
    BuiltinSpec::ordinary(
        "ContainsNone",
        "Tests whether a List contains none of the requested elements in the current exact structural subset.",
        predicates::eval_contains_none,
    ),
    BuiltinSpec::ordinary(
        "ContainsOnly",
        "Tests whether every element of a List belongs to the allowed-element List in the current exact structural subset.",
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

const SET_OPERATION_ATTRIBUTES: AttributeSet = AttributeSet::empty()
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

pub(crate) const STRUCTURE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FullForm",
        "Renders the held argument as a canonical structural-form string in the current core expression subset.",
        structure_ops::eval_full_form,
    ),
    BuiltinSpec::ordinary(
        "Head",
        "Returns the structural head of an expression, optionally wrapped in a second head.",
        structure_ops::eval_head,
    ),
    BuiltinSpec::ordinary(
        "Length",
        "Returns the number of structural elements in an expression.",
        structure_ops::eval_length,
    ),
    BuiltinSpec::ordinary(
        "Depth",
        "Returns the structural depth of an expression, optionally including heads.",
        structure_ops::eval_depth,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LeafCount",
        "Counts atomic leaves in an expression's full structural tree, including heads.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_leaf_count,
    ),
    BuiltinSpec::ordinary_with_hold(
        "First",
        "Returns the first structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        structure_ops::eval_first,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Last",
        "Returns the last structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        structure_ops::eval_last,
    ),
    BuiltinSpec::ordinary(
        "Rest",
        "Returns an expression with its first structural element removed.",
        structure_ops::eval_rest,
    ),
    BuiltinSpec::ordinary(
        "Most",
        "Returns an expression with its last structural element removed.",
        structure_ops::eval_most,
    ),
    BuiltinSpec::ordinary(
        "Take",
        "Takes structural elements using the current standard sequence-spec subset.",
        structure_ops::eval_take,
    ),
    BuiltinSpec::ordinary(
        "Drop",
        "Drops structural elements using the current standard sequence-spec subset.",
        structure_ops::eval_drop,
    ),
    BuiltinSpec::ordinary(
        "TakeDrop",
        "Returns both the taken and dropped structural elements using the current standard sequence-spec subset.",
        structure_ops::eval_take_drop,
    ),
    BuiltinSpec::ordinary(
        "Insert",
        "Inserts an element at a top-level structural position in the current first-pass subset.",
        structure_ops::eval_insert,
    ),
    BuiltinSpec::ordinary(
        "Delete",
        "Deletes one or more top-level structural positions in the current first-pass subset.",
        structure_ops::eval_delete,
    ),
    BuiltinSpec::ordinary(
        "TakeList",
        "Splits a normal expression into sequential chunks by non-negative counts.",
        structure_ops::eval_take_list,
    ),
    BuiltinSpec::ordinary(
        "Range",
        "Constructs a plain list of exact integers from the current one-, two-, and three-argument subset.",
        structure_ops::eval_range,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Differences",
        "Returns successive differences for plain lists, including the current non-negative order subset.",
        AttributeSet::empty()
            .with(Attribute::NonThreadable)
            .with(Attribute::Protected),
        structure_ops::eval_differences,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ConstantArray",
        "Constructs a constant plain list for non-negative integer scalar and list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_constant_array,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PadLeft",
        "Pads or truncates a normal expression on the left using the current length and padding-element subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_pad_left,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PadRight",
        "Pads or truncates a normal expression on the right using the current length and padding-element subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_pad_right,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Subsets",
        "Returns subsets of a normal expression in Wolfram-compatible size-major order for the current size-spec subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_subsets,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Tuples",
        "Returns repeated tuples or cartesian products for normal expressions in the current subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_tuples,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Partition",
        "Partitions a normal expression into fixed-size blocks using the current positive offset subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_partition,
    ),
    BuiltinSpec::ordinary(
        "Flatten",
        "Flattens nested expressions through the current non-negative level, single-head-spec, and level-list subset.",
        structure_ops::eval_flatten,
    ),
    BuiltinSpec::ordinary(
        "Join",
        "Concatenates same-head normal expressions and associations in the current top-level subset.",
        structure_ops::eval_join,
    ),
    BuiltinSpec::ordinary(
        "Reverse",
        "Reverses normal-expression arguments at the current positive level subset.",
        structure_ops::eval_reverse,
    ),
    BuiltinSpec::ordinary(
        "Transpose",
        "Transposes rectangular plain-list matrices in the current first-pass subset.",
        structure_ops::eval_transpose,
    ),
    BuiltinSpec::ordinary(
        "DeleteDuplicates",
        "Keeps the first occurrence of structurally equal top-level elements.",
        structure_ops::eval_delete_duplicates,
    ),
    BuiltinSpec::ordinary(
        "DeleteDuplicatesBy",
        "Keeps the first top-level element for each evaluated key in the current unary-function subset.",
        structure_ops::eval_delete_duplicates_by,
    ),
    BuiltinSpec::ordinary(
        "Sort",
        "Sorts top-level elements using the current structural ordering subset.",
        structure_ops::eval_sort,
    ),
    BuiltinSpec::ordinary(
        "SortBy",
        "Sorts top-level elements by an evaluated key in the current unary-function subset.",
        structure_ops::eval_sort_by,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Union",
        "Returns sorted unique elements across same-head normal expressions.",
        SET_OPERATION_ATTRIBUTES,
        structure_ops::eval_union,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Intersection",
        "Returns sorted unique elements common to all same-head normal expressions.",
        SET_OPERATION_ATTRIBUTES,
        structure_ops::eval_intersection,
    ),
    BuiltinSpec::ordinary(
        "Complement",
        "Returns sorted unique elements from the first expression that are absent from later same-head expressions.",
        structure_ops::eval_complement,
    ),
    BuiltinSpec::ordinary(
        "Append",
        "Appends an element to the end of a normal expression, with current operator-form support.",
        structure_ops::eval_append,
    ),
    BuiltinSpec::ordinary(
        "Prepend",
        "Prepends an element to the front of a normal expression, with current operator-form support.",
        structure_ops::eval_prepend,
    ),
    BuiltinSpec::ordinary(
        "Symbol",
        "Resolves a symbol from a string name using the current context path.",
        structure_ops::eval_symbol,
    ),
    BuiltinSpec::ordinary(
        "Construct",
        "Constructs an expression from a head and zero or more arguments.",
        structure_ops::eval_construct,
    ),
];

pub(crate) const ASSOCIATION_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Association",
        "Builds an association from Rule/RuleDelayed entries in the current first-pass subset.",
        associations::eval_association,
    ),
    BuiltinSpec::ordinary(
        "AssociationQ",
        "Tests whether an expression is an Association with rule entries.",
        associations::eval_association_q,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Lookup",
        "Looks up a key in an association in the current first-pass subset.",
        HoldAttribute::HoldAll,
        associations::eval_lookup,
    ),
    BuiltinSpec::ordinary(
        "Key",
        "Represents a keyed selector in the current first-pass association subset.",
        associations::eval_key,
    ),
    BuiltinSpec::ordinary(
        "Keys",
        "Returns association keys in structural order.",
        associations::eval_keys,
    ),
    BuiltinSpec::ordinary(
        "Values",
        "Returns association values in structural order.",
        associations::eval_values,
    ),
    BuiltinSpec::ordinary(
        "KeyExistsQ",
        "Tests whether an association contains a given key in the current first-pass subset.",
        associations::eval_key_exists_q,
    ),
    BuiltinSpec::ordinary(
        "KeyTake",
        "Keeps selected keys from an association in the current first-pass subset.",
        associations::eval_key_take,
    ),
    BuiltinSpec::ordinary(
        "KeyDrop",
        "Drops selected keys from an association in the current first-pass subset.",
        associations::eval_key_drop,
    ),
    BuiltinSpec::ordinary(
        "KeyMap",
        "Maps a function over association keys in the current first-pass subset.",
        associations::eval_key_map,
    ),
    BuiltinSpec::ordinary(
        "KeyValueMap",
        "Maps a binary function over association key-value pairs in the current first-pass subset.",
        associations::eval_key_value_map,
    ),
    BuiltinSpec::ordinary(
        "KeySelect",
        "Selects association entries whose keys satisfy a criterion in the current first-pass subset.",
        associations::eval_key_select,
    ),
    BuiltinSpec::ordinary(
        "KeyMemberQ",
        "Tests whether an association contains a key in the current first-pass subset.",
        associations::eval_key_member_q,
    ),
    BuiltinSpec::ordinary(
        "KeyFreeQ",
        "Tests whether an association does not contain a key in the current first-pass subset.",
        associations::eval_key_free_q,
    ),
    BuiltinSpec::ordinary(
        "KeySort",
        "Sorts association entries by key in the current canonical ordering subset.",
        associations::eval_key_sort,
    ),
    BuiltinSpec::ordinary(
        "KeySortBy",
        "Sorts association entries by transformed keys in the current first-pass subset.",
        associations::eval_key_sort_by,
    ),
    BuiltinSpec::ordinary(
        "AssociationMap",
        "Builds an association by mapping a function over keys in the current first-pass subset.",
        associations::eval_association_map,
    ),
    BuiltinSpec::ordinary(
        "AssociationThread",
        "Threads rule or key/value inputs into an association in the current first-pass subset.",
        associations::eval_association_thread,
    ),
    BuiltinSpec::ordinary(
        "KeyUnion",
        "Builds keyed unions, including the current missing-key fill-function form.",
        associations::eval_key_union,
    ),
    BuiltinSpec::ordinary(
        "KeyIntersection",
        "Builds a key-intersection association in the current first-pass subset.",
        associations::eval_key_intersection,
    ),
    BuiltinSpec::ordinary(
        "KeyComplement",
        "Builds a key-complement association in the current first-pass subset.",
        associations::eval_key_complement,
    ),
    BuiltinSpec::ordinary(
        "Normal",
        "Converts an association into a list of rules in the current first-pass subset.",
        associations::eval_normal,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AssociateTo",
        "Adds or replaces association entries, including held symbol mutation in the current first-pass subset.",
        HoldAttribute::HoldFirst,
        associations::eval_associate_to,
    ),
    BuiltinSpec::ordinary_with_hold(
        "KeyDropFrom",
        "Drops keys from an association, including held symbol mutation in the current first-pass subset.",
        HoldAttribute::HoldFirst,
        associations::eval_key_drop_from,
    ),
    BuiltinSpec::ordinary(
        "KeyValuePattern",
        "Represents an association key/value pattern specification in the current first-pass subset.",
        associations::eval_key_value_pattern,
    ),
    BuiltinSpec::ordinary(
        "Merge",
        "Merges associations by key using a combiner in the current first-pass subset.",
        associations::eval_merge,
    ),
    BuiltinSpec::ordinary(
        "Catenate",
        "Concatenates lists and association values into a list in the current first-pass subset.",
        associations::eval_catenate,
    ),
    BuiltinSpec::ordinary(
        "Counts",
        "Counts repeated elements and returns an association in the current first-pass subset.",
        associations::eval_counts,
    ),
    BuiltinSpec::ordinary(
        "CountsBy",
        "Counts transformed elements and returns an association in the current first-pass subset.",
        associations::eval_counts_by,
    ),
    BuiltinSpec::ordinary(
        "GroupBy",
        "Groups list elements by transformed keys, including the current reducer form.",
        associations::eval_group_by,
    ),
    BuiltinSpec::ordinary(
        "PositionIndex",
        "Builds an association from elements to 1-based position lists in the current first-pass subset.",
        associations::eval_position_index,
    ),
    BuiltinSpec::ordinary(
        "Missing",
        "Represents missing-data markers in the current first-pass subset.",
        associations::eval_missing,
    ),
    BuiltinSpec::ordinary(
        "DeleteMissing",
        "Deletes Missing entries from lists, associations, and rule-list data in the current first-pass subset.",
        associations::eval_delete_missing,
    ),
];

pub(crate) const COMBINATORICS_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "Binomial",
        "Computes exact binomial coefficients for integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        combinatorics::eval_binomial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Fibonacci",
        "Computes exact Fibonacci numbers for integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_fibonacci,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PartitionsP",
        "Computes the number of integer partitions by Euler's pentagonal recurrence.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_partitions_p,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PartitionsQ",
        "Computes the number of partitions into distinct parts.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_partitions_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "BernoulliB",
        "Computes exact Bernoulli numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_bernoulli_b,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Subfactorial",
        "Computes derangement numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_subfactorial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StirlingS1",
        "Computes signed Stirling numbers of the first kind.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_stirling_s1,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StirlingS2",
        "Computes Stirling numbers of the second kind.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_stirling_s2,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "CatalanNumber",
        "Computes exact Catalan numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_catalan_number,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Multinomial",
        "Computes exact multinomial coefficients for non-negative integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Orderless)
            .with(Attribute::Protected),
        combinatorics::eval_multinomial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Pochhammer",
        "Computes finite rising factorial products for non-negative integer counts.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        combinatorics::eval_pochhammer,
    ),
];

pub(crate) const CONTROL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "CompoundExpression",
        "Evaluates expressions sequentially and returns the last result.",
        HoldAttribute::HoldAll,
        control_flow::eval_compound_expression,
    ),
    BuiltinSpec::ordinary_with_hold(
        "If",
        "Evaluates one of several branches conditionally.",
        HoldAttribute::HoldRest,
        control_flow::eval_if,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Which",
        "Evaluates conditions in order and returns the first matching branch.",
        HoldAttribute::HoldAll,
        control_flow::eval_which,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Switch",
        "Matches one evaluated expression against held form/value pairs.",
        HoldAttribute::HoldRest,
        control_flow::eval_switch,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Do",
        "Evaluates an expression repeatedly in the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_do,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Product",
        "Multiplies expression values over concrete iterator specifications.",
        AttributeSet::from_hold(HoldAttribute::HoldAll)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        control_flow::eval_product,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Sum",
        "Adds expression values over concrete iterator specifications.",
        AttributeSet::from_hold(HoldAttribute::HoldAll)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        control_flow::eval_sum,
    ),
    BuiltinSpec::ordinary_with_hold(
        "While",
        "Evaluates a body while its test yields True.",
        HoldAttribute::HoldAll,
        control_flow::eval_while,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Until",
        "Evaluates a body until its test yields True.",
        HoldAttribute::HoldAll,
        control_flow::eval_until,
    ),
    BuiltinSpec::ordinary_with_hold(
        "For",
        "Evaluates start, test, increment, and body in the current first-pass procedural subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_for,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Table",
        "Generates lists using the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_table,
    ),
    BuiltinSpec::ordinary(
        "Abort",
        "Generates an abort that terminates the current evaluation unless caught.",
        control_flow::eval_abort,
    ),
    BuiltinSpec::ordinary(
        "Exit",
        "Requests host-level termination with an optional integer exit code.",
        control_flow::eval_exit,
    ),
    BuiltinSpec::ordinary_with_hold(
        "CheckAbort",
        "Evaluates an expression and substitutes a fallback if an Abort occurs.",
        HoldAttribute::HoldAll,
        control_flow::eval_check_abort,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AbortProtect",
        "Evaluates an expression while delaying any abort until the protected region exits in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_abort_protect,
    ),
    BuiltinSpec::ordinary_with_hold(
        "WithCleanup",
        "Evaluates cleanup code even when the main expression exits abnormally in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_with_cleanup,
    ),
    BuiltinSpec::ordinary(
        "Failsafe",
        "Creates a callable wrapper that preserves failure-like inputs in the current first-pass subset.",
        control_flow::eval_failsafe,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Assert",
        "Represents an assertion site and supports the current session flag plus $AssertFunction hook.",
        HoldAttribute::HoldAll,
        control_flow::eval_assert,
    ),
    BuiltinSpec::new(
        "$AssertFunction",
        "Stores the optional assertion failure callback used by Assert in the current first-pass subset.",
    ),
    BuiltinSpec::new(
        "PropagateAborts",
        "Represents the current first-pass CheckAbort option for rethrowing caught aborts.",
    ),
    BuiltinSpec::ordinary_with_hold(
        "Enclose",
        "Evaluates an expression and catches confirmation failures in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_enclose,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Confirm",
        "Confirms that an expression does not evaluate to a failure-like result in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmBy",
        "Confirms that a test function returns True for an expression in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_by,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmMatch",
        "Confirms that an expression matches a held pattern in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_match,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmQuiet",
        "Confirms that an evaluation emits no matching messages in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_quiet,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmAssert",
        "Confirms that a test evaluates to True in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_assert,
    ),
    BuiltinSpec::ordinary(
        "Return",
        "Exits the current function-style evaluation with an optional result.",
        control_flow::eval_return,
    ),
    BuiltinSpec::ordinary(
        "Break",
        "Exits the nearest enclosing loop.",
        control_flow::eval_break,
    ),
    BuiltinSpec::ordinary(
        "Continue",
        "Skips to the next iteration of the nearest enclosing loop.",
        control_flow::eval_continue,
    ),
    BuiltinSpec::ordinary(
        "Throw",
        "Propagates a nonlocal value, optionally tagged for Catch matching.",
        control_flow::eval_throw,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Catch",
        "Evaluates an expression and intercepts matching Throw values.",
        HoldAttribute::HoldAll,
        control_flow::eval_catch,
    ),
    BuiltinSpec::ordinary(
        "Failure",
        "Represents a failure object with associated details in the current first-pass subset.",
        control_flow::eval_failure,
    ),
    BuiltinSpec::ordinary(
        "FailureQ",
        "Tests whether an expression is a Failure object.",
        control_flow::eval_failure_q,
    ),
    BuiltinSpec::ordinary(
        "Success",
        "Represents a success object with associated details in the current first-pass subset.",
        control_flow::eval_success,
    ),
    BuiltinSpec::ordinary(
        "TerminatedEvaluation",
        "Represents a terminated evaluation object in the current first-pass subset.",
        control_flow::eval_terminated_evaluation,
    ),
];

pub(crate) const FUNCTIONAL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "BlockMap",
        "Maps a function over contiguous blocks in the current first-pass subset.",
        functional_ops::eval_block_map,
    ),
    BuiltinSpec::ordinary(
        "SubsetMap",
        "Replaces selected exact parts using a mapped subset result in the current first-pass subset.",
        functional_ops::eval_subset_map,
    ),
    BuiltinSpec::ordinary(
        "Comap",
        "Applies selected parts of a function collection to a common expression in the current first-pass subset.",
        functional_ops::eval_comap,
    ),
    BuiltinSpec::ordinary(
        "ComapApply",
        "Applies selected parts of a function collection as heads of a common expression in the current first-pass subset.",
        functional_ops::eval_comap_apply,
    ),
    BuiltinSpec::ordinary(
        "Accumulate",
        "Returns running totals in the current first-pass subset.",
        functional_ops::eval_accumulate,
    ),
    BuiltinSpec::ordinary(
        "MapIndexed",
        "Applies a function to parts together with their positions in the current first-pass subset.",
        functional_ops::eval_map_indexed,
    ),
    BuiltinSpec::ordinary(
        "Map",
        "Applies a function over selected levels of an expression in the current first-pass subset.",
        functional_ops::eval_map,
    ),
    BuiltinSpec::ordinary(
        "MapAll",
        "Applies a function to every subexpression in the current first-pass subset.",
        functional_ops::eval_map_all,
    ),
    BuiltinSpec::ordinary(
        "MapAt",
        "Applies a function at exact part specifications in the current first-pass subset.",
        functional_ops::eval_map_at,
    ),
    BuiltinSpec::ordinary(
        "Scan",
        "Evaluates a function across selected levels for side effects in the current first-pass subset.",
        functional_ops::eval_scan,
    ),
    BuiltinSpec::ordinary(
        "MapThread",
        "Threads corresponding elements from multiple expressions in the current first-pass subset.",
        functional_ops::eval_map_thread,
    ),
    BuiltinSpec::ordinary(
        "TakeWhile",
        "Takes the longest prefix satisfying a criterion in the current first-pass subset.",
        functional_ops::eval_take_while,
    ),
    BuiltinSpec::ordinary(
        "LengthWhile",
        "Counts the longest prefix satisfying a criterion in the current first-pass subset.",
        functional_ops::eval_length_while,
    ),
    BuiltinSpec::ordinary(
        "Inner",
        "Combines matching tensor dimensions in the current first-pass subset.",
        functional_ops::eval_inner,
    ),
    BuiltinSpec::ordinary(
        "Dot",
        "Contracts list tensors over the last axis of each left operand and the first axis of each right operand.",
        functional_ops::eval_dot,
    ),
    BuiltinSpec::ordinary(
        "Fold",
        "Accumulates over elements in the current first-pass subset.",
        functional_ops::eval_fold,
    ),
    BuiltinSpec::ordinary(
        "FoldList",
        "Returns successive accumulator states in the current first-pass subset.",
        functional_ops::eval_fold_list,
    ),
    BuiltinSpec::ordinary(
        "FoldPair",
        "Accumulates pair-valued state transitions in the current first-pass subset.",
        functional_ops::eval_fold_pair,
    ),
    BuiltinSpec::ordinary(
        "FoldWhile",
        "Accumulates while a test stays True in the current first-pass subset.",
        functional_ops::eval_fold_while,
    ),
    BuiltinSpec::ordinary(
        "SequenceFold",
        "Folds a sequence with a sliding history in the current first-pass subset.",
        functional_ops::eval_sequence_fold,
    ),
    BuiltinSpec::ordinary(
        "SequenceFoldList",
        "Returns the running history of a sequence fold in the current first-pass subset.",
        functional_ops::eval_sequence_fold_list,
    ),
    BuiltinSpec::ordinary(
        "Nest",
        "Applies a function repeatedly a fixed number of times.",
        functional_ops::eval_nest,
    ),
    BuiltinSpec::ordinary(
        "NestWhile",
        "Repeatedly applies a function while a test stays True in the current first-pass subset.",
        functional_ops::eval_nest_while,
    ),
    BuiltinSpec::ordinary(
        "NestList",
        "Returns repeated applications of a function in the current first-pass subset.",
        functional_ops::eval_nest_list,
    ),
    BuiltinSpec::ordinary(
        "NestWhileList",
        "Returns repeated applications while a test stays True in the current first-pass subset.",
        functional_ops::eval_nest_while_list,
    ),
    BuiltinSpec::ordinary(
        "FixedPoint",
        "Repeatedly applies a function until the result stops changing in the current first-pass subset.",
        functional_ops::eval_fixed_point,
    ),
    BuiltinSpec::ordinary(
        "FixedPointList",
        "Returns successive repeated applications until a fixed point in the current first-pass subset.",
        functional_ops::eval_fixed_point_list,
    ),
    BuiltinSpec::ordinary(
        "Apply",
        "Replaces the head of an expression in the current first-pass subset.",
        functional_ops::eval_apply,
    ),
    BuiltinSpec::ordinary(
        "MapApply",
        "Applies head replacement one level down in the current first-pass subset.",
        functional_ops::eval_map_apply,
    ),
    BuiltinSpec::ordinary(
        "OperatorApplied",
        "Represents a curried operator wrapper in the current first-pass subset.",
        functional_ops::eval_operator_applied,
    ),
    BuiltinSpec::ordinary(
        "CurryApplied",
        "Represents a left-to-right curried operator wrapper in the current first-pass subset.",
        functional_ops::eval_curry_applied,
    ),
    BuiltinSpec::ordinary(
        "ReverseApplied",
        "Represents a wrapper that reverses current call arguments before application in the current first-pass subset.",
        functional_ops::eval_reverse_applied,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Composition",
        "Represents a left composition of functions in the current first-pass subset.",
        AttributeSet::empty()
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        functional_ops::eval_composition,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "RightComposition",
        "Represents a right composition of functions in the current first-pass subset.",
        AttributeSet::empty()
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        functional_ops::eval_right_composition,
    ),
    BuiltinSpec::ordinary(
        "Through",
        "Threads arguments through a functional head in the current first-pass subset.",
        functional_ops::eval_through,
    ),
    BuiltinSpec::ordinary(
        "Operate",
        "Applies an operator to heads in the head chain of an expression in the current first-pass subset.",
        functional_ops::eval_operate,
    ),
    BuiltinSpec::ordinary(
        "Array",
        "Constructs a dense list-like array by applying a function to generated indices in the current first-pass subset.",
        functional_ops::eval_array,
    ),
    BuiltinSpec::ordinary(
        "ArrayFilter",
        "Applies a function to local list-array neighborhoods in the current first-pass subset.",
        functional_ops::eval_array_filter,
    ),
    BuiltinSpec::ordinary(
        "ArrayReduce",
        "Reduces rectangular list arrays across selected dimensions in the current first-pass subset.",
        functional_ops::eval_array_reduce,
    ),
    BuiltinSpec::ordinary(
        "Outer",
        "Forms outer combinations of same-head expressions in the current dense-array subset.",
        functional_ops::eval_outer,
    ),
    BuiltinSpec::ordinary(
        "Select",
        "Selects elements satisfying a criterion in the current sequence subset.",
        functional_ops::eval_select,
    ),
    BuiltinSpec::ordinary(
        "SelectFirst",
        "Returns the first element satisfying a criterion in the current sequence subset.",
        functional_ops::eval_select_first,
    ),
];

pub(crate) const REWRITE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::special(
        "MatchQ",
        "Tests whether an expression matches a pattern.",
        rewrite_builtins::eval_match_q,
    ),
    BuiltinSpec::special(
        "Cases",
        "Collects subexpressions matching a pattern or rule.",
        rewrite_builtins::eval_cases,
    ),
    BuiltinSpec::special(
        "DeleteCases",
        "Deletes subexpressions matching a pattern on levels specified by a standard levelspec.",
        rewrite_builtins::eval_delete_cases,
    ),
    BuiltinSpec::special(
        "Replace",
        "Applies rules at the current first-pass Replace levelspec subset.",
        rewrite_builtins::eval_replace,
    ),
    BuiltinSpec::special(
        "ReplaceList",
        "Returns the current first-pass root replacement results across a rule list, with optional max-result limiting.",
        rewrite_builtins::eval_replace_list,
    ),
    BuiltinSpec::special(
        "ReplaceAll",
        "Applies rules everywhere in an expression.",
        rewrite_builtins::eval_replace_all,
    ),
    BuiltinSpec::special(
        "ReplaceRepeated",
        "Repeatedly applies rules until an expression reaches a fixed point.",
        rewrite_builtins::eval_replace_repeated,
    ),
];

pub(crate) const INTEGER_DIGITS_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "IntegerLength",
        "Returns the number of digits in an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        integer_digits::eval_integer_length,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerDigits",
        "Returns the digits of an exact integer in a given base.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        integer_digits::eval_integer_digits,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerReverse",
        "Reverses the digits of an exact integer in a given base.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        integer_digits::eval_integer_reverse,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FromDigits",
        "Constructs an exact integer from a list or string of digits.",
        AttributeSet::from_attribute(Attribute::Protected),
        integer_digits::eval_from_digits,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DigitCount",
        "Counts digits in the absolute value of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        integer_digits::eval_digit_count,
    ),
];

pub(crate) const NUMBER_THEORY_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "Divisors",
        "Returns the sorted positive divisors of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_divisors,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "EulerPhi",
        "Computes Euler's totient function for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_euler_phi,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "MoebiusMu",
        "Computes the Moebius mu function for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_moebius_mu,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DivisorSigma",
        "Computes exact divisor power sums for exact integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NHoldAll)
            .with(Attribute::Protected),
        number_theory::eval_divisor_sigma,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Prime",
        "Returns the n-th positive prime for a positive integer n.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_prime,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PrimePi",
        "Counts primes not greater than an exact integer or rational argument.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_prime_pi,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeNu",
        "Counts distinct prime factors of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_prime_nu,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeOmega",
        "Counts prime factors with multiplicity for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_prime_omega,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "NextPrime",
        "Returns the next or previous signed prime in Wolfram order.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_next_prime,
    ),
];

pub(crate) const CALCULUS_BUILTINS: &[BuiltinSpec] = &[BuiltinSpec::ordinary_with_hold(
    "D",
    "Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.",
    HoldAttribute::HoldRest,
    calculus::eval_d,
)];

pub(crate) const POLYNOMIAL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Expand",
        "Expands rational-coefficient polynomial expressions over pattern-free indeterminates.",
        polynomials::eval_expand,
    ),
    BuiltinSpec::ordinary(
        "Coefficient",
        "Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.",
        polynomials::eval_coefficient,
    ),
    BuiltinSpec::ordinary(
        "CoefficientList",
        "Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.",
        polynomials::eval_coefficient_list,
    ),
    BuiltinSpec::ordinary(
        "Together",
        "Combines rational-function terms over a common polynomial denominator.",
        polynomials::eval_together,
    ),
    BuiltinSpec::ordinary(
        "Factor",
        "Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.",
        polynomials::eval_factor,
    ),
];

pub(crate) const STRING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "StringJoin",
        "Concatenates strings or a list of strings into a single string.",
        strings::eval_string_join,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StringLength",
        "Returns the Unicode code-point length of a string.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_string_length,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StringReverse",
        "Reverses the Unicode code-point characters in a string.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_string_reverse,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToLowerCase",
        "Converts a string to lowercase using the current Unicode-aware subset.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_to_lower_case,
    ),
    BuiltinSpec::ordinary(
        "ToUpperCase",
        "Converts a string or list of strings to uppercase using the current Unicode-aware subset.",
        strings::eval_to_upper_case,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Capitalize",
        "Uppercases the first character of a string or each string in a list.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        strings::eval_capitalize,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Decapitalize",
        "Lowercases the first character of a string or each string in a list.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        strings::eval_decapitalize,
    ),
    BuiltinSpec::ordinary(
        "StringForm",
        "Formats a template string using the current simple `` placeholder subset.",
        strings::eval_string_form,
    ),
    BuiltinSpec::ordinary(
        "Row",
        "Renders a list of expressions as a single text string in the current harness subset.",
        strings::eval_row,
    ),
    BuiltinSpec::ordinary(
        "StringRiffle",
        "Riffles strings together using the current flat and nested-list separator subset.",
        strings::eval_string_riffle,
    ),
    BuiltinSpec::ordinary(
        "StringPadLeft",
        "Pads strings on the left to a target width using the current simple padding subset.",
        strings::eval_string_pad_left,
    ),
    BuiltinSpec::ordinary(
        "StringTemplate",
        "Builds a simple TemplateObject wrapper from a template string in the current first-pass subset.",
        strings::eval_string_template,
    ),
    BuiltinSpec::ordinary(
        "TemplateApply",
        "Applies a simple named or positional string template in the current first-pass subset.",
        strings::eval_template_apply,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "TextWords",
        "Splits text into alphanumeric word runs in the current first-pass subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_text_words,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToCharacterCode",
        "Returns Unicode scalar values for strings in the current first-pass subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_to_character_code,
    ),
    BuiltinSpec::ordinary(
        "ToExpression",
        "Parses and evaluates input strings using the current InputForm/StandardForm-compatible subset.",
        strings::eval_to_expression,
    ),
    BuiltinSpec::ordinary(
        "StringToByteArray",
        "Converts strings to ByteArray objects using the current UTF-8 subset.",
        strings::eval_string_to_byte_array,
    ),
    BuiltinSpec::ordinary(
        "Hash",
        "Computes SHA256 hashes over the current string/expression subset.",
        strings::eval_hash,
    ),
    BuiltinSpec::ordinary(
        "Compress",
        "Encodes expressions into the current project-local compressed string envelope.",
        strings::eval_compress,
    ),
    BuiltinSpec::ordinary(
        "Uncompress",
        "Recovers values from the current project-local compressed string envelope.",
        strings::eval_uncompress,
    ),
    BuiltinSpec::ordinary(
        "Encrypt",
        "Wraps values in the current first-pass encrypted object subset using string keys.",
        strings::eval_encrypt,
    ),
    BuiltinSpec::ordinary(
        "Decrypt",
        "Decrypts the current first-pass encrypted object subset using string or ByteArray keys.",
        strings::eval_decrypt,
    ),
    BuiltinSpec::ordinary(
        "Nearest",
        "Finds nearest strings or exact numeric values in the current first-pass subset.",
        strings::eval_nearest,
    ),
    BuiltinSpec::ordinary(
        "Pluralize",
        "Applies a simple English pluralization subset.",
        strings::eval_pluralize,
    ),
    BuiltinSpec::ordinary(
        "SyntaxQ",
        "Tests whether a string parses successfully in the current syntax subset.",
        strings::eval_syntax_q,
    ),
    BuiltinSpec::ordinary(
        "TextString",
        "Converts expressions to the current canonical text subset, leaving raw strings unquoted.",
        strings::eval_text_string,
    ),
    BuiltinSpec::ordinary(
        "FileTemplateApply",
        "Applies a simple file-backed string template and writes the rendered result to a temporary file in the current first-pass subset.",
        strings::eval_file_template_apply,
    ).with_capability(Capability::FileWrite),
    BuiltinSpec::ordinary(
        "CharacterCounts",
        "Counts code-point occurrences in a string and returns an association in the current first-pass subset.",
        strings::eval_character_counts,
    ),
    BuiltinSpec::ordinary(
        "RemoveDiacritics",
        "Removes common Latin diacritics from strings in the current first-pass subset.",
        strings::eval_remove_diacritics,
    ),
    BuiltinSpec::ordinary(
        "Alphabet",
        "Returns the current English lowercase alphabet subset.",
        strings::eval_alphabet,
    ),
    BuiltinSpec::ordinary(
        "AlphabeticSort",
        "Sorts strings using the current English alphabet subset with simple diacritic folding.",
        strings::eval_alphabetic_sort,
    ),
    BuiltinSpec::ordinary(
        "EditDistance",
        "Computes unit-cost Levenshtein distance for strings or lists in the current first-pass subset.",
        strings::eval_edit_distance,
    ),
    BuiltinSpec::ordinary(
        "HammingDistance",
        "Computes mismatch counts for equal-length strings or lists in the current first-pass subset.",
        strings::eval_hamming_distance,
    ),
    BuiltinSpec::ordinary(
        "LongestCommonSubsequence",
        "Finds the longest common contiguous substring or sublist in the current first-pass subset.",
        strings::eval_longest_common_subsequence,
    ),
    BuiltinSpec::ordinary(
        "SequenceAlignment",
        "Builds a simple global alignment summary for strings or lists in the current first-pass subset.",
        strings::eval_sequence_alignment,
    ),
    BuiltinSpec::ordinary(
        "Diff",
        "Builds a simple DiffObject for strings or lists in the current first-pass subset.",
        strings::eval_diff,
    ),
    BuiltinSpec::ordinary(
        "Characters",
        "Splits a string into a list of single-code-point strings.",
        strings::eval_characters,
    ),
    BuiltinSpec::ordinary(
        "CharacterRange",
        "Builds an inclusive range of single-character strings between two endpoints.",
        strings::eval_character_range,
    ),
    BuiltinSpec::ordinary(
        "ToString",
        "Renders an evaluated expression into the current canonical kernel text form.",
        strings::eval_to_string,
    ),
    BuiltinSpec::ordinary(
        "StringRepeat",
        "Repeats a string a non-negative number of times.",
        strings::eval_string_repeat,
    ),
    BuiltinSpec::ordinary(
        "LetterQ",
        "Tests whether all characters in a string belong to the current letter subset.",
        strings::eval_letter_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LowerCaseQ",
        "Tests whether every character in a string is lowercase.",
        AttributeSet::from_attribute(Attribute::Protected),
        strings::eval_lower_case_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "UpperCaseQ",
        "Tests whether every character in a string is uppercase.",
        AttributeSet::from_attribute(Attribute::Protected),
        strings::eval_upper_case_q,
    ),
    BuiltinSpec::ordinary(
        "DigitQ",
        "Tests whether all characters in a string belong to the current digit subset.",
        strings::eval_digit_q,
    ),
    BuiltinSpec::ordinary(
        "StringContainsQ",
        "Tests whether a string or list of strings contains an exact substring in the current first-pass string subset.",
        strings::eval_string_contains_q,
    ),
    BuiltinSpec::ordinary(
        "StringStartsQ",
        "Tests whether a string or list of strings starts with an exact substring in the current first-pass string subset.",
        strings::eval_string_starts_q,
    ),
    BuiltinSpec::ordinary(
        "StringEndsQ",
        "Tests whether a string or list of strings ends with an exact substring in the current first-pass string subset.",
        strings::eval_string_ends_q,
    ),
    BuiltinSpec::ordinary(
        "StringPosition",
        "Returns 1-based inclusive positions of exact substring matches, including overlaps, in the current first-pass string subset.",
        strings::eval_string_position,
    ),
    BuiltinSpec::ordinary(
        "StringSplit",
        "Splits strings using whitespace or exact delimiters in the current first-pass string subset.",
        strings::eval_string_split,
    ),
    BuiltinSpec::ordinary(
        "StringTrim",
        "Trims whitespace or exact boundary substrings in the current first-pass string subset.",
        strings::eval_string_trim,
    ),
    BuiltinSpec::ordinary(
        "StringReplace",
        "Replaces exact string needles using immediate string rules in the current first-pass string subset.",
        strings::eval_string_replace,
    ),
    BuiltinSpec::ordinary(
        "StringCases",
        "Extracts non-overlapping exact string matches or immediate exact-string rule outputs in the current first-pass string subset.",
        strings::eval_string_cases,
    ),
    BuiltinSpec::ordinary(
        "StringApply",
        "Applies a list-oriented function to string code points and reassembles string output in the current first-pass subset.",
        strings::eval_string_apply,
    ),
    BuiltinSpec::ordinary(
        "StringDelete",
        "Deletes exact or current string-pattern matches from strings in the current first-pass subset.",
        strings::eval_string_delete,
    ),
    BuiltinSpec::ordinary(
        "StringTake",
        "Extracts a substring using integer counts or inclusive 1-based ranges.",
        strings::eval_string_take,
    ),
    BuiltinSpec::ordinary(
        "StringDrop",
        "Drops a substring using integer counts or inclusive 1-based ranges.",
        strings::eval_string_drop,
    ),
    BuiltinSpec::ordinary(
        "StringTakeDrop",
        "Returns both the taken and dropped substrings using the current StringTake/StringDrop spec subset.",
        strings::eval_string_take_drop,
    ),
    BuiltinSpec::ordinary(
        "StringPart",
        "Extracts individual characters by 1-based positions.",
        strings::eval_string_part,
    ),
    BuiltinSpec::ordinary(
        "StringCount",
        "Counts exact or current string-pattern matches, including the current IgnoreCase and Overlaps option subset.",
        strings::eval_string_count,
    ),
    BuiltinSpec::ordinary(
        "StringMatchQ",
        "Tests whether a whole string matches an exact or current string-pattern spec.",
        strings::eval_string_match_q,
    ),
    BuiltinSpec::ordinary(
        "StringFreeQ",
        "Tests whether a string is free of exact or current string-pattern matches.",
        strings::eval_string_free_q,
    )
];
