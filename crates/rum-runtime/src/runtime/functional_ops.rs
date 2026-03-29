use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompositionDirection {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub(super) struct CompositionSpec<'a> {
    pub(super) direction: CompositionDirection,
    pub(super) functions: Vec<&'a Expr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CurriedOperatorKind {
    OperatorApplied,
    CurryApplied,
}

#[derive(Clone, Debug)]
struct CurriedOperatorSpec<'a> {
    kind: CurriedOperatorKind,
    function: &'a Expr,
    arity: usize,
    order: Vec<usize>,
    accumulated: Vec<Expr>,
}

#[derive(Clone, Debug)]
struct ReverseAppliedSpec<'a> {
    function: &'a Expr,
    reversed_count: Option<usize>,
}

impl CurriedOperatorKind {
    fn symbol(self) -> &'static str {
        match self {
            Self::OperatorApplied => "OperatorApplied",
            Self::CurryApplied => "CurryApplied",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TraversalConfig {
    levels: CasesLevelSpec,
    include_heads: bool,
}

pub(super) fn parse_composition(expr: &Expr) -> Option<CompositionSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Composition") => {
            Some(CompositionSpec {
                direction: CompositionDirection::Left,
                functions: args.iter().collect(),
            })
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "RightComposition") => {
            Some(CompositionSpec {
                direction: CompositionDirection::Right,
                functions: args.iter().collect(),
            })
        }
        _ => None,
    }
}

pub(super) fn apply_composition_to_args(spec: &CompositionSpec<'_>, args: &[Expr]) -> Expr {
    match spec.direction {
        CompositionDirection::Left => {
            let Some((last, rest)) = spec.functions.split_last() else {
                return Expr::call(system_expr("Identity"), args.to_vec());
            };

            let mut current = Expr::call((*last).clone(), args.to_vec());
            for function in rest.iter().rev() {
                current = Expr::call((**function).clone(), vec![current]);
            }
            current
        }
        CompositionDirection::Right => {
            let Some((first, rest)) = spec.functions.split_first() else {
                return Expr::call(system_expr("Identity"), args.to_vec());
            };

            let mut current = Expr::call((*first).clone(), args.to_vec());
            for function in rest {
                current = Expr::call((**function).clone(), vec![current]);
            }
            current
        }
    }
}

pub(super) fn is_callable_operator_wrapper(expr: &Expr) -> bool {
    parse_curried_operator(expr).is_some() || parse_reverse_applied_wrapper(expr).is_some()
}

pub(super) fn apply_callable_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    if let Some(spec) = parse_curried_operator(head) {
        return Some(apply_curried_operator_to_args(&spec, args));
    }

    parse_reverse_applied_wrapper(head).map(|spec| apply_reverse_applied_to_args(&spec, args))
}

impl Session {
    pub(super) fn eval_block_map(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [operator, size] => Ok(unary_operator_form_with_tail(
                "BlockMap",
                operator.clone(),
                size.clone(),
            )),
            [operator, size, offset]
                if expr_as_positive_usize(size).is_some()
                    && expr_as_positive_usize(offset).is_some() =>
            {
                Ok(unary_operator_form_with_tails(
                    "BlockMap",
                    operator.clone(),
                    vec![size.clone(), offset.clone()],
                ))
            }
            [operator, expr, size_expr] => {
                let Some(size) = expr_as_positive_usize(size_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "BlockMap",
                        args,
                        "expected a positive integer block size as the third argument",
                    ));
                };

