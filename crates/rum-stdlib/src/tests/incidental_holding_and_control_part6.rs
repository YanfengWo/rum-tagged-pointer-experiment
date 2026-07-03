use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn failsafe_wrapper_preserves_failure_like_inputs_and_supports_test_forms() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Failsafe[Plus][1, $Failed, 3]")
            .unwrap()
            .value
            .to_string(),
        "$Failed"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[Function[Plus[Slot[1], Slot[1]]], Positive][4]")
            .unwrap()
            .value
            .to_string(),
        "8"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Failsafe[Function[Plus[Slot[1], Slot[1]]], Positive][-1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, Positive][-1][All]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"MessageTemplate\", \"The test failed on the arguments `1`.\"], RuleDelayed[\"MessageParameters\", List[HoldForm[-1]]], Rule[\"Arguments\", Hold[-1]]]"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, Positive][-1][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"The test failed on the arguments -1.\""
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, Function[False]][a, b][\"MessageParameters\"]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldForm[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, Function[False]][a, b][\"Arguments\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, Positive][-1][\"Function\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"Function\"]"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Failsafe[f, Function[p]][1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Failsafe[f, Function[1]][1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[Divide, Function[UnsameQ[Slot[2], 0]], Function[Missing[\"ZeroDiv\"]]][1, 0]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"ZeroDiv\"]"
    );
    assert_eq!(
        session.eval_text("Failsafe[]").unwrap().value.to_string(),
        "Failsafe[]"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, t, g, h][x]")
            .unwrap()
            .value
            .to_string(),
        "Failsafe[f, t, g, h][x]"
    );
}
