use super::*;
use std::fs;

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
        eval_to_string(&mut session, "StringTakeDrop[\"abcdef\", 2]"),
        "List[\"ab\", \"cdef\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringTakeDrop[\"abcdef\", {2, 4}]"),
        "List[\"bcd\", \"aef\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringTakeDrop[{\"abcd\", \"wxyz\"}, -1]"),
        "List[List[\"d\", \"abc\"], List[\"z\", \"wxy\"]]"
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
        eval_to_string(&mut session, "StringTakeDrop[\"abc\", x]"),
        "StringTakeDrop[\"abc\", x]"
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
        eval_to_string(&mut session, "StringCount[\"abc\", StringExpression[]]"),
        "StringCount[\"abc\", StringExpression[]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"abc\", StringExpression[]]"),
        "StringMatchQ[\"abc\", StringExpression[]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringContainsQ[\"abc\", StringExpression[]]"),
        "StringContainsQ[\"abc\", StringExpression[]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringFreeQ[\"abc\", x]"),
        "StringFreeQ[\"abc\", x]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"abc\", StringExpression[\"a\", x] -> \"b\"]"
        ),
        "StringReplace[\"abc\", Rule[StringExpression[\"a\", x], \"b\"]]"
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
fn string_character_classes_and_case_mapping_extend_the_string_subset() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "ToUpperCase[\"ab🙂c\"]"),
        "\"AB🙂C\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToUpperCase[{\"alpha\", \"Beta\"}]"),
        "List[\"ALPHA\", \"BETA\"]"
    );

    assert_eq!(eval_to_string(&mut session, "LetterQ[\"\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "LetterQ[\"abc\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "LetterQ[\"ab1\"]"), "False");
    assert_eq!(
        eval_to_string(&mut session, "LetterQ[{\"abc\", \"123\"}]"),
        "List[True, False]"
    );

    assert_eq!(eval_to_string(&mut session, "DigitQ[\"\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "DigitQ[\"123\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "DigitQ[\"12a\"]"), "False");
    assert_eq!(
        eval_to_string(&mut session, "DigitQ[{\"123\", \"1a\"}]"),
        "List[True, False]"
    );

    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\" \\t\", Whitespace]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringFreeQ[\"abc\", Whitespace]"),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"x=-1.5 y=.25 z=1e3\", NumberString]"
        ),
        "List[\"-1.5\", \".25\", \"1\", \"3\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[{\"123\", \".5\", \"1e3\"}, NumberString]"
        ),
        "List[True, True, False]"
    );
}

#[test]
fn string_formatting_padding_and_riffle_cover_the_next_surface_wave() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringForm[\"a``b``\", 1, x]"),
        "\"a1bx\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringRiffle[{\"a\", \"b\", \"c\"}]"),
        "\"a b c\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringRiffle[{{\"a\", \"b\"}, {\"c\", \"d\"}}, \",\", \";\"]"
        ),
        "\"a;b,c;d\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadLeft[\"7\", 3]"),
        "\"  7\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadLeft[{\"7\", \"88\"}, 3, \"0\"]"),
        "List[\"007\", \"088\"]"
    );
}

#[test]
fn repeated_anchors_and_shortest_longest_extend_string_patterns() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"aaa\", Repeated[\"a\"]]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"\", RepeatedNull[\"a\"]]"),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"warn\", StringExpression[StartOfString, \"wa\", Repeated[\"r\"], \"n\", EndOfString]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringContainsQ[\"baaa\", StringExpression[StartOfString, \"a\"]]"
        ),
        "False"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"aaa\", Repeated[\"a\"]]"),
        "List[List[1, 3], List[2, 3], List[3, 3]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaab\", Shortest[Repeated[\"a\"]]]"
        ),
        "List[\"a\", \"a\", \"a\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaab\", Longest[Repeated[\"a\"]]]"
        ),
        "List[\"aaa\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"\", RepeatedNull[\"a\"]]"),
        "List[\"\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"\", RepeatedNull[\"a\"]]"),
        "List[List[1, 0]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"abc\", EndOfString]"),
        "List[List[4, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCount[\"abc\", RepeatedNull[\"a\"]]"),
        "4"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringContainsQ[\"abc\", EndOfString]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringFreeQ[\"abc\", EndOfString]"),
        "False"
    );
}

