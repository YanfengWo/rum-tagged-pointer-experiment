use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringTakeDropSpec {
    Count(isize),
    InclusiveRange(isize, isize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StringPattern {
    Literal(Vec<char>),
    Whitespace,
    NumberString,
    Except {
        excluded: Box<StringPattern>,
        inner: Option<Box<StringPattern>>,
    },
    StartOfString,
    EndOfString,
    Repeated {
        pattern: Box<StringPattern>,
        min: usize,
        max: Option<usize>,
    },
    Shortest(Box<StringPattern>),
    Longest(Box<StringPattern>),
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringPatternMatchConfig {
    ignore_case: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringDeleteConfig {
    ignore_case: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringCountOverlaps {
    NonOverlapping,
    Overlapping,
}

impl Default for StringCountOverlaps {
    fn default() -> Self {
        Self::NonOverlapping
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StringCountConfig {
    ignore_case: bool,
    overlaps: StringCountOverlaps,
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum TemplateSegment {
    Text(String),
    PositionalSlot,
    NamedSlot(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AlignmentBlock<T> {
    Common(Vec<T>),
    Different { left: Vec<T>, right: Vec<T> },
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

    pub(super) fn eval_to_upper_case(
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

    pub(super) fn eval_string_form(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(self.invalid_call_with_arg_range("StringForm", args, 1, usize::MAX));
        }

        let Some(template) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::string(string_form_value(template, &args[1..])))
    }

    pub(super) fn eval_string_riffle(
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

    pub(super) fn eval_string_pad_left(
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

    pub(super) fn eval_string_template(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("StringTemplate", args, 1));
        }

        let Some(template) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_template_object(template))
    }

    pub(super) fn eval_template_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [template] => Ok(Expr::call(
                system_expr("Function"),
                vec![Expr::call(
                    system_expr("TemplateApply"),
                    vec![template.clone(), string_slot_expr()],
                )],
            )),
            [template, data] => {
                let Some(template) = expr_as_template_source(template) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                let Some(rendered) = template_apply_value(template, data) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                Ok(Expr::string(rendered))
            }
            _ => Ok(self.invalid_call_with_arg_range("TemplateApply", args, 1, 2)),
        }
    }

    pub(super) fn eval_text_words(
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

    pub(super) fn eval_to_character_code(
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

    pub(super) fn eval_to_expression(
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

    pub(super) fn eval_string_to_byte_array(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("StringToByteArray", args, 1));
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(byte_array_expr(value.as_bytes()));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| byte_array_expr(value.as_bytes()))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_hash(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Hash", args, 1, 3));
        }

        if let Some(algorithm_expr) = args.get(1) {
            let Some(algorithm) = expr_as_string(algorithm_expr) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            if algorithm != "SHA256" {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
        }

        let digest = sha256_bytes(&hash_input_bytes(&args[0]));
        let integer_value = sha256_integer(&digest);

        match args.get(2).and_then(|format| expr_as_string(format)) {
            None => Ok(Expr::integer(integer_value)),
            Some("Integer") => Ok(Expr::integer(integer_value)),
            Some("HexString") => Ok(Expr::string(bytes_to_hex(&digest))),
            Some("DecimalString") => Ok(Expr::string(sha256_decimal_string(&integer_value))),
            Some("ByteArray") => Ok(byte_array_expr(&digest)),
            Some(_) => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(super) fn eval_compress(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Compress", args, 1));
        }

        Ok(Expr::string(compress_envelope(&args[0])))
    }

    pub(super) fn eval_encrypt(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Encrypt", args, 2));
        }

        let Some(key) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let (payload, original_form) = expr_transport_bytes(&args[1]);
        let ciphertext = xor_with_sha256_keystream(key.as_bytes(), &payload);
        Ok(encrypted_object_expr(
            &ciphertext,
            &sha256_bytes(key.as_bytes()),
            original_form,
        ))
    }

    pub(super) fn eval_nearest(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Nearest", args, 2, 3));
        }

        let count = match args.get(2) {
            Some(expr) => {
                let Some(value) = expr_as_non_negative_usize(expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                Some(value)
            }
            None => None,
        };

        if let Some(target) = expr_as_string(&args[1]) {
            if let Some(values) = expr_as_string_list(&args[0]) {
                return Ok(string_list_expr(nearest_strings(&values, target, count)));
            }

            if let Some((keys, values)) = parse_string_rule_dataset(&args[0]) {
                return Ok(list_expr(nearest_values_for_strings(
                    &keys, &values, target, count,
                )));
            }
        }

        if let Some(target) = expr_as_exact_rational(&args[1]) {
            if let Some((keys, values)) = parse_numeric_dataset(&args[0]) {
                return Ok(list_expr(nearest_values_for_numbers(
                    &keys, &values, &target, count,
                )));
            }
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_pluralize(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [word_expr] => {
                let Some(word) = expr_as_string(word_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                Ok(Expr::string(pluralize_english_word(word)))
            }
            [word_expr, count_expr] => {
                let Some(word) = expr_as_string(word_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                let Some(count) = expr_as_exact_rational(count_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                if count == Rational::one() {
                    Ok(Expr::string(word.to_string()))
                } else {
                    Ok(Expr::string(pluralize_english_word(word)))
                }
            }
            _ => Ok(self.invalid_call_with_arg_range("Pluralize", args, 1, 2)),
        }
    }

    pub(super) fn eval_syntax_q(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("SyntaxQ", args, 1));
        }

        let Some(source) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(bool_expr(parse_expr(source).is_ok()))
    }

    pub(super) fn eval_text_string(
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

        Ok(Expr::string(args[0].to_string()))
    }

    pub(super) fn eval_file_template_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [template_path] => Ok(Expr::call(
                system_expr("Function"),
                vec![Expr::call(
                    system_expr("FileTemplateApply"),
                    vec![template_path.clone(), string_slot_expr()],
                )],
            )),
            [template_path, data] => {
                let Some(path) = expr_as_string(template_path) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };

                let template_source = fs::read_to_string(path).map_err(|source| EvalError::Io {
                    path: path.to_string(),
                    source,
                })?;
                let Some(rendered) = template_apply_value(&template_source, data) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };

                let output_path = temp_rendered_template_path(Path::new(path));
                fs::write(&output_path, rendered).map_err(|source| EvalError::Io {
                    path: output_path.display().to_string(),
                    source,
                })?;

                Ok(Expr::string(output_path.display().to_string()))
            }
            _ => Ok(self.invalid_call_with_arg_range("FileTemplateApply", args, 1, 2)),
        }
    }

    pub(super) fn eval_character_counts(
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

    pub(super) fn eval_remove_diacritics(
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

    pub(super) fn eval_alphabet(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let supported = match args {
            [] => true,
            [arg] => expr_as_string(arg).is_some_and(|name| name == "English"),
            _ => false,
        };

        if !supported {
            return Ok(if args.len() <= 1 {
                Expr::call(head.clone(), args.to_vec())
            } else {
                self.invalid_call_with_arg_range("Alphabet", args, 0, 1)
            });
        }

        Ok(string_list_expr(english_alphabet()))
    }

    pub(super) fn eval_alphabetic_sort(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("AlphabeticSort", args, 1, 2));
        }

        if let Some(alphabet_name) = args.get(1).and_then(expr_as_string) {
            if alphabet_name != "English" {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
        } else if args.len() == 2 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let Some(values) = expr_as_string_list(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut values = values;
        values.sort_by_key(|value| alphabetic_sort_key(value));
        Ok(string_list_expr(values))
    }

    pub(super) fn eval_edit_distance(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("EditDistance", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            return Ok(Expr::integer(BigInt::from(
                levenshtein_distance(&lhs, &rhs) as u64,
            )));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            return Ok(Expr::integer(BigInt::from(
                levenshtein_distance(lhs, rhs) as u64
            )));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_hamming_distance(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("HammingDistance", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            let Some(distance) = hamming_distance(&lhs, &rhs) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            return Ok(Expr::integer(BigInt::from(distance as u64)));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            let Some(distance) = hamming_distance(lhs, rhs) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            return Ok(Expr::integer(BigInt::from(distance as u64)));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_longest_common_subsequence(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("LongestCommonSubsequence", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            let common = longest_common_contiguous_slice(&lhs, &rhs);
            return Ok(Expr::string(common.into_iter().collect::<String>()));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            return Ok(list_expr(longest_common_contiguous_slice(lhs, rhs)));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_sequence_alignment(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("SequenceAlignment", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            let alignment = global_alignment_blocks(&lhs, &rhs);
            return Ok(render_string_alignment_blocks(&alignment));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            let alignment = global_alignment_blocks(lhs, rhs);
            return Ok(render_expr_alignment_blocks(&alignment));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_diff(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Diff", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
            let lhs = lhs.chars().collect::<Vec<_>>();
            let rhs = rhs.chars().collect::<Vec<_>>();
            let alignment = global_alignment_blocks(&lhs, &rhs);
            return Ok(render_diff_object_from_chars(&alignment));
        }

        if let (Some(lhs), Some(rhs)) = (
            call_items_with_head(&args[0], &system_expr("List")),
            call_items_with_head(&args[1], &system_expr("List")),
        ) {
            let alignment = global_alignment_blocks(lhs, rhs);
            return Ok(render_diff_object_from_exprs(&alignment));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
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
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringPosition", args, 2, 4));
        }

        let Some(pattern) = parse_string_pattern(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some((max_results, config)) = parse_string_position_args(&args[2..]) else {
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
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringReplace", args, 2, 4));
        }

        let Some(rules) = parse_exact_string_replacement_rules(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some((max_replacements, config)) = parse_string_match_limit_and_options(&args[2..])
        else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(replace_string_value(
                value,
                &rules,
                max_replacements,
                config,
            )));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values
                    .into_iter()
                    .map(|value| replace_string_value(&value, &rules, max_replacements, config))
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
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringCases", args, 2, 4));
        }

        let Some(spec) = parse_exact_string_case_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some((max_matches, config)) = parse_string_match_limit_and_options(&args[2..]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(list_expr(collect_string_cases(
                value,
                &spec,
                max_matches,
                config,
            )));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        list_expr(collect_string_cases(&value, &spec, max_matches, config))
                    })
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

    pub(super) fn eval_letter_q(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        Ok(
            self.eval_string_value_predicate(head, args, "LetterQ", |value| {
                value.chars().all(char::is_alphabetic)
            }),
        )
    }

    pub(super) fn eval_digit_q(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        Ok(
            self.eval_string_value_predicate(head, args, "DigitQ", |value| {
                value.chars().all(|ch| ch.is_ascii_digit())
            }),
        )
    }

    pub(super) fn eval_string_take_drop(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("StringTakeDrop", args, 2));
        }

        let Some(spec) = parse_string_take_drop_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            let (taken, dropped) = string_take_drop_value(value, spec);
            return Ok(list_expr(vec![Expr::string(taken), Expr::string(dropped)]));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        let (taken, dropped) = string_take_drop_value(&value, spec);
                        list_expr(vec![Expr::string(taken), Expr::string(dropped)])
                    })
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_string_count(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_arg_range("StringCount", args, 2, 4));
        }

        let Some(patterns) = parse_string_patterns(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(config) = parse_string_count_options(&args[2..]) else {
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

    pub(super) fn eval_string_delete(
        &mut self,
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
            return Ok(self.invalid_call_with_arg_range("StringDelete", args, 1, 3));
        }

        let Some(patterns) = parse_string_patterns(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(config) = parse_string_delete_options(&args[2..]) else {
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

    pub(super) fn eval_string_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(string_unary_operator_form("StringApply", operator)),
            [operator, input] => {
                if let Some(value) = expr_as_string(input) {
                    return string_apply_value(
                        self,
                        operator,
                        value,
                        Expr::call(head.clone(), args.to_vec()),
                    );
                }

                if let Some(values) = expr_as_string_list(input) {
                    let mut outputs = Vec::with_capacity(values.len());
                    for value in values {
                        outputs.push(string_apply_value(
                            self,
                            operator,
                            &value,
                            Expr::call(
                                head.clone(),
                                vec![operator.clone(), Expr::string(value.clone())],
                            ),
                        )?);
                    }
                    return Ok(list_expr(outputs));
                }

                Ok(Expr::call(head.clone(), args.to_vec()))
            }
            _ => Ok(self.invalid_call_with_arg_range("StringApply", args, 1, 2)),
        }
    }

    pub(super) fn eval_string_match_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_string_patterns_predicate(head, args, "StringMatchQ", string_match_q))
    }

    pub(super) fn eval_string_free_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_string_patterns_predicate(head, args, "StringFreeQ", string_free_q))
    }

    fn eval_string_pattern_predicate(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        predicate: fn(&[char], &StringPattern, StringPatternMatchConfig) -> bool,
    ) -> Expr {
        if !(2..=3).contains(&args.len()) {
            return self.invalid_call_with_arg_range(symbol, args, 2, 3);
        }

        let Some(pattern) = parse_string_pattern(&args[1]) else {
            return Expr::call(head.clone(), args.to_vec());
        };
        let Some(config) = parse_string_match_config(&args[2..]) else {
            return Expr::call(head.clone(), args.to_vec());
        };

        if let Some(value) = expr_as_string(&args[0]) {
            let chars = value.chars().collect::<Vec<_>>();
            return bool_expr(predicate(&chars, &pattern, config));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        let chars = value.chars().collect::<Vec<_>>();
                        bool_expr(predicate(&chars, &pattern, config))
                    })
                    .collect(),
            );
        }

        Expr::call(head.clone(), args.to_vec())
    }

    fn eval_string_patterns_predicate(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        predicate: fn(&[char], &[StringPattern], StringPatternMatchConfig) -> bool,
    ) -> Expr {
        if !(2..=3).contains(&args.len()) {
            return self.invalid_call_with_arg_range(symbol, args, 2, 3);
        }

        let Some(patterns) = parse_string_patterns(&args[1]) else {
            return Expr::call(head.clone(), args.to_vec());
        };
        let Some(config) = parse_string_match_config(&args[2..]) else {
            return Expr::call(head.clone(), args.to_vec());
        };

        if let Some(value) = expr_as_string(&args[0]) {
            let chars = value.chars().collect::<Vec<_>>();
            return bool_expr(predicate(&chars, &patterns, config));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        let chars = value.chars().collect::<Vec<_>>();
                        bool_expr(predicate(&chars, &patterns, config))
                    })
                    .collect(),
            );
        }

        Expr::call(head.clone(), args.to_vec())
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

    if is_system_symbol_name(expr, "Whitespace") {
        return Some(StringPattern::Whitespace);
    }

    if is_system_symbol_name(expr, "NumberString") {
        return Some(StringPattern::NumberString);
    }

    if is_system_symbol_name(expr, "Except") {
        return None;
    }

    if is_system_symbol_name(expr, "StartOfString") {
        return Some(StringPattern::StartOfString);
    }

    if is_system_symbol_name(expr, "EndOfString") {
        return Some(StringPattern::EndOfString);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "Except") && args.len() == 1 {
        return Some(StringPattern::Except {
            excluded: Box::new(parse_string_pattern(&args[0])?),
            inner: None,
        });
    }

    if is_system_symbol_name(head, "Except") && args.len() == 2 {
        return Some(StringPattern::Except {
            excluded: Box::new(parse_string_pattern(&args[0])?),
            inner: Some(Box::new(parse_string_pattern(&args[1])?)),
        });
    }

    if is_system_symbol_name(head, "Repeated") && (args.len() == 1 || args.len() == 2) {
        let (min, max) = if let Some(bounds) = args.get(1) {
            parse_string_repetition_bounds(bounds)?
        } else {
            (1, None)
        };
        return Some(StringPattern::Repeated {
            pattern: Box::new(parse_string_pattern(&args[0])?),
            min,
            max,
        });
    }

    if is_system_symbol_name(head, "RepeatedNull") && (args.len() == 1 || args.len() == 2) {
        let (min, max) = if let Some(bounds) = args.get(1) {
            parse_string_repetition_bounds(bounds)?
        } else {
            (0, None)
        };
        return Some(StringPattern::Repeated {
            pattern: Box::new(parse_string_pattern(&args[0])?),
            min,
            max,
        });
    }

    if is_system_symbol_name(head, "Shortest") && args.len() == 1 {
        return Some(StringPattern::Shortest(Box::new(parse_string_pattern(
            &args[0],
        )?)));
    }

    if is_system_symbol_name(head, "Longest") && args.len() == 1 {
        return Some(StringPattern::Longest(Box::new(parse_string_pattern(
            &args[0],
        )?)));
    }

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

fn parse_string_delete_options(options: &[Expr]) -> Option<StringDeleteConfig> {
    let mut config = StringDeleteConfig::default();

    for option in options {
        let Expr::Call { head, args } = option else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }

        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }

        return None;
    }

    Some(config)
}

fn parse_string_match_config(options: &[Expr]) -> Option<StringPatternMatchConfig> {
    let mut config = StringPatternMatchConfig::default();

    for option in options {
        let Expr::Call { head, args } = option else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }

        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }

        return None;
    }

    Some(config)
}

fn parse_string_match_limit_and_options(
    extras: &[Expr],
) -> Option<(usize, StringPatternMatchConfig)> {
    let mut limit = usize::MAX;
    let mut config = StringPatternMatchConfig::default();

    for extra in extras {
        if let Some(value) = expr_as_non_negative_usize(extra) {
            if limit != usize::MAX {
                return None;
            }
            limit = value;
            continue;
        }

        let Expr::Call { head, args } = extra else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }
        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }
        return None;
    }

    Some((limit, config))
}

