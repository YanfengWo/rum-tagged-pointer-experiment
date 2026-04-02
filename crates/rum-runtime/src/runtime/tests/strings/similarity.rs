use super::*;

#[test]
fn alphabet_distance_alignment_and_diff_extend_cl07() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "Alphabet[]"),
        "List[\"a\", \"b\", \"c\", \"d\", \"e\", \"f\", \"g\", \"h\", \"i\", \"j\", \"k\", \"l\", \"m\", \"n\", \"o\", \"p\", \"q\", \"r\", \"s\", \"t\", \"u\", \"v\", \"w\", \"x\", \"y\", \"z\"]"
    );
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
