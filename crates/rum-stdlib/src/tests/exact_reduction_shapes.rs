use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn exact_aggregation_shapes_reduce_arithmetic_terms() {
    let mut session = core_session();

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
    assert_eq!(
        session
            .eval_text("Mean[{1, 2, 1/2}]")
            .unwrap()
            .value
            .to_string(),
        "7/6"
    );
    assert_eq!(
        session
            .eval_text("Mean[{2, 4, 6, 8}]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
}

#[test]
fn mean_reports_empty_and_invalid_arity_boundaries() {
    let mut session = core_session();

    let empty = session.eval_text("Mean[{}]").unwrap();
    assert_eq!(empty.value.to_string(), "Mean[List[]]");
    assert!(
        empty
            .messages
            .iter()
            .any(|message| message.symbol == "Mean" && message.tag == "empty")
    );

    let invalid_arity = session.eval_text("Mean[]").unwrap();
    assert_eq!(invalid_arity.value.to_string(), "Mean[]");
    assert!(
        invalid_arity
            .messages
            .iter()
            .any(|message| message.symbol == "Mean" && message.tag == "argx")
    );
}

#[test]
fn exact_statistics_cover_median_commonest_and_quantile_contracts() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Median[{1, 2, 3, 4}]")
            .unwrap()
            .value
            .to_string(),
        "5/2"
    );
    assert_eq!(
        session
            .eval_text("Median[{3, 1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Commonest[{a, b, a, c, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Quantile[Range[10], 1/4]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Quantile[{5, 7}, 0]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
}

#[test]
fn commonest_accepts_count_limit_form() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Commonest[{a, b, a, c, b, a}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Commonest[{a, b, a, c, b, a}, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[a]"
    );
    assert_eq!(
        session
            .eval_text("Commonest[{a, b, a, c, b, a}, 10]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Commonest[{a, b, a, c, b, a}, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Commonest[{a, b, a, c, b, a}]")
            .unwrap()
            .value
            .to_string(),
        "List[a]"
    );
}

#[test]
fn quantile_accepts_parameter_lists_and_fails_closed() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Quantile[{1, 2, 3}, {0, 1/2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Quantile[{1, 2, 3}, 1/2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );

    let bad_quantile = session.eval_text("Quantile[{1, 2, 3}, {1/2, 5}]").unwrap();
    assert_eq!(
        bad_quantile.value.to_string(),
        "Quantile[List[1, 2, 3], List[1/2, 5]]"
    );
    assert!(
        bad_quantile
            .messages
            .iter()
            .any(|message| message.symbol == "Quantile" && message.tag == "range")
    );

    let empty_quantile = session.eval_text("Quantile[{}, {1/2}]").unwrap();
    assert_eq!(
        empty_quantile.value.to_string(),
        "Quantile[List[], List[1/2]]"
    );
    assert!(
        empty_quantile
            .messages
            .iter()
            .any(|message| message.symbol == "Quantile" && message.tag == "empty")
    );
}

#[test]
fn exact_statistics_report_empty_and_invalid_boundaries() {
    let mut session = core_session();

    let empty_median = session.eval_text("Median[{}]").unwrap();
    assert_eq!(empty_median.value.to_string(), "Median[List[]]");
    assert!(
        empty_median
            .messages
            .iter()
            .any(|message| message.symbol == "Median" && message.tag == "empty")
    );

    let empty_quantile = session.eval_text("Quantile[{}, 1/2]").unwrap();
    assert_eq!(empty_quantile.value.to_string(), "Quantile[List[], 1/2]");
    assert!(
        empty_quantile
            .messages
            .iter()
            .any(|message| message.symbol == "Quantile" && message.tag == "empty")
    );

    let bad_quantile = session.eval_text("Quantile[{1, 2}, 3/2]").unwrap();
    assert_eq!(bad_quantile.value.to_string(), "Quantile[List[1, 2], 3/2]");
    assert!(
        bad_quantile
            .messages
            .iter()
            .any(|message| message.symbol == "Quantile" && message.tag == "range")
    );
}

#[test]
fn exact_denominator_and_apply_integer_reducers_keep_certificate_shape() {
    let mut session = core_session();

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
    let mut session = core_session();
    let source = "Module[{\u{03C3} = <||>, \u{03B1}Ty = 7}, {\u{03C3}, \u{03B1}Ty}]";

    assert_eq!(
        session.eval_text(source).unwrap().value.to_string(),
        "List[Association[], 7]"
    );
}
