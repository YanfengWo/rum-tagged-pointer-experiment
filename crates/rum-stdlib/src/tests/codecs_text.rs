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

fn assert_failed_with_message(
    value: Expr,
    messages: Vec<RuntimeMessage>,
    symbol: &str,
    tag: &str,
    needle: &str,
) {
    assert_eq!(value.to_string(), "$Failed");
    assert!(
        messages.iter().any(|message| {
            message.symbol == symbol && message.tag == tag && message.text.contains(needle)
        }),
        "missing {symbol}::{tag} containing {needle:?}; messages: {messages:?}"
    );
}

#[test]
fn from_csv_parses_core_rows_quoting_newlines_and_unicode() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "FromCsv[\"a,\\\"b,1\\\"\\nc,d\"]"),
        "List[List[\"a\", \"b,1\"], List[\"c\", \"d\"]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromCsv[\"\\\"a\\\"\\\"b\\\",\\\"line\\nwrap\\\"\\r\\nα,世界\\n\"]"
        ),
        "List[List[\"a\\\"b\", \"line\\nwrap\"], List[\"α\", \"世界\"]]"
    );
    assert_eq!(eval_to_string(&mut session, "FromCsv[\"\"]"), "List[]");
    assert_eq!(
        eval_to_string(&mut session, "FromCsv[\"a,,b\\n\\nc,d\"]"),
        "List[List[\"a\", \"\", \"b\"], List[], List[\"c\", \"d\"]]"
    );
}

#[test]
fn csv_headers_tsv_ragged_rows_and_round_trips_are_pinned() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "FromCsv[\"name,qty\\napple,3\\nkiwi,10\", \"Headers\" -> True]"
        ),
        "List[Association[Rule[\"name\", \"apple\"], Rule[\"qty\", \"3\"]], Association[Rule[\"name\", \"kiwi\"], Rule[\"qty\", \"10\"]]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "FromCsv[\"a,b\\n1\"]"),
        "List[List[\"a\", \"b\"], List[\"1\"]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromCsv[\"a\\tb\\n1\\t2\", \"Delimiter\" -> \"\\t\"]"
        ),
        "List[List[\"a\", \"b\"], List[\"1\", \"2\"]]"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToCsv[{{\"a\", \"b\"}, {\"1\", \"2\"}}, \"Delimiter\" -> \"\\t\"]"
        ),
        "a\tb\n1\t2"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromCsv[ToCsv[{{\"a,b\", \"q\\\"r\"}, {\"line\\nx\", \"z\"}}]]"
        ),
        "List[List[\"a,b\", \"q\\\"r\"], List[\"line\\nx\", \"z\"]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromCsv[ToCsv[{<|\"name\" -> \"apple\", \"qty\" -> \"3\"|>, <|\"qty\" -> \"10\", \"name\" -> \"kiwi\"|>}], \"Headers\" -> True]"
        ),
        "List[Association[Rule[\"name\", \"apple\"], Rule[\"qty\", \"3\"]], Association[Rule[\"name\", \"kiwi\"], Rule[\"qty\", \"10\"]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "headers = Map[ToString, Range[20]]; Length[FromCsv[ToCsv[{headers, headers}], \"Headers\" -> True]]",
        ),
        "1"
    );
}

#[test]
fn to_csv_encodes_lists_and_association_rows() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(&mut session, "ToCsv[{{\"x\", \"y\\\"z\"}, {\"1\", \"2\"}}]"),
        "x,\"y\"\"z\"\n1,2"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToCsv[{{\"a,b\", \"line\\nwrap\", \"plain\"}}]"
        ),
        "\"a,b\",\"line\nwrap\",plain"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToCsv[{<|\"name\" -> \"apple\", \"qty\" -> \"3\"|>, <|\"qty\" -> \"10\", \"name\" -> \"kiwi\"|>}]"
        ),
        "name,qty\napple,3\nkiwi,10"
    );
}

#[test]
fn to_csv_wide_association_rows_follow_header_order() {
    let mut session = core_session();
    let csv = eval_string_atom(
        &mut session,
        "headers = Map[ToString, Range[20]]; ToCsv[{AssociationThread[headers, headers], AssociationThread[Reverse[headers], Reverse[headers]]}]",
    );
    let lines = csv.lines().collect::<Vec<_>>();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[1], lines[2]);
}

#[test]
fn csv_codecs_fail_closed_with_messages() {
    let mut session = core_session();

    for (source, needle) in [
        ("FromCsv[\"a,\\\"unterminated\"]", "byte offset"),
        ("FromCsv[\"\\\"a\\\"x\"]", "closing quote"),
        (
            "FromCsv[\"a,a\\n1,2\", \"Headers\" -> True]",
            "duplicate header",
        ),
        (
            "FromCsv[\"a,b\\n1\", \"Headers\" -> True]",
            "header has 2 fields",
        ),
        (
            "FromCsv[\"a\", \"Delimiter\" -> \",,\"]",
            "single character",
        ),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_failed_with_message(outcome.value, outcome.messages, "FromCsv", "csv", needle);
    }

    for (source, needle) in [
        ("ToCsv[{{\"a\", 1}}]", "row 1, column 2"),
        ("ToCsv[{<|x -> \"1\"|>}]", "Non-String CSV header key"),
        (
            "ToCsv[{<|\"a\" -> \"1\"|>, <|\"b\" -> \"2\"|>}]",
            "missing CSV header key",
        ),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_failed_with_message(outcome.value, outcome.messages, "ToCsv", "csv", needle);
    }
}

#[test]
fn from_ini_parses_documented_dialect() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "FromIni[\"; c\\ntop = 1\\n[db]\\nhost = h ; not a comment\\n\"]"
        ),
        "Association[Rule[\"\", Association[Rule[\"top\", \"1\"]]], Rule[\"db\", Association[Rule[\"host\", \"h ; not a comment\"]]]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "FromIni[\"# c\\n\\n[s]\\nk = \\\"v\\\"\\n\"]"),
        "Association[Rule[\"s\", Association[Rule[\"k\", \"\\\"v\\\"\"]]]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "FromIni[\"\"]"),
        "Association[]"
    );
}

