use super::*;
mod matcher;
mod parts;
mod regex;
mod search;

pub(crate) use parts::{
    eval_string_drop, eval_string_part, eval_string_take, eval_string_take_drop,
};
pub(crate) use search::eval_string_apply;

use self::regex::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::strings) enum StringPattern {
    Literal(Vec<char>),
    AnyCharacter,
    AnySequence {
        min: usize,
    },
    Capture {
        symbol: Symbol,
        inner: Box<StringPattern>,
    },
    Whitespace,
    WhitespaceCharacter,
    LetterCharacter,
    DigitCharacter,
    WordCharacter,
    PunctuationCharacter,
    RegularExpression(RegexPattern),
    NumberString,
    Except {
        excluded: Box<StringPattern>,
        inner: Option<Box<StringPattern>>,
    },
    StartOfString,
    EndOfString,
    StartOfLine,
    EndOfLine,
    WordBoundary,
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
pub(in crate::strings) struct StringPatternMatchConfig {
    ignore_case: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringDeleteConfig {
    ignore_case: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringSearchOverlaps {
    NonOverlapping,
    Overlapping,
}

impl Default for StringSearchOverlaps {
    fn default() -> Self {
        Self::NonOverlapping
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringCasesConfig {
    match_config: StringPatternMatchConfig,
    overlaps: StringSearchOverlaps,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StringPositionConfig {
    match_config: StringPatternMatchConfig,
    overlaps: StringSearchOverlaps,
}

impl Default for StringPositionConfig {
    fn default() -> Self {
        Self {
            match_config: StringPatternMatchConfig::default(),
            overlaps: StringSearchOverlaps::Overlapping,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringCountConfig {
    ignore_case: bool,
    overlaps: StringSearchOverlaps,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactStringReplacementRule {
    pattern: StringPattern,
    replacement: ExactStringReplacementValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExactStringReplacementValue {
    Immediate(Expr),
    Delayed(Expr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactStringCaseRule {
    pattern: StringPattern,
    output: ExactStringCaseOutput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExactStringCaseOutput {
    Immediate(Expr),
    Delayed(Expr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExactStringCaseSpec {
    Patterns(Vec<StringPattern>),
    Rules(Vec<ExactStringCaseRule>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StringRepeatedBoundsError {
    symbol: &'static str,
    spec: Expr,
}

fn first_invalid_string_repeated_bounds(expr: &Expr) -> Option<StringRepeatedBoundsError> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if (is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        && args.len() == 2
    {
        return first_invalid_string_repeated_bounds(&args[0]);
    }

    for symbol in ["Repeated", "RepeatedNull"] {
        if !is_system_symbol_name(head, symbol) {
            continue;
        }

        return match args.as_slice() {
            [inner] => first_invalid_string_repeated_bounds(inner),
            [inner, spec] => {
                if parse_string_repetition_bounds(spec).is_none() {
                    Some(StringRepeatedBoundsError {
                        symbol,
                        spec: spec.clone(),
                    })
                } else {
                    first_invalid_string_repeated_bounds(inner)
                }
            }
            _ => None,
        };
    }

    first_invalid_string_repeated_bounds(head)
        .or_else(|| args.iter().find_map(first_invalid_string_repeated_bounds))
}

fn eval_string_pattern_with_bounds_guard(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    evaluator: fn(&mut Session, &Expr, &[Expr]) -> Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    if let Some(error) = args.get(1).and_then(first_invalid_string_repeated_bounds) {
        session.push_message(
            error.symbol,
            "range",
            format!(
                "{} is not a valid repetition bound specification; expected n, {{n}}, or {{min, max}} with 0 <= min <= max, where max may be Infinity.",
                error.spec
            ),
        );
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    evaluator(session, head, args)
}

macro_rules! guarded_string_pattern_evaluator {
    ($name:ident, $implementation:path) => {
        pub(crate) fn $name(
            session: &mut Session,
            head: &Expr,
            args: &[Expr],
        ) -> Result<Expr, EvalError> {
            eval_string_pattern_with_bounds_guard(session, head, args, $implementation)
        }
    };
}

guarded_string_pattern_evaluator!(eval_string_cases, search::eval_string_cases);
guarded_string_pattern_evaluator!(eval_string_contains_q, search::eval_string_contains_q);
guarded_string_pattern_evaluator!(eval_string_count, search::eval_string_count);
guarded_string_pattern_evaluator!(eval_string_delete, search::eval_string_delete);
guarded_string_pattern_evaluator!(eval_string_ends_q, search::eval_string_ends_q);
guarded_string_pattern_evaluator!(eval_string_free_q, search::eval_string_free_q);
guarded_string_pattern_evaluator!(eval_string_match_q, search::eval_string_match_q);
guarded_string_pattern_evaluator!(eval_string_position, search::eval_string_position);
guarded_string_pattern_evaluator!(eval_string_replace, search::eval_string_replace);
guarded_string_pattern_evaluator!(eval_string_split, search::eval_string_split);
guarded_string_pattern_evaluator!(eval_string_starts_q, search::eval_string_starts_q);
guarded_string_pattern_evaluator!(eval_string_trim, search::eval_string_trim);

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

pub(in crate::strings) fn parse_string_pattern(expr: &Expr) -> Option<StringPattern> {
    if let Some(value) = expr_as_string(expr).filter(|value| !value.is_empty()) {
        return Some(parse_string_literal_pattern(value));
    }

    if is_system_symbol_name(expr, "Blank") {
        return Some(StringPattern::AnyCharacter);
    }

    if is_system_symbol_name(expr, "BlankSequence") {
        return Some(StringPattern::AnySequence { min: 1 });
    }

    if is_system_symbol_name(expr, "BlankNullSequence") {
        return Some(StringPattern::AnySequence { min: 0 });
    }

    if is_system_symbol_name(expr, "Whitespace") {
        return Some(StringPattern::Whitespace);
    }

    if is_system_symbol_name(expr, "WhitespaceCharacter") {
        return Some(StringPattern::WhitespaceCharacter);
    }

    if is_system_symbol_name(expr, "LetterCharacter") {
        return Some(StringPattern::LetterCharacter);
    }

    if is_system_symbol_name(expr, "DigitCharacter") {
        return Some(StringPattern::DigitCharacter);
    }

    if is_system_symbol_name(expr, "WordCharacter") {
        return Some(StringPattern::WordCharacter);
    }

    if is_system_symbol_name(expr, "PunctuationCharacter") {
        return Some(StringPattern::PunctuationCharacter);
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

    if is_system_symbol_name(expr, "StartOfLine") {
        return Some(StringPattern::StartOfLine);
    }

    if is_system_symbol_name(expr, "EndOfLine") {
        return Some(StringPattern::EndOfLine);
    }

    if is_system_symbol_name(expr, "WordBoundary") {
        return Some(StringPattern::WordBoundary);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "Pattern") && args.len() == 2 {
        return Some(StringPattern::Capture {
            symbol: expr_as_symbol(&args[0])?.clone(),
            inner: Box::new(parse_string_pattern(&args[1])?),
        });
    }

    if is_system_symbol_name(head, "Blank") && args.is_empty() {
        return Some(StringPattern::AnyCharacter);
    }

    if is_system_symbol_name(head, "Blank") && args.len() == 1 {
        return parse_string_pattern(&args[0]);
    }

    if is_system_symbol_name(head, "BlankSequence") && args.is_empty() {
        return Some(StringPattern::AnySequence { min: 1 });
    }

    if is_system_symbol_name(head, "BlankSequence") && args.len() == 1 {
        return Some(StringPattern::Repeated {
            pattern: Box::new(parse_string_pattern(&args[0])?),
            min: 1,
            max: None,
        });
    }

    if is_system_symbol_name(head, "BlankNullSequence") && args.is_empty() {
        return Some(StringPattern::AnySequence { min: 0 });
    }

    if is_system_symbol_name(head, "BlankNullSequence") && args.len() == 1 {
        return Some(StringPattern::Repeated {
            pattern: Box::new(parse_string_pattern(&args[0])?),
            min: 0,
            max: None,
        });
    }

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

    if is_system_symbol_name(head, "RegularExpression") && args.len() == 1 {
        let pattern = expr_as_string(&args[0])?;
        return compile_string_regex_pattern(pattern).map(StringPattern::RegularExpression);
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

pub(in crate::strings) fn string_pattern_match_end(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Option<usize> {
    matcher::best_pattern_match_end(pattern, chars, start, config)
}

fn parse_string_literal_pattern(value: &str) -> StringPattern {
    if !value.chars().any(|ch| matches!(ch, '*' | '?')) {
        return StringPattern::Literal(value.chars().collect());
    }

    let mut parts = Vec::new();
    let mut literal = Vec::new();

    for ch in value.chars() {
        match ch {
            '*' => {
                if !literal.is_empty() {
                    parts.push(StringPattern::Literal(std::mem::take(&mut literal)));
                }
                parts.push(StringPattern::AnySequence { min: 0 });
            }
            '?' => {
                if !literal.is_empty() {
                    parts.push(StringPattern::Literal(std::mem::take(&mut literal)));
                }
                parts.push(StringPattern::AnyCharacter);
            }
            other => literal.push(other),
        }
    }

    if !literal.is_empty() {
        parts.push(StringPattern::Literal(literal));
    }

    match parts.as_slice() {
        [only] => only.clone(),
        _ => StringPattern::Sequence(parts),
    }
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

    if args.len() != 2 {
        return None;
    }

    let replacement = if is_system_symbol_name(head, "Rule") {
        ExactStringReplacementValue::Immediate(args[1].clone())
    } else if is_system_symbol_name(head, "RuleDelayed") {
        ExactStringReplacementValue::Delayed(args[1].clone())
    } else {
        return None;
    };

    Some(ExactStringReplacementRule {
        pattern: parse_string_pattern(&args[0])?,
        replacement,
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

    if args.len() != 2 {
        return None;
    }

    let output = if is_system_symbol_name(head, "Rule") {
        ExactStringCaseOutput::Immediate(args[1].clone())
    } else if is_system_symbol_name(head, "RuleDelayed") {
        ExactStringCaseOutput::Delayed(args[1].clone())
    } else {
        return None;
    };

    Some(ExactStringCaseRule {
        pattern: parse_string_pattern(&args[0])?,
        output,
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

fn parse_string_cases_limit_and_options(extras: &[Expr]) -> Option<(usize, StringCasesConfig)> {
    let mut limit = usize::MAX;
    let mut config = StringCasesConfig::default();

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
            config.match_config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }
        if is_system_symbol_name(&args[0], "Overlaps") {
            config.overlaps = parse_boolean_overlaps(&args[1])?;
            continue;
        }
        return None;
    }

    Some((limit, config))
}

fn parse_string_position_args(extras: &[Expr]) -> Option<(Option<usize>, StringPositionConfig)> {
    let mut max_results = None;
    let mut config = StringPositionConfig::default();

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
            config.match_config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }
        if is_system_symbol_name(&args[0], "Overlaps") {
            config.overlaps = parse_boolean_overlaps(&args[1])?;
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
                StringSearchOverlaps::Overlapping
            } else if matches!(truthiness_symbol(&args[1]), Some(false)) {
                StringSearchOverlaps::NonOverlapping
            } else {
                return None;
            };
            continue;
        }

        return None;
    }

    Some(config)
}

fn parse_boolean_overlaps(expr: &Expr) -> Option<StringSearchOverlaps> {
    if truthiness_symbol(expr)? {
        Some(StringSearchOverlaps::Overlapping)
    } else {
        Some(StringSearchOverlaps::NonOverlapping)
    }
}

fn next_string_match_index(
    index: usize,
    end: usize,
    haystack_len: usize,
    overlaps: StringSearchOverlaps,
) -> Option<usize> {
    if end == index {
        if index == haystack_len {
            None
        } else {
            Some(index + 1)
        }
    } else {
        Some(match overlaps {
            StringSearchOverlaps::NonOverlapping => end,
            StringSearchOverlaps::Overlapping => index + 1,
        })
    }
}

fn parse_string_repetition_bounds(expr: &Expr) -> Option<(usize, Option<usize>)> {
    if let Some(count) = expr_as_non_negative_usize(expr) {
        return Some((count, Some(count)));
    }

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
