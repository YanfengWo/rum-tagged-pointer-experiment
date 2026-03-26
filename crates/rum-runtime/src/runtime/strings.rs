use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringTakeDropSpec {
    Count(isize),
    InclusiveRange(isize, isize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StringPattern {
    Literal(Vec<char>),
    Sequence(Vec<StringPattern>),
    Alternatives(Vec<StringPattern>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StringBoundarySpec {
    Whitespace,
    Patterns(Vec<StringPattern>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringSplitLimit {
    Unlimited,
    MaxSubstrings(usize),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactStringReplacementRule {
    pattern: StringPattern,
    replacement: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactStringCaseRule {
    pattern: StringPattern,
    output: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExactStringCaseSpec {
    Patterns(Vec<StringPattern>),
    Rules(Vec<ExactStringCaseRule>),
}

impl Session {
    pub(super) fn eval_string_join(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let mut pieces = Vec::new();

        if args.len() == 1 {
            if let Some(items) = expr_as_string_list(&args[0]) {
                pieces = items;
            } else {
                for arg in args {
                    let Some(value) = expr_as_string(arg) else {
                        return Ok(Expr::call(head.clone(), args.to_vec()));
                    };
                    pieces.push(value.to_string());
                }
            }
        } else {
            for arg in args {
                let Some(value) = expr_as_string(arg) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                pieces.push(value.to_string());
            }
        }

        Ok(Expr::string(pieces.concat()))
    }

    pub(super) fn eval_string_length(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("StringLength", args, 1));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(BigInt::from(value.chars().count() as u64)))
    }

    pub(super) fn eval_characters(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Characters", args, 1));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(list_expr(
            value
                .chars()
                .map(|ch| Expr::string(ch.to_string()))
                .collect(),
        ))
    }

    pub(super) fn eval_character_range(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("CharacterRange", args, 2));
        }

        let (Some(start), Some(end)) = (
            single_code_point_string(&args[0]),
            single_code_point_string(&args[1]),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if start > end {
            return Ok(list_expr(Vec::new()));
        }

        let mut items = Vec::new();
        for value in (start as u32)..=(end as u32) {
            let Some(ch) = char::from_u32(value) else {
                continue;
            };
            items.push(Expr::string(ch.to_string()));
        }

        Ok(list_expr(items))
    }

    pub(super) fn eval_to_string(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("ToString", args, 1));
        }

        Ok(Expr::string(args[0].to_string()))
    }

    pub(super) fn eval_string_repeat(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("StringRepeat", args, 2));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(count) = expr_as_non_negative_usize(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::string(value.repeat(count)))
    }

    pub(super) fn eval_string_contains_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_string_pattern_predicate(head, args, "StringContainsQ", string_contains_q))
    }

    pub(super) fn eval_string_starts_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_string_pattern_predicate(head, args, "StringStartsQ", string_starts_q))
    }

    pub(super) fn eval_string_ends_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_string_pattern_predicate(head, args, "StringEndsQ", string_ends_q))
    }

    pub(super) fn eval_string_position(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringPosition", args, 2, 3));
        }

        let Some(pattern) = parse_string_pattern(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let max_results = if args.len() == 3 {
            let Some(value) = expr_as_non_negative_usize(&args[2]) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            Some(value)
        } else {
            None
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(string_position_expr(value, &pattern, max_results));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| string_position_expr(&value, &pattern, max_results))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_string_split(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringSplit", args, 1, 3));
        }

        let delimiter_spec = match args.get(1) {
            Some(expr) => parse_string_boundary_spec(expr),
            None => Some(StringBoundarySpec::Whitespace),
        };
        let Some(delimiter_spec) = delimiter_spec else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let limit = match args.get(2) {
            Some(expr) => parse_string_split_limit(expr),
            None => Some(StringSplitLimit::Unlimited),
        };
        let Some(limit) = limit else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(string_list_expr(split_string_value(
                value,
                &delimiter_spec,
                limit,
            )));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        string_list_expr(split_string_value(&value, &delimiter_spec, limit))
                    })
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_string_trim(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringTrim", args, 1, 2));
        }

        let trim_spec = match args.get(1) {
            Some(expr) => parse_string_boundary_spec(expr),
            None => Some(StringBoundarySpec::Whitespace),
        };
        let Some(trim_spec) = trim_spec else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(trim_string_value(value, &trim_spec)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values
                    .into_iter()
                    .map(|value| trim_string_value(&value, &trim_spec))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_string_replace(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringReplace", args, 2, 3));
        }

        let Some(rules) = parse_exact_string_replacement_rules(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let max_replacements = match args.get(2) {
            Some(expr) => expr_as_non_negative_usize(expr),
            None => Some(usize::MAX),
        };
        let Some(max_replacements) = max_replacements else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(replace_string_value(
                value,
                &rules,
                max_replacements,
            )));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values
                    .into_iter()
                    .map(|value| replace_string_value(&value, &rules, max_replacements))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_string_cases(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringCases", args, 2, 3));
        }

        let Some(spec) = parse_exact_string_case_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let max_matches = match args.get(2) {
            Some(expr) => expr_as_non_negative_usize(expr),
            None => Some(usize::MAX),
        };
        let Some(max_matches) = max_matches else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(list_expr(collect_string_cases(value, &spec, max_matches)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| list_expr(collect_string_cases(&value, &spec, max_matches)))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_string_take(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("StringTake", args, 2));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(spec) = parse_string_take_drop_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let chars = value.chars().collect::<Vec<_>>();
        let taken = match spec {
            StringTakeDropSpec::Count(count) => take_chars_by_count(&chars, count),
            StringTakeDropSpec::InclusiveRange(start, end) => {
                let Some((start, end)) = normalize_clamped_inclusive_range(start, end, chars.len())
                else {
                    return Ok(Expr::string(String::new()));
                };
                chars[start..=end].iter().collect::<String>()
            }
        };

        Ok(Expr::string(taken))
    }

    pub(super) fn eval_string_drop(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("StringDrop", args, 2));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(spec) = parse_string_take_drop_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let chars = value.chars().collect::<Vec<_>>();
        let dropped = match spec {
            StringTakeDropSpec::Count(count) => drop_chars_by_count(&chars, count),
            StringTakeDropSpec::InclusiveRange(start, end) => {
                let Some((start, end)) = normalize_clamped_inclusive_range(start, end, chars.len())
                else {
                    return Ok(Expr::string(value.to_string()));
                };

                let mut output = String::new();
                output.extend(chars[..start].iter());
                output.extend(chars[end + 1..].iter());
                output
            }
        };

        Ok(Expr::string(dropped))
    }

    pub(super) fn eval_string_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("StringPart", args, 2));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let chars = value.chars().collect::<Vec<_>>();
        let Some(specs) = parse_string_part_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut output = String::new();
        for spec in specs {
            let Some(index) = normalize_exact_position(spec, chars.len()) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            output.push(chars[index]);
        }

        Ok(Expr::string(output))
    }

    fn eval_string_pattern_predicate(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        predicate: fn(&[char], &StringPattern) -> bool,
    ) -> Expr {
        if args.len() != 2 {
            return self.invalid_call_with_exact_arg_count(symbol, args, 2);
        }

        let Some(pattern) = parse_string_pattern(&args[1]) else {
            return Expr::call(head.clone(), args.to_vec());
        };

        if let Some(value) = expr_as_string(&args[0]) {
            let chars = value.chars().collect::<Vec<_>>();
            return bool_expr(predicate(&chars, &pattern));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        let chars = value.chars().collect::<Vec<_>>();
                        bool_expr(predicate(&chars, &pattern))
                    })
                    .collect(),
            );
        }

        Expr::call(head.clone(), args.to_vec())
    }
}

