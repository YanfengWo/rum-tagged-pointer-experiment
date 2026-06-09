use super::*;

impl Session {
    pub(in crate::runtime) fn eval_range(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let Some((start, end, step)) = range_spec(args) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(values) = integer_range_values(start, end, step) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(system_expr(heads::structure::LIST), values))
    }
}

fn range_spec(args: &[Expr]) -> Option<(BigInt, BigInt, BigInt)> {
    match args {
        [end] => Some((
            BigInt::from(1_u8),
            expr_as_exact_integer(end)?,
            BigInt::from(1_u8),
        )),
        [start, end] => Some((
            expr_as_exact_integer(start)?,
            expr_as_exact_integer(end)?,
            BigInt::from(1_u8),
        )),
        [start, end, step] => Some((
            expr_as_exact_integer(start)?,
            expr_as_exact_integer(end)?,
            expr_as_exact_integer(step)?,
        )),
        _ => None,
    }
}

fn integer_range_values(start: BigInt, end: BigInt, step: BigInt) -> Option<Vec<Expr>> {
    let zero = BigInt::from(0_u8);
    if step == zero {
        return None;
    }

    let mut values = Vec::new();
    let mut current = start;

    if step > zero {
        while current <= end {
            values.push(Expr::integer(current.clone()));
            current += step.clone();
        }
    } else {
        while current >= end {
            values.push(Expr::integer(current.clone()));
            current += step.clone();
        }
    }

    Some(values)
}
