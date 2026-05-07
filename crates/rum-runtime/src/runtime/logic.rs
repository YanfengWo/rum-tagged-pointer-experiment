use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonRelation {
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ComparableValue {
    Number(Rational),
    String(String),
}

impl Session {
    pub(super) fn eval_and(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.resume_and_from_index(args.to_vec(), 0, Vec::new())
    }

    pub(super) fn eval_or(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.resume_or_from_index(args.to_vec(), 0, Vec::new())
    }

    pub(super) fn eval_nand(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let result = self.resume_and_from_index(args.to_vec(), 0, Vec::new());
        let value =
            self.suspend_with_frame(result, |_, value| Ok(negate_logical_value(value, "Nand")))?;
        Ok(negate_logical_value(value, "Nand"))
    }

    pub(super) fn eval_nor(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let result = self.resume_or_from_index(args.to_vec(), 0, Vec::new());
        let value =
            self.suspend_with_frame(result, |_, value| Ok(negate_logical_value(value, "Nor")))?;
        Ok(negate_logical_value(value, "Nor"))
    }

    pub(super) fn eval_xor(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let mut invert = false;
        let mut residual = Vec::new();

        for arg in args {
            match truthiness_symbol(arg) {
                Some(true) => invert = !invert,
                Some(false) => {}
                None => toggle_residual_expr(&mut residual, arg.clone()),
            }
        }

        Ok(finalize_xor(head, residual, invert))
    }

    pub(super) fn eval_equivalent(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let mut saw_true = false;
        let mut saw_false = false;
        let mut residual = Vec::new();

        for arg in args {
            match truthiness_symbol(arg) {
                Some(true) => saw_true = true,
                Some(false) => saw_false = true,
                None => push_unique_residual_expr(&mut residual, arg.clone()),
            }
        }

        Ok(finalize_equivalent(head, residual, saw_true, saw_false))
    }

    pub(super) fn eval_implies(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [lhs, rhs] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Implies", args, 2));
        };

        if lhs == rhs {
            return Ok(system_expr(heads::structure::TRUE));
        }

        Ok(match (truthiness_symbol(lhs), truthiness_symbol(rhs)) {
            (Some(false), _) | (_, Some(true)) => system_expr(heads::structure::TRUE),
            (Some(true), _) => rhs.clone(),
            (_, Some(false)) => Expr::call(system_expr("Not"), vec![lhs.clone()]),
            _ => Expr::call(head.clone(), args.to_vec()),
        })
    }

    pub(super) fn eval_not(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Not", args, 1));
        };

        match truthiness_symbol(arg) {
            Some(value) => Ok(predicate_bool_expr(!value)),
            None => {
                if let Expr::Call {
                    head: inner_head,
                    args: inner_args,
                } = arg
                {
                    if is_system_symbol_name(inner_head, "Not") && inner_args.len() == 1 {
                        return Ok(inner_args[0].clone());
                    }

                    if let Some(opposite) = negated_relation_head_name(inner_head) {
                        return Ok(Expr::call(system_expr(opposite), inner_args.clone()));
                    }
                }

                Ok(Expr::call(head.clone(), args.to_vec()))
            }
        }
    }

    pub(super) fn eval_equal(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::Equal))
    }

    pub(super) fn eval_unequal(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::Unequal))
    }

    pub(super) fn eval_less(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::Less))
    }

    pub(super) fn eval_greater(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::Greater))
    }

    pub(super) fn eval_less_equal(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::LessEqual))
    }

    pub(super) fn eval_greater_equal(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::GreaterEqual))
    }

    pub(super) fn eval_comparison_operator_wrapper(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    fn resume_and_from_index(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        residual: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if index >= args.len() {
            return Ok(finalize_logical_sequence("And", residual, true));
        }

        let arg = args[index].clone();
        if let Some(truth) = truthiness_symbol(&arg) {
            return self.resume_and_with_value(args, index, residual, predicate_bool_expr(truth));
        }

        let suspended_args = args.clone();
        let suspended_residual = residual.clone();
        let result = self.evaluate(arg);
        let value = self.suspend_with_frame(result, move |session, value| {
            session.resume_and_with_value(suspended_args, index, suspended_residual, value)
        })?;
        self.resume_and_with_value(args, index, residual, value)
    }

    fn resume_and_with_value(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        mut residual: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        match truthiness_symbol(&value) {
            Some(true) => self.resume_and_from_index(args, index + 1, residual),
            Some(false) => Ok(system_expr(heads::structure::FALSE)),
            None => {
                extend_flat_logical_residual(&mut residual, value, "And");
                self.resume_and_from_index(args, index + 1, residual)
            }
        }
    }

    fn resume_or_from_index(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        residual: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if index >= args.len() {
            return Ok(finalize_logical_sequence("Or", residual, false));
        }

        let arg = args[index].clone();
        if let Some(truth) = truthiness_symbol(&arg) {
            return self.resume_or_with_value(args, index, residual, predicate_bool_expr(truth));
        }

        let suspended_args = args.clone();
        let suspended_residual = residual.clone();
        let result = self.evaluate(arg);
        let value = self.suspend_with_frame(result, move |session, value| {
            session.resume_or_with_value(suspended_args, index, suspended_residual, value)
        })?;
        self.resume_or_with_value(args, index, residual, value)
    }

    fn resume_or_with_value(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        mut residual: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        match truthiness_symbol(&value) {
            Some(true) => Ok(system_expr(heads::structure::TRUE)),
            Some(false) => self.resume_or_from_index(args, index + 1, residual),
            None => {
                extend_flat_logical_residual(&mut residual, value, "Or");
                self.resume_or_from_index(args, index + 1, residual)
            }
        }
    }

    fn eval_comparison_subset(
        &mut self,
        head: &Expr,
        args: &[Expr],
        relation: ComparisonRelation,
    ) -> Expr {
        if args.len() <= 1 {
            return predicate_bool_expr(true);
        }

        if relation == ComparisonRelation::Equal {
            if let Some(result) = reduce_equal_prefix_chain(args) {
                return predicate_bool_expr(result);
            }
        } else if relation == ComparisonRelation::Unequal {
            if let Some(result) = reduce_unequal_chain(args) {
                return predicate_bool_expr(result);
            }
        } else if let Some(result) = comparison_result_from_expr_identity(args, relation) {
            return predicate_bool_expr(result);
        }

        if matches!(
            relation,
            ComparisonRelation::Less
                | ComparisonRelation::Greater
                | ComparisonRelation::LessEqual
                | ComparisonRelation::GreaterEqual
        ) {
            if known_order_chain_contradiction(args, relation) {
                return predicate_bool_expr(false);
            }
            if let Some(result) = reduce_order_chain(head, args, relation) {
                return result;
            }
        }

        let result = match relation {
            ComparisonRelation::Equal | ComparisonRelation::Unequal => args
                .iter()
                .map(comparable_value_from_expr)
                .collect::<Option<Vec<_>>>()
                .map(|values| compare_equality_values(&values, relation)),
            ComparisonRelation::Less
            | ComparisonRelation::Greater
            | ComparisonRelation::LessEqual
            | ComparisonRelation::GreaterEqual => args
                .iter()
                .map(numeric_comparable_value_from_expr)
                .collect::<Option<Vec<_>>>()
                .map(|values| compare_numeric_values(&values, relation)),
        };

        match result {
            Some(result) => predicate_bool_expr(result),
            None => Expr::call(head.clone(), args.to_vec()),
        }
    }
}