fn parse_string_take_drop_spec(expr: &Expr) -> Option<StringTakeDropSpec> {
    if let Some(count) = expr_as_signed_isize(expr) {
        return Some(StringTakeDropSpec::Count(count));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") || args.len() != 2 {
        return None;
    }

    Some(StringTakeDropSpec::InclusiveRange(
        expr_as_signed_isize(&args[0])?,
        expr_as_signed_isize(&args[1])?,
    ))
}

fn parse_string_part_spec(expr: &Expr) -> Option<Vec<isize>> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(vec![index]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(expr_as_signed_isize).collect()
}

fn parse_string_boundary_spec(expr: &Expr) -> Option<StringBoundarySpec> {
    Some(StringBoundarySpec::Patterns(parse_string_patterns(expr)?))
}

fn parse_string_split_limit(expr: &Expr) -> Option<StringSplitLimit> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(StringSplitLimit::All);
    }

    expr_as_non_negative_usize(expr).map(StringSplitLimit::MaxSubstrings)
}

fn parse_string_patterns(expr: &Expr) -> Option<Vec<StringPattern>> {
    if let Some(pattern) = parse_string_pattern(expr) {
        return Some(vec![pattern]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") || args.is_empty() {
        return None;
    }

    args.iter()
        .map(parse_string_pattern)
        .collect::<Option<Vec<_>>>()
}

fn parse_string_pattern(expr: &Expr) -> Option<StringPattern> {
    if let Some(chars) = expr_as_non_empty_char_vec(expr) {
        return Some(StringPattern::Literal(chars));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "StringExpression") && !args.is_empty() {
        return args
            .iter()
            .map(parse_string_pattern)
            .collect::<Option<Vec<_>>>()
            .map(StringPattern::Sequence);
    }

    if is_system_symbol_name(head, "Alternatives") && !args.is_empty() {
        return args
            .iter()
            .map(parse_string_pattern)
            .collect::<Option<Vec<_>>>()
            .map(StringPattern::Alternatives);
    }

    None
}

fn parse_exact_string_replacement_rules(expr: &Expr) -> Option<Vec<ExactStringReplacementRule>> {
    if let Some(rule) = parse_exact_string_replacement_rule(expr) {
        return Some(vec![rule]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") || args.is_empty() {
        return None;
    }

    args.iter()
        .map(parse_exact_string_replacement_rule)
        .collect()
}

fn parse_exact_string_replacement_rule(expr: &Expr) -> Option<ExactStringReplacementRule> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return None;
    }

    Some(ExactStringReplacementRule {
        pattern: parse_string_pattern(&args[0])?,
        replacement: expr_as_string(&args[1])?.to_string(),
    })
}

