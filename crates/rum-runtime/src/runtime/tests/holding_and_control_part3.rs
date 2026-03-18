use super::*;

#[test]
fn configurable_replace_repeated_limit_is_respected() {
    let mut session = Session::new();
    session.set_replace_repeated_limit(8);

    let error = session
        .eval_text("1 //. {1 -> 2, 2 -> 1}")
        .expect_err("expected iteration limit error");

    assert!(matches!(
        error,
        EvalError::IterationLimitExceeded { limit: 8 }
    ));
    assert_eq!(session.resource_limits().replace_repeated_limit, 8);
}

#[test]
fn with_evaluates_bindings_and_substitutes_lexically() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("With[{x = Identity[1], y = x}, y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("With[{x = 1}, Hold[x]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[1]"
    );
}

#[test]
fn with_supports_rule_style_bindings_and_nested_shadowing() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("With[{x -> 1}, x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("With[{x = 1}, With[{x = 2}, x]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn with_respects_pattern_local_names_but_substitutes_pattern_metadata() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("With[{x = 1}, 7 /. x_Integer :> x]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("With[{t = Integer}, MatchQ[7, x_t]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn module_generates_fresh_lexical_symbols_and_initializes_sequentially() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Module[{x = 1, y = x}, y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );

    let first = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;
    let second = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;

    let first_text = first.to_string();
    let second_text = second.to_string();

    assert!(first_text.starts_with("Hold[x$"));
    assert!(second_text.starts_with("Hold[x$"));
    assert_ne!(first_text, second_text);
}

#[test]
fn module_respects_nested_shadowing_and_pattern_locals() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Module[{x = 1}, Module[{x = 2}, x]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    let nested = session
        .eval_text("Module[{x = 1}, Module[{x}, Hold[x]]]")
        .unwrap()
        .value
        .to_string();
    assert!(nested.starts_with("Hold[x$"));
    assert_ne!(nested, "Hold[x]");
    assert_ne!(nested, "Hold[1]");
    assert_eq!(
        session
            .eval_text("Module[{x = 1}, 7 /. x_Integer :> x]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn block_temporarily_rebinds_and_restores_symbol_values() {
    let mut session = Session::new();

    session.eval_text("x = 10").unwrap();

    assert_eq!(
        session
            .eval_text("Block[{x = 1}, x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
    assert_eq!(
        session
            .eval_text("Block[{x}, x]")
            .unwrap()
            .value
            .to_string(),
        "10"
    );
    assert_eq!(
        session
            .eval_text("Block[{x}, Hold[x]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[x]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
}

#[test]
fn block_applies_dynamic_scope_across_called_definitions() {
    let mut session = Session::new();

    session.eval_text("x = 10").unwrap();
    session.eval_text("f[] := x").unwrap();

    assert_eq!(
        session
            .eval_text("Block[{x = 1}, f[]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("f[]").unwrap().value.to_string(), "10");
    assert_eq!(
        session
            .eval_text("Block[{x}, x = 3]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
}

#[test]
fn block_supports_sequential_initializers_and_nested_shadowing() {
    let mut session = Session::new();

    session.eval_text("x = 10").unwrap();

    assert_eq!(
        session
            .eval_text("Block[{x = 1, y = x}, y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Block[{x -> 1}, x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Block[{x = 1}, Block[{x = 2}, x]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
}
