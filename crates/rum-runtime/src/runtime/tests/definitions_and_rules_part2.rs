use super::*;

#[test]
fn compiled_matcher_matches_interpreted_matcher_on_supported_subset() {
    let mut session = Session::new();

    for (pattern_src, value_src, expected) in [
        ("x_Integer", "7", true),
        ("x_String", "7", false),
        ("f[x_Integer, y_String]", "f[7, \"hi\"]", true),
        ("f[x_Integer /; MatchQ[x, 7]]", "f[7]", true),
        ("f[x_Integer /; MatchQ[x, 8]]", "f[7]", false),
        ("f[x__, y_Integer]", "f[1, 2, 3]", true),
        ("f[x___]", "f[]", true),
    ] {
        let pattern = lower_syntax_expr(&parse_expr(pattern_src).unwrap()).unwrap();
        let pattern = session.normalize_pattern_expr(&pattern).unwrap();
        let value = lower_syntax_expr(&parse_expr(value_src).unwrap()).unwrap();
        let value = session.evaluate(value).unwrap();
        let mut interpreted_bindings = BTreeMap::new();
        let mut compiled_bindings = BTreeMap::new();

        let interpreted = session
            .match_expr_interpreted(&pattern, &value, &mut interpreted_bindings)
            .unwrap();
        let compiled = session
            .match_expr(&pattern, &value, &mut compiled_bindings)
            .unwrap();

        assert_eq!(
            interpreted, expected,
            "interpreted mismatch for {pattern_src}"
        );
        assert_eq!(compiled, expected, "compiled mismatch for {pattern_src}");
        assert_eq!(
            compiled_bindings, interpreted_bindings,
            "binding mismatch for {pattern_src}"
        );
    }
}

#[test]
fn inspect_definition_catalog_reports_kinds_and_delayedness() {
    let mut session = Session::new();

    session.eval_text("x = 42").unwrap();
    session.eval_text("y := x").unwrap();
    session.eval_text("f[z_Integer] := z").unwrap();
    session.eval_text("a /: g[a] := 1").unwrap();
    session.eval_text("h[u_][v_] := pair[u, v]").unwrap();

    let own_entries = session.inspect_definition_catalog("x").unwrap();
    assert_eq!(own_entries.len(), 1);
    assert_eq!(own_entries[0].kind, DefinitionKind::OwnValue);
    assert!(!own_entries[0].delayed);
    assert_eq!(own_entries[0].lhs.to_string(), "x");
    assert_eq!(own_entries[0].rhs.to_string(), "42");

    let delayed_entries = session.inspect_definition_catalog("y").unwrap();
    assert_eq!(delayed_entries.len(), 1);
    assert_eq!(delayed_entries[0].kind, DefinitionKind::DelayedOwnValue);
    assert!(delayed_entries[0].delayed);

    let down_entries = session.inspect_definition_catalog("f").unwrap();
    assert_eq!(down_entries.len(), 1);
    assert_eq!(down_entries[0].kind, DefinitionKind::DownValue);
    assert!(down_entries[0].delayed);
    assert_eq!(
        down_entries[0].lhs.to_string(),
        "f[Pattern[z, Blank[Integer]]]"
    );

    let up_entries = session.inspect_definition_catalog("a").unwrap();
    assert_eq!(up_entries.len(), 1);
    assert_eq!(up_entries[0].kind, DefinitionKind::UpValue);
    assert!(up_entries[0].delayed);
    assert_eq!(up_entries[0].lhs.to_string(), "g[a]");

    let sub_entries = session.inspect_definition_catalog("h").unwrap();
    assert_eq!(sub_entries.len(), 1);
    assert_eq!(sub_entries[0].kind, DefinitionKind::SubValue);
    assert!(sub_entries[0].delayed);
    assert_eq!(
        sub_entries[0].lhs.to_string(),
        "h[Pattern[u, Blank[]]][Pattern[v, Blank[]]]"
    );
}

