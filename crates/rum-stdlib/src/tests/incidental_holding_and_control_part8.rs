use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn generated_confirmation_failures_expose_current_oracle_metadata_slice() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("FailureQ[$Canceled]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f][$Canceled]")
            .unwrap()
            .value
            .to_string(),
        "$Canceled"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f][Missing[]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[]"
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[Missing[]], \"ConfirmationType\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Confirm\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[Failure[\"tag\", Association[\"Code\" -> 1]]], \"ConfirmationType\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Confirm\""
    );
    assert_eq!(
        session
            .eval_text(
                "Enclose[Confirm[Failure[\"tag\", Association[\"Code\" -> 1]]], \"Message\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "\"A failure of type \\\"tag\\\" occurred.\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[$Canceled], \"ConfirmationType\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Confirm\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[$Canceled], \"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"$Canceled encountered.\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmBy[x, IntegerQ], \"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"IntegerQ[x] did not return True.\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmAssert[False], \"HeldTest\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[False]"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmQuiet[First[f[]]], \"HeldMessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[First::nofirst]"
    );
}