fn parse_exact_string_case_spec(expr: &Expr) -> Option<ExactStringCaseSpec> {
    if let Some(patterns) = parse_string_patterns(expr) {
        return Some(ExactStringCaseSpec::Patterns(patterns));
    }
    if let Some(rule) = parse_exact_string_case_rule(expr) {
        return Some(ExactStringCaseSpec::Rules(vec![rule]));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") || args.is_empty() {
        return None;
    }

    if let Some(patterns) = args
        .iter()
        .map(parse_string_pattern)
        .collect::<Option<Vec<_>>>()
    {
        return Some(ExactStringCaseSpec::Patterns(patterns));
    }

    args.iter()
        .map(parse_exact_string_case_rule)
        .collect::<Option<Vec<_>>>()
        .map(ExactStringCaseSpec::Rules)
}

fn parse_exact_string_case_rule(expr: &Expr) -> Option<ExactStringCaseRule> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return None;
    }

    Some(ExactStringCaseRule {
        pattern: parse_string_pattern(&args[0])?,
        output: args[1].clone(),
    })
}

fn expr_as_non_empty_char_vec(expr: &Expr) -> Option<Vec<char>> {
    let chars = expr_as_string(expr)?.chars().collect::<Vec<_>>();
    (!chars.is_empty()).then_some(chars)
}

fn single_code_point_string(expr: &Expr) -> Option<char> {
    let value = expr_as_string(expr)?;
    let mut chars = value.chars();
    let ch = chars.next()?;
    chars.next().is_none().then_some(ch)
}

fn normalize_exact_position(position: isize, len: usize) -> Option<usize> {
    if position == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position - 1
    } else {
        len + position
    };

    (0..len)
        .contains(&normalized)
        .then_some(normalized as usize)
}

