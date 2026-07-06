use crate::install_core;
use rum_core::{Atom, Expr};
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

fn assert_failed_with_schema_message(
    value: Expr,
    messages: Vec<RuntimeMessage>,
    symbol: &str,
    needle: &str,
) {
    assert_eq!(value.to_string(), "$Failed");
    assert!(
        messages.iter().any(|message| {
            message.symbol == symbol && message.tag == "schema" && message.text.contains(needle)
        }),
        "missing {symbol}::schema containing {needle:?}; messages: {messages:?}"
    );
}

const CONFIG_SCHEMA: &str = r#"<|
    "name" -> _String,
    "db" -> <|"host" -> _String, "port" -> p_Integer /; p > 0, "ssl" -> Optional[True | False, False]|>,
    "servers" -> {<|"host" -> _String, "port" -> p_Integer /; p > 0|>},
    "debug" -> Optional[True | False, False]
|>"#;

#[test]
fn conform_schema_validates_nested_config_and_fills_defaults() {
    let mut session = core_session();
    let value = r#"<|
        "name" -> "web",
        "db" -> <|"host" -> "h", "port" -> 5432|>,
        "servers" -> {<|"host" -> "a", "port" -> 80|>, <|"host" -> "b", "port" -> 443|>}
    |>"#;
    let expected = r#"<|
        "name" -> "web",
        "db" -> <|"host" -> "h", "port" -> 5432, "ssl" -> False|>,
        "servers" -> {<|"host" -> "a", "port" -> 80|>, <|"host" -> "b", "port" -> 443|>},
        "debug" -> False
    |>"#;

    assert_eq!(
        eval_to_string(
            &mut session,
            &format!("SameQ[ConformSchema[{value}, {CONFIG_SCHEMA}], {expected}]")
        ),
        "True"
    );
}

#[test]
fn schema_errors_returns_exact_nested_invalid_config_errors() {
    let mut session = core_session();
    let value = r#"<|
        "name" -> 1,
        "db" -> <|"host" -> "h", "port" -> "x", "extra" -> 1|>,
        "servers" -> {<|"host" -> "a", "port" -> 80|>, <|"host" -> 2, "port" -> -1|>}
    |>"#;
    let expected = r#"{
        <|"Path" -> "name", "Message" -> "expected to match _String, got 1"|>,
        <|"Path" -> "db.port", "Message" -> "expected to match p_Integer /; Greater[p, 0], got \"x\""|>,
        <|"Path" -> "db.extra", "Message" -> "extra key is not allowed"|>,
        <|"Path" -> "servers[2].host", "Message" -> "expected to match _String, got 2"|>,
        <|"Path" -> "servers[2].port", "Message" -> "expected to match p_Integer /; Greater[p, 0], got -1"|>
    }"#;

    assert_eq!(
        eval_to_string(
            &mut session,
            &format!("SameQ[SchemaErrors[{value}, {CONFIG_SCHEMA}], {expected}]")
        ),
        "True"
    );
}

#[test]
fn pattern_leaf_breadth_and_held_schema_semantics() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[
                {
                    SchemaErrors[7, _Integer],
                    SchemaErrors["s", _String],
                    SchemaErrors["a", "a" | "b"],
                    SchemaErrors["abc", _String?StringQ],
                    SchemaErrors[3, x_Integer /; x > 0],
                    SchemaErrors["literal", "literal"]
                },
                {{}, {}, {}, {}, {}, {}}
            ]"#
        ),
        "True"
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"flag = 0; SchemaErrors["x", x_Integer /; (flag = 1; True)]; flag"#
        ),
        "0"
    );
}

