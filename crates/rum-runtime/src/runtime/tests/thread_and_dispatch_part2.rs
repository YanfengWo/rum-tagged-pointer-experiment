use super::*;

#[test]
fn thread_builtin_mismatch_emits_message_and_stays_unevaluated() {
    let mut session = Session::new();

    let outcome = session.eval_text("Thread[f[{a, b}, {c}]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Thread[f[List[a, b], List[c]]]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "Thread" && message.tag == "tdlen"));

    let custom = session.eval_text("Thread[f[g[a, b], g[c]], g]").unwrap();
    assert_eq!(custom.value.to_string(), "Thread[f[g[a, b], g[c]], g]");
    assert!(custom
        .messages
        .iter()
        .any(|message| message.symbol == "Thread" && message.tag == "tdlen"));

    assert_eq!(
        session
            .eval_text("Check[Thread[f[{a, b}, {c}]], fallback, Thread]")
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );

    let invalid_spec = session.eval_text("Thread[f[{a, b}], List, bad]").unwrap();
    assert_eq!(
        invalid_spec.value.to_string(),
        "Thread[f[List[a, b]], List, bad]"
    );
    assert!(invalid_spec
        .messages
        .iter()
        .any(|message| message.symbol == "Thread" && message.tag == "argform"));
}
