use super::*;

pub(crate) fn eval_parse_args(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [items] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ParseArgs", args, 1));
    };
    let Some(items) = expr_as_string_list(items) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(parse_args(items).into_expr())
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

fn parse_args(items: Vec<String>) -> ParsedArgs {
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
        } else {
            parsed.positional.push(item);
        }
    }

    parsed
}
