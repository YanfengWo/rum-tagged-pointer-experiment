use super::*;

impl Session {
    pub(in crate::runtime) fn eval_subsets(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, size_spec) = match args {
            [expr] => (expr, None),
            [expr, size_spec] => (expr, Some(size_spec)),
            _ => return Ok(self.invalid_call_with_arg_range("Subsets", args, 1, 2)),
        };

        let Expr::Call {
            head: expr_head,
            args: items,
        } = expr
        else {
            return Ok(self.invalid_subsets_normal_call(args));
        };

        let Some(range) = parse_subset_size_range(size_spec, items.len()) else {
            return Ok(self.invalid_subsets_size_spec_call(args));
        };

        let subset_count = subset_range_count(items.len(), range.min, range.max);
        self.enforce_generated_element_limit_for_big_count(&subset_count)?;
        self.enforce_active_memory_limit_for_projected_bytes(
            generated_expr_nodes_memory_proxy_bytes(
                bigint_count_to_usize(&subset_count).unwrap_or(usize::MAX),
            ),
        )?;

        let mut subsets = Vec::new();
        for size in range.min..=range.max {
            append_subsets_of_size(
                expr_head.as_ref(),
                items,
                size,
                0,
                &mut Vec::new(),
                &mut subsets,
            );
        }

