use super::*;

pub(super) const STRUCTURE_BUILTINS: &[BuiltinSpec] = &[
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
        "Flatten",
        "Flattens same-head nested expressions through the current non-negative level subset.",
        BuiltinHandler::Flatten,
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
