use super::*;

impl Session {
    pub(in crate::runtime) fn eval_string_join(
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

    pub(in crate::runtime) fn eval_string_length(
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

    pub(in crate::runtime) fn eval_to_upper_case(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("ToUpperCase", args, 1));
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(
                value
                    .chars()
                    .flat_map(char::to_uppercase)
                    .collect::<String>(),
            ));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values
                    .into_iter()
                    .map(|value| value.chars().flat_map(char::to_uppercase).collect())
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_riffle(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringRiffle", args, 1, 3));
        }

        if let Some(items) = expr_as_string_list(&args[0]) {
            let separator = match args.get(1) {
                Some(expr) => expr_as_string(expr).map(str::to_string),
                None => Some(" ".to_string()),
            };
            let Some(separator) = separator else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            if args.len() == 3 {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
            return Ok(Expr::string(items.join(&separator)));
        }

        if let Some(rows) = expr_as_nested_string_list(&args[0]) {
            let (within_row_separator, between_rows_separator) = match args {
                [_] => (" ".to_string(), "\n".to_string()),
                [_, item_separator] => {
                    let Some(separator) = expr_as_string(item_separator) else {
                        return Ok(Expr::call(head.clone(), args.to_vec()));
                    };
                    (separator.to_string(), separator.to_string())
                }
                [_, between_rows_separator, within_row_separator] => {
                    let (Some(between_rows_separator), Some(within_row_separator)) = (
                        expr_as_string(between_rows_separator),
                        expr_as_string(within_row_separator),
                    ) else {
                        return Ok(Expr::call(head.clone(), args.to_vec()));
                    };
                    (
                        within_row_separator.to_string(),
                        between_rows_separator.to_string(),
                    )
                }
                _ => unreachable!(),
            };

            let riffled_rows = rows
                .into_iter()
                .map(|row| row.join(&within_row_separator))
                .collect::<Vec<_>>();
            return Ok(Expr::string(riffled_rows.join(&between_rows_separator)));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_pad_left(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringPadLeft", args, 2, 3));
        }

        let Some(width) = expr_as_non_negative_usize(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let padding = match args.get(2) {
            Some(expr) => expr_as_string(expr).filter(|value| !value.is_empty()),
            None => Some(" "),
        };
        let Some(padding) = padding else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(string_pad_left_value(value, width, padding)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values
                    .into_iter()
                    .map(|value| string_pad_left_value(&value, width, padding))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_text_words(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("TextWords", args, 1));
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(string_list_expr(text_words_value(value)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| string_list_expr(text_words_value(&value)))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_to_character_code(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("ToCharacterCode", args, 1));
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(character_code_expr(value));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| character_code_expr(&value))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_to_expression(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ToExpression", args, 1, 3));
        }

        let Some(source) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(format) = args.get(1) {
            let supported = is_system_symbol_name(format, "InputForm")
                || is_system_symbol_name(format, "StandardForm");
            if !supported {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
        }

        let syntax = parse_expr(source)?;
        let lowered = self.lower_syntax_expr(&syntax)?;
        let evaluated = self.evaluate(lowered);
        let value = self.suspend_with_frame(evaluated, |_, value| Ok(value))?;

        if let Some(wrapper) = args.get(2) {
            let wrapped = Expr::call(wrapper.clone(), vec![value]);
            let wrapped_result = self.evaluate(wrapped);
            return self.suspend_with_frame(wrapped_result, |_, value| Ok(value));
        }

        Ok(value)
    }

    pub(in crate::runtime) fn eval_syntax_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("SyntaxQ", args, 1));
        }

        let Some(source) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(bool_expr(parse_expr(source).is_ok()))
    }

    pub(in crate::runtime) fn eval_text_string(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("TextString", args, 1));
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(value.to_string()));
        }