fn normalize_clamped_endpoint(position: isize, len: usize) -> Option<usize> {
    if position == 0 || len == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position
    } else {
        len + position + 1
    };

    let clamped = normalized.clamp(1, len);
    Some((clamped - 1) as usize)
}

fn normalize_clamped_inclusive_range(
    start: isize,
    end: isize,
    len: usize,
) -> Option<(usize, usize)> {
    let start = normalize_clamped_endpoint(start, len)?;
    let end = normalize_clamped_endpoint(end, len)?;
    (start <= end).then_some((start, end))
}

fn take_chars_by_count(chars: &[char], count: isize) -> String {
    if count == 0 {
        return String::new();
    }

    let len = chars.len();
    let magnitude = count.unsigned_abs().min(len);

    if count > 0 {
        chars[..magnitude].iter().collect()
    } else {
        chars[len - magnitude..].iter().collect()
    }
}

fn drop_chars_by_count(chars: &[char], count: isize) -> String {
    if count == 0 {
        return chars.iter().collect();
    }

    let len = chars.len();
    let magnitude = count.unsigned_abs().min(len);

    if count > 0 {
        chars[magnitude..].iter().collect()
    } else {
        chars[..len - magnitude].iter().collect()
    }
}

fn bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

fn string_contains_q(haystack: &[char], pattern: &StringPattern) -> bool {
    (0..haystack.len()).any(|start| best_pattern_match_end(pattern, haystack, start).is_some())
}

fn string_starts_q(haystack: &[char], pattern: &StringPattern) -> bool {
    best_pattern_match_end(pattern, haystack, 0).is_some()
}

fn string_ends_q(haystack: &[char], pattern: &StringPattern) -> bool {
    (0..haystack.len())
        .any(|start| best_pattern_match_end(pattern, haystack, start) == Some(haystack.len()))
}

fn split_string_value(
    value: &str,
    spec: &StringBoundarySpec,
    limit: StringSplitLimit,
) -> Vec<String> {
    let chars = value.chars().collect::<Vec<_>>();
    match spec {
        StringBoundarySpec::Whitespace => split_by_whitespace(&chars, limit),
        StringBoundarySpec::Patterns(patterns) => split_by_string_patterns(&chars, patterns, limit),
    }
}

fn trim_string_value(value: &str, spec: &StringBoundarySpec) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    match spec {
        StringBoundarySpec::Whitespace => trim_whitespace_chars(&chars),
        StringBoundarySpec::Patterns(patterns) => trim_string_patterns(&chars, patterns),
    }
}

fn replace_string_value(
    value: &str,
    rules: &[ExactStringReplacementRule],
    max_replacements: usize,
) -> String {
    if max_replacements == 0 {
        return value.to_string();
    }

    let chars = value.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0;
    let mut replacements = 0usize;

    while index < chars.len() {
        if replacements >= max_replacements {
            output.extend(chars[index..].iter());
            break;
        }

        if let Some((rule, end)) = rules.iter().find_map(|rule| {
            best_pattern_match_end(&rule.pattern, &chars, index).map(|end| (rule, end))
        }) {
            output.push_str(&rule.replacement);
            index = end;
            replacements += 1;
        } else {
            output.push(chars[index]);
            index += 1;
        }
    }

    output
}

fn collect_string_cases(value: &str, spec: &ExactStringCaseSpec, max_matches: usize) -> Vec<Expr> {
    if max_matches == 0 {
        return Vec::new();
    }

    let chars = value.chars().collect::<Vec<_>>();
    let mut index = 0;
    let mut matches = Vec::new();

    while index < chars.len() && matches.len() < max_matches {
        match spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, end)) = best_pattern_match_at(patterns, &chars, index)
                {
                    matches.push(Expr::string(pattern_match_text(&chars[index..end])));
                    index = end;
                } else {
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((rule, end)) = rules.iter().find_map(|rule| {
                    best_pattern_match_end(&rule.pattern, &chars, index).map(|end| (rule, end))
                }) {
                    matches.push(rule.output.clone());
                    index = end;
                } else {
                    index += 1;
                }
            }
        }
    }

    matches
}

