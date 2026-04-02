use super::*;
mod matcher;
mod parts;
mod search;

#[derive(Clone, Debug, PartialEq, Eq)]
enum StringPattern {
    Literal(Vec<char>),
    Whitespace,
    NumberString,
    Except {
        excluded: Box<StringPattern>,
        inner: Option<Box<StringPattern>>,
    },
    StartOfString,
    EndOfString,
    Repeated {
        pattern: Box<StringPattern>,
        min: usize,
        max: Option<usize>,
    },
    Shortest(Box<StringPattern>),
    Longest(Box<StringPattern>),
    Sequence(Vec<StringPattern>),
    Alternatives(Vec<StringPattern>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StringBoundarySpec {
    Whitespace,
    Patterns(Vec<StringPattern>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringSplitLimit {
    Unlimited,
    MaxSubstrings(usize),
    All,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringPatternMatchConfig {
    ignore_case: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringDeleteConfig {
    ignore_case: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringCountOverlaps {
    NonOverlapping,
    Overlapping,
}

impl Default for StringCountOverlaps {
    fn default() -> Self {
        Self::NonOverlapping
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringCountConfig {
    ignore_case: bool,
    overlaps: StringCountOverlaps,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactStringReplacementRule {
    pattern: StringPattern,
    replacement: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactStringCaseRule {
    pattern: StringPattern,
    output: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExactStringCaseSpec {
    Patterns(Vec<StringPattern>),
    Rules(Vec<ExactStringCaseRule>),
}

fn parse_string_boundary_spec(expr: &Expr) -> Option<StringBoundarySpec> {
    Some(StringBoundarySpec::Patterns(parse_string_patterns(expr)?))
}

fn parse_string_split_limit(expr: &Expr) -> Option<StringSplitLimit> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(StringSplitLimit::All);
    }

    expr_as_non_negative_usize(expr).map(StringSplitLimit::MaxSubstrings)
}

fn parse_string_patterns(expr: &Expr) -> Option<Vec<StringPattern>> {
    if let Some(pattern) = parse_string_pattern(expr) {
        return Some(vec![pattern]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") || args.is_empty() {
        return None;
    }

    args.iter()
        .map(parse_string_pattern)
        .collect::<Option<Vec<_>>>()
}

fn parse_string_pattern(expr: &Expr) -> Option<StringPattern> {
    if let Some(chars) = expr_as_non_empty_char_vec(expr) {
        return Some(StringPattern::Literal(chars));
    }

    if is_system_symbol_name(expr, "Whitespace") {
        return Some(StringPattern::Whitespace);
    }

    if is_system_symbol_name(expr, "NumberString") {
        return Some(StringPattern::NumberString);
    }

    if is_system_symbol_name(expr, "Except") {
        return None;
    }

    if is_system_symbol_name(expr, "StartOfString") {
        return Some(StringPattern::StartOfString);
    }

    if is_system_symbol_name(expr, "EndOfString") {
        return Some(StringPattern::EndOfString);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "Except") && args.len() == 1 {
        return Some(StringPattern::Except {
            excluded: Box::new(parse_string_pattern(&args[0])?),
            inner: None,
        });
    }

    if is_system_symbol_name(head, "Except") && args.len() == 2 {
        return Some(StringPattern::Except {
            excluded: Box::new(parse_string_pattern(&args[0])?),
            inner: Some(Box::new(parse_string_pattern(&args[1])?)),
        });
    }

    if is_system_symbol_name(head, "Repeated") && (args.len() == 1 || args.len() == 2) {
        let (min, max) = if let Some(bounds) = args.get(1) {
            parse_string_repetition_bounds(bounds)?
        } else {
            (1, None)
        };
        return Some(StringPattern::Repeated {
            pattern: Box::new(parse_string_pattern(&args[0])?),
            min,
            max,
        });
    }

    if is_system_symbol_name(head, "RepeatedNull") && (args.len() == 1 || args.len() == 2) {
        let (min, max) = if let Some(bounds) = args.get(1) {
            parse_string_repetition_bounds(bounds)?
        } else {
            (0, None)
        };
        return Some(StringPattern::Repeated {
            pattern: Box::new(parse_string_pattern(&args[0])?),
            min,
            max,
        });
    }

    if is_system_symbol_name(head, "Shortest") && args.len() == 1 {
        return Some(StringPattern::Shortest(Box::new(parse_string_pattern(
            &args[0],
        )?)));
    }

    if is_system_symbol_name(head, "Longest") && args.len() == 1 {
        return Some(StringPattern::Longest(Box::new(parse_string_pattern(
            &args[0],
        )?)));
    }

    if is_system_symbol_name(head, "StringExpression") && !args.is_empty() {
        return args
            .iter()
            .map(parse_string_pattern)
            .collect::<Option<Vec<_>>>()
            .map(StringPattern::Sequence);
    }

    if is_system_symbol_name(head, "Alternatives") && !args.is_empty() {
        return args
            .iter()
            .map(parse_string_pattern)
            .collect::<Option<Vec<_>>>()
            .map(StringPattern::Alternatives);
    }

    None
}

fn parse_exact_string_replacement_rules(expr: &Expr) -> Option<Vec<ExactStringReplacementRule>> {
    if let Some(rule) = parse_exact_string_replacement_rule(expr) {
        return Some(vec![rule]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") || args.is_empty() {
        return None;
    }

    args.iter()
        .map(parse_exact_string_replacement_rule)
        .collect()
}

fn parse_exact_string_replacement_rule(expr: &Expr) -> Option<ExactStringReplacementRule> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return None;
    }

    Some(ExactStringReplacementRule {
        pattern: parse_string_pattern(&args[0])?,
        replacement: expr_as_string(&args[1])?.to_string(),
    })
}

fn parse_exact_string_case_spec(expr: &Expr) -> Option<ExactStringCaseSpec> {
    if let Some(patterns) = parse_string_patterns(expr) {
        return Some(ExactStringCaseSpec::Patterns(patterns));
    }
    if let Some(rule) = parse_exact_string_case_rule(expr) {
        return Some(ExactStringCaseSpec::Rules(vec![rule]));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") || args.is_empty() {
        return None;
    }

    if let Some(patterns) = args
        .iter()
        .map(parse_string_pattern)
        .collect::<Option<Vec<_>>>()
    {
        return Some(ExactStringCaseSpec::Patterns(patterns));
    }

    args.iter()
        .map(parse_exact_string_case_rule)
        .collect::<Option<Vec<_>>>()
        .map(ExactStringCaseSpec::Rules)
}

fn parse_exact_string_case_rule(expr: &Expr) -> Option<ExactStringCaseRule> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return None;
    }

    Some(ExactStringCaseRule {
        pattern: parse_string_pattern(&args[0])?,
        output: args[1].clone(),
    })
}

fn parse_string_delete_options(options: &[Expr]) -> Option<StringDeleteConfig> {
    let mut config = StringDeleteConfig::default();

    for option in options {
        let Expr::Call { head, args } = option else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }

        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }

        return None;
    }

    Some(config)
}

fn parse_string_match_config(options: &[Expr]) -> Option<StringPatternMatchConfig> {
    let mut config = StringPatternMatchConfig::default();

    for option in options {
        let Expr::Call { head, args } = option else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }

        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }

        return None;
    }

    Some(config)
}

fn parse_string_match_limit_and_options(
    extras: &[Expr],
) -> Option<(usize, StringPatternMatchConfig)> {
    let mut limit = usize::MAX;
    let mut config = StringPatternMatchConfig::default();

    for extra in extras {
        if let Some(value) = expr_as_non_negative_usize(extra) {
            if limit != usize::MAX {
                return None;
            }
            limit = value;
            continue;
        }

        let Expr::Call { head, args } = extra else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }
        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }
        return None;
    }

    Some((limit, config))
}

fn parse_string_position_args(
    extras: &[Expr],
) -> Option<(Option<usize>, StringPatternMatchConfig)> {
    let mut max_results = None;
    let mut config = StringPatternMatchConfig::default();

    for extra in extras {
        if let Some(value) = expr_as_non_negative_usize(extra) {
            if max_results.is_some() {
                return None;
            }
            max_results = Some(value);
            continue;
        }

        let Expr::Call { head, args } = extra else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }
        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }
        return None;
    }

