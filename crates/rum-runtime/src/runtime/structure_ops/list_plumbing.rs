use super::*;

impl Session {
    pub(in crate::runtime) fn eval_differences(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (list, order) = match args {
            [list] => (list, 1),
            [list, order_expr] => {
                let Some(order) = expr_as_non_negative_usize(order_expr) else {
                    return Ok(self.invalid_differences_order_call(args));
                };
                (list, order)
            }
            _ => return Ok(self.invalid_call_with_arg_range("Differences", args, 1, 2)),
        };

        let Some(items) = list_items(list) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut current = items.to_vec();
        for _ in 0..order {
            current = self.successive_differences(&current)?;
            if current.is_empty() {
                break;
            }
        }

        self.enforce_generated_element_limit_for_count(current.len())?;
        Ok(Expr::call(system_expr(heads::structure::LIST), current))
    }

    pub(in crate::runtime) fn eval_constant_array(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [value, dims_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("ConstantArray", args, 2));
        };

        let Some(shape) = parse_non_negative_dimensions(dims_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        self.check_generated_shape_counts(&shape)?;

        Ok(constant_array_expr(value, &shape))
    }

    pub(in crate::runtime) fn eval_pad_left(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_pad(args, PadSide::Left)
    }

    pub(in crate::runtime) fn eval_pad_right(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_pad(args, PadSide::Right)
    }

    fn successive_differences(&mut self, items: &[Expr]) -> Result<Vec<Expr>, EvalError> {
        if items.len() < 2 {
            return Ok(Vec::new());
        }

        let mut differences = Vec::with_capacity(items.len() - 1);
        for pair in items.windows(2) {
            differences.push(self.difference_expr(&pair[0], &pair[1])?);
        }
        Ok(differences)
    }

    fn difference_expr(&mut self, left: &Expr, right: &Expr) -> Result<Expr, EvalError> {
        if let (Some(left), Some(right)) =
            (expr_as_exact_integer(left), expr_as_exact_integer(right))
        {
            return Ok(Expr::integer(right - left));
        }

        self.evaluate(Expr::call(
            system_expr("Plus"),
            vec![
                right.clone(),
                Expr::call(
                    system_expr("Times"),
                    vec![Expr::small_integer(-1), left.clone()],
                ),
            ],
        ))
    }

    fn eval_pad(&mut self, args: &[Expr], side: PadSide) -> Result<Expr, EvalError> {
        let symbol = side.symbol();
        let (expr, target_len, padding) = match args {
            [expr, target_len] => (expr, target_len, None),
            [expr, target_len, padding] => (expr, target_len, Some(padding)),
            _ => return Ok(self.invalid_call_with_arg_range(symbol, args, 2, 3)),
        };

        let Some(target_len) = expr_as_non_negative_usize(target_len) else {
            return Ok(self.invalid_pad_length_call(symbol, args));
        };

        let Expr::Call { head, args: items } = expr else {
            return Ok(self.invalid_pad_normal_call(symbol, args));
        };

        self.enforce_generated_element_limit_for_count(target_len)?;
        self.enforce_active_memory_limit_for_projected_bytes(
            generated_expr_nodes_memory_proxy_bytes(target_len.saturating_add(1)),
        )?;

        let result = pad_items(items, target_len, padding, side)
            .map(|items| Expr::call(head.as_ref().clone(), items))
            .unwrap_or_else(|| Expr::call(system_expr(symbol), args.to_vec()));
        Ok(result)
    }

    fn invalid_differences_order_call(&mut self, args: &[Expr]) -> Expr {
        self.invalid_call_with_message(
            "Differences",
            args,
            "ilsmn",
            format!(
                "Single or list of non-negative machine-sized integers expected at position 2 of {}.",
                Expr::call(system_expr("Differences"), args.to_vec())
            ),
        )
    }

    fn invalid_pad_normal_call(&mut self, symbol: &str, args: &[Expr]) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "normal",
            format!(
                "Nonatomic expression expected at position 1 in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        )
    }

