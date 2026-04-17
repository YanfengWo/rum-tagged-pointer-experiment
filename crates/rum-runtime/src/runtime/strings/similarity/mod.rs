use super::*;

mod distance;
mod lexical;
mod nearest;

use self::distance::*;
use self::lexical::*;
use self::nearest::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NearestResultSpec {
    BestTies,
    Count(usize),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NearestSelectionSpec {
    Results(NearestResultSpec),
    ResultsWithin {
        results: NearestResultSpec,
        max_distance: Expr,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NearestDistanceFunction {
    Automatic,
    EditDistance,
    HammingDistance,
}

impl Session {
    pub(in crate::runtime) fn eval_nearest(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if let [data] = args {
            return Ok(Expr::call(
                system_expr("Function"),
                vec![Expr::call(
                    system_expr("Nearest"),
                    vec![data.clone(), string_slot_expr()],
                )],
            ));
        }

        if let [data, option_expr] = args {
            if parse_nearest_options(std::slice::from_ref(option_expr)).is_some() {
                return Ok(Expr::call(
                    system_expr("Function"),
                    vec![Expr::call(
                        system_expr("Nearest"),
                        vec![data.clone(), string_slot_expr(), option_expr.clone()],
                    )],
                ));
            }
        }

        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Nearest", args, 1, 4));
        }

        let (result_spec, distance_function) = match args {
            [_, _] => (
                NearestSelectionSpec::Results(NearestResultSpec::BestTies),
                NearestDistanceFunction::Automatic,
            ),
            [_, _, third] => {
                if let Some(spec) = parse_nearest_selection_spec(third) {
                    (spec, NearestDistanceFunction::Automatic)
                } else if let Some(options) = parse_nearest_options(std::slice::from_ref(third)) {
                    (
                        NearestSelectionSpec::Results(NearestResultSpec::BestTies),
                        options.distance_function,
                    )
                } else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
            }
            [_, _, third, fourth] => {
                let Some(spec) = parse_nearest_selection_spec(third) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                let Some(options) = parse_nearest_options(std::slice::from_ref(fourth)) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                (spec, options.distance_function)
            }
            _ => unreachable!(),
        };

        if let Some(target) = expr_as_string(&args[1]) {
            if let Some(values) = expr_as_string_list(&args[0]) {
                let Some(results) =
                    nearest_strings(&values, target, result_spec, distance_function)
                else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                return Ok(string_list_expr(results));
            }

            if let Some((keys, values)) = parse_string_rule_dataset(&args[0]) {
                let Some(results) = nearest_values_for_strings(
                    &keys,
                    &values,
                    target,
                    result_spec,
                    distance_function,
                ) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                return Ok(list_expr(results));
            }
        }

        if let Some(target) = expr_as_exact_rational(&args[1]) {
            if distance_function != NearestDistanceFunction::Automatic {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
            if let Some((keys, values)) = parse_numeric_dataset(&args[0]) {
                return Ok(list_expr(nearest_values_for_numbers(
                    &keys,
                    &values,
                    &target,
                    result_spec,
                )));
            }
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_pluralize(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_alphabet(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_alphabetic_sort(
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

    pub(in crate::runtime) fn eval_edit_distance(
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

    pub(in crate::runtime) fn eval_hamming_distance(
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

    pub(in crate::runtime) fn eval_longest_common_subsequence(
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

    pub(in crate::runtime) fn eval_sequence_alignment(
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

    pub(in crate::runtime) fn eval_diff(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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
}
