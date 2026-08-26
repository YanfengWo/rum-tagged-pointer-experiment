use super::*;

#[test]
fn string_repeat_and_exact_substring_predicates_cover_first_tier_two_wave() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringRepeat[\"ha\", 3]"),
        "\"hahaha\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringRepeat[\"ha\", 0]"),
        "\"\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringContainsQ[\"error.log\", \".log\"]"),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringContainsQ[{\"alpha\", \"beta\"}, \"ph\"]"
        ),
        "List[True, False]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringStartsQ[{\"alpha\", \"beta\"}, \"a\"]"),
        "List[True, False]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringEndsQ[\"kernel.err\", \".err\"]"),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringEndsQ[\"\\u{732B}\\u{1F642}\", \"\\u{1F642}\"]"
        ),
        "True"
    );
}

#[test]
fn string_position_reports_overlapping_one_based_ranges() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"banana\", \"an\"]"),
        "List[List[2, 3], List[4, 5]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"ababa\", \"aba\"]"),
        "List[List[1, 3], List[3, 5]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"banana\", \"an\", 1]"),
        "List[List[2, 3]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringPosition[{\"banana\", \"berry\"}, \"rr\"]"
        ),
        "List[List[], List[List[3, 4]]]"
    );
}

#[test]
fn wide_literal_search_preserves_predicate_overlap_and_collection_semantics() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "text = StringRepeat[\"a\", 300]; needle = StringRepeat[\"a\", 16]; missing = StringJoin[needle, \"b\"]; {StringContainsQ[text, needle], StringFreeQ[text, missing], StringEndsQ[text, needle], StringCount[text, needle], StringCount[text, needle, Overlaps -> True], StringPosition[text, needle, 3], StringLength[StringDelete[text, needle]], Length[StringCases[text, needle]], StringLength[StringReplace[text, missing -> \"x\"]], Length[StringCases[text, missing -> \"x\"]]}"
        ),
        "List[True, True, True, 18, 285, List[List[1, 16], List[2, 17], List[3, 18]], 12, 18, 300, 0]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringLength[StringSplit[StringJoin[text, \"X\", text], StringJoin[needle, \"X\"]]], StringTrim[StringJoin[needle, needle, \"middle\", needle], needle]}"
        ),
        "List[List[284, 300], \"middle\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "delimiter = StringRepeat[\"x\", 64]; splitText = StringJoin[delimiter, \"a\", delimiter, delimiter, \"b\", delimiter]; {StringSplit[splitText, delimiter], StringSplit[splitText, delimiter, All]}"
        ),
        "List[List[\"a\", \"\", \"b\", \"\"], List[\"\", \"a\", \"\", \"b\", \"\"]]"
    );
}

#[test]
fn string_cases_and_position_honor_boolean_overlaps_options() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaaa\", \"aa\", Overlaps -> True]"
        ),
        "List[\"aa\", \"aa\", \"aa\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"aaaa\", \"aa\"]"),
        "List[\"aa\", \"aa\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringPosition[\"aaaa\", \"aa\", Overlaps -> False]"
        ),
        "List[List[1, 2], List[3, 4]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"aaaa\", \"aa\"]"),
        "List[List[1, 2], List[2, 3], List[3, 4]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaaa\", \"aa\", Overlaps -> All]"
        ),
        "StringCases[\"aaaa\", \"aa\", Rule[Overlaps, All]]"
    );
}

#[test]
fn string_split_trim_replace_and_cases_cover_common_text_processing_flows() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringSplit[\"  alpha\\tbeta\\n gamma  \"]"),
        "List[\"alpha\", \"beta\", \"gamma\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringSplit[\"key=value=tail\", \"=\", 2]"),
        "List[\"key\", \"value=tail\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringSplit[\",a,,b,\", \",\", All]"),
        "List[\"\", \"a\", \"\", \"b\", \"\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringSplit[{\"a,b\", \"c;d\"}, {\",\", \";\"}]"
        ),
        "List[List[\"a\", \"b\"], List[\"c\", \"d\"]]"
    );

    assert_eq!(
        eval_to_string(&mut session, "StringTrim[\"  [warn]  \"]"),
        "\"[warn]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringTrim[\"foofoobarfoo\", \"foo\"]"),
        "\"bar\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringTrim[{\"  alpha  \", \"--beta--\"}, {\"-\", \" \"}]"
        ),
        "List[\"alpha\", \"beta\"]"
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"error: error\", \"error\" -> \"warn\"]"
        ),
        "\"warn: warn\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"aaaa\", {\"aa\" -> \"b\", \"a\" -> \"x\"}]"
        ),
        "\"bb\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[{\"error\", \"warn\"}, \"r\" -> \"R\", 1]"
        ),
        "List[\"eRror\", \"waRn\"]"
    );

    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"bananarama\", \"ana\"]"),
        "List[\"ana\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"error warn error\", {\"error\" -> \"E\", \"warn\" -> \"W\"}]"
        ),
        "List[\"E\", \"W\", \"E\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[{\"abcabc\", \"zz\"}, \"abc\", 1]"
        ),
        "List[List[\"abc\"], List[]]"
    );
}