fn reduce_equal_prefix_chain(args: &[Expr]) -> Option<bool> {
    let mut saw_known_pair = false;

    for pair in args.windows(2) {
        match adjacent_equal_truth(&pair[0], &pair[1]) {
            Some(true) => saw_known_pair = true,
            Some(false) => return Some(false),
            None => return None,
        }
    }

    saw_known_pair.then_some(true)
}

fn adjacent_equal_truth(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    if lhs == rhs {
        return Some(true);
    }

    let lhs = comparable_value_from_expr(lhs)?;
    let rhs = comparable_value_from_expr(rhs)?;
    Some(compare_comparable_values(&lhs, &rhs)? == std::cmp::Ordering::Equal)
}

fn negated_relation_head_name(head: &Expr) -> Option<&'static str> {
    [
        ("Equal", "Unequal"),
        ("Unequal", "Equal"),
        ("Less", "GreaterEqual"),
        ("Greater", "LessEqual"),
        ("LessEqual", "Greater"),
        ("GreaterEqual", "Less"),
    ]
    .into_iter()
    .find_map(|(name, opposite)| is_system_symbol_name(head, name).then_some(opposite))
}

fn reduce_unequal_chain(args: &[Expr]) -> Option<bool> {
    for pair in args.windows(2) {
        match adjacent_unequal_truth(&pair[0], &pair[1]) {
            Some(true) => continue,
            Some(false) => return Some(false),
            None => return None,
        }
    }

    None
}

