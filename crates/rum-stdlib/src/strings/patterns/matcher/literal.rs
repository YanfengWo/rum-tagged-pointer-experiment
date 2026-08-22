use super::*;

const LINEAR_LITERAL_MIN_NEEDLE: usize = 16;
const LINEAR_LITERAL_MIN_WORK: usize = 4_096;

pub(super) fn exact_literal_pattern<'a>(
    pattern: &'a StringPattern,
    config: StringPatternMatchConfig,
) -> Option<&'a [char]> {
    if config.ignore_case {
        return None;
    }

    match pattern {
        StringPattern::Literal(literal) => {
            debug_assert!(!literal.is_empty(), "parsed string literals are non-empty");
            Some(literal)
        }
        _ => None,
    }
}

pub(super) fn linear_literal_pattern<'a>(
    pattern: &'a StringPattern,
    config: StringPatternMatchConfig,
    haystack_len: usize,
) -> Option<&'a [char]> {
    let literal = exact_literal_pattern(pattern, config)?;
    linear_literal_search_worthwhile(haystack_len, literal.len()).then_some(literal)
}

pub(super) fn single_linear_literal_pattern<'a>(
    patterns: &'a [StringPattern],
    config: StringPatternMatchConfig,
    haystack_len: usize,
) -> Option<&'a [char]> {
    let [pattern] = patterns else {
        return None;
    };
    linear_literal_pattern(pattern, config, haystack_len)
}

pub(in crate::strings::patterns) fn linear_literal_match_ranges(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
    overlapping: bool,
    max_matches: usize,
) -> Option<Vec<(usize, usize)>> {
    let literal = linear_literal_pattern(pattern, config, haystack.len())?;
    if max_matches == 0 {
        return Some(Vec::new());
    }
    if max_matches == 1 && haystack.starts_with(literal) {
        return Some(vec![(0, literal.len())]);
    }
    Some(
        LiteralMatches::new(haystack, literal, overlapping)
            .take(max_matches)
            .collect(),
    )
}

pub(super) fn linear_literal_search_worthwhile(haystack_len: usize, needle_len: usize) -> bool {
    if needle_len < LINEAR_LITERAL_MIN_NEEDLE || needle_len > haystack_len {
        return false;
    }

    haystack_len
        .saturating_sub(needle_len)
        .saturating_add(1)
        .saturating_mul(needle_len)
        >= LINEAR_LITERAL_MIN_WORK
}

pub(super) struct LiteralMatches<'a> {
    haystack: &'a [char],
    needle: &'a [char],
    prefix: Vec<usize>,
    overlapping: bool,
    index: usize,
    matched: usize,
    #[cfg(test)]
    comparisons: usize,
}

impl<'a> LiteralMatches<'a> {
    pub(super) fn new(haystack: &'a [char], needle: &'a [char], overlapping: bool) -> Self {
        debug_assert!(!needle.is_empty());
        Self {
            haystack,
            needle,
            prefix: literal_prefix_table(needle),
            overlapping,
            index: 0,
            matched: 0,
            #[cfg(test)]
            comparisons: 0,
        }
    }

    fn characters_equal(&mut self, actual: char, expected: char) -> bool {
        #[cfg(test)]
        {
            self.comparisons += 1;
        }
        actual == expected
    }

    #[cfg(test)]
    pub(super) fn comparisons(&self) -> usize {
        self.comparisons
    }
}

impl Iterator for LiteralMatches<'_> {
    type Item = (usize, usize);

    #[inline(never)]
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.haystack.len() {
            let actual = self.haystack[self.index];
            loop {
                if self.characters_equal(actual, self.needle[self.matched]) {
                    self.matched += 1;
                    break;
                }
                if self.matched == 0 {
                    break;
                }
                self.matched = self.prefix[self.matched - 1];
            }

            self.index += 1;
            if self.matched != self.needle.len() {
                continue;
            }

            let end = self.index;
            let start = end - self.needle.len();
            self.matched = if self.overlapping {
                self.prefix[self.matched - 1]
            } else {
                0
            };
            return Some((start, end));
        }

        None
    }
}

fn literal_prefix_table(needle: &[char]) -> Vec<usize> {
    let mut prefix = vec![0; needle.len()];
    let mut matched = 0usize;

    for index in 1..needle.len() {
        while matched > 0 && needle[index] != needle[matched] {
            matched = prefix[matched - 1];
        }
        if needle[index] == needle[matched] {
            matched += 1;
        }
        prefix[index] = matched;
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(value: &str) -> Vec<char> {
        value.chars().collect()
    }

    #[test]
    fn literal_matches_preserve_overlap_and_unicode_character_offsets() {
        let haystack = chars("aaaaa");
        let needle = chars("aaa");
        assert_eq!(
            LiteralMatches::new(&haystack, &needle, true).collect::<Vec<_>>(),
            vec![(0, 3), (1, 4), (2, 5)]
        );
        assert_eq!(
            LiteralMatches::new(&haystack, &needle, false).collect::<Vec<_>>(),
            vec![(0, 3)]
        );

        let haystack = chars("αβαβα");
        let needle = chars("αβα");
        assert_eq!(
            LiteralMatches::new(&haystack, &needle, true).collect::<Vec<_>>(),
            vec![(0, 3), (2, 5)]
        );
    }

    #[test]
    fn linear_literal_search_bounds_long_prefix_comparison_work() {
        let haystack = vec!['a'; 16_000];
        let mut needle = vec!['a'; 8_000];
        needle.push('b');

        let mut matches = LiteralMatches::new(&haystack, &needle, false);
        assert_eq!(matches.next(), None);
        assert_eq!(matches.comparisons(), 24_000);
        assert!(linear_literal_search_worthwhile(
            haystack.len(),
            needle.len()
        ));
        assert!(!linear_literal_search_worthwhile(32, 8));
    }
}
