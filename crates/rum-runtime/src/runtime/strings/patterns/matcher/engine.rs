use super::*;

pub(in crate::runtime::strings::patterns) fn pattern_match_text(chars: &[char]) -> String {
    chars.iter().collect()
}

pub(in crate::runtime::strings::patterns) fn best_pattern_match_at(
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

pub(in crate::runtime::strings::patterns) fn best_pattern_match_at_with_bindings(
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

pub(in crate::runtime::strings::patterns) fn best_pattern_match_end(
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

pub(in crate::runtime::strings::patterns) fn best_pattern_match_state(
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
        StringPattern::WordCharacter => single_char_match_end_positions(chars, start, |ch| {
            ch.is_alphabetic() || ch.is_ascii_digit() || ch == '_'
        }),
        StringPattern::RegularExpression(pattern) => {
            regex_pattern_match_ends(pattern, chars, start)
        }
        StringPattern::NumberString => number_string_match_end_positions(chars, start),
        StringPattern::Except { excluded, inner } => {
            let candidates = match inner {
                Some(inner) => pattern_match_end_positions(inner, chars, start, config),
                None => any_character_match_end_positions(chars, start),
            };

            candidates
                .into_iter()
                .filter(|end| {
                    !pattern_match_end_positions(excluded, chars, start, config).contains(end)
                })
                .collect()
        }
        StringPattern::StartOfString => (start == 0).then_some(start).into_iter().collect(),
        StringPattern::EndOfString => (start == chars.len())
            .then_some(start)
            .into_iter()
            .collect(),
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
        | StringPattern::RegularExpression(_)
        | StringPattern::NumberString
        | StringPattern::Except { .. }
        | StringPattern::StartOfString
        | StringPattern::EndOfString => pattern_match_end_positions(pattern, chars, start, config)
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
                let mut next_states = Vec::new();
                for state in states {
                    for part_state in pattern_match_states(part, chars, state.end, config) {
                        if let Some(bindings) =
                            merge_match_bindings(&state.bindings, &part_state.bindings)
                        {
                            push_unique_match_state(
                                &mut next_states,
                                StringPatternMatchState {
                                    end: part_state.end,
                                    bindings,
                                },
                            );
                        }
                    }
                }
                if next_states.is_empty() {
                    return Vec::new();
                }
                states = next_states;
            }
            states
        }
        StringPattern::Alternatives(alternatives) => {
            let mut states = Vec::new();
            for alternative in alternatives {
                for state in pattern_match_states(alternative, chars, start, config) {
                    push_unique_match_state(&mut states, state);
                }
            }
            states
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
    let mut states = if min == 0 {
        vec![empty_match_state(start)]
    } else {
        Vec::new()
    };
    let mut steps = 0usize;

    loop {
        if max.is_some_and(|max| steps >= max) {
            break;
        }

        let mut next = Vec::new();
        for state in frontier {
            for part_state in pattern_match_states(pattern, chars, state.end, config) {
                if part_state.end <= state.end {
                    continue;
                }
                if let Some(bindings) = merge_match_bindings(&state.bindings, &part_state.bindings)
                {
                    push_unique_match_state(
                        &mut next,
                        StringPatternMatchState {
                            end: part_state.end,
                            bindings,
                        },
                    );
                }
            }
        }

        if next.is_empty() {
            break;
        }

        steps += 1;
        if steps >= min {
            for state in next.iter().cloned() {
                push_unique_match_state(&mut states, state);
            }
        }
        frontier = next;
    }

    states
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
        bindings: BTreeMap::new(),
    }
}

fn merge_symbol_binding(
    bindings: &BTreeMap<Symbol, Expr>,
    symbol: &Symbol,
    value: Expr,
) -> Option<BTreeMap<Symbol, Expr>> {
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

fn merge_match_bindings(
    left: &BTreeMap<Symbol, Expr>,
    right: &BTreeMap<Symbol, Expr>,
) -> Option<BTreeMap<Symbol, Expr>> {
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

fn push_unique_match_state(
    states: &mut Vec<StringPatternMatchState>,
    candidate: StringPatternMatchState,
) {
    if !states.contains(&candidate) {
        states.push(candidate);
    }
}
