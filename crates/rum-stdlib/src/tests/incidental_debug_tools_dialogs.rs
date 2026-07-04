use super::*;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn dialog_catches_return_and_respects_current_option_subset() {
    let mut session = core_session();
    session.eval_text("x = global").unwrap();

    assert_eq!(
        session
            .eval_text("1 + Dialog[Return[2]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );

    let localized = session
        .eval_text("Dialog[x, DialogSymbols -> {x = local}, DialogProlog :> (x = start)]")
        .unwrap();
    assert_eq!(localized.value.to_string(), "start");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "global");
}

#[test]
fn dialog_without_body_uses_a_current_single_input_line_subset() {
    let mut session = core_session();
    session.queue_input_response("Return[2 + 3]");

    let response = session.eval_text("Dialog[]").unwrap();
    assert_eq!(response.value.to_string(), "5");
}

#[test]
fn dialog_uses_current_default_options_from_setoptions() {
    let mut session = core_session();
    session
        .eval_text("SetOptions[Dialog, DialogProlog :> Print[\"hi\"]]")
        .unwrap();
    session.queue_input_response("Return[7]");

    let response = session.eval_text("Dialog[]").unwrap();
    assert_eq!(response.value.to_string(), "7");
    assert_eq!(response.prints, vec!["hi"]);
}
