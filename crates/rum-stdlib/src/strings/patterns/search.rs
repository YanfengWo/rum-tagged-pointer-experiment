use super::matcher::{
    collect_string_cases, split_string_value, string_contains_q, string_count_matches,
    string_delete_matches, string_ends_q, string_free_q, string_match_q, string_position_expr,
    string_starts_q, trim_string_value,
};
use super::*;
use rules::{
    collect_string_cases_values_with_rule_evaluation_expr,
    collect_string_cases_with_rule_evaluation, replace_string_value_with_rules,
    replace_string_values_with_rules_expr,
};

mod rules;

pub(crate) fn eval_string_contains_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_string_pattern_predicate(session, head, args, "StringContainsQ", string_contains_q)
}

pub(crate) fn eval_string_starts_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_string_pattern_predicate(session, head, args, "StringStartsQ", string_starts_q)
}

pub(crate) fn eval_string_ends_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_string_pattern_predicate(session, head, args, "StringEndsQ", string_ends_q)
}

pub(crate) fn eval_string_position(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !(2..=4).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringPosition", args, 2, 4));
    }

    let Some(pattern) = parse_string_pattern(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let option_exprs =
        session.merged_option_exprs_for_system_symbol("StringPosition", &args[2..])?;
    let Some((max_results, config)) = parse_string_position_args(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(string_position_expr(value, &pattern, max_results, config));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| string_position_expr(&value, &pattern, max_results, config))
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_split(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !(1..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringSplit", args, 1, 3));
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
                .map(|value| string_list_expr(split_string_value(&value, &delimiter_spec, limit)))
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_trim(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringTrim", args, 1, 2));
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

pub(crate) fn eval_string_replace(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !(2..=4).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringReplace", args, 2, 4));
    }

    let Some(rules) = parse_exact_string_replacement_rules(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs =
        session.merged_option_exprs_for_system_symbol("StringReplace", &args[2..])?;
    let Some((max_replacements, config)) = parse_string_match_limit_and_options(&option_exprs)
    else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        let result =
            replace_string_value_with_rules(session, value, &rules, max_replacements, config);
        return session.drive_internal_value::<String, Expr, _>(result, |_, rendered| {
            Ok(Expr::string(rendered))
        });
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return replace_string_values_with_rules_expr(
            session,
            values,
            &rules,
            max_replacements,
            config,
        );
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_cases(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !(2..=4).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringCases", args, 2, 4));
    }

    let Some(spec) = parse_exact_string_case_spec(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol("StringCases", &args[2..])?;
    let Some((max_matches, config)) = parse_string_match_limit_and_options(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        if matches!(&spec, ExactStringCaseSpec::Rules(_)) {
            let result = collect_string_cases_with_rule_evaluation(
                session,
                value,
                &spec,
                max_matches,
                config,
            );
            return session.drive_internal_value::<Vec<Expr>, Expr, _>(result, |_, items| {
                Ok(list_expr(items))
            });
        }

        return Ok(list_expr(collect_string_cases(
            value,
            &spec,
            max_matches,
            config,
        )));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        if matches!(&spec, ExactStringCaseSpec::Rules(_)) {
            return collect_string_cases_values_with_rule_evaluation_expr(
                session,
                values,
                &spec,
                max_matches,
                config,
            );
        }

        let mut outputs = Vec::with_capacity(values.len());
        for value in values {
            let items = collect_string_cases(&value, &spec, max_matches, config);
            outputs.push(list_expr(items));
        }
        return Ok(list_expr(outputs));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_count(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() < 2 {
        return Ok(session.invalid_call_with_arg_range("StringCount", args, 2, 4));
    }

    let Some(patterns) = parse_string_patterns(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol("StringCount", &args[2..])?;
    let Some(config) = parse_string_count_options(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(Expr::integer(BigInt::from(
            string_count_matches(value, &patterns, config) as u64,
        )));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| {
                    Expr::integer(BigInt::from(
                        string_count_matches(&value, &patterns, config) as u64,
                    ))
                })
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_delete(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() == 1 {
        return Ok(string_unary_operator_form_slot_first(
            "StringDelete",
            &args[0],
        ));
    }
    if args.len() < 2 {
        return Ok(session.invalid_call_with_arg_range("StringDelete", args, 1, 3));
    }

    let Some(patterns) = parse_string_patterns(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol("StringDelete", &args[2..])?;
    let Some(config) = parse_string_delete_options(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(Expr::string(string_delete_matches(
            value, &patterns, config,
        )));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(string_list_expr(
            values
                .into_iter()
                .map(|value| string_delete_matches(&value, &patterns, config))
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_apply(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [operator] => Ok(string_unary_operator_form("StringApply", operator)),
        [operator, input] => {
            if let Some(value) = expr_as_string(input) {
                return string_apply_value(session, head, operator, value);
            }

            if let Some(values) = expr_as_string_list(input) {
                return resume_string_apply_values(session, head, operator, values);
            }

            Ok(Expr::call(head.clone(), args.to_vec()))
        }
        _ => Ok(session.invalid_call_with_arg_range("StringApply", args, 1, 2)),
    }
}

pub(crate) fn eval_string_match_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_string_patterns_predicate(session, head, args, "StringMatchQ", string_match_q)
}

pub(crate) fn eval_string_free_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_string_patterns_predicate(session, head, args, "StringFreeQ", string_free_q)
}

fn eval_string_pattern_predicate(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    predicate: fn(&[char], &StringPattern, StringPatternMatchConfig) -> bool,
) -> Result<Expr, EvalError> {
    if !(2..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range(symbol, args, 2, 3));
    }

    let Some(pattern) = parse_string_pattern(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol(symbol, &args[2..])?;
    let Some(config) = parse_string_match_config(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        let chars = value.chars().collect::<Vec<_>>();
        return Ok(bool_expr(predicate(&chars, &pattern, config)));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| {
                    let chars = value.chars().collect::<Vec<_>>();
                    bool_expr(predicate(&chars, &pattern, config))
                })
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

fn eval_string_patterns_predicate(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    predicate: fn(&[char], &[StringPattern], StringPatternMatchConfig) -> bool,
) -> Result<Expr, EvalError> {
    if !(2..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range(symbol, args, 2, 3));
    }

    let Some(patterns) = parse_string_patterns(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol(symbol, &args[2..])?;
    let Some(config) = parse_string_match_config(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        let chars = value.chars().collect::<Vec<_>>();
        return Ok(bool_expr(predicate(&chars, &patterns, config)));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| {
                    let chars = value.chars().collect::<Vec<_>>();
                    bool_expr(predicate(&chars, &patterns, config))
                })
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}
