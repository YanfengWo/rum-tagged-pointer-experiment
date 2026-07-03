use super::*;

pub(crate) fn eval_string_reverse(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("StringReverse", args, 1));
    }

    let Some(value) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(value.chars().rev().collect::<String>()))
}

pub(crate) fn eval_to_lower_case(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("ToLowerCase", args, 1));
    }

    let Some(value) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(
        value
            .chars()
            .flat_map(char::to_lowercase)
            .collect::<String>(),
    ))
}

pub(crate) fn eval_lower_case_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    Ok(eval_string_scalar_predicate(
        session,
        head,
        args,
        "LowerCaseQ",
        |value| value.chars().all(char::is_lowercase),
    ))
}

pub(crate) fn eval_upper_case_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    Ok(eval_string_scalar_predicate(
        session,
        head,
        args,
        "UpperCaseQ",
        |value| value.chars().all(char::is_uppercase),
    ))
}

pub(crate) fn eval_capitalize(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_string_or_string_list_map(session, head, args, "Capitalize", capitalize_value)
}

pub(crate) fn eval_decapitalize(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_string_or_string_list_map(session, head, args, "Decapitalize", decapitalize_value)
}

fn eval_string_scalar_predicate(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    predicate: fn(&str) -> bool,
) -> Expr {
    if args.len() != 1 {
        return session.invalid_call_with_exact_arg_count(symbol, args, 1);
    }

    let Some(value) = expr_as_string(&args[0]) else {
        return Expr::call(head.clone(), args.to_vec());
    };

    bool_expr(predicate(value))
}

fn eval_string_or_string_list_map(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    mapper: fn(&str) -> String,
) -> Result<Expr, EvalError> {
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count(symbol, args, 1));
    }

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(Expr::string(mapper(value)));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(string_list_expr(
            values.into_iter().map(|value| mapper(&value)).collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

fn capitalize_value(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    first.to_uppercase().chain(chars).collect()
}

fn decapitalize_value(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    first.to_lowercase().chain(chars).collect()
}
