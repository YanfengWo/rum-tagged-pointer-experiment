use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn join_across_inner_join_is_left_major() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"JoinAcross[{<|"id"->1,"a"->"x"|>,<|"id"->2,"a"->"y"|>}, {<|"id"->1,"b"->10|>,<|"id"->3,"b"->30|>}, "id"]"#,
        ),
        "List[Association[Rule[\"id\", 1], Rule[\"a\", \"x\"], Rule[\"b\", 10]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"JoinAcross[{<|"id"->1,"a"->"x"|>}, {<|"id"->1,"b"->10|>}, "id", "Inner"]"#,
        ),
        "List[Association[Rule[\"id\", 1], Rule[\"a\", \"x\"], Rule[\"b\", 10]]]"
    );
}

#[test]
fn join_across_left_right_and_outer_preserve_walk_order_and_bare_rows() {
    let mut session = core_session();
    let left = r#"{<|"id"->1,"l"->"L1"|>,<|"id"->2,"l"->"L2"|>}"#;
    let right = r#"{<|"id"->1,"r"->"R1"|>,<|"id"->3,"r"->"R3"|>}"#;

    assert_eq!(
        eval_to_string(
            &mut session,
            &format!(r#"JoinAcross[{left}, {right}, "id", "Left"]"#),
        ),
        "List[Association[Rule[\"id\", 1], Rule[\"l\", \"L1\"], Rule[\"r\", \"R1\"]], Association[Rule[\"id\", 2], Rule[\"l\", \"L2\"]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            &format!(r#"JoinAcross[{left}, {right}, "id", "Right"]"#),
        ),
        "List[Association[Rule[\"id\", 1], Rule[\"l\", \"L1\"], Rule[\"r\", \"R1\"]], Association[Rule[\"id\", 3], Rule[\"r\", \"R3\"]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            &format!(r#"JoinAcross[{left}, {right}, "id", "Outer"]"#),
        ),
        "List[Association[Rule[\"id\", 1], Rule[\"l\", \"L1\"], Rule[\"r\", \"R1\"]], Association[Rule[\"id\", 2], Rule[\"l\", \"L2\"]], Association[Rule[\"id\", 3], Rule[\"r\", \"R3\"]]]"
    );
}

#[test]
fn join_across_supports_multi_key_and_expression_key_specs() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"JoinAcross[{<|"id"->1,"group"->"a","l"->10|>,<|"id"->1,"group"->"b","l"->20|>}, {<|"id"->1,"group"->"b","r"->30|>}, {"id", Key["group"]}]"#,
        ),
        "List[Association[Rule[\"id\", 1], Rule[\"group\", \"b\"], Rule[\"l\", 20], Rule[\"r\", 30]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"JoinAcross[{<|f[1]->1,"l"->a|>}, {<|f[1]->1,"r"->b|>}, Key[f[1]]]"#,
        ),
        "List[Association[Rule[f[1], 1], Rule[\"l\", a], Rule[\"r\", b]]]"
    );
}

#[test]
fn join_across_rule_renamed_keys_keep_both_columns() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"JoinAcross[{<|"u"->1,"a"->"x"|>}, {<|"v"->1,"b"->10|>}, "u"->"v"]"#,
        ),
        "List[Association[Rule[\"u\", 1], Rule[\"a\", \"x\"], Rule[\"v\", 1], Rule[\"b\", 10]]]"
    );
}

#[test]
fn join_across_duplicate_values_form_an_ordered_cross_product() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"JoinAcross[{<|"k"->1,"l"->"L1"|>,<|"k"->1,"l"->"L2"|>}, {<|"k"->1,"r"->"R1"|>,<|"k"->1,"r"->"R2"|>}, "k"]"#,
        ),
        "List[Association[Rule[\"k\", 1], Rule[\"l\", \"L1\"], Rule[\"r\", \"R1\"]], Association[Rule[\"k\", 1], Rule[\"l\", \"L1\"], Rule[\"r\", \"R2\"]], Association[Rule[\"k\", 1], Rule[\"l\", \"L2\"], Rule[\"r\", \"R1\"]], Association[Rule[\"k\", 1], Rule[\"l\", \"L2\"], Rule[\"r\", \"R2\"]]]"
    );
}

#[test]
fn join_across_uses_right_row_values_on_column_collisions() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"JoinAcross[{<|"k"->1,"c"->"L"|>}, {<|"k"->1,"c"->"R"|>}, "k"]"#,
        ),
        "List[Association[Rule[\"k\", 1], Rule[\"c\", \"R\"]]]"
    );
}

