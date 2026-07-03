use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn cycle_detection_allows_same_call_after_state_progress() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("n = 2; f[] := If[n > 0, n--; f[], 7]; f[]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("n = 2; g[x_] := If[n > 0, n--; g[x], x]; g[7]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}