                apply_block_map(self, head, operator, expr, size, size, args)
            }
            [operator, expr, size_expr, offset_expr] => {
                let Some(size) = expr_as_positive_usize(size_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "BlockMap",
                        args,
                        "expected a positive integer block size as the third argument",
                    ));
                };
                let Some(offset) = expr_as_positive_usize(offset_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "BlockMap",
                        args,
                        "expected a positive integer offset as the fourth argument",
                    ));
                };

                apply_block_map(self, head, operator, expr, size, offset, args)
            }
            _ => Ok(self.invalid_call_with_arg_range("BlockMap", args, 2, 4)),
        }
    }

    pub(super) fn eval_subset_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, positions] => Ok(unary_operator_form_with_tail(
                "SubsetMap",
                operator.clone(),
                positions.clone(),
            )),
            [operator, expr, positions] => {
                let Some(paths) = subset_map_paths_from_expr(expr, positions) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "SubsetMap",
                        args,
                        "expected an exact part specification or list of exact part specifications as the third argument",
                    ));
                };

                if !subset_map_paths_are_compatible(&paths) {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }

                apply_subset_map(self, head, operator, expr, &paths, args)
            }
            _ => Ok(self.invalid_call_with_arg_range("SubsetMap", args, 2, 3)),
        }
    }

    pub(super) fn eval_comap(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match self.normalize_comap_call(head, args) {
            CommonArgMapNormalization::OperatorForm(expr)
            | CommonArgMapNormalization::Immediate(expr) => Ok(expr),
            CommonArgMapNormalization::Call {
                functions,
                common_expr,
                config,
            } => {
                let operator =
                    function_body_operator(Expr::call(slot_expr(), vec![common_expr.clone()]));
                let (mapped, _) = map_parts_with_config(&functions, &operator, config);
                self.evaluate(mapped)
            }
        }
    }

    pub(super) fn eval_comap_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_comap_apply_call(head, args) {
            CommonArgMapNormalization::OperatorForm(expr)
            | CommonArgMapNormalization::Immediate(expr) => Ok(expr),
            CommonArgMapNormalization::Call {
                functions,
                common_expr,
                config,
            } => {
                let operator = function_body_operator(Expr::call(
                    system_expr("Apply"),
                    vec![slot_expr(), common_expr.clone()],
                ));
                let (mapped, _) = map_parts_with_config(&functions, &operator, config);
                self.evaluate(mapped)
            }
        }
    }

    pub(super) fn eval_accumulate(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Accumulate", args, 1));
        }

        let Some(sequence) = sequence_like_from_expr(&args[0]) else {
            return Ok(Expr::call(system_expr("Accumulate"), args.to_vec()));
        };

        accumulate_sequence(self, sequence)
    }

    pub(super) fn eval_map_indexed(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::MapIndexed) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_indexed_parts_with_config(&expr, &operator, config).0),
        }
    }

    pub(super) fn eval_map(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::Map) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_parts_with_config(&expr, &operator, config).0),
        }
    }

    pub(super) fn eval_map_all(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match self.normalize_map_all_call(head, args) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_parts_with_mode(&expr, &operator, config, MapTraversalFlavor::MapAll).0),
        }
    }

    pub(super) fn eval_scan(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::Scan) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => {
                let parts = collect_scan_parts_with_config(&expr, config);
                self.evaluate_scan_parts(&operator, parts)
            }
        }
    }

    pub(super) fn eval_map_at(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [operator, position_spec] => Ok(unary_operator_form_with_tail(
                "MapAt",
                operator.clone(),
                position_spec.clone(),
            )),
            [operator, expr, position_spec] => {
                let Some(paths) = exact_position_paths_from_expr(expr, position_spec) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "MapAt",
                        args,
                        "expected an exact part specification or list of exact part specifications as the third argument",
                    ));
                };

                self.apply_map_at_paths(operator, expr.clone(), paths)
            }
            _ => Ok(self.invalid_call_with_arg_range("MapAt", args, 2, 3)),
        }
    }

    pub(super) fn eval_map_thread(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(unary_operator_form("MapThread", operator)),
            [operator, expr] => self.finish_map_thread(
                head,
                args,
                operator,
                expr,
                Expr::integer(BigInt::from(1_u8)),
                1,
            ),
            [operator, expr, level_expr] => {
                let Some(level) = expr_as_non_negative_usize(level_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "MapThread",
                        args,
                        "expected a positive integer level as the third argument",
                    ));
                };

                if level == 0 {
                    return Ok(self.invalid_call_with_arg_form(
                        "MapThread",
                        args,
                        "expected a positive integer level as the third argument",
                    ));
                }

                self.finish_map_thread(head, args, operator, expr, level_expr.clone(), level)
            }
            _ => Ok(self.invalid_call_with_arg_range("MapThread", args, 1, 3)),
        }
    }

    pub(super) fn eval_take_while(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [criterion] => Ok(unary_operator_form_slot_first("TakeWhile", criterion)),
            [expr, criterion] => {
                let Some(sequence) = sequence_like_from_expr(expr) else {
                    return Ok(Expr::call(system_expr("TakeWhile"), args.to_vec()));
                };

                self.take_while_sequence(sequence, criterion)
            }
            _ => Ok(self.invalid_call_with_arg_range("TakeWhile", args, 1, 2)),
        }
    }

    pub(super) fn eval_length_while(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [criterion] => Ok(unary_operator_form_slot_first("LengthWhile", criterion)),
            [expr, criterion] => {
                let Some(sequence) = sequence_like_from_expr(expr) else {
                    return Ok(Expr::call(system_expr("LengthWhile"), args.to_vec()));
                };

                let count = self.length_while_sequence(sequence, criterion)?;
                Ok(Expr::integer(BigInt::from(count)))
            }
            _ => Ok(self.invalid_call_with_arg_range("LengthWhile", args, 1, 2)),
        }
    }

    pub(super) fn eval_inner(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (f, left, right, g) = match args {
            [f, left, right] => (f, left, right, system_expr("Plus")),
            [f, left, right, g] => (f, left, right, g.clone()),
            _ => return Ok(self.invalid_call_with_arg_range("Inner", args, 3, 4)),
        };

        let Some(result) = compute_inner(self, f, left, right, &g)? else {
            return Ok(Expr::call(system_expr("Inner"), args.to_vec()));
        };

        Ok(result)
    }

    pub(super) fn eval_fold(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(variadic_operator_form("Fold", vec![operator.clone()])),
            [operator, list_expr] => apply_fold(self, operator, None, list_expr, false),
            [operator, initial, list_expr] => {
                apply_fold(self, operator, Some(initial.clone()), list_expr, false)
            }
            _ => Ok(self.invalid_call_with_arg_range("Fold", args, 1, 3)),
        }
    }

    pub(super) fn eval_fold_list(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(variadic_operator_form("FoldList", vec![operator.clone()])),
            [operator, list_expr] => apply_fold(self, operator, None, list_expr, true),
            [operator, initial, list_expr] => {
                apply_fold(self, operator, Some(initial.clone()), list_expr, true)
            }
            _ => Ok(self.invalid_call_with_arg_range("FoldList", args, 1, 3)),
        }
    }

    pub(super) fn eval_fold_pair(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [operator, list_expr] => {
                apply_fold_pair(self, head, operator, None, list_expr, None, args)
            }
            [operator, initial, list_expr] => apply_fold_pair(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                None,
                args,
            ),
            [operator, initial, list_expr, projector] => apply_fold_pair(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                Some(projector.clone()),
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("FoldPair", args, 2, 4)),
        }
    }

    pub(super) fn eval_fold_while(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, test] => Ok(variadic_operator_form_with_suffix(
                "FoldWhile",
                vec![operator.clone()],
                vec![test.clone()],
            )),
            [operator, list_expr, test] => apply_fold_while(
                self, head, operator, None, list_expr, test, None, None, args,
            ),
            [operator, initial, list_expr, test] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                None,
                None,
                args,
            ),
            [operator, initial, list_expr, test, m_expr] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                Some(m_expr),
                None,
                args,
            ),
            [operator, initial, list_expr, test, m_expr, n_expr] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                Some(m_expr),
                Some(n_expr),
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("FoldWhile", args, 2, 6)),
        }
    }

    pub(super) fn eval_sequence_fold(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, seeds, items] => {
                apply_sequence_fold(self, head, operator, seeds, items, None, false, args)
            }
            [operator, seeds, items, arity_expr] => apply_sequence_fold(
                self,
                head,
                operator,
                seeds,
                items,
                Some(arity_expr),
                false,
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("SequenceFold", args, 3, 4)),
        }
    }

    pub(super) fn eval_sequence_fold_list(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, seeds, items] => {
                apply_sequence_fold(self, head, operator, seeds, items, None, true, args)
            }
            [operator, seeds, items, arity_expr] => apply_sequence_fold(
                self,
                head,
                operator,
                seeds,
                items,
                Some(arity_expr),
                true,
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("SequenceFoldList", args, 3, 4)),
        }
    }

    pub(super) fn eval_nest(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [operator, count_expr] => {
                let Some(count) = expr_as_non_negative_usize(count_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Nest",
                        args,
                        "expected a non-negative integer iteration count",
                    ));
                };

                Ok(nest_operator_form("Nest", operator, count))
            }
            [operator, expr, count_expr] => {
                let Some(count) = expr_as_non_negative_usize(count_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Nest",
                        args,
                        "expected a non-negative integer iteration count",
                    ));
                };

                apply_nest(self, operator, expr, count)
            }
            _ => Ok(self.invalid_call_with_arg_range("Nest", args, 2, 3)),
        }
    }

    pub(super) fn eval_nest_while(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, test] => Ok(unary_operator_form_with_tail(
                "NestWhile",
                operator.clone(),
                test.clone(),
            )),
            [operator, expr, test] => apply_nest_while(
                self,
                operator,
                expr,
                test,
                NestWhileConfig::default(),
                false,
            ),
            [operator, expr, test, m_expr] => {
                let Some(config) = parse_nest_while_config(m_expr, None) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "NestWhile",
                        args,
                        "expected All or a positive integer test arity as the fourth argument",
                    ));
                };

                apply_nest_while(self, operator, expr, test, config, false)
            }
            [operator, expr, test, m_expr, max_expr] => {
                let Some(config) = parse_nest_while_config(m_expr, Some(max_expr)) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "NestWhile",
                        args,
                        "expected All or a positive integer test arity followed by a non-negative integer iteration limit",
                    ));
                };

                apply_nest_while(self, operator, expr, test, config, false)
            }
            _ => Ok(self.invalid_call_with_arg_range("NestWhile", args, 2, 5)),
        }
    }

    pub(super) fn eval_nest_list(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, count_expr] => {
                let Some(count) = expr_as_non_negative_usize(count_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "NestList",
                        args,
                        "expected a non-negative integer iteration count",
                    ));
                };

                Ok(nest_operator_form("NestList", operator, count))
            }
            [operator, expr, count_expr] => {
                let Some(count) = expr_as_non_negative_usize(count_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "NestList",
                        args,
                        "expected a non-negative integer iteration count",
                    ));
                };

                apply_nest_list(self, operator, expr, count)
            }
            _ => Ok(self.invalid_call_with_arg_range("NestList", args, 2, 3)),
        }
    }

    pub(super) fn eval_nest_while_list(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, test] => Ok(unary_operator_form_with_tail(
                "NestWhileList",
                operator.clone(),
                test.clone(),
            )),
            [operator, expr, test] => {
                apply_nest_while(self, operator, expr, test, NestWhileConfig::default(), true)
            }
            [operator, expr, test, m_expr] => {
                let Some(config) = parse_nest_while_config(m_expr, None) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "NestWhileList",
                        args,
                        "expected All or a positive integer test arity as the fourth argument",
                    ));
                };

                apply_nest_while(self, operator, expr, test, config, true)
            }
            [operator, expr, test, m_expr, max_expr] => {
                let Some(config) = parse_nest_while_config(m_expr, Some(max_expr)) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "NestWhileList",
                        args,
                        "expected All or a positive integer test arity followed by a non-negative integer iteration limit",
                    ));
                };

                apply_nest_while(self, operator, expr, test, config, true)
            }
            _ => Ok(self.invalid_call_with_arg_range("NestWhileList", args, 2, 5)),
        }
    }

    pub(super) fn eval_fixed_point_list(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(unary_operator_form("FixedPointList", operator)),
            [operator, expr] => apply_fixed_point_list(self, operator, expr, None),
            [operator, expr, max_expr] => {
                let Some(max_iterations) = expr_as_non_negative_usize(max_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "FixedPointList",
                        args,
                        "expected a non-negative integer iteration limit",
                    ));
                };

                apply_fixed_point_list(self, operator, expr, Some(max_iterations))
            }
            _ => Ok(self.invalid_call_with_arg_range("FixedPointList", args, 1, 3)),
        }
    }

    pub(super) fn eval_fixed_point(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(unary_operator_form("FixedPoint", operator)),
            [operator, expr] => apply_fixed_point(self, operator, expr, None),
            [operator, expr, max_expr] => {
                let Some(max_iterations) = expr_as_non_negative_usize(max_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "FixedPoint",
                        args,
                        "expected a non-negative integer iteration limit",
                    ));
                };

                apply_fixed_point(self, operator, expr, Some(max_iterations))
            }
            _ => Ok(self.invalid_call_with_arg_range("FixedPoint", args, 1, 3)),
        }
    }

    pub(super) fn eval_apply(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match self.normalize_apply_call(head, args, ApplyFlavor::Apply) {
            ApplyNormalization::OperatorForm(expr) | ApplyNormalization::Immediate(expr) => {
                Ok(expr)
            }
            ApplyNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(apply_heads_with_config(&expr, &operator, config, ApplyFlavor::Apply).0),
        }
    }

    pub(super) fn eval_map_apply(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match self.normalize_apply_call(head, args, ApplyFlavor::MapApply) {
            ApplyNormalization::OperatorForm(expr) | ApplyNormalization::Immediate(expr) => {
                Ok(expr)
            }
            ApplyNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(apply_heads_with_config(&expr, &operator, config, ApplyFlavor::MapApply).0),
        }
    }

    pub(super) fn eval_operator_applied(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(Expr::call(head.clone(), vec![function.clone()])),
            [function, spec] if parse_curried_operator_spec(spec).is_some() => {
                Ok(Expr::call(head.clone(), vec![function.clone(), spec.clone()]))
            }
            [function, spec, accumulated]
                if parse_curried_operator_spec(spec).is_some()
                    && parse_callable_wrapper_accumulated(accumulated).is_some() =>
            {
                Ok(Expr::call(
                    head.clone(),
                    vec![function.clone(), spec.clone(), accumulated.clone()],
                ))
            }
            [_, _] | [_, _, _] => Ok(self.invalid_call_with_arg_form(
                "OperatorApplied",
                args,
                "expected a non-negative integer, an index list, or k -> index-list as the specification argument",
            )),
            _ => Ok(self.invalid_call_with_arg_range("OperatorApplied", args, 1, 3)),
        }
    }

    pub(super) fn eval_curry_applied(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [spec] => {
                if parse_curried_operator_spec(spec).is_some() {
                    Ok(unary_operator_form_slot_first("CurryApplied", spec))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "CurryApplied",
                        args,
                        "expected a non-negative integer, an index list, or k -> index-list as the specification argument",
                    ))
                }
            }
            [function, spec] if parse_curried_operator_spec(spec).is_some() => Ok(Expr::call(
                system_expr("CurryApplied"),
                vec![function.clone(), spec.clone()],
            )),
            [function, spec, accumulated]
                if parse_curried_operator_spec(spec).is_some()
                    && parse_callable_wrapper_accumulated(accumulated).is_some() =>
            {
                Ok(Expr::call(
                    system_expr("CurryApplied"),
                    vec![function.clone(), spec.clone(), accumulated.clone()],
                ))
            }
            [_, _] | [_, _, _] => Ok(self.invalid_call_with_arg_form(
                "CurryApplied",
                args,
                "expected a non-negative integer, an index list, or k -> index-list as the specification argument",
            )),
            _ => Ok(self.invalid_call_with_arg_range("CurryApplied", args, 1, 3)),
        }
    }

    pub(super) fn eval_reverse_applied(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [count] if expr_as_non_negative_usize(count).is_some() => {
                Ok(unary_operator_form_slot_first("ReverseApplied", count))
            }
            [function] => Ok(Expr::call(head.clone(), vec![function.clone()])),
            [function, count] if expr_as_non_negative_usize(count).is_some() => Ok(Expr::call(
                head.clone(),
                vec![function.clone(), count.clone()],
            )),
            [_, _] => Ok(self.invalid_call_with_arg_form(
                "ReverseApplied",
                args,
                "expected a non-negative integer as the second argument",
            )),
            _ => Ok(self.invalid_call_with_arg_range("ReverseApplied", args, 1, 2)),
        }
    }

    pub(super) fn eval_composition(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(simplify_composition_head(head, args))
    }

    pub(super) fn eval_right_composition(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(simplify_composition_head(head, args))
    }

    pub(super) fn eval_through(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Through", args, 1, 2));
        }

        let target = args.get(1);
        self.evaluate(through_expression(&args[0], target))
    }

    pub(super) fn eval_operate(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(unary_operator_form("Operate", operator)),
            [operator, level_expr] if expr_as_non_negative_usize(level_expr).is_some() => Ok(
                unary_operator_form_with_tail("Operate", operator.clone(), level_expr.clone()),
            ),
            [operator, expr] => self.evaluate(operate_on_head_level(expr, operator, 1)),
            [operator, expr, level_expr] => {
                let Some(level) = expr_as_non_negative_usize(level_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Operate",
                        args,
                        "expected a non-negative integer as the third argument",
                    ));
                };

                self.evaluate(operate_on_head_level(expr, operator, level))
            }
            _ => Ok(self.invalid_call_with_arg_range("Operate", args, 1, 3)),
        }
    }

    pub(super) fn eval_array_filter(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (operator, array, radius_spec, padding) = match args {
            [operator, array, radius_spec] => {
                (operator, array, radius_spec, ArrayFilterPadding::Fixed)
            }
            [operator, array, radius_spec, padding_expr] => {
                let Some(padding) = parse_array_filter_padding(padding_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "ArrayFilter",
                        args,
                        "expected Padding -> value, Padding -> None, Padding -> \"Fixed\", or Padding -> \"Periodic\" as the fourth argument",
                    ));
                };
                (operator, array, radius_spec, padding)
            }
            _ => return Ok(self.invalid_call_with_arg_range("ArrayFilter", args, 3, 4)),
        };

        let Some((shape, flat)) = parse_rectangular_list_array(array) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArrayFilter",
                args,
                "expected a rectangular List array as the second argument",
            ));
        };
        let Some(spec) = parse_array_filter_spec(radius_spec, shape.len()) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArrayFilter",
                args,
                "expected a non-negative integer radius, a matching list of non-negative integer radii, or a matching rectangular 0/1 template array as the third argument",
            ));
        };

        apply_array_filter(self, operator, &shape, &flat, &spec, &padding)
    }

    pub(super) fn eval_array_reduce(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [operator, array, dims_spec] = args else {
            return Ok(self.invalid_call_with_arg_range("ArrayReduce", args, 3, 3));
        };

        let Some((shape, flat)) = parse_rectangular_list_array(array) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArrayReduce",
                args,
                "expected a rectangular List array as the second argument",
            ));
        };
        let Some(spec) = parse_array_reduce_spec(dims_spec, shape.len()) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArrayReduce",
                args,
                "expected a positive integer dimension, a dimension Span, a list of dimensions, or a grouped dimension list as the third argument",
            ));
        };

        apply_array_reduce(self, operator, &shape, &flat, &spec)
    }

    fn normalize_apply_call(
        &mut self,
        head: &Expr,
        args: &[Expr],
        flavor: ApplyFlavor,
    ) -> ApplyNormalization {
        let symbol = flavor.symbol();

        if !(1..=flavor.max_args()).contains(&args.len()) {
            return ApplyNormalization::Immediate(self.invalid_call_with_arg_range(
                symbol,
                args,
                1,
                flavor.max_args(),
            ));
        }

        if args.len() == 1 {
            return ApplyNormalization::OperatorForm(unary_operator_form(symbol, &args[0]));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return ApplyNormalization::OperatorForm(unary_operator_form_with_tail(
                symbol,
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let default_levels = flavor.default_levels();

        let (expr, levels, include_heads) = match args {
            [_, expr] => (expr.clone(), default_levels, false),
            [_, expr, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr.clone(), default_levels, include_heads)
                } else if flavor == ApplyFlavor::Apply {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return ApplyNormalization::Immediate(self.invalid_call_with_arg_form(
                            symbol,
                            args,
                            "expected a standard levelspec or Heads -> True/False as the third argument",
                        ));
                    };
                    (expr.clone(), levels, false)
                } else {
                    return ApplyNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a Heads -> True/False option as the third argument",
                    ));
                }
            }
            [_, expr, third, fourth] if flavor == ApplyFlavor::Apply => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return ApplyNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a standard levelspec as the third argument",
                    ));
                };
                let Some(include_heads) = parse_heads_option(fourth) else {
                    return ApplyNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };
                (expr.clone(), levels, include_heads)
            }
            _ => return ApplyNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
        };

        ApplyNormalization::Call {
            operator: args[0].clone(),
            expr,
            config: TraversalConfig {
                levels,
                include_heads,
            },
        }
    }

    fn normalize_map_like_call(
        &mut self,
        head: &Expr,
        args: &[Expr],
        flavor: MapLikeFlavor,
    ) -> MapLikeNormalization {
        let symbol = flavor.symbol();

        if !(1..=4).contains(&args.len()) {
            return MapLikeNormalization::Immediate(
                self.invalid_call_with_arg_range(symbol, args, 1, 4),
            );
        }

        if args.len() == 1 {
            return MapLikeNormalization::OperatorForm(unary_operator_form(symbol, &args[0]));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return MapLikeNormalization::OperatorForm(unary_operator_form_with_tail(
                symbol,
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let (expr, levels, include_heads) = match args {
            [_, expr] => (expr.clone(), CasesLevelSpec::exact(1), false),
            [_, expr, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr.clone(), CasesLevelSpec::exact(1), include_heads)
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return MapLikeNormalization::Immediate(self.invalid_call_with_arg_form(
                            symbol,
                            args,
                            "expected a standard levelspec or Heads -> True/False as the third argument",
                        ));
                    };
                    (expr.clone(), levels, false)
                }
            }
            [_, expr, third, fourth] => {
                let Some(levels) = parse_standard_level_spec(third) else {
                    return MapLikeNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a standard levelspec as the third argument",
                    ));
                };
                let Some(include_heads) = parse_heads_option(fourth) else {
                    return MapLikeNormalization::Immediate(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };
                (expr.clone(), levels, include_heads)
            }
            _ => return MapLikeNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
        };

        MapLikeNormalization::Call {
            operator: args[0].clone(),
            expr,
            config: TraversalConfig {
                levels,
                include_heads,
            },
        }
    }

    fn normalize_map_all_call(&mut self, head: &Expr, args: &[Expr]) -> MapLikeNormalization {
        if !(1..=3).contains(&args.len()) {
            return MapLikeNormalization::Immediate(
                self.invalid_call_with_arg_range("MapAll", args, 1, 3),
            );
        }

        if args.len() == 1 {
            return MapLikeNormalization::OperatorForm(unary_operator_form("MapAll", &args[0]));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return MapLikeNormalization::OperatorForm(unary_operator_form_with_tail(
                "MapAll",
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let include_heads = match args {
            [_, _] => false,
            [_, _, option] => {
                let Some(include_heads) = parse_heads_option(option) else {
                    return MapLikeNormalization::Immediate(self.invalid_call_with_arg_form(
                        "MapAll",
                        args,
                        "expected a Heads -> True/False option as the third argument",
                    ));
                };
                include_heads
            }
            _ => return MapLikeNormalization::Immediate(Expr::call(head.clone(), args.to_vec())),
        };

        MapLikeNormalization::Call {
            operator: args[0].clone(),
            expr: args[1].clone(),
            config: TraversalConfig {
                levels: CasesLevelSpec::range(0, None),
                include_heads,
            },
        }
    }

    fn normalize_comap_call(&mut self, head: &Expr, args: &[Expr]) -> CommonArgMapNormalization {
        if !(1..=4).contains(&args.len()) {
            return CommonArgMapNormalization::Immediate(
                self.invalid_call_with_arg_range("Comap", args, 1, 4),
            );
        }

        if args.len() == 1 {
            return CommonArgMapNormalization::OperatorForm(unary_operator_form("Comap", &args[0]));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return CommonArgMapNormalization::OperatorForm(unary_operator_form_with_tail(
                "Comap",
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let (functions, common_expr, levels, include_heads) = match args {
            [functions, common_expr] => (
                functions.clone(),
                common_expr.clone(),
                CasesLevelSpec::exact(1),
                false,
            ),
            [functions, common_expr, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (
                        functions.clone(),
                        common_expr.clone(),
                        CasesLevelSpec::exact(1),
                        include_heads,
                    )
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return CommonArgMapNormalization::Immediate(
                            self.invalid_call_with_arg_form(
                                "Comap",
                                args,
                                "expected a standard levelspec or Heads -> True/False as the third argument",
                            ),
                        );
                    };
                    (functions.clone(), common_expr.clone(), levels, false)
                }
            }
            [functions, common_expr, levels_expr, option] => {
                let Some(levels) = parse_standard_level_spec(levels_expr) else {
                    return CommonArgMapNormalization::Immediate(self.invalid_call_with_arg_form(
                        "Comap",
                        args,
                        "expected a standard levelspec as the third argument",
                    ));
                };
                let Some(include_heads) = parse_heads_option(option) else {
                    return CommonArgMapNormalization::Immediate(self.invalid_call_with_arg_form(
                        "Comap",
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };
                (
                    functions.clone(),
                    common_expr.clone(),
                    levels,
                    include_heads,
                )
            }
            _ => {
                return CommonArgMapNormalization::Immediate(Expr::call(
                    head.clone(),
                    args.to_vec(),
                ))
            }
        };

        CommonArgMapNormalization::Call {
            functions,
            common_expr,
            config: TraversalConfig {
                levels,
                include_heads,
            },
        }
    }

    fn normalize_comap_apply_call(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> CommonArgMapNormalization {
        if !(1..=3).contains(&args.len()) {
            return CommonArgMapNormalization::Immediate(self.invalid_call_with_arg_range(
                "ComapApply",
                args,
                1,
                3,
            ));
        }

        if args.len() == 1 {
            return CommonArgMapNormalization::OperatorForm(unary_operator_form(
                "ComapApply",
                &args[0],
            ));
        }

        if args.len() == 2 && parse_heads_option(&args[1]).is_some() {
            return CommonArgMapNormalization::OperatorForm(unary_operator_form_with_tail(
                "ComapApply",
                args[0].clone(),
                args[1].clone(),
            ));
        }

        let include_heads = match args {
            [_, _] => false,
            [_, _, option] => {
                let Some(include_heads) = parse_heads_option(option) else {
                    return CommonArgMapNormalization::Immediate(self.invalid_call_with_arg_form(
                        "ComapApply",
                        args,
                        "expected a Heads -> True/False option as the third argument",
                    ));
                };
                include_heads
            }
            _ => {
                return CommonArgMapNormalization::Immediate(Expr::call(
                    head.clone(),
                    args.to_vec(),
                ))
            }
        };

        CommonArgMapNormalization::Call {
            functions: args[0].clone(),
            common_expr: args[1].clone(),
            config: TraversalConfig {
                levels: CasesLevelSpec::exact(1),
                include_heads,
            },
        }
    }

    fn finish_map_thread(
        &mut self,
        head: &Expr,
        args: &[Expr],
        operator: &Expr,
        expr: &Expr,
        level_expr: Expr,
        level: usize,
    ) -> Result<Expr, EvalError> {
        let Some(inputs) = call_items_with_head(expr, &system_expr("List")) else {
            return Ok(self.invalid_call_with_arg_form(
                "MapThread",
                args,
                "expected a list of expressions as the second argument",
            ));
        };

        match map_thread_inputs(self, operator, inputs, level)? {
            MapThreadDispatch::Threaded(threaded) => Ok(threaded),
            MapThreadDispatch::NotApplicable | MapThreadDispatch::Mismatch => Ok(if level == 1 {
                Expr::call(head.clone(), vec![operator.clone(), expr.clone()])
            } else {
                Expr::call(
                    head.clone(),
                    vec![operator.clone(), expr.clone(), level_expr],
                )
            }),
        }
    }

    fn evaluate_scan_parts(
        &mut self,
        operator: &Expr,
        parts: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        for part in parts {
            let result = self.evaluate(Expr::call(operator.clone(), vec![part]));
            self.suspend_with_frame(result, |_, value| Ok(value))?;
        }

        Ok(system_expr("Null"))
    }

    fn apply_map_at_paths(
        &mut self,
        operator: &Expr,
        expr: Expr,
        paths: Vec<Vec<isize>>,
    ) -> Result<Expr, EvalError> {
        let mut current = expr;

        for path in paths {
            let Some(part) = exact_part_at_path(&current, &path) else {
                continue;
            };

            let mapped = self.evaluate(Expr::call(operator.clone(), vec![part]))?;
            current = replace_exact_path(current, &path, mapped);
        }

        Ok(current)
    }

    fn take_while_sequence(
        &mut self,
        sequence: SequenceLikeExpr,
        criterion: &Expr,
    ) -> Result<Expr, EvalError> {
        match sequence {
            SequenceLikeExpr::Call { head, items } => {
                let mut taken = Vec::new();
                for item in items {
                    if !criterion_accepts(self, criterion, &item)? {
                        break;
                    }
                    taken.push(item);
                }

                Ok(Expr::call(head, taken))
            }
            SequenceLikeExpr::Association(entries) => {
                let mut taken = Vec::new();
                for (key, value) in entries {
                    if !criterion_accepts(self, criterion, &value)? {
                        break;
                    }
                    taken.push((key, value));
                }

                Ok(render_association(taken))
            }
        }
    }

    fn length_while_sequence(
        &mut self,
        sequence: SequenceLikeExpr,
        criterion: &Expr,
    ) -> Result<usize, EvalError> {
        match sequence {
            SequenceLikeExpr::Call { items, .. } => {
                let mut count = 0usize;
                for item in items {
                    if !criterion_accepts(self, criterion, &item)? {
                        break;
                    }
                    count += 1;
                }
                Ok(count)
            }
            SequenceLikeExpr::Association(entries) => {
                let mut count = 0usize;
                for (_, value) in entries {
                    if !criterion_accepts(self, criterion, &value)? {
                        break;
                    }
                    count += 1;
                }
                Ok(count)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ApplyFlavor {
    Apply,
    MapApply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapLikeFlavor {
    Map,
    Scan,
    MapIndexed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapTraversalFlavor {
    Map,
    MapAll,
}

enum ApplyNormalization {
    OperatorForm(Expr),
    Call {
        operator: Expr,
        expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

enum MapLikeNormalization {
    OperatorForm(Expr),
    Call {
        operator: Expr,
        expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

enum CommonArgMapNormalization {
    OperatorForm(Expr),
    Call {
        functions: Expr,
        common_expr: Expr,
        config: TraversalConfig,
    },
    Immediate(Expr),
}

impl ApplyFlavor {
    fn symbol(self) -> &'static str {
        match self {
            Self::Apply => "Apply",
            Self::MapApply => "MapApply",
        }
    }

    fn default_levels(self) -> CasesLevelSpec {
        match self {
            Self::Apply => CasesLevelSpec::exact(0),
            Self::MapApply => CasesLevelSpec::exact(1),
        }
    }

    fn max_args(self) -> usize {
        match self {
            Self::Apply => 4,
            Self::MapApply => 3,
        }
    }
}

impl MapLikeFlavor {
    fn symbol(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Scan => "Scan",
            Self::MapIndexed => "MapIndexed",
        }
    }
}

enum MapThreadDispatch {
    Threaded(Expr),
    NotApplicable,
    Mismatch,
}

enum SequenceLikeExpr {
    Call { head: Expr, items: Vec<Expr> },
    Association(Vec<(Expr, Expr)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TensorNode {
    Scalar(Expr),
    Array(Vec<TensorNode>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArrayFilterPadding {
    Fixed,
    Periodic,
    Constant(Expr),
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArrayFilterSpec {
    Radii(Vec<usize>),
    Template(ArrayFilterTemplate),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArrayFilterTemplate {
    shape: Vec<usize>,
    center: Vec<usize>,
    mask: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArrayReduceSpec {
    Elementwise,
    Grouped(Vec<Vec<usize>>),
}

fn slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}

fn slot_sequence_expr() -> Expr {
    Expr::call(
        system_expr("SlotSequence"),
        vec![Expr::integer(BigInt::from(1_u8))],
    )
}

fn unary_operator_form(symbol: &str, operator: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![operator.clone(), slot_expr()],
        )],
    )
}

fn unary_operator_form_slot_first(symbol: &str, tail: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![slot_expr(), tail.clone()],
        )],
    )
}

fn unary_operator_form_with_tail(symbol: &str, operator: Expr, tail: Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![operator, slot_expr(), tail],
        )],
    )
}

fn unary_operator_form_with_tails(symbol: &str, operator: Expr, tails: Vec<Expr>) -> Expr {
    let mut args = vec![operator, slot_expr()];
    args.extend(tails);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), args)],
    )
}

fn variadic_operator_form(symbol: &str, mut fixed_args: Vec<Expr>) -> Expr {
    fixed_args.push(slot_sequence_expr());
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), fixed_args)],
    )
}

fn variadic_operator_form_with_suffix(
    symbol: &str,
    mut prefix_args: Vec<Expr>,
    suffix_args: Vec<Expr>,
) -> Expr {
    prefix_args.push(slot_sequence_expr());
    prefix_args.extend(suffix_args);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), prefix_args)],
    )
}

