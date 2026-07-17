use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructuralSequenceSpec {
    All,
    None,
    UpTo(Option<isize>),
    Count(isize),
    Single(isize),
    Range {
        start: isize,
        end: isize,
        step: isize,
    },
}

pub(crate) fn eval_take(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() < 2 {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_message(
            "Take",
            args,
            "argx",
            "expected at least 2 arguments",
        ));
    }

    let Some(specs) = parse_structural_sequence_specs(session, &args[1..]) else {
        return Ok(Expr::call(system_expr("Take"), args.to_vec()));
    };

    match apply_take_specs(&args[0], &specs) {
        Some(result) => Ok(result),
        None if matches!(args[0], Expr::Atom(_)) => Ok(invalid_take_drop_atom_call(
            session, "Take", "take", args, specs[0],
        )),
        None => Ok(invalid_take_drop_sequence_call(
            session, "Take", "take", args,
        )),
    }
}

pub(crate) fn eval_drop(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() < 2 {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_message(
            "Drop",
            args,
            "argx",
            "expected at least 2 arguments",
        ));
    }

    let Some(specs) = parse_structural_sequence_specs(session, &args[1..]) else {
        return Ok(Expr::call(system_expr("Drop"), args.to_vec()));
    };

    match apply_drop_specs(&args[0], &specs) {
        Some(result) => Ok(result),
        None if matches!(args[0], Expr::Atom(_)) => Ok(invalid_take_drop_atom_call(
            session, "Drop", "drop", args, specs[0],
        )),
        None => Ok(invalid_take_drop_sequence_call(
            session, "Drop", "drop", args,
        )),
    }
}

pub(crate) fn eval_take_drop(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() < 2 {
        return Ok(session.invalid_call_with_message(
            "TakeDrop",
            args,
            "argx",
            "expected at least 2 arguments",
        ));
    }

    let Some(specs) = parse_structural_sequence_specs(session, &args[1..]) else {
        return Ok(Expr::call(system_expr("TakeDrop"), args.to_vec()));
    };

    match (
        apply_take_specs(&args[0], &specs),
        apply_drop_specs(&args[0], &specs),
    ) {
        (Some(taken), Some(dropped)) => Ok(Expr::call(system_expr("List"), vec![taken, dropped])),
        _ if matches!(args[0], Expr::Atom(_)) => {
            Ok(invalid_take_drop_atom_sequence_call(session, args))
        }
        _ => Ok(session.invalid_call_with_message(
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

fn invalid_take_drop_sequence_call(
    session: &mut Session,
    symbol: &str,
    tag: &str,
    args: &[Expr],
) -> Expr {
    session.push_message(
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
    session: &mut Session,
    symbol: &str,
    tag: &str,
    args: &[Expr],
    spec: StructuralSequenceSpec,
) -> Expr {
    let (start, end) = take_drop_message_range(spec);
    session.push_message(
        symbol,
        tag,
        format!(
            "Cannot {} positions {} through {} in {}.",
            tag, start, end, args[0]
        ),
    );
    Expr::call(system_expr(symbol), args.to_vec())
}

fn invalid_take_drop_atom_sequence_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.push_message(
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

fn take_drop_message_range(spec: StructuralSequenceSpec) -> (isize, isize) {
    match spec {
        StructuralSequenceSpec::All => (1, -1),
        StructuralSequenceSpec::None => (1, 0),
        StructuralSequenceSpec::UpTo(Some(limit)) => (1, limit),
        StructuralSequenceSpec::UpTo(None) => (1, -1),
        StructuralSequenceSpec::Count(count) if count >= 0 => (1, count),
        StructuralSequenceSpec::Count(count) => (count, -1),
        StructuralSequenceSpec::Single(index) => (index, index),
        StructuralSequenceSpec::Range { start, end, .. } => (start, end),
    }
}

fn parse_structural_sequence_specs(
    session: &mut Session,
    args: &[Expr],
) -> Option<Vec<StructuralSequenceSpec>> {
    args.iter()
        .map(|expr| parse_structural_sequence_spec(session, expr))
        .collect()
}

fn parse_structural_sequence_spec(
    session: &mut Session,
    expr: &Expr,
) -> Option<StructuralSequenceSpec> {
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

    if is_system_symbol_name(head, "UpTo") {
        let [limit] = args.as_slice() else {
            session.push_message(
                "UpTo",
                "argx",
                format!(
                    "UpTo called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            );
            return None;
        };

        if is_system_symbol_name(limit, heads::structure::INFINITY) {
            return Some(StructuralSequenceSpec::UpTo(None));
        }

        if let Some(limit) = expr_as_signed_isize(limit).filter(|limit| *limit >= 0) {
            return Some(StructuralSequenceSpec::UpTo(Some(limit)));
        }

        // Message only on definitely-invalid contents (negative integer or
        // rational), matching the kernel Part-span rule; anything else stays
        // on the silent inert path like any other out-of-subset spec.
        let definitely_invalid = match limit {
            Expr::Atom(Atom::Integer(value)) => value.to_bigint() < BigInt::from(0),
            Expr::Atom(Atom::Rational(_)) => true,
            _ => false,
        };
        if definitely_invalid {
            session.push_message(
                "UpTo",
                "innf",
                format!("Non-negative integer or Infinity expected at position 1 in {expr}."),
            );
        }
        return None;
    }

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
        StructuralSequenceSpec::UpTo(limit) => {
            let end = match limit {
                Some(limit) => usize::try_from(limit).ok()?.min(len),
                None => len,
            };
            Some((1..=end).map(|index| index as isize).collect())
        }
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