fn parse_string_position_args(
    extras: &[Expr],
) -> Option<(Option<usize>, StringPatternMatchConfig)> {
    let mut max_results = None;
    let mut config = StringPatternMatchConfig::default();

    for extra in extras {
        if let Some(value) = expr_as_non_negative_usize(extra) {
            if max_results.is_some() {
                return None;
            }
            max_results = Some(value);
            continue;
        }

        let Expr::Call { head, args } = extra else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }
        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }
        return None;
    }

    Some((max_results, config))
}

fn parse_string_count_options(options: &[Expr]) -> Option<StringCountConfig> {
    let mut config = StringCountConfig::default();

    for option in options {
        let Expr::Call { head, args } = option else {
            return None;
        };
        if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
            return None;
        }

        if is_system_symbol_name(&args[0], "IgnoreCase") {
            config.ignore_case = truthiness_symbol(&args[1])?;
            continue;
        }

        if is_system_symbol_name(&args[0], "Overlaps") {
            config.overlaps = if is_system_symbol_name(&args[1], heads::structure::ALL)
                || matches!(truthiness_symbol(&args[1]), Some(true))
            {
                StringCountOverlaps::Overlapping
            } else if matches!(truthiness_symbol(&args[1]), Some(false)) {
                StringCountOverlaps::NonOverlapping
            } else {
                return None;
            };
            continue;
        }

        return None;
    }

    Some(config)
}

