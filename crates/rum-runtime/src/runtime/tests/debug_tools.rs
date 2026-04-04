use super::*;

#[test]
fn trace_returns_a_first_pass_flat_expression_list_without_leaking_session_trace_state() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();

    let traced = session.eval_text("Trace[f[1]]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Hold[f[1]], Hold[Plus[1, 1]], Hold[2], Hold[f[1]], Hold[2]]"
    );
    assert!(session.take_trace().is_empty());
}

#[test]
fn trace_print_emits_trace_lines_and_returns_the_underlying_value() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();

    let traced = session.eval_text("TracePrint[f[1]]").unwrap();
    assert_eq!(traced.value.to_string(), "2");
    assert_eq!(traced.prints, vec!["f[1]", "Plus[1, 1]", "2", "f[1]", "2"]);
    assert!(session.take_trace().is_empty());
}

#[test]
fn timing_and_absolute_timing_return_elapsed_and_value() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[Timing[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[Timing[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("Part[AbsoluteTiming[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[AbsoluteTiming[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn echo_family_prints_and_returns_values_in_the_current_first_pass_subset() {
    let mut session = Session::new();

    let echoed = session.eval_text("Echo[1 + 1]").unwrap();
    assert_eq!(echoed.value.to_string(), "2");
    assert_eq!(echoed.prints, vec!["2"]);

    let labeled = session.eval_text("Echo[1 + 1, \"sum\"]").unwrap();
    assert_eq!(labeled.value.to_string(), "2");
    assert_eq!(labeled.prints, vec!["sum: 2"]);

    let via_function = session.eval_text("EchoFunction[][1 + 1]").unwrap();
    assert_eq!(via_function.value.to_string(), "2");
    assert_eq!(via_function.prints, vec!["2"]);

    let via_label = session.eval_text("EchoLabel[\"sum\"][1 + 1]").unwrap();
    assert_eq!(via_label.value.to_string(), "2");
    assert_eq!(via_label.prints, vec!["sum: 2"]);
}

#[test]
fn byte_count_uses_the_current_canonical_text_proxy_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ByteCount[\"abc\"]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("ByteCount[1 + 1]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
}