#[test]
fn string_apply_delete_and_count_options_cover_the_next_wave() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringApply[Function[StringJoin[Rest[#]]], \"abc\"]"
        ),
        "\"bc\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringApply[Function[StringJoin[Rest[#]]], {\"ab\", \"cd\"}]"
        ),
        "List[\"b\", \"d\"]"
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringDelete[\"a=>b:=c\", Alternatives[\"=>\", \":=\"]]"
        ),
        "\"abc\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringDelete[{\"abba\", \"beta\"}, \"b\"]"),
        "List[\"aa\", \"eta\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringDelete[\"AbBa\", \"b\", IgnoreCase -> True]"
        ),
        "\"Aa\""
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCount[\"aaaa\", \"aa\", Overlaps -> True]"
        ),
        "3"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCount[\"aaaa\", \"aa\", Overlaps -> All]"
        ),
        "3"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCount[\"abAB\", \"a\", IgnoreCase -> True]"
        ),
        "2"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringContainsQ[\"AbC\", \"a\", IgnoreCase -> True]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringStartsQ[\"AbC\", \"a\", IgnoreCase -> True]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringEndsQ[\"xyAB\", \"ab\", IgnoreCase -> True]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"Ab\", \"aB\", IgnoreCase -> True]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringFreeQ[\"AbC\", \"a\", IgnoreCase -> True]"
        ),
        "False"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"AbBa\", \"b\" -> \"x\", IgnoreCase -> True]"
        ),
        "\"Axxa\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"AbBa\", \"b\", IgnoreCase -> True]"
        ),
        "List[\"b\", \"B\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringPosition[\"AbBa\", \"b\", IgnoreCase -> True]"
        ),
        "List[List[2, 2], List[3, 3]]"
    );
}

#[test]
fn templates_text_words_codes_and_to_expression_extend_cl07() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringTemplate[\"Hello, `name`!\"]"),
        "TemplateObject[\"Hello, `name`!\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "TemplateApply[StringTemplate[\"Hello, `name`!\"], <|\"name\" -> \"Alice\"|>]"
        ),
        "\"Hello, Alice!\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TemplateApply[\"x = ``\", 7]"),
        "\"x = 7\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TextWords[\"alpha, beta 123 gamma\"]"),
        "List[\"alpha\", \"beta\", \"123\", \"gamma\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ToCharacterCode[\"Ab\"]"),
        "List[65, 98]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ToCharacterCode[{\"Ab\", \"中\"}]"),
        "List[List[65, 98], List[20013]]"
    );
    assert_eq!(eval_to_string(&mut session, "ToExpression[\"1+2\"]"), "3");
    assert_eq!(
        eval_to_string(
            &mut session,
            "ToExpression[\"1+2\", InputForm, HoldAllComplete]"
        ),
        "HoldAllComplete[3]"
    );
}

#[test]
fn bounded_repetition_and_except_extend_string_patterns() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"aaaa\", Repeated[\"a\", {2}]]"),
        "List[\"aa\", \"aa\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaaa\", Repeated[\"a\", {2, 3}]]"
        ),
        "List[\"aaa\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"abc\", Except[\"b\"]]"),
        "List[\"a\", \"c\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"abc\", Except[\"b\", \"c\"]]"),
        "List[\"c\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"abc\", Repeated[Except[\"b\"]]]"
        ),
        "List[\"a\", \"c\"]"
    );
}

