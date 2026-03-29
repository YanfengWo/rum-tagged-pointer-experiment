use super::*;

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
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaaa\", Repeated[\"a\", {2, Infinity}]]"
        ),
        "List[\"aaaa\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"aaa\", Repeated[\"a\", {2, Infinity}]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaa\", RepeatedNull[\"a\", {1, Infinity}]]"
        ),
        "List[\"aaa\"]"
    );
}