#[test]
fn string_search_and_rewrite_builtins_support_wl_operator_forms() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringContainsQ[\"a\"][\"cat\"], StringStartsQ[\"a\"][\"abc\"], StringEndsQ[\"c\"][\"abc\"], StringMatchQ[LetterCharacter..][\"abc\"], StringFreeQ[\"z\"][\"abc\"], StringReplace[\"a\" -> \"b\"][\"cat\"], StringCases[\"a\", IgnoreCase -> True][\"A cat\"], StringDelete[\"a\"][\"banana\"], StringContainsQ[\"a\"][{\"cat\", \"dog\"}]}"
        ),
        "List[True, True, True, True, True, \"cbt\", List[\"A\", \"a\"], \"bnn\", List[True, False]]"
    );

    let inert = session
        .eval_text("{StringReplace[\"a\" -> \"b\"], StringCases[\"a\"]}")
        .unwrap();
    assert_eq!(
        inert.value.to_string(),
        "List[StringReplace[Rule[\"a\", \"b\"]], StringCases[\"a\"]]"
    );
    assert!(inert.messages.is_empty());

    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringContainsQ[\"cat\", \"a\"], StringReplace[\"cat\", \"a\" -> \"b\"], StringDelete[\"banana\", \"a\"]}"
        ),
        "List[True, \"cbt\", \"bnn\"]"
    );
}

#[test]
fn string_expression_and_alternatives_extend_the_current_string_subset() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringContainsQ[\"ERROR: bad\", StringExpression[\"ERROR\", Alternatives[\": \", \" - \"]]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringStartsQ[\"[WARN] tail\", StringExpression[\"[\", Alternatives[\"WARN\", \"ERROR\"], \"]\"]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringEndsQ[\"prefix::warn\", StringExpression[Alternatives[\"::\", \"--\"], \"warn\"]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringPosition[\"aba\", StringExpression[\"a\", Alternatives[\"b\", \"ba\"]]]"
        ),
        "List[List[1, 3]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringSplit[\"a=>b:=c\", Alternatives[\"=>\", \":=\"]]"
        ),
        "List[\"a\", \"b\", \"c\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringTrim[\"[[warn]]\", Alternatives[StringExpression[\"[\", \"[\"], StringExpression[\"]\", \"]\"]]]"
        ),
        "\"warn\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"error => warn := info\", Alternatives[\"=>\", \":=\"] -> \"=\"]"
        ),
        "\"error = warn = info\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"[ERROR][WARN]\", StringExpression[\"[\", Alternatives[\"ERROR\", \"WARN\"], \"]\"]]"
        ),
        "List[\"[ERROR]\", \"[WARN]\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"[ERROR][WARN]\", StringExpression[\"[\", Alternatives[\"ERROR\", \"WARN\"], \"]\"] -> \"tag\"]"
        ),
        "List[\"tag\", \"tag\"]"
    );
}

#[test]
fn string_count_matchq_and_freeq_extend_current_pattern_subset() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringCount[\"bananarama\", \"ana\"]"),
        "1"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCount[\"a=>b:=c\", Alternatives[\"=>\", \":=\"]]"
        ),
        "2"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCount[{\"abc\", \"aba\", \"zzz\"}, StringExpression[\"a\", Alternatives[\"b\", \"ba\"]]]"
        ),
        "List[1, 1, 0]"
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"[WARN]\", StringExpression[\"[\", Alternatives[\"WARN\", \"ERROR\"], \"]\"]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[{\"warn\", \"warning\"}, Alternatives[\"warn\", \"error\"]]"
        ),
        "List[True, False]"
    );

    assert_eq!(
        eval_to_string(&mut session, "StringFreeQ[\"kernel.err\", \".log\"]"),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringFreeQ[\"error => warn\", Alternatives[\"=>\", \":=\"]]"
        ),
        "False"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringFreeQ[{\"alpha\", \"beta\"}, StringExpression[\"a\", Alternatives[\"l\", \"z\"]]]"
        ),
        "List[False, True]"
    );
}

