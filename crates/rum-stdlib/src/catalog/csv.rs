use rum_runtime::kernel_api::BuiltinSpec;

use crate::csv;

pub(crate) const CSV_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FromCsv",
        "Parses CSV text into Lists of Strings, optionally using the first row as Association headers.",
        csv::eval_from_csv,
    ),
    BuiltinSpec::ordinary(
        "ToCsv",
        "Renders Lists of String rows or String-keyed Association rows as CSV text.",
        csv::eval_to_csv,
    ),
];