fn function_body_operator(body: Expr) -> Expr {
    Expr::call(system_expr("Function"), vec![body])
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    let value = expr_as_non_negative_usize(expr)?;
    (value > 0).then_some(value)
}

fn parse_curried_operator(expr: &Expr) -> Option<CurriedOperatorSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "OperatorApplied") => {
            match args.as_slice() {
                [function] => Some(CurriedOperatorSpec {
                    kind: CurriedOperatorKind::OperatorApplied,
                    function,
                    arity: 2,
                    order: vec![1, 0],
                    accumulated: Vec::new(),
                }),
                [function, spec] => {
                    let (arity, order) = parse_curried_operator_spec(spec)?;
                    Some(CurriedOperatorSpec {
                        kind: CurriedOperatorKind::OperatorApplied,
                        function,
                        arity,
                        order,
                        accumulated: Vec::new(),
                    })
                }
                [function, spec, accumulated] => {
                    let (arity, order) = parse_curried_operator_spec(spec)?;
                    Some(CurriedOperatorSpec {
                        kind: CurriedOperatorKind::OperatorApplied,
                        function,
                        arity,
                        order,
                        accumulated: parse_callable_wrapper_accumulated(accumulated)?,
                    })
                }
                _ => None,
            }
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "CurryApplied") => {
            match args.as_slice() {
                [function, spec] => {
                    let (arity, order) = parse_curried_operator_spec(spec)?;
                    Some(CurriedOperatorSpec {
                        kind: CurriedOperatorKind::CurryApplied,
                        function,
                        arity,
                        order,
                        accumulated: Vec::new(),
                    })
                }
                [function, spec, accumulated] => {
                    let (arity, order) = parse_curried_operator_spec(spec)?;
                    Some(CurriedOperatorSpec {
                        kind: CurriedOperatorKind::CurryApplied,
                        function,
                        arity,
                        order,
                        accumulated: parse_callable_wrapper_accumulated(accumulated)?,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn parse_curried_operator_spec(spec: &Expr) -> Option<(usize, Vec<usize>)> {
    if let Some(arity) = expr_as_non_negative_usize(spec) {
        return Some((arity, (0..arity).collect()));
    }

    if let Some(order) = expr_as_index_list(spec) {
        let arity = order.iter().copied().max().map_or(0usize, |max| max + 1);
        return Some((arity, order));
    }

    match spec {
        Expr::Call { head, args } if is_system_symbol_name(head, "Rule") && args.len() == 2 => {
            let arity = expr_as_non_negative_usize(&args[0])?;
            let order = expr_as_index_list(&args[1])?;
            if order.iter().any(|index| *index >= arity) {
                return None;
            }
            Some((arity, order))
        }
        _ => None,
    }
}

fn expr_as_index_list(expr: &Expr) -> Option<Vec<usize>> {
    let items = call_items_with_head(expr, &system_expr("List"))?;
    items
        .iter()
        .map(|item| {
            let one_based = expr_as_non_negative_usize(item)?;
            if one_based == 0 {
                None
            } else {
                Some(one_based - 1)
            }
        })
        .collect()
}

fn parse_callable_wrapper_accumulated(expr: &Expr) -> Option<Vec<Expr>> {
    call_items_with_head(expr, &system_expr("List")).map(|items| items.to_vec())
}

fn parse_reverse_applied_wrapper(expr: &Expr) -> Option<ReverseAppliedSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "ReverseApplied") => {
            match args.as_slice() {
                [function] => Some(ReverseAppliedSpec {
                    function,
                    reversed_count: None,
                }),
                [function, count] => Some(ReverseAppliedSpec {
                    function,
                    reversed_count: Some(expr_as_non_negative_usize(count)?),
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

fn apply_curried_operator_to_args(spec: &CurriedOperatorSpec<'_>, args: &[Expr]) -> Expr {
    let mut collected = spec.accumulated.clone();
    collected.extend(args.iter().cloned());

    if collected.len() < spec.arity {
        return render_curried_operator_expr(
            spec.kind,
            spec.function,
            spec.arity,
            &spec.order,
            &collected,
        );
    }

    let ordered = spec
        .order
        .iter()
        .filter_map(|index| collected.get(*index).cloned())
        .collect::<Vec<_>>();
    let applied = Expr::call(spec.function.clone(), ordered);
    let remaining = collected.into_iter().skip(spec.arity).collect::<Vec<_>>();

    if remaining.is_empty() {
        applied
    } else {
        Expr::call(applied, remaining)
    }
}

fn apply_reverse_applied_to_args(spec: &ReverseAppliedSpec<'_>, args: &[Expr]) -> Expr {
    let reverse_count = spec.reversed_count.unwrap_or(args.len()).min(args.len());
    let mut reordered = args[..reverse_count].to_vec();
    reordered.reverse();
    reordered.extend_from_slice(&args[reverse_count..]);
    Expr::call(spec.function.clone(), reordered)
}

fn render_curried_operator_expr(
    kind: CurriedOperatorKind,
    function: &Expr,
    arity: usize,
    order: &[usize],
    accumulated: &[Expr],
) -> Expr {
    if kind == CurriedOperatorKind::OperatorApplied
        && arity == 2
        && order == [1usize, 0usize]
        && accumulated.is_empty()
    {
        return Expr::call(system_expr("OperatorApplied"), vec![function.clone()]);
    }

    let mut args = vec![
        function.clone(),
        render_curried_operator_spec_expr(arity, order),
    ];
    if !accumulated.is_empty() {
        args.push(Expr::call(system_expr("List"), accumulated.to_vec()));
    }

    Expr::call(system_expr(kind.symbol()), args)
}

fn render_curried_operator_spec_expr(arity: usize, order: &[usize]) -> Expr {
    let max_index = order.iter().copied().max().map_or(0usize, |max| max + 1);
    let index_list = Expr::call(
        system_expr("List"),
        order
            .iter()
            .map(|index| Expr::integer(BigInt::from(index + 1)))
            .collect(),
    );

    if order.iter().enumerate().all(|(index, item)| *item == index) {
        Expr::integer(BigInt::from(arity))
    } else if arity == max_index {
        index_list
    } else {
        Expr::call(
            system_expr("Rule"),
            vec![Expr::integer(BigInt::from(arity)), index_list],
        )
    }
}

fn nest_operator_form(symbol: &str, operator: &Expr, count: usize) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![
                operator.clone(),
                slot_expr(),
                Expr::integer(BigInt::from(count)),
            ],
        )],
    )
}

fn simplify_composition_head(head: &Expr, args: &[Expr]) -> Expr {
    let normalized = args
        .iter()
        .filter(|arg| !is_system_symbol_name(arg, "Identity"))
        .cloned()
        .collect::<Vec<_>>();

    match normalized.as_slice() {
        [] => system_expr("Identity"),
        [single] => single.clone(),
        _ => Expr::call(head.clone(), normalized),
    }
}

fn through_expression(expr: &Expr, target: Option<&Expr>) -> Expr {
    let Expr::Call { head, args } = expr else {
        return expr.clone();
    };

    let Some((transformed, matched)) = through_head_expression(head, args, target) else {
        return expr.clone();
    };

    if matched {
        transformed
    } else {
        expr.clone()
    }
}

fn through_head_expression(
    head_expr: &Expr,
    call_args: &[Expr],
    target: Option<&Expr>,
) -> Option<(Expr, bool)> {
    let Expr::Call { head, args } = head_expr else {
        return None;
    };

    if target.is_none_or(|target_head| head.as_ref() == target_head) {
        return Some((
            Expr::call(
                head.as_ref().clone(),
                args.iter()
                    .map(|arg| Expr::call(arg.clone(), call_args.to_vec()))
                    .collect(),
            ),
            true,
        ));
    }

    let mut matched = false;
    let transformed_args = args
        .iter()
        .map(|arg| {
            if let Some((transformed, subtree_matched)) =
                through_head_expression(arg, call_args, target)
            {
                matched |= subtree_matched;
                transformed
            } else {
                arg.clone()
            }
        })
        .collect();

    Some((Expr::call(head.as_ref().clone(), transformed_args), matched))
}

fn operate_on_head_level(expr: &Expr, operator: &Expr, level: usize) -> Expr {
    if level == 0 {
        return Expr::call(operator.clone(), vec![expr.clone()]);
    }

    match expr {
        Expr::Call { head, args } => {
            if level == 1 {
                Expr::call(
                    Expr::call(operator.clone(), vec![head.as_ref().clone()]),
                    args.clone(),
                )
            } else {
                let replaced_head = operate_on_head_level(head, operator, level - 1);
                if replaced_head == **head {
                    expr.clone()
                } else {
                    Expr::call(replaced_head, args.clone())
                }
            }
        }
        Expr::Atom(_) => expr.clone(),
    }
}

fn parse_array_filter_padding(expr: &Expr) -> Option<ArrayFilterPadding> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    let symbol = expr_as_symbol(&args[0])?;
    if symbol.name() != "Padding" {
        return None;
    }

    match &args[1] {
        value if expr_as_symbol(value).is_some_and(|symbol| symbol.name() == "None") => {
            Some(ArrayFilterPadding::None)
        }
        Expr::Atom(Atom::String(value)) if value == "Fixed" => Some(ArrayFilterPadding::Fixed),
        Expr::Atom(Atom::String(value)) if value == "Periodic" => {
            Some(ArrayFilterPadding::Periodic)
        }
        other => Some(ArrayFilterPadding::Constant(other.clone())),
    }
}

fn parse_rectangular_list_array(expr: &Expr) -> Option<(Vec<usize>, Vec<Expr>)> {
    let Expr::Call { head, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    let tensor = parse_tensor_node(expr, &system_expr("List"));
    let shape = tensor_shape(&tensor)?;
    let flat = flatten_tensor_scalars(&tensor);
    Some((shape, flat))
}

fn parse_array_filter_spec(expr: &Expr, rank: usize) -> Option<ArrayFilterSpec> {
    if let Some(radius) = expr_as_non_negative_usize(expr) {
        return Some(ArrayFilterSpec::Radii(vec![radius; rank]));
    }

    if let Some(items) = call_items_with_head(expr, &system_expr("List")) {
        if items.len() == rank {
            if let Some(radii) = items.iter().map(expr_as_non_negative_usize).collect() {
                return Some(ArrayFilterSpec::Radii(radii));
            }
        }
    }

    parse_array_filter_template(expr, rank).map(ArrayFilterSpec::Template)
}

fn parse_array_filter_template(expr: &Expr, rank: usize) -> Option<ArrayFilterTemplate> {
    let (shape, flat) = parse_rectangular_list_array(expr)?;
    if shape.len() != rank || shape.contains(&0) {
        return None;
    }

    let mask = flat
        .into_iter()
        .map(|item| match item {
            Expr::Atom(Atom::Integer(value)) if value == BigInt::from(1_u8) => Some(true),
            Expr::Atom(Atom::Integer(value)) if value == BigInt::from(0_u8) => Some(false),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let center = shape.iter().map(|len| len / 2).collect();

    Some(ArrayFilterTemplate {
        shape,
        center,
        mask,
    })
}

fn parse_array_reduce_spec(expr: &Expr, rank: usize) -> Option<ArrayReduceSpec> {
    if let Some(dimension) = expr_as_positive_usize(expr) {
        return (dimension <= rank).then_some(ArrayReduceSpec::Grouped(vec![vec![dimension - 1]]));
    }

    if let Some(span) = parse_array_reduce_span(expr, rank) {
        return Some(ArrayReduceSpec::Grouped(vec![span]));
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    if items.is_empty() {
        return Some(ArrayReduceSpec::Elementwise);
    }

    if items
        .iter()
        .all(|item| call_items_with_head(item, &system_expr("List")).is_some())
    {
        let mut seen = std::collections::HashSet::new();
        let mut groups = Vec::with_capacity(items.len());
        for group_expr in items {
            let group_items = call_items_with_head(group_expr, &system_expr("List"))?;
            if group_items.is_empty() {
                return None;
            }

            let mut group = Vec::with_capacity(group_items.len());
            for item in group_items {
                let dimension = expr_as_positive_usize(item)?;
                if dimension > rank {
                    return None;
                }
                let zero_based = dimension - 1;
                if !seen.insert(zero_based) {
                    return None;
                }
                group.push(zero_based);
            }
            group.sort_unstable();
            groups.push(group);
        }

        return Some(ArrayReduceSpec::Grouped(groups));
    }

    let mut seen = std::collections::HashSet::new();
    let mut dimensions = Vec::with_capacity(items.len());
    for item in items {
        let dimension = expr_as_positive_usize(item)?;
        if dimension > rank {
            return None;
        }
        let zero_based = dimension - 1;
        if !seen.insert(zero_based) {
            return None;
        }
        dimensions.push(zero_based);
    }
    dimensions.sort_unstable();
    Some(ArrayReduceSpec::Grouped(vec![dimensions]))
}

fn parse_array_reduce_span(expr: &Expr, rank: usize) -> Option<Vec<usize>> {
    let items = call_items_with_head(expr, &system_expr("Span"))?;

    let (start, end) = match items {
        [end] => (1usize, parse_array_reduce_span_endpoint(end, rank, rank)?),
        [start, end] => (
            parse_array_reduce_span_endpoint(start, 1, rank)?,
            parse_array_reduce_span_endpoint(end, rank, rank)?,
        ),
        [start, end, step] if expr_as_positive_usize(step) == Some(1) => (
            parse_array_reduce_span_endpoint(start, 1, rank)?,
            parse_array_reduce_span_endpoint(end, rank, rank)?,
        ),
        _ => return None,
    };

    (start <= end).then(|| ((start - 1)..end).collect())
}

fn parse_array_reduce_span_endpoint(expr: &Expr, all_value: usize, rank: usize) -> Option<usize> {
    if is_system_symbol_name(expr, "All") {
        return Some(all_value);
    }

    let value = expr_as_positive_usize(expr)?;
    (value <= rank).then_some(value)
}

fn array_filter_window_shape(spec: &ArrayFilterSpec) -> Vec<usize> {
    match spec {
        ArrayFilterSpec::Radii(radii) => radii
            .iter()
            .map(|radius| radius.saturating_mul(2).saturating_add(1))
            .collect(),
        ArrayFilterSpec::Template(template) => template.shape.clone(),
    }
}

fn array_filter_window_center(spec: &ArrayFilterSpec) -> Vec<usize> {
    match spec {
        ArrayFilterSpec::Radii(radii) => radii.clone(),
        ArrayFilterSpec::Template(template) => template.center.clone(),
    }
}

fn array_filter_template_includes(spec: &ArrayFilterSpec, window_index: &[usize]) -> bool {
    match spec {
        ArrayFilterSpec::Radii(_) => true,
        ArrayFilterSpec::Template(template) => {
            let offset = tensor_flat_offset(&template.shape, window_index)
                .expect("template positions should follow the template shape");
            template.mask[offset]
        }
    }
}

fn apply_heads_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
) -> (Expr, usize) {
    apply_heads_recursive(expr, operator, config, flavor, 0)
}

fn apply_heads_recursive(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
    level: usize,
) -> (Expr, usize) {
    match expr {
        Expr::Atom(_) => (expr.clone(), 1),
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 0usize;
                let mut transformed_entries = Vec::with_capacity(entries.len());

                for (key, value) in entries {
                    let (transformed_value, value_depth) =
                        apply_heads_recursive(&value, operator, config, flavor, level + 1);
                    max_depth = max_depth.max(value_depth);
                    transformed_entries.push((key, transformed_value));
                }

                let depth = 1 + max_depth;
                let rendered =
                    if config.levels.matches(level, depth) && flavor == ApplyFlavor::Apply {
                        Expr::call(
                            operator.clone(),
                            transformed_entries
                                .iter()
                                .map(|(_, value)| value.clone())
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        render_association(transformed_entries)
                    };

                (rendered, depth)
            } else {
                apply_heads_on_standard_call(expr, operator, config, flavor, level)
            }
        }
        Expr::Call { .. } => apply_heads_on_standard_call(expr, operator, config, flavor, level),
    }
}

fn apply_heads_on_standard_call(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
    level: usize,
) -> (Expr, usize) {
    let Expr::Call { head, args } = expr else {
        return (expr.clone(), 1);
    };

    let (transformed_head, head_depth) = if config.include_heads {
        apply_heads_recursive(head, operator, config, flavor, level + 1)
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 0usize;
    for arg in args {
        let (transformed_arg, arg_depth) =
            apply_heads_recursive(arg, operator, config, flavor, level + 1);
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = 1 + head_depth.max(max_arg_depth);
    let rebuilt = Expr::call(transformed_head, transformed_args.clone());

    if config.levels.matches(level, depth) {
        (Expr::call(operator.clone(), transformed_args), depth)
    } else {
        (rebuilt, depth)
    }
}

fn map_parts_with_config(expr: &Expr, operator: &Expr, config: TraversalConfig) -> (Expr, usize) {
    map_parts_with_mode(expr, operator, config, MapTraversalFlavor::Map)
}

fn map_parts_with_mode(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
) -> (Expr, usize) {
    map_parts_recursive(expr, operator, config, flavor, 0)
}

fn map_parts_recursive(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
    level: usize,
) -> (Expr, usize) {
    match expr {
        Expr::Atom(_) => {
            let depth = 1;
            let rendered = if config.levels.matches(level, depth) {
                Expr::call(operator.clone(), vec![expr.clone()])
            } else {
                expr.clone()
            };

            (rendered, depth)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 0usize;
                let mut transformed_entries = Vec::with_capacity(entries.len());

                for (key, value) in entries {
                    let (transformed_value, value_depth) =
                        map_parts_recursive(&value, operator, config, flavor, level + 1);
                    max_depth = max_depth.max(value_depth);
                    transformed_entries.push((key, transformed_value));
                }

                let depth = 1 + max_depth;
                let rebuilt = render_association(transformed_entries);
                let rendered = if config.levels.matches(level, depth)
                    && !(flavor == MapTraversalFlavor::MapAll
                        && association_entries_from_expr(expr).is_some())
                {
                    Expr::call(operator.clone(), vec![rebuilt])
                } else {
                    rebuilt
                };

                (rendered, depth)
            } else {
                map_parts_on_standard_call(expr, operator, config, flavor, level)
            }
        }
        Expr::Call { .. } => map_parts_on_standard_call(expr, operator, config, flavor, level),
    }
}

fn map_parts_on_standard_call(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
    level: usize,
) -> (Expr, usize) {
    let Expr::Call { head, args } = expr else {
        return (expr.clone(), 1);
    };

    let (transformed_head, head_depth) = if config.include_heads {
        map_parts_recursive(head, operator, config, flavor, level + 1)
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 0usize;
    for arg in args {
        let (transformed_arg, arg_depth) =
            map_parts_recursive(arg, operator, config, flavor, level + 1);
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = 1 + head_depth.max(max_arg_depth);
    let rebuilt = Expr::call(transformed_head, transformed_args);
    let rendered = if config.levels.matches(level, depth) {
        Expr::call(operator.clone(), vec![rebuilt])
    } else {
        rebuilt
    };

    (rendered, depth)
}

fn map_indexed_parts_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
) -> (Expr, usize) {
    map_indexed_parts_recursive(expr, operator, config, 0, &[])
}

fn map_indexed_parts_recursive(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    level: usize,
    path: &[isize],
) -> (Expr, usize) {
    match expr {
        Expr::Atom(_) => {
            let depth = 1;
            let rendered = if config.levels.matches(level, depth) {
                Expr::call(
                    operator.clone(),
                    vec![expr.clone(), position_expr_from_path(path)],
                )
            } else {
                expr.clone()
            };
            (rendered, depth)
        }
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 0usize;
                let mut rebuilt_entries = Vec::with_capacity(entries.len());

                for (index, (key, value)) in entries.into_iter().enumerate() {
                    let child_path = extend_isize_path(path, index as isize + 1);
                    let (mapped_value, child_depth) = map_indexed_parts_recursive(
                        &value,
                        operator,
                        config,
                        level + 1,
                        &child_path,
                    );
                    max_depth = max_depth.max(child_depth);
                    rebuilt_entries.push((key, mapped_value));
                }

                let depth = 1 + max_depth;
                let rebuilt = render_association(rebuilt_entries);
                let rendered = if config.levels.matches(level, depth) {
                    Expr::call(
                        operator.clone(),
                        vec![rebuilt, position_expr_from_path(path)],
                    )
                } else {
                    rebuilt
                };

                (rendered, depth)
            } else {
                map_indexed_parts_on_standard_call(expr, operator, config, level, path)
            }
        }
        Expr::Call { .. } => {
            map_indexed_parts_on_standard_call(expr, operator, config, level, path)
        }
    }
}

fn map_indexed_parts_on_standard_call(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    level: usize,
    path: &[isize],
) -> (Expr, usize) {
    let Expr::Call { head, args } = expr else {
        return (expr.clone(), 1);
    };

    let (transformed_head, head_depth) = if config.include_heads {
        let child_path = extend_isize_path(path, 0);
        map_indexed_parts_recursive(head, operator, config, level + 1, &child_path)
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 0usize;
    for (index, arg) in args.iter().enumerate() {
        let child_path = extend_isize_path(path, index as isize + 1);
        let (transformed_arg, arg_depth) =
            map_indexed_parts_recursive(arg, operator, config, level + 1, &child_path);
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = 1 + head_depth.max(max_arg_depth);
    let rebuilt = Expr::call(transformed_head, transformed_args);
    let rendered = if config.levels.matches(level, depth) {
        Expr::call(
            operator.clone(),
            vec![rebuilt, position_expr_from_path(path)],
        )
    } else {
        rebuilt
    };

    (rendered, depth)
}

fn collect_scan_parts_with_config(expr: &Expr, config: TraversalConfig) -> Vec<Expr> {
    collect_scan_parts_recursive(expr, config, 0).parts
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScanCollection {
    parts: Vec<Expr>,
    depth: usize,
}

fn collect_scan_parts_recursive(
    expr: &Expr,
    config: TraversalConfig,
    level: usize,
) -> ScanCollection {
    match expr {
        Expr::Atom(_) => {
            let mut parts = Vec::new();
            if config.levels.matches(level, 1) {
                parts.push(expr.clone());
            }

            ScanCollection { parts, depth: 1 }
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut parts = Vec::new();
                let mut max_depth = 0usize;

                for (_, value) in entries {
                    let child = collect_scan_parts_recursive(&value, config, level + 1);
                    max_depth = max_depth.max(child.depth);
                    parts.extend(child.parts);
                }

                let depth = 1 + max_depth;
                if config.levels.matches(level, depth) {
                    parts.push(expr.clone());
                }

                ScanCollection { parts, depth }
            } else {
                collect_scan_parts_on_standard_call(expr, config, level)
            }
        }
        Expr::Call { .. } => collect_scan_parts_on_standard_call(expr, config, level),
    }
}

fn collect_scan_parts_on_standard_call(
    expr: &Expr,
    config: TraversalConfig,
    level: usize,
) -> ScanCollection {
    let Expr::Call { head, args } = expr else {
        return ScanCollection {
            parts: vec![expr.clone()],
            depth: 1,
        };
    };

    let mut parts = Vec::new();
    let mut max_child_depth = 0usize;

    if config.include_heads {
        let head_result = collect_scan_parts_recursive(head, config, level + 1);
        max_child_depth = max_child_depth.max(head_result.depth);
        parts.extend(head_result.parts);
    }

    for arg in args {
        let child = collect_scan_parts_recursive(arg, config, level + 1);
        max_child_depth = max_child_depth.max(child.depth);
        parts.extend(child.parts);
    }

    let depth = 1 + max_child_depth;
    if config.levels.matches(level, depth) {
        parts.push(expr.clone());
    }

    ScanCollection { parts, depth }
}

fn map_thread_inputs(
    session: &mut Session,
    operator: &Expr,
    inputs: &[Expr],
    level: usize,
) -> Result<MapThreadDispatch, EvalError> {
    if level == 1 {
        return Ok(
            match session.thread_call_args(
                operator,
                inputs,
                &system_expr("List"),
                &system_expr("List"),
                &ThreadArgSpec::All,
            )? {
                ThreadingDispatch::Threaded(expr) => MapThreadDispatch::Threaded(expr),
                ThreadingDispatch::NotApplicable => MapThreadDispatch::NotApplicable,
                ThreadingDispatch::Mismatch => MapThreadDispatch::Mismatch,
            },
        );
    }

    let Some(first) = inputs.first() else {
        return Ok(MapThreadDispatch::Threaded(Expr::call(
            system_expr("List"),
            Vec::new(),
        )));
    };

    let Expr::Call {
        head: common_head,
        args: first_args,
    } = first
    else {
        return Ok(MapThreadDispatch::NotApplicable);
    };

    if is_system_symbol_name(common_head, "Association") {
        return Ok(MapThreadDispatch::NotApplicable);
    }

    let mut all_args = vec![first_args.clone()];
    let mut lengths = vec![first_args.len()];

    for input in inputs.iter().skip(1) {
        let Expr::Call { head, args } = input else {
            return Ok(MapThreadDispatch::NotApplicable);
        };

        if head.as_ref() != common_head.as_ref() {
            return Ok(MapThreadDispatch::NotApplicable);
        }

        lengths.push(args.len());
        if args.len() != first_args.len() {
            session.emit_thread_length_message(&system_expr("MapThread"), common_head, &lengths);
            return Ok(MapThreadDispatch::Mismatch);
        }

        all_args.push(args.clone());
    }

    let mut mapped = Vec::with_capacity(first_args.len());
    for index in 0..first_args.len() {
        let child_inputs = all_args
            .iter()
            .map(|args| args[index].clone())
            .collect::<Vec<_>>();

        match map_thread_inputs(session, operator, &child_inputs, level - 1)? {
            MapThreadDispatch::Threaded(expr) => mapped.push(expr),
            MapThreadDispatch::NotApplicable => return Ok(MapThreadDispatch::NotApplicable),
            MapThreadDispatch::Mismatch => return Ok(MapThreadDispatch::Mismatch),
        }
    }

    Ok(MapThreadDispatch::Threaded(Expr::call(
        common_head.as_ref().clone(),
        mapped,
    )))
}

fn exact_part_at_path(expr: &Expr, path: &[isize]) -> Option<Expr> {
    let Some((first, rest)) = path.split_first() else {
        return Some(expr.clone());
    };

    if *first == 0 {
        return exact_part_at_path(&functional_structural_head_expr(expr), rest);
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            let index = normalize_exact_path_component(*first, args.len())?;
            exact_part_at_path(&args[index], rest)
        }
    }
}

fn normalize_exact_path_component(component: isize, len: usize) -> Option<usize> {
    if component == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if component > 0 {
        component - 1
    } else {
        len + component
    };

    (0..len)
        .contains(&normalized)
        .then_some(normalized as usize)
}

fn extend_isize_path(path: &[isize], next: isize) -> Vec<isize> {
    let mut extended = path.to_vec();
    extended.push(next);
    extended
}

fn position_expr_from_path(path: &[isize]) -> Expr {
    Expr::call(
        system_expr("List"),
        path.iter()
            .map(|component| Expr::integer(BigInt::from(*component)))
            .collect(),
    )
}

fn functional_structural_head_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Atom(Atom::Integer(_)) => system_expr("Integer"),
        Expr::Atom(Atom::Rational(_)) => system_expr("Rational"),
        Expr::Atom(Atom::String(_)) => system_expr("String"),
        Expr::Atom(Atom::Symbol(_)) => system_expr("Symbol"),
        Expr::Call { head, .. } => head.as_ref().clone(),
    }
}

fn sequence_like_from_expr(expr: &Expr) -> Option<SequenceLikeExpr> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(SequenceLikeExpr::Association(entries));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    Some(SequenceLikeExpr::Call {
        head: head.as_ref().clone(),
        items: args.clone(),
    })
}

fn criterion_accepts(
    session: &mut Session,
    criterion: &Expr,
    item: &Expr,
) -> Result<bool, EvalError> {
    let value = session.evaluate(Expr::call(criterion.clone(), vec![item.clone()]))?;
    Ok(truthiness_symbol(&value) == Some(true))
}

fn accumulate_sequence(
    session: &mut Session,
    sequence: SequenceLikeExpr,
) -> Result<Expr, EvalError> {
    match sequence {
        SequenceLikeExpr::Call { head, items } => {
            let accumulated = accumulate_items(session, items)?;
            Ok(Expr::call(head, accumulated))
        }
        SequenceLikeExpr::Association(entries) => {
            let keys = entries
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>();
            let values = entries
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>();
            let accumulated = accumulate_items(session, values)?;
            Ok(render_association(
                keys.into_iter().zip(accumulated).collect(),
            ))
        }
    }
}

fn accumulate_items(session: &mut Session, items: Vec<Expr>) -> Result<Vec<Expr>, EvalError> {
    let Some((first, rest)) = items.split_first() else {
        return Ok(Vec::new());
    };

    let mut accumulated = vec![first.clone()];
    let mut current = first.clone();
    for item in rest {
        current = session.evaluate(Expr::call(system_expr("Plus"), vec![current, item.clone()]))?;
        accumulated.push(current.clone());
    }

    Ok(accumulated)
}

fn apply_array_filter(
    session: &mut Session,
    operator: &Expr,
    shape: &[usize],
    flat: &[Expr],
    spec: &ArrayFilterSpec,
    padding: &ArrayFilterPadding,
) -> Result<Expr, EvalError> {
    let list_head = system_expr("List");
    let window_shape = array_filter_window_shape(spec);
    let window_center = array_filter_window_center(spec);
    let output_shape = match padding {
        ArrayFilterPadding::None => shape
            .iter()
            .zip(window_shape.iter().zip(window_center.iter()))
            .map(|(len, (window_len, center))| {
                let left = *center;
                let right = window_len.saturating_sub(left + 1);
                let required = left + right + 1;
                if *len >= required {
                    len - left - right
                } else {
                    0
                }
            })
            .collect::<Vec<_>>(),
        _ => shape.to_vec(),
    };

    let mut outputs = Vec::new();
    for output_index in enumerate_indices(&output_shape) {
        let center = output_index
            .iter()
            .zip(window_center.iter())
            .map(|(index, center)| match padding {
                ArrayFilterPadding::None => index + center,
                _ => *index,
            })
            .collect::<Vec<_>>();
        let mut window_index = Vec::with_capacity(shape.len());
        let block = build_array_filter_block(
            &list_head,
            flat,
            shape,
            &center,
            spec,
            &window_shape,
            &window_center,
            padding,
            0,
            &mut window_index,
        )
        .expect("validated array neighborhoods should rebuild");
        outputs.push(session.evaluate(Expr::call(operator.clone(), vec![block]))?);
    }

    Ok(rebuild_tensor_expr(&list_head, &output_shape, &outputs))
}

fn build_array_filter_block(
    head: &Expr,
    flat: &[Expr],
    shape: &[usize],
    center: &[usize],
    spec: &ArrayFilterSpec,
    window_shape: &[usize],
    window_center: &[usize],
    padding: &ArrayFilterPadding,
    axis: usize,
    window_index: &mut Vec<usize>,
) -> Option<Expr> {
    if axis == shape.len() {
        if !array_filter_template_includes(spec, window_index) {
            return Some(Expr::call(system_expr("Missing"), Vec::new()));
        }

        let source_index = window_index
            .iter()
            .enumerate()
            .map(|(window_axis, position)| {
                let center_position = isize::try_from(center[window_axis]).ok()?;
                let window_position = isize::try_from(*position).ok()?;
                let anchor = isize::try_from(window_center[window_axis]).ok()?;
                Some(center_position + window_position - anchor)
            })
            .collect::<Option<Vec<_>>>()?;
        return array_filter_scalar_at(flat, shape, &source_index, padding);
    }

    let mut items = Vec::with_capacity(window_shape[axis]);
    for position in 0..window_shape[axis] {
        window_index.push(position);
        items.push(build_array_filter_block(
            head,
            flat,
            shape,
            center,
            spec,
            window_shape,
            window_center,
            padding,
            axis + 1,
            window_index,
        )?);
        window_index.pop();
    }

    Some(Expr::call(head.clone(), items))
}

fn array_filter_scalar_at(
    flat: &[Expr],
    shape: &[usize],
    index: &[isize],
    padding: &ArrayFilterPadding,
) -> Option<Expr> {
    let mut normalized = Vec::with_capacity(index.len());
    for (axis, raw_index) in index.iter().enumerate() {
        let len = *shape.get(axis)?;
        let len_isize = isize::try_from(len).ok()?;
        let in_bounds = (0..len_isize).contains(raw_index);
        match padding {
            ArrayFilterPadding::None => {
                if !in_bounds {
                    return None;
                }
                normalized.push(usize::try_from(*raw_index).ok()?);
            }
            ArrayFilterPadding::Fixed => {
                if len == 0 {
                    return None;
                }
                let clamped = (*raw_index).clamp(0, len_isize - 1);
                normalized.push(usize::try_from(clamped).ok()?);
            }
            ArrayFilterPadding::Periodic => {
                if len == 0 {
                    return None;
                }
                let periodic = raw_index.rem_euclid(len_isize);
                normalized.push(usize::try_from(periodic).ok()?);
            }
            ArrayFilterPadding::Constant(expr) if !in_bounds => return Some(expr.clone()),
            ArrayFilterPadding::Constant(_) => {
                normalized.push(usize::try_from(*raw_index).ok()?);
            }
        }
    }

    tensor_scalar_at(flat, shape, &normalized).cloned()
}

fn apply_array_reduce(
    session: &mut Session,
    operator: &Expr,
    shape: &[usize],
    flat: &[Expr],
    spec: &ArrayReduceSpec,
) -> Result<Expr, EvalError> {
    let list_head = system_expr("List");

    let ArrayReduceSpec::Grouped(groups) = spec else {
        let mapped = flat
            .iter()
            .map(|item| session.evaluate(Expr::call(operator.clone(), vec![item.clone()])))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(rebuild_tensor_expr(&list_head, shape, &mapped));
    };

    let rank = shape.len();
    let reduced_dims = groups.iter().flatten().copied().collect::<Vec<_>>();
    let kept_dims = (0..rank)
        .filter(|axis| !reduced_dims.contains(axis))
        .collect::<Vec<_>>();
    let output_shape = kept_dims
        .iter()
        .map(|axis| shape[*axis])
        .collect::<Vec<_>>();
    let kept_indices = enumerate_indices(&output_shape);
    let grouped_shape = groups
        .iter()
        .map(|group| group.iter().map(|axis| shape[*axis]).product())
        .collect::<Vec<_>>();
    let grouped_indices = enumerate_indices(&grouped_shape);

    let mut outputs = Vec::with_capacity(kept_indices.len());
    for kept_index in kept_indices {
        let mut values = Vec::with_capacity(grouped_indices.len());
        for grouped_index in &grouped_indices {
            let full_index = array_reduce_full_index(
                rank,
                &kept_dims,
                &kept_index,
                groups,
                grouped_index,
                shape,
            );
            let value = tensor_scalar_at(flat, shape, &full_index)
                .expect("validated array reductions should resolve a scalar")
                .clone();
            values.push(value);
        }

        let operator_arg = rebuild_tensor_expr(&list_head, &grouped_shape, &values);
        outputs.push(session.evaluate(Expr::call(operator.clone(), vec![operator_arg]))?);
    }

    Ok(rebuild_tensor_expr(&list_head, &output_shape, &outputs))
}

fn array_reduce_full_index(
    rank: usize,
    kept_dims: &[usize],
    kept_index: &[usize],
    groups: &[Vec<usize>],
    grouped_index: &[usize],
    shape: &[usize],
) -> Vec<usize> {
    let mut full_index = vec![0usize; rank];
    for (position, axis) in kept_dims.iter().copied().enumerate() {
        full_index[axis] = kept_index[position];
    }

    for (group_position, group) in groups.iter().enumerate() {
        let group_shape = group.iter().map(|axis| shape[*axis]).collect::<Vec<_>>();
        let decoded = decode_grouped_axis_index(grouped_index[group_position], &group_shape);
        for (axis_position, axis) in group.iter().copied().enumerate() {
            full_index[axis] = decoded[axis_position];
        }
    }

    full_index
}

fn iterable_items(expr: &Expr) -> Option<Vec<Expr>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => Some(
            association_entries_from_expr(expr)?
                .into_iter()
                .map(|(_, value)| value)
                .collect(),
        ),
        Expr::Call { args, .. } => Some(args.clone()),
        Expr::Atom(_) => None,
    }
}

fn apply_block_map(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    expr: &Expr,
    size: usize,
    offset: usize,
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: block_head,
        args: items,
    } = expr
    else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let mut results = Vec::new();
    let mut index = 0usize;
    while index + size <= items.len() {
        let block = Expr::call(
            block_head.as_ref().clone(),
            items[index..index + size].to_vec(),
        );
        results.push(session.evaluate(Expr::call(operator.clone(), vec![block]))?);
        index += offset;
    }

    Ok(Expr::call(system_expr("List"), results))
}