fn adjacent_unequal_truth(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    if lhs == rhs {
        return Some(false);
    }

    let lhs = comparable_value_from_expr(lhs)?;
    let rhs = comparable_value_from_expr(rhs)?;
    Some(compare_comparable_values(&lhs, &rhs)? != std::cmp::Ordering::Equal)
}

fn reduce_order_chain(head: &Expr, args: &[Expr], relation: ComparisonRelation) -> Option<Expr> {
    if args.len() <= 2 {
        return None;
    }

    let mut kept = Vec::new();
    let mut had_known_true_pair = false;

    for (index, pair) in args.windows(2).enumerate() {
        let Some(ordering) = compare_numeric_expr_pair(&pair[0], &pair[1]) else {
            push_unique_expr(&mut kept, pair[0].clone());
            push_unique_expr(&mut kept, pair[1].clone());
            continue;
        };

        if !ordering_satisfies_relation(ordering, relation) {
            return Some(system_expr(heads::structure::FALSE));
        }

        had_known_true_pair = true;
        if index > 0 && index + 1 < args.len() {
            push_unique_expr(&mut kept, pair[0].clone());
        }
    }

    if !had_known_true_pair || kept.len() == args.len() {
        return None;
    }

    Some(match kept.len() {
        0 | 1 => system_expr(heads::structure::TRUE),
        _ => Expr::call(head.clone(), kept),
    })
}

fn known_order_chain_contradiction(args: &[Expr], relation: ComparisonRelation) -> bool {
    for (left_index, lhs) in args.iter().enumerate() {
        let Some(lhs) = numeric_comparable_value_from_expr(lhs) else {
            continue;
        };

        for rhs in args.iter().skip(left_index + 1) {
            let Some(rhs) = numeric_comparable_value_from_expr(rhs) else {
                continue;
            };
            let ordering = compare_rationals(&lhs, &rhs);
            if !ordering_satisfies_relation(ordering, relation) {
                return true;
            }
        }
    }

    false
}

fn compare_numeric_expr_pair(lhs: &Expr, rhs: &Expr) -> Option<std::cmp::Ordering> {
    let lhs = numeric_comparable_value_from_expr(lhs)?;
    let rhs = numeric_comparable_value_from_expr(rhs)?;
    Some(compare_rationals(&lhs, &rhs))
}

fn numeric_comparable_value_from_expr(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.clone())),
        Expr::Atom(Atom::Rational(value)) => Some(value.clone()),
        _ => None,
    }
}

fn ordering_satisfies_relation(ordering: std::cmp::Ordering, relation: ComparisonRelation) -> bool {
    match relation {
        ComparisonRelation::Less => ordering == std::cmp::Ordering::Less,
        ComparisonRelation::Greater => ordering == std::cmp::Ordering::Greater,
        ComparisonRelation::LessEqual => ordering != std::cmp::Ordering::Greater,
        ComparisonRelation::GreaterEqual => ordering != std::cmp::Ordering::Less,
        ComparisonRelation::Equal | ComparisonRelation::Unequal => false,
    }
}

fn push_unique_expr(values: &mut Vec<Expr>, value: Expr) {
    if values.last() != Some(&value) {
        values.push(value);
    }
}

fn extend_flat_logical_residual(residual: &mut Vec<Expr>, value: Expr, head_name: &str) {
    match value {
        Expr::Call { head, args } if is_system_symbol_name(&head, head_name) => {
            residual.extend(args);
        }
        other => residual.push(other),
    }
}

fn toggle_residual_expr(residual: &mut Vec<Expr>, value: Expr) {
    if let Some(index) = residual.iter().position(|existing| existing == &value) {
        residual.remove(index);
    } else {
        residual.push(value);
    }
}

fn push_unique_residual_expr(residual: &mut Vec<Expr>, value: Expr) {
    if !residual.iter().any(|existing| existing == &value) {
        residual.push(value);
    }
}

fn finalize_xor(head: &Expr, mut residual: Vec<Expr>, invert: bool) -> Expr {
    let expr = match residual.len() {
        0 => predicate_bool_expr(invert),
        1 => residual.pop().expect("single residual value"),
        _ => Expr::call(head.clone(), residual),
    };

    if invert && truthiness_symbol(&expr).is_none() {
        Expr::call(system_expr("Not"), vec![expr])
    } else {
        expr
    }
}

