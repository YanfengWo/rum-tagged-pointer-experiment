use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RegexPattern {
    branches: Vec<Vec<RegexToken>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegexToken {
    atom: RegexAtom,
    quantifier: RegexQuantifier,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RegexAtom {
    Literal(char),
    AnyChar,
    Digit,
    Word,
    Whitespace,
    Group(Box<RegexPattern>),
    Class {
        negated: bool,
        items: Vec<RegexClassItem>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RegexClassItem {
    Char(char),
    Range(char, char),
    Digit,
    Word,
    Whitespace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegexQuantifier {
    One,
    ZeroOrMore,
    OneOrMore,
    ZeroOrOne,
    Exact(usize),
    Range { min: usize, max: Option<usize> },
}

pub(super) fn compile_names_regex_pattern(raw: &str) -> Option<RegexPattern> {
    let stripped = strip_full_match_anchors(raw);
    let branches = split_regex_branches(&stripped)?
        .into_iter()
        .map(|branch| compile_names_regex_branch(&branch))
        .collect::<Option<Vec<_>>>()?;

    Some(RegexPattern { branches })
}

fn strip_full_match_anchors(raw: &str) -> String {
    let mut chars = raw.chars().collect::<Vec<_>>();

    if chars.first() == Some(&'^') {
        chars.remove(0);
    }

    if chars.last() == Some(&'$') {
        chars.pop();
    }

    chars.into_iter().collect()
}

fn split_regex_branches(raw: &str) -> Option<Vec<String>> {
    let chars = raw.chars().collect::<Vec<_>>();
    let mut branches = Vec::new();
    let mut current = String::new();
    let mut index = 0;
    let mut in_class = false;
    let mut brace_depth = 0_usize;
    let mut paren_depth = 0_usize;

    while index < chars.len() {
        let ch = chars[index];

        if ch == '\\' {
            current.push(ch);
            index += 1;
            if let Some(next) = chars.get(index) {
                current.push(*next);
                index += 1;
            }
            continue;
        }

        match ch {
            '[' if !in_class => {
                in_class = true;
                current.push(ch);
            }
            ']' if in_class => {
                in_class = false;
                current.push(ch);
            }
            '(' if !in_class => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' if !in_class => {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
                current.push(ch);
            }
            '{' if !in_class => {
                brace_depth += 1;
                current.push(ch);
            }
            '}' if !in_class => {
                if brace_depth == 0 {
                    return None;
                }
                brace_depth -= 1;
                current.push(ch);
            }
            '|' if !in_class && brace_depth == 0 && paren_depth == 0 => {
                branches.push(current);
                current = String::new();
            }
            _ => current.push(ch),
        }

        index += 1;
    }

    if in_class || brace_depth != 0 || paren_depth != 0 {
        return None;
    }

    branches.push(current);
    Some(branches)
}

fn compile_names_regex_branch(raw: &str) -> Option<Vec<RegexToken>> {
    let chars = raw.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        let atom = parse_regex_atom(&chars, &mut index)?;
        let quantifier = parse_regex_quantifier(&chars, &mut index)?;

        tokens.push(RegexToken { atom, quantifier });
    }

    Some(tokens)
}

fn parse_regex_atom(chars: &[char], index: &mut usize) -> Option<RegexAtom> {
    let ch = *chars.get(*index)?;
    *index += 1;

    match ch {
        '.' => Some(RegexAtom::AnyChar),
        '\\' => parse_regex_escape(chars, index),
        '[' => parse_regex_class(chars, index),
        '(' => parse_regex_group(chars, index),
        ')' | '|' | '{' | '}' | '*' | '+' | '?' => None,
        _ => Some(RegexAtom::Literal(ch)),
    }
}

fn parse_regex_group(chars: &[char], index: &mut usize) -> Option<RegexAtom> {
    let mut raw = String::new();
    let mut depth = 1_usize;
    let mut in_class = false;
    let mut saw_group_prefix = false;

    if chars.get(*index) == Some(&'?') && chars.get(*index + 1) == Some(&':') {
        *index += 2;
        saw_group_prefix = true;
    }

    while *index < chars.len() {
        let ch = chars[*index];
        *index += 1;

        if ch == '\\' {
            raw.push(ch);
            let next = *chars.get(*index)?;
            raw.push(next);
            *index += 1;
            continue;
        }

        match ch {
            '[' if !in_class => {
                in_class = true;
                raw.push(ch);
            }
            ']' if in_class => {
                in_class = false;
                raw.push(ch);
            }
            '(' if !in_class => {
                depth += 1;
                raw.push(ch);
            }
            ')' if !in_class => {
                depth -= 1;
                if depth == 0 {
                    if saw_group_prefix {
                        // Non-capturing groups do not affect matching shape in the current
                        // names-pattern subset; we lower them to ordinary nested branches.
                    }
                    return compile_names_regex_pattern(&raw)
                        .map(|pattern| RegexAtom::Group(Box::new(pattern)));
                }
                raw.push(ch);
            }
            _ => raw.push(ch),
        }
    }

    None
}

fn parse_regex_quantifier(chars: &[char], index: &mut usize) -> Option<RegexQuantifier> {
    let Some(ch) = chars.get(*index).copied() else {
        return Some(RegexQuantifier::One);
    };

    match ch {
        '*' => {
            *index += 1;
            Some(RegexQuantifier::ZeroOrMore)
        }
        '+' => {
            *index += 1;
            Some(RegexQuantifier::OneOrMore)
        }
        '?' => {
            *index += 1;
            Some(RegexQuantifier::ZeroOrOne)
        }
        '{' => parse_regex_braced_quantifier(chars, index),
        _ => Some(RegexQuantifier::One),
    }
}

fn parse_regex_braced_quantifier(chars: &[char], index: &mut usize) -> Option<RegexQuantifier> {
    *index += 1;
    let min = parse_regex_usize(chars, index)?;

    match chars.get(*index).copied()? {
        '}' => {
            *index += 1;
            Some(RegexQuantifier::Exact(min))
        }
        ',' => {
            *index += 1;

            if chars.get(*index) == Some(&'}') {
                *index += 1;
                return Some(RegexQuantifier::Range { min, max: None });
            }

            let max = parse_regex_usize(chars, index)?;
            if chars.get(*index) != Some(&'}') || max < min {
                return None;
            }
            *index += 1;

            Some(RegexQuantifier::Range {
                min,
                max: Some(max),
            })
        }
        _ => None,
    }
}

fn parse_regex_usize(chars: &[char], index: &mut usize) -> Option<usize> {
    let start = *index;

    while chars.get(*index).is_some_and(char::is_ascii_digit) {
        *index += 1;
    }

    (start != *index)
        .then(|| chars[start..*index].iter().collect::<String>().parse().ok())
        .flatten()
}

fn parse_regex_escape(chars: &[char], index: &mut usize) -> Option<RegexAtom> {
    let escaped = *chars.get(*index)?;
    *index += 1;

    Some(match escaped {
        'd' => RegexAtom::Digit,
        'w' => RegexAtom::Word,
        's' => RegexAtom::Whitespace,
        other => RegexAtom::Literal(other),
    })
}

fn parse_regex_class(chars: &[char], index: &mut usize) -> Option<RegexAtom> {
    let mut negated = false;

    if chars.get(*index) == Some(&'^') {
        negated = true;
        *index += 1;
    }

    let mut items = Vec::new();
    let mut saw_item = false;

    while *index < chars.len() {
        if chars[*index] == ']' {
            *index += 1;
            return saw_item.then_some(RegexAtom::Class { negated, items });
        }

        let first = parse_regex_class_item(chars, index)?;
        saw_item = true;

        if chars.get(*index) == Some(&'-')
            && chars.get(*index + 1).is_some()
            && chars.get(*index + 1) != Some(&']')
        {
            *index += 1;
            let second = parse_regex_class_item(chars, index)?;
            items.push(match (first, second) {
                (RegexClassItem::Char(start), RegexClassItem::Char(end)) => {
                    RegexClassItem::Range(start, end)
                }
                _ => return None,
            });
        } else {
            items.push(first);
        }
    }

    None
}

fn parse_regex_class_item(chars: &[char], index: &mut usize) -> Option<RegexClassItem> {
    let ch = *chars.get(*index)?;
    *index += 1;

    if ch == '\\' {
        let escaped = *chars.get(*index)?;
        *index += 1;

        return Some(match escaped {
            'd' => RegexClassItem::Digit,
            'w' => RegexClassItem::Word,
            's' => RegexClassItem::Whitespace,
            other => RegexClassItem::Char(other),
        });
    }

    Some(RegexClassItem::Char(ch))
}

pub(super) fn regex_pattern_matches(pattern: &RegexPattern, haystack: &[char]) -> bool {
    pattern
        .branches
        .iter()
        .any(|branch| regex_branch_matches(branch, haystack))
}

fn regex_branch_matches(tokens: &[RegexToken], haystack: &[char]) -> bool {
    regex_branch_match_ends(tokens, haystack, 0)
        .into_iter()
        .any(|end| end == haystack.len())
}

fn regex_pattern_match_ends(pattern: &RegexPattern, haystack: &[char], start: usize) -> Vec<usize> {
    let mut ends = Vec::new();

    for branch in &pattern.branches {
        ends.extend(regex_branch_match_ends(branch, haystack, start));
    }

    ends.sort_unstable();
    ends.dedup();
    ends
}

fn regex_branch_match_ends(tokens: &[RegexToken], haystack: &[char], start: usize) -> Vec<usize> {
    fn recur(
        tokens: &[RegexToken],
        token_index: usize,
        haystack: &[char],
        hay_index: usize,
        memo: &mut BTreeMap<(usize, usize), Vec<usize>>,
    ) -> Vec<usize> {
        if let Some(cached) = memo.get(&(token_index, hay_index)) {
            return cached.clone();
        }

        let result = if token_index == tokens.len() {
            vec![hay_index]
        } else {
            let token = &tokens[token_index];
            let mut ends = regex_token_match_ends(token, haystack, hay_index);
            let mut result = Vec::new();

            for end in ends.drain(..) {
                result.extend(recur(tokens, token_index + 1, haystack, end, memo));
            }

            result.sort_unstable();
            result.dedup();
            result
        };

        memo.insert((token_index, hay_index), result.clone());
        result
    }

    let mut memo = BTreeMap::new();
    recur(tokens, 0, haystack, start, &mut memo)
}

fn regex_token_match_ends(token: &RegexToken, haystack: &[char], start: usize) -> Vec<usize> {
    fn recur(
        atom: &RegexAtom,
        haystack: &[char],
        current: usize,
        repeats: usize,
        min: usize,
        max: Option<usize>,
        ends: &mut Vec<usize>,
    ) {
        if repeats >= min {
            ends.push(current);
        }

        if max.is_some_and(|bound| repeats >= bound) {
            return;
        }

        for next in regex_atom_match_ends(atom, haystack, current) {
            if next == current {
                continue;
            }
            recur(atom, haystack, next, repeats + 1, min, max, ends);
        }
    }

    let (min, max) = regex_quantifier_bounds(token.quantifier);
    let mut ends = Vec::new();
    recur(&token.atom, haystack, start, 0, min, max, &mut ends);
    ends.sort_unstable();
    ends.dedup();
    ends
}

fn regex_atom_match_ends(atom: &RegexAtom, haystack: &[char], start: usize) -> Vec<usize> {
    match atom {
        RegexAtom::Group(pattern) => regex_pattern_match_ends(pattern, haystack, start),
        _ => haystack
            .get(start)
            .copied()
            .filter(|ch| regex_atom_matches(atom, *ch))
            .map(|_| vec![start + 1])
            .unwrap_or_default(),
    }
}

fn regex_quantifier_bounds(quantifier: RegexQuantifier) -> (usize, Option<usize>) {
    match quantifier {
        RegexQuantifier::One => (1, Some(1)),
        RegexQuantifier::ZeroOrMore => (0, None),
        RegexQuantifier::OneOrMore => (1, None),
        RegexQuantifier::ZeroOrOne => (0, Some(1)),
        RegexQuantifier::Exact(count) => (count, Some(count)),
        RegexQuantifier::Range { min, max } => (min, max),
    }
}

fn regex_atom_matches(atom: &RegexAtom, ch: char) -> bool {
    match atom {
        RegexAtom::Literal(expected) => *expected == ch,
        RegexAtom::AnyChar => true,
        RegexAtom::Digit => ch.is_ascii_digit(),
        RegexAtom::Word => ch.is_ascii_alphanumeric() || ch == '_',
        RegexAtom::Whitespace => ch.is_whitespace(),
        RegexAtom::Group(_) => false,
        RegexAtom::Class { negated, items } => {
            let matched = items.iter().any(|item| regex_class_item_matches(item, ch));
            if *negated { !matched } else { matched }
        }
    }
}

fn regex_class_item_matches(item: &RegexClassItem, ch: char) -> bool {
    match item {
        RegexClassItem::Char(expected) => *expected == ch,
        RegexClassItem::Range(start, end) => *start <= ch && ch <= *end,
        RegexClassItem::Digit => ch.is_ascii_digit(),
        RegexClassItem::Word => ch.is_ascii_alphanumeric() || ch == '_',
        RegexClassItem::Whitespace => ch.is_whitespace(),
    }
}
