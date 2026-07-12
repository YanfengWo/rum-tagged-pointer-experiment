use super::*;

#[test]
fn syntax_information_defaults_assignment_and_unset_cover_current_first_pass_subset() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );

    let assigned = session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_, OptionsPattern[]}}")
        .unwrap();
    assert_eq!(
        assigned.value.to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], OptionsPattern[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], OptionsPattern[]]]]"
    );

    session.eval_text("Unset[SyntaxInformation[f]]").unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn syntax_information_participates_in_block_and_clearall() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("SyntaxInformation[f] = {\"LocalVariables\" -> {x}}")
        .unwrap();

    assert_eq!(
        session
            .eval_text("Block[{f}, SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_}}; SyntaxInformation[f]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"LocalVariables\", List[x]]]"
    );

    session.eval_text("Clear[f]").unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"LocalVariables\", List[x]]]"
    );

    session.eval_text("ClearAll[f]").unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn protect_blocks_direct_syntax_information_assignment_and_unset() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_}}")
        .unwrap();
    session.eval_text("Protect[f]").unwrap();

    let assigned = session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_, _}}")
        .unwrap();
    assert!(
        assigned
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "wrsym")
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );

    let unset = session.eval_text("Unset[SyntaxInformation[f]]").unwrap();
    assert!(
        unset
            .messages
            .iter()
            .any(|message| message.symbol == "Unset" && message.tag == "wrsym")
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
}

#[test]
fn unsetting_system_syntax_information_reverts_to_default_rules() {
    let mut session = session_with_test_control_builtins();

    // Options is Protected (whole-registry sweep, brief-210); setting syntax
    // information on it requires Unprotect first, as in WL. This test uses
    // Options as the vehicle because it carries a built-in default to revert to.
    session.eval_text("Unprotect[Options]").unwrap();
    session
        .eval_text("SyntaxInformation[Options] = {\"LocalVariables\" -> {x}}")
        .unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Options]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"LocalVariables\", List[x]]]"
    );

    session
        .eval_text("Unset[SyntaxInformation[Options]]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Options]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
}
