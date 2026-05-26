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
                Ok(self.invalid_take_drop_atom_call("Take", "take", args, specs[0]))
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
                Ok(self.invalid_take_drop_atom_call("Drop", "drop", args, specs[0]))
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
                Ok(self.invalid_take_drop_atom_sequence_call(args))
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

        if matches!(expr, Expr::Atom(_)) {
            return Ok(self.invalid_insert_position_call(args));
        }

        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_normal_expression_call("Insert", args));
        };

        let Some(index) = normalize_insert_arg_position(position, expr_args.len()) else {
            return Ok(self.invalid_insert_position_call(args));
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

        if matches!(expr, Expr::Atom(_)) {
            if let Some(part) = delete_atom_part_message_part(positions_expr) {
                return Ok(self.invalid_delete_atom_part_call(args, &part));
            }
            return Ok(self.invalid_normal_expression_call("Delete", args));
        }

        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_normal_expression_call("Delete", args));
        };

        let Some(paths) = parse_delete_position_paths(positions_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut canonical_paths = Vec::new();
        for path in paths {
            match canonical_delete_path(expr, &path) {
                Ok(path) => canonical_paths.push(path),
                Err(failure) => return Ok(self.invalid_delete_path_call(args, failure)),
            }
        }

        Ok(delete_canonical_paths(
            Expr::call(expr_head.as_ref().clone(), expr_args.clone()),
            canonical_paths,
        ))
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
            return Ok(self.invalid_sequence_normal_expression_call("TakeList", args));
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

    fn invalid_take_drop_atom_call(
        &mut self,
        symbol: &str,
        tag: &str,
        args: &[Expr],
        spec: StructuralSequenceSpec,
    ) -> Expr {
        let (start, end) = take_drop_message_range(spec);
        self.push_message(
            symbol,
            tag,
            format!(
                "Cannot {} positions {} through {} in {}.",
                tag, start, end, args[0]
            ),
        );
        Expr::call(system_expr(symbol), args.to_vec())
    }

    fn invalid_take_drop_atom_sequence_call(&mut self, args: &[Expr]) -> Expr {
        self.push_message(
            "TakeDrop",
            "iseq",
            format!(
                "Invalid sequence specification {} for an expression of length {}.",
                args[1],
                structural_length(&args[0])
            ),
        );
        Expr::call(system_expr("TakeDrop"), args.to_vec())
    }

    fn invalid_insert_position_call(&mut self, args: &[Expr]) -> Expr {
        self.push_message(
            "Insert",
            "ins",
            format!("Cannot insert at position {{{}}} in {}", args[2], args[0]),
        );
        Expr::call(system_expr("Insert"), args.to_vec())
    }

    fn invalid_delete_atom_part_call(&mut self, args: &[Expr], part: &Expr) -> Expr {
        self.push_message(
            "Delete",
            "partw",
            format!("Part {} of {} does not exist.", part, args[0]),
        );
        Expr::call(system_expr("Delete"), args.to_vec())
    }

    fn invalid_delete_path_call(&mut self, args: &[Expr], failure: DeletePathFailure) -> Expr {
        self.push_message(
            "Delete",
            "partw",
            format!(
                "Part {} of {} does not exist.",
                failure.part, failure.target
            ),
        );
        Expr::call(system_expr("Delete"), args.to_vec())
    }

    fn invalid_sequence_normal_expression_call(&mut self, symbol: &str, args: &[Expr]) -> Expr {
        self.push_message(
            symbol,
            "normal",
            format!(
                "Nonatomic expression expected at position 1 in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        );
        Expr::call(system_expr(symbol), args.to_vec())
    }
}

fn delete_atom_part_message_part(positions_expr: &Expr) -> Option<Expr> {
    if expr_as_signed_isize(positions_expr).is_some() {
        return Some(positions_expr.clone());
    }

    let Expr::Call { head, args } = positions_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let part = args.first()?;
    expr_as_signed_isize(part).is_some().then(|| part.clone())
}

fn take_drop_message_range(spec: StructuralSequenceSpec) -> (isize, isize) {
    match spec {
        StructuralSequenceSpec::All => (1, -1),
        StructuralSequenceSpec::None => (1, 0),
        StructuralSequenceSpec::Count(count) if count >= 0 => (1, count),
        StructuralSequenceSpec::Count(count) => (count, -1),
        StructuralSequenceSpec::Single(index) => (index, index),
        StructuralSequenceSpec::Range { start, end, .. } => (start, end),
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeletePathFailure {
    part: String,
    target: Expr,
}

fn parse_delete_position_paths(positions_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    if let Some(position) = expr_as_signed_isize(positions_expr) {
        return Some(vec![vec![position]]);
    }

    let Expr::Call { head, args } = positions_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    if args.is_empty() {
        return Some(Vec::new());
    }

    if args.iter().all(|expr| expr_as_signed_isize(expr).is_some()) {
        return Some(vec![
            args.iter()
                .map(expr_as_signed_isize)
                .collect::<Option<Vec<_>>>()?,
        ]);
    }

    args.iter().map(parse_delete_position_path_list).collect()
}

fn parse_delete_position_path_list(expr: &Expr) -> Option<Vec<isize>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) || args.is_empty() {
        return None;
    }

    args.iter().map(expr_as_signed_isize).collect()
}

fn canonical_delete_path(expr: &Expr, path: &[isize]) -> Result<Vec<isize>, DeletePathFailure> {
    let mut target = expr.clone();
    let mut canonical = Vec::with_capacity(path.len());

    for (depth, position) in path.iter().copied().enumerate() {
        if position == 0 {
            match &target {
                Expr::Atom(_) => {
                    return Err(DeletePathFailure {
                        part: delete_failure_part(path, depth, position),
                        target,
                    });
                }
                Expr::Call { .. } => {
                    canonical.push(0);
                    target = structural_head_expr(&target);
                }
            }
            continue;
        }

        let Expr::Call { args, .. } = &target else {
            return Err(DeletePathFailure {
                part: delete_failure_part(path, depth, position),
                target,
            });
        };

        let Some(index) = normalize_structural_arg_position(position, args.len()) else {
            return Err(DeletePathFailure {
                part: delete_failure_part(path, depth, position),
                target,
            });
        };
        canonical.push(index as isize + 1);
        target = args[index].clone();
    }

    Ok(canonical)
}

fn delete_failure_part(path: &[isize], depth: usize, position: isize) -> String {
    if depth == 0 {
        return format!(
            "{{{}}}",
            path.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    position.to_string()
}

fn delete_canonical_paths(expr: Expr, mut paths: Vec<Vec<isize>>) -> Expr {
    paths.sort_by(|left, right| right.cmp(left));
    paths.dedup();
    paths
        .into_iter()
        .fold(expr, |expr, path| delete_canonical_path(expr, &path))
}

fn delete_canonical_path(expr: Expr, path: &[isize]) -> Expr {
    let Some((first, rest)) = path.split_first() else {
        return expr;
    };

    match expr {
        Expr::Atom(_) => expr,
        Expr::Call { head, args } if *first == 0 => {
            if rest.is_empty() {
                return expression_without_head(args);
            }

            let replaced_head = delete_canonical_path(head.as_ref().clone(), rest);
            Expr::call(replaced_head, args)
        }
        Expr::Call { head, mut args } => {
            let index = (*first as usize).saturating_sub(1);
            if index >= args.len() {
                return Expr::Call { head, args };
            }

            if rest.is_empty() {
                args.remove(index);
            } else {
                args[index] = delete_canonical_path(args[index].clone(), rest);
            }
            Expr::Call { head, args }
        }
    }
}

fn expression_without_head(args: Vec<Expr>) -> Expr {
    match args.as_slice() {
        [single] => single.clone(),
        _ => Expr::call(system_expr("Sequence"), args),
    }
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
