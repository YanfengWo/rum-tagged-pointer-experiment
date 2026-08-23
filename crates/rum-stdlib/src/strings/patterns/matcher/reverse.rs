use crate::strings::patterns::regex::reversed_regex_pattern;

use super::*;

pub(super) fn reversed_string_pattern(pattern: &StringPattern) -> Option<StringPattern> {
    Some(match pattern {
        StringPattern::Literal(literal) => {
            StringPattern::Literal(literal.iter().rev().copied().collect())
        }
        StringPattern::AnyCharacter
        | StringPattern::AnySequence { .. }
        | StringPattern::Whitespace
        | StringPattern::WhitespaceCharacter
        | StringPattern::LetterCharacter
        | StringPattern::DigitCharacter
        | StringPattern::WordCharacter
        | StringPattern::PunctuationCharacter
        | StringPattern::WordBoundary => pattern.clone(),
        StringPattern::Capture { symbol, inner } => StringPattern::Capture {
            symbol: symbol.clone(),
            inner: Box::new(reversed_string_pattern(inner)?),
        },
        StringPattern::RegularExpression(pattern) => {
            StringPattern::RegularExpression(reversed_regex_pattern(pattern))
        }
        StringPattern::NumberString => StringPattern::ReversedNumberString,
        StringPattern::ReversedNumberString => StringPattern::NumberString,
        StringPattern::Except { excluded, inner } => StringPattern::Except {
            excluded: Box::new(reversed_string_pattern(excluded)?),
            inner: match inner {
                Some(inner) => Some(Box::new(reversed_string_pattern(inner)?)),
                None => None,
            },
        },
        StringPattern::StartOfString => StringPattern::EndOfString,
        StringPattern::EndOfString => StringPattern::StartOfString,
        StringPattern::StartOfLine => StringPattern::EndOfLine,
        StringPattern::EndOfLine => StringPattern::StartOfLine,
        StringPattern::Repeated { pattern, min, max } => StringPattern::Repeated {
            pattern: Box::new(reversed_string_pattern(pattern)?),
            min: *min,
            max: *max,
        },
        StringPattern::Shortest(inner) => {
            StringPattern::Shortest(Box::new(reversed_string_pattern(inner)?))
        }
        StringPattern::Longest(inner) => {
            StringPattern::Longest(Box::new(reversed_string_pattern(inner)?))
        }
        StringPattern::Sequence(parts) => StringPattern::Sequence(
            parts
                .iter()
                .rev()
                .map(reversed_string_pattern)
                .collect::<Option<Vec<_>>>()?,
        ),
        StringPattern::Alternatives(alternatives) => StringPattern::Alternatives(
            alternatives
                .iter()
                .map(reversed_string_pattern)
                .collect::<Option<Vec<_>>>()?,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::super::engine::best_pattern_match_state;
    use super::*;

    #[test]
    fn reversal_swaps_sequence_direction_and_boundaries() {
        let pattern = StringPattern::Sequence(vec![
            StringPattern::StartOfLine,
            StringPattern::Literal(vec!['a', 'b']),
            StringPattern::EndOfString,
        ]);

        assert_eq!(
            reversed_string_pattern(&pattern),
            Some(StringPattern::Sequence(vec![
                StringPattern::StartOfString,
                StringPattern::Literal(vec!['b', 'a']),
                StringPattern::EndOfLine,
            ]))
        );
    }

    #[test]
    fn reversed_number_atom_preserves_full_match_language() {
        let pattern = StringPattern::NumberString;
        let reversed_pattern = reversed_string_pattern(&pattern).unwrap();

        for text in ["1", "+1", "-12.30", "1.", ".5", "+.5", "+", ".", "1x", "x1"] {
            let chars = text.chars().collect::<Vec<_>>();
            let reversed_chars = chars.iter().rev().copied().collect::<Vec<_>>();
            let original_matches =
                best_pattern_match_state(&pattern, &chars, 0, StringPatternMatchConfig::default())
                    .is_some_and(|state| state.end == chars.len());
            let reversed_matches = best_pattern_match_state(
                &reversed_pattern,
                &reversed_chars,
                0,
                StringPatternMatchConfig::default(),
            )
            .is_some_and(|state| state.end == reversed_chars.len());

            assert_eq!(original_matches, reversed_matches, "{text}");
        }
    }
}