        Ok(Expr::string(render_to_string_expr(
            &args[0],
            ToStringFormat::InputLike,
        )))
    }

    pub(in crate::runtime) fn eval_character_counts(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("CharacterCounts", args, 1));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(character_counts_expr(value))
    }

    pub(in crate::runtime) fn eval_remove_diacritics(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("RemoveDiacritics", args, 1));
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(remove_diacritics_value(value)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values
                    .into_iter()
                    .map(|value| remove_diacritics_value(&value))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_characters(
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

    pub(in crate::runtime) fn eval_character_range(
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

    pub(in crate::runtime) fn eval_to_string(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ToString", args, 1, 2));
        }

        let format = match args.get(1) {
            Some(form) if is_system_symbol_name(form, "InputForm") => ToStringFormat::InputLike,
            Some(form) if is_system_symbol_name(form, "OutputForm") => ToStringFormat::InputLike,
            Some(form) if is_system_symbol_name(form, "StandardForm") => ToStringFormat::InputLike,
            Some(form) if is_system_symbol_name(form, "FullForm") => ToStringFormat::FullForm,
            Some(form) if is_system_symbol_name(form, "TraditionalForm") => {
                ToStringFormat::InputLike
            }
            Some(_) => return Ok(Expr::call(head.clone(), args.to_vec())),
            None => ToStringFormat::InputLike,
        };

        Ok(Expr::string(render_to_string_expr(&args[0], format)))
    }

    pub(in crate::runtime) fn eval_string_repeat(
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

    pub(in crate::runtime) fn eval_letter_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(
            self.eval_string_value_predicate(head, args, "LetterQ", |value| {
                value.chars().all(char::is_alphabetic)
            }),
        )
    }

    pub(in crate::runtime) fn eval_digit_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(
            self.eval_string_value_predicate(head, args, "DigitQ", |value| {
                value.chars().all(|ch| ch.is_ascii_digit())
            }),
        )
    }

    fn eval_string_value_predicate(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        predicate: fn(&str) -> bool,
    ) -> Expr {
        if args.len() != 1 {
            return self.invalid_call_with_exact_arg_count(symbol, args, 1);
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return bool_expr(predicate(value));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return list_expr(
                values
                    .into_iter()
                    .map(|value| bool_expr(predicate(&value)))
                    .collect(),
            );
        }

        Expr::call(head.clone(), args.to_vec())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToStringFormat {
    FullForm,
    InputLike,
}

const PREC_LOWEST: u8 = 0;
const PREC_RULE: u8 = 10;
const PREC_STRING_EXPRESSION: u8 = 15;
const PREC_PLUS: u8 = 20;
const PREC_TIMES: u8 = 30;
const PREC_POWER: u8 = 40;
const PREC_CALL: u8 = 90;

pub(in crate::runtime) fn render_input_like_expr(expr: &Expr) -> String {
    render_to_string_expr(expr, ToStringFormat::InputLike)
}

fn render_to_string_expr(expr: &Expr, format: ToStringFormat) -> String {
    render_to_string_expr_with_precedence(expr, format, PREC_LOWEST)
}

fn render_to_string_expr_with_precedence(
    expr: &Expr,
    format: ToStringFormat,
    parent_precedence: u8,
) -> String {
    if format == ToStringFormat::FullForm {
        return expr.to_string();
    }

    match expr {
        Expr::Atom(_) => expr.to_string(),
        Expr::Call { head, args } => {
            if is_system_symbol_name(head, heads::messages::MESSAGE_NAME) && args.len() == 2 {
                return expr.to_string();
            }

            if let Some(entries) = association_entries_from_expr(expr) {
                let rendered = format!(
                    "<|{}|>",
                    entries
                        .iter()
                        .map(|(key, value)| format!(
                            "{} -> {}",
                            render_to_string_expr_with_precedence(key, format, PREC_RULE + 1),
                            render_to_string_expr_with_precedence(value, format, PREC_RULE + 1)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, heads::structure::LIST) {
                return format!(
                    "{{{}}}",
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_LOWEST))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            if is_system_symbol_name(head, heads::functions::SLOT) && args.len() == 1 {
                if let Some(index) = expr_as_non_negative_usize(&args[0]) {
                    return if index == 1 {
                        "#".to_string()
                    } else {
                        format!("#{index}")
                    };
                }
            }

            if is_system_symbol_name(head, heads::functions::SLOT_SEQUENCE) && args.len() == 1 {
                if let Some(index) = expr_as_non_negative_usize(&args[0]) {
                    return if index == 1 {
                        "##".to_string()
                    } else {
                        format!("##{index}")
                    };
                }
            }

            if is_system_symbol_name(head, heads::functions::FUNCTION) && args.len() == 1 {
                let rendered = format!(
                    "{} &",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if (is_system_symbol_name(head, "HoldForm")
                || is_system_symbol_name(head, "HoldCompleteForm")
                || is_system_symbol_name(head, "Defer")
                || is_system_symbol_name(head, "Unevaluated"))
                && args.len() == 1
            {
                return render_to_string_expr_with_precedence(&args[0], format, parent_precedence);
            }

            if is_system_symbol_name(head, "Part") && args.len() >= 2 {
                let rendered = format!(
                    "{}[[{}]]",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_CALL),
                    args[1..]
                        .iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_LOWEST))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, heads::arithmetic::PLUS) && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_PLUS))
                        .collect::<Vec<_>>()
                        .join(" + "),
                    PREC_PLUS,
                    parent_precedence,
                );
            }

            if is_system_symbol_name(head, heads::arithmetic::TIMES) && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_TIMES))
                        .collect::<Vec<_>>()
                        .join(" * "),
                    PREC_TIMES,
                    parent_precedence,
                );
            }

            if is_system_symbol_name(head, heads::arithmetic::POWER) && args.len() == 2 {
                let rendered = format!(
                    "{} ^ {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_POWER),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_POWER)
                );
                return wrap_rendered(rendered, PREC_POWER, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::SET) && args.len() == 2 {
                let rendered = format!(
                    "{} = {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::SET_DELAYED) && args.len() == 2 {
                let rendered = format!(
                    "{} := {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::UNSET) && args.len() == 1 {
                let rendered = format!(
                    "{} =.",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::RULE) && args.len() == 2 {
                let rendered = format!(
                    "{} -> {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, "Map") && args.len() == 2 {
                let rendered = format!(
                    "{} /@ {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, "Apply") && args.len() == 2 {
                let rendered = format!(
                    "{} @@ {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, "MapApply") && args.len() == 2 {
                let rendered = format!(
                    "{} @@@ {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::REPLACE_ALL) && args.len() == 2 {
                let rendered = format!(
                    "{} /. {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::REPLACE_REPEATED) && args.len() == 2 {
                let rendered = format!(
                    "{} //. {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::patterns::ALTERNATIVES) && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_RULE))
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREC_RULE,
                    parent_precedence,
                );
            }

            if is_system_symbol_name(head, "StringExpression") && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| {
                            render_to_string_expr_with_precedence(
                                arg,
                                format,
                                PREC_STRING_EXPRESSION,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" ~~ "),
                    PREC_STRING_EXPRESSION,
                    parent_precedence,
                );
            }

            if is_system_symbol_name(head, "Repeated") && args.len() == 1 {
                let rendered = format!(
                    "{}..",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_CALL)
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, "RepeatedNull") && args.len() == 1 {
                let rendered = format!(
                    "{}...",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_CALL)
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, heads::patterns::CONDITION) && args.len() == 2 {
                let rendered = format!(
                    "{} /; {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::patterns::PATTERN_TEST) && args.len() == 2 {
                let rendered = format!(
                    "{}?{}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_CALL),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_CALL)
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, heads::patterns::PATTERN) && args.len() == 2 {
                if let Some(shorthand) = render_named_blank_pattern(expr, format) {
                    return shorthand;
                }

                let rendered = format!(
                    "{} : {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if let Some(blank) = render_blank_pattern(head, args, format) {
                return blank;
            }

            if is_system_symbol_name(head, heads::patterns::OPTIONAL) && !args.is_empty() {
                let pattern = render_to_string_expr_with_precedence(&args[0], format, PREC_CALL);
                let rendered = match args.as_slice() {
                    [_] => format!("{pattern}."),
                    [_, default] => format!(
                        "{pattern} : {}",
                        render_to_string_expr_with_precedence(default, format, PREC_RULE + 1)
                    ),
                    _ => expr.to_string(),
                };
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::RULE_DELAYED) && args.len() == 2 {
                let rendered = format!(
                    "{} :> {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::structure::SPAN) {
                let rendered = match args.as_slice() {
                    [start] if is_system_symbol_name(start, heads::structure::ALL) => {
                        ";;".to_string()
                    }
                    [start] => format!(
                        "{} ;;",
                        render_to_string_expr_with_precedence(start, format, PREC_RULE + 1)
                    ),
                    [start, end] if is_system_symbol_name(start, heads::structure::ALL) => format!(
                        ";; {}",
                        render_to_string_expr_with_precedence(end, format, PREC_RULE + 1)
                    ),
                    [start, end] if is_system_symbol_name(end, heads::structure::ALL) => format!(
                        "{} ;;",
                        render_to_string_expr_with_precedence(start, format, PREC_RULE + 1)
                    ),
                    [start, end] => format!(
                        "{} ;; {}",
                        render_to_string_expr_with_precedence(start, format, PREC_RULE + 1),
                        render_to_string_expr_with_precedence(end, format, PREC_RULE + 1)
                    ),
                    [start, end, step]
                        if is_system_symbol_name(start, heads::structure::ALL)
                            && is_system_symbol_name(end, heads::structure::ALL) =>
                    {
                        format!(
                            ";; ;; {}",
                            render_to_string_expr_with_precedence(step, format, PREC_RULE + 1)
                        )
                    }
                    [start, end, step] => format!(
                        "{} ;; {} ;; {}",
                        render_to_string_expr_with_precedence(start, format, PREC_RULE + 1),
                        render_to_string_expr_with_precedence(end, format, PREC_RULE + 1),
                        render_to_string_expr_with_precedence(step, format, PREC_RULE + 1)
                    ),
                    _ => expr.to_string(),
                };
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, "CompoundExpression") && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_RULE))
                        .collect::<Vec<_>>()
                        .join("; "),
                    PREC_RULE,
                    parent_precedence,
                );
            }

            let rendered_head = render_to_string_expr_with_precedence(head, format, PREC_CALL);
            format!(
                "{}[{}]",
                rendered_head,
                args.iter()
                    .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_LOWEST))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

fn wrap_rendered(rendered: String, own_precedence: u8, parent_precedence: u8) -> String {
    if own_precedence < parent_precedence {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn render_named_blank_pattern(expr: &Expr, format: ToStringFormat) -> Option<String> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, heads::patterns::PATTERN) || args.len() != 2 {
        return None;
    }

    let name = render_to_string_expr_with_precedence(&args[0], format, PREC_CALL);
    let Expr::Call {
        head: blank_head,
        args: blank_args,
    } = &args[1]
    else {
        return None;
    };

    let prefix = if is_system_symbol_name(blank_head, heads::patterns::BLANK) {
        "_"
    } else if is_system_symbol_name(blank_head, heads::patterns::BLANK_SEQUENCE) {
        "__"
    } else if is_system_symbol_name(blank_head, heads::patterns::BLANK_NULL_SEQUENCE) {
        "___"
    } else {
        return None;
    };

    let suffix = match blank_args.as_slice() {
        [] => String::new(),
        [constraint] => render_to_string_expr_with_precedence(constraint, format, PREC_CALL),
        _ => return None,
    };

    Some(format!("{name}{prefix}{suffix}"))
}

fn render_blank_pattern(head: &Expr, args: &[Expr], format: ToStringFormat) -> Option<String> {
    let prefix = if is_system_symbol_name(head, heads::patterns::BLANK) {
        "_"
    } else if is_system_symbol_name(head, heads::patterns::BLANK_SEQUENCE) {
        "__"
    } else if is_system_symbol_name(head, heads::patterns::BLANK_NULL_SEQUENCE) {
        "___"
    } else {
        return None;
    };

    let suffix = match args {
        [] => String::new(),
        [constraint] => render_to_string_expr_with_precedence(constraint, format, PREC_CALL),
        _ => return None,
    };

    Some(format!("{prefix}{suffix}"))
}

fn expr_as_nested_string_list(expr: &Expr) -> Option<Vec<Vec<String>>> {
    let items = call_items_with_head(expr, &system_expr("List"))?;
    items.iter().map(expr_as_string_list).collect()
}

fn single_code_point_string(expr: &Expr) -> Option<char> {
    let value = expr_as_string(expr)?;
    let mut chars = value.chars();
    let ch = chars.next()?;
    chars.next().is_none().then_some(ch)
}

fn string_pad_left_value(value: &str, width: usize, padding: &str) -> String {
    let value_len = value.chars().count();
    if value_len >= width {
        return value.to_string();
    }

    let needed = width - value_len;
    let pad_chars = padding.chars().collect::<Vec<_>>();
    let mut prefix = String::new();
    for index in 0..needed {
        prefix.push(pad_chars[index % pad_chars.len()]);
    }
    prefix.push_str(value);
    prefix
}

fn text_words_value(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn character_code_expr(value: &str) -> Expr {
    list_expr(
        value
            .chars()
            .map(|ch| Expr::integer(BigInt::from(ch as u32)))
            .collect(),
    )
}

fn character_counts_expr(value: &str) -> Expr {
    let mut counts = BTreeMap::<String, usize>::new();
    for ch in value.chars() {
        *counts.entry(ch.to_string()).or_insert(0) += 1;
    }

    let mut entries = counts.into_iter().collect::<Vec<_>>();
    entries.sort_by(|(lhs_key, lhs_count), (rhs_key, rhs_count)| {
        rhs_count.cmp(lhs_count).then_with(|| rhs_key.cmp(lhs_key))
    });

    render_association(
        entries
            .into_iter()
            .map(|(key, count)| (Expr::string(key), Expr::integer(BigInt::from(count as u64))))
            .collect(),
    )
}

pub(super) fn remove_diacritics_value(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        output.push_str(&strip_diacritic_char(ch));
    }
    output
}

fn strip_diacritic_char(ch: char) -> String {
    match ch {
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' | 'Ǎ' | 'Ȁ' | 'Ȃ' => {
            "A".into()
        }
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' | 'ȁ' | 'ȃ' => {
            "a".into()
        }
        'Ç' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' => "C".into(),
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c".into(),
        'Ð' | 'Ď' | 'Đ' => "D".into(),
        'ð' | 'ď' | 'đ' => "d".into(),
        'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' | 'Ȅ' | 'Ȇ' => "E".into(),
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'ȅ' | 'ȇ' => "e".into(),
        'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' => "G".into(),
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g".into(),
        'Ĥ' | 'Ħ' => "H".into(),
        'ĥ' | 'ħ' => "h".into(),
        'Ì' | 'Í' | 'Î' | 'Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' | 'Ǐ' | 'Ȉ' | 'Ȋ' => {
            "I".into()
        }
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' | 'ǐ' | 'ȉ' | 'ȋ' => {
            "i".into()
        }
        'Ĵ' => "J".into(),
        'ĵ' => "j".into(),
        'Ķ' => "K".into(),
        'ķ' => "k".into(),
        'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' => "L".into(),
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l".into(),
        'Ñ' | 'Ń' | 'Ņ' | 'Ň' => "N".into(),
        'ñ' | 'ń' | 'ņ' | 'ň' => "n".into(),
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' | 'Ǒ' | 'Ȍ' | 'Ȏ' => {
            "O".into()
        }
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' | 'ǒ' | 'ȍ' | 'ȏ' => {
            "o".into()
        }
        'Ŕ' | 'Ŗ' | 'Ř' => "R".into(),
        'ŕ' | 'ŗ' | 'ř' => "r".into(),
        'Ś' | 'Ŝ' | 'Ş' | 'Š' => "S".into(),
        'ś' | 'ŝ' | 'ş' | 'š' => "s".into(),
        'Ţ' | 'Ť' | 'Ŧ' => "T".into(),
        'ţ' | 'ť' | 'ŧ' => "t".into(),
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' | 'Ǔ' | 'Ȕ' | 'Ȗ' => {
            "U".into()
        }
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ǔ' | 'ȕ' | 'ȗ' => {
            "u".into()
        }
        'Ý' | 'Ÿ' | 'Ŷ' => "Y".into(),
        'ý' | 'ÿ' | 'ŷ' => "y".into(),
        'Ź' | 'Ż' | 'Ž' => "Z".into(),
        'ź' | 'ż' | 'ž' => "z".into(),
        'Æ' => "AE".into(),
        'æ' => "ae".into(),
        'Œ' => "OE".into(),
        'œ' => "oe".into(),
        _ => ch.to_string(),
    }
}
