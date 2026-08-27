use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_composition(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(simplify_composition_head(head, args))
}

pub(crate) fn eval_right_composition(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(simplify_composition_head(head, args))
}

fn simplify_composition_head(head: &Expr, args: &[Expr]) -> Expr {
    let normalized = args
        .iter()
        .filter(|arg| !is_system_symbol_name(arg, "Identity"))
        .cloned()
        .collect::<Vec<_>>();

    match normalized.as_slice() {
        [] => system_expr("Identity"),
        [single] => single.clone(),
        _ => Expr::call(head.clone(), normalized),
    }
}
