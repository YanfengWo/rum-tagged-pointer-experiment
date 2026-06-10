use super::shared::{negated_relation_head_name, predicate_bool_expr};
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_and(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.resume_and_from_index(args.to_vec(), 0, Vec::new())
    }

    pub(in crate::runtime) fn eval_or(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.resume_or_from_index(args.to_vec(), 0, Vec::new())
    }

    pub(in crate::runtime) fn eval_nand(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.resume_nand_from_index(args.to_vec(), 0, Vec::new())
    }

    pub(in crate::runtime) fn eval_nor(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.resume_nor_from_index(args.to_vec(), 0, Vec::new())
    }

    pub(in crate::runtime) fn eval_xor(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_equivalent(
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

    pub(in crate::runtime) fn eval_implies(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [lhs, rhs] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Implies", args, 2));
        };

        if lhs == rhs {
            return Ok(system_expr(heads::structure::TRUE));
        }

        Ok(match (truthiness_symbol(lhs), truthiness_symbol(rhs)) {
            (Some(false), _) | (_, Some(true)) => system_expr(heads::structure::TRUE),
            (Some(true), _) => rhs.clone(),
            (_, Some(false)) => negate_logical_expr(lhs.clone()),
            _ => Expr::call(head.clone(), args.to_vec()),
        })
    }

    pub(in crate::runtime) fn eval_not(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Not", args, 1));
        };

        Ok(negate_logical_expr(arg.clone()))
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

    fn resume_nand_from_index(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        residual: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if index >= args.len() {
            return Ok(finalize_negated_logical_sequence("Nand", residual, false));
        }

        let arg = args[index].clone();
        if let Some(truth) = truthiness_symbol(&arg) {
            return self.resume_nand_with_value(args, index, residual, predicate_bool_expr(truth));
        }

        let suspended_args = args.clone();
        let suspended_residual = residual.clone();
        let result = self.evaluate(arg);
        let value = self.suspend_with_frame(result, move |session, value| {
            session.resume_nand_with_value(suspended_args, index, suspended_residual, value)
        })?;
        self.resume_nand_with_value(args, index, residual, value)
    }

    fn resume_nand_with_value(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        mut residual: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        match truthiness_symbol(&value) {
            Some(false) => Ok(system_expr(heads::structure::TRUE)),
            Some(true) => self.resume_nand_from_index(args, index + 1, residual),
            None => {
                residual.push(value);
                self.resume_nand_from_index(args, index + 1, residual)
            }
        }
    }

    fn resume_nor_from_index(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        residual: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if index >= args.len() {
            return Ok(finalize_negated_logical_sequence("Nor", residual, true));
        }

        let arg = args[index].clone();
        if let Some(truth) = truthiness_symbol(&arg) {
            return self.resume_nor_with_value(args, index, residual, predicate_bool_expr(truth));
        }

        let suspended_args = args.clone();
        let suspended_residual = residual.clone();
        let result = self.evaluate(arg);
        let value = self.suspend_with_frame(result, move |session, value| {
            session.resume_nor_with_value(suspended_args, index, suspended_residual, value)
        })?;
        self.resume_nor_with_value(args, index, residual, value)
    }

    fn resume_nor_with_value(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        mut residual: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        match truthiness_symbol(&value) {
            Some(true) => Ok(system_expr(heads::structure::FALSE)),
            Some(false) => self.resume_nor_from_index(args, index + 1, residual),
            None => {
                residual.push(value);
                self.resume_nor_from_index(args, index + 1, residual)
            }
        }
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
        negate_logical_expr(expr)
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
                negate_logical_expr(expr)
            } else {
                expr
            }
        }
        _ => {
            let args = if negate_residuals {
                residual.into_iter().map(negate_logical_expr).collect()
            } else {
                residual
            };
            Expr::call(system_expr("And"), args)
        }
    }
}

fn negate_logical_expr(expr: Expr) -> Expr {
    match truthiness_symbol(&expr) {
        Some(truth) => return predicate_bool_expr(!truth),
        None => {}
    }

    if let Expr::Call { head, args } = expr {
        if is_system_symbol_name(&head, "Not") && args.len() == 1 {
            return args.into_iter().next().expect("single Not argument");
        }

        if args.len() == 2 {
            if let Some(opposite) = negated_relation_head_name(&head) {
                return Expr::call_with_shared_args(system_expr(opposite), args);
            }
        }

        return Expr::call(system_expr("Not"), vec![Expr::Call { head, args }]);
    }

    Expr::call(system_expr("Not"), vec![expr])
}

fn finalize_logical_sequence(head_name: &str, mut residual: Vec<Expr>, identity: bool) -> Expr {
    match residual.len() {
        0 => predicate_bool_expr(identity),
        1 => residual.pop().expect("single residual value"),
        _ => Expr::call(system_expr(head_name), residual),
    }
}

fn finalize_negated_logical_sequence(
    head_name: &str,
    mut residual: Vec<Expr>,
    empty_result: bool,
) -> Expr {
    match residual.len() {
        0 => predicate_bool_expr(empty_result),
        1 => negate_logical_expr(residual.pop().expect("single residual value")),
        _ => Expr::call(system_expr(head_name), residual),
    }
}