#[test]
fn delayed_string_rules_evaluate_at_application_time() {
    let mut session = core_session();

    session.eval_text("x = \"first\"").unwrap();
    assert_eq!(
        eval_to_string(&mut session, "StringReplace[\"aa\", \"a\" :> x]"),
        "\"firstfirst\""
    );

    session.eval_text("x = \"second\"").unwrap();
    assert_eq!(
        eval_to_string(&mut session, "StringReplace[\"aa\", \"a\" :> x]"),
        "\"secondsecond\""
    );

    session.eval_text("i = 0").unwrap();
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"aaa\", \"a\" :> (i = i + 1)]"),
        "List[1, 2, 3]"
    );
}

#[test]
fn rhs_condition_guards_control_string_replace_for_both_rule_heads() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringReplace[\"abc\", \"a\" -> (\"X\" /; True)], StringReplace[\"abc\", \"a\" -> (\"X\" /; False)], StringReplace[\"abc\", \"a\" :> (\"X\" /; True)], StringReplace[\"abc\", \"a\" :> (\"X\" /; False)]}"
        ),
        "List[\"Xbc\", \"abc\", \"Xbc\", \"abc\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringReplace[\"a1b\", x_ ~~ NumberString -> (\"N\" /; True)], StringReplace[\"a1b\", x_ ~~ NumberString -> (\"N\" /; False)], StringReplace[\"a1b\", x_ ~~ NumberString :> (\"N\" /; StringMatchQ[x, LetterCharacter])], StringReplace[\"a1b\", x_ ~~ NumberString :> (\"N\" /; StringMatchQ[x, DigitCharacter])]}"
        ),
        "List[\"Nb\", \"a1b\", \"Nb\", \"a1b\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringReplace[{\"a1\", \"b2\"}, x_ ~~ NumberString :> (x /; True)], StringCases[\"a1b2\", x : DigitCharacter :> (x /; SameQ[x, \"1\"])], StringCases[\"a1b2\", DigitCharacter -> (\"d\" /; True)], StringCases[\"a1b2\", DigitCharacter :> (\"d\" /; False)], StringCases[{\"a1\", \"b2\"}, DigitCharacter :> (\"d\" /; True)]}"
        ),
        "List[List[\"a\", \"b\"], List[\"1\"], List[\"d\", \"d\"], List[], List[List[\"d\"], List[\"d\"]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringReplace[\"ab12\", StringExpression[Repeated[LetterCharacter], NumberString] :> (\"P\" /; True)], StringReplace[\"ab12\", Pattern[letters, Repeated[LetterCharacter]] ~~ NumberString :> (letters /; StringMatchQ[letters, LetterCharacter..])], StringCases[\"ab12\", StringExpression[Pattern[letters, Repeated[LetterCharacter]], Pattern[digits, Repeated[DigitCharacter]]] :> (StringJoin[digits, \"-\", letters] /; StringMatchQ[letters, LetterCharacter..])]}"
        ),
        "List[\"P\", \"ab\", List[\"12-ab\"]]"
    );
}

#[test]
fn guarded_string_rule_lists_try_later_rules_at_the_same_position() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "reverseCount = 0; {StringReplace[\"abc\", {\"a\" -> (\"X\" /; False), \"a\" -> \"Y\"}], StringReplace[\"abc\", {\"a\" -> \"Y\", \"a\" :> (\"X\" /; (reverseCount = reverseCount + 1; False))}], reverseCount, StringReplace[\"aba\", {\"a\" :> (\"X\" /; False), \"b\" :> (\"B\" /; True)}], StringCases[\"a\", {\"a\" :> (\"X\" /; False), \"a\" -> \"Y\"}], StringCases[\"a\", {\"a\" -> \"Y\", \"a\" :> (\"X\" /; False)}]}"
        ),
        "List[\"Ybc\", \"Ybc\", 0, \"aBa\", List[\"Y\"], List[\"Y\"]]"
    );
}

#[test]
fn guarded_string_rules_preserve_character_positions_and_candidate_counts() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "g1 = 0; g2 = 0; result = StringReplace[\"aba\", {\"a\" :> (\"X\" /; (g1 = g1 + 1; False)), x_ :> (\"Y\" /; (g2 = g2 + 1; SameQ[x, \"b\"]))}]; {result, g1, g2}"
        ),
        "List[\"aYa\", 2, 3]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringReplace[\"猫a猫\", {\"猫\" :> (\"X\" /; False), \"a\" :> (\"A\" /; True)}], StringReplace[\"a\", {RepeatedNull[\"z\"] :> (\"x\" /; False), \"a\" :> (\"A\" /; True)}]}"
        ),
        "List[\"猫A猫\", \"A\"]"
    );
}