fn subset_map_paths_are_compatible(paths: &[Vec<isize>]) -> bool {
    for (index, path) in paths.iter().enumerate() {
        for other in paths.iter().skip(index + 1) {
            if path == other
                || path.starts_with(other.as_slice())
                || other.starts_with(path.as_slice())
            {
                return false;
            }
        }
    }

    true
}

fn subset_map_paths_from_expr(expr: &Expr, spec_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    let Some(items) = call_items_with_head(spec_expr, &system_expr("List")) else {
        return exact_position_paths_from_expr(expr, spec_expr);
    };

    if items
        .iter()
        .all(|item| !matches!(item, Expr::Call { head, .. } if is_system_symbol_name(head, "List")))
    {
        let mut paths = Vec::new();
        for item in items {
            paths.extend(exact_position_paths_from_expr(expr, item)?);
        }
        Some(paths)
    } else {
        exact_position_paths_from_expr(expr, spec_expr)
    }
}

fn apply_subset_map(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    expr: &Expr,
    paths: &[Vec<isize>],
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some(parts) = paths
        .iter()
        .map(|path| exact_part_at_path(expr, path))
        .collect::<Option<Vec<_>>>()
    else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let mapped = session.evaluate(Expr::call(
        operator.clone(),
        vec![Expr::call(system_expr("List"), parts)],
    ))?;
    let Expr::Call {
        args: replacements, ..
    } = mapped
    else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    if replacements.len() != paths.len() {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    }

    let mut current = expr.clone();
    for (path, replacement) in paths.iter().zip(replacements.into_iter()) {
        current = replace_exact_path(current, path, replacement);
    }

    session.evaluate(current)
}

