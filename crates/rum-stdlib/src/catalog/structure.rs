use rum_runtime::kernel_api::{
    Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec, HoldAttribute,
};

use super::message_templates::*;

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
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Head",
        "Returns the structural head of an expression, optionally wrapped in a second head.",
        structure_ops::eval_head,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Length",
        "Returns the number of structural elements in an expression.",
        structure_ops::eval_length,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Depth",
        "Returns the structural depth of an expression, optionally including heads.",
        structure_ops::eval_depth,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid level specification or Heads option for Depth.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "LeafCount",
        "Counts atomic leaves in an expression's full structural tree, including heads.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_leaf_count,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_hold(
        "First",
        "Returns the first structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        structure_ops::eval_first,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nofirst",
            template: "`1` has zero length and no `2` element.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Last",
        "Returns the last structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        structure_ops::eval_last,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nolast",
            template: "`1` has zero length and no `2` element.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Rest",
        "Returns an expression with its first structural element removed.",
        structure_ops::eval_rest,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "norest",
            template: "Cannot take `1` of expression `2` with length zero.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Most",
        "Returns an expression with its last structural element removed.",
        structure_ops::eval_most,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "nomost",
            template: "Cannot take `1` of expression `2` with length zero.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Take",
        "Takes structural elements using the current standard sequence-spec subset.",
        structure_ops::eval_take,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "take",
            template: "Cannot apply the requested sequence specification in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Drop",
        "Drops structural elements using the current standard sequence-spec subset.",
        structure_ops::eval_drop,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "drop",
            template: "Cannot apply the requested sequence specification in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeDrop",
        "Returns both the taken and dropped structural elements using the current standard sequence-spec subset.",
        structure_ops::eval_take_drop,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "iseq",
            template: "Invalid sequence specification `1` for an expression of length `2`.",
        },
        BuiltinMessageSpec {
            tag: "take",
            template: "Cannot apply the requested sequence specification in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Insert",
        "Inserts an element at a top-level structural position in the current first-pass subset.",
        structure_ops::eval_insert,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "ins",
            template: "Cannot insert at position `1` in `2`",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Delete",
        "Deletes one or more top-level structural positions in the current first-pass subset.",
        structure_ops::eval_delete,
    )
    .with_messages(&[
        ARGX_EXACT,
        PARTW,
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeList",
        "Splits a normal expression into sequential chunks by non-negative counts.",
        structure_ops::eval_take_list,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Range",
        "Constructs a plain list of exact integers from the current one-, two-, and three-argument subset.",
        structure_ops::eval_range,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Differences",
        "Returns successive differences for plain lists, including the current non-negative order subset.",
        AttributeSet::empty()
            .with(Attribute::NonThreadable)
            .with(Attribute::Protected),
        structure_ops::eval_differences,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsmn",
            template: "Single or list of non-negative machine-sized integers expected at position 2 of `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ConstantArray",
        "Constructs a constant plain list for non-negative integer scalar and list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_constant_array,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "PadLeft",
        "Pads or truncates a normal expression on the left using scalar lengths or rectangular list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_pad_left,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsm",
            template: "Non-negative machine-sized integer or non-empty list of non-negative machine-sized integers expected at position 2 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "PadRight",
        "Pads or truncates a normal expression on the right using scalar lengths or rectangular list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_pad_right,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsm",
            template: "Non-negative machine-sized integer or non-empty list of non-negative machine-sized integers expected at position 2 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Subsets",
        "Returns subsets of a normal expression in rum's size-major order for scalar, All, Infinity, and range size specs.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_subsets,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nninfseq",
            template: "Position 2 of `1` must be All, Infinity, nmax, {nmin}, {nmin, nmax} or {nmin, nmax, dn}, where nmin is a non-negative integer, nmax is non-negative integer or Infinity and dn is a nonzero integer.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Tuples",
        "Returns repeated tuples or cartesian products for normal expressions in the current subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_tuples,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsmn",
            template: "Single or list of non-negative machine-sized integers expected at position 2 of `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position `1` in `2`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Partition",
        "Partitions a flat normal expression into fixed-size blocks; aligned forms anchor the first element at kL and stop once the last element appears at or beyond kR.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_partition,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "align",
            template: "Position 4 of `1` must be {kL, kR}, where kL and kR are positive machine-sized integers not greater than the block size `2`.",
        },
        ARGTU,
        ARGX_OPERATOR,
        BuiltinMessageSpec {
            tag: "ilsmp",
            template: "Positive machine-sized integer expected at position `1` of `2`.",
        },
        BuiltinMessageSpec {
            tag: "npart",
            template: "The expression `1` cannot be partitioned.",
        },
    ]),
    BuiltinSpec::ordinary(
        "Flatten",
        "Flattens nested expressions through the current non-negative level, single-head-spec, and level-list subset.",
        structure_ops::eval_flatten,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "fldep",
            template: "Level `1` exceeds the levels, `2`, which can be flattened together in `3`.",
        },
        BuiltinMessageSpec {
            tag: "flev",
            template: "The level argument `1` in position 2 of `2` should be a non-negative integer or Infinity.",
        },
        BuiltinMessageSpec {
            tag: "flpi",
            template: "Levels to be flattened together in `1` should be lists of positive integers.",
        },
        BuiltinMessageSpec {
            tag: "flrep",
            template: "Level `1` should not be repeated.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Join",
        "Concatenates same-head normal expressions and associations in the current top-level subset.",
        structure_ops::eval_join,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions 1 and `3` are expected to be the same.",
        },
        BuiltinMessageSpec {
            tag: "headsd",
            template: "Expression `1` at position `2` is expected to have head `3` for all expressions at level `4`.",
        },
        BuiltinMessageSpec {
            tag: "incpt",
            template: "Incompatible elements in `1` cannot be joined.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal1",
            template: "Expression `1` at position 1 is expected to have nonatomic subexpression at level `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Reverse",
        "Reverses normal-expression arguments at positive levels or at every level with All.",
        structure_ops::eval_reverse,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsmp",
            template: "All, a positive machine-sized integer, or a list of positive machine-sized integers expected at position 2 of `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Transpose",
        "Transposes rectangular plain-list matrices and tensors using identity, first-two-swap, or rank-matching positive permutations.",
        structure_ops::eval_transpose,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nmtx",
            template: "The first two levels of `1` cannot be transposed.",
        },
        BuiltinMessageSpec {
            tag: "permspec",
            template: "Invalid permutation specification found at position 2 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "DeleteDuplicates",
        "Keeps the first occurrence of structurally equal top-level elements, or of elements equivalent under a custom binary test.",
        structure_ops::eval_delete_duplicates,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "DeleteDuplicatesBy",
        "Keeps the first top-level element for each evaluated key in the current unary-function subset.",
        structure_ops::eval_delete_duplicates_by,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Duplicates",
        "Returns structurally equal top-level List elements that occur two or more times, once each in first-appearance order.",
        structure_ops::eval_duplicates,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Tally",
        "Tallies structurally same top-level elements, or elements equivalent under a custom binary test, with the first element as representative.",
        structure_ops::eval_tally,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "GatherBy",
        "Groups top-level elements by evaluated keys in first-occurrence key order, including hierarchical grouping for a list of functions.",
        structure_ops::eval_gather_by,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Sort",
        "Sorts top-level elements using the current structural ordering subset.",
        structure_ops::eval_sort,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "SortBy",
        "Sorts top-level elements by an evaluated key, optionally ordering keys with a binary predicate and preserving input order for predicate ties.",
        structure_ops::eval_sort_by,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "BinarySearch",
        "Binary-searches an ascending List under rum's canonical order without validating sortedness, returning a 1-based position or Missing[\"NotFound\"].",
        structure_ops::eval_binary_search,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ReverseSort",
        "Sorts top-level elements using structural ordering or a binary predicate, then reverses the result.",
        structure_ops::eval_reverse_sort,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ReverseSortBy",
        "Sorts top-level elements by an evaluated key, then reverses the result.",
        structure_ops::eval_reverse_sort_by,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "MaxBy",
        "Returns the first top-level element attaining the maximum evaluated key, or the top n in decreasing key order with input-order ties.",
        structure_ops::eval_max_by,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "empty",
            template: "MaxBy cannot select from an empty list.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "MinBy",
        "Returns the first top-level element attaining the minimum evaluated key, or the bottom n in increasing key order with input-order ties.",
        structure_ops::eval_min_by,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "empty",
            template: "MinBy cannot select from an empty list.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeLargestBy",
        "Takes top-level elements with largest evaluated keys in descending key order, preserving tie order.",
        structure_ops::eval_take_largest_by,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeSmallestBy",
        "Takes top-level elements with smallest evaluated keys in ascending key order, preserving tie order.",
        structure_ops::eval_take_smallest_by,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Riffle",
        "Inserts one separator between top-level elements of a normal expression, with optional interval or position specs.",
        structure_ops::eval_riffle,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "seqs",
            template: "Position 3 of `1` must be a positive machine-sized integer or {imin, imax, di} with nonzero integer step.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "SelectPositions",
        "Returns one-element 1-based position paths for top-level List elements whose criterion evaluates to True, matching Position's path convention, with an optional first-n limit.",
        structure_ops::eval_select_positions,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer selection limit",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Union",
        "Returns sorted unique elements across same-head normal expressions; SameTest keeps the first representative in structural scan order.",
        SET_OPERATION_ATTRIBUTES,
        structure_ops::eval_union,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions `3` and 1 are expected to be the same.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Intersection",
        "Returns sorted unique elements common to all same-head normal expressions; SameTest keeps the first representative in structural scan order.",
        SET_OPERATION_ATTRIBUTES,
        structure_ops::eval_intersection,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argm",
            template: "Intersection called with 0 arguments; 1 or more arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions `3` and 1 are expected to be the same.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary(
        "Complement",
        "Returns sorted unique elements from the first expression that are absent from later same-head expressions; SameTest keeps the first representative in structural scan order.",
        structure_ops::eval_complement,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argm",
            template: "Complement called with 0 arguments; 1 or more arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions `3` and 1 are expected to be the same.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Append",
        "Appends an element to the end of a normal expression, with current operator-form support.",
        structure_ops::eval_append,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Prepend",
        "Prepends an element to the front of a normal expression, with current operator-form support.",
        structure_ops::eval_prepend,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Symbol",
        "Resolves a symbol from a string name using the current context path.",
        structure_ops::eval_symbol,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "string",
            template: "String expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Construct",
        "Constructs an expression from a head and zero or more arguments.",
        structure_ops::eval_construct,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
];
