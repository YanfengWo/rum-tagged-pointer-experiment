use super::*;

#[test]
fn failure_property_accessors_cover_current_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500, \"Body\" -> 7}][\"Missing\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"Missing\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", \"Code\" -> 500][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "500"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", \"Code\" :> x][All]")
            .unwrap()
            .value
            .to_string(),
        "Association[RuleDelayed[\"Code\", x]]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", \"Code\" :> x][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" :> x}][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "Failure[\"oops\", List[RuleDelayed[\"Code\", x]]][\"Code\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][All]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"Code\", 500]]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][\"All\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"All\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500, \"Body\" -> 7}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Body\", \"Code\", \"Message\", \"StyledMessage\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Code -> 500, Body -> 7}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Message\", \"StyledMessage\", Body, Code]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"A failure of type \\\"oops\\\" occurred.\""
    );
    assert_eq!(
        session
            .eval_text("Failure[tag, {\"Code\" -> 500}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"An unknown failure occurred.\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][\"StyledMessage\"]")
            .unwrap()
            .value
            .to_string(),
        "Style[\"A failure of type \\\"oops\\\" occurred.\", \"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[tag, {\"Code\" -> 500}][\"StyledMessage\"]")
            .unwrap()
            .value
            .to_string(),
        "Style[\"An unknown failure occurred.\", \"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {}][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "\"oops\""
    );
    assert_eq!(
        session
            .eval_text("Failure[tag, {}][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "tag"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Tag\" -> override}][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "override"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Tag -> override}][Tag]")
            .unwrap()
            .value
            .to_string(),
        "override"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Tag -> override}][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "\"oops\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", 1][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "Failure[\"oops\", 1][\"Tag\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", 1][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "Failure[\"oops\", 1][\"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", 1][All]")
            .unwrap()
            .value
            .to_string(),
        "Failure[\"oops\", 1][All]"
    );
}