fn parse_string_repetition_bounds(expr: &Expr) -> Option<(usize, Option<usize>)> {
    let items = call_items_with_head(expr, &system_expr("List"))?;

    match items {
        [count] => {
            let count = expr_as_non_negative_usize(count)?;
            Some((count, Some(count)))
        }
        [min, max] => {
            let min = expr_as_non_negative_usize(min)?;
            let max = expr_as_non_negative_usize(max)?;
            (min <= max).then_some((min, Some(max)))
        }
        _ => None,
    }
}

fn expr_as_non_empty_char_vec(expr: &Expr) -> Option<Vec<char>> {
    let chars = expr_as_string(expr)?.chars().collect::<Vec<_>>();
    (!chars.is_empty()).then_some(chars)
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

fn string_take_drop_value(value: &str, spec: StringTakeDropSpec) -> (String, String) {
    let chars = value.chars().collect::<Vec<_>>();
    match spec {
        StringTakeDropSpec::Count(count) => (
            take_chars_by_count(&chars, count),
            drop_chars_by_count(&chars, count),
        ),
        StringTakeDropSpec::InclusiveRange(start, end) => {
            let Some((start, end)) = normalize_clamped_inclusive_range(start, end, chars.len())
            else {
                return (String::new(), value.to_string());
            };

            let taken = chars[start..=end].iter().collect::<String>();
            let mut dropped = String::new();
            dropped.extend(chars[..start].iter());
            dropped.extend(chars[end + 1..].iter());
            (taken, dropped)
        }
    }
}

fn string_form_value(template: &str, args: &[Expr]) -> String {
    let mut remaining = template;
    let mut output = String::new();
    let mut replacements = args.iter();

    while let Some(index) = remaining.find("``") {
        output.push_str(&remaining[..index]);
        if let Some(arg) = replacements.next() {
            output.push_str(&arg.to_string());
        } else {
            output.push_str("``");
        }
        remaining = &remaining[index + 2..];
    }

    output.push_str(remaining);
    output
}

fn render_template_object(template: &str) -> Expr {
    Expr::call(
        system_expr("TemplateObject"),
        vec![Expr::string(template.to_string())],
    )
}

fn expr_as_template_source(expr: &Expr) -> Option<&str> {
    if let Some(template) = expr_as_string(expr) {
        return Some(template);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "TemplateObject") || args.len() != 1 {
        return None;
    }

    expr_as_string(&args[0])
}

fn template_apply_value(template: &str, data: &Expr) -> Option<String> {
    let segments = parse_template_segments(template)?;
    let named_slots = template_named_slot_values(data);
    let positional_slots = template_positional_slot_values(data)?;
    let mut positional_index = 0usize;
    let mut output = String::new();

    for segment in segments {
        match segment {
            TemplateSegment::Text(text) => output.push_str(&text),
            TemplateSegment::PositionalSlot => {
                let value = positional_slots.get(positional_index)?;
                output.push_str(value);
                positional_index += 1;
            }
            TemplateSegment::NamedSlot(name) => {
                let value = named_slots.get(&name)?;
                output.push_str(value);
            }
        }
    }

    Some(output)
}

fn parse_template_segments(template: &str) -> Option<Vec<TemplateSegment>> {
    let chars = template.chars().collect::<Vec<_>>();
    let mut segments = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '`' {
            index += 1;
            continue;
        }

        let closing = chars[index + 1..]
            .iter()
            .position(|ch| *ch == '`')
            .map(|offset| index + 1 + offset)?;

        if start < index {
            segments.push(TemplateSegment::Text(chars[start..index].iter().collect()));
        }

        let slot_name = chars[index + 1..closing].iter().collect::<String>();
        if slot_name.is_empty() {
            segments.push(TemplateSegment::PositionalSlot);
        } else {
            segments.push(TemplateSegment::NamedSlot(slot_name));
        }

        index = closing + 1;
        start = index;
    }

    if start < chars.len() {
        segments.push(TemplateSegment::Text(chars[start..].iter().collect()));
    }

    Some(segments)
}

