use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum QuoteMode {
    None,
    Single,
    Double,
}

pub(crate) fn eval_shell_split(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [text] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ShellSplit", args, 1));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match shell_split(text) {
        Ok(parts) => Ok(string_list_expr(parts)),
        Err(message) => Ok(failed_with_message(
            session,
            "ShellSplit",
            "syntax",
            message,
        )),
    }
}

pub(crate) fn eval_shell_quote(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [text] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ShellQuote", args, 1));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(shell_quote(text)))
}

fn shell_split(text: &str) -> Result<Vec<String>, String> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = QuoteMode::None;
    let mut in_word = false;
    let mut index = 0usize;

    while index < chars.len() {
        let ch = chars[index];
        match quote {
            QuoteMode::None => match ch {
                '\'' => {
                    quote = QuoteMode::Single;
                    in_word = true;
                    index += 1;
                }
                '"' => {
                    quote = QuoteMode::Double;
                    in_word = true;
                    index += 1;
                }
                '\\' => {
                    let Some(next) = chars.get(index + 1).copied() else {
                        return Err("ShellSplit ended after a backslash escape.".to_string());
                    };
                    current.push(next);
                    in_word = true;
                    index += 2;
                }
                whitespace if whitespace.is_whitespace() => {
                    if in_word {
                        words.push(current);
                        current = String::new();
                        in_word = false;
                    }
                    index += 1;
                }
                other => {
                    current.push(other);
                    in_word = true;
                    index += 1;
                }
            },
            QuoteMode::Single => match ch {
                '\'' => {
                    quote = QuoteMode::None;
                    index += 1;
                }
                other => {
                    current.push(other);
                    index += 1;
                }
            },
            QuoteMode::Double => match ch {
                '"' => {
                    quote = QuoteMode::None;
                    index += 1;
                }
                '\\' => {
                    let Some(next) = chars.get(index + 1).copied() else {
                        return Err("ShellSplit ended after a backslash escape.".to_string());
                    };
                    current.push(next);
                    index += 2;
                }
                other => {
                    current.push(other);
                    index += 1;
                }
            },
        }
    }

    match quote {
        QuoteMode::None => {
            if in_word {
                words.push(current);
            }
            Ok(words)
        }
        QuoteMode::Single => Err("ShellSplit found an unterminated single quote.".to_string()),
        QuoteMode::Double => Err("ShellSplit found an unterminated double quote.".to_string()),
    }
}

fn shell_quote(text: &str) -> String {
    if text.is_empty() {
        return "''".to_string();
    }

    format!("'{}'", text.replace('\'', "'\\''"))
}
