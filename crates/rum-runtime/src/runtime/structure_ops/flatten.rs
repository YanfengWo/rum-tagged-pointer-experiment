use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlattenDepth {
    Finite(usize),
    Infinity,
}

impl Session {
    pub(in crate::runtime) fn eval_flatten(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, depth) = match args {
            [expr] => (expr, FlattenDepth::Infinity),
            [expr, depth_expr] => {
                let Some(depth) = parse_flatten_depth(depth_expr) else {
                    return Ok(self.invalid_flatten_level_call(args, depth_expr));
                };
                (expr, depth)
            }
            _ => return Ok(self.invalid_call_with_arg_range("Flatten", args, 1, 2)),
        };

        if matches!(expr, Expr::Atom(_)) {
            return Ok(self.invalid_normal_expression_call("Flatten", args));
        }

        Ok(flatten_same_head(expr, depth))
    }

    fn invalid_flatten_level_call(&mut self, args: &[Expr], depth_expr: &Expr) -> Expr {
        let call = Expr::call(system_expr("Flatten"), args.to_vec());
        self.invalid_call_with_message(
            "Flatten",
            args,
            "flev",
            format!(
                "The level argument {depth_expr} in position 2 of {call} should be a non-negative integer or Infinity."
            ),
        )
    }
}

fn parse_flatten_depth(expr: &Expr) -> Option<FlattenDepth> {
    if is_system_symbol_name(expr, "Infinity") {
        Some(FlattenDepth::Infinity)
    } else {
        expr_as_non_negative_usize(expr).map(FlattenDepth::Finite)
    }
}

fn flatten_same_head(expr: &Expr, depth: FlattenDepth) -> Expr {
    let Expr::Call { head, args } = expr else {
        return expr.clone();
    };

    if depth == FlattenDepth::Finite(0) {
        return expr.clone();
    }

    let target_head = head.as_ref();
    let mut flattened = Vec::new();
    for arg in args {
        flatten_matching_arg(arg, target_head, depth, &mut flattened);
    }

    Expr::call(target_head.clone(), flattened)
}

fn flatten_matching_arg(
    expr: &Expr,
    target_head: &Expr,
    depth: FlattenDepth,
    flattened: &mut Vec<Expr>,
) {
    match depth {
        FlattenDepth::Finite(0) => flattened.push(expr.clone()),
        FlattenDepth::Finite(level) => match expr {
            Expr::Call { head, args } if head.as_ref() == target_head => {
                for arg in args {
                    flatten_matching_arg(
                        arg,
                        target_head,
                        FlattenDepth::Finite(level.saturating_sub(1)),
                        flattened,
                    );
                }
            }
            _ => flattened.push(expr.clone()),
        },
        FlattenDepth::Infinity => match expr {
            Expr::Call { head, args } if head.as_ref() == target_head => {
                for arg in args {
                    flatten_matching_arg(arg, target_head, FlattenDepth::Infinity, flattened);
                }
            }
            _ => flattened.push(expr.clone()),
        },
    }
}