fn template_named_slot_values(data: &Expr) -> BTreeMap<String, String> {
    let mut named = BTreeMap::new();

    if let Some(entries) = association_entries_from_expr(data) {
        for (key, value) in entries {
            if let Some(name) = template_slot_name_from_expr(&key) {
                named.insert(name, template_fragment_value(&value));
            }
        }
        return named;
    }

    if let Some(rules) = template_rule_entries(data) {
        for (key, value) in rules {
            if let Some(name) = template_slot_name_from_expr(&key) {
                named.insert(name, template_fragment_value(&value));
            }
        }
    }

    named
}

fn template_positional_slot_values(data: &Expr) -> Option<Vec<String>> {
    if let Some(items) = call_items_with_head(data, &system_expr("List")) {
        if items.iter().all(|item| parse_template_rule(item).is_some()) {
            return Some(Vec::new());
        }
        return Some(items.iter().map(template_fragment_value).collect());
    }

    if association_entries_from_expr(data).is_some() || template_rule_entries(data).is_some() {
        return Some(Vec::new());
    }

    Some(vec![template_fragment_value(data)])
}

fn template_rule_entries(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(rule) = parse_template_rule(expr) {
        return Some(vec![rule]);
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    items.iter().map(parse_template_rule).collect()
}

fn parse_template_rule(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

fn template_slot_name_from_expr(expr: &Expr) -> Option<String> {
    if let Some(value) = expr_as_string(expr) {
        return Some(value.to_string());
    }

    expr_as_symbol(expr).map(|symbol| symbol.name().to_string())
}

fn template_fragment_value(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::String(value)) => value.clone(),
        _ => expr.to_string(),
    }
}

fn temp_rendered_template_path(source_path: &Path) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let mut filename = format!("template-{}-{nanos}", std::process::id());
    if let Some(extension) = source_path.extension().and_then(|ext| ext.to_str()) {
        filename.push('.');
        filename.push_str(extension);
    }
    std::env::temp_dir().join(filename)
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

fn byte_array_expr(bytes: &[u8]) -> Expr {
    Expr::call(
        system_expr("ByteArray"),
        vec![Expr::string(base64_encode(bytes))],
    )
}

fn hash_input_bytes(expr: &Expr) -> Vec<u8> {
    if let Some(value) = expr_as_string(expr) {
        value.as_bytes().to_vec()
    } else {
        expr.to_string().into_bytes()
    }
}

fn sha256_integer(digest: &[u8; SHA256_OUTPUT_LEN]) -> BigInt {
    BigInt::parse_bytes(bytes_to_hex(digest).as_bytes(), 16)
        .expect("sha256 digests always parse as hexadecimal integers")
}

fn sha256_decimal_string(value: &BigInt) -> String {
    const SHA256_DECIMAL_WIDTH: usize = 78;
    let decimal = value.to_str_radix(10);
    format!("{decimal:0>SHA256_DECIMAL_WIDTH$}")
}

fn compress_envelope(expr: &Expr) -> String {
    let (payload, original_form) = expr_transport_bytes(expr);
    let kind = if is_system_symbol_name(&original_form, "String") {
        "s"
    } else {
        "e"
    };
    format!("rum:1:{kind}:{}", base64_encode(&payload))
}

fn expr_transport_bytes(expr: &Expr) -> (Vec<u8>, Expr) {
    if let Some(value) = expr_as_string(expr) {
        (value.as_bytes().to_vec(), system_expr("String"))
    } else {
        (expr.to_string().into_bytes(), system_expr("InputForm"))
    }
}

fn encrypted_object_expr(ciphertext: &[u8], key_hash: &[u8], original_form: Expr) -> Expr {
    Expr::call(
        system_expr("EncryptedObject"),
        vec![render_association(vec![
            (Expr::string("Cipher"), Expr::string("XOR-SHA256")),
            (Expr::string("KeyHash"), byte_array_expr(key_hash)),
            (Expr::string("Data"), byte_array_expr(ciphertext)),
            (Expr::string("OriginalForm"), original_form),
        ])],
    )
}

fn parse_string_rule_dataset(expr: &Expr) -> Option<(Vec<String>, Vec<Expr>)> {
    let (keys, values) = parse_nearest_rule_pairs(expr)?;
    Some((
        keys.iter()
            .map(expr_as_string)
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .map(str::to_string)
            .collect(),
        values,
    ))
}

fn parse_numeric_dataset(expr: &Expr) -> Option<(Vec<Rational>, Vec<Expr>)> {
    if let Some(items) = call_items_with_head(expr, &system_expr("List")) {
        let keys = items
            .iter()
            .map(expr_as_exact_rational)
            .collect::<Option<Vec<_>>>()?;
        return Some((keys, items.to_vec()));
    }

    let (keys, values) = parse_nearest_rule_pairs(expr)?;
    Some((
        keys.iter()
            .map(expr_as_exact_rational)
            .collect::<Option<Vec<_>>>()?,
        values,
    ))
}