#[test]
fn join_across_missing_keys_are_unmatched_without_padding() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"JoinAcross[{<|"k"->1,"l"->"match"|>,<|"l"->"left missing"|>}, {<|"k"->1,"r"->"match"|>,<|"r"->"right missing"|>}, "k", "Outer"]"#,
        ),
        "List[Association[Rule[\"k\", 1], Rule[\"l\", \"match\"], Rule[\"r\", \"match\"]], Association[Rule[\"l\", \"left missing\"]], Association[Rule[\"r\", \"right missing\"]]]"
    );
}

#[test]
fn join_across_accepts_empty_sides_for_every_join_type() {
    let mut session = core_session();
    let row = r#"<|"k"->1,"v"->2|>"#;

    let cases = [
        (format!(r#"JoinAcross[{{}}, {{{row}}}, "k"]"#), "List[]"),
        (format!(r#"JoinAcross[{{{row}}}, {{}}, "k"]"#), "List[]"),
        (
            format!(r#"JoinAcross[{{}}, {{{row}}}, "k", "Left"]"#),
            "List[]",
        ),
        (
            format!(r#"JoinAcross[{{{row}}}, {{}}, "k", "Left"]"#),
            "List[Association[Rule[\"k\", 1], Rule[\"v\", 2]]]",
        ),
        (
            format!(r#"JoinAcross[{{}}, {{{row}}}, "k", "Right"]"#),
            "List[Association[Rule[\"k\", 1], Rule[\"v\", 2]]]",
        ),
        (
            format!(r#"JoinAcross[{{{row}}}, {{}}, "k", "Right"]"#),
            "List[]",
        ),
        (
            format!(r#"JoinAcross[{{}}, {{{row}}}, "k", "Outer"]"#),
            "List[Association[Rule[\"k\", 1], Rule[\"v\", 2]]]",
        ),
        (
            format!(r#"JoinAcross[{{{row}}}, {{}}, "k", "Outer"]"#),
            "List[Association[Rule[\"k\", 1], Rule[\"v\", 2]]]",
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, &source), expected, "{source}");
    }
}

fn assert_join_across_error(
    session: &mut Session,
    source: &str,
    expected_value: &str,
    expected_tag: &str,
    expected_text: &str,
) {
    let outcome = session.eval_text(source).unwrap();
    assert_eq!(outcome.value.to_string(), expected_value);
    assert_eq!(outcome.messages.len(), 1);
    assert_eq!(outcome.messages[0].symbol, "JoinAcross");
    assert_eq!(outcome.messages[0].tag, expected_tag);
    assert_eq!(outcome.messages[0].text, expected_text);
}

#[test]
fn join_across_failures_emit_one_declared_message_and_stay_inert() {
    let mut session = core_session();

    assert_join_across_error(
        &mut session,
        r#"JoinAcross[{}, {}]"#,
        "JoinAcross[List[], List[]]",
        "argb",
        "JoinAcross called with 2 arguments; 3 or 4 arguments are expected.",
    );
    assert_join_across_error(
        &mut session,
        r#"JoinAcross[42, {}, "id"]"#,
        "JoinAcross[42, List[], \"id\"]",
        "lists",
        "The argument at position 1 in JoinAcross[42, List[], \"id\"] is not a valid list of Associations.",
    );
    assert_join_across_error(
        &mut session,
        r#"JoinAcross[{}, 42, "id"]"#,
        "JoinAcross[List[], 42, \"id\"]",
        "lists",
        "The argument at position 2 in JoinAcross[List[], 42, \"id\"] is not a valid list of Associations.",
    );
    assert_join_across_error(
        &mut session,
        r#"JoinAcross[{}, {}, bad]"#,
        "JoinAcross[List[], List[], bad]",
        "keys",
        "The argument bad is not a valid key specification: a key, a rule of keys, or a non-empty list of these.",
    );
    assert_join_across_error(
        &mut session,
        r#"JoinAcross[{}, {}, {}]"#,
        "JoinAcross[List[], List[], List[]]",
        "keys",
        "The argument List[] is not a valid key specification: a key, a rule of keys, or a non-empty list of these.",
    );
    assert_join_across_error(
        &mut session,
        r#"JoinAcross[{}, {}, "id" -> 2]"#,
        "JoinAcross[List[], List[], Rule[\"id\", 2]]",
        "keys",
        "The argument Rule[\"id\", 2] is not a valid key specification: a key, a rule of keys, or a non-empty list of these.",
    );
    assert_join_across_error(
        &mut session,
        r#"JoinAcross[{}, {}, "id", "Sideways"]"#,
        "JoinAcross[List[], List[], \"id\", \"Sideways\"]",
        "jtype",
        "The argument \"Sideways\" is not a valid join type; \"Inner\", \"Left\", \"Right\", or \"Outer\" is expected.",
    );
}
