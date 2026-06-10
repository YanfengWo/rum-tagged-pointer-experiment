use super::*;

#[test]
fn exact_aggregation_shapes_used_by_mma_hol_arith_reduce() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Total[{}]").unwrap().value.to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("Total[{1, 2, 1/2}]")
            .unwrap()
            .value
            .to_string(),
        "7/2"
    );
    assert_eq!(
        session
            .eval_text("Total[{{1, 2}, {3, 4}}]")
            .unwrap()
            .value
            .to_string(),
        "List[4, 6]"
    );
    assert_eq!(
        session.eval_text("Total[x]").unwrap().value.to_string(),
        "Total[x]"
    );
    assert_eq!(
        session
            .eval_text(
                "Merge[{Association[x -> 1/2, y -> 2], Association[x -> 3/2, y -> -2]}, Total]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, 2], Rule[y, 0]]"
    );
}

#[test]
fn exact_denominator_and_apply_integer_reducers_match_mma_hol_certificate_shape() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Denominator /@ {1/2, 2/3, 4}")
            .unwrap()
            .value
            .to_string(),
        "List[2, 3, 1]"
    );
    assert_eq!(
        session
            .eval_text("Apply[LCM, Denominator /@ Values[Association[a -> 1/2, b -> 2/3, c -> 4]]]")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
    assert_eq!(
        session
            .eval_text("Apply[GCD, Values[Association[a -> 6, b -> 9, c -> 15]]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn unicode_letter_identifiers_load_as_ordinary_source_symbols() {
    let mut session = Session::new();
    let source = "Module[{\u{03C3} = <||>, \u{03B1}Ty = 7}, {\u{03C3}, \u{03B1}Ty}]";

    assert_eq!(
        session.eval_text(source).unwrap().value.to_string(),
        "List[Association[], 7]"
    );
}

#[test]
fn logical_and_comparison_syntax_lowers_to_existing_builtins() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("True && !False")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("False || True && False")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("1 === 1 && 1 =!= 2")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("0 <= 1 < 3").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("\"z\" <> ToString[3]")
            .unwrap()
            .value
            .to_string(),
        "\"z3\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"HOLError\", <|\"tag\" -> \"rule\"|>][[2, \"tag\"]]")
            .unwrap()
            .value
            .to_string(),
        "\"rule\""
    );
}