fn parse_nearest_rule_pairs(expr: &Expr) -> Option<(Vec<Expr>, Vec<Expr>)> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Rule") && args.len() == 2 => {
            let keys = call_items_with_head(&args[0], &system_expr("List"))?.to_vec();
            let values = call_items_with_head(&args[1], &system_expr("List"))?.to_vec();
            (keys.len() == values.len()).then_some((keys, values))
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut keys = Vec::with_capacity(args.len());
            let mut values = Vec::with_capacity(args.len());
            for item in args {
                let Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } = item
                else {
                    return None;
                };
                if !is_system_symbol_name(rule_head, "Rule") || rule_args.len() != 2 {
                    return None;
                }
                keys.push(rule_args[0].clone());
                values.push(rule_args[1].clone());
            }
            Some((keys, values))
        }
        _ => None,
    }
}

fn nearest_strings(values: &[String], target: &str, count: Option<usize>) -> Vec<String> {
    let target_chars = target.chars().collect::<Vec<_>>();
    let distances = values
        .iter()
        .map(|value| {
            let value_chars = value.chars().collect::<Vec<_>>();
            levenshtein_distance(&value_chars, &target_chars)
        })
        .collect::<Vec<_>>();
    select_nearest_indices_by_usize(&distances, count)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

fn nearest_values_for_strings(
    keys: &[String],
    values: &[Expr],
    target: &str,
    count: Option<usize>,
) -> Vec<Expr> {
    let target_chars = target.chars().collect::<Vec<_>>();
    let distances = keys
        .iter()
        .map(|value| {
            let value_chars = value.chars().collect::<Vec<_>>();
            levenshtein_distance(&value_chars, &target_chars)
        })
        .collect::<Vec<_>>();
    select_nearest_indices_by_usize(&distances, count)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

fn nearest_values_for_numbers(
    keys: &[Rational],
    values: &[Expr],
    target: &Rational,
    count: Option<usize>,
) -> Vec<Expr> {
    let distances = keys
        .iter()
        .map(|value| rational_abs_difference(value, target))
        .collect::<Vec<_>>();
    select_nearest_indices_by_rational(&distances, count)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

fn select_nearest_indices_by_usize(distances: &[usize], count: Option<usize>) -> Vec<usize> {
    match count {
        Some(0) => Vec::new(),
        Some(limit) => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by_key(|index| (distances[*index], *index));
            indices.truncate(limit.min(indices.len()));
            indices
        }
        None => {
            let Some(best) = distances.iter().copied().min() else {
                return Vec::new();
            };
            distances
                .iter()
                .enumerate()
                .filter_map(|(index, distance)| (*distance == best).then_some(index))
                .collect()
        }
    }
}

fn select_nearest_indices_by_rational(distances: &[Rational], count: Option<usize>) -> Vec<usize> {
    match count {
        Some(0) => Vec::new(),
        Some(limit) => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by(|lhs, rhs| {
                compare_rationals_local(&distances[*lhs], &distances[*rhs]).then(lhs.cmp(rhs))
            });
            indices.truncate(limit.min(indices.len()));
            indices
        }
        None => {
            let Some((best_index, _)) = distances
                .iter()
                .enumerate()
                .min_by(|(_, lhs), (_, rhs)| compare_rationals_local(lhs, rhs))
            else {
                return Vec::new();
            };
            distances
                .iter()
                .enumerate()
                .filter_map(|(index, distance)| {
                    (compare_rationals_local(distance, &distances[best_index])
                        == std::cmp::Ordering::Equal)
                        .then_some(index)
                })
                .collect()
        }
    }
}

fn rational_abs_difference(lhs: &Rational, rhs: &Rational) -> Rational {
    let difference = lhs.sub(rhs);
    if compare_rationals_local(&difference, &Rational::zero()) == std::cmp::Ordering::Less {
        difference.negated()
    } else {
        difference
    }
}

fn compare_rationals_local(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn pluralize_english_word(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    if lower.ends_with("ch")
        || lower.ends_with("sh")
        || lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with('z')
    {
        return format!("{word}es");
    }

    if lower.ends_with('y') {
        let mut chars = word.chars().collect::<Vec<_>>();
        if chars.len() >= 2 {
            let previous = chars[chars.len() - 2].to_ascii_lowercase();
            if !matches!(previous, 'a' | 'e' | 'i' | 'o' | 'u') {
                chars.pop();
                return format!("{}ies", chars.into_iter().collect::<String>());
            }
        }
    }

    format!("{word}s")
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

fn remove_diacritics_value(value: &str) -> String {
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

fn bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

fn string_contains_q(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
) -> bool {
    (0..=haystack.len())
        .any(|start| best_pattern_match_end(pattern, haystack, start, config).is_some())
}

fn string_starts_q(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
) -> bool {
    best_pattern_match_end(pattern, haystack, 0, config).is_some()
}

fn string_ends_q(
    haystack: &[char],
    pattern: &StringPattern,
    config: StringPatternMatchConfig,
) -> bool {
    (0..=haystack.len()).any(|start| {
        best_pattern_match_end(pattern, haystack, start, config) == Some(haystack.len())
    })
}

fn string_match_q(
    haystack: &[char],
    patterns: &[StringPattern],
    config: StringPatternMatchConfig,
) -> bool {
    best_pattern_match_at(patterns, haystack, 0, config)
        .is_some_and(|(_, end)| end == haystack.len())
}

fn string_free_q(
    haystack: &[char],
    patterns: &[StringPattern],
    config: StringPatternMatchConfig,
) -> bool {
    (0..=haystack.len())
        .all(|start| best_pattern_match_at(patterns, haystack, start, config).is_none())
}

fn string_count_matches(
    value: &str,
    patterns: &[StringPattern],
    config: StringCountConfig,
) -> usize {
    let chars = value.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut matches = 0usize;
    let match_config = StringPatternMatchConfig {
        ignore_case: config.ignore_case,
    };

    while index <= chars.len() {
        if let Some((_, end)) = best_pattern_match_at(patterns, &chars, index, match_config) {
            matches += 1;
            index = if end == index {
                if index == chars.len() {
                    break;
                }
                index + 1
            } else {
                match config.overlaps {
                    StringCountOverlaps::NonOverlapping => end,
                    StringCountOverlaps::Overlapping => index + 1,
                }
            };
        } else {
            if index == chars.len() {
                break;
            }
            index += 1;
        }
    }

    matches
}

fn string_delete_matches(
    value: &str,
    patterns: &[StringPattern],
    config: StringDeleteConfig,
) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    let match_config = StringPatternMatchConfig {
        ignore_case: config.ignore_case,
    };

    while index < chars.len() {
        if let Some((_, end)) = best_pattern_match_at(patterns, &chars, index, match_config) {
            if end == index {
                output.push(chars[index]);
                index += 1;
            } else {
                index = end;
            }
        } else {
            output.push(chars[index]);
            index += 1;
        }
    }

    output
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
    config: StringPatternMatchConfig,
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
            best_pattern_match_end(&rule.pattern, &chars, index, config).map(|end| (rule, end))
        }) {
            output.push_str(&rule.replacement);
            if end == index {
                output.push(chars[index]);
                index += 1;
            } else {
                index = end;
            }
            replacements += 1;
        } else {
            output.push(chars[index]);
            index += 1;
        }
    }

    output
}