#[test]
fn guarded_string_cases_pin_overlapping_adjacent_and_empty_matches() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "overlapCount = 0; adjacentCount = 0; {StringCases[\"aaaa\", \"aa\" :> (\"x\" /; (overlapCount = overlapCount + 1; OddQ[overlapCount])), Overlaps -> True], overlapCount, StringReplace[\"aaaa\", \"aa\" :> (\"X\" /; (adjacentCount = adjacentCount + 1; True))], adjacentCount, StringCases[\"\", RepeatedNull[\"a\"] :> (\"x\" /; True)], StringCases[\"\", RepeatedNull[\"a\"] :> (\"x\" /; False)]}"
        ),
        "List[List[\"x\", \"x\"], 3, \"XX\", 2, List[\"x\"], List[]]"
    );
}

#[test]
fn non_boolean_string_guards_decline_and_delayed_bodies_wait_for_acceptance() {
    let mut session = core_session();

    let outcome = session
        .eval_text(
            "bodyCount = 0; acceptedBodyCount = 0; {StringReplace[\"aba\", {\"a\" :> (\"X\" /; unknownGuard), \"b\" :> (\"B\" /; True)}], StringReplace[\"a\", \"a\" :> ((bodyCount = bodyCount + 1; \"X\") /; False)], bodyCount, StringReplace[\"a\", \"a\" :> ((acceptedBodyCount = acceptedBodyCount + 1; \"X\") /; True)], acceptedBodyCount}",
        )
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[\"aBa\", \"a\", 0, \"X\", 1]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn string_rule_guards_and_bodies_share_the_enclosing_option_scope() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "Options[f] = {accept -> False}; f[OptionsPattern[]] := StringReplace[\"a\", \"a\" :> (If[OptionValue[accept], \"X\", \"bad\"] /; OptionValue[accept])]; {f[], f[accept -> True]}"
        ),
        "List[\"a\", \"X\"]"
    );
}

#[test]
fn string_split_does_not_accept_rule_delimiter_specs() {
    let mut session = core_session();

    let outcome = session
        .eval_text("StringSplit[\"a-b\", \"-\" -> (\"s\" /; True)]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "StringSplit[\"a-b\", Rule[\"-\", Condition[\"s\", True]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn symbolic_string_patterns_bind_named_captures_in_rules() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"ab\", x_ :> StringJoin[\"<\", x, \">\"]]"
        ),
        "\"<a><b>\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringReplace[\"ab\", x_ -> x]"),
        "\"ab\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"ab12\", StringExpression[Pattern[letters, Repeated[LetterCharacter]], Pattern[digits, Repeated[DigitCharacter]]] -> {letters, digits}]"
        ),
        "List[List[\"ab\", \"12\"]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"ab12\", StringExpression[Pattern[letters, Repeated[LetterCharacter]], Pattern[digits, Repeated[DigitCharacter]]] :> StringJoin[digits, \"-\", letters]]"
        ),
        "List[\"12-ab\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"aa\", StringExpression[Pattern[x, LetterCharacter], Pattern[x, LetterCharacter]]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"ab\", StringExpression[Pattern[x, LetterCharacter], Pattern[x, LetterCharacter]]]"
        ),
        "False"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"foo bar\", Pattern[word, BlankSequence[LetterCharacter]] :> word]"
        ),
        "List[\"foo\", \"bar\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"ab12\", Pattern[digits, RegularExpression[\"\\\\d+\"]] :> digits]"
        ),
        "List[\"12\"]"
    );
}

// The list walks must iterate instead of recursing per string: a 50000-string
// input aborted the process with a native stack overflow before the loop
// conversion, and the per-string state clone made the walk quadratic.

#[test]
fn string_replace_and_cases_walk_fifty_thousand_strings() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "result = StringReplace[Table[StringJoin[\"ab\", ToString[i]], {i, 50000}], \"b\" -> \"x\"]; {Length[result], result[[1]], result[[25000]], result[[50000]]}"
        ),
        "List[50000, \"ax1\", \"ax25000\", \"ax50000\"]"
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            "result = StringCases[Table[StringJoin[\"ab\", ToString[i]], {i, 50000}], \"b\" -> \"x\"]; {Length[result], result[[1]], result[[25000]], result[[50000]]}"
        ),
        "List[50000, List[\"x\"], List[\"x\"], List[\"x\"]]"
    );
}
