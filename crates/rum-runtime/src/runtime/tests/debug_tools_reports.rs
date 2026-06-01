use super::*;

#[test]
fn testcreate_testreport_and_stack_cover_current_first_pass_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Head[TestCreate[1 + 1, 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association"
    );
    assert_eq!(
        session
            .eval_text("Lookup[TestCreate[1 + 1, 2, {TestID -> \"sum\"}], \"TestID\"]")
            .unwrap()
            .value
            .to_string(),
        "\"sum\""
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], \"TestsRunCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], \"TestsSucceededCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], \"TestsFailedCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], \"Success\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 1, {SameTest -> Greater}], TestCreate[Message[FindFile::notfound], Null, {ExpectedMessages -> {FindFile::notfound}}]}], \"TestsSucceededCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[Part[Lookup[TestReport[{TestCreate[Message[FindFile::notfound], Null, {ExpectedMessages -> {FindFile::notfound}}]}], \"TestResults\"], 1], \"MessagesMatch\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[Part[Lookup[TestReport[{TestCreate[Message[FindFile::notfound], Null, {ExpectedMessages -> {FindFile::notfound}}]}], \"TestResults\"], 1], \"ActualMessagesText\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"message emitted\"]"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[Part[Lookup[TestReport[{TestCreate[1 + 1, 2]}], \"TestResults\"], 1], \"Input\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "Hold[Plus[1, 1]]"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[Part[Lookup[TestReport[{TestCreate[Echo[1 + 1], 2]}], \"TestResults\"], 1], \"OutputLog\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"2\"]"
    );
    assert_eq!(
        session
            .eval_text(
                "NumberQ[Lookup[Part[Lookup[TestReport[{TestCreate[Echo[1 + 1], 2]}], \"TestResults\"], 1], \"Timing\"]]",
            )
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(
                "TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}, \"TestsRunCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}, TestsRunCount]",)
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    let properties = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, \"Properties\"]")
        .unwrap();
    assert!(properties.value.to_string().contains("\"TestsRunCount\""));
    assert!(properties.value.to_string().contains("\"TestResults\""));
    let symbol_properties = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, Properties]")
        .unwrap();
    assert!(
        symbol_properties
            .value
            .to_string()
            .contains("\"TestsRunCount\"")
    );
    assert!(
        symbol_properties
            .value
            .to_string()
            .contains("\"TestResults\"")
    );
    let all_report = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, All]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Head"),
                vec![all_report.value.clone()]
            ))
            .unwrap()
            .to_string(),
        "Association"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![all_report.value, Expr::string("TestsSucceededCount")],
            ))
            .unwrap()
            .to_string(),
        "1"
    );
    let mixed_projection = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, {TestsRunCount, \"Success\"}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![
                    mixed_projection.value.clone(),
                    Expr::string("TestsRunCount")
                ],
            ))
            .unwrap()
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![mixed_projection.value, Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("TestReport[{TestCreate[1 + 1, 2]}, Missing]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Missing]"
    );
    assert_eq!(
        session
            .eval_text("TestReport[{TestCreate[1 + 1, 2]}, \"Missing\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", \"Missing\"]"
    );
    let mixed_missing_report = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, {TestsRunCount, Missing}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![
                    mixed_missing_report.value.clone(),
                    Expr::string("TestsRunCount")
                ],
            ))
            .unwrap()
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![mixed_missing_report.value, Expr::string("Missing")],
            ))
            .unwrap()
            .to_string(),
        "Missing[\"KeyAbsent\", Missing]"
    );
    assert!(session.take_messages().is_empty());

    let stack = session
        .eval_text("stackFrame[] := Stack[]; stackFrame[]")
        .unwrap();
    let rendered = stack.value.to_string();
    assert!(rendered.contains("Hold[stackFrame[]]"));
}

#[test]
fn testreport_merges_existing_reports_and_accepts_report_level_options() {
    let mut session = Session::new();

    let merged = session
        .eval_text(
            "TestReport[{TestReport[{TestCreate[1 + 1, 2]}], TestReport[{TestCreate[2 + 2, 5]}]}]",
        )
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![merged.value.clone(), Expr::string("TestsRunCount")],
            ))
            .unwrap()
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![merged.value, Expr::string("TestsSucceededCount")],
            ))
            .unwrap()
            .to_string(),
        "1"
    );

    let same_test = session
        .eval_text("TestReport[{TestCreate[1 + 1, 1]}, {SameTest -> Greater}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![same_test.value, Expr::string("TestsSucceededCount")],
            ))
            .unwrap()
            .to_string(),
        "1"
    );

    let projected = session
        .eval_text(
            "TestReport[{TestCreate[1 + 1, 1]}, \"TestsSucceededCount\", {SameTest -> Greater}]",
        )
        .unwrap();
    assert_eq!(projected.value.to_string(), "1");

    let time_limited = session
        .eval_text(
            "Part[Lookup[TestReport[{TestCreate[Pause[1/50], 1]}, {TimeConstraint -> 1/100}], \"TestResults\"], 1]",
        )
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![time_limited.value.clone(), Expr::string("FailureType")],
            ))
            .unwrap()
            .to_string(),
        "\"TimeConstraint\""
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![time_limited.value, Expr::string("Outcome")],
            ))
            .unwrap()
            .to_string(),
        "\"Failure\""
    );

    let memory_limited = session
        .eval_text(
            "Part[Lookup[TestReport[{TestCreate[Range[100], Range[100]]}, {MemoryConstraint -> 20}], \"TestResults\"], 1]",
        )
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![memory_limited.value.clone(), Expr::string("FailureType")],
            ))
            .unwrap()
            .to_string(),
        "\"MemoryConstraint\""
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![memory_limited.value, Expr::string("Outcome")],
            ))
            .unwrap()
            .to_string(),
        "\"Failure\""
    );
}
