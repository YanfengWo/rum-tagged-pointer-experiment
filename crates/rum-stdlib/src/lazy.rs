use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use rum_core::{BigInt, Expr, Rational};
use rum_runtime::kernel_api::{
    EvalError, Session, expr_as_exact_rational, expr_as_non_negative_usize, is_system_symbol_name,
    system_expr, truthiness_symbol,
};

#[derive(Clone, Copy)]
enum PullBudget {
    AtMost(usize),
    UntilExhausted,
}

impl PullBudget {
    fn reached(self, pulled: usize) -> bool {
        match self {
            Self::AtMost(limit) => pulled >= limit,
            Self::UntilExhausted => false,
        }
    }
}

struct PullContext {
    iteration_limit: usize,
    source_pulls: usize,
    interrupt: Arc<AtomicBool>,
}

impl PullContext {
    fn new(session: &Session) -> Self {
        Self {
            iteration_limit: session.resource_limits().iteration_work_limit,
            source_pulls: 0,
            interrupt: session.interrupt_handle(),
        }
    }

    fn charge_source_pull(&mut self) -> Result<(), EvalError> {
        if self.interrupt.load(Ordering::Relaxed) {
            return Err(EvalError::Cancelled);
        }
        if self.source_pulls >= self.iteration_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: self.iteration_limit,
            });
        }

        self.source_pulls += 1;
        Ok(())
    }
}

enum PullStepper {
    Range {
        current: Rational,
        step: Rational,
    },
    Map {
        function: Expr,
        inner: Box<PullStepper>,
    },
    Select {
        predicate: Expr,
        inner: Box<PullStepper>,
    },
    Take {
        inner: Box<PullStepper>,
        remaining: usize,
    },
    Drop {
        inner: Box<PullStepper>,
        remaining: usize,
    },
}

impl PullStepper {
    fn next(
        &mut self,
        session: &mut Session,
        ctx: &mut PullContext,
    ) -> Result<Option<Expr>, EvalError> {
        match self {
            Self::Range { current, step } => {
                ctx.charge_source_pull()?;
                let value = Expr::rational(current.clone());
                *current = current.add(step);
                Ok(Some(value))
            }
            Self::Map { function, inner } => {
                let Some(value) = inner.next(session, ctx)? else {
                    return Ok(None);
                };
                Ok(Some(
                    session.evaluate(Expr::call(function.clone(), vec![value]))?,
                ))
            }
            Self::Select { predicate, inner } => loop {
                let Some(value) = inner.next(session, ctx)? else {
                    return Ok(None);
                };
                let selected =
                    session.evaluate(Expr::call(predicate.clone(), vec![value.clone()]))?;
                if truthiness_symbol(&selected) == Some(true) {
                    return Ok(Some(value));
                }
            },
            Self::Take { inner, remaining } => {
                if *remaining == 0 {
                    return Ok(None);
                }
                *remaining -= 1;
                inner.next(session, ctx)
            }
            Self::Drop { inner, remaining } => {
                while *remaining > 0 {
                    let Some(_) = inner.next(session, ctx)? else {
                        *remaining = 0;
                        return Ok(None);
                    };
                    *remaining -= 1;
                }
                inner.next(session, ctx)
            }
        }
    }
}

pub(crate) fn eval_lazy_drop(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lazy_list, count_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyDrop", args, 2));
    };
    let (Some(descriptor), Some(count)) = (
        lazy_list_descriptor(lazy_list),
        expr_as_non_negative_usize(count_expr),
    ) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyDrop",
            args,
            "expected a LazyList and a non-negative integer",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyDrop"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn eval_lazy_map(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [function, lazy_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyMap", args, 2));
    };
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyMap",
            args,
            "expected a LazyList as the second argument",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyMap"),
        vec![function.clone(), descriptor.clone()],
    )))
}

pub(crate) fn eval_lazy_range(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() > 2 {
        return Ok(session.invalid_call_with_arg_range("LazyRange", args, 0, 2));
    }

    let start = args
        .first()
        .cloned()
        .unwrap_or_else(|| Expr::small_integer(1));
    let step = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| Expr::small_integer(1));

    if expr_as_exact_rational(&start).is_none() || expr_as_exact_rational(&step).is_none() {
        return Ok(session.invalid_call_with_arg_form(
            "LazyRange",
            args,
            "start and step must be Integer or Rational values",
        ));
    }

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyRange"),
        vec![start, step],
    )))
}