fn finalize_equivalent(head: &Expr, residual: Vec<Expr>, saw_true: bool, saw_false: bool) -> Expr {
    if saw_true && saw_false {
        return predicate_bool_expr(false);
    }

    if saw_true {
        return finalize_equivalent_constant_side(residual, false);
    }

    if saw_false {
        return finalize_equivalent_constant_side(residual, true);
    }

    match residual.len() {
        0 | 1 => predicate_bool_expr(true),
        _ => Expr::call(head.clone(), residual),
    }
}

fn finalize_equivalent_constant_side(mut residual: Vec<Expr>, negate_residuals: bool) -> Expr {
    match residual.len() {
        0 => predicate_bool_expr(true),
        1 => {
            let expr = residual.pop().expect("single residual value");
            if negate_residuals {
                Expr::call(system_expr("Not"), vec![expr])
            } else {
                expr
            }
        }
        _ => {
            let args = if negate_residuals {
                residual
                    .into_iter()
                    .map(|expr| Expr::call(system_expr("Not"), vec![expr]))
                    .collect()
            } else {
                residual
            };
            Expr::call(system_expr("And"), args)
        }
    }
}

fn negate_logical_value(value: Expr, wrapper_head_name: &str) -> Expr {
    match truthiness_symbol(&value) {
        Some(truth) => return predicate_bool_expr(!truth),
        None => {}
    }

    let source_head_name = match wrapper_head_name {
        "Nand" => "And",
        "Nor" => "Or",
        _ => unreachable!("unsupported negated logical wrapper"),
    };

    match value {
        Expr::Call { head, args } if is_system_symbol_name(&head, source_head_name) => {
            Expr::call(system_expr(wrapper_head_name), args)
        }
        other => Expr::call(system_expr("Not"), vec![other]),
    }
}

fn comparison_result_from_expr_identity(
    args: &[Expr],
    relation: ComparisonRelation,
) -> Option<bool> {
    match relation {
        ComparisonRelation::Equal => args
            .windows(2)
            .all(|pair| pair[0] == pair[1])
            .then_some(true),
        ComparisonRelation::Unequal => None,
        ComparisonRelation::Less
        | ComparisonRelation::Greater
        | ComparisonRelation::LessEqual
        | ComparisonRelation::GreaterEqual => None,
    }
}

fn comparable_value_from_expr(expr: &Expr) -> Option<ComparableValue> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(ComparableValue::Number(Rational::from_integer(
            value.clone(),
        ))),
        Expr::Atom(Atom::Rational(value)) => Some(ComparableValue::Number(value.clone())),
        Expr::Atom(Atom::String(value)) => Some(ComparableValue::String(value.clone())),
        _ => None,
    }
}

fn compare_equality_values(values: &[ComparableValue], relation: ComparisonRelation) -> bool {
    match relation {
        ComparisonRelation::Equal => values.windows(2).all(|pair| pair[0] == pair[1]),
        ComparisonRelation::Unequal => !values
            .iter()
            .enumerate()
            .any(|(index, lhs)| values[index + 1..].iter().any(|rhs| lhs == rhs)),
        ComparisonRelation::Less
        | ComparisonRelation::Greater
        | ComparisonRelation::LessEqual
        | ComparisonRelation::GreaterEqual => {
            unreachable!("order relations use numeric comparison")
        }
    }
}

fn compare_numeric_values(values: &[Rational], relation: ComparisonRelation) -> bool {
    values
        .windows(2)
        .all(|pair| ordering_satisfies_relation(compare_rationals(&pair[0], &pair[1]), relation))
}

fn compare_comparable_values(
    lhs: &ComparableValue,
    rhs: &ComparableValue,
) -> Option<std::cmp::Ordering> {
    match (lhs, rhs) {
        (ComparableValue::Number(lhs), ComparableValue::Number(rhs)) => {
            Some(compare_rationals(lhs, rhs))
        }
        (ComparableValue::String(lhs), ComparableValue::String(rhs)) => Some(lhs.cmp(rhs)),
        _ => None,
    }
}

fn compare_rationals(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn finalize_logical_sequence(head_name: &str, mut residual: Vec<Expr>, identity: bool) -> Expr {
    match residual.len() {
        0 => predicate_bool_expr(identity),
        1 => residual.pop().expect("single residual value"),
        _ => Expr::call(system_expr(head_name), residual),
    }
}

fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}
