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
        Session::eval_full_form,
    ),
    BuiltinSpec::ordinary(
        "Head",
        "Returns the structural head of an expression, optionally wrapped in a second head.",
        Session::eval_head,
    ),
    BuiltinSpec::ordinary(
        "Length",
        "Returns the number of structural elements in an expression.",
        Session::eval_length,
    ),
    BuiltinSpec::ordinary(
        "Depth",
        "Returns the structural depth of an expression, optionally including heads.",
        Session::eval_depth,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LeafCount",
        "Counts atomic leaves in an expression's full structural tree, including heads.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_leaf_count,
    ),
    BuiltinSpec::ordinary_with_hold(
        "First",
        "Returns the first structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        Session::eval_first,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Last",
        "Returns the last structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        Session::eval_last,
    ),
    BuiltinSpec::ordinary(
        "Rest",
        "Returns an expression with its first structural element removed.",
        Session::eval_rest,
    ),
    BuiltinSpec::ordinary(
        "Most",
        "Returns an expression with its last structural element removed.",
        Session::eval_most,
    ),
    BuiltinSpec::ordinary(
        "Take",
        "Takes structural elements using the current standard sequence-spec subset.",
        Session::eval_take,
    ),
    BuiltinSpec::ordinary(
        "Drop",
        "Drops structural elements using the current standard sequence-spec subset.",
        Session::eval_drop,
    ),
    BuiltinSpec::ordinary(
        "TakeDrop",
        "Returns both the taken and dropped structural elements using the current standard sequence-spec subset.",
        Session::eval_take_drop,
    ),
    BuiltinSpec::ordinary(
        "Insert",
        "Inserts an element at a top-level structural position in the current first-pass subset.",
        Session::eval_insert,
    ),
    BuiltinSpec::ordinary(
        "Delete",
        "Deletes one or more top-level structural positions in the current first-pass subset.",
        Session::eval_delete,
    ),
    BuiltinSpec::ordinary(
        "TakeList",
        "Splits a normal expression into sequential chunks by non-negative counts.",
        Session::eval_take_list,
    ),
    BuiltinSpec::ordinary(
        "Range",
        "Constructs a plain list of exact integers from the current one-, two-, and three-argument subset.",
        Session::eval_range,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Differences",
        "Returns successive differences for plain lists, including the current non-negative order subset.",
        AttributeSet::empty()
            .with(Attribute::NonThreadable)
            .with(Attribute::Protected),
        Session::eval_differences,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ConstantArray",
        "Constructs a constant plain list for non-negative integer scalar and list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_constant_array,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PadLeft",
        "Pads or truncates a normal expression on the left using the current length and padding-element subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_pad_left,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PadRight",
        "Pads or truncates a normal expression on the right using the current length and padding-element subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_pad_right,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Subsets",
        "Returns subsets of a normal expression in Wolfram-compatible size-major order for the current size-spec subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_subsets,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Tuples",
        "Returns repeated tuples or cartesian products for normal expressions in the current subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_tuples,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Partition",
        "Partitions a normal expression into fixed-size blocks using the current positive offset subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_partition,
    ),
    BuiltinSpec::ordinary(
        "Flatten",
        "Flattens nested expressions through the current non-negative level, single-head-spec, and level-list subset.",
        Session::eval_flatten,
    ),
    BuiltinSpec::ordinary(
        "Join",
        "Concatenates same-head normal expressions and associations in the current top-level subset.",
        Session::eval_join,
    ),
    BuiltinSpec::ordinary(
        "Reverse",
        "Reverses normal-expression arguments at the current positive level subset.",
        Session::eval_reverse,
    ),
    BuiltinSpec::ordinary(
        "Transpose",
        "Transposes rectangular plain-list matrices in the current first-pass subset.",
        Session::eval_transpose,
    ),
    BuiltinSpec::ordinary(
        "DeleteDuplicates",
        "Keeps the first occurrence of structurally equal top-level elements.",
        Session::eval_delete_duplicates,
    ),
    BuiltinSpec::ordinary(
        "DeleteDuplicatesBy",
        "Keeps the first top-level element for each evaluated key in the current unary-function subset.",
        Session::eval_delete_duplicates_by,
    ),
    BuiltinSpec::ordinary(
        "Sort",
        "Sorts top-level elements using the current structural ordering subset.",
        Session::eval_sort,
    ),
    BuiltinSpec::ordinary(
        "SortBy",
        "Sorts top-level elements by an evaluated key in the current unary-function subset.",
        Session::eval_sort_by,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Union",
        "Returns sorted unique elements across same-head normal expressions.",
        SET_OPERATION_ATTRIBUTES,
        Session::eval_union,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Intersection",
        "Returns sorted unique elements common to all same-head normal expressions.",
        SET_OPERATION_ATTRIBUTES,
        Session::eval_intersection,
    ),
    BuiltinSpec::ordinary(
        "Complement",
        "Returns sorted unique elements from the first expression that are absent from later same-head expressions.",
        Session::eval_complement,
    ),
    BuiltinSpec::ordinary(
        "Append",
        "Appends an element to the end of a normal expression, with current operator-form support.",
        Session::eval_append,
    ),
    BuiltinSpec::ordinary(
        "Prepend",
        "Prepends an element to the front of a normal expression, with current operator-form support.",
        Session::eval_prepend,
    ),
    BuiltinSpec::ordinary(
        "Symbol",
        "Resolves a symbol from a string name using the current context path.",
        Session::eval_symbol,
    ),
    BuiltinSpec::ordinary(
        "Construct",
        "Constructs an expression from a head and zero or more arguments.",
        Session::eval_construct,
    ),
];