fn string_position_expr(value: &str, pattern: &StringPattern, max_results: Option<usize>) -> Expr {
    let haystack = value.chars().collect::<Vec<_>>();
    let positions = string_pattern_positions(&haystack, pattern, max_results)
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
) -> impl Iterator<Item = (usize, usize)> + 'a {
    let max_results = max_results.unwrap_or(usize::MAX);

    if haystack.is_empty() || max_results == 0 {
        return Vec::new().into_iter();
    }

    let mut positions = Vec::new();
    for start in 0..haystack.len() {
        if let Some(end) = best_pattern_match_end(pattern, haystack, start) {
            positions.push((start + 1, end));
            if positions.len() >= max_results {
                break;
            }
        }
    }

    positions.into_iter()
}

fn split_by_whitespace(chars: &[char], limit: StringSplitLimit) -> Vec<String> {
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

fn split_by_string_patterns(
    chars: &[char],
    patterns: &[StringPattern],
    limit: StringSplitLimit,
) -> Vec<String> {
    if chars.is_empty() {
        return if matches!(limit, StringSplitLimit::All) {
            vec![String::new()]
        } else {
            Vec::new()
        };
    }

    let mut parts = Vec::new();
    let mut segment_start = 0usize;
    let mut index = 0usize;
    let include_edge_empties = matches!(limit, StringSplitLimit::All);

    while index < chars.len() {
        if matches!(limit, StringSplitLimit::MaxSubstrings(max) if parts.len() + 1 >= max) {
            break;
        }

        let Some(end) = best_pattern_match_at(patterns, chars, index).map(|(_, end)| end) else {
            index += 1;
            continue;
        };

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

fn trim_whitespace_chars(chars: &[char]) -> String {
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

fn trim_string_patterns(chars: &[char], patterns: &[StringPattern]) -> String {
    let mut start = 0usize;
    let mut end = chars.len();

    while start < end {
        let Some(match_end) =
            best_pattern_match_at(patterns, &chars[..end], start).map(|(_, end)| end)
        else {
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

fn pattern_match_text(chars: &[char]) -> String {
    chars.iter().collect()
}

fn best_pattern_match_at(
    patterns: &[StringPattern],
    chars: &[char],
    start: usize,
) -> Option<(usize, usize)> {
    patterns
        .iter()
        .enumerate()
        .filter_map(|(index, pattern)| {
            best_pattern_match_end(pattern, chars, start).map(|end| (index, end))
        })
        .max_by_key(|(index, end)| (*end, usize::MAX - *index))
}

fn best_pattern_suffix_start(patterns: &[StringPattern], chars: &[char]) -> Option<usize> {
    let end = chars.len();
    (0..end)
        .filter(|start| {
            patterns
                .iter()
                .any(|pattern| best_pattern_match_end(pattern, chars, *start) == Some(end))
        })
        .min()
}

fn best_pattern_match_end(pattern: &StringPattern, chars: &[char], start: usize) -> Option<usize> {
    pattern_match_end_positions(pattern, chars, start)
        .into_iter()
        .max()
}

fn pattern_match_end_positions(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
) -> Vec<usize> {
    if start > chars.len() {
        return Vec::new();
    }

    match pattern {
        StringPattern::Literal(literal) => chars[start..]
            .starts_with(literal.as_slice())
            .then_some(start + literal.len())
            .into_iter()
            .collect(),
        StringPattern::Sequence(parts) => {
            let mut positions = vec![start];

            for part in parts {
                let mut next_positions = Vec::new();
                for position in positions {
                    next_positions.extend(pattern_match_end_positions(part, chars, position));
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
                positions.extend(pattern_match_end_positions(alternative, chars, start));
            }
            positions.sort_unstable();
            positions.dedup();
            positions
        }
    }
}

fn is_string_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r')
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn string_list_expr(items: Vec<String>) -> Expr {
    list_expr(items.into_iter().map(Expr::string).collect())
}