#[test]
fn ini_to_ini_output_and_round_trips_are_pinned() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(&mut session, "ToIni[FromIni[\"[s]\\nk = v\\n\"]]"),
        "[s]\nk = v"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToIni[<|\"\" -> <|\"top\" -> \"1\"|>, \"db\" -> <|\"host\" -> \"h ; not a comment\"|>|>]"
        ),
        "top = 1\n[db]\nhost = h ; not a comment"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromIni[ToIni[<|\"\" -> <|\"top\" -> \"1\"|>, \"s\" -> <|\"k\" -> \"v\"|>|>]]"
        ),
        "Association[Rule[\"\", Association[Rule[\"top\", \"1\"]]], Rule[\"s\", Association[Rule[\"k\", \"v\"]]]]"
    );
}

#[test]
fn ini_wide_key_and_section_indexes_preserve_round_trips() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "names = Map[ToString, Range[20]]; data = Association[Rule[\"s\", AssociationThread[names, names]]]; SameQ[FromIni[ToIni[data]], data]",
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "names = Map[ToString, Range[20]]; data = AssociationThread[names, Map[Function[Association[Rule[\"k\", Slot[1]]]], names]]; SameQ[FromIni[ToIni[data]], data]",
        ),
        "True"
    );
}

#[test]
fn ini_codecs_fail_closed_with_messages() {
    let mut session = core_session();

    for (source, needle) in [
        ("FromIni[\"bad line\\n\"]", "byte offset"),
        ("FromIni[\"[dup]\\n[dup]\\n\"]", "duplicate section"),
        ("FromIni[\"[s]\\nk = 1\\nk = 2\\n\"]", "duplicate key"),
        ("FromIni[\" = v\\n\"]", "empty key"),
        ("FromIni[\"[unterminated\\n\"]", "unterminated"),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_failed_with_message(outcome.value, outcome.messages, "FromIni", "ini", needle);
    }

    for (source, needle) in [
        (
            "ToIni[<|\"s\" -> <|\"bad=key\" -> \"v\"|>|>]",
            "not representable",
        ),
        (
            "ToIni[<|\"s\" -> <|\"k\" -> \" v\"|>|>]",
            "not representable",
        ),
        ("ToIni[<|\"\" -> <||>|>]", "empty default section"),
        ("ToIni[<|\"s\" -> <|\"k\" -> 1|>|>]", "must be a String"),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_failed_with_message(outcome.value, outcome.messages, "ToIni", "ini", needle);
    }
}

#[test]
fn codecs_compose_with_json_and_editing_flows() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToJson[Map[Function[row, <|\"item\" -> Lookup[row, \"name\"], \"qty\" -> FromDigits[Lookup[row, \"qty\"]]|>], FromCsv[\"name,qty\\napple,3\\nkiwi,10\", \"Headers\" -> True]]]"
        ),
        "[{\"item\":\"apple\",\"qty\":3},{\"item\":\"kiwi\",\"qty\":10}]"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToIni[<|\"s\" -> <|\"k\" -> StringJoin[Lookup[Lookup[FromIni[\"[s]\\nk = v\\n\"], \"s\"], \"k\"], \"2\"]|>|>]"
        ),
        "[s]\nk = v2"
    );
}
