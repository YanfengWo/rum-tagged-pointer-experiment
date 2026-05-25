use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum CompositionDirection {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub(in crate::runtime) struct CompositionSpec<'a> {
    pub(in crate::runtime) direction: CompositionDirection,
    pub(in crate::runtime) functions: Vec<&'a Expr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CurriedOperatorKind {
    OperatorApplied,
    CurryApplied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonOperatorKind {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
    Unequal,
    Same,
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

#[derive(Clone, Debug)]
struct ComparisonOperatorSpec<'a> {
    kind: ComparisonOperatorKind,
    bound: &'a Expr,
}

#[derive(Clone, Debug)]
struct NearestToSpec<'a> {
    target: &'a Expr,
    selection: Option<&'a Expr>,
}

impl CurriedOperatorKind {
    fn symbol(self) -> &'static str {
        match self {
            Self::OperatorApplied => "OperatorApplied",
            Self::CurryApplied => "CurryApplied",
        }
    }
}

pub(in crate::runtime) fn parse_composition(expr: &Expr) -> Option<CompositionSpec<'_>> {
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

pub(in crate::runtime) fn apply_composition_to_args(
    spec: &CompositionSpec<'_>,
    args: &[Expr],
) -> Expr {
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

pub(in crate::runtime) fn is_callable_operator_wrapper(expr: &Expr) -> bool {
    parse_curried_operator(expr).is_some()
        || parse_reverse_applied_wrapper(expr).is_some()
        || parse_comparison_operator_wrapper(expr).is_some()
        || parse_nearest_to_wrapper(expr).is_some()
        || parse_key_wrapper(expr).is_some()
}

pub(in crate::runtime) fn apply_callable_operator_wrapper_to_args(
    head: &Expr,
    args: &[Expr],
) -> Option<Expr> {
    if let Some(spec) = parse_curried_operator(head) {
        return Some(apply_curried_operator_to_args(&spec, args));
    }

    if let Some(spec) = parse_reverse_applied_wrapper(head) {
        return Some(apply_reverse_applied_to_args(&spec, args));
    }

    if let Some(spec) = parse_comparison_operator_wrapper(head) {
        return apply_comparison_operator_wrapper_to_args(&spec, args);
    }

    if let Some(spec) = parse_nearest_to_wrapper(head) {
        return apply_nearest_to_wrapper_to_args(&spec, args);
    }

    let [expr] = args else {
        return None;
    };

    let key = parse_key_wrapper(head)?;
    let entries = association_entries_from_expr(expr)?;
    Some(
        entries
            .iter()
            .find(|(entry_key, _)| entry_key == key)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
    )
}

impl Session {
    pub(in crate::runtime) fn eval_operator_applied(
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

    pub(in crate::runtime) fn eval_curry_applied(
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

    pub(in crate::runtime) fn eval_reverse_applied(
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

    pub(in crate::runtime) fn eval_composition(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(simplify_composition_head(head, args))
    }

    pub(in crate::runtime) fn eval_right_composition(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(simplify_composition_head(head, args))
    }

    pub(in crate::runtime) fn eval_through(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Through", args, 1, 2));
        }

        let target = args.get(1);
        self.evaluate(through_expression(&args[0], target))
    }

    pub(in crate::runtime) fn eval_operate(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

fn parse_comparison_operator_wrapper(expr: &Expr) -> Option<ComparisonOperatorSpec<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    let [bound] = args.as_slice() else {
        return None;
    };

    let kind = if is_system_symbol_name(head, "LessThan") {
        ComparisonOperatorKind::Less
    } else if is_system_symbol_name(head, "GreaterThan") {
        ComparisonOperatorKind::Greater
    } else if is_system_symbol_name(head, "LessEqualThan") {
        ComparisonOperatorKind::LessEqual
    } else if is_system_symbol_name(head, "GreaterEqualThan") {
        ComparisonOperatorKind::GreaterEqual
    } else if is_system_symbol_name(head, "EqualTo") {
        ComparisonOperatorKind::Equal
    } else if is_system_symbol_name(head, "UnequalTo") {
        ComparisonOperatorKind::Unequal
    } else if is_system_symbol_name(head, "SameAs") {
        ComparisonOperatorKind::Same
    } else {
        return None;
    };

    Some(ComparisonOperatorSpec { kind, bound })
}

fn apply_comparison_operator_wrapper_to_args(
    spec: &ComparisonOperatorSpec<'_>,
    args: &[Expr],
) -> Option<Expr> {
    let head = match spec.kind {
        ComparisonOperatorKind::Less => "Less",
        ComparisonOperatorKind::Greater => "Greater",
        ComparisonOperatorKind::LessEqual => "LessEqual",
        ComparisonOperatorKind::GreaterEqual => "GreaterEqual",
        ComparisonOperatorKind::Equal => "Equal",
        ComparisonOperatorKind::Unequal => "Unequal",
        ComparisonOperatorKind::Same => "SameQ",
    };

    match spec.kind {
        ComparisonOperatorKind::Equal
        | ComparisonOperatorKind::Unequal
        | ComparisonOperatorKind::Same => {
            let mut comparison_args = args.to_vec();
            comparison_args.push(spec.bound.clone());
            Some(Expr::call(system_expr(head), comparison_args))
        }
        ComparisonOperatorKind::Less
        | ComparisonOperatorKind::Greater
        | ComparisonOperatorKind::LessEqual
        | ComparisonOperatorKind::GreaterEqual => {
            let [arg] = args else {
                return None;
            };
            Some(Expr::call(
                system_expr(head),
                vec![arg.clone(), spec.bound.clone()],
            ))
        }
    }
}

fn parse_nearest_to_wrapper(expr: &Expr) -> Option<NearestToSpec<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "NearestTo") {
        return None;
    }

    match args.as_slice() {
        [target] => Some(NearestToSpec {
            target,
            selection: None,
        }),
        [target, selection] if is_nearest_to_selection_spec(selection) => Some(NearestToSpec {
            target,
            selection: Some(selection),
        }),
        _ => None,
    }
}

fn apply_nearest_to_wrapper_to_args(spec: &NearestToSpec<'_>, args: &[Expr]) -> Option<Expr> {
    let [data] = args else {
        return None;
    };

    let mut nearest_args = vec![data.clone(), spec.target.clone()];
    if let Some(selection) = spec.selection {
        nearest_args.push(selection.clone());
    }
    Some(Expr::call(system_expr("Nearest"), nearest_args))
}

fn is_nearest_to_selection_spec(expr: &Expr) -> bool {
    if expr_as_positive_usize(expr).is_some() {
        return true;
    }

    let Expr::Call { head, args } = expr else {
        return false;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return false;
    }
    let [count, radius] = args.as_slice() else {
        return false;
    };

    (is_system_symbol_name(count, heads::structure::ALL) || expr_as_positive_usize(count).is_some())
        && is_non_negative_nearest_to_radius(radius)
}

fn is_non_negative_nearest_to_radius(expr: &Expr) -> bool {
    if is_system_symbol_name(expr, heads::structure::INFINITY) {
        return true;
    }
    expr_as_exact_rational(expr).is_some_and(|value| {
        compare_rationals_for_nearest_to(&value, &Rational::zero()) != std::cmp::Ordering::Less
    })
}

fn compare_rationals_for_nearest_to(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
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

fn parse_key_wrapper(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") => match args.as_slice() {
            [key] => Some(key),
            _ => None,
        },
        _ => None,
    }
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

    if matched { transformed } else { expr.clone() }
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