    fn invalid_pad_length_call(&mut self, symbol: &str, args: &[Expr]) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "ilsm",
            format!(
                "List of machine-sized integers expected at position 2 in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        )
    }

    fn check_generated_shape_counts(&self, shape: &[usize]) -> Result<(), EvalError> {
        let shape = shape
            .iter()
            .map(|dim| BigInt::from(*dim))
            .collect::<Vec<_>>();
        let scalar_count = shape_scalar_count(&shape);
        let node_count = shape_generated_node_count(&shape);
        let generated_count = scalar_count.max(node_count.clone());
        self.enforce_active_memory_limit_for_projected_bytes(
            generated_expr_nodes_memory_proxy_bytes(
                bigint_count_to_usize(&node_count).unwrap_or(usize::MAX),
            ),
        )?;
        self.enforce_generated_element_limit_for_big_count(&generated_count)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PadSide {
    Left,
    Right,
}

impl PadSide {
    fn symbol(self) -> &'static str {
        match self {
            Self::Left => "PadLeft",
            Self::Right => "PadRight",
        }
    }
}

fn parse_non_negative_dimensions(expr: &Expr) -> Option<Vec<usize>> {
    if let Some(dim) = expr_as_non_negative_usize(expr) {
        return Some(vec![dim]);
    }

    let items = list_items(expr)?;
    items.iter().map(expr_as_non_negative_usize).collect()
}

fn constant_array_expr(value: &Expr, shape: &[usize]) -> Expr {
    let Some((dim, rest)) = shape.split_first() else {
        return value.clone();
    };

    Expr::call(
        system_expr(heads::structure::LIST),
        (0..*dim)
            .map(|_| constant_array_expr(value, rest))
            .collect(),
    )
}

fn pad_items(
    items: &[Expr],
    target_len: usize,
    padding: Option<&Expr>,
    side: PadSide,
) -> Option<Vec<Expr>> {
    if items.len() >= target_len {
        return Some(match side {
            PadSide::Left => items[items.len() - target_len..].to_vec(),
            PadSide::Right => items[..target_len].to_vec(),
        });
    }

    let pad_count = target_len - items.len();
    let pad_items = padding_items(padding, pad_count, side)?;
    let mut result = Vec::with_capacity(target_len);

    match side {
        PadSide::Left => {
            result.extend(pad_items);
            result.extend_from_slice(items);
        }
        PadSide::Right => {
            result.extend_from_slice(items);
            result.extend(pad_items);
        }
    }

    Some(result)
}

fn padding_items(padding: Option<&Expr>, pad_count: usize, side: PadSide) -> Option<Vec<Expr>> {
    let Some(padding) = padding else {
        return Some(vec![Expr::small_integer(0); pad_count]);
    };

    if let Some(cycle) = list_items(padding) {
        if cycle.is_empty() {
            return None;
        }
        let start = match side {
            PadSide::Left => (cycle.len() - (pad_count % cycle.len())) % cycle.len(),
            PadSide::Right => 0,
        };
        return Some(
            (0..pad_count)
                .map(|index| cycle[(start + index) % cycle.len()].clone())
                .collect(),
        );
    }

    Some(vec![padding.clone(); pad_count])
}

fn shape_scalar_count(shape: &[BigInt]) -> BigInt {
    shape.iter().cloned().product::<BigInt>()
}

fn shape_generated_node_count(shape: &[BigInt]) -> BigInt {
    let Some((first, rest)) = shape.split_first() else {
        return BigInt::from(1_u8);
    };

    BigInt::from(1_u8) + first * shape_generated_node_count(rest)
}

fn bigint_count_to_usize(count: &BigInt) -> Option<usize> {
    count.to_string().parse().ok()
}

fn generated_expr_nodes_memory_proxy_bytes(count: usize) -> usize {
    count.saturating_mul(std::mem::size_of::<Expr>())
}
