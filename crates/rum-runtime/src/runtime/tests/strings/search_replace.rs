use super::*;

#[test]
fn string_repeat_and_exact_substring_predicates_cover_first_tier_two_wave() {
    let mut session = Session::new();

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
    let mut session = Session::new();

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
fn string_split_trim_replace_and_cases_cover_common_text_processing_flows() {
    let mut session = Session::new();

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
fn string_expression_and_alternatives_extend_the_current_string_subset() {
    let mut session = Session::new();

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
    let mut session = Session::new();

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
    let mut session = Session::new();

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
fn symbolic_string_patterns_bind_named_captures_in_rules() {
    let mut session = Session::new();

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
}
