use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::csv;

pub(crate) const CSV_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromCsv",
        "Parses CSV text into Lists of Strings, optionally using the first row as Association headers.",
        AttributeSet::empty().with(Attribute::Protected),
        csv::eval_from_csv,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToCsv",
        "Renders Lists of String rows or String-keyed Association rows as CSV text.",
        AttributeSet::empty().with(Attribute::Protected),
        csv::eval_to_csv,
    ),
];
