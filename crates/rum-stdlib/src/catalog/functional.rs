use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::functional_ops;

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
