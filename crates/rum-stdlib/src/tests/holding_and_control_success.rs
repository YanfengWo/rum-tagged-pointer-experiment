use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn missingq_and_success_cover_current_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("MissingQ[Missing[]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MissingQ[Missing[\"KeyAbsent\"]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("MissingQ[1]").unwrap().value.to_string(),
        "False"
    );
    let malformed_failure = session.eval_text("Failure[\"oops\"]").unwrap();
    assert_eq!(malformed_failure.value.to_string(), "Failure[\"oops\"]");
    assert!(malformed_failure.messages.is_empty());
    let malformed_success = session.eval_text("Success[\"ok\"]").unwrap();
    assert_eq!(malformed_success.value.to_string(), "Success[\"ok\"]");
    assert!(malformed_success.messages.is_empty());
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200]]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", Association[Rule[\"Code\", 200]]]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "200"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[Code -> 200, Body -> 7]][Code]")
            .unwrap()
            .value
            .to_string(),
        "200"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][Code]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Code]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][{\"Body\", \"Missing\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"KeyAbsent\", \"Missing\"]]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[Code -> 200, Body -> 7]][{Body, Missing}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"KeyAbsent\", Missing]]"
    );
    assert_eq!(
        session
            .eval_text(
                "Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][\"Properties\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"Code\", \"Body\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200]][All]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", All]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][Properties]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Properties]"
    );
    assert_eq!(
        session
            .eval_text("Success[tag, Association[Code -> 200]][Code]")
            .unwrap()
            .value
            .to_string(),
        "Success[tag, Association[Rule[Code, 200]]][Code]"
    );
    assert_eq!(
        session
            .eval_text("Success[1, Association[Code -> 200]][Code]")
            .unwrap()
            .value
            .to_string(),
        "Success[1, Association[Rule[Code, 200]]][Code]"
    );
    assert_eq!(
        session
            .eval_text(
                "Success[\"ok\", Association[Properties -> 1, \"Properties\" -> 2]][\"Properties\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Properties, \"Properties\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {\"Code\" -> 200}][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", List[Rule[\"Code\", 200]]][\"Code\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", \"Code\" -> 200][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", Rule[\"Code\", 200]][\"Code\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", 1][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", 1][\"Properties\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", 1][All]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", 1][All]"
    );
}
