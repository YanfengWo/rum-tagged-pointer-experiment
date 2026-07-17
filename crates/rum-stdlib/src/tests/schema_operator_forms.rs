use crate::install_core;
use rum_runtime::{RuntimeMessage, Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn schema_operator_forms_cover_success_failure_and_error_lists() {
    let mut session = core_session();
    let success = session
        .eval_text(r#"ConformSchema[<|"a" -> _Integer|>][<|"a" -> 1|>]"#)
        .unwrap();
    assert_eq!(success.value.to_string(), "Association[Rule[\"a\", 1]]");
    assert!(success.messages.is_empty());

    let mut direct_session = core_session();
    let direct_failure = direct_session
        .eval_text(r#"ConformSchema[<|"a" -> "x"|>, <|"a" -> _Integer|>]"#)
        .unwrap();
    let operator_failure = session
        .eval_text(r#"ConformSchema[<|"a" -> _Integer|>][<|"a" -> "x"|>]"#)
        .unwrap();
    assert_eq!(operator_failure.value, direct_failure.value);
    assert_eq!(operator_failure.messages, direct_failure.messages);
    assert_eq!(operator_failure.value.to_string(), "$Failed");
    assert_eq!(
        operator_failure.messages,
        vec![RuntimeMessage {
            symbol: "ConformSchema".to_string(),
            tag: "schema".to_string(),
            text: "a: expected to match _Integer, got \"x\"".to_string(),
        }]
    );

    let mut direct_session = core_session();
    let direct_errors = direct_session
        .eval_text(r#"SchemaErrors[<|"a" -> "x"|>, <|"a" -> _Integer|>]"#)
        .unwrap();
    let operator_errors = session
        .eval_text(r#"SchemaErrors[<|"a" -> _Integer|>][<|"a" -> "x"|>]"#)
        .unwrap();
    assert_eq!(operator_errors.value, direct_errors.value);
    assert_eq!(operator_errors.messages, direct_errors.messages);
    assert_eq!(
        operator_errors.value.to_string(),
        "List[Association[Rule[\"Path\", \"a\"], Rule[\"Message\", \"expected to match _Integer, got \\\"x\\\"\"]]]"
    );
    assert!(operator_errors.messages.is_empty());
}

#[test]
fn schema_capture_is_quiet_and_defers_schema_side_effects_until_validation() {
    let mut operator_session = core_session();
    let capture = operator_session
        .eval_text(r#"c = 0; w = ConformSchema[(c = c + 1; <|"a" -> _|>)]; c"#)
        .unwrap();
    assert_eq!(capture.value.to_string(), "0");
    assert!(capture.messages.is_empty());

    let operator = operator_session.eval_text(r#"w[<|"a" -> 1|>]"#).unwrap();
    let operator_final_counter = operator_session.eval_text("c").unwrap();

    let mut direct_session = core_session();
    direct_session.eval_text("c = 0").unwrap();
    let direct = direct_session
        .eval_text(r#"ConformSchema[<|"a" -> 1|>, (c = c + 1; <|"a" -> _|>)]"#)
        .unwrap();
    let direct_final_counter = direct_session.eval_text("c").unwrap();

    assert_eq!(operator.value, direct.value);
    assert_eq!(operator.messages, direct.messages);
    assert_eq!(operator_final_counter.value, direct_final_counter.value);
    assert_eq!(operator.value.to_string(), "$Failed");
    assert_eq!(operator_final_counter.value.to_string(), "1");
}

#[test]
fn schema_operator_value_side_effect_fires_once() {
    let mut session = core_session();
    let outcome = session
        .eval_text(r#"d = 0; {ConformSchema[<|"a" -> _Integer|>][(d = d + 1; <|"a" -> 1|>)], d}"#)
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[Association[Rule[\"a\", 1]], 1]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn schema_captures_are_inert_and_wrong_application_arity_reports_argx() {
    let mut session = core_session();
    let captures = session
        .eval_text(r#"{ConformSchema[<|"a" -> _Integer|>], SchemaErrors[<|"a" -> _Integer|>]}"#)
        .unwrap();
    assert_eq!(
        captures.value.to_string(),
        "List[ConformSchema[Association[Rule[\"a\", Blank[Integer]]]], SchemaErrors[Association[Rule[\"a\", Blank[Integer]]]]]"
    );
    assert!(captures.messages.is_empty());

    for symbol in ["ConformSchema", "SchemaErrors"] {
        let source = format!(r#"{symbol}[<|"a" -> _Integer|>][1, 2]"#);
        let outcome = session.eval_text(&source).unwrap();
        assert_eq!(
            outcome.value.to_string(),
            format!("{symbol}[Association[Rule[\"a\", Blank[Integer]]]][1, 2]")
        );
        assert_eq!(
            outcome.messages,
            vec![RuntimeMessage {
                symbol: symbol.to_string(),
                tag: "argx".to_string(),
                text: format!(
                    "{symbol}[Association[Rule[\"a\", Blank[Integer]]]] called with 2 arguments; 1 argument is expected."
                ),
            }]
        );
    }
}

#[test]
fn direct_schema_calls_keep_attributes_and_allow_extra_keys_option() {
    let mut session = core_session();
    let attributes = session
        .eval_text("{Attributes[ConformSchema], Attributes[SchemaErrors]}")
        .unwrap();
    assert_eq!(
        attributes.value.to_string(),
        "List[List[HoldRest, Protected], List[HoldRest, Protected]]"
    );
    assert!(attributes.messages.is_empty());

    let conform = session
        .eval_text(
            r#"ConformSchema[<|"a" -> 1, "extra" -> 2|>, <|"a" -> _Integer|>, "AllowExtraKeys" -> True]"#,
        )
        .unwrap();
    assert_eq!(
        conform.value.to_string(),
        "Association[Rule[\"a\", 1], Rule[\"extra\", 2]]"
    );
    assert!(conform.messages.is_empty());

    let errors = session
        .eval_text(
            r#"SchemaErrors[<|"a" -> 1, "extra" -> 2|>, <|"a" -> _Integer|>, "AllowExtraKeys" -> True]"#,
        )
        .unwrap();
    assert_eq!(errors.value.to_string(), "List[]");
    assert!(errors.messages.is_empty());
}

#[test]
fn direct_schema_value_evaluation_resumes_after_input() {
    let mut session = core_session();
    let failure = session
        .eval_request(r#"ConformSchema[InputString["value: "], _String]"#)
        .expect_err("schema value evaluation should request input");

    match failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("schema validation should resume after value input");
    assert_eq!(response.value.to_string(), "\"ok\"");
    assert!(response.messages.is_empty());
}
