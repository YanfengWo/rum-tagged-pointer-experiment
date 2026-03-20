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
fn context_variables_and_builtin_inspection_reflect_session_contexts() {
    let mut session = Session::new();
    session.set_current_context("Rules");

    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"System`\", \"Global`\"]"
    );

    let info = session.inspect("Identity").unwrap();
    assert_eq!(info.qualified_name, "System`Identity");
    assert_eq!(info.context, "System`");
    assert_eq!(info.display_name, "Identity");
    assert!(info.is_builtin);
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

#[test]
fn bare_symbols_search_context_path_before_current_context() {
    let mut session = Session::new();

    session.eval_text("Global`x = 10").unwrap();
    session.set_current_context("Rules");
    session.eval_text("Rules`x = 20").unwrap();
    session.eval_text("local = 30").unwrap();

    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");

    let info = session.inspect("local").unwrap();
    assert_eq!(info.qualified_name, "Rules`local");
    assert_eq!(info.context, "Rules`");
    assert_eq!(
        session.eval_text("Rules`x").unwrap().value.to_string(),
        "20"
    );
}

#[test]
fn context_builtin_reports_session_and_symbol_contexts_without_evaluating_symbols() {
    let mut session = Session::new();

    session.eval_text("x = 1").unwrap();

    assert_eq!(
        session.eval_text("Context[]").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session.eval_text("Context[x]").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session
            .eval_text("Context[Identity]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Context[\"Identity\"]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Context[\"MissingSymbol\"]")
            .unwrap()
            .value
            .to_string(),
        "Context[\"MissingSymbol\"]"
    );
}

#[test]
fn begin_and_end_balance_context_and_context_path() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Begin[\"Rules`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Rules`\", \"System`\", \"Global`\"]"
    );

    assert_eq!(
        session
            .eval_text("Begin[\"`Private`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("End[]").unwrap().value.to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Rules`\", \"System`\", \"Global`\"]"
    );
}

#[test]
fn begin_package_and_end_package_restore_outer_context_and_export_package_context() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("BeginPackage[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Demo`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Demo`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Demo`\", \"System`\"]"
    );

    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("hidden = 1").unwrap();
    assert_eq!(
        session.eval_text("End[]").unwrap().value.to_string(),
        "\"Demo`Private`\""
    );

    session.eval_text("public = 2").unwrap();

    assert_eq!(
        session.eval_text("EndPackage[]").unwrap().value.to_string(),
        "Null"
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Demo`\", \"System`\", \"Global`\"]"
    );
    assert_eq!(session.eval_text("public").unwrap().value.to_string(), "2");
    assert_eq!(
        session.eval_text("hidden").unwrap().value.to_string(),
        "hidden"
    );
}

#[test]
fn get_loads_a_script_file_and_returns_the_last_value() {
    let dir = TempDirGuard::new("get");
    let file = dir.write("script.rum", "x = 1\npair = {\n  x,\n  2\n}\npair");
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
}

#[test]
fn split_script_into_forms_stops_hard_syntax_errors_from_poisoning_later_forms() {
    let forms = split_script_into_forms("1 2\n3").unwrap();

    assert_eq!(forms, vec!["1 2".to_string(), "3".to_string()]);
}

#[test]
fn eval_script_preserves_messages_emitted_before_a_later_error() {
    let mut session = Session::new();

    let error = session
        .eval_script("FindFile[\"Missing`\"]\nself := self\nself")
        .expect_err("expected a later evaluation error");

    assert!(matches!(
        error,
        EvalError::EvaluationCycleDetected { ref expr } if expr == "self"
    ));
    assert!(
        session
            .take_messages()
            .iter()
            .any(|message| { message.symbol == "FindFile" && message.tag == "notfound" })
    );
}