fn collect_string_cases(
    value: &str,
    spec: &ExactStringCaseSpec,
    max_matches: usize,
    config: StringPatternMatchConfig,
) -> Vec<Expr> {
    if max_matches == 0 {
        return Vec::new();
    }

    let chars = value.chars().collect::<Vec<_>>();
    let mut index = 0;
    let mut matches = Vec::new();

    while index <= chars.len() && matches.len() < max_matches {
        match spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, end)) =
                    best_pattern_match_at(patterns, &chars, index, config)
                {
                    matches.push(Expr::string(pattern_match_text(&chars[index..end])));
                    index = if end == index {
                        if index == chars.len() {
                            break;
                        }
                        index + 1
                    } else {
                        end
                    };
                } else {
                    if index == chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((rule, end)) = rules.iter().find_map(|rule| {
                    best_pattern_match_end(&rule.pattern, &chars, index, config)
                        .map(|end| (rule, end))
                }) {
                    matches.push(rule.output.clone());
                    index = if end == index {
                        if index == chars.len() {
                            break;
                        }
                        index + 1
                    } else {
                        end
                    };
                } else {
                    if index == chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
        }
    }

    matches
}

fn string_position_expr(
    value: &str,
    pattern: &StringPattern,
    max_results: Option<usize>,
    config: StringPatternMatchConfig,
) -> Expr {
    let haystack = value.chars().collect::<Vec<_>>();
    let positions = string_pattern_positions(&haystack, pattern, max_results, config)
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
    config: StringPatternMatchConfig,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    let max_results = max_results.unwrap_or(usize::MAX);

    if max_results == 0 {
        return Vec::new().into_iter();
    }

    let mut positions = Vec::new();
    for start in 0..=haystack.len() {
        if let Some(end) = best_pattern_match_end(pattern, haystack, start, config) {
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

        let Some(end) =
            best_pattern_match_at(patterns, chars, index, StringPatternMatchConfig::default())
                .map(|(_, end)| end)
        else {
            index += 1;
            continue;
        };

        if end == index {
            index += 1;
            continue;
        }

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
        let Some(match_end) = best_pattern_match_at(
            patterns,
            &chars[..end],
            start,
            StringPatternMatchConfig::default(),
        )
        .map(|(_, end)| end) else {
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
    config: StringPatternMatchConfig,
) -> Option<(usize, usize)> {
    patterns
        .iter()
        .enumerate()
        .filter_map(|(index, pattern)| {
            best_pattern_match_end(pattern, chars, start, config).map(|end| (index, end))
        })
        .max_by_key(|(index, end)| (*end, usize::MAX - *index))
}

fn best_pattern_suffix_start(patterns: &[StringPattern], chars: &[char]) -> Option<usize> {
    let end = chars.len();
    (0..end)
        .filter(|start| {
            patterns.iter().any(|pattern| {
                best_pattern_match_end(pattern, chars, *start, StringPatternMatchConfig::default())
                    == Some(end)
            })
        })
        .min()
}

fn best_pattern_match_end(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Option<usize> {
    let positions = pattern_match_end_positions(pattern, chars, start, config);
    if pattern_prefers_shortest(pattern) {
        positions.into_iter().min()
    } else {
        positions.into_iter().max()
    }
}

fn pattern_match_end_positions(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Vec<usize> {
    if start > chars.len() {
        return Vec::new();
    }

    match pattern {
        StringPattern::Literal(literal) => literal_match_end(chars, literal, start, config)
            .into_iter()
            .collect(),
        StringPattern::Whitespace => whitespace_match_end_positions(chars, start),
        StringPattern::NumberString => number_string_match_end_positions(chars, start),
        StringPattern::Except { excluded, inner } => {
            let candidates = match inner {
                Some(inner) => pattern_match_end_positions(inner, chars, start, config),
                None => any_character_match_end_positions(chars, start),
            };

            candidates
                .into_iter()
                .filter(|end| {
                    !pattern_match_end_positions(excluded, chars, start, config).contains(end)
                })
                .collect()
        }
        StringPattern::StartOfString => (start == 0).then_some(start).into_iter().collect(),
        StringPattern::EndOfString => (start == chars.len())
            .then_some(start)
            .into_iter()
            .collect(),
        StringPattern::Repeated { pattern, min, max } => {
            repeated_pattern_end_positions(pattern, chars, start, *min, *max, config)
        }
        StringPattern::Shortest(inner) | StringPattern::Longest(inner) => {
            pattern_match_end_positions(inner, chars, start, config)
        }
        StringPattern::Sequence(parts) => {
            let mut positions = vec![start];

            for part in parts {
                let mut next_positions = Vec::new();
                for position in positions {
                    next_positions
                        .extend(pattern_match_end_positions(part, chars, position, config));
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
                positions.extend(pattern_match_end_positions(
                    alternative,
                    chars,
                    start,
                    config,
                ));
            }
            positions.sort_unstable();
            positions.dedup();
            positions
        }
    }
}

fn pattern_prefers_shortest(pattern: &StringPattern) -> bool {
    matches!(pattern, StringPattern::Shortest(_))
}

fn repeated_pattern_end_positions(
    pattern: &StringPattern,
    chars: &[char],
    start: usize,
    min: usize,
    max: Option<usize>,
    config: StringPatternMatchConfig,
) -> Vec<usize> {
    let mut frontier = vec![start];
    let mut positions = if min == 0 { vec![start] } else { Vec::new() };
    let mut steps = 0usize;

    loop {
        if max.is_some_and(|max| steps >= max) {
            break;
        }
        let mut next = Vec::new();
        for position in frontier {
            for end in pattern_match_end_positions(pattern, chars, position, config) {
                if end > position {
                    next.push(end);
                }
            }
        }
        next.sort_unstable();
        next.dedup();

        if next.is_empty() {
            break;
        }

        steps += 1;
        if steps >= min {
            positions.extend(next.iter().copied());
        }
        frontier = next;
    }

    positions.sort_unstable();
    positions.dedup();
    positions
}

fn any_character_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    (start < chars.len())
        .then_some(start + 1)
        .into_iter()
        .collect()
}

fn literal_match_end(
    chars: &[char],
    literal: &[char],
    start: usize,
    config: StringPatternMatchConfig,
) -> Option<usize> {
    if chars.len().saturating_sub(start) < literal.len() {
        return None;
    }

    for (offset, expected) in literal.iter().enumerate() {
        let actual = chars[start + offset];
        let matches = if config.ignore_case {
            actual.to_lowercase().to_string() == expected.to_lowercase().to_string()
        } else {
            actual == *expected
        };
        if !matches {
            return None;
        }
    }

    Some(start + literal.len())
}

fn whitespace_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    if start >= chars.len() || !is_string_whitespace(chars[start]) {
        return Vec::new();
    }

    let mut end = start;
    let mut positions = Vec::new();
    while end < chars.len() && is_string_whitespace(chars[end]) {
        end += 1;
        positions.push(end);
    }
    positions
}

fn number_string_match_end_positions(chars: &[char], start: usize) -> Vec<usize> {
    if start >= chars.len() {
        return Vec::new();
    }

    let mut positions = Vec::new();
    let mut index = start;

    if matches!(chars.get(index), Some(&'+') | Some(&'-')) {
        index += 1;
    }

    let integer_start = index;
    while index < chars.len() && chars[index].is_ascii_digit() {
        index += 1;
        positions.push(index);
    }
    let integer_end = index;

    if chars.get(index) == Some(&'.') {
        index += 1;
        if integer_end > integer_start {
            positions.push(index);
        }

        let fraction_start = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
            positions.push(index);
        }

        if integer_end == integer_start && fraction_start == index {
            return Vec::new();
        }
    }

    positions
}

fn string_slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}

fn string_unary_operator_form(symbol: &str, operator: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![operator.clone(), string_slot_expr()],
        )],
    )
}

