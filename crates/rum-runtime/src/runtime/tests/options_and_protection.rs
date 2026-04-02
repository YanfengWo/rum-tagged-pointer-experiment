use super::*;

#[test]
fn options_assignment_and_queries_cover_current_first_pass_subset() {
    let mut session = Session::new();

    let assigned = session.eval_text("Options[f] = {a -> 0, b -> 2}").unwrap();
    assert_eq!(assigned.value.to_string(), "List[Rule[a, 0], Rule[b, 2]]");

    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[Rule[a, 0], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Options[\"f\"]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 0], Rule[b, 2]]"
    );

    assert_eq!(
        session
            .eval_text("OptionValue[f, a]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("OptionValue[f, {a -> 9}, a]")
            .unwrap()
            .value
            .to_string(),
        "9"
    );
}

#[test]
fn optionvalue_emits_current_optnf_and_nodef_messages() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 0}").unwrap();

    let missing_default = session.eval_text("OptionValue[f, b]").unwrap();
    assert_eq!(missing_default.value.to_string(), "b");
    assert!(
        missing_default
            .messages
            .iter()
            .any(|message| { message.symbol == "OptionValue" && message.tag == "optnf" })
    );

    let explicit_unknown = session.eval_text("OptionValue[f, {b -> 9}, b]").unwrap();
    assert_eq!(explicit_unknown.value.to_string(), "9");
    assert!(
        explicit_unknown
            .messages
            .iter()
            .any(|message| { message.symbol == "OptionValue" && message.tag == "nodef" })
    );
}

#[test]
fn setoptions_updates_known_options_but_rejects_unknown_ones() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 0, b -> 2}").unwrap();

    let updated = session.eval_text("SetOptions[f, a -> 1]").unwrap();
    assert_eq!(updated.value.to_string(), "List[Rule[a, 1], Rule[b, 2]]");
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[Rule[a, 1], Rule[b, 2]]"
    );

    let rejected = session.eval_text("SetOptions[f, c -> 3]").unwrap();
    assert_eq!(rejected.value.to_string(), "SetOptions[f, Rule[c, 3]]");
    assert!(
        rejected
            .messages
            .iter()
            .any(|message| { message.symbol == "SetOptions" && message.tag == "optnf" })
    );
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[Rule[a, 1], Rule[b, 2]]"
    );
}

#[test]
fn options_assignment_unset_and_clearall_interact_with_symbol_state() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 0}").unwrap();
    session.eval_text("Clear[f]").unwrap();
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[Rule[a, 0]]"
    );

    session.eval_text("Unset[Options[f]]").unwrap();
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[]"
    );

    session.eval_text("Options[f] = {a -> 0}").unwrap();
    session.eval_text("ClearAll[f]").unwrap();
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[]"
    );
}

#[test]
fn protect_blocks_direct_options_assignment_but_not_setoptions_updates() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 0}").unwrap();
    session.eval_text("Protect[f]").unwrap();

    let assigned = session.eval_text("Options[f] = {a -> 1}").unwrap();
    assert_eq!(assigned.value.to_string(), "List[Rule[a, 1]]");
    assert!(
        assigned
            .messages
            .iter()
            .any(|message| { message.symbol == "Set" && message.tag == "wrsym" })
    );
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[Rule[a, 0]]"
    );

    let unset = session.eval_text("Unset[Options[f]]").unwrap();
    assert_eq!(unset.value.to_string(), "Null");
    assert!(
        unset
            .messages
            .iter()
            .any(|message| { message.symbol == "Unset" && message.tag == "wrsym" })
    );
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[Rule[a, 0]]"
    );

    let updated = session.eval_text("SetOptions[f, a -> 1]").unwrap();
    assert_eq!(updated.value.to_string(), "List[Rule[a, 1]]");
    assert_eq!(
        session
            .eval_text("OptionValue[f, a]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
}

#[test]
fn block_localizes_options_alongside_other_symbol_state() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 0}").unwrap();

    assert_eq!(
        session
            .eval_text("Block[{f}, Options[f] = {a -> 9}; OptionValue[f, a]]")
            .unwrap()
            .value
            .to_string(),
        "9"
    );

    assert_eq!(
        session
            .eval_text("OptionValue[f, a]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
}

#[test]
fn system_default_options_cover_current_first_pass_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Options[StringCount]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[Overlaps, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[CheckAbort]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[PropagateAborts, Automatic]]"
    );
    assert_eq!(
        session
            .eval_text("Options[ArrayFilter]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Padding, \"Fixed\"]]"
    );
}

#[test]
fn setoptions_updates_current_system_default_subset_behavior() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("SetOptions[StringContainsQ, IgnoreCase -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, True]]"
    );
    assert_eq!(
        session
            .eval_text("StringContainsQ[\"Ab\", \"a\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("StringContainsQ[\"Ab\", \"a\", IgnoreCase -> False]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );

    assert_eq!(
        session
            .eval_text("SetOptions[StringCount, Overlaps -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[Overlaps, True]]"
    );
    assert_eq!(
        session
            .eval_text("StringCount[\"aaaa\", \"aa\"]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn optionspattern_and_one_arg_optionvalue_cover_current_first_pass_subset() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 1, b -> 2}").unwrap();
    session
        .eval_text("f[x_, OptionsPattern[]] := {x, OptionValue[a], OptionValue[b]}")
        .unwrap();

    assert_eq!(
        session.eval_text("f[7]").unwrap().value.to_string(),
        "List[7, 1, 2]"
    );
    assert_eq!(
        session.eval_text("f[7, a -> 9]").unwrap().value.to_string(),
        "List[7, 9, 2]"
    );
}

#[test]
fn optionspattern_can_use_an_explicit_target_symbol() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 1}").unwrap();
    session
        .eval_text("g[x_, OptionsPattern[f]] := {x, OptionValue[a]}")
        .unwrap();

    assert_eq!(
        session.eval_text("g[7]").unwrap().value.to_string(),
        "List[7, 1]"
    );
    assert_eq!(
        session.eval_text("g[7, a -> 9]").unwrap().value.to_string(),
        "List[7, 9]"
    );
}

#[test]
fn setoptions_accepts_target_lists_and_returns_per_target_results() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 0, b -> 2}").unwrap();
    session.eval_text("Options[g] = {a -> 1, b -> 3}").unwrap();

    assert_eq!(
        session
            .eval_text("SetOptions[{f, g}, {a -> 9, b -> 8}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[Rule[a, 9], Rule[b, 8]], List[Rule[a, 9], Rule[b, 8]]]"
    );
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[Rule[a, 9], Rule[b, 8]]"
    );
    assert_eq!(
        session.eval_text("Options[g]").unwrap().value.to_string(),
        "List[Rule[a, 9], Rule[b, 8]]"
    );
}

#[test]
fn optionvalue_accepts_lists_of_option_names() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 1, b -> 2}").unwrap();
    session
        .eval_text("f[x_, OptionsPattern[]] := {OptionValue[{a, b}], OptionValue[{\"a\", \"b\"}]}")
        .unwrap();

    assert_eq!(
        session
            .eval_text("OptionValue[f, {a -> 9, b -> 8}, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[9, 8]"
    );
    assert_eq!(
        session
            .eval_text("OptionValue[f, {a -> 9, b -> 8}, {\"a\", \"b\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[9, 8]"
    );
    assert_eq!(
        session.eval_text("f[7]").unwrap().value.to_string(),
        "List[List[1, 2], List[1, 2]]"
    );
    assert_eq!(
        session.eval_text("f[7, a -> 9]").unwrap().value.to_string(),
        "List[List[9, 2], List[9, 2]]"
    );
}
