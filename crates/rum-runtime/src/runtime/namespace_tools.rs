use super::*;

const EXTRA_SYSTEM_NAME_SYMBOLS: &[&str] = &[
    "Alternatives",
    "Flat",
    "Heads",
    "HoldAll",
    "HoldAllComplete",
    "HoldFirst",
    "HoldRest",
    "Listable",
    "OneIdentity",
    "OptionsPattern",
    "Orderless",
    "PatternTest",
    "Protected",
    "RegularExpression",
    "SequenceHold",
    "StringExpression",
];

#[derive(Clone, Debug, PartialEq, Eq)]
enum NamesPattern {
    Wildcard(String),
    Regex(RegexPattern),
    Sequence(Vec<NamesPattern>),
    Alternatives(Vec<NamesPattern>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegexPattern {
    tokens: Vec<RegexToken>,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NamesNormalizationError {
    InvalidPattern,
}

impl NamesNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidPattern => {
                "expected a string pattern, RegularExpression, StringExpression, Alternatives, or a list of them"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SyntaxInformationNormalizationError {
    InvalidTarget,
    InvalidRules,
}

impl SyntaxInformationNormalizationError {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::InvalidTarget => "expected a symbol or symbol-name string argument",
            Self::InvalidRules => {
                "expected a list of rules whose left-hand sides are string property names"
            }
        }
    }
}

impl Session {
    pub(super) fn eval_names(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (patterns, explicit_context) = match args {
            [] => (vec![NamesPattern::Wildcard("*".to_string())], false),
            [pattern_expr] => match self.compile_names_patterns(pattern_expr) {
                Some(compiled) => compiled,
                None => {
                    return Ok(self.invalid_call_with_arg_form(
                        "Names",
                        args,
                        NamesNormalizationError::InvalidPattern.message(),
                    ));
                }
            },
            _ => return Ok(self.invalid_call_with_exact_arg_count("Names", args, 1)),
        };

        Ok(string_list_expr(
            self.names_matching_patterns(&patterns, explicit_context),
        ))
    }

    pub(super) fn eval_syntax_information(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [target] => {
                let Some(symbol) = self.normalize_symbol_target_expr(target, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "SyntaxInformation",
                        args,
                        SyntaxInformationNormalizationError::InvalidTarget.message(),
                    ));
                };

                Ok(self.syntax_information_for_symbol(&symbol))
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("SyntaxInformation", args, 1)),
        }
    }

    pub(super) fn syntax_information_for_symbol(&self, symbol: &Symbol) -> Expr {
        self.syntax_information
            .get(symbol)
            .cloned()
            .unwrap_or_else(empty_list_expr)
    }

    pub(super) fn set_syntax_information_for_symbol(&mut self, symbol: Symbol, syntax: Expr) {
        if syntax == empty_list_expr() {
            self.syntax_information.remove(&symbol);
        } else {
            self.syntax_information.insert(symbol, syntax);
        }
    }

    pub(super) fn compile_assigned_syntax_information_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Result<Expr, SyntaxInformationNormalizationError> {
        let Expr::Call { head, args } = resolved else {
            return Err(SyntaxInformationNormalizationError::InvalidRules);
        };

        if !is_system_symbol_name(head, "List") {
            return Err(SyntaxInformationNormalizationError::InvalidRules);
        }

        for item in args {
            let Expr::Call {
                head: rule_head,
                args: rule_args,
            } = item
            else {
                return Err(SyntaxInformationNormalizationError::InvalidRules);
            };

            if !is_system_symbol_name(rule_head, "Rule") || rule_args.len() != 2 {
                return Err(SyntaxInformationNormalizationError::InvalidRules);
            }

            if !matches!(rule_args[0], Expr::Atom(Atom::String(_))) {
                return Err(SyntaxInformationNormalizationError::InvalidRules);
            }
        }

        Ok(resolved.clone())
    }

    fn compile_names_patterns(&self, expr: &Expr) -> Option<(Vec<NamesPattern>, bool)> {
        if let Some(pattern) = compile_names_pattern(expr) {
            return Some((vec![pattern], names_pattern_mentions_context(expr)));
        }

        let Expr::Call { head, args } = expr else {
            return None;
        };

        if !is_system_symbol_name(head, "List") {
            return None;
        }

        let patterns = args
            .iter()
            .map(compile_names_pattern)
            .collect::<Option<Vec<_>>>()?;

        Some((patterns, names_pattern_mentions_context(expr)))
    }

    fn names_matching_patterns(
        &self,
        patterns: &[NamesPattern],
        explicit_context: bool,
    ) -> Vec<String> {
        let visible_contexts = self.visible_contexts();
        let mut names = BTreeSet::new();

        for symbol in self.enumerated_names_symbols() {
            let candidate = if explicit_context {
                symbol.fully_qualified_name()
            } else {
                if symbol.context() != SYSTEM_CONTEXT
                    && symbol.context() != GLOBAL_CONTEXT
                    && !visible_contexts.contains(symbol.context())
                {
                    continue;
                }
                symbol.qualified_name()
            };

            if patterns
                .iter()
                .any(|pattern| names_pattern_matches(pattern, &candidate))
            {
                names.insert(candidate);
            }
        }

        names.into_iter().collect()
    }

    fn enumerated_names_symbols(&self) -> BTreeSet<Symbol> {
        let mut symbols = BTreeSet::new();

        for name in self.builtins.keys() {
            symbols.insert(Symbol::system(name));
        }

        for name in rum_core::CANONICAL_CORE_HEADS {
            symbols.insert(Symbol::system(*name));
        }

        for name in EXTRA_SYSTEM_NAME_SYMBOLS {
            symbols.insert(Symbol::system(*name));
        }

        symbols.extend(self.definitions.keys().cloned());
        symbols.extend(self.attributes.keys().cloned());
        symbols.extend(self.options.keys().cloned());
        symbols.extend(self.syntax_information.keys().cloned());
        symbols.extend(
            self.message_definitions
                .keys()
                .map(|key| key.symbol.clone()),
        );

        symbols
    }

    fn visible_contexts(&self) -> BTreeSet<&str> {
        self.context_path
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.current_context.as_str()))
            .collect()
    }
}

