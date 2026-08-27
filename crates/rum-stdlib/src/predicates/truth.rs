use super::*;
use rum_core::SharedArgs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TruthAggregateKind {
    All,
    Any,
    None,
}

pub(crate) fn eval_true_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("TrueQ", args, 1));
    };

    Ok(predicate_bool_expr(is_system_symbol_name(
        expr,
        heads::structure::TRUE,
    )))
}

pub(crate) fn eval_boolean_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("BooleanQ", args, 1));
    };

    Ok(predicate_bool_expr(truthiness_symbol(expr).is_some()))
}

pub(crate) fn eval_boole(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Boole", args, 1));
    };

    Ok(match truthiness_symbol(expr) {
        Some(true) => Expr::integer(BigInt::from(1_i64)),
        Some(false) => Expr::integer(BigInt::from(0_i64)),
        None => Expr::call(head.clone(), args.to_vec()),
    })
}

pub(crate) fn eval_all_true(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_truth_aggregate(session, head, args, TruthAggregateKind::All)
}

pub(crate) fn eval_any_true(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_truth_aggregate(session, head, args, TruthAggregateKind::Any)
}

pub(crate) fn eval_none_true(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_truth_aggregate(session, head, args, TruthAggregateKind::None)
}

fn eval_truth_aggregate(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    kind: TruthAggregateKind,
) -> Result<Expr, EvalError> {
    if args.len() == 1 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    if !(2..=3).contains(&args.len()) {
        let symbol = truth_aggregate_name(kind);
        return Ok(session.invalid_call_with_arg_range(symbol, args, 2, 3));
    }

    let traversal = if let Some(levelspec) = args.get(2) {
        let Some(levels) = parse_standard_level_spec(levelspec) else {
            return Ok(session.invalid_call_with_arg_form(
                truth_aggregate_name(kind),
                args,
                "expected a standard levelspec as the third argument",
            ));
        };
        CasesTraversalConfig::default_cases().with_levels(levels)
    } else {
        CasesTraversalConfig::default_cases()
    };

    let parts = collect_level_parts_owned(&args[0], traversal, 0);

    resume_truth_aggregate_from_index(
        session,
        kind,
        head.clone(),
        args.to_vec(),
        parts,
        args[1].clone(),
        0,
        false,
    )
}

fn resume_truth_aggregate_from_index(
    session: &mut Session,
    kind: TruthAggregateKind,
    head: Expr,
    call_args: Vec<Expr>,
    parts: Vec<Expr>,
    criterion: Expr,
    mut index: usize,
    mut saw_unknown: bool,
) -> Result<Expr, EvalError> {
    while let Some(part) = parts.get(index).cloned() {
        let call = Expr::call(criterion.clone(), vec![part]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_truth_aggregate_with_value(
                                session,
                                kind,
                                head,
                                call_args,
                                parts,
                                criterion,
                                index,
                                saw_unknown,
                                value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        match truthiness_symbol(&value) {
            Some(true) => match kind {
                TruthAggregateKind::All => {}
                TruthAggregateKind::Any => return Ok(system_expr(heads::structure::TRUE)),
                TruthAggregateKind::None => return Ok(system_expr(heads::structure::FALSE)),
            },
            Some(false) => match kind {
                TruthAggregateKind::All => return Ok(system_expr(heads::structure::FALSE)),
                TruthAggregateKind::Any | TruthAggregateKind::None => {}
            },
            None => saw_unknown = true,
        }
        index += 1;
    }

    Ok(finalize_truth_aggregate(kind, head, call_args, saw_unknown))
}

#[allow(clippy::too_many_arguments)]
fn resume_truth_aggregate_with_value(
    session: &mut Session,
    kind: TruthAggregateKind,
    head: Expr,
    call_args: Vec<Expr>,
    parts: Vec<Expr>,
    criterion: Expr,
    index: usize,
    saw_unknown: bool,
    value: Expr,
) -> Result<Expr, EvalError> {
    match truthiness_symbol(&value) {
        Some(true) => match kind {
            TruthAggregateKind::All => resume_truth_aggregate_from_index(
                session,
                kind,
                head,
                call_args,
                parts,
                criterion,
                index + 1,
                saw_unknown,
            ),
            TruthAggregateKind::Any => Ok(system_expr(heads::structure::TRUE)),
            TruthAggregateKind::None => Ok(system_expr(heads::structure::FALSE)),
        },
        Some(false) => match kind {
            TruthAggregateKind::All => Ok(system_expr(heads::structure::FALSE)),
            TruthAggregateKind::Any | TruthAggregateKind::None => {
                resume_truth_aggregate_from_index(
                    session,
                    kind,
                    head,
                    call_args,
                    parts,
                    criterion,
                    index + 1,
                    saw_unknown,
                )
            }
        },
        None => resume_truth_aggregate_from_index(
            session,
            kind,
            head,
            call_args,
            parts,
            criterion,
            index + 1,
            true,
        ),
    }
}

fn truth_aggregate_name(kind: TruthAggregateKind) -> &'static str {
    match kind {
        TruthAggregateKind::All => "AllTrue",
        TruthAggregateKind::Any => "AnyTrue",
        TruthAggregateKind::None => "NoneTrue",
    }
}

fn finalize_truth_aggregate(
    kind: TruthAggregateKind,
    head: Expr,
    call_args: Vec<Expr>,
    saw_unknown: bool,
) -> Expr {
    if saw_unknown {
        Expr::call(head, call_args)
    } else {
        match kind {
            TruthAggregateKind::All | TruthAggregateKind::None => {
                system_expr(heads::structure::TRUE)
            }
            TruthAggregateKind::Any => system_expr(heads::structure::FALSE),
        }
    }
}
