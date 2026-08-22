use super::engine::{
    best_pattern_match_at, best_pattern_match_at_with_bindings, best_pattern_match_end,
    best_pattern_match_state, pattern_can_match_through_end, pattern_match_text,
};
use super::literal::{
    LiteralMatches, exact_literal_pattern, linear_literal_pattern,
    linear_literal_search_worthwhile, single_linear_literal_pattern,
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
    if let Some(literal) = exact_literal_pattern(pattern, config) {
        if linear_literal_search_worthwhile(haystack.len(), literal.len()) {
            if haystack.starts_with(literal) {
                return true;
            }
            return LiteralMatches::new(haystack, literal, false)
                .next()
                .is_some();
        }
        return literal.len() <= haystack.len()
            && haystack
                .windows(literal.len())
                .any(|window| window == literal);
    }

    (0..=haystack.len())
        .any(|start| best_pattern_match_state(pattern, haystack, start, config).is_some())
}

pub(in crate::strings::patterns) fn string_starts_q(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
) -> bool {
    if let Some(literal) = exact_literal_pattern(pattern, config) {
        return haystack.starts_with(literal);
    }

    best_pattern_match_state(pattern, haystack, 0, config).is_some()
}

pub(in crate::strings::patterns) fn string_ends_q(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
) -> bool {
    if let Some(literal) = exact_literal_pattern(pattern, config) {
        return haystack.ends_with(literal);
    }

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
    patterns.iter().any(|pattern| {
        if let Some(literal) = exact_literal_pattern(pattern, config) {
            return haystack == literal;
        }
        pattern_can_match_through_end(pattern, haystack, 0, haystack.len(), config)
    })
}

pub(in crate::strings::patterns) fn string_free_q(
    haystack: &[char],
    patterns: &[StringPattern],
    config: StringPatternMatchConfig,
) -> bool {
    if let [pattern] = patterns {
        if let Some(literal) = exact_literal_pattern(pattern, config) {
            if linear_literal_search_worthwhile(haystack.len(), literal.len()) {
                if haystack.starts_with(literal) {
                    return false;
                }
                return LiteralMatches::new(haystack, literal, false)
                    .next()
                    .is_none();
            }
            return literal.len() > haystack.len()
                || !haystack
                    .windows(literal.len())
                    .any(|window| window == literal);
        }
    }

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
    if let Some(literal) = single_linear_literal_pattern(
        patterns,
        StringPatternMatchConfig {
            ignore_case: config.ignore_case,
        },
        chars.len(),
    ) {
        return LiteralMatches::new(
            &chars,
            literal,
            matches!(config.overlaps, StringSearchOverlaps::Overlapping),
        )
        .count();
    }

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
            let Some(next_index) =
                next_string_match_index(index, state.end, chars.len(), config.overlaps)
            else {
                break;
            };
            index = next_index;
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
    if let Some(literal) = single_linear_literal_pattern(
        patterns,
        StringPatternMatchConfig {
            ignore_case: config.ignore_case,
        },
        chars.len(),
    ) {
        let mut output = String::with_capacity(value.len());
        let mut copied_through = 0usize;
        for (start, end) in LiteralMatches::new(&chars, literal, false) {
            output.extend(chars[copied_through..start].iter());
            copied_through = end;
        }
        output.extend(chars[copied_through..].iter());
        return output;
    }

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
    config: StringCasesConfig,
) -> Vec<Expr> {
    if max_matches == 0 {
        return Vec::new();
    }

    let chars = value.chars().collect::<Vec<_>>();
    if let ExactStringCaseSpec::Patterns(patterns) = spec {
        if let Some(literal) =
            single_linear_literal_pattern(patterns, config.match_config, chars.len())
        {
            if max_matches == 1 && chars.starts_with(literal) {
                return vec![Expr::string(pattern_match_text(literal))];
            }
            return LiteralMatches::new(
                &chars,
                literal,
                matches!(config.overlaps, StringSearchOverlaps::Overlapping),
            )
            .take(max_matches)
            .map(|(start, end)| Expr::string(pattern_match_text(&chars[start..end])))
            .collect();
        }
    }

    let mut index = 0;
    let mut matches = Vec::new();

    while index <= chars.len() && matches.len() < max_matches {
        match spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, end)) =
                    best_pattern_match_at(patterns, &chars, index, config.match_config)
                {
                    matches.push(Expr::string(pattern_match_text(&chars[index..end])));
                    let Some(next_index) =
                        next_string_match_index(index, end, chars.len(), config.overlaps)
                    else {
                        break;
                    };
                    index = next_index;
                } else {
                    if index == chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((rule, end)) = rules.iter().find_map(|rule| {
                    best_pattern_match_end(&rule.pattern, &chars, index, config.match_config)
                        .map(|end| (rule, end))
                }) {
                    let output = match &rule.output {
                        ExactStringCaseOutput::Immediate(expr) => expr.clone(),
                        ExactStringCaseOutput::Delayed(_) => {
                            Expr::string(pattern_match_text(&chars[index..end]))
                        }
                    };
                    matches.push(output);
                    let Some(next_index) =
                        next_string_match_index(index, end, chars.len(), config.overlaps)
                    else {
                        break;
                    };
                    index = next_index;
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
    config: StringPositionConfig,
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
    config: StringPositionConfig,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    let max_results = max_results.unwrap_or(usize::MAX);

    if max_results == 0 {
        return Vec::new().into_iter();
    }

    if let Some(literal) = linear_literal_pattern(pattern, config.match_config, haystack.len()) {
        if max_results == 1 && haystack.starts_with(literal) {
            return vec![(1, literal.len())].into_iter();
        }
        return LiteralMatches::new(
            haystack,
            literal,
            matches!(config.overlaps, StringSearchOverlaps::Overlapping),
        )
        .take(max_results)
        .map(|(start, end)| (start + 1, end))
        .collect::<Vec<_>>()
        .into_iter();
    }

    let mut positions = Vec::new();
    let mut start = 0usize;
    while start <= haystack.len() {
        if let Some(state) = best_pattern_match_state(pattern, haystack, start, config.match_config)
        {
            positions.push((start + 1, state.end));
            if positions.len() >= max_results {
                break;
            }
            let Some(next_start) =
                next_string_match_index(start, state.end, haystack.len(), config.overlaps)
            else {
                break;
            };
            start = next_start;
        } else {
            if start == haystack.len() {
                break;
            }
            start += 1;
        }
    }

    positions.into_iter()
}
