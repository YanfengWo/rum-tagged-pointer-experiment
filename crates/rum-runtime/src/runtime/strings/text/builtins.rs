use super::diacritics::remove_diacritics_value;
use super::render::{ToStringFormat, render_to_string_expr};
use super::values::{
    character_code_expr, character_counts_expr, expr_as_nested_string_list,
    single_code_point_string, string_pad_left_value, text_words_value,
};
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
            ToStringFormat::OutputLike,
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

        let count = character_range_count(start, end);
        self.enforce_active_memory_limit_for_projected_bytes(character_range_memory_proxy_bytes(
            count,
        ))?;
        self.enforce_generated_element_limit_for_count(count)?;

        let mut items = Vec::with_capacity(count);
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
            Some(form) if is_system_symbol_name(form, "InputForm") => ToStringFormat::InputForm,
            Some(form) if is_system_symbol_name(form, "OutputForm") => ToStringFormat::OutputLike,
            Some(form) if is_system_symbol_name(form, "StandardForm") => ToStringFormat::OutputLike,
            Some(form) if is_system_symbol_name(form, "FullForm") => ToStringFormat::FullForm,
            Some(form) if is_system_symbol_name(form, "TraditionalForm") => {
                ToStringFormat::OutputLike
            }
            Some(_) => return Ok(Expr::call(head.clone(), args.to_vec())),
            None => ToStringFormat::OutputLike,
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

        let output_bytes = generated_string_bytes(value, count);
        self.enforce_active_memory_limit_for_projected_bytes(generated_string_memory_proxy_bytes(
            output_bytes,
        ))?;
        self.enforce_generated_element_limit_for_count(output_bytes)?;

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

fn character_range_count(start: char, end: char) -> usize {
    let raw_start = start as u32;
    let raw_end = end as u32;
    let raw_count = raw_end.saturating_sub(raw_start).saturating_add(1);
    let surrogate_start = 0xD800_u32;
    let surrogate_end = 0xDFFF_u32;
    let invalid_overlap = if raw_start <= surrogate_end && raw_end >= surrogate_start {
        let overlap_start = raw_start.max(surrogate_start);
        let overlap_end = raw_end.min(surrogate_end);
        overlap_end.saturating_sub(overlap_start).saturating_add(1)
    } else {
        0
    };

    usize::try_from(raw_count.saturating_sub(invalid_overlap)).unwrap_or(usize::MAX)
}

fn character_range_memory_proxy_bytes(count: usize) -> usize {
    let element_bytes = std::mem::size_of::<Expr>()
        .saturating_add(std::mem::size_of::<String>())
        .saturating_add(4);
    std::mem::size_of::<Expr>().saturating_add(count.saturating_mul(element_bytes))
}

fn generated_string_bytes(value: &str, count: usize) -> usize {
    value.len().saturating_mul(count)
}

fn generated_string_memory_proxy_bytes(bytes: usize) -> usize {
    std::mem::size_of::<Expr>().saturating_add(bytes)
}
