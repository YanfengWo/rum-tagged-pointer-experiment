use std::sync::atomic::Ordering;

use rum_core::{Expr, Rational};
use rum_runtime::kernel_api::{
    EvalError, Session, expr_as_exact_rational, is_system_symbol_name, system_expr,
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

    let descriptor = Expr::call(system_expr("LazyRange"), vec![start, step]);
    Ok(Expr::call(system_expr("LazyList"), vec![descriptor]))
}

pub(crate) fn is_lazy_list(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, "LazyList")
    )
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
    let Some(items) = pull_descriptor(session, descriptor, budget)? else {
        return Ok(None);
    };

    Ok(Some(Expr::call(system_expr("List"), items)))
}

fn lazy_list_descriptor(expr: &Expr) -> Option<&Expr> {
    let Expr::Call { args, .. } = expr else {
        return None;
    };
    let [descriptor] = args.as_slice() else {
        return None;
    };
    Some(descriptor)
}

fn pull_descriptor(
    session: &mut Session,
    descriptor: &Expr,
    budget: PullBudget,
) -> Result<Option<Vec<Expr>>, EvalError> {
    match descriptor {
        Expr::Call { head, args } if is_system_symbol_name(head, "LazyRange") => {
            let [start, step] = args.as_slice() else {
                return Ok(None);
            };
            let Some(start) = expr_as_exact_rational(start) else {
                return Ok(None);
            };
            let Some(step) = expr_as_exact_rational(step) else {
                return Ok(None);
            };

            pull_lazy_range(session, start, step, budget).map(Some)
        }
        _ => Ok(None),
    }
}

fn pull_lazy_range(
    session: &mut Session,
    mut current: Rational,
    step: Rational,
    budget: PullBudget,
) -> Result<Vec<Expr>, EvalError> {
    let iteration_limit = session.resource_limits().iteration_work_limit;
    let interrupt = session.interrupt_handle();
    let mut items = Vec::new();

    loop {
        if budget.reached(items.len()) {
            return Ok(items);
        }
        if interrupt.load(Ordering::Relaxed) {
            return Err(EvalError::Cancelled);
        }
        if items.len() >= iteration_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: iteration_limit,
            });
        }

        items.push(Expr::rational(current.clone()));
        current = current.add(&step);
    }
}
