use super::*;

fn eval_to_string(session: &mut Session, input: &str) -> String {
    session.eval_text(input).unwrap().value.to_string()
}

#[test]
fn string_construction_and_measurement_use_code_point_semantics() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringJoin[\"ru\", \"m\"]"),
        "\"rum\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringJoin[{\"a\", \"\\u{1F642}\", \"b\"}]"),
        "\"a🙂b\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringLength[\"\\u{732B}\\u{1F642}\"]"),
        "2"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringLength[{\"a\", \"bc\"}]"),
        "List[1, 2]"
    );
}

#[test]
fn characters_character_range_and_tostring_cover_basic_text_flows() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "Characters[\"a\\u{1F642}\"]"),
        "List[\"a\", \"🙂\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "CharacterRange[\"x\", \"z\"]"),
        "List[\"x\", \"y\", \"z\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[1/2 + x]"),
        "\"Plus[1/2, x]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[\"line\\n\"]"),
        "\"\\\"line\\\\n\\\"\""
    );
}

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
fn string_take_drop_and_part_support_basic_integer_and_range_specs() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringTake[\"abcdef\", 3]"),
        "\"abc\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringTake[\"abcdef\", -2]"),
        "\"ef\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringTake[\"abcdef\", {2, 4}]"),
        "\"bcd\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringDrop[\"abcdef\", 2]"),
        "\"cdef\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringDrop[\"abcdef\", -2]"),
        "\"abcd\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringDrop[\"abcdef\", {2, 4}]"),
        "\"aef\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPart[\"abcdef\", 2]"),
        "\"b\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPart[\"abcdef\", {1, 3, -1}]"),
        "\"acf\""
    );
}

#[test]
fn string_builtins_preserve_current_boundary_for_non_string_or_invalid_specs() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringLength[x]"),
        "StringLength[x]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringTake[\"abc\", x]"),
        "StringTake[\"abc\", x]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPart[\"abc\", 0]"),
        "StringPart[\"abc\", 0]"
    );
    assert_eq!(
        eval_to_string(&mut session, "CharacterRange[\"ab\", \"z\"]"),
        "CharacterRange[\"ab\", \"z\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringRepeat[\"abc\", -1]"),
        "StringRepeat[\"abc\", -1]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringContainsQ[\"abc\", x]"),
        "StringContainsQ[\"abc\", x]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"abc\", \"\"]"),
        "StringPosition[\"abc\", \"\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringSplit[\"abc\", {}]"),
        "StringSplit[\"abc\", List[]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringTrim[\"abc\", x]"),
        "StringTrim[\"abc\", x]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringReplace[\"abc\", x]"),
        "StringReplace[\"abc\", x]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"abc\", \"\"]"),
        "StringCases[\"abc\", \"\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringContainsQ[\"abc\", StringExpression[]]"),
        "StringContainsQ[\"abc\", StringExpression[]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"abc\", StringExpression[\"a\", x] -> \"b\"]"
        ),
        "StringReplace[\"abc\", Rule[StringExpression[\"a\", x], \"b\"]]"
    );
}
