use super::*;

const SET_OPERATION_ATTRIBUTES: AttributeSet = AttributeSet::empty()
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

pub(super) const STRUCTURE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FullForm",
        "Renders the held argument as a canonical structural-form string in the current core expression subset.",
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
    BuiltinSpec::ordinary_with_attributes(
        "LeafCount",
        "Counts atomic leaves in an expression's full structural tree, including heads.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::LeafCount,
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
        "Range",
        "Constructs a plain list of exact integers from the current one-, two-, and three-argument subset.",
        BuiltinHandler::Range,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Differences",
        "Returns successive differences for plain lists, including the current non-negative order subset.",
        AttributeSet::empty()
            .with(Attribute::NonThreadable)
            .with(Attribute::Protected),
        BuiltinHandler::Differences,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ConstantArray",
        "Constructs a constant plain list for non-negative integer scalar and list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::ConstantArray,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PadLeft",
        "Pads or truncates a normal expression on the left using the current length and padding-element subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::PadLeft,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PadRight",
        "Pads or truncates a normal expression on the right using the current length and padding-element subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::PadRight,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Subsets",
        "Returns subsets of a normal expression in Wolfram-compatible size-major order for the current size-spec subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::Subsets,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Tuples",
        "Returns repeated tuples or cartesian products for normal expressions in the current subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::Tuples,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Partition",
        "Partitions a normal expression into fixed-size blocks using the current positive offset subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::Partition,
    ),
    BuiltinSpec::ordinary(
        "Flatten",
        "Flattens nested expressions through the current non-negative level, single-head-spec, and level-list subset.",
        BuiltinHandler::Flatten,
    ),
    BuiltinSpec::ordinary(
        "Join",
        "Concatenates same-head normal expressions and associations in the current top-level subset.",
        BuiltinHandler::Join,
    ),
    BuiltinSpec::ordinary(
        "Reverse",
        "Reverses normal-expression arguments at the current positive level subset.",
        BuiltinHandler::Reverse,
    ),
    BuiltinSpec::ordinary(
        "Transpose",
        "Transposes rectangular plain-list matrices in the current first-pass subset.",
        BuiltinHandler::Transpose,
    ),
    BuiltinSpec::ordinary(
        "DeleteDuplicates",
        "Keeps the first occurrence of structurally equal top-level elements.",
        BuiltinHandler::DeleteDuplicates,
    ),
    BuiltinSpec::ordinary(
        "DeleteDuplicatesBy",
        "Keeps the first top-level element for each evaluated key in the current unary-function subset.",
        BuiltinHandler::DeleteDuplicatesBy,
    ),
    BuiltinSpec::ordinary(
        "Sort",
        "Sorts top-level elements using the current structural ordering subset.",
        BuiltinHandler::Sort,
    ),
    BuiltinSpec::ordinary(
        "SortBy",
        "Sorts top-level elements by an evaluated key in the current unary-function subset.",
        BuiltinHandler::SortBy,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Union",
        "Returns sorted unique elements across same-head normal expressions.",
        SET_OPERATION_ATTRIBUTES,
        BuiltinHandler::Union,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Intersection",
        "Returns sorted unique elements common to all same-head normal expressions.",
        SET_OPERATION_ATTRIBUTES,
        BuiltinHandler::Intersection,
    ),
    BuiltinSpec::ordinary(
        "Complement",
        "Returns sorted unique elements from the first expression that are absent from later same-head expressions.",
        BuiltinHandler::Complement,
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
];
