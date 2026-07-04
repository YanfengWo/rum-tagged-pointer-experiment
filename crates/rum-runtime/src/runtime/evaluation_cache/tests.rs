use super::*;

fn session_with_compound_expression() -> Session {
    let mut session = Session::default();
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "CompoundExpression",
        "Test-local moved control builtin.",
        HoldAttribute::HoldAll,
        test_compound_expression,
    ));
    session
}

fn test_compound_expression(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    session.eval_control_data_builtin(ControlDataBuiltin::CompoundExpression, head, args)
}

#[test]
fn evaluation_cache_is_invalidated_by_new_down_value() {
    let mut session = Session::default();

    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "f[1]");
    assert!(!session.symbol_call_effect_cache.is_empty());

    session.eval_text("f[x_] := x + 1").unwrap();

    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "2");
}

#[test]
fn evaluation_cache_is_invalidated_by_attribute_change() {
    let mut session = Session::default();

    assert_eq!(
        session.eval_text("f[{a, b}]").unwrap().value.to_string(),
        "f[List[a, b]]"
    );
    assert!(!session.symbol_call_effect_cache.is_empty());

    session.eval_text("SetAttributes[f, Listable]").unwrap();

    assert_eq!(
        session.eval_text("f[{a, b}]").unwrap().value.to_string(),
        "List[f[a], f[b]]"
    );
}

#[test]
fn repeated_writes_to_one_symbol_keep_invalidation_sound() {
    let mut session = session_with_compound_expression();

    session.eval_text("g[] := (s = s + 1; s)").unwrap();
    session.eval_text("s = 0").unwrap();
    assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "1");
    assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "2");
    assert_eq!(session.eval_text("s").unwrap().value.to_string(), "2");
}

#[test]
fn stability_cache_survives_unrelated_symbol_writes() {
    let mut session = Session::default();

    // Populate the call-stability cache with a stable expression.
    assert_eq!(
        session.eval_text("h[1, 2]").unwrap().value.to_string(),
        "h[1, 2]"
    );

    // Writing an unrelated symbol must not change the stable verdict...
    session.eval_text("unrelated = 5").unwrap();
    assert_eq!(
        session.eval_text("h[1, 2]").unwrap().value.to_string(),
        "h[1, 2]"
    );

    // ...while a write to the observed head must flip it.
    session.eval_text("h[x_, y_] := x + y").unwrap();
    assert_eq!(session.eval_text("h[1, 2]").unwrap().value.to_string(), "3");
}

#[test]
fn repeated_unrelated_writes_do_not_bump_call_stability_generation() {
    let mut session = Session::default();

    session.eval_text("stableExpr = h[1, 2, 3]").unwrap();
    session.clear_evaluation_caches();
    assert_eq!(
        session.eval_text("stableExpr").unwrap().value.to_string(),
        "h[1, 2, 3]"
    );
    assert!(!session.call_stability_cache.is_empty());

    let generation = session.call_stability_generation;
    session.eval_text("unrelatedWrite = 1").unwrap();
    session.eval_text("unrelatedWrite = 2").unwrap();
    session.eval_text("unrelatedWrite = 3").unwrap();

    assert_eq!(session.call_stability_generation, generation);
    assert_eq!(
        session.eval_text("stableExpr").unwrap().value.to_string(),
        "h[1, 2, 3]"
    );
}
