use super::super::*;
use super::exact::expr_as_exact_integer;

impl Session {
    pub(super) fn eval_factorial(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Factorial", args, 1));
        }

        let Some(value) = expr_as_exact_integer(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if value < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        self.enforce_integer_work_limit_for_factorial(&value)?;

        let one = BigInt::from(1_u8);
        let mut remaining = value;
        let mut result = one.clone();
        let mut checks_until_resource_poll = 0_usize;
        while remaining > one {
            result *= remaining.clone();
            remaining -= BigInt::from(1_u8);

            if checks_until_resource_poll == 0 {
                self.enforce_active_resource_limits_for_expr(&Expr::integer(result.clone()))?;
                checks_until_resource_poll = 256;
            } else {
                checks_until_resource_poll -= 1;
            }
        }

        Ok(Expr::integer(result))
    }

    pub(super) fn eval_factorial2(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Factorial2", args, 1));
        }

        let Some(value) = expr_as_exact_integer(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if value < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        self.enforce_integer_work_limit_for_factorial(&value)?;

        let one = BigInt::from(1_u8);
        let two = BigInt::from(2_u8);
        let mut remaining = value;
        let mut result = one.clone();
        let mut checks_until_resource_poll = 0_usize;
        while remaining > one {
            result *= remaining.clone();
            remaining -= two.clone();

            if checks_until_resource_poll == 0 {
                self.enforce_active_resource_limits_for_expr(&Expr::integer(result.clone()))?;
                checks_until_resource_poll = 256;
            } else {
                checks_until_resource_poll -= 1;
            }
        }

        Ok(Expr::integer(result))
    }

    fn enforce_integer_work_limit_for_factorial(&self, value: &BigInt) -> Result<(), EvalError> {
        let one = BigInt::from(1_u8);
        let work_units = if value > &one {
            value - one
        } else {
            BigInt::from(0_u8)
        };
        let limit = self.resource_limits.integer_work_limit;
        if work_units > BigInt::from(limit) {
            return Err(EvalError::IntegerWorkLimitExceeded { limit });
        }

        Ok(())
    }
}
