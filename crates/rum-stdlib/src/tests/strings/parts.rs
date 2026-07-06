use super::*;

#[test]
fn string_take_drop_and_part_support_basic_integer_and_range_specs() {
    let mut session = core_session();

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
    let mut session = core_session();

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
fn string_apply_delete_and_count_options_cover_the_next_wave() {
    let mut session = core_session();

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
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringSplit[\"AxbXc\", \"x\", IgnoreCase -> True]"
        ),
        "List[\"A\", \"b\", \"c\"]"
    );
}