fn apply_fold(
    session: &mut Session,
    operator: &Expr,
    initial: Option<Expr>,
    list_expr: &Expr,
    collect_all: bool,
) -> Result<Expr, EvalError> {
    let Some(items) = iterable_items(list_expr) else {
        return Ok(Expr::call(
            system_expr(if collect_all { "FoldList" } else { "Fold" }),
            match initial {
                Some(initial) => vec![operator.clone(), initial, list_expr.clone()],
                None => vec![operator.clone(), list_expr.clone()],
            },
        ));
    };

    let (mut current, start_index, mut history) = if let Some(initial) = initial {
        let history = if collect_all {
            vec![initial.clone()]
        } else {
            Vec::new()
        };
        (initial, 0, history)
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call(
                system_expr(if collect_all { "FoldList" } else { "Fold" }),
                vec![operator.clone(), list_expr.clone()],
            ));
        };
        let history = if collect_all {
            vec![first.clone()]
        } else {
            Vec::new()
        };
        let start_index = items.len() - rest.len();
        (first.clone(), start_index, history)
    };

    for item in items.into_iter().skip(start_index) {
        current = session.evaluate(Expr::call(operator.clone(), vec![current, item]))?;
        if collect_all {
            history.push(current.clone());
        }
    }

    if collect_all {
        Ok(Expr::call(system_expr("List"), history))
    } else {
        Ok(current)
    }
}

