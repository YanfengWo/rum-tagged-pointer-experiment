use super::regex::{RegexPattern, compile_names_regex_pattern, regex_pattern_matches};
use super::*;

const EXTRA_SYSTEM_NAME_SYMBOLS: &[&str] = &[
    "Alternatives",
    "DialogProlog",
    "DialogSymbols",
    "Flat",
    "EvaluationMonitor",
    "Heads",
    "HoldAll",
    "HoldAllComplete",
    "HoldFirst",
    "HoldRest",
    "Listable",
    "OneIdentity",
    "OptionsPattern",
    "Orderless",
    "OrderlessPatternSequence",
    "PatternTest",
    "PatternSequence",
    "Protected",
    "RegularExpression",
    "SequenceHold",
    "Temporary",
    "Verbatim",
    "StepMonitor",
    "StringExpression",
];

#[derive(Clone, Debug, PartialEq, Eq)]
enum NamesPattern {
    Wildcard(String),
    Regex(RegexPattern),
    Sequence(Vec<NamesPattern>),
    Alternatives(Vec<NamesPattern>),
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

impl Session {
    pub(in crate::runtime) fn eval_names(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn resolve_names_matching_expr(
        &self,
        expr: &Expr,
    ) -> Option<Vec<Symbol>> {
        let (patterns, explicit_context) = self.compile_names_patterns(expr)?;
        Some(
            self.names_matching_patterns(&patterns, explicit_context)
                .into_iter()
                .filter_map(|name| self.resolve_symbol_name(&name, false))
                .collect(),
        )
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
        symbols.extend(self.declared_stubs.iter().cloned());
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
