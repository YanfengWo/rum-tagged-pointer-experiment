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
    "PatternTest",
    "Protected",
    "RegularExpression",
    "SequenceHold",
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegexPattern {
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
            .unwrap_or_else(|| default_system_syntax_information(symbol))
    }

    pub(super) fn set_syntax_information_for_symbol(&mut self, symbol: Symbol, syntax: Expr) {
        if syntax == default_system_syntax_information(&symbol) {
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

    pub(super) fn resolve_names_matching_expr(&self, expr: &Expr) -> Option<Vec<Symbol>> {
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

fn empty_list_expr() -> Expr {
    Expr::call(system_expr("List"), Vec::new())
}

fn syntax_rule(name: &str, value: Expr) -> Expr {
    Expr::call(system_expr("Rule"), vec![Expr::string(name), value])
}

fn syntax_argument_pattern(items: Vec<Expr>) -> Expr {
    syntax_rule("ArgumentsPattern", Expr::call(system_expr("List"), items))
}

fn blank_expr() -> Expr {
    Expr::call(system_expr("Blank"), Vec::new())
}

fn blank_null_sequence_expr() -> Expr {
    Expr::call(system_expr("BlankNullSequence"), Vec::new())
}

fn blank_sequence_expr() -> Expr {
    Expr::call(system_expr("BlankSequence"), Vec::new())
}

fn optional_blank_expr() -> Expr {
    Expr::call(system_expr("Optional"), vec![blank_expr()])
}

fn optional_blank_association_expr() -> Expr {
    Expr::call(
        system_expr("Optional"),
        vec![Expr::call(
            system_expr("Blank"),
            vec![system_expr("Association")],
        )],
    )
}

fn default_system_syntax_information(symbol: &Symbol) -> Expr {
    if symbol.context() != SYSTEM_CONTEXT {
        return empty_list_expr();
    }

    let rules = match symbol.name() {
        "Begin" | "FindFile" | "Get" | "Names" | "Options" | "Protect" | "SyntaxInformation"
        | "Unprotect" => {
            vec![syntax_argument_pattern(vec![blank_expr()])]
        }
        "BeginPackage" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_null_sequence_expr(),
        ])],
        "End" | "EndPackage" => vec![syntax_argument_pattern(Vec::new())],
        "SetOptions" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_null_sequence_expr(),
        ])],
        "OptionValue" => vec![syntax_argument_pattern(vec![blank_null_sequence_expr()])],
        "ArgumentsOptions" | "CheckArguments" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            optional_blank_association_expr(),
        ])],
        "Trace" | "TracePrint" => vec![
            syntax_argument_pattern(vec![blank_expr(), optional_blank_expr()]),
            syntax_rule("OptionNames", string_list_expr(vec!["Heads".to_string()])),
        ],
        "RepeatedTiming" => vec![syntax_argument_pattern(vec![blank_expr()])],
        "Short" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
        ])],
        "On" | "Off" => vec![syntax_argument_pattern(vec![blank_sequence_expr()])],
        "Sow" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
        ])],
        "Reap" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
        ])],
        "TimeConstrained" | "MemoryConstrained" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            optional_blank_expr(),
        ])],
        "Dialog" => vec![
            syntax_argument_pattern(vec![optional_blank_expr(), system_expr("OptionsPattern")]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "DialogProlog".to_string(),
                    "DialogSymbols".to_string(),
                ]),
            ),
        ],
        "MemoryInUse" | "Stack" => vec![syntax_argument_pattern(Vec::new())],
        "Monitor" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "EvaluationMonitor".to_string(),
                    "StepMonitor".to_string(),
                ]),
            ),
        ],
        "EvaluationData" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
        ])],
        "TestCreate" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            optional_blank_expr(),
        ])],
        "TestReport" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                optional_blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "SameTest".to_string(),
                    "TimeConstraint".to_string(),
                    "MemoryConstraint".to_string(),
                ]),
            ),
        ],
        "StringCount" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "StringDelete" | "StringContainsQ" | "StringStartsQ" | "StringEndsQ" | "StringMatchQ"
        | "StringFreeQ" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "StringPosition" | "StringCases" | "StringReplace" | "Cases" | "Count" | "MemberQ"
        | "FreeQ" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "DeleteCases" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                optional_blank_expr(),
                optional_blank_expr(),
                optional_blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule("OptionNames", string_list_expr(vec!["Heads".to_string()])),
        ],
        "Position" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "Level" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            optional_blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "Depth" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "CheckAbort" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "ArrayFilter" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "UpSet" | "UpSetDelayed" => {
            vec![syntax_argument_pattern(vec![blank_expr(), blank_expr()])]
        }
        "MessageName" => vec![syntax_argument_pattern(vec![blank_expr(), blank_expr()])],
        "Message" => vec![syntax_argument_pattern(vec![blank_sequence_expr()])],
        "Needs" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
        ])],
        "Module" | "Block" => vec![syntax_argument_pattern(vec![
            Expr::call(system_expr("List"), vec![blank_sequence_expr()]),
            blank_expr(),
        ])],
        "Function" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
        ])],
        "Table" | "Do" => vec![
            syntax_argument_pattern(vec![blank_expr(), blank_sequence_expr()]),
            syntax_rule(
                "LocalVariables",
                Expr::call(
                    system_expr("List"),
                    vec![
                        Expr::string("Table"),
                        Expr::call(
                            system_expr("List"),
                            vec![Expr::integer(BigInt::from(2)), system_expr("Infinity")],
                        ),
                    ],
                ),
            ),
        ],
        "DeclarePackage" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            Expr::call(system_expr("List"), vec![blank_sequence_expr()]),
        ])],
        "Save" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "ExcludedContexts".to_string(),
                    "IncludedContexts".to_string(),
                ]),
            ),
        ],
        "DumpSave" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "CompressionLevel".to_string(),
                    "SymbolAttributes".to_string(),
                ]),
            ),
        ],
        "Once" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                optional_blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "ExpirationDate".to_string(),
                    "PersistenceTime".to_string(),
                ]),
            ),
        ],
        _ => Vec::new(),
    };

    if rules.is_empty() {
        empty_list_expr()
    } else {
        Expr::call(system_expr("List"), rules)
    }
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

fn regex_pattern_matches(pattern: &RegexPattern, haystack: &[char]) -> bool {
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
