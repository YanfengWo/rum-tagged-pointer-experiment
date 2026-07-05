use super::super::*;

// Moved from the runtime cold-lane tests when these checks moved under stdlib.

#[test]
fn logical_and_comparison_syntax_lowers_to_existing_builtins() {
    let mut session = core_session();

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
