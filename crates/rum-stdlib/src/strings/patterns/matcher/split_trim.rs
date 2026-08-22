use super::engine::{best_pattern_match_at_with_bindings, best_pattern_suffix_start};
use super::literal::{LiteralMatches, exact_literal_pattern, single_linear_literal_pattern};
use super::*;

pub(super) fn split_by_whitespace(chars: &[char], limit: StringSplitLimit) -> Vec<String> {
    let mut parts = Vec::new();
    let mut index = 0usize;

    while index < chars.len() {
        if matches!(limit, StringSplitLimit::MaxSubstrings(max) if parts.len() + 1 >= max) {
            let remainder = chars[index..].iter().collect::<String>();
            if !remainder.is_empty() {
                parts.push(remainder);
            }
            return parts;
        }

        let Some(run_start) = chars[index..]
            .iter()
            .position(|ch| is_string_whitespace(*ch))
            .map(|offset| index + offset)
        else {
            let tail = chars[index..].iter().collect::<String>();
            if !tail.is_empty() {
                parts.push(tail);
            }
            return parts;
        };

        if run_start > index {
            parts.push(chars[index..run_start].iter().collect());
        } else if matches!(limit, StringSplitLimit::All) {
            parts.push(String::new());
        }

        let mut run_end = run_start;
        while run_end < chars.len() && is_string_whitespace(chars[run_end]) {
            run_end += 1;
        }

        if run_end == chars.len() {
            if matches!(limit, StringSplitLimit::All) {
                parts.push(String::new());
            }
            return parts;
        }

        index = run_end;
    }

    if matches!(limit, StringSplitLimit::All) {
        vec![String::new()]
    } else {
        Vec::new()
    }
}

pub(super) fn split_by_string_patterns(
    chars: &[char],
    patterns: &[StringPattern],
    limit: StringSplitLimit,
    config: StringPatternMatchConfig,
) -> Vec<String> {
    if chars.is_empty() {
        return if matches!(limit, StringSplitLimit::All) {
            vec![String::new()]
        } else {
            Vec::new()
        };
    }

    if let Some(literal) = single_linear_literal_pattern(patterns, config, chars.len()) {
        return split_by_literal(chars, literal, limit);
    }

    let mut parts = Vec::new();
    let mut segment_start = 0usize;
    let mut index = 0usize;
    let include_edge_empties = matches!(limit, StringSplitLimit::All);

    while index <= chars.len() {
        if matches!(limit, StringSplitLimit::MaxSubstrings(max) if parts.len() + 1 >= max) {
            break;
        }

        let Some(end) = best_pattern_match_at_with_bindings(patterns, chars, index, config)
            .map(|(_, state)| state.end)
        else {
            if index == chars.len() {
                break;
            }
            index += 1;
            continue;
        };

        if end == index {
            let segment = chars[segment_start..index].iter().collect::<String>();
            if include_edge_empties || !segment.is_empty() || !parts.is_empty() {
                parts.push(segment);
            }
            segment_start = index;
            if index == chars.len() {
                break;
            }
            index += 1;
            continue;
        }

        let segment = chars[segment_start..index].iter().collect::<String>();
        if include_edge_empties || !segment.is_empty() || !parts.is_empty() {
            parts.push(segment);
        }

        index = end;
        segment_start = index;
    }

    let remainder = chars[segment_start..].iter().collect::<String>();
    if include_edge_empties || !remainder.is_empty() || !parts.is_empty() {
        parts.push(remainder);
    }

    parts
}

fn split_by_literal(chars: &[char], literal: &[char], limit: StringSplitLimit) -> Vec<String> {
    let include_edge_empties = matches!(limit, StringSplitLimit::All);
    let mut parts = Vec::new();
    let mut segment_start = 0usize;

    for (start, end) in LiteralMatches::new(chars, literal, false) {
        if matches!(limit, StringSplitLimit::MaxSubstrings(max) if parts.len() + 1 >= max) {
            break;
        }

        let segment = chars[segment_start..start].iter().collect::<String>();
        if include_edge_empties || !segment.is_empty() || !parts.is_empty() {
            parts.push(segment);
        }
        segment_start = end;
    }

    let remainder = chars[segment_start..].iter().collect::<String>();
    if include_edge_empties || !remainder.is_empty() || !parts.is_empty() {
        parts.push(remainder);
    }

    parts
}

pub(super) fn trim_whitespace_chars(chars: &[char]) -> String {
    let mut start = 0usize;
    let mut end = chars.len();

    while start < end && is_string_whitespace(chars[start]) {
        start += 1;
    }
    while start < end && is_string_whitespace(chars[end - 1]) {
        end -= 1;
    }

    chars[start..end].iter().collect()
}

pub(super) fn trim_string_patterns(chars: &[char], patterns: &[StringPattern]) -> String {
    if let [pattern] = patterns {
        if let Some(literal) = exact_literal_pattern(pattern, StringPatternMatchConfig::default()) {
            return trim_literal(chars, literal);
        }
    }

    let mut start = 0usize;
    let mut end = chars.len();

    while start < end {
        let Some(match_end) = best_pattern_match_at_with_bindings(
            patterns,
            &chars[..end],
            start,
            StringPatternMatchConfig::default(),
        )
        .map(|(_, state)| state.end) else {
            break;
        };
        start = match_end;
    }

    while start < end {
        let Some(match_start) =
            best_pattern_suffix_start(patterns, &chars[start..end]).map(|offset| start + offset)
        else {
            break;
        };
        end = match_start;
    }

    chars[start..end].iter().collect()
}

fn trim_literal(chars: &[char], literal: &[char]) -> String {
    let mut start = 0usize;
    let mut end = chars.len();

    while start < end && chars[start..end].starts_with(literal) {
        start += literal.len();
    }
    while start < end && chars[start..end].ends_with(literal) {
        end -= literal.len();
    }

    chars[start..end].iter().collect()
}