#[test]
fn optional_defaults_and_extra_keys_follow_schema_contract() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[ConformSchema[<|"a" -> 1|>, <|"a" -> _Integer, "b" -> Optional[_String]|>], <|"a" -> 1|>]"#
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[ConformSchema[<|"a" -> 1|>, <|"a" -> _Integer, "b" -> Optional[_String, "x"]|>], <|"a" -> 1, "b" -> "x"|>]"#
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[SchemaErrors[<||>, <|"a" -> Optional[_Integer, "x"]|>], {<|"Path" -> "a", "Message" -> "expected to match _Integer, got \"x\""|>}]"#
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[SchemaErrors[<|"a" -> 1, "extra" -> 2|>, <|"a" -> _Integer|>], {<|"Path" -> "extra", "Message" -> "extra key is not allowed"|>}]"#
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[ConformSchema[<|"a" -> 1, "extra" -> 2|>, <|"a" -> _Integer|>, "AllowExtraKeys" -> True], <|"a" -> 1, "extra" -> 2|>]"#
        ),
        "True"
    );
}

#[test]
fn list_schema_covers_empty_and_nested_associations() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, r#"SameQ[SchemaErrors[{}, {_Integer}], {}]"#),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[
                SchemaErrors[{<|"host" -> "a"|>, <|"host" -> 2|>}, {<|"host" -> _String|>}],
                {<|"Path" -> "[2].host", "Message" -> "expected to match _String, got 2"|>}
            ]"#
        ),
        "True"
    );
}

#[test]
fn schema_errors_are_capped_with_summary() {
    let mut session = core_session();
    let value_entries = (1..=70)
        .map(|index| format!("\"k{index}\" -> {index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let schema_entries = (1..=70)
        .map(|index| format!("\"k{index}\" -> _String"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"errors = SchemaErrors[<|{value_entries}|>, <|{schema_entries}|>];
           SameQ[Length[errors], 65] && SameQ[Last[errors], <|"Path" -> "", "Message" -> "… and 6 more errors"|>]"#
    );

    assert_eq!(eval_to_string(&mut session, &source), "True");
}

#[test]
fn schema_validation_composes_with_codecs_and_error_data_serializes() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[
                ConformSchema[FromYaml["db:\n  host: h\n  port: 5432\n"], <|"db" -> <|"host" -> _String, "port" -> _Integer|>|>],
                <|"db" -> <|"host" -> "h", "port" -> 5432|>|>
            ]"#
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"SameQ[
                ConformSchema[FromJson["{\"db\":{\"host\":\"h\",\"port\":5432}}"], <|"db" -> <|"host" -> _String, "port" -> _Integer|>|>],
                <|"db" -> <|"host" -> "h", "port" -> 5432|>|>
            ]"#
        ),
        "True"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            r#"ToJson[SchemaErrors[<|"port" -> "x"|>, <|"port" -> _Integer|>]]"#
        ),
        r#"[{"Path":"port","Message":"expected to match _Integer, got \"x\""}]"#
    );
}

#[test]
fn conform_schema_emits_schema_messages_and_fails_closed() {
    let mut session = core_session();

    let outcome = session
        .eval_text(r#"ConformSchema[<|"port" -> "x"|>, <|"port" -> _Integer|>]"#)
        .unwrap();
    assert_failed_with_schema_message(
        outcome.value,
        outcome.messages,
        "ConformSchema",
        "port: expected to match _Integer, got \"x\"",
    );

    let outcome = session
        .eval_text(r#"ConformSchema[<|"a" -> 1|>, <|"a" -> {_Integer, _Integer}|>]"#)
        .unwrap();
    assert_failed_with_schema_message(
        outcome.value,
        outcome.messages,
        "ConformSchema",
        "Schema path a: list schema must contain exactly one element; got 2",
    );

    let outcome = session
        .eval_text(r#"SchemaErrors[<|"a" -> 1|>, <|a -> _Integer|>]"#)
        .unwrap();
    assert_failed_with_schema_message(
        outcome.value,
        outcome.messages,
        "SchemaErrors",
        "Schema path <root>: association schema keys must be Strings; got a",
    );
}