#[test]
fn set_supports_immediate_pattern_based_downvalues_and_subvalues() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("f[x_] = 1").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(session.eval_text("f[z]").unwrap().value.to_string(), "1");

    assert_eq!(
        session
            .eval_text("g[x_][y_] = pair[x, y]")
            .unwrap()
            .value
            .to_string(),
        "pair[x, y]"
    );
    assert_eq!(
        session.eval_text("g[1][2]").unwrap().value.to_string(),
        "pair[1, 2]"
    );
}

#[test]
fn lowers_named_pattern_assignment() {
    let syntax = parse_expr("x_Integer := f[x]").unwrap();
    let lowered = lower_syntax_expr(&syntax).unwrap();

    assert_eq!(
        lowered.to_string(),
        "SetDelayed[Pattern[x, Blank[Integer]], f[x]]"
    );
}

#[test]
fn lowers_slots_and_pure_functions() {
    let syntax = parse_expr("#1 + #2 &").unwrap();
    let lowered = lower_syntax_expr(&syntax).unwrap();

    assert_eq!(lowered.to_string(), "Function[Plus[Slot[1], Slot[2]]]");

    let sequence_syntax = parse_expr("## + ##2 &").unwrap();
    let sequence_lowered = lower_syntax_expr(&sequence_syntax).unwrap();

    assert_eq!(
        sequence_lowered.to_string(),
        "Function[Plus[SlotSequence[1], SlotSequence[2]]]"
    );

    let named_syntax = parse_expr("#x &").unwrap();
    let named_lowered = lower_syntax_expr(&named_syntax).unwrap();

    assert_eq!(named_lowered.to_string(), "Function[Slot[\"x\"]]");
}

#[test]
fn setdelayed_stores_and_applies_simple_downvalue() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("f[x_Integer] := x")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");

    let info = session.inspect("f").unwrap();
    assert!(info.has_down_values);
}

#[test]
fn matchq_reports_true_for_matching_pattern() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[7, x_Integer]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_reports_false_for_non_matching_pattern() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[\"hi\", x_Integer]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_supports_blanksequence_in_argument_lists() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2], f[x__]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[x__]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_supports_blanknullsequence_in_argument_lists() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[x___]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2, 3], f[x__, y_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_supports_conditions_on_sequence_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2], f[x__ /; MatchQ[x, Sequence[1, 2]]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2], f[x__ /; MatchQ[x, Sequence[2, 1]]]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn downvalue_matches_after_argument_evaluation() {
    let mut session = Session::new();

    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("y = 11").unwrap();

    assert_eq!(session.eval_text("f[y]").unwrap().value.to_string(), "11");
}

#[test]
fn downvalue_can_substitute_multiple_named_patterns() {
    let mut session = Session::new();

    session.eval_text("pair[a_, b_] := {a, b}").unwrap();

    assert_eq!(
        session.eval_text("pair[1, 2]").unwrap().value.to_string(),
        "List[1, 2]"
    );
}

#[test]
fn downvalue_splices_named_blanksequence_into_rhs_calls() {
    let mut session = Session::new();

    session.eval_text("echo[x__] := g[x]").unwrap();

    assert_eq!(
        session
            .eval_text("echo[1, 2, 3]")
            .unwrap()
            .value
            .to_string(),
        "g[1, 2, 3]"
    );
}

#[test]
fn downvalue_splices_named_blanknullsequence_into_empty_lists() {
    let mut session = Session::new();

    session.eval_text("pack[x___] := {x}").unwrap();

    assert_eq!(
        session.eval_text("pack[]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("pack[1, 2]").unwrap().value.to_string(),
        "List[1, 2]"
    );
}

#[test]
fn unmatched_downvalue_leaves_call_in_place() {
    let mut session = Session::new();

    session.eval_text("f[x_Integer] := x").unwrap();

    assert_eq!(
        session.eval_text("f[\"hi\"]").unwrap().value.to_string(),
        "f[\"hi\"]"
    );
}