fn apply_fold_pair(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    initial: Option<Expr>,
    list_expr: &Expr,
    projector: Option<Expr>,
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some(items) = iterable_items(list_expr) else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let (mut state, start_index) = if let Some(initial) = initial {
        (initial, 0usize)
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call(head.clone(), raw_args.to_vec()));
        };
        (first.clone(), items.len() - rest.len())
    };

    let mut last_output = None;
    for item in items.into_iter().skip(start_index) {
        let pair = session.evaluate(Expr::call(operator.clone(), vec![state, item]))?;
        let Some((x_value, y_value)) = parse_pair_result(&pair) else {
            return Ok(Expr::call(head.clone(), raw_args.to_vec()));
        };

        let output = if let Some(projector) = &projector {
            session.evaluate(Expr::call(
                projector.clone(),
                vec![Expr::call(
                    system_expr("List"),
                    vec![x_value.clone(), y_value.clone()],
                )],
            ))?
        } else {
            x_value.clone()
        };

        last_output = Some(output);
        state = y_value;
    }

    Ok(last_output.unwrap_or_else(|| Expr::call(head.clone(), raw_args.to_vec())))
}

fn parse_pair_result(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { args, .. } = expr else {
        return None;
    };

    match args.as_slice() {
        [first, second] => Some((first.clone(), second.clone())),
        _ => None,
    }
}

