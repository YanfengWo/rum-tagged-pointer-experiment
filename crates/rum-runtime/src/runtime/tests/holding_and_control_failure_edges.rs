use super::*;

#[test]
fn failure_symbolic_property_and_predicate_edges_cover_current_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Code -> 500, Body -> 7}][Code]")
            .unwrap()
            .value
            .to_string(),
        "500"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Message -> 1, StyledMessage -> 2}][Message]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"MessageName\" -> 1}][\"MessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"HeldMessageTemplate\" -> 2}][\"HeldMessageTemplate\"]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "Failure[\"oops\", {\"Code\" -> 500, \"Body\" -> 7}][{\"Body\", \"Missing\"}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"NotAvailable\", List[\"Body\", \"Missing\"]]]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Code -> 500, Body -> 7}][{Body, Missing}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"NotAvailable\", List[Body, Missing]]]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Message\" -> 1, \"StyledMessage\" -> 2}][{\"Message\", \"StyledMessage\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][Properties]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", Properties]"
    );
    assert_eq!(
        session
            .eval_text("Failure[1, {\"Code\" -> 500}][All]")
            .unwrap()
            .value
            .to_string(),
        "Failure[1, Association[Rule[\"Code\", 500]]][All]"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[$Aborted]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[$Failed]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    let malformed_failure_q = session.eval_text("FailureQ[Failure[\"oops\"]]").unwrap();
    assert_eq!(malformed_failure_q.value.to_string(), "True");
    assert!(malformed_failure_q.messages.is_empty());
    assert_eq!(
        session
            .eval_text("FailureQ[Failure[\"oops\", bad]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Failure[1, {\"Code\" -> 500}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Failure[\"oops\", {\"Code\" -> 500}], _Failure]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[$Failed, _Failure]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[\"RecursionLimit\"]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[]"
    );
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[\"RecursionLimit\", \"extra\"]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\", \"extra\"]"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[TerminatedEvaluation[\"RecursionLimit\"]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[TerminatedEvaluation[\"IterationLimit\"]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[\"RecursionLimit\"][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"][\"Properties\"]"
    );

    assert_eq!(Session::new().resource_limits().recursion_limit, 1024);

    let mut recursion_session = Session::new();
    recursion_session.set_recursion_limit(32);
    recursion_session
        .eval_text("grow[x_] := grow[f[x]]")
        .unwrap();
    let recursion = recursion_session.eval_text("grow[1]").unwrap();
    assert_eq!(
        recursion.value.to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert!(
        recursion
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reclim")
    );

    let mut iteration_session = Session::new();
    iteration_session.set_evaluation_pass_limit(1);
    let iteration = iteration_session
        .eval_text("f[f = Hold, Identity[1]]")
        .unwrap();
    assert_eq!(
        iteration.value.to_string(),
        "TerminatedEvaluation[\"IterationLimit\"]"
    );
    assert!(
        iteration
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "passlim")
    );

    let mut request_session = Session::new();
    request_session.set_recursion_limit(32);
    request_session
        .eval_request("grow[x_] := grow[f[x]]")
        .unwrap();
    let request = request_session.eval_request("grow[1]").unwrap();
    assert_eq!(
        request.value.to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert!(
        request
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reclim")
    );

    let mut iteration_request_session = Session::new();
    iteration_request_session.set_evaluation_pass_limit(1);
    let iteration_request = iteration_request_session
        .eval_request("f[f = Hold, Identity[1]]")
        .unwrap();
    assert_eq!(
        iteration_request.value.to_string(),
        "TerminatedEvaluation[\"IterationLimit\"]"
    );
    assert!(
        iteration_request
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "passlim")
    );

    let mut data_session = Session::new();
    data_session.set_recursion_limit(32);
    data_session.eval_text("grow[x_] := grow[f[x]]").unwrap();
    let data = data_session.eval_text("EvaluationData[grow[1]]").unwrap();
    assert_eq!(
        data_session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![data.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert_eq!(
        data_session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![data.value, Expr::string("FailureType")],
            ))
            .unwrap()
            .to_string(),
        "\"RecursionLimit\""
    );
}