#[test]
fn syntaxq_textstring_filetemplateapply_charactercounts_and_removediacritics_extend_cl07() {
    let mut session = Session::new();

    assert_eq!(eval_to_string(&mut session, "SyntaxQ[\"1+2\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "SyntaxQ[\"1+\"]"), "False");
    assert_eq!(eval_to_string(&mut session, "TextString[123]"), "\"123\"");
    assert_eq!(
        eval_to_string(&mut session, "TextString[{1, 2}]"),
        "\"List[1, 2]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "CharacterCounts[\"abca\"]"),
        "Association[Rule[\"a\", 2], Rule[\"c\", 1], Rule[\"b\", 1]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "CharacterCounts[\"中中a\"]"),
        "Association[Rule[\"中\", 2], Rule[\"a\", 1]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "RemoveDiacritics[\"café naïve façade\"]"),
        "\"cafe naive facade\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "RemoveDiacritics[{\"café\", \"São\", \"Žižek\"}]"
        ),
        "List[\"cafe\", \"Sao\", \"Zizek\"]"
    );

    let source_path =
        std::env::temp_dir().join(format!("rum-file-template-{}.txt", std::process::id()));
    fs::write(&source_path, "Hello, `name`!").unwrap();

    let expr = format!(
        "FileTemplateApply[\"{}\", <|\"name\" -> \"Alice\"|>]",
        source_path.display()
    );
    let output = session.eval_text(&expr).unwrap().value;
    let Expr::Atom(Atom::String(output_path)) = output else {
        panic!("FileTemplateApply should return a path string");
    };

    assert_eq!(fs::read_to_string(&output_path).unwrap(), "Hello, Alice!");

    fs::remove_file(&source_path).unwrap();
    fs::remove_file(&output_path).unwrap();
}

#[test]
fn alphabet_distance_alignment_and_diff_extend_cl07() {
    let mut session = Session::new();

    assert_eq!(eval_to_string(&mut session, "Alphabet[]"), "List[\"a\", \"b\", \"c\", \"d\", \"e\", \"f\", \"g\", \"h\", \"i\", \"j\", \"k\", \"l\", \"m\", \"n\", \"o\", \"p\", \"q\", \"r\", \"s\", \"t\", \"u\", \"v\", \"w\", \"x\", \"y\", \"z\"]");
    assert_eq!(
        eval_to_string(&mut session, "Alphabet[\"English\"]"),
        "List[\"a\", \"b\", \"c\", \"d\", \"e\", \"f\", \"g\", \"h\", \"i\", \"j\", \"k\", \"l\", \"m\", \"n\", \"o\", \"p\", \"q\", \"r\", \"s\", \"t\", \"u\", \"v\", \"w\", \"x\", \"y\", \"z\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "AlphabeticSort[{\"z\", \"a\", \"ä\"}]"),
        "List[\"a\", \"ä\", \"z\"]"
    );

    assert_eq!(
        eval_to_string(&mut session, "EditDistance[\"kitten\", \"sitting\"]"),
        "3"
    );
    assert_eq!(
        eval_to_string(&mut session, "EditDistance[{a, b, c}, {a, x, c, d}]"),
        "2"
    );
    assert_eq!(
        eval_to_string(&mut session, "HammingDistance[\"karolin\", \"kathrin\"]"),
        "3"
    );
    assert_eq!(
        eval_to_string(&mut session, "HammingDistance[{a, b, c}, {a, x, c}]"),
        "1"
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            "LongestCommonSubsequence[\"abcdef\", \"acf\"]"
        ),
        "\"a\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "LongestCommonSubsequence[{a, b, c, d}, {b, c, e}]"
        ),
        "List[b, c]"
    );

    assert_eq!(
        eval_to_string(&mut session, "SequenceAlignment[\"ABC\", \"ADC\"]"),
        "List[\"A\", List[\"B\", \"D\"], \"C\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "SequenceAlignment[{a, b, c}, {a, x, c}]"),
        "List[List[a], List[List[b], List[x]], List[c]]"
    );

    assert_eq!(
        eval_to_string(&mut session, "Diff[\"abc\", \"adc\"]"),
        "DiffObject[1, \"Characters\", Association[Rule[\"Changes\", List[List[Rule[List[1, 1], \"a\"]], List[Rule[2, \"b\"], Rule[2, \"d\"]], List[Rule[List[3, 3], \"c\"]]]], Rule[\"View\", List[Automatic]]]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Diff[{a, b, c}, {a, x, c}]"),
        "DiffObject[1, \"Elements\", Association[Rule[\"Changes\", List[List[Rule[List[1, 1], List[a]]], List[Rule[2, List[b]], Rule[2, List[x]]], List[Rule[List[3, 3], List[c]]]]], Rule[\"View\", List[Automatic]]]]"
    );
}

#[test]
fn byte_arrays_hashing_transport_and_nearest_extend_cl07() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringToByteArray[\"abc\"]"),
        "ByteArray[\"YWJj\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringToByteArray[\"🙂\"]"),
        "ByteArray[\"8J+Zgg==\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Hash[\"abc\"]"),
        "84342368487090800366523834928142263660104883695016514377462985829716817089965"
    );
    assert_eq!(
        eval_to_string(&mut session, "Hash[\"abc\", \"SHA256\", \"HexString\"]"),
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Hash[\"abc\", \"SHA256\", \"DecimalString\"]"),
        "\"084342368487090800366523834928142263660104883695016514377462985829716817089965\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Compress[1 + 1]"),
        "\"rum:1:e:Mg==\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Compress[\"abc\"]"),
        "\"rum:1:s:YWJj\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Head[Encrypt[\"secret\", \"abc\"]]"),
        "EncryptedObject"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3]"),
        "List[2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3, 2]"),
        "List[2, 1]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5} -> {a, b, c}, 3]"),
        "List[b]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"dog\"}, \"cut\"]"
        ),
        "List[\"cat\", \"cot\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"cat\"]"),
        "\"cats\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"cat\", 1]"),
        "\"cat\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"city\", 2]"),
        "\"cities\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"box\", 2]"),
        "\"boxes\""
    );
}
