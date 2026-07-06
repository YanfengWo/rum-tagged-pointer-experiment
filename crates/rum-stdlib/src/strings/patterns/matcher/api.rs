use super::engine::{
    best_pattern_match_at, best_pattern_match_at_with_bindings, best_pattern_match_end,
    best_pattern_match_state, pattern_can_match_through_end, pattern_match_text,
};
use super::split_trim::{
    split_by_string_patterns, split_by_whitespace, trim_string_patterns, trim_whitespace_chars,
};
use super::*;

pub(in crate::strings::patterns) fn string_contains_q(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
) -> bool {
    (0..=haystack.len())
        .any(|start| best_pattern_match_state(pattern, haystack, start, config).is_some())
}

pub(in crate::strings::patterns) fn string_starts_q(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
) -> bool {
    best_pattern_match_state(pattern, haystack, 0, config).is_some()
}

pub(in crate::strings::patterns) fn string_ends_q(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
) -> bool {
    (0..=haystack.len()).any(|start| {
        best_pattern_match_state(pattern, haystack, start, config)
            .is_some_and(|state| state.end == haystack.len())
    })
}

pub(in crate::strings::patterns) fn string_match_q(
    haystack: &[char],
    patterns: &[StringPattern],
    config: StringPatternMatchConfig,
) -> bool {
    patterns
        .iter()
        .any(|pattern| pattern_can_match_through_end(pattern, haystack, 0, haystack.len(), config))
}

pub(in crate::strings::patterns) fn string_free_q(
    haystack: &[char],
    patterns: &[StringPattern],
    config: StringPatternMatchConfig,
) -> bool {
    (0..=haystack.len()).all(|start| {
        best_pattern_match_at_with_bindings(patterns, haystack, start, config).is_none()
    })
}

pub(in crate::strings::patterns) fn string_count_matches(
    value: &str,
    patterns: &[StringPattern],
    config: StringCountConfig,
) -> usize {
    let chars = value.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut matches = 0usize;
    let match_config = StringPatternMatchConfig {
        ignore_case: config.ignore_case,
    };

    while index <= chars.len() {
        if let Some((_, state)) =
            best_pattern_match_at_with_bindings(patterns, &chars, index, match_config)
        {
            matches += 1;
            index = if state.end == index {
                if index == chars.len() {
                    break;
                }
                index + 1
            } else {
                match config.overlaps {
                    StringCountOverlaps::NonOverlapping => state.end,
                    StringCountOverlaps::Overlapping => index + 1,
                }
            };
        } else {
            if index == chars.len() {
                break;
            }
            index += 1;
        }
    }

    matches
}

pub(in crate::strings::patterns) fn string_delete_matches(
    value: &str,
    patterns: &[StringPattern],
    config: StringDeleteConfig,
) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    let match_config = StringPatternMatchConfig {
        ignore_case: config.ignore_case,
    };

    while index < chars.len() {
        if let Some((_, state)) =
            best_pattern_match_at_with_bindings(patterns, &chars, index, match_config)
        {
            if state.end == index {
                output.push(chars[index]);
                index += 1;
            } else {
                index = state.end;
            }
        } else {
            output.push(chars[index]);
            index += 1;
        }
    }

    output
}

pub(in crate::strings::patterns) fn split_string_value(
    value: &str,
    spec: &StringBoundarySpec,
    limit: StringSplitLimit,
    config: StringPatternMatchConfig,
) -> Vec<String> {
    let chars = value.chars().collect::<Vec<_>>();
    match spec {
        StringBoundarySpec::Whitespace => split_by_whitespace(&chars, limit),
        StringBoundarySpec::Patterns(patterns) => {
            split_by_string_patterns(&chars, patterns, limit, config)
        }
    }
}

pub(in crate::strings::patterns) fn trim_string_value(
    value: &str,
    spec: &StringBoundarySpec,
) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    match spec {
        StringBoundarySpec::Whitespace => trim_whitespace_chars(&chars),
        StringBoundarySpec::Patterns(patterns) => trim_string_patterns(&chars, patterns),
    }
}

pub(in crate::strings::patterns) fn collect_string_cases(
    value: &str,
    spec: &ExactStringCaseSpec,
    max_matches: usize,
    config: StringPatternMatchConfig,
) -> Vec<Expr> {
    if max_matches == 0 {
        return Vec::new();
    }

    let chars = value.chars().collect::<Vec<_>>();
    let mut index = 0;
    let mut matches = Vec::new();

    while index <= chars.len() && matches.len() < max_matches {
        match spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, end)) =
                    best_pattern_match_at(patterns, &chars, index, config)
                {
                    matches.push(Expr::string(pattern_match_text(&chars[index..end])));
                    index = if end == index {
                        if index == chars.len() {
                            break;
                        }
                        index + 1
                    } else {
                        end
                    };
                } else {
                    if index == chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((rule, end)) = rules.iter().find_map(|rule| {
                    best_pattern_match_end(&rule.pattern, &chars, index, config)
                        .map(|end| (rule, end))
                }) {
                    let output = match &rule.output {
                        ExactStringCaseOutput::Immediate(expr) => expr.clone(),
                        ExactStringCaseOutput::Delayed(_) => {
                            Expr::string(pattern_match_text(&chars[index..end]))
                        }
                    };
                    matches.push(output);
                    index = if end == index {
                        if index == chars.len() {
                            break;
                        }
                        index + 1
                    } else {
                        end
                    };
                } else {
                    if index == chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
        }
    }

    matches
}

pub(in crate::strings::patterns) fn string_position_expr(
    value: &str,
    pattern: &StringPattern,
    max_results: Option<usize>,
    config: StringPatternMatchConfig,
) -> Expr {
    let haystack = value.chars().collect::<Vec<_>>();
    let positions = string_pattern_positions(&haystack, pattern, max_results, config)
        .map(|(start, end)| {
            list_expr(vec![
                Expr::integer(BigInt::from(start)),
                Expr::integer(BigInt::from(end)),
            ])
        })
        .collect();
    list_expr(positions)
}

fn string_pattern_positions<'a>(
    haystack: &'a [char],
    pattern: &'a StringPattern,
    max_results: Option<usize>,
    config: StringPatternMatchConfig,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    let max_results = max_results.unwrap_or(usize::MAX);

    if max_results == 0 {
        return Vec::new().into_iter();
    }

    let mut positions = Vec::new();
    for start in 0..=haystack.len() {
        if let Some(state) = best_pattern_match_state(pattern, haystack, start, config) {
            positions.push((start + 1, state.end));
            if positions.len() >= max_results {
                break;
            }
        }
    }

    positions.into_iter()
}
