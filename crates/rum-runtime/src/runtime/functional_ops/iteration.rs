use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NestWhileConfig {
    test_arity: NestWhileTestArity,
    max_iterations: Option<usize>,
}

impl Default for NestWhileConfig {
    fn default() -> Self {
        Self {
            test_arity: NestWhileTestArity::Count(1),
            max_iterations: None,
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_nest(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_nest_while(
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

    pub(in crate::runtime) fn eval_nest_list(
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

    pub(in crate::runtime) fn eval_nest_while_list(
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

    pub(in crate::runtime) fn eval_fixed_point_list(
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

    pub(in crate::runtime) fn eval_fixed_point(
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
