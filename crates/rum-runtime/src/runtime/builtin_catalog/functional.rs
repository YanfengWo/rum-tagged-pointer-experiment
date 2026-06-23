use super::*;

pub(super) const FUNCTIONAL_BUILTINS: &[BuiltinSpec] = &[
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
        "Dot",
        "Contracts list tensors over the last axis of each left operand and the first axis of each right operand.",
        BuiltinHandler::Dot,
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
        "Array",
        "Constructs a dense list-like array by applying a function to generated indices in the current first-pass subset.",
        BuiltinHandler::Array,
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
    BuiltinSpec::ordinary(
        "Outer",
        "Forms outer combinations of same-head expressions in the current dense-array subset.",
        BuiltinHandler::Outer,
    ),
    BuiltinSpec::ordinary(
        "Select",
        "Selects elements satisfying a criterion in the current sequence subset.",
        BuiltinHandler::Select,
    ),
    BuiltinSpec::ordinary(
        "SelectFirst",
        "Returns the first element satisfying a criterion in the current sequence subset.",
        BuiltinHandler::SelectFirst,
    ),
];
