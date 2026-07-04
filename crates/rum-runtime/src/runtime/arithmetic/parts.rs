use super::super::*;

impl Session {
    pub(super) fn eval_total(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let Expr::Call {
            head: list_head,
            args: items,
        } = &args[0]
        else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if !is_system_symbol_name(list_head, "List") {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        self.evaluate(Expr::call_with_shared_args(
            system_expr("Plus"),
            items.clone(),
        ))
    }

    pub(super) fn eval_denominator(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Denominator", args, 1));
        }

        match &args[0] {
            Expr::Atom(Atom::Integer(_)) => Ok(Expr::integer(1_i32.into())),
            Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.denominator().clone())),
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(super) fn eval_numerator(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Numerator", args, 1));
        }

        match &args[0] {
            Expr::Atom(Atom::Integer(value)) => Ok(Expr::integer(value.to_bigint())),
            Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.numerator().clone())),
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }
}
