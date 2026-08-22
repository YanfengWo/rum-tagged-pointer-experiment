use crate::install_core;
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::system_expr;
use rum_runtime::{RuntimeMessage, Session};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn eval_string_atom(session: &mut Session, source: &str) -> String {
    match session.eval_text(source).unwrap().value {
        Expr::Atom(Atom::String(value)) => value,
        other => panic!("expected string atom, got {other}"),
    }
}

fn eval_expr_with_messages(session: &mut Session, expr: Expr) -> (Expr, Vec<RuntimeMessage>) {
    let value = session.evaluate(expr).unwrap();
    let messages = session.take_messages();
    (value, messages)
}

fn from_json_call(source: &str) -> Expr {
    Expr::call(system_expr("FromJson"), vec![Expr::string(source)])
}

fn from_json_value(session: &mut Session, source: &str) -> Expr {
    let (value, messages) = eval_expr_with_messages(session, from_json_call(source));
    assert!(messages.is_empty(), "{source}: {messages:?}");
    value
}

fn assert_failed_with_message(value: Expr, messages: Vec<RuntimeMessage>, symbol: &str, tag: &str) {
    assert_eq!(value.to_string(), "$Failed");
    assert!(
        messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == tag),
        "missing {symbol}::{tag}; messages: {messages:?}"
    );
}

#[test]
fn from_json_maps_nested_values_to_exact_rum_expressions() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "FromJson[\"{\\\"a\\\": [1, 2.5, true, null], \\\"b\\\": {\\\"c\\\": \\\"x\\\"}}\"]"
        ),
        "Association[Rule[\"a\", List[1, 5/2, True, Null]], Rule[\"b\", Association[Rule[\"c\", \"x\"]]]]"
    );
    assert_eq!(
        from_json_value(&mut session, "[0.1, 1.5e3, 1e-2, -0]").to_string(),
        "List[1/10, 1500, 1/100, 0]"
    );
    assert_eq!(
        from_json_value(&mut session, " \n\t[true, false, null]\r ").to_string(),
        "List[True, False, Null]"
    );

    let big = "1234567890".repeat(30);
    assert_eq!(from_json_value(&mut session, &big).to_string(), big);

    match from_json_value(&mut session, "\"\\uD83D\\uDE42 héllo\"") {
        Expr::Atom(Atom::String(value)) => assert_eq!(value, "🙂 héllo"),
        other => panic!("expected string atom, got {other}"),
    }
}

#[test]
fn from_json_wide_object_key_index_preserves_all_members() {
    let mut session = core_session();
    let members = (0..20)
        .map(|index| format!("\"k{index}\":{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let value = from_json_value(&mut session, &format!("{{{members}}}"));

    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Length"), vec![value]))
            .unwrap()
            .to_string(),
        "20"
    );
}

#[test]
fn to_json_encodes_supported_subset_compactly_and_pretty() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToJson[<|\"n\" -> 1/4, \"big\" -> 2^100, \"s\" -> \"héllo\", \"l\" -> {True, Null}|>]"
        ),
        "{\"n\":0.25,\"big\":1267650600228229401496703205376,\"s\":\"héllo\",\"l\":[true,null]}"
    );
    assert_eq!(
        eval_string_atom(&mut session, "ToJson[\"🙂\"]"),
        "\"\\uD83D\\uDE42\""
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToJson[<|\"a\" -> {1, True}, \"b\" -> <||>|>, \"Indent\" -> 2]"
        ),
        "{\n  \"a\": [\n    1,\n    true\n  ],\n  \"b\": {}\n}"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromJson[ToJson[<|\"k\" -> {1, 2/5, \"z\"}|>]]"
        ),
        "Association[Rule[\"k\", List[1, 2/5, \"z\"]]]"
    );
}

#[test]
fn to_json_rejects_unsupported_values_fail_closed() {
    let mut session = core_session();

    for source in [
        "ToJson[<|\"x\" -> 1/3|>]",
        "ToJson[<|a -> 1|>]",
        "ToJson[x]",
        "ToJson[f[1]]",
        "ToJson[<|\"x\" -> 1|>, \"Other\" -> 2]",
        "ToJson[<|\"x\" -> 1|>, \"Indent\" -> -1]",
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_failed_with_message(outcome.value, outcome.messages, "ToJson", "json");
    }
}

#[test]
fn from_json_rejects_invalid_json_fail_closed() {
    let mut session = core_session();

    let cases = [
        ("trailing", "1 2"),
        ("unquoted key", "{a: 1}"),
        ("single quotes", "{'a': 1}"),
        ("comment", "{\"a\": /* nope */ 1}"),
        ("nan", "NaN"),
        ("infinity", "Infinity"),
        ("control", "\"line\nbreak\""),
        ("escape", "\"\\x\""),
        ("lone high surrogate", "\"\\uD800\""),
        ("lone low surrogate", "\"\\uDC00\""),
        ("duplicate key", "{\"a\": 1, \"a\": 2}"),
        ("trailing comma", "{\"a\": 1,}"),
    ];

    for (name, source) in cases {
        let (value, messages) = eval_expr_with_messages(&mut session, from_json_call(source));
        assert_failed_with_message(value, messages, "FromJson", "json");
        assert!(session.take_messages().is_empty(), "{name}");
    }
}

#[test]
fn from_json_depth_cap_fails_before_native_stack_depth() {
    let mut session = core_session();
    let source = format!("{}0{}", "[".repeat(300), "]".repeat(300));

    let (value, messages) = eval_expr_with_messages(&mut session, from_json_call(&source));
    assert_failed_with_message(value, messages, "FromJson", "depth");
}

#[test]
fn json_codec_supports_transform_round_trip() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToJson[Mean[Map[Lookup[#, \"v\"] &, First[GatherBy[FromJson[\"[{\\\"g\\\":\\\"a\\\",\\\"v\\\":1},{\\\"g\\\":\\\"a\\\",\\\"v\\\":3},{\\\"g\\\":\\\"b\\\",\\\"v\\\":5}]\"], Lookup[#, \"g\"] &]]]]]"
        ),
        "2"
    );
}
