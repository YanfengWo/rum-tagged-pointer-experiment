use super::*;

#[test]
fn evaluation_data_captures_result_messages_prints_and_failure_type_without_leaking_streams() {
    let mut session = Session::new();

    let success = session.eval_text("EvaluationData[Echo[1 + 1]; 3]").unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![success.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![success.value.clone(), Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![success.value.clone(), Expr::string("OutputLog")],
            ))
            .unwrap()
            .to_string(),
        "List[\"2\"]"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![success.value.clone(), Expr::string("Input")],
            ))
            .unwrap()
            .to_string(),
        "Hold[CompoundExpression[Echo[Plus[1, 1]], 3]]"
    );
    assert!(session.take_messages().is_empty());
    assert!(session.take_prints().is_empty());

    let failure = session.eval_text("EvaluationData[Abort[]]").unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![failure.value.clone(), Expr::string("FailureType")],
            ))
            .unwrap()
            .to_string(),
        "\"Abort\""
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![failure.value, Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "$Aborted"
    );

    let projected = session
        .eval_text("EvaluationData[Echo[1 + 1]; 3, {\"Result\", \"Success\"}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![projected.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![projected.value, Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    let symbol_projected = session
        .eval_text("EvaluationData[Echo[1 + 1]; 3, {Result, \"Success\"}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![symbol_projected.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![symbol_projected.value, Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    let properties = session
        .eval_text("EvaluationData[3, \"Properties\"]")
        .unwrap();
    assert!(properties.value.to_string().contains("\"Result\""));
    assert!(properties.value.to_string().contains("\"Trace\""));

    let symbol_properties = session.eval_text("EvaluationData[3, Properties]").unwrap();
    assert!(symbol_properties.value.to_string().contains("\"Result\""));
    assert!(symbol_properties.value.to_string().contains("\"Trace\""));

    let all_projection = session.eval_text("EvaluationData[3, All]").unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Head"),
                vec![all_projection.value.clone()]
            ))
            .unwrap()
            .to_string(),
        "Association"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![all_projection.value, Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "3"
    );

    session.eval_text("f[x_] := x + 1").unwrap();
    let trace_projection = session
        .eval_text("EvaluationData[f[1], \"Trace\"]")
        .unwrap();
    assert_eq!(
        trace_projection.value.to_string(),
        "List[Hold[f[1]], List[Hold[Plus[1, 1]], Hold[2]], Hold[2]]"
    );

    let message_exprs = session
        .eval_text("EvaluationData[Message[FindFile::notfound], \"MessagesExpressions\"]")
        .unwrap();
    assert_eq!(
        message_exprs.value.to_string(),
        "List[Hold[Message[FindFile::notfound]]]"
    );
    assert_eq!(
        session
            .eval_text("EvaluationData[3, Missing]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Missing]"
    );
    assert_eq!(
        session
            .eval_text("EvaluationData[3, \"Missing\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", \"Missing\"]"
    );
    let mixed_missing_projection = session
        .eval_text("EvaluationData[3, {Result, Missing}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![
                    mixed_missing_projection.value.clone(),
                    Expr::string("Result")
                ],
            ))
            .unwrap()
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![mixed_missing_projection.value, Expr::string("Missing")],
            ))
            .unwrap()
            .to_string(),
        "Missing[\"KeyAbsent\", Missing]"
    );
}
