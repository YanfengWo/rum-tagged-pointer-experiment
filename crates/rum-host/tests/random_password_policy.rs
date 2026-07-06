use rum_host::{HostConfig, HostEvalOutcome, HostEvalSuccess, HostSession};

fn successful_eval(outcome: HostEvalOutcome) -> HostEvalSuccess {
    match outcome {
        HostEvalOutcome::Success(success) => success,
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn sandboxed_host_config_blocks_random_password_policy_options() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();
    let success = successful_eval(
        session.eval("RandomPassword[8, \"Require\" -> {\"A\"}, \"NoAdjacentRepeats\" -> True]"),
    );

    assert_eq!(success.value.text, "$Failed");
    assert!(
        success
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox"),
        "messages={:?}",
        success.messages
    );
}

#[test]
fn missing_entropy_provider_fails_random_password_policy_options() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().without_entropy_source())
        .build();
    let success = successful_eval(
        session.eval("RandomPassword[8, \"Require\" -> {\"A\"}, \"NoAdjacentRepeats\" -> True]"),
    );

    assert_eq!(success.value.text, "$Failed");
    assert_eq!(success.messages.len(), 1);
    assert_eq!(success.messages[0].symbol, "General");
    assert_eq!(success.messages[0].tag, "noentropy");
    assert_eq!(success.messages[0].text, "no entropy provider installed");
}