fn empty_list_expr() -> Expr {
    Expr::call(system_expr("List"), Vec::new())
}

fn string_list_expr(items: Vec<String>) -> Expr {
    Expr::call(
        system_expr("List"),
        items.into_iter().map(Expr::string).collect(),
    )
}

fn compile_names_pattern(expr: &Expr) -> Option<NamesPattern> {
    if let Expr::Atom(Atom::String(value)) = expr {
        return Some(NamesPattern::Wildcard(value.clone()));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "RegularExpression") && args.len() == 1 {
        let Expr::Atom(Atom::String(value)) = &args[0] else {
            return None;
        };

        return compile_names_regex_pattern(value).map(NamesPattern::Regex);
    }

    if is_system_symbol_name(head, "StringExpression") && !args.is_empty() {
        return args
            .iter()
            .map(compile_names_pattern)
            .collect::<Option<Vec<_>>>()
            .map(NamesPattern::Sequence);
    }

    if is_system_symbol_name(head, "Alternatives") && !args.is_empty() {
        return args
            .iter()
            .map(compile_names_pattern)
            .collect::<Option<Vec<_>>>()
            .map(NamesPattern::Alternatives);
    }

    None
}

fn names_pattern_mentions_context(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(Atom::String(value)) => value.contains('`'),
        Expr::Call { args, .. } => args.iter().any(names_pattern_mentions_context),
        _ => false,
    }
}

fn names_pattern_matches(pattern: &NamesPattern, candidate: &str) -> bool {
    let haystack = candidate.chars().collect::<Vec<_>>();
    names_pattern_match_ends(pattern, &haystack, 0)
        .into_iter()
        .any(|end| end == haystack.len())
}

fn names_pattern_match_ends(pattern: &NamesPattern, haystack: &[char], start: usize) -> Vec<usize> {
    match pattern {
        NamesPattern::Wildcard(pattern) => wildcard_match_ends(pattern, haystack, start),
        NamesPattern::Regex(pattern) => {
            if start == 0 && regex_pattern_matches(pattern, haystack) {
                vec![haystack.len()]
            } else {
                Vec::new()
            }
        }
        NamesPattern::Sequence(parts) => {
            let mut current = vec![start];

            for part in parts {
                let mut next = Vec::new();

                for end in current {
                    next.extend(names_pattern_match_ends(part, haystack, end));
                }

                next.sort_unstable();
                next.dedup();

                if next.is_empty() {
                    return next;
                }

                current = next;
            }

            current
        }
        NamesPattern::Alternatives(parts) => {
            let mut ends = Vec::new();

            for part in parts {
                ends.extend(names_pattern_match_ends(part, haystack, start));
            }

            ends.sort_unstable();
            ends.dedup();
            ends
        }
    }
}

