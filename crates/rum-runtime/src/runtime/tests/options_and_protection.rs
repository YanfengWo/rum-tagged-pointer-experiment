use super::*;

#[test]
fn protected_targets_reject_attribute_updates_and_preserve_attributes() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("SetAttributes[s, HoldFirst]").unwrap();
    session.eval_text("Protect[s]").unwrap();

    let inserted = session.eval_text("SetAttributes[s, Listable]").unwrap();
    assert!(
        inserted
            .messages
            .iter()
            .any(|message| message.symbol == "SetAttributes" && message.tag == "wrsym")
    );
    assert_eq!(
        session
            .eval_text("Attributes[s]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, Protected]"
    );

    let removed = session.eval_text("ClearAttributes[s, HoldFirst]").unwrap();
    assert!(
        removed
            .messages
            .iter()
            .any(|message| message.symbol == "ClearAttributes" && message.tag == "wrsym")
    );
    assert_eq!(
        session
            .eval_text("Attributes[s]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, Protected]"
    );
}

#[test]
fn protected_attribute_removal_and_unprotect_remain_escape_hatches() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("SetAttributes[s, HoldFirst]").unwrap();
    session.eval_text("Protect[s]").unwrap();

    let cleared = session.eval_text("ClearAttributes[s, Protected]").unwrap();
    assert!(cleared.messages.is_empty());
    assert_eq!(
        session
            .eval_text("Attributes[s]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst]"
    );

    let updated = session.eval_text("SetAttributes[s, Listable]").unwrap();
    assert!(updated.messages.is_empty());
    assert_eq!(
        session
            .eval_text("Attributes[s]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, Listable]"
    );

    session.eval_text("Protect[s]").unwrap();
    let unprotected = session.eval_text("Unprotect[s]").unwrap();
    assert!(unprotected.messages.is_empty());
    assert_eq!(
        session
            .eval_text("Attributes[s]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, Listable]"
    );

    let user_update = session.eval_text("SetAttributes[u, HoldRest]").unwrap();
    assert!(user_update.messages.is_empty());
    assert_eq!(
        session
            .eval_text("Attributes[u]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldRest]"
    );

    session.eval_text("SetAttributes[t, HoldFirst]").unwrap();
    session.eval_text("Protect[t]").unwrap();
    let cleared_many = session
        .eval_text("ClearAttributes[t, {Protected, HoldFirst}]")
        .unwrap();
    assert!(cleared_many.messages.is_empty());
    assert_eq!(
        session
            .eval_text("Attributes[t]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn protect_and_unprotect_expand_name_patterns_without_literal_wildcards() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("cl213ProtectA = 1; cl213ProtectB = 2")
        .unwrap();

    let protected = session
        .eval_text("Protect[\"Global`cl213Protect*\"]")
        .unwrap();
    assert!(protected.messages.is_empty());
    assert_eq!(
        protected.value.to_string(),
        "List[\"cl213ProtectA\", \"cl213ProtectB\"]"
    );
    assert_eq!(
        session
            .eval_text("{Attributes[cl213ProtectA], Attributes[cl213ProtectB]}")
            .unwrap()
            .value
            .to_string(),
        "List[List[Protected], List[Protected]]"
    );
    assert_eq!(
        session
            .eval_text("NameQ[\"Global`cl213Protect*\"]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );

    let unprotected = session
        .eval_text("Unprotect[\"Global`cl213Protect*\"]")
        .unwrap();
    assert!(unprotected.messages.is_empty());
    assert_eq!(
        unprotected.value.to_string(),
        "List[\"cl213ProtectA\", \"cl213ProtectB\"]"
    );
    assert_eq!(
        session
            .eval_text("{Attributes[cl213ProtectA], Attributes[cl213ProtectB]}")
            .unwrap()
            .value
            .to_string(),
        "List[List[], List[]]"
    );

    let unmatched = session
        .eval_text("Protect[\"Global`cl213ProtectMissing*\"]")
        .unwrap();
    assert!(unmatched.messages.is_empty());
    assert_eq!(unmatched.value.to_string(), "List[]");
    assert_eq!(
        session
            .eval_text("NameQ[\"Global`cl213ProtectMissing*\"]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn options_assignment_and_queries_cover_current_first_pass_subset() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
fn protect_blocks_direct_options_assignment_and_unset_until_unprotected() {
    let mut session = session_with_test_control_builtins();

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

    session.eval_text("Unprotect[f]").unwrap();
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
fn setoptions_blocks_protected_system_head_and_unprotect_allows_update() {
    let mut session = session_with_test_control_builtins();
    let expected = "List[Rule[Delimiter, \",\"], Rule[Headers, False]]";

    assert_eq!(
        session
            .eval_text("Options[System`FromCsv]")
            .unwrap()
            .value
            .to_string(),
        expected
    );

    let blocked = session
        .eval_text("SetOptions[System`FromCsv, Delimiter -> \";\"]")
        .unwrap();
    assert!(
        blocked
            .messages
            .iter()
            .any(|message| message.symbol == "SetOptions" && message.tag == "wrsym")
    );
    assert_eq!(
        session
            .eval_text("Options[System`FromCsv]")
            .unwrap()
            .value
            .to_string(),
        expected
    );

    session.eval_text("Unprotect[System`FromCsv]").unwrap();
    let updated = session
        .eval_text("SetOptions[System`FromCsv, Delimiter -> \";\"]")
        .unwrap();
    assert!(updated.messages.is_empty());
    assert_eq!(
        session
            .eval_text("Options[System`FromCsv]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Delimiter, \";\"], Rule[Headers, False]]"
    );
}

#[test]
fn block_localizes_options_alongside_other_symbol_state() {
    let mut session = session_with_test_control_builtins();

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
fn optionspattern_and_one_arg_optionvalue_cover_current_first_pass_subset() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
fn setoptions_and_optionvalue_match_known_system_options_by_name() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("Unprotect[Save]").unwrap();
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
fn codec_and_schema_options_are_discoverable() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Options[System`ToJson]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Indent, None]]"
    );
    assert_eq!(
        session
            .eval_text("Options[System`ToXml]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Indent, None]]"
    );
    assert_eq!(
        session
            .eval_text("Options[System`FromCsv]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Delimiter, \",\"], Rule[Headers, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[System`ToCsv]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Delimiter, \",\"], Rule[Headers, Automatic]]"
    );
    assert_eq!(
        session
            .eval_text("Options[System`ConformSchema]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[AllowExtraKeys, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[System`SchemaErrors]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[AllowExtraKeys, False]]"
    );
    assert_eq!(
        session
            .eval_text("OptionValue[System`FromCsv, \"Delimiter\"]")
            .unwrap()
            .value
            .to_string(),
        "\",\""
    );
}

#[test]
fn test_create_options_are_discoverable_and_match_consumed_defaults() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Options[TestCreate]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[TestID, None], Rule[ExpectedMessages, List[]], Rule[SameTest, SameQ]]"
    );
    assert_eq!(
        session
            .eval_text("TestCreate[1 + 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"Input\", Hold[Plus[1, 1]]], Rule[\"ExpectedOutput\", Hold[2]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "TestCreate[1 + 1, 2, {TestID -> \"sum\", ExpectedMessages -> {FindFile::notfound}, SameTest -> Greater}]",
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"Input\", Hold[Plus[1, 1]]], Rule[\"ExpectedOutput\", Hold[2]], Rule[\"TestID\", \"sum\"], Rule[\"ExpectedMessages\", Hold[List[FindFile::notfound]]], Rule[\"SameTest\", Greater]]"
    );
}

#[test]
fn argumentsoptions_and_checkarguments_cover_current_first_pass_subset() {
    let mut session = session_with_test_control_builtins();

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
