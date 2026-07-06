use super::*;

fn system_rule_option(name: &str, value: Expr) -> SymbolOptionEntry {
    SymbolOptionEntry {
        name: Symbol::system(name),
        value,
        delayed: false,
    }
}

fn bool_option(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}

pub(in crate::runtime) fn default_system_options(symbol: &Symbol) -> SymbolOptions {
    if symbol.context() != SYSTEM_CONTEXT {
        return SymbolOptions::default();
    }

    let entries = match symbol.name() {
        "ArrayFilter" => vec![system_rule_option("Padding", Expr::string("Fixed"))],
        "Cases" | "DeleteCases" | "Count" | "MemberQ" | "Level" | "Depth" => {
            vec![system_rule_option("Heads", bool_option(false))]
        }
        "Trace" | "TracePrint" => vec![system_rule_option("Heads", bool_option(false))],
        "FreeQ" | "Position" => vec![system_rule_option("Heads", bool_option(true))],
        "CheckAbort" => vec![system_rule_option(
            "PropagateAborts",
            system_expr("Automatic"),
        )],
        "Monitor" => vec![
            system_rule_option("EvaluationMonitor", system_expr("None")),
            system_rule_option("StepMonitor", system_expr("None")),
        ],
        "Dialog" => vec![
            system_rule_option("DialogProlog", system_expr("Null")),
            system_rule_option("DialogSymbols", Expr::call(system_expr("List"), Vec::new())),
        ],
        "Input" | "InputString" => vec![system_rule_option("FieldMasked", system_expr("False"))],
        "TestReport" => vec![
            system_rule_option("SameTest", system_expr("SameQ")),
            system_rule_option("TimeConstraint", system_expr("Infinity")),
            system_rule_option("MemoryConstraint", system_expr("Infinity")),
        ],
        "Save" => vec![
            system_rule_option("ExcludedContexts", system_expr("Automatic")),
            system_rule_option("IncludedContexts", system_expr("All")),
        ],
        "DumpSave" => vec![
            system_rule_option("CompressionLevel", Expr::integer(BigInt::from(1))),
            system_rule_option("SymbolAttributes", bool_option(true)),
        ],
        "Once" => vec![
            system_rule_option("ExpirationDate", system_expr("None")),
            system_rule_option("PersistenceTime", system_expr("None")),
        ],
        "Names" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("ResolveContextAliases", bool_option(true)),
            system_rule_option("SpellingCorrection", bool_option(false)),
        ],
        "Nearest" => vec![system_rule_option(
            "DistanceFunction",
            system_expr("Automatic"),
        )],
        "StringContainsQ" | "StringStartsQ" | "StringEndsQ" | "StringMatchQ" | "StringFreeQ"
        | "StringDelete" | "StringReplace" | "StringSplit" => {
            vec![system_rule_option("IgnoreCase", bool_option(false))]
        }
        "StringCases" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("Overlaps", bool_option(false)),
        ],
        "StringPosition" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("Overlaps", bool_option(true)),
        ],
        "StringCount" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("Overlaps", bool_option(false)),
        ],
        _ => Vec::new(),
    };

    SymbolOptions::from_entries(entries)
}
