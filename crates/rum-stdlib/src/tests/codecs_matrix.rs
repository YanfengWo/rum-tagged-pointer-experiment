use crate::install_core;
use rum_core::{Atom, Expr};
use rum_runtime::Session;

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

// A value representable in JSON, TOML, and YAML alike: String keys, nested
// Association, list, exact Integer, terminating Rational, booleans, and an
// adversarial "no" string. (Null is excluded — TOML has no null.)
const TOUR_VALUE: &str = r#"<|"name" -> "web", "port" -> 8080, "ratio" -> 1/4, "enabled" -> True, "tags" -> {"a", "no"}, "db" -> <|"host" -> "h"|>|>"#;

#[test]
fn json_toml_json_round_trip_preserves_the_shared_value() {
    let mut session = core_session();
    session
        .eval_text(&format!("shared = {TOUR_VALUE}"))
        .unwrap();
    assert_eq!(
        eval_to_string(&mut session, "FromToml[ToToml[shared]] === shared"),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromJson[ToJson[FromToml[ToToml[shared]]]] === shared"
        ),
        "True"
    );
}

#[test]
fn json_yaml_json_round_trip_preserves_the_shared_value() {
    let mut session = core_session();
    session
        .eval_text(&format!("shared = {TOUR_VALUE}"))
        .unwrap();
    assert_eq!(
        eval_to_string(&mut session, "FromYaml[ToYaml[shared]] === shared"),
        "True"
    );
}

#[test]
fn grand_tour_json_toml_yaml_json_is_identity() {
    let mut session = core_session();
    session
        .eval_text(&format!("shared = {TOUR_VALUE}"))
        .unwrap();
    let tour =
        "ToJson[FromYaml[ToYaml[FromToml[ToToml[FromJson[ToJson[shared]]]]]]] === ToJson[shared]";
    assert_eq!(eval_to_string(&mut session, tour), "True");
}

#[test]
fn csv_headers_flow_into_json_and_back_to_csv() {
    let mut session = core_session();
    let json = eval_string_atom(
        &mut session,
        r#"ToJson[FromCsv["name,qty\napple,3\nkiwi,10", "Headers" -> True]]"#,
    );
    assert_eq!(
        json,
        r#"[{"name":"apple","qty":"3"},{"name":"kiwi","qty":"10"}]"#
    );
    let csv = eval_string_atom(
        &mut session,
        r#"ToCsv[FromJson["[{\"name\":\"apple\",\"qty\":\"3\"},{\"name\":\"kiwi\",\"qty\":\"10\"}]"], "Headers" -> True]"#,
    );
    assert_eq!(csv, "name,qty\napple,3\nkiwi,10");
}

#[test]
fn ini_flows_into_json_and_back_as_strings() {
    let mut session = core_session();
    let json = eval_string_atom(
        &mut session,
        r#"ToJson[FromIni["top = 1\n[db]\nhost = h\n"]]"#,
    );
    assert_eq!(json, r#"{"":{"top":"1"},"db":{"host":"h"}}"#);
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"ToIni[FromJson["{\"\":{\"top\":\"1\"},\"db\":{\"host\":\"h\"}}"]]"#
        ),
        r#""top = 1\n[db]\nhost = h""#
    );
}

#[test]
fn xml_values_extract_into_the_shared_model() {
    let mut session = core_session();
    let json = eval_string_atom(
        &mut session,
        r#"ToJson[<|"env" -> FromXml["<cfg env=\"dev\"><p>8080</p></cfg>"][[2]]["env"]|>]"#,
    );
    assert_eq!(json, r#"{"env":"dev"}"#);
}
