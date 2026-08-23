use super::state_set::UniqueMatchStates;
use super::*;

#[cfg(test)]
std::thread_local! {
    static EXCEPT_EXCLUDED_EVALUATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static EXCEPT_MEMBERSHIP_COMPARISONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn reset_except_work_counts() {
    EXCEPT_EXCLUDED_EVALUATIONS.set(0);
    EXCEPT_MEMBERSHIP_COMPARISONS.set(0);
}

#[cfg(test)]
fn except_work_counts() -> (usize, usize) {
    (
        EXCEPT_EXCLUDED_EVALUATIONS.get(),
        EXCEPT_MEMBERSHIP_COMPARISONS.get(),
    )
}

pub(in crate::strings::patterns) fn pattern_match_text(chars: &[char]) -> String {
    chars.iter().collect()
}

pub(in crate::strings::patterns) fn best_pattern_match_at(
    patterns: &[StringPattern],
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Option<(usize, usize)> {
    patterns
        .iter()
        .enumerate()
        .filter_map(|(index, pattern)| {
            best_pattern_match_end(pattern, chars, start, config).map(|end| (index, end))
        })
        .max_by_key(|(index, end)| (*end, usize::MAX - *index))
}

pub(in crate::strings::patterns) fn best_pattern_match_at_with_bindings(
    patterns: &[StringPattern],
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Option<(usize, StringPatternMatchState)> {
    patterns
        .iter()
        .enumerate()
        .filter_map(|(index, pattern)| {
            best_pattern_match_state(pattern, chars, start, config).map(|state| (index, state))
        })
        .max_by_key(|(index, state)| (state.end, usize::MAX - *index))
}

pub(super) fn best_pattern_suffix_start(
    patterns: &[StringPattern],
    chars: &[char],
) -> Option<usize> {
    let end = chars.len();
    (0..end)
        .filter(|start| {
            patterns.iter().any(|pattern| {
                best_pattern_match_state(
                    pattern,
                    chars,
                    *start,
                    StringPatternMatchConfig::default(),
                )
                .is_some_and(|state| state.end == end)
            })
        })
        .min()
}

pub(in crate::strings::patterns) fn best_pattern_match_end(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Option<usize> {
    let positions = pattern_match_end_positions(pattern, chars, start, config);
    if pattern_prefers_shortest(pattern) {
        positions.into_iter().min()
    } else {
        positions.into_iter().max()
    }
}

pub(in crate::strings::patterns) fn best_pattern_match_state(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Option<StringPatternMatchState> {
    let states = pattern_match_states(pattern, chars, start, config);
    if pattern_prefers_shortest(pattern) {
        states.into_iter().min_by_key(|state| state.end)
    } else {
        states.into_iter().max_by_key(|state| state.end)
    }
}

pub(in crate::strings::patterns) fn pattern_can_match_through_end(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    end: usize,
    config: StringPatternMatchConfig,
) -> bool {
    pattern_match_states(pattern, chars, start, config)
        .into_iter()
        .any(|state| state.end == end)
}

fn pattern_match_end_positions(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Vec<usize> {
    if start > chars.len() {
        return Vec::new();
    }

    match pattern {
        StringPattern::Literal(literal) => literal_match_end(chars, literal, start, config)
            .into_iter()
            .collect(),
        StringPattern::AnyCharacter => any_character_match_end_positions(chars, start),
        StringPattern::AnySequence { min } => any_sequence_match_end_positions(chars, start, *min),
        StringPattern::Capture { inner, .. } => {
            pattern_match_end_positions(inner, chars, start, config)
        }
        StringPattern::Whitespace => whitespace_match_end_positions(chars, start),
        StringPattern::WhitespaceCharacter => {
            single_char_match_end_positions(chars, start, is_string_whitespace)
        }
        StringPattern::LetterCharacter => {
            single_char_match_end_positions(chars, start, char::is_alphabetic)
        }
        StringPattern::DigitCharacter => {
            single_char_match_end_positions(chars, start, |ch| ch.is_ascii_digit())
        }
        StringPattern::WordCharacter => {
            single_char_match_end_positions(chars, start, is_string_word_character)
        }
        StringPattern::PunctuationCharacter => single_char_match_end_positions(
            chars,
            start,
            crate::character_classes::is_punctuation_character,
        ),
        StringPattern::RegularExpression(pattern) => {
            regex_pattern_match_ends(pattern, chars, start)
        }
        StringPattern::NumberString => number_string_match_end_positions(chars, start),
        StringPattern::Except { excluded, inner } => {
            let candidates = match inner {
                Some(inner) => pattern_match_end_positions(inner, chars, start, config),
                None => any_character_match_end_positions(chars, start),
            };
            if candidates.is_empty() {
                candidates
            } else {
                #[cfg(test)]
                EXCEPT_EXCLUDED_EVALUATIONS.set(EXCEPT_EXCLUDED_EVALUATIONS.get() + 1);
                let excluded_positions =
                    pattern_match_end_positions(excluded, chars, start, config);
                subtract_sorted_positions(candidates, &excluded_positions)
            }
        }
        StringPattern::StartOfString => (start == 0).then_some(start).into_iter().collect(),
        StringPattern::EndOfString => (start == chars.len())
            .then_some(start)
            .into_iter()
            .collect(),
        StringPattern::StartOfLine => start_of_line_match_end_positions(chars, start),
        StringPattern::EndOfLine => end_of_line_match_end_positions(chars, start),
        StringPattern::WordBoundary => word_boundary_match_end_positions(chars, start),
        StringPattern::Repeated { pattern, min, max } => {
            repeated_pattern_end_positions(pattern, chars, start, *min, *max, config)
        }
        StringPattern::Shortest(inner) | StringPattern::Longest(inner) => {
            pattern_match_end_positions(inner, chars, start, config)
        }
        StringPattern::Sequence(parts) => {
            let mut positions = vec![start];

            for part in parts {
                let mut next_positions = Vec::new();
                for position in positions {
                    next_positions
                        .extend(pattern_match_end_positions(part, chars, position, config));
                }
                next_positions.sort_unstable();
                next_positions.dedup();
                if next_positions.is_empty() {
                    return Vec::new();
                }
                positions = next_positions;
            }

            positions
        }
        StringPattern::Alternatives(alternatives) => {
            let mut positions = Vec::new();
            for alternative in alternatives {
                positions.extend(pattern_match_end_positions(
                    alternative,
                    chars,
                    start,
                    config,
                ));
            }
            positions.sort_unstable();
            positions.dedup();
            positions
        }
    }
}

fn subtract_sorted_positions(candidates: Vec<usize>, excluded: &[usize]) -> Vec<usize> {
    let mut excluded_index = 0usize;
    candidates
        .into_iter()
        .filter(|candidate| {
            loop {
                let Some(excluded_position) = excluded.get(excluded_index) else {
                    return true;
                };
                #[cfg(test)]
                EXCEPT_MEMBERSHIP_COMPARISONS.set(EXCEPT_MEMBERSHIP_COMPARISONS.get() + 1);
                match excluded_position.cmp(candidate) {
                    std::cmp::Ordering::Less => excluded_index += 1,
                    std::cmp::Ordering::Equal => {
                        excluded_index += 1;
                        return false;
                    }
                    std::cmp::Ordering::Greater => return true,
                }
            }
        })
        .collect()
}

fn pattern_prefers_shortest(pattern: &StringPattern) -> bool {
    matches!(pattern, StringPattern::Shortest(_))
}

fn pattern_match_states(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Vec<StringPatternMatchState> {
    if start > chars.len() {
        return Vec::new();
    }

    match pattern {
        StringPattern::Literal(_)
        | StringPattern::AnyCharacter
        | StringPattern::AnySequence { .. }
        | StringPattern::Whitespace
        | StringPattern::WhitespaceCharacter
        | StringPattern::LetterCharacter
        | StringPattern::DigitCharacter
        | StringPattern::WordCharacter
        | StringPattern::PunctuationCharacter
        | StringPattern::RegularExpression(_)
        | StringPattern::NumberString
        | StringPattern::Except { .. }
        | StringPattern::StartOfString
        | StringPattern::EndOfString
        | StringPattern::StartOfLine
        | StringPattern::EndOfLine
        | StringPattern::WordBoundary => pattern_match_end_positions(pattern, chars, start, config)
            .into_iter()
            .map(empty_match_state)
            .collect(),
        StringPattern::Capture { symbol, inner } => {
            pattern_match_states(inner, chars, start, config)
                .into_iter()
                .filter_map(|state| {
                    let binding = Expr::string(pattern_match_text(&chars[start..state.end]));
                    let bindings = merge_symbol_binding(&state.bindings, symbol, binding)?;
                    Some(StringPatternMatchState {
                        end: state.end,
                        bindings,
                    })
                })
                .collect()
        }
        StringPattern::Repeated { pattern, min, max } => {
            repeated_pattern_match_states(pattern, chars, start, *min, *max, config)
        }
        StringPattern::Shortest(inner) | StringPattern::Longest(inner) => {
            pattern_match_states(inner, chars, start, config)
        }
        StringPattern::Sequence(parts) => {
            let mut states = vec![empty_match_state(start)];
            for part in parts {
                let mut next_states = UniqueMatchStates::new();
                for state in states {
                    for part_state in pattern_match_states(part, chars, state.end, config) {
                        if let Some(bindings) =
                            merge_match_bindings(&state.bindings, &part_state.bindings)
                        {
                            next_states.insert(StringPatternMatchState {
                                end: part_state.end,
                                bindings,
                            });
                        }
                    }
                }
                if next_states.is_empty() {
                    return Vec::new();
                }
                states = next_states.into_vec();
            }
            states
        }
        StringPattern::Alternatives(alternatives) => {
            let mut states = UniqueMatchStates::new();
            for alternative in alternatives {
                for state in pattern_match_states(alternative, chars, start, config) {
                    states.insert(state);
                }
            }
            states.into_vec()
        }
    }
}

fn repeated_pattern_end_positions(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    min: usize,
    max: Option<usize>,
    config: StringPatternMatchConfig,
) -> Vec<usize> {
    let mut frontier = vec![start];
    let mut positions = if min == 0 { vec![start] } else { Vec::new() };
    let mut steps = 0usize;

    loop {
        if max.is_some_and(|max| steps >= max) {
            break;
        }
        let mut next = Vec::new();
        for position in frontier {
            for end in pattern_match_end_positions(pattern, chars, position, config) {
                if end > position {
                    next.push(end);
                }
            }
        }
        next.sort_unstable();
        next.dedup();

        if next.is_empty() {
            break;
        }

        steps += 1;
        if steps >= min {
            positions.extend(next.iter().copied());
        }
        frontier = next;
    }

    positions.sort_unstable();
    positions.dedup();
    positions
}

fn repeated_pattern_match_states(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    min: usize,
    max: Option<usize>,
    config: StringPatternMatchConfig,
) -> Vec<StringPatternMatchState> {
    let mut frontier = vec![empty_match_state(start)];
    let mut states = UniqueMatchStates::new();
    if min == 0 {
        states.insert(empty_match_state(start));
    }
    let mut steps = 0usize;

    loop {
        if max.is_some_and(|max| steps >= max) {
            break;
        }

        let mut next = UniqueMatchStates::new();
        for state in frontier {
            for part_state in pattern_match_states(pattern, chars, state.end, config) {
                if part_state.end <= state.end {
                    continue;
                }
                if let Some(bindings) = merge_match_bindings(&state.bindings, &part_state.bindings)
                {
                    next.insert(StringPatternMatchState {
                        end: part_state.end,
                        bindings,
                    });
                }
            }
        }

        if next.is_empty() {
            break;
        }

        steps += 1;
        if steps >= min {
            for state in next.iter().cloned() {
                states.insert(state);
            }
        }
        frontier = next.into_vec();
    }

    states.into_vec()
}

fn any_character_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    (start < chars.len())
        .then_some(start + 1)
        .into_iter()
        .collect()
}

fn any_sequence_match_end_positions(chars: &[char], start: usize, min: usize) -> Vec<usize> {
    let start = start.saturating_add(min);
    if start > chars.len() {
        return Vec::new();
    }
    (start..=chars.len()).collect()
}

fn literal_match_end(
    chars: &[char],
    literal: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Option<usize> {
    if chars.len().saturating_sub(start) < literal.len() {
        return None;
    }

    for (offset, expected) in literal.iter().enumerate() {
        let actual = chars[start + offset];
        let matches = if config.ignore_case {
            actual.to_lowercase().to_string() == expected.to_lowercase().to_string()
        } else {
            actual == *expected
        };
        if !matches {
            return None;
        }
    }

    Some(start + literal.len())
}

fn whitespace_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    if start >= chars.len() || !is_string_whitespace(chars[start]) {
        return Vec::new();
    }

    let mut end = start;
    let mut positions = Vec::new();
    while end < chars.len() && is_string_whitespace(chars[end]) {
        end += 1;
        positions.push(end);
    }
    positions
}

fn single_char_match_end_positions(
    chars: &[char],
    start: usize,
    predicate: fn(char) -> bool,
) -> Vec<usize> {
    if start >= chars.len() || !predicate(chars[start]) {
        return Vec::new();
    }

    vec![start + 1]
}

fn start_of_line_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    (start == 0 || start.checked_sub(1).and_then(|index| chars.get(index)) == Some(&'\n'))
        .then_some(start)
        .into_iter()
        .collect()
}

fn end_of_line_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    (start == chars.len() || chars.get(start) == Some(&'\n'))
        .then_some(start)
        .into_iter()
        .collect()
}

fn word_boundary_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    let previous_is_word = start
        .checked_sub(1)
        .and_then(|index| chars.get(index))
        .is_some_and(|ch| is_string_word_character(*ch));
    let next_is_word = chars
        .get(start)
        .is_some_and(|ch| is_string_word_character(*ch));

    (previous_is_word != next_is_word)
        .then_some(start)
        .into_iter()
        .collect()
}

fn is_string_word_character(ch: char) -> bool {
    ch.is_alphabetic() || ch.is_ascii_digit() || ch == '_'
}

fn number_string_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    if start >= chars.len() {
        return Vec::new();
    }

    let mut positions = Vec::new();
    let mut index = start;

    if matches!(chars.get(index), Some(&'+') | Some(&'-')) {
        index += 1;
    }

    let integer_start = index;
    while index < chars.len() && chars[index].is_ascii_digit() {
        index += 1;
        positions.push(index);
    }
    let integer_end = index;

    if chars.get(index) == Some(&'.') {
        index += 1;
        if integer_end > integer_start {
            positions.push(index);
        }

        let fraction_start = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
            positions.push(index);
        }

        if integer_end == integer_start && fraction_start == index {
            return Vec::new();
        }
    }

    positions
}

fn empty_match_state(end: usize) -> StringPatternMatchState {
    StringPatternMatchState {
        end,
        bindings: SymbolExprMap::default(),
    }
}

fn merge_symbol_binding(
    bindings: &SymbolExprMap,
    symbol: &Symbol,
    value: Expr,
) -> Option<SymbolExprMap> {
    let mut merged = bindings.clone();
    if let Some(existing) = merged.get(symbol) {
        if existing != &value {
            return None;
        }
    } else {
        merged.insert(symbol.clone(), value);
    }
    Some(merged)
}

fn merge_match_bindings(left: &SymbolExprMap, right: &SymbolExprMap) -> Option<SymbolExprMap> {
    let mut merged = left.clone();
    for (symbol, value) in right {
        if let Some(existing) = merged.get(symbol) {
            if existing != value {
                return None;
            }
        } else {
            merged.insert(symbol.clone(), value.clone());
        }
    }
    Some(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn except_filters_with_one_excluded_evaluation() {
        let pattern = StringPattern::Except {
            excluded: Box::new(StringPattern::Repeated {
                pattern: Box::new(StringPattern::Literal(vec!['b'])),
                min: 1,
                max: None,
            }),
            inner: Some(Box::new(StringPattern::AnySequence { min: 1 })),
        };
        let chars = vec!['b'; 32];

        reset_except_work_counts();
        let positions =
            pattern_match_end_positions(&pattern, &chars, 0, StringPatternMatchConfig::default());

        assert!(positions.is_empty());
        assert_eq!(except_work_counts(), (1, 32));
    }
}
