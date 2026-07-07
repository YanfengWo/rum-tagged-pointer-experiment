use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec, HoldAttribute};

use crate::structure_ops;

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
        "Pads or truncates a normal expression on the left using scalar lengths or rectangular list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_pad_left,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PadRight",
        "Pads or truncates a normal expression on the right using scalar lengths or rectangular list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_pad_right,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Subsets",
        "Returns subsets of a normal expression in rum's size-major order for scalar, All, Infinity, and range size specs.",
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
        "Partitions a flat normal expression into fixed-size blocks; aligned forms anchor the first element at kL and stop once the last element appears at or beyond kR.",
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
        "Reverses normal-expression arguments at positive levels or at every level with All.",
        structure_ops::eval_reverse,
    ),
    BuiltinSpec::ordinary(
        "Transpose",
        "Transposes rectangular plain-list matrices and tensors using identity, first-two-swap, or rank-matching positive permutations.",
        structure_ops::eval_transpose,
    ),
    BuiltinSpec::ordinary(
        "DeleteDuplicates",
        "Keeps the first occurrence of structurally equal top-level elements, or of elements equivalent under a custom binary test.",
        structure_ops::eval_delete_duplicates,
    ),
    BuiltinSpec::ordinary(
        "DeleteDuplicatesBy",
        "Keeps the first top-level element for each evaluated key in the current unary-function subset.",
        structure_ops::eval_delete_duplicates_by,
    ),
    BuiltinSpec::ordinary(
        "Duplicates",
        "Returns structurally equal top-level List elements that occur two or more times, once each in first-appearance order.",
        structure_ops::eval_duplicates,
    ),
    BuiltinSpec::ordinary(
        "Tally",
        "Tallies structurally same top-level elements, or elements equivalent under a custom binary test, with the first element as representative.",
        structure_ops::eval_tally,
    ),
    BuiltinSpec::ordinary(
        "GatherBy",
        "Groups top-level elements by evaluated keys in first-occurrence key order, including hierarchical grouping for a list of functions.",
        structure_ops::eval_gather_by,
    ),
    BuiltinSpec::ordinary(
        "Sort",
        "Sorts top-level elements using the current structural ordering subset.",
        structure_ops::eval_sort,
    ),
    BuiltinSpec::ordinary(
        "SortBy",
        "Sorts top-level elements by an evaluated key, optionally ordering keys with a binary predicate and preserving input order for predicate ties.",
        structure_ops::eval_sort_by,
    ),
    BuiltinSpec::ordinary(
        "BinarySearch",
        "Binary-searches an ascending List under rum's canonical order without validating sortedness, returning a 1-based position or Missing[\"NotFound\"].",
        structure_ops::eval_binary_search,
    ),
    BuiltinSpec::ordinary(
        "ReverseSort",
        "Sorts top-level elements using structural ordering or a binary predicate, then reverses the result.",
        structure_ops::eval_reverse_sort,
    ),
    BuiltinSpec::ordinary(
        "ReverseSortBy",
        "Sorts top-level elements by an evaluated key, then reverses the result.",
        structure_ops::eval_reverse_sort_by,
    ),
    BuiltinSpec::ordinary(
        "MaxBy",
        "Returns the first top-level element attaining the maximum evaluated key, or the top n in decreasing key order with input-order ties.",
        structure_ops::eval_max_by,
    ),
    BuiltinSpec::ordinary(
        "MinBy",
        "Returns the first top-level element attaining the minimum evaluated key, or the bottom n in increasing key order with input-order ties.",
        structure_ops::eval_min_by,
    ),
    BuiltinSpec::ordinary(
        "TakeLargestBy",
        "Takes top-level elements with largest evaluated keys in descending key order, preserving tie order.",
        structure_ops::eval_take_largest_by,
    ),
    BuiltinSpec::ordinary(
        "TakeSmallestBy",
        "Takes top-level elements with smallest evaluated keys in ascending key order, preserving tie order.",
        structure_ops::eval_take_smallest_by,
    ),
    BuiltinSpec::ordinary(
        "Riffle",
        "Inserts one separator between top-level elements of a normal expression, with optional interval or position specs.",
        structure_ops::eval_riffle,
    ),
    BuiltinSpec::ordinary(
        "SelectPositions",
        "Returns one-element 1-based position paths for top-level List elements whose criterion evaluates to True, matching Position's path convention, with an optional first-n limit.",
        structure_ops::eval_select_positions,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Union",
        "Returns sorted unique elements across same-head normal expressions; SameTest keeps the first representative in structural scan order.",
        SET_OPERATION_ATTRIBUTES,
        structure_ops::eval_union,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Intersection",
        "Returns sorted unique elements common to all same-head normal expressions; SameTest keeps the first representative in structural scan order.",
        SET_OPERATION_ATTRIBUTES,
        structure_ops::eval_intersection,
    ),
    BuiltinSpec::ordinary(
        "Complement",
        "Returns sorted unique elements from the first expression that are absent from later same-head expressions; SameTest keeps the first representative in structural scan order.",
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