        Ok(Expr::call(system_expr(heads::structure::LIST), subsets))
    }

    pub(in crate::runtime) fn eval_tuples(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [factors_expr] => self.eval_cartesian_tuples(head, args, factors_expr),
            [expr, length_expr] => self.eval_repeated_tuples(head, args, expr, length_expr),
            _ => Ok(self.invalid_call_with_arg_range("Tuples", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_partition(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, size_expr, offset_expr) = match args {
            [expr, size_expr] => (expr, size_expr, None),
            [expr, size_expr, offset_expr] => (expr, size_expr, Some(offset_expr)),
            _ => return Ok(self.invalid_call_with_arg_range("Partition", args, 2, 3)),
        };

        let Some(size) = expr_as_positive_usize(size_expr) else {
            return Ok(self.invalid_partition_positive_integer_call(args, 2));
        };
        let offset = match offset_expr {
            Some(offset_expr) => {
                let Some(offset) = expr_as_positive_usize(offset_expr) else {
                    return Ok(self.invalid_partition_positive_integer_call(args, 3));
                };
                offset
            }
            None => size,
        };

        let Expr::Call {
            head: expr_head,
            args: items,
        } = expr
        else {
            return Ok(self.invalid_partition_normal_call(args));
        };

        let block_count = partition_block_count(items.len(), size, offset);
        self.enforce_generated_element_limit_for_count(block_count)?;
        self.enforce_active_memory_limit_for_projected_bytes(
            generated_expr_nodes_memory_proxy_bytes(block_count.saturating_mul(size + 1)),
        )?;

        let mut blocks = Vec::with_capacity(block_count);
        let mut start = 0_usize;
        while start.saturating_add(size) <= items.len() {
            blocks.push(Expr::call(
                expr_head.as_ref().clone(),
                items[start..start + size].to_vec(),
            ));
            start = start.saturating_add(offset);
        }

        Ok(Expr::call(expr_head.as_ref().clone(), blocks))
    }

    fn eval_repeated_tuples(
        &mut self,
        _head: &Expr,
        args: &[Expr],
        expr: &Expr,
        length_expr: &Expr,
    ) -> Result<Expr, EvalError> {
        let Some(length) = expr_as_non_negative_usize(length_expr) else {
            return Ok(self.invalid_tuples_length_call(args));
        };

        let Expr::Call {
            head: tuple_head,
            args: items,
        } = expr
        else {
            return Ok(self.invalid_tuples_normal_call(args, "1"));
        };

        let tuple_count = repeated_tuple_count(items.len(), length);
        self.enforce_generated_element_limit_for_big_count(&tuple_count)?;
        self.enforce_active_memory_limit_for_projected_bytes(
            generated_expr_nodes_memory_proxy_bytes(
                bigint_count_to_usize(&tuple_count)
                    .unwrap_or(usize::MAX)
                    .saturating_mul(length + 1),
            ),
        )?;

        let mut tuples = Vec::new();
        append_repeated_tuples(
            tuple_head.as_ref(),
            items,
            length,
            &mut Vec::new(),
            &mut tuples,
        );
        Ok(Expr::call(system_expr(heads::structure::LIST), tuples))
    }

    fn eval_cartesian_tuples(
        &mut self,
        _head: &Expr,
        args: &[Expr],
        factors_expr: &Expr,
    ) -> Result<Expr, EvalError> {
        let Expr::Call {
            head: tuple_head,
            args: factor_exprs,
        } = factors_expr
        else {
            return Ok(self.invalid_tuples_normal_call(args, "1"));
        };

        let mut factors = Vec::with_capacity(factor_exprs.len());
        for (index, factor) in factor_exprs.iter().enumerate() {
            let Expr::Call { args, .. } = factor else {
                return Ok(self.invalid_tuples_normal_call(args, &format!("{{1, {}}}", index + 1)));
            };
            factors.push(args.as_slice());
        }

        let tuple_count = cartesian_tuple_count(&factors);
        self.enforce_generated_element_limit_for_big_count(&tuple_count)?;
        self.enforce_active_memory_limit_for_projected_bytes(
            generated_expr_nodes_memory_proxy_bytes(
                bigint_count_to_usize(&tuple_count)
                    .unwrap_or(usize::MAX)
                    .saturating_mul(factors.len() + 1),
            ),
        )?;

        let mut tuples = Vec::new();
        append_cartesian_tuples(
            tuple_head.as_ref(),
            &factors,
            0,
            &mut Vec::new(),
            &mut tuples,
        );
        Ok(Expr::call(system_expr(heads::structure::LIST), tuples))
    }

    fn invalid_subsets_normal_call(&mut self, args: &[Expr]) -> Expr {
        self.invalid_call_with_message(
            "Subsets",
            args,
            "normal",
            format!(
                "Nonatomic expression expected at position 1 in {}.",
                Expr::call(system_expr("Subsets"), args.to_vec())
            ),
        )
    }

    fn invalid_subsets_size_spec_call(&mut self, args: &[Expr]) -> Expr {
        self.invalid_call_with_message(
            "Subsets",
            args,
            "nninfseq",
            format!(
                "Position 2 of {} must be All, Infinity, nmax, {{nmin}}, {{nmin, nmax}} or {{nmin, nmax, dn}}, where nmin is a non-negative integer, nmax is non-negative integer or Infinity and dn is a nonzero integer.",
                Expr::call(system_expr("Subsets"), args.to_vec())
            ),
        )
    }

    fn invalid_tuples_normal_call(&mut self, args: &[Expr], position: &str) -> Expr {
        self.invalid_call_with_message(
            "Tuples",
            args,
            "normal",
            format!(
                "Nonatomic expression expected at position {position} in {}.",
                Expr::call(system_expr("Tuples"), args.to_vec())
            ),
        )
    }

    fn invalid_tuples_length_call(&mut self, args: &[Expr]) -> Expr {
        self.invalid_call_with_message(
            "Tuples",
            args,
            "ilsmn",
            format!(
                "Single or list of non-negative machine-sized integers expected at position 2 of {}.",
                Expr::call(system_expr("Tuples"), args.to_vec())
            ),
        )
    }

    fn invalid_partition_normal_call(&mut self, args: &[Expr]) -> Expr {
        self.invalid_call_with_message(
            "Partition",
            args,
            "npart",
            format!("The expression {} cannot be partitioned.", args[0]),
        )
    }

    fn invalid_partition_positive_integer_call(&mut self, args: &[Expr], position: usize) -> Expr {
        self.invalid_call_with_message(
            "Partition",
            args,
            "ilsmp",
            format!(
                "Single or list of positive machine-sized integers expected at position {position} of {}.",
                Expr::call(system_expr("Partition"), args.to_vec())
            ),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SubsetSizeRange {
    min: usize,
    max: usize,
}

fn parse_subset_size_range(size_spec: Option<&Expr>, item_count: usize) -> Option<SubsetSizeRange> {
    match size_spec {
        None => Some(SubsetSizeRange {
            min: 0,
            max: item_count,
        }),
        Some(expr) => {
            if let Some(max) = expr_as_non_negative_usize(expr) {
                return Some(SubsetSizeRange {
                    min: 0,
                    max: max.min(item_count),
                });
            }

            let items = list_items(expr)?;
            match items {
                [exact] => {
                    let exact = expr_as_non_negative_usize(exact)?;
                    Some(SubsetSizeRange {
                        min: exact,
                        max: exact.min(item_count),
                    })
                }
                [min, max] => {
                    let min = expr_as_non_negative_usize(min)?;
                    let max = expr_as_non_negative_usize(max)?;
                    Some(SubsetSizeRange {
                        min,
                        max: max.min(item_count),
                    })
                }
                _ => None,
            }
        }
    }
}

fn subset_range_count(item_count: usize, min: usize, max: usize) -> BigInt {
    if min > max {
        return BigInt::from(0_u8);
    }

    (min..=max)
        .map(|size| binomial_count(item_count, size))
        .sum()
}

fn binomial_count(item_count: usize, size: usize) -> BigInt {
    if size > item_count {
        return BigInt::from(0_u8);
    }

    let size = size.min(item_count - size);
    let mut count = BigInt::from(1_u8);
    for step in 1..=size {
        count *= BigInt::from(item_count - size + step);
        count /= BigInt::from(step);
    }
    count
}

fn append_subsets_of_size(
    subset_head: &Expr,
    items: &[Expr],
    size: usize,
    start: usize,
    current: &mut Vec<Expr>,
    subsets: &mut Vec<Expr>,
) {
    if current.len() == size {
        subsets.push(Expr::call(subset_head.clone(), current.clone()));
        return;
    }

    let needed = size - current.len();
    let available = items.len().saturating_sub(start);
    if needed > available {
        return;
    }

    let last_start = items.len() - needed;
    for index in start..=last_start {
        current.push(items[index].clone());
        append_subsets_of_size(subset_head, items, size, index + 1, current, subsets);
        current.pop();
    }
}

fn repeated_tuple_count(item_count: usize, length: usize) -> BigInt {
    let mut count = BigInt::from(1_u8);
    for _ in 0..length {
        count *= BigInt::from(item_count);
    }
    count
}

fn cartesian_tuple_count(factors: &[&[Expr]]) -> BigInt {
    factors
        .iter()
        .map(|items| BigInt::from(items.len()))
        .product()
}

fn append_repeated_tuples(
    tuple_head: &Expr,
    items: &[Expr],
    remaining: usize,
    current: &mut Vec<Expr>,
    tuples: &mut Vec<Expr>,
) {
    if remaining == 0 {
        tuples.push(Expr::call(tuple_head.clone(), current.clone()));
        return;
    }

    for item in items {
        current.push(item.clone());
        append_repeated_tuples(tuple_head, items, remaining - 1, current, tuples);
        current.pop();
    }
}

fn append_cartesian_tuples(
    tuple_head: &Expr,
    factors: &[&[Expr]],
    index: usize,
    current: &mut Vec<Expr>,
    tuples: &mut Vec<Expr>,
) {
    let Some(items) = factors.get(index) else {
        tuples.push(Expr::call(tuple_head.clone(), current.clone()));
        return;
    };

    for item in *items {
        current.push(item.clone());
        append_cartesian_tuples(tuple_head, factors, index + 1, current, tuples);
        current.pop();
    }
}

fn partition_block_count(item_count: usize, size: usize, offset: usize) -> usize {
    if item_count < size {
        0
    } else {
        ((item_count - size) / offset) + 1
    }
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr).filter(|value| *value > 0)
}

fn bigint_count_to_usize(count: &BigInt) -> Option<usize> {
    count.to_string().parse().ok()
}

fn generated_expr_nodes_memory_proxy_bytes(count: usize) -> usize {
    count.saturating_mul(std::mem::size_of::<Expr>())
}