fn apply_fold_while(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    initial: Option<Expr>,
    list_expr: &Expr,
    test: &Expr,
    m_expr: Option<&Expr>,
    n_expr: Option<&Expr>,
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some(config) = parse_fold_while_config(m_expr, n_expr) else {
        return Ok(session.invalid_call_with_arg_form(
            "FoldWhile",
            raw_args,
            "expected a positive integer or All test arity, with an optional signed integer tail adjustment",
        ));
    };

    let Some(items) = iterable_items(list_expr) else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let (mut current, start_index, mut history) = if let Some(initial) = initial {
        (initial.clone(), 0usize, vec![initial])
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call(head.clone(), raw_args.to_vec()));
        };
        (first.clone(), items.len() - rest.len(), vec![first.clone()])
    };

    let mut index = start_index;
    let mut failed = false;
    while index < items.len() {
        current = session.evaluate(Expr::call(
            operator.clone(),
            vec![current, items[index].clone()],
        ))?;
        history.push(current.clone());
        index += 1;

        let test_value = session.evaluate(Expr::call(
            test.clone(),
            nest_while_test_args(&history, config.test_arity),
        ))?;
        if truthiness_symbol(&test_value) != Some(true) {
            failed = true;
            break;
        }
    }

    if failed {
        if config.tail_adjustment > 0 {
            for _ in 0..usize::try_from(config.tail_adjustment).unwrap_or(0) {
                if index >= items.len() {
                    break;
                }
                let previous = history.last().cloned().unwrap_or_else(|| current.clone());
                current = session.evaluate(Expr::call(
                    operator.clone(),
                    vec![previous, items[index].clone()],
                ))?;
                history.push(current.clone());
                index += 1;
            }
        } else if config.tail_adjustment < 0 {
            let drop_count = usize::try_from(-config.tail_adjustment).unwrap_or(usize::MAX);
            let keep_len = history.len().saturating_sub(drop_count).max(1);
            history.truncate(keep_len);
        }
    }

    Ok(history
        .last()
        .cloned()
        .unwrap_or_else(|| Expr::call(head.clone(), raw_args.to_vec())))
}

fn apply_sequence_fold(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    seeds_expr: &Expr,
    items_expr: &Expr,
    arity_expr: Option<&Expr>,
    collect_all: bool,
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some(seeds) = iterable_items(seeds_expr) else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };
    let Some(items) = iterable_items(items_expr) else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let seed_len = seeds.len();
    let arity = match arity_expr {
        Some(expr) => {
            let Some(arity) = expr_as_non_negative_usize(expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    if collect_all {
                        "SequenceFoldList"
                    } else {
                        "SequenceFold"
                    },
                    raw_args,
                    "expected a non-negative integer arity as the fourth argument",
                ));
            };
            arity
        }
        None => seed_len.saturating_add(1),
    };

    if arity <= seed_len {
        return Ok(session.invalid_call_with_arg_form(
            if collect_all {
                "SequenceFoldList"
            } else {
                "SequenceFold"
            },
            raw_args,
            "expected an arity larger than the seed length in the current first-pass subset",
        ));
    }

    let consumed_per_step = arity - seed_len;
    let mut history = seeds.clone();
    let mut offset = 0usize;

    while offset + consumed_per_step <= items.len() {
        let mut call_args = if seed_len == 0 {
            Vec::new()
        } else {
            history
                .iter()
                .rev()
                .take(seed_len)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
        };
        call_args.extend(items[offset..offset + consumed_per_step].iter().cloned());

        history.push(session.evaluate(Expr::call(operator.clone(), call_args))?);
        offset += consumed_per_step;
    }

    if collect_all {
        Ok(Expr::call(system_expr("List"), history))
    } else {
        history
            .last()
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| Ok(Expr::call(head.clone(), raw_args.to_vec())))
    }
}

