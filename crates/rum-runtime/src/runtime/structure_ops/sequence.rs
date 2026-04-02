use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructuralSequenceSpec {
    All,
    None,
    Count(isize),
    Single(isize),
    Range {
        start: isize,
        end: isize,
        step: isize,
    },
}

impl Session {
    pub(in crate::runtime) fn eval_take(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Take",
                args,
                "argx",
                "expected at least 2 arguments",
            ));
        }

        let Some(specs) = parse_structural_sequence_specs(&args[1..]) else {
            return Ok(Expr::call(system_expr("Take"), args.to_vec()));
        };

        match apply_take_specs(&args[0], &specs) {
            Some(result) => Ok(result),
            None if matches!(args[0], Expr::Atom(_)) => {
                Ok(self.invalid_normal_expression_call("Take", args))
            }
            None => Ok(self.invalid_take_drop_sequence_call("Take", "take", args)),
        }
    }

    pub(in crate::runtime) fn eval_drop(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Drop",
                args,
                "argx",
                "expected at least 2 arguments",
            ));
        }

        let Some(specs) = parse_structural_sequence_specs(&args[1..]) else {
            return Ok(Expr::call(system_expr("Drop"), args.to_vec()));
        };

        match apply_drop_specs(&args[0], &specs) {
            Some(result) => Ok(result),
            None if matches!(args[0], Expr::Atom(_)) => {
                Ok(self.invalid_normal_expression_call("Drop", args))
            }
            None => Ok(self.invalid_take_drop_sequence_call("Drop", "drop", args)),
        }
    }

    pub(in crate::runtime) fn eval_take_drop(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "TakeDrop",
                args,
                "argx",
                "expected at least 2 arguments",
            ));
        }

        let Some(specs) = parse_structural_sequence_specs(&args[1..]) else {
            return Ok(Expr::call(system_expr("TakeDrop"), args.to_vec()));
        };

        match (
            apply_take_specs(&args[0], &specs),
            apply_drop_specs(&args[0], &specs),
        ) {
            (Some(taken), Some(dropped)) => {
                Ok(Expr::call(system_expr("List"), vec![taken, dropped]))
            }
            _ if matches!(args[0], Expr::Atom(_)) => {
                Ok(self.invalid_normal_expression_call("TakeDrop", args))
            }
            _ => Ok(self.invalid_call_with_message(
                "TakeDrop",
                args,
                "take",
                format!(
                    "Cannot apply the requested sequence specification in {}.",
                    Expr::call(system_expr("TakeDrop"), args.to_vec())
                ),
            )),
        }
    }

    pub(in crate::runtime) fn eval_insert(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, item, position] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Insert", args, 3));
        };

        let Some(position) = expr_as_signed_isize(position) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_normal_expression_call("Insert", args));
        };

        let Some(index) = normalize_insert_arg_position(position, expr_args.len()) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut inserted = expr_args.clone();
        inserted.insert(index, item.clone());
        Ok(Expr::call(expr_head.as_ref().clone(), inserted))
    }

    pub(in crate::runtime) fn eval_delete(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, positions_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Delete", args, 2));
        };

        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_normal_expression_call("Delete", args));
        };

        let Some(positions) = parse_delete_positions(positions_expr, expr_args.len()) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let dropped = positions.into_iter().collect::<BTreeSet<_>>();
        let kept = expr_args
            .iter()
            .enumerate()
            .filter(|(index, _)| !dropped.contains(index))
            .map(|(_, arg)| arg.clone())
            .collect::<Vec<_>>();
        Ok(Expr::call(expr_head.as_ref().clone(), kept))
    }

    pub(in crate::runtime) fn eval_take_list(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, counts] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("TakeList", args, 2));
        };

        let Expr::Call { .. } = expr else {
            return Ok(self.invalid_normal_expression_call("TakeList", args));
        };

        let Some(counts) = parse_take_list_counts(counts) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(chunks) = apply_take_list_counts(expr, &counts) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(system_expr("List"), chunks))
    }

    fn invalid_take_drop_sequence_call(&mut self, symbol: &str, tag: &str, args: &[Expr]) -> Expr {
        self.push_message(
            symbol,
            tag,
            format!(
                "Cannot apply the requested sequence specification in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        );
        Expr::call(system_expr(symbol), args.to_vec())
    }
}

fn parse_structural_sequence_specs(args: &[Expr]) -> Option<Vec<StructuralSequenceSpec>> {
    args.iter().map(parse_structural_sequence_spec).collect()
}

fn parse_structural_sequence_spec(expr: &Expr) -> Option<StructuralSequenceSpec> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(StructuralSequenceSpec::All);
    }

    if is_system_symbol_name(expr, heads::structure::NONE) {
        return Some(StructuralSequenceSpec::None);
    }

    if let Some(count) = expr_as_signed_isize(expr) {
        return Some(StructuralSequenceSpec::Count(count));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    match args.as_slice() {
        [index] => Some(StructuralSequenceSpec::Single(expr_as_signed_isize(index)?)),
        [start, end] => Some(StructuralSequenceSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: 1,
        }),
        [start, end, step] => Some(StructuralSequenceSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: expr_as_signed_isize(step)?,
        }),
        _ => None,
    }
}