fn string_unary_operator_form_slot_first(symbol: &str, tail: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![string_slot_expr(), tail.clone()],
        )],
    )
}

fn string_apply_value(
    session: &mut Session,
    operator: &Expr,
    value: &str,
    fallback: Expr,
) -> Result<Expr, EvalError> {
    let applied = Expr::call(
        operator.clone(),
        vec![list_expr(
            value
                .chars()
                .map(|ch| Expr::string(ch.to_string()))
                .collect(),
        )],
    );
    let evaluated = session.evaluate(applied)?;
    Ok(string_apply_output_expr(evaluated).unwrap_or(fallback))
}

fn string_apply_output_expr(expr: Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::String(_)) => Some(expr),
        Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
            if args
                .iter()
                .all(|item| matches!(item, Expr::Atom(Atom::String(_))))
            {
                let joined = args
                    .into_iter()
                    .filter_map(|item| match item {
                        Expr::Atom(Atom::String(value)) => Some(value),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .concat();
                return Some(Expr::string(joined));
            }

            let converted = args
                .into_iter()
                .map(string_apply_output_expr)
                .collect::<Option<Vec<_>>>()?;
            Some(list_expr(converted))
        }
        _ => None,
    }
}

fn english_alphabet() -> Vec<String> {
    ('a'..='z').map(|ch| ch.to_string()).collect()
}

fn alphabetic_sort_key(value: &str) -> Vec<(usize, String)> {
    value
        .chars()
        .map(|ch| {
            let folded = remove_diacritics_value(&ch.to_string()).to_lowercase();
            let primary = folded
                .chars()
                .next()
                .and_then(english_letter_index)
                .unwrap_or(usize::MAX);
            (primary, ch.to_lowercase().to_string())
        })
        .collect()
}

fn english_letter_index(ch: char) -> Option<usize> {
    ch.is_ascii_lowercase().then(|| (ch as u8 - b'a') as usize)
}