fn compute_inner(
    session: &mut Session,
    f: &Expr,
    left: &Expr,
    right: &Expr,
    g: &Expr,
) -> Result<Option<Expr>, EvalError> {
    let (
        Expr::Call {
            head: left_head, ..
        },
        Expr::Call {
            head: right_head, ..
        },
    ) = (left, right)
    else {
        return Ok(None);
    };

    if left_head.as_ref() != right_head.as_ref() {
        return Ok(None);
    }

    let head = left_head.as_ref().clone();
    let left_tensor = parse_tensor_node(left, &head);
    let right_tensor = parse_tensor_node(right, &head);
    let Some(left_shape) = tensor_shape(&left_tensor) else {
        return Ok(None);
    };
    let Some(right_shape) = tensor_shape(&right_tensor) else {
        return Ok(None);
    };
    let Some((&contract_dim, right_suffix)) = right_shape.split_first() else {
        return Ok(None);
    };
    let Some((&left_contract_dim, left_prefix)) = left_shape.split_last() else {
        return Ok(None);
    };

    if left_contract_dim != contract_dim {
        return Ok(None);
    }

    let left_flat = flatten_tensor_scalars(&left_tensor);
    let right_flat = flatten_tensor_scalars(&right_tensor);
    let output_shape = left_prefix
        .iter()
        .chain(right_suffix.iter())
        .copied()
        .collect::<Vec<_>>();
    let output_indices = enumerate_indices(&output_shape);
    let left_prefix_rank = left_prefix.len();
    let mut output_scalars = Vec::with_capacity(output_indices.len());

    for output_index in output_indices {
        let (left_index, right_index) = output_index.split_at(left_prefix_rank);
        let mut terms = Vec::with_capacity(contract_dim);

        for contract_index in 0..contract_dim {
            let mut lhs = left_index.to_vec();
            lhs.push(contract_index);
            let mut rhs = vec![contract_index];
            rhs.extend_from_slice(right_index);

            let Some(lhs_value) = tensor_scalar_at(&left_flat, &left_shape, &lhs) else {
                return Ok(None);
            };
            let Some(rhs_value) = tensor_scalar_at(&right_flat, &right_shape, &rhs) else {
                return Ok(None);
            };
            let term = session.evaluate(Expr::call(
                f.clone(),
                vec![lhs_value.clone(), rhs_value.clone()],
            ))?;
            terms.push(term);
        }

        let reduced = session.evaluate(Expr::call(g.clone(), terms))?;
        output_scalars.push(reduced);
    }

    Ok(Some(rebuild_tensor_expr(
        &head,
        &output_shape,
        &output_scalars,
    )))
}

fn apply_nest(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    count: usize,
) -> Result<Expr, EvalError> {
    let limit = session.resource_limits.evaluation_pass_limit;
    let mut current = expr.clone();

    for iteration in 0..count {
        if iteration >= limit {
            return Err(EvalError::EvaluationPassLimitExceeded { limit });
        }

        let next = session.evaluate(Expr::call(operator.clone(), vec![current]))?;
        current = next;
    }

    Ok(current)
}

fn apply_nest_list(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    count: usize,
) -> Result<Expr, EvalError> {
    let limit = session.resource_limits.evaluation_pass_limit;
    let mut current = expr.clone();
    let mut items = vec![current.clone()];

    for iteration in 0..count {
        if iteration >= limit {
            return Err(EvalError::EvaluationPassLimitExceeded { limit });
        }

        current = session.evaluate(Expr::call(operator.clone(), vec![current]))?;
        items.push(current.clone());
    }

    Ok(Expr::call(system_expr("List"), items))
}

fn apply_fixed_point(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    max_iterations: Option<usize>,
) -> Result<Expr, EvalError> {
    let limit = max_iterations.unwrap_or(session.resource_limits.evaluation_pass_limit);
    let mut current = expr.clone();
    let mut seen = vec![current.clone()];

    for iteration in 0..limit {
        let next = session.evaluate(Expr::call(operator.clone(), vec![current.clone()]))?;
        if next == current {
            return Ok(next);
        }

        if max_iterations.is_none() && seen.iter().any(|previous| previous == &next) {
            return Err(EvalError::EvaluationCycleDetected {
                expr: next.to_string(),
            });
        }

        seen.push(next.clone());
        current = next;

        if iteration + 1 >= limit && max_iterations.is_some() {
            return Ok(current);
        }
    }

    if max_iterations.is_some() {
        Ok(current)
    } else {
        Err(EvalError::EvaluationPassLimitExceeded { limit })
    }
}

fn parse_tensor_node(expr: &Expr, expected_head: &Expr) -> TensorNode {
    match expr {
        Expr::Call { head, args } if head.as_ref() == expected_head => TensorNode::Array(
            args.iter()
                .map(|arg| parse_tensor_node(arg, expected_head))
                .collect(),
        ),
        _ => TensorNode::Scalar(expr.clone()),
    }
}

fn tensor_shape(node: &TensorNode) -> Option<Vec<usize>> {
    match node {
        TensorNode::Scalar(_) => Some(Vec::new()),
        TensorNode::Array(items) => {
            let Some(first) = items.first() else {
                return Some(vec![0]);
            };
            let child_shape = tensor_shape(first)?;
            for item in items.iter().skip(1) {
                if tensor_shape(item)? != child_shape {
                    return None;
                }
            }

            let mut shape = vec![items.len()];
            shape.extend(child_shape);
            Some(shape)
        }
    }
}

fn flatten_tensor_scalars(node: &TensorNode) -> Vec<Expr> {
    let mut flat = Vec::new();
    flatten_tensor_scalars_into(node, &mut flat);
    flat
}

fn flatten_tensor_scalars_into(node: &TensorNode, flat: &mut Vec<Expr>) {
    match node {
        TensorNode::Scalar(expr) => flat.push(expr.clone()),
        TensorNode::Array(items) => {
            for item in items {
                flatten_tensor_scalars_into(item, flat);
            }
        }
    }
}

fn tensor_scalar_at<'a>(flat: &'a [Expr], shape: &[usize], index: &[usize]) -> Option<&'a Expr> {
    let offset = tensor_flat_offset(shape, index)?;
    flat.get(offset)
}

fn tensor_flat_offset(shape: &[usize], index: &[usize]) -> Option<usize> {
    if shape.len() != index.len() {
        return None;
    }

    if shape.is_empty() {
        return Some(0);
    }

    let mut offset = 0usize;
    let mut stride = shape.iter().product::<usize>();
    for (&axis_len, &axis_index) in shape.iter().zip(index.iter()) {
        if axis_index >= axis_len || axis_len == 0 {
            return None;
        }
        stride /= axis_len;
        offset += axis_index * stride;
    }

    Some(offset)
}

fn enumerate_indices(shape: &[usize]) -> Vec<Vec<usize>> {
    if shape.is_empty() {
        return vec![Vec::new()];
    }

    let suffixes = enumerate_indices(&shape[1..]);
    let mut indices = Vec::new();
    for first in 0..shape[0] {
        for suffix in &suffixes {
            let mut index = Vec::with_capacity(shape.len());
            index.push(first);
            index.extend_from_slice(suffix);
            indices.push(index);
        }
    }
    indices
}

fn decode_grouped_axis_index(mut index: usize, shape: &[usize]) -> Vec<usize> {
    if shape.is_empty() {
        return Vec::new();
    }

    let mut decoded = vec![0usize; shape.len()];
    for axis in (0..shape.len()).rev() {
        let len = shape[axis];
        if len == 0 {
            decoded[axis] = 0;
        } else {
            decoded[axis] = index % len;
            index /= len;
        }
    }
    decoded
}

fn rebuild_tensor_expr(head: &Expr, shape: &[usize], flat: &[Expr]) -> Expr {
    let mut offset = 0usize;
    rebuild_tensor_expr_from_offset(head, shape, flat, &mut offset)
}

fn rebuild_tensor_expr_from_offset(
    head: &Expr,
    shape: &[usize],
    flat: &[Expr],
    offset: &mut usize,
) -> Expr {
    if shape.is_empty() {
        let expr = flat
            .get(*offset)
            .cloned()
            .unwrap_or_else(|| Expr::call(head.clone(), Vec::new()));
        *offset += 1;
        return expr;
    }

    let items = (0..shape[0])
        .map(|_| rebuild_tensor_expr_from_offset(head, &shape[1..], flat, offset))
        .collect();
    Expr::call(head.clone(), items)
}

fn apply_fixed_point_list(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    max_iterations: Option<usize>,
) -> Result<Expr, EvalError> {
    let limit = max_iterations.unwrap_or(session.resource_limits.evaluation_pass_limit);
    let mut current = expr.clone();
    let mut seen = vec![current.clone()];
    let mut items = vec![current.clone()];

    for _ in 0..limit {
        let next = session.evaluate(Expr::call(operator.clone(), vec![current.clone()]))?;
        items.push(next.clone());

        if next == current {
            return Ok(Expr::call(system_expr("List"), items));
        }

        if max_iterations.is_none() && seen.iter().any(|previous| previous == &next) {
            return Err(EvalError::EvaluationCycleDetected {
                expr: next.to_string(),
            });
        }

        seen.push(next.clone());
        current = next;
    }

    Ok(Expr::call(system_expr("List"), items))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NestWhileConfig {
    test_arity: NestWhileTestArity,
    max_iterations: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FoldWhileConfig {
    test_arity: NestWhileTestArity,
    tail_adjustment: isize,
}

impl Default for NestWhileConfig {
    fn default() -> Self {
        Self {
            test_arity: NestWhileTestArity::Count(1),
            max_iterations: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NestWhileTestArity {
    Count(usize),
    All,
}

fn parse_fold_while_config(
    m_expr: Option<&Expr>,
    n_expr: Option<&Expr>,
) -> Option<FoldWhileConfig> {
    let test_arity = match m_expr {
        Some(expr) if is_system_symbol_name(expr, "All") => NestWhileTestArity::All,
        Some(expr) => {
            let count = expr_as_non_negative_usize(expr)?;
            if count == 0 {
                return None;
            }
            NestWhileTestArity::Count(count)
        }
        None => NestWhileTestArity::Count(1),
    };

    let tail_adjustment = match n_expr {
        Some(expr) => expr_as_signed_isize(expr)?,
        None => 0,
    };

    Some(FoldWhileConfig {
        test_arity,
        tail_adjustment,
    })
}

fn parse_nest_while_config(m_expr: &Expr, max_expr: Option<&Expr>) -> Option<NestWhileConfig> {
    let test_arity = if is_system_symbol_name(m_expr, "All") {
        NestWhileTestArity::All
    } else {
        let count = expr_as_non_negative_usize(m_expr)?;
        if count == 0 {
            return None;
        }
        NestWhileTestArity::Count(count)
    };

    let max_iterations = match max_expr {
        Some(expr) => Some(expr_as_non_negative_usize(expr)?),
        None => None,
    };

    Some(NestWhileConfig {
        test_arity,
        max_iterations,
    })
}

fn apply_nest_while(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    test: &Expr,
    config: NestWhileConfig,
    collect_all: bool,
) -> Result<Expr, EvalError> {
    let limit = config
        .max_iterations
        .unwrap_or(session.resource_limits.evaluation_pass_limit);
    let mut history = vec![expr.clone()];

    for iteration in 0..=limit {
        let test_args = nest_while_test_args(&history, config.test_arity);
        let test_value = session.evaluate(Expr::call(test.clone(), test_args))?;
        if truthiness_symbol(&test_value) != Some(true) {
            return Ok(if collect_all {
                Expr::call(system_expr("List"), history)
            } else {
                history.last().cloned().unwrap_or_else(|| expr.clone())
            });
        }

        if iteration >= limit {
            return if config.max_iterations.is_some() {
                Ok(if collect_all {
                    Expr::call(system_expr("List"), history)
                } else {
                    history.last().cloned().unwrap_or_else(|| expr.clone())
                })
            } else {
                Err(EvalError::EvaluationPassLimitExceeded { limit })
            };
        }

        let current = history.last().cloned().unwrap_or_else(|| expr.clone());
        let next = session.evaluate(Expr::call(operator.clone(), vec![current]))?;
        history.push(next);
    }

    Ok(if collect_all {
        Expr::call(system_expr("List"), history)
    } else {
        history.last().cloned().unwrap_or_else(|| expr.clone())
    })
}

fn nest_while_test_args(history: &[Expr], arity: NestWhileTestArity) -> Vec<Expr> {
    match arity {
        NestWhileTestArity::Count(count) => history
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        NestWhileTestArity::All => history.to_vec(),
    }
}