pub(crate) fn eval_lazy_select(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [predicate, lazy_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazySelect", args, 2));
    };
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazySelect",
            args,
            "expected a LazyList as the second argument",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazySelect"),
        vec![predicate.clone(), descriptor.clone()],
    )))
}

pub(crate) fn eval_lazy_take(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lazy_list, count_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyTake", args, 2));
    };
    let (Some(descriptor), Some(count)) = (
        lazy_list_descriptor(lazy_list),
        parse_lazy_take_count(count_expr),
    ) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyTake",
            args,
            "expected a LazyList and a non-negative integer or UpTo[n]",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyTake"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn is_lazy_list(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, "LazyList")
    )
}

pub(crate) fn lazy_drop(lazy_list: &Expr, count: usize) -> Option<Expr> {
    let descriptor = lazy_list_descriptor(lazy_list)?;
    Some(wrap_descriptor(Expr::call(
        system_expr("LazyDrop"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn realize_all(
    session: &mut Session,
    lazy_list: &Expr,
) -> Result<Option<Expr>, EvalError> {
    realize(session, lazy_list, PullBudget::UntilExhausted)
}

pub(crate) fn realize_at_most(
    session: &mut Session,
    lazy_list: &Expr,
    limit: usize,
) -> Result<Option<Expr>, EvalError> {
    realize(session, lazy_list, PullBudget::AtMost(limit))
}

fn realize(
    session: &mut Session,
    lazy_list: &Expr,
    budget: PullBudget,
) -> Result<Option<Expr>, EvalError> {
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(None);
    };
    let Some(mut stepper) = stepper_from_descriptor(descriptor) else {
        return Ok(None);
    };
    let mut ctx = PullContext::new(session);
    let mut items = Vec::new();

    loop {
        if budget.reached(items.len()) {
            return Ok(Some(Expr::call(system_expr("List"), items)));
        }
        let Some(item) = stepper.next(session, &mut ctx)? else {
            return Ok(Some(Expr::call(system_expr("List"), items)));
        };
        items.push(item);
    }
}

fn lazy_list_descriptor(expr: &Expr) -> Option<&Expr> {
    if !is_lazy_list(expr) {
        return None;
    }
    let Expr::Call { args, .. } = expr else {
        return None;
    };
    let [descriptor] = args.as_slice() else {
        return None;
    };
    Some(descriptor)
}

fn parse_lazy_take_count(expr: &Expr) -> Option<usize> {
    if let Some(count) = expr_as_non_negative_usize(expr) {
        return Some(count);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "UpTo") {
        return None;
    }
    let [count] = args.as_slice() else {
        return None;
    };
    expr_as_non_negative_usize(count)
}

fn stepper_from_descriptor(descriptor: &Expr) -> Option<PullStepper> {
    let Expr::Call { head, args } = descriptor else {
        return None;
    };

    if is_system_symbol_name(head, "LazyRange") {
        let [start, step] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Range {
            current: expr_as_exact_rational(start)?,
            step: expr_as_exact_rational(step)?,
        });
    }
    if is_system_symbol_name(head, "LazyMap") {
        let [function, inner] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Map {
            function: function.clone(),
            inner: Box::new(stepper_from_descriptor(inner)?),
        });
    }
    if is_system_symbol_name(head, "LazySelect") {
        let [predicate, inner] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Select {
            predicate: predicate.clone(),
            inner: Box::new(stepper_from_descriptor(inner)?),
        });
    }
    if is_system_symbol_name(head, "LazyTake") {
        let [inner, count] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Take {
            inner: Box::new(stepper_from_descriptor(inner)?),
            remaining: expr_as_non_negative_usize(count)?,
        });
    }
    if is_system_symbol_name(head, "LazyDrop") {
        let [inner, count] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Drop {
            inner: Box::new(stepper_from_descriptor(inner)?),
            remaining: expr_as_non_negative_usize(count)?,
        });
    }

    None
}

fn wrap_descriptor(descriptor: Expr) -> Expr {
    Expr::call(system_expr("LazyList"), vec![descriptor])
}
