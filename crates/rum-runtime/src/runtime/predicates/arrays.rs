use super::*;

impl Session {
    pub(in crate::runtime) fn eval_vector_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("VectorQ", args, 1));
        };

        Ok(predicate_bool_expr(
            array_shape(expr).is_some_and(|shape| shape.len() == 1),
        ))
    }

    pub(in crate::runtime) fn eval_matrix_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("MatrixQ", args, 1));
        };

        Ok(predicate_bool_expr(
            array_shape(expr).is_some_and(|shape| shape.len() == 2),
        ))
    }

    pub(in crate::runtime) fn eval_array_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ArrayQ", args, 1, 2));
        }

        let Some(shape) = array_shape(&args[0]) else {
            return Ok(predicate_bool_expr(false));
        };

        if args.len() == 1 {
            return Ok(predicate_bool_expr(true));
        }

        let Some(rank) = expr_as_non_negative_usize(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(predicate_bool_expr(shape.len() == rank))
    }
}
