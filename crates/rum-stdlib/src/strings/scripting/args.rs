use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_parse_args(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [items, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_arg_range("ParseArgs", args, 1, usize::MAX));
    };
    let Some(items) = expr_as_string_list(items) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let Some(options) = parse_parse_args_options(session, option_args)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(parse_args(items, options.short_options).into_expr())
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ParsedArgs {
    positional: Vec<String>,
    options: Vec<(String, String)>,
    flags: Vec<String>,
}

impl ParsedArgs {
    fn into_expr(self) -> Expr {
        render_association(vec![
            (
                Expr::string("Positional"),
                string_list_expr(self.positional),
            ),
            (
                Expr::string("Options"),
                render_association(
                    self.options
                        .into_iter()
                        .map(|(key, value)| (Expr::string(key), Expr::string(value)))
                        .collect(),
                ),
            ),
            (Expr::string("Flags"), string_list_expr(self.flags)),
        ])
    }
}

struct ParseArgsOptions {
    short_options: bool,
}

fn parse_parse_args_options(
    session: &mut Session,
    explicit: &[Expr],
) -> Result<Option<ParseArgsOptions>, EvalError> {
    let option_exprs = session.merged_option_exprs_for_system_symbol("ParseArgs", explicit)?;
    let mut options = ParseArgsOptions {
        short_options: false,
    };

    for expr in option_exprs {
        let Some((name, value)) = option_rule(&expr) else {
            return Ok(None);
        };
        match name {
            "ShortOptions" => {
                let Some(value) = bool_option_value(value) else {
                    return Ok(None);
                };
                options.short_options = value;
            }
            _ => return Ok(None),
        }
    }

    Ok(Some(options))
}

fn parse_args(items: Vec<String>, short_options: bool) -> ParsedArgs {
    let mut parsed = ParsedArgs::default();
    let mut parse_options = true;

    for item in items {
        if parse_options && item == "--" {
            parse_options = false;
        } else if parse_options && item.starts_with("--") {
            let item = &item[2..];
            if let Some((key, value)) = item.split_once('=') {
                parsed.options.push((key.to_string(), value.to_string()));
            } else {
                parsed.flags.push(item.to_string());
            }
        } else if parse_options && short_options && item.starts_with('-') && item != "-" {
            for flag in item[1..].chars() {
                parsed.flags.push(flag.to_string());
            }
        } else {
            parsed.positional.push(item);
        }
    }

    parsed
}

fn option_rule(expr: &Expr) -> Option<(&str, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }
    Some((option_name(&args[0])?, &args[1]))
}

fn option_name(expr: &Expr) -> Option<&str> {
    if let Some(name) = expr_as_string(expr) {
        return Some(name);
    }
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name()),
        _ => None,
    }
}

fn bool_option_value(expr: &Expr) -> Option<bool> {
    if is_system_symbol_name(expr, "True") {
        Some(true)
    } else if is_system_symbol_name(expr, "False") {
        Some(false)
    } else {
        None
    }
}