fn wildcard_match_ends(pattern: &str, haystack: &[char], start: usize) -> Vec<usize> {
    fn recur(
        pattern: &[char],
        pi: usize,
        haystack: &[char],
        hi: usize,
        memo: &mut BTreeMap<(usize, usize), Vec<usize>>,
    ) -> Vec<usize> {
        if let Some(cached) = memo.get(&(pi, hi)) {
            return cached.clone();
        }

        let result = if pi == pattern.len() {
            vec![hi]
        } else if pattern[pi] == '*' {
            let mut ends = Vec::new();

            for next_hi in hi..=haystack.len() {
                ends.extend(recur(pattern, pi + 1, haystack, next_hi, memo));
            }

            ends.sort_unstable();
            ends.dedup();
            ends
        } else if hi < haystack.len() && pattern[pi] == haystack[hi] {
            recur(pattern, pi + 1, haystack, hi + 1, memo)
        } else {
            Vec::new()
        };

        memo.insert((pi, hi), result.clone());
        result
    }

    let pattern = pattern.chars().collect::<Vec<_>>();
    let mut memo = BTreeMap::new();
    recur(&pattern, 0, haystack, start, &mut memo)
}

fn compile_names_regex_pattern(raw: &str) -> Option<RegexPattern> {
    let chars = strip_full_match_anchors(raw).chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        let atom = parse_regex_atom(&chars, &mut index)?;
        let quantifier = if index < chars.len() {
            match chars[index] {
                '*' => {
                    index += 1;
                    RegexQuantifier::ZeroOrMore
                }
                '+' => {
                    index += 1;
                    RegexQuantifier::OneOrMore
                }
                '?' => {
                    index += 1;
                    RegexQuantifier::ZeroOrOne
                }
                _ => RegexQuantifier::One,
            }
        } else {
            RegexQuantifier::One
        };

        tokens.push(RegexToken { atom, quantifier });
    }

    Some(RegexPattern { tokens })
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

fn parse_regex_atom(chars: &[char], index: &mut usize) -> Option<RegexAtom> {
    let ch = *chars.get(*index)?;
    *index += 1;

    match ch {
        '.' => Some(RegexAtom::AnyChar),
        '\\' => parse_regex_escape(chars, index),
        '[' => parse_regex_class(chars, index),
        '(' | ')' | '|' | '{' | '}' | '*' | '+' | '?' => None,
        _ => Some(RegexAtom::Literal(ch)),
    }
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

fn regex_pattern_matches(pattern: &RegexPattern, haystack: &[char]) -> bool {
    fn recur(
        tokens: &[RegexToken],
        token_index: usize,
        haystack: &[char],
        hay_index: usize,
        memo: &mut BTreeMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(cached) = memo.get(&(token_index, hay_index)) {
            return *cached;
        }

        let result = if token_index == tokens.len() {
            hay_index == haystack.len()
        } else {
            let token = &tokens[token_index];
            match token.quantifier {
                RegexQuantifier::One => {
                    hay_index < haystack.len()
                        && regex_atom_matches(&token.atom, haystack[hay_index])
                        && recur(tokens, token_index + 1, haystack, hay_index + 1, memo)
                }
                RegexQuantifier::ZeroOrOne => {
                    recur(tokens, token_index + 1, haystack, hay_index, memo)
                        || (hay_index < haystack.len()
                            && regex_atom_matches(&token.atom, haystack[hay_index])
                            && recur(tokens, token_index + 1, haystack, hay_index + 1, memo))
                }
                RegexQuantifier::ZeroOrMore => {
                    if recur(tokens, token_index + 1, haystack, hay_index, memo) {
                        true
                    } else {
                        let mut end = hay_index;
                        let mut matched = false;

                        while end < haystack.len() && regex_atom_matches(&token.atom, haystack[end])
                        {
                            matched = true;
                            end += 1;

                            if recur(tokens, token_index + 1, haystack, end, memo) {
                                break;
                            }
                        }

                        matched && recur(tokens, token_index + 1, haystack, end, memo)
                    }
                }
                RegexQuantifier::OneOrMore => {
                    if hay_index >= haystack.len()
                        || !regex_atom_matches(&token.atom, haystack[hay_index])
                    {
                        false
                    } else {
                        let mut end = hay_index + 1;

                        loop {
                            if recur(tokens, token_index + 1, haystack, end, memo) {
                                break true;
                            }

                            if end >= haystack.len()
                                || !regex_atom_matches(&token.atom, haystack[end])
                            {
                                break false;
                            }

                            end += 1;
                        }
                    }
                }
            }
        };

        memo.insert((token_index, hay_index), result);
        result
    }

    let mut memo = BTreeMap::new();
    recur(&pattern.tokens, 0, haystack, 0, &mut memo)
}

fn regex_atom_matches(atom: &RegexAtom, ch: char) -> bool {
    match atom {
        RegexAtom::Literal(expected) => *expected == ch,
        RegexAtom::AnyChar => true,
        RegexAtom::Digit => ch.is_ascii_digit(),
        RegexAtom::Word => ch.is_ascii_alphanumeric() || ch == '_',
        RegexAtom::Whitespace => ch.is_whitespace(),
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