fn levenshtein_distance<T: Eq>(lhs: &[T], rhs: &[T]) -> usize {
    let mut previous = (0..=rhs.len()).collect::<Vec<_>>();
    let mut current = vec![0usize; rhs.len() + 1];

    for (i, left) in lhs.iter().enumerate() {
        current[0] = i + 1;
        for (j, right) in rhs.iter().enumerate() {
            let substitution = previous[j] + usize::from(left != right);
            let insertion = current[j] + 1;
            let deletion = previous[j + 1] + 1;
            current[j + 1] = substitution.min(insertion).min(deletion);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[rhs.len()]
}

fn hamming_distance<T: Eq>(lhs: &[T], rhs: &[T]) -> Option<usize> {
    if lhs.len() != rhs.len() {
        return None;
    }

    Some(
        lhs.iter()
            .zip(rhs)
            .filter(|(left, right)| left != right)
            .count(),
    )
}

fn longest_common_contiguous_slice<T: Clone + Eq>(lhs: &[T], rhs: &[T]) -> Vec<T> {
    if lhs.is_empty() || rhs.is_empty() {
        return Vec::new();
    }

    let mut dp = vec![vec![0usize; rhs.len() + 1]; lhs.len() + 1];
    let mut best_len = 0usize;
    let mut best_end = 0usize;

    for i in 1..=lhs.len() {
        for j in 1..=rhs.len() {
            if lhs[i - 1] == rhs[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
                if dp[i][j] > best_len {
                    best_len = dp[i][j];
                    best_end = i;
                }
            }
        }
    }

    lhs[best_end.saturating_sub(best_len)..best_end].to_vec()
}

fn global_alignment_blocks<T: Clone + Eq>(lhs: &[T], rhs: &[T]) -> Vec<AlignmentBlock<T>> {
    let m = lhs.len();
    let n = rhs.len();
    let mut scores = vec![vec![0isize; n + 1]; m + 1];
    let mut steps = vec![vec![AlignmentStep::Done; n + 1]; m + 1];

    for i in 1..=m {
        scores[i][0] = -(i as isize);
        steps[i][0] = AlignmentStep::Up;
    }
    for j in 1..=n {
        scores[0][j] = -(j as isize);
        steps[0][j] = AlignmentStep::Left;
    }

    for i in 1..=m {
        for j in 1..=n {
            let diag_score = scores[i - 1][j - 1] + if lhs[i - 1] == rhs[j - 1] { 1 } else { -1 };
            let up_score = scores[i - 1][j] - 1;
            let left_score = scores[i][j - 1] - 1;
            let (score, step) = if diag_score >= up_score && diag_score >= left_score {
                (diag_score, AlignmentStep::Diag)
            } else if up_score >= left_score {
                (up_score, AlignmentStep::Up)
            } else {
                (left_score, AlignmentStep::Left)
            };
            scores[i][j] = score;
            steps[i][j] = step;
        }
    }

    let mut aligned = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 || j > 0 {
        match steps[i][j] {
            AlignmentStep::Diag => {
                aligned.push((Some(lhs[i - 1].clone()), Some(rhs[j - 1].clone())));
                i -= 1;
                j -= 1;
            }
            AlignmentStep::Up => {
                aligned.push((Some(lhs[i - 1].clone()), None));
                i -= 1;
            }
            AlignmentStep::Left => {
                aligned.push((None, Some(rhs[j - 1].clone())));
                j -= 1;
            }
            AlignmentStep::Done => break,
        }
    }
    aligned.reverse();

    let mut blocks = Vec::new();
    let mut common = Vec::new();
    let mut left_diff = Vec::new();
    let mut right_diff = Vec::new();

    for (left, right) in aligned {
        match (left, right) {
            (Some(left), Some(right)) if left == right => {
                if !left_diff.is_empty() || !right_diff.is_empty() {
                    blocks.push(AlignmentBlock::Different {
                        left: std::mem::take(&mut left_diff),
                        right: std::mem::take(&mut right_diff),
                    });
                }
                common.push(left);
            }
            (left, right) => {
                if !common.is_empty() {
                    blocks.push(AlignmentBlock::Common(std::mem::take(&mut common)));
                }
                if let Some(left) = left {
                    left_diff.push(left);
                }
                if let Some(right) = right {
                    right_diff.push(right);
                }
            }
        }
    }

    if !common.is_empty() {
        blocks.push(AlignmentBlock::Common(common));
    }
    if !left_diff.is_empty() || !right_diff.is_empty() {
        blocks.push(AlignmentBlock::Different {
            left: left_diff,
            right: right_diff,
        });
    }

    blocks
}

fn render_string_alignment_blocks(blocks: &[AlignmentBlock<char>]) -> Expr {
    list_expr(
        blocks
            .iter()
            .map(|block| match block {
                AlignmentBlock::Common(common) => Expr::string(common.iter().collect::<String>()),
                AlignmentBlock::Different { left, right } => list_expr(vec![
                    Expr::string(left.iter().collect::<String>()),
                    Expr::string(right.iter().collect::<String>()),
                ]),
            })
            .collect(),
    )
}

fn render_expr_alignment_blocks(blocks: &[AlignmentBlock<Expr>]) -> Expr {
    list_expr(
        blocks
            .iter()
            .map(|block| match block {
                AlignmentBlock::Common(common) => list_expr(common.clone()),
                AlignmentBlock::Different { left, right } => {
                    list_expr(vec![list_expr(left.clone()), list_expr(right.clone())])
                }
            })
            .collect(),
    )
}

fn render_diff_object_from_chars(blocks: &[AlignmentBlock<char>]) -> Expr {
    render_diff_object("Characters", blocks, |items| {
        Expr::string(items.iter().collect::<String>())
    })
}

fn render_diff_object_from_exprs(blocks: &[AlignmentBlock<Expr>]) -> Expr {
    render_diff_object("Elements", blocks, |items| list_expr(items.to_vec()))
}

fn render_diff_object<T: Clone>(
    mode: &str,
    blocks: &[AlignmentBlock<T>],
    render_items: impl Fn(&[T]) -> Expr,
) -> Expr {
    let mut left_index = 0usize;
    let mut right_index = 0usize;
    let mut changes = Vec::new();

    for block in blocks {
        match block {
            AlignmentBlock::Common(common) => {
                let rule = Expr::call(
                    system_expr("Rule"),
                    vec![
                        list_expr(vec![
                            Expr::integer(BigInt::from(left_index + 1)),
                            Expr::integer(BigInt::from(right_index + 1)),
                        ]),
                        render_items(common),
                    ],
                );
                changes.push(list_expr(vec![rule]));
                left_index += common.len();
                right_index += common.len();
            }
            AlignmentBlock::Different { left, right } => {
                let left_key = if left.is_empty() {
                    left_index
                } else {
                    left_index + 1
                };
                let right_key = if right.is_empty() {
                    right_index
                } else {
                    right_index + 1
                };
                changes.push(list_expr(vec![
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::integer(BigInt::from(left_key)), render_items(left)],
                    ),
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::integer(BigInt::from(right_key)), render_items(right)],
                    ),
                ]));
                left_index += left.len();
                right_index += right.len();
            }
        }
    }

    Expr::call(
        system_expr("DiffObject"),
        vec![
            Expr::integer(BigInt::from(1_u8)),
            Expr::string(mode.to_string()),
            render_association(vec![
                (Expr::string("Changes"), list_expr(changes)),
                (
                    Expr::string("View"),
                    list_expr(vec![system_expr("Automatic")]),
                ),
            ]),
        ],
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AlignmentStep {
    Done,
    Diag,
    Up,
    Left,
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

const SHA256_OUTPUT_LEN: usize = 32;

const SHA256_INITIAL_STATE: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | (bytes[index + 2] as u32);
        rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[(chunk & 0x3f) as usize] as char);
        index += 3;
    }

    match bytes.len() - index {
        1 => {
            let chunk = (bytes[index] as u32) << 16;
            rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            rendered.push('=');
            rendered.push('=');
        }
        2 => {
            let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
            rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
            rendered.push('=');
        }
        _ => {}
    }

    rendered
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        rendered.push(nibble_to_hex(byte >> 4));
        rendered.push(nibble_to_hex(byte & 0x0f));
    }
    rendered
}

fn xor_with_sha256_keystream(key: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(payload.len());
    let mut counter = 0u64;
    while output.len() < payload.len() {
        let mut counter_bytes = [0u8; 8];
        counter_bytes.copy_from_slice(&counter.to_be_bytes());
        let block = sha256_many([key, &counter_bytes]);
        for byte in block {
            if output.len() == payload.len() {
                break;
            }
            let index = output.len();
            output.push(payload[index] ^ byte);
        }
        counter += 1;
    }
    output
}

fn sha256_bytes(bytes: &[u8]) -> [u8; SHA256_OUTPUT_LEN] {
    sha256_many([bytes])
}

fn sha256_many<const N: usize>(chunks: [&[u8]; N]) -> [u8; SHA256_OUTPUT_LEN] {
    let total_len = chunks.iter().map(|chunk| chunk.len()).sum::<usize>();
    let bit_len = (total_len as u64) * 8;
    let padded_len = ((total_len + 9 + 63) / 64) * 64;
    let mut message = Vec::with_capacity(padded_len);
    for chunk in chunks {
        message.extend_from_slice(chunk);
    }
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = SHA256_INITIAL_STATE;
    for block in message.chunks_exact(64) {
        sha256_compress(&mut state, block);
    }

    let mut digest = [0u8; SHA256_OUTPUT_LEN];
    for (index, word) in state.into_iter().enumerate() {
        digest[index * 4..(index + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn sha256_compress(state: &mut [u32; 8], block: &[u8]) {
    let mut schedule = [0u32; 64];
    for (index, chunk) in block.chunks_exact(4).take(16).enumerate() {
        schedule[index] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }

    for index in 16..64 {
        let s0 = schedule[index - 15].rotate_right(7)
            ^ schedule[index - 15].rotate_right(18)
            ^ (schedule[index - 15] >> 3);
        let s1 = schedule[index - 2].rotate_right(17)
            ^ schedule[index - 2].rotate_right(19)
            ^ (schedule[index - 2] >> 10);
        schedule[index] = schedule[index - 16]
            .wrapping_add(s0)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(s1);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for index in 0..64 {
        let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choice = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(sum1)
            .wrapping_add(choice)
            .wrapping_add(SHA256_ROUND_CONSTANTS[index])
            .wrapping_add(schedule[index]);
        let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = sum0.wrapping_add(majority);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + (nibble - 10)),
        _ => unreachable!("nibbles are always in 0..=15"),
    }
}