fn apply_take_specs(expr: &Expr, specs: &[StructuralSequenceSpec]) -> Option<Expr> {
    let Some((first, rest)) = specs.split_first() else {
        return Some(expr.clone());
    };

    let Expr::Call { .. } = expr else {
        return None;
    };

    let components = resolve_take_components(*first, structural_length(expr))?;
    let selectors = vec![PartSelector::Multi(
        components
            .into_iter()
            .map(|component| PartSelector::Single(AtomicPartSpec::Index(component)))
            .collect(),
    )];
    let selected = apply_part_selectors(expr, &selectors)?;

    if rest.is_empty() {
        Some(selected)
    } else {
        apply_take_drop_rest_to_expr(selected, rest, apply_take_specs)
    }
}

fn apply_drop_specs(expr: &Expr, specs: &[StructuralSequenceSpec]) -> Option<Expr> {
    let Some((first, rest)) = specs.split_first() else {
        return Some(expr.clone());
    };

    let Expr::Call { head, args } = expr else {
        return None;
    };

    let dropped = resolve_take_components(*first, args.len())?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let kept_args = args
        .iter()
        .enumerate()
        .filter(|(index, _)| !dropped.contains(&(*index as isize + 1)))
        .map(|(_, arg)| arg.clone())
        .collect::<Vec<_>>();
    let kept = Expr::call(head.as_ref().clone(), kept_args);

    if rest.is_empty() {
        Some(kept)
    } else {
        apply_take_drop_rest_to_expr(kept, rest, apply_drop_specs)
    }
}

fn apply_take_drop_rest_to_expr(
    expr: Expr,
    rest: &[StructuralSequenceSpec],
    recurse: fn(&Expr, &[StructuralSequenceSpec]) -> Option<Expr>,
) -> Option<Expr> {
    match expr {
        Expr::Atom(_) => None,
        Expr::Call { head, args } => Some(Expr::call(
            head.as_ref().clone(),
            args.iter()
                .map(|arg| recurse(arg, rest))
                .collect::<Option<Vec<_>>>()?,
        )),
    }
}

fn resolve_take_components(spec: StructuralSequenceSpec, len: usize) -> Option<Vec<isize>> {
    match spec {
        StructuralSequenceSpec::All => Some((1..=len).map(|index| index as isize).collect()),
        StructuralSequenceSpec::None => Some(Vec::new()),
        StructuralSequenceSpec::Count(0) => Some(Vec::new()),
        StructuralSequenceSpec::Count(count) if count > 0 => {
            let count = usize::try_from(count).ok()?;
            (count <= len).then(|| (1..=count).map(|index| index as isize).collect())
        }
        StructuralSequenceSpec::Count(count) => {
            let count = count.unsigned_abs();
            if count > len {
                None
            } else {
                Some(
                    ((len - count + 1)..=len)
                        .map(|index| index as isize)
                        .collect(),
                )
            }
        }
        StructuralSequenceSpec::Single(index) => {
            Some(vec![normalize_exact_structural_component(index, len)?])
        }
        StructuralSequenceSpec::Range { start, end, step } => {
            if step == 0 {
                return None;
            }

            let start = normalize_exact_structural_component(start, len)?;
            let end = normalize_exact_structural_component(end, len)?;
            let mut current = start;
            let mut components = Vec::new();

            if step > 0 {
                while current <= end {
                    components.push(current);
                    current += step;
                }
            } else {
                while current >= end {
                    components.push(current);
                    current += step;
                }
            }

            Some(components)
        }
    }
}

fn normalize_exact_structural_component(position: isize, len: usize) -> Option<isize> {
    Some(normalize_structural_arg_position(position, len)? as isize + 1)
}

fn normalize_insert_arg_position(position: isize, len: usize) -> Option<usize> {
    let len = isize::try_from(len).ok()?;
    if position > 0 {
        if position <= len + 1 {
            return Some((position - 1) as usize);
        }
        return None;
    }

    if position < 0 {
        let normalized = len + position + 1;
        if (0..=len).contains(&normalized) {
            return Some(normalized as usize);
        }
    }

    None
}

fn parse_delete_positions(positions_expr: &Expr, len: usize) -> Option<Vec<usize>> {
    if let Some(position) = expr_as_signed_isize(positions_expr) {
        return Some(vec![normalize_structural_arg_position(position, len)?]);
    }

    let Expr::Call { head, args } = positions_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) || args.is_empty() {
        return None;
    }

    args.iter()
        .map(|expr| normalize_structural_arg_position(expr_as_signed_isize(expr)?, len))
        .collect()
}

fn parse_take_list_counts(counts_expr: &Expr) -> Option<Vec<usize>> {
    let Expr::Call { head, args } = counts_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    args.iter()
        .map(|arg| {
            let count = expr_as_signed_isize(arg)?;
            (count >= 0).then_some(count as usize)
        })
        .collect()
}

fn apply_take_list_counts(expr: &Expr, counts: &[usize]) -> Option<Vec<Expr>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    let mut offset = 0_usize;
    let mut chunks = Vec::with_capacity(counts.len());
    for count in counts {
        let end = offset.checked_add(*count)?;
        if end > args.len() {
            return None;
        }

        chunks.push(Expr::call(
            head.as_ref().clone(),
            args[offset..end].to_vec(),
        ));
        offset = end;
    }

    Some(chunks)
}
