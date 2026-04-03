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
            .eval_text("Options[Cases]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[FreeQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, True]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Position]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, True]]"
    );
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
    assert_eq!(
        session
            .eval_text("Options[Save]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[ExcludedContexts, Automatic], Rule[IncludedContexts, All]]"
    );
    assert_eq!(
        session
            .eval_text("Options[DumpSave]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[CompressionLevel, 1], Rule[SymbolAttributes, True]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Once]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[ExpirationDate, None], Rule[PersistenceTime, None]]"
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

#[test]
fn delayed_option_rules_are_preserved_and_evaluated_on_lookup() {
    let mut session = Session::new();

    session.eval_text("x = 1").unwrap();
    session.eval_text("Options[f] = {a :> x = x + 1}").unwrap();

    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[RuleDelayed[a, Set[x, Plus[x, 1]]]]"
    );
    assert_eq!(
        session
            .eval_text("OptionValue[f, a]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("OptionValue[f, a]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );

    assert_eq!(
        session
            .eval_text("SetOptions[f, a :> x = x + 10]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[a, Set[x, Plus[x, 10]]]]"
    );
    assert_eq!(
        session
            .eval_text("OptionValue[f, a]")
            .unwrap()
            .value
            .to_string(),
        "13"
    );
}

#[test]
fn delayed_system_default_options_feed_current_builtin_consumers() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("SetOptions[StringContainsQ, IgnoreCase :> True]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[IgnoreCase, True]]"
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
            .eval_text("SetOptions[Depth, Heads :> True]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[Heads, True]]"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[g][x]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn setoptions_and_optionvalue_match_known_system_options_by_name() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("SetOptions[Save, IncludedContexts -> None]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[ExcludedContexts, Automatic], Rule[IncludedContexts, None]]"
    );
    assert_eq!(
        session
            .eval_text("OptionValue[Save, IncludedContexts]")
            .unwrap()
            .value
            .to_string(),
        "None"
    );
}

#[test]
fn setoptions_updates_system_heads_defaults_for_cases_position_and_depth() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[x], f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    session
        .eval_text("SetOptions[Cases, Heads -> True]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("Cases[f[x], f]")
            .unwrap()
            .value
            .to_string(),
        "List[f]"
    );

    assert_eq!(
        session
            .eval_text("Position[f[x], f]")
            .unwrap()
            .value
            .to_string(),
        "List[List[0]]"
    );
    session
        .eval_text("SetOptions[Position, Heads -> False]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("Position[f[x], f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );

    assert_eq!(
        session
            .eval_text("Depth[f[g][x]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    session
        .eval_text("SetOptions[Depth, Heads -> True]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("Depth[f[g][x]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn argumentsoptions_and_checkarguments_cover_current_first_pass_subset() {
    let mut session = Session::new();

    session.eval_text("Options[f] = {a -> 1, b -> 2}").unwrap();

    assert_eq!(
        session
            .eval_text("ArgumentsOptions[f[x, a -> 9, b -> 8], 1]")
            .unwrap()
            .value
            .to_string(),
        "List[List[x], List[Rule[a, 9], Rule[b, 8]]]"
    );
    assert_eq!(
        session
            .eval_text("ArgumentsOptions[f[x, y, a -> 9], {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[x, y], List[Rule[a, 9]]]"
    );
    assert_eq!(
        session
            .eval_text("CheckArguments[f[x, a -> 9], 1]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    let bad_unknown = session
        .eval_text("ArgumentsOptions[f[x, c -> 9], 1]")
        .unwrap();
    assert!(
        bad_unknown
            .value
            .to_string()
            .starts_with("Failure[\"optx\",")
    );
    assert!(
        bad_unknown
            .messages
            .iter()
            .any(|message| message.symbol == "f" && message.tag == "optx")
    );

    let bad_nonopt = session
        .eval_text("CheckArguments[f[x, y], {0, 1}]")
        .unwrap();
    assert_eq!(bad_nonopt.value.to_string(), "False");
    assert!(
        bad_nonopt
            .messages
            .iter()
            .any(|message| message.symbol == "f" && message.tag == "nonopt")
    );
}
