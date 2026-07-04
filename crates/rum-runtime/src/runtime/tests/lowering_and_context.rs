use super::*;
use rum_syntax::Span;

#[test]
fn lowers_big_integer_into_core_expr() {
    let lowered = lower_syntax_expr(&SyntaxExpr::integer(
        "123456789012345678901234567890",
        Span::default(),
    ))
    .unwrap();

    assert_eq!(lowered.to_string(), "123456789012345678901234567890");
}

#[test]
fn lowers_simple_decimal_literals_to_exact_rationals() {
    let lowered = lower_syntax_expr(&parse_expr("0.01").unwrap()).unwrap();

    assert_eq!(lowered.to_string(), "1/100");

    let lowered = lower_syntax_expr(&parse_expr(".5").unwrap()).unwrap();
    assert_eq!(lowered.to_string(), "1/2");

    let lowered = lower_syntax_expr(&parse_expr("1.").unwrap()).unwrap();
    assert_eq!(lowered.to_string(), "1");

    let lowered = lower_syntax_expr(&parse_expr("1.5e3").unwrap()).unwrap();
    assert_eq!(lowered.to_string(), "1500");
}

#[test]
fn lowering_rejects_invalid_integer_literals_before_runtime() {
    let error = lower_syntax_expr(&SyntaxExpr::integer("not-a-number", Span::default()))
        .expect_err("invalid integer syntax atoms should fail during lowering");

    assert!(matches!(
        error,
        EvalError::InvalidInteger { ref literal } if literal == "not-a-number"
    ));
}

#[test]
fn lowers_message_name_syntax_into_core_expr() {
    let syntax = parse_expr("FindFile::notfound").unwrap();
    let lowered = lower_syntax_expr(&syntax).unwrap();

    assert_eq!(lowered.to_string(), "FindFile::notfound");
}
#[test]
fn lowers_tagset_and_unset_syntax_into_core_expr() {
    let tagged_assignment = parse_expr("f /: g[x_] := rhs").unwrap();
    let lowered_assignment = lower_syntax_expr(&tagged_assignment).unwrap();

    assert_eq!(
        lowered_assignment.to_string(),
        "TagSetDelayed[f, g[Pattern[x, Blank[]]], rhs]"
    );

    let tagged_unset = parse_expr("f /: g[x_] =.").unwrap();
    let lowered_unset = lower_syntax_expr(&tagged_unset).unwrap();

    assert_eq!(
        lowered_unset.to_string(),
        "TagUnset[f, g[Pattern[x, Blank[]]]]"
    );
}

#[test]
fn lowering_preserves_tagset_shape_for_runtime_semantic_validation() {
    let syntax = parse_expr("f /: g[h[f[x_]]] := 1").unwrap();
    let lowered = lower_syntax_expr(&syntax).unwrap();

    assert_eq!(
        lowered.to_string(),
        "TagSetDelayed[f, g[h[f[Pattern[x, Blank[]]]]], 1]"
    );

    let mut session = Session::new();
    let outcome = session.eval_text("f /: g[h[f[x_]]] := 1").unwrap();

    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "TagSetDelayed" && message.tag == "tagpos")
    );
}

#[test]
fn evaluates_identity_call() {
    let mut session = Session::new();
    let outcome = session
        .eval_text("Identity[123456789012345678901234567890]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "123456789012345678901234567890");
}

#[test]
fn compound_expression_sugar_flows_through_lowering_and_evaluation() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("x = 1; y = x + 1; y")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn attributes_reports_builtin_and_user_defined_attribute_sets() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Attributes[If]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldRest]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[HoldAllComplete]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldAllComplete]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[Identity]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[Plus]")
            .unwrap()
            .value
            .to_string(),
        "List[Listable, Orderless, Flat, OneIdentity]"
    );

    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();

    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst]"
    );

    let info = session.inspect("f").unwrap();
    assert_eq!(info.attribute_count, 1);
    assert_eq!(info.attributes, vec!["HoldFirst".to_string()]);
}