    Some((max_results, config))
}

fn parse_string_count_options(options: &[Expr]) -> Option<StringCountConfig> {
    let mut config = StringCountConfig::default();

    for option in options {
        let Expr::Call { head, args } = option else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }

        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }

        if is_system_symbol_name(&args[0], "Overlaps") {
            config.overlaps = if is_system_symbol_name(&args[1], heads::structure::ALL)
                || matches!(truthiness_symbol(&args[1]), Some(true))
            {
                StringCountOverlaps::Overlapping
            } else if matches!(truthiness_symbol(&args[1]), Some(false)) {
                StringCountOverlaps::NonOverlapping
            } else {
                return None;
            };
            continue;
        }

        return None;
    }

    Some(config)
}

fn parse_string_repetition_bounds(expr: &Expr) -> Option<(usize, Option<usize>)> {
    let items = call_items_with_head(expr, &system_expr("List"))?;

    match items {
        [count] => {
            let count = expr_as_non_negative_usize(count)?;
            Some((count, Some(count)))
        }
        [min, max] => {
            let min = expr_as_non_negative_usize(min)?;
            if is_system_symbol_name(max, "Infinity") {
                return Some((min, None));
            }
            let max = expr_as_non_negative_usize(max)?;
            (min <= max).then_some((min, Some(max)))
        }
        _ => None,
    }
}

fn expr_as_non_empty_char_vec(expr: &Expr) -> Option<Vec<char>> {
    let chars = expr_as_string(expr)?.chars().collect::<Vec<_>>();
    (!chars.is_empty()).then_some(chars)
}

