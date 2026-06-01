use super::*;

#[test]
fn normalize_rules_lists_single_rules_rule_lists_and_preserves_delayedness() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("NormalizeRules[x_Integer -> 1]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Pattern[x, Blank[Integer]], 1]]"
    );

    session
        .eval_text("rules = {x_Integer -> 1, y_String :> y}")
        .unwrap();

    assert_eq!(
        session
            .eval_text("NormalizeRules[rules]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Pattern[x, Blank[Integer]], 1], RuleDelayed[Pattern[y, Blank[String]], y]]"
    );
}

#[test]
fn inspect_rewrite_rules_reports_normalized_rules_and_delayed_flags() {
    let mut session = Session::new();
    let syntax = parse_expr("{x_Integer -> 1, y_String :> y}").unwrap();
    let expr = lower_syntax_expr(&syntax).unwrap();

    let rules = session.inspect_rewrite_rules(expr).unwrap().unwrap();

    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].lhs.to_string(), "Pattern[x, Blank[Integer]]");
    assert_eq!(rules[0].rhs.to_string(), "1");
    assert!(!rules[0].delayed);
    assert_eq!(rules[1].lhs.to_string(), "Pattern[y, Blank[String]]");
    assert_eq!(rules[1].rhs.to_string(), "y");
    assert!(rules[1].delayed);
}

#[test]
fn inspect_rewrite_rules_can_suspend_and_resume_with_host_input() {
    let mut session = Session::new();
    session
        .eval_text("ruleSource[\"Alice\"] := {x_Integer -> 1, y_String :> y}")
        .unwrap();
    session
        .eval_text("rules := ruleSource[InputString[\"name: \"]]")
        .unwrap();
    let syntax = parse_expr("rules").unwrap();
    let expr = lower_syntax_expr(&syntax).unwrap();

    let failure = session
        .inspect_rewrite_rules(expr)
        .expect_err("rewrite-rule inspection should suspend on InputString");

    match failure {
        EvalError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required inspection failure, got {other:?}"),
    }

    let rules = session
        .resume_inspect_rewrite_rules_with_input("Alice")
        .expect("resume should continue rewrite-rule inspection")
        .expect("rule source should produce normalized rules");

    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].lhs.to_string(), "Pattern[x, Blank[Integer]]");
    assert_eq!(rules[0].rhs.to_string(), "1");
    assert!(!rules[0].delayed);
    assert_eq!(rules[1].lhs.to_string(), "Pattern[y, Blank[String]]");
    assert_eq!(rules[1].rhs.to_string(), "y");
    assert!(rules[1].delayed);
}
