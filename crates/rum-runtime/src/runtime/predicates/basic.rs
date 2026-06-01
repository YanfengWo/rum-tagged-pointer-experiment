use super::*;

impl Session {
    pub(in crate::runtime) fn eval_atom_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AtomQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(expr, Expr::Atom(_))))
    }

    pub(in crate::runtime) fn eval_list_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("ListQ", args, 1));
        };

        Ok(predicate_bool_expr(
            matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST)),
        ))
    }

    pub(in crate::runtime) fn eval_string_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("StringQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(
            expr,
            Expr::Atom(Atom::String(_))
        )))
    }

    pub(in crate::runtime) fn eval_missing_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("MissingQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(
            expr,
            Expr::Call { head, .. } if is_system_symbol_name(head, "Missing")
        )))
    }

    pub(in crate::runtime) fn eval_date_object_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("DateObjectQ", args, 1));
        };

        Ok(predicate_bool_expr(expr_has_system_head(
            expr,
            "DateObject",
        )))
    }

    pub(in crate::runtime) fn eval_time_object_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("TimeObjectQ", args, 1));
        };

        Ok(predicate_bool_expr(expr_has_system_head(
            expr,
            "TimeObject",
        )))
    }
}
