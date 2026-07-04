use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn default_pattern_syntax_lowers_to_optional_patterns() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("FullForm[Hold[x_:9]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[Optional[Pattern[x, Blank[]], 9]]\""
    );
    assert_eq!(
        session
            .eval_text("FullForm[Hold[_:9]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[Optional[Blank[], 9]]\""
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1], f[1, x_:9]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("g[1, x_:9] := {1, x}; g[1]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 9]"
    );
    assert_eq!(
        session
            .eval_text("FullForm[Hold[x:_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[Pattern[x, Blank[Integer]]]\""
    );
    assert_eq!(
        session
            .eval_text("FullForm[Hold[x:y]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[Pattern[x, y]]\""
    );
}

#[test]
fn single_dot_default_pattern_syntax_lowers_to_optional() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("FullForm[Hold[x_.]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[Optional[Pattern[x, Blank[]]]]\""
    );
    assert_eq!(
        session
            .eval_text("FullForm[Hold[_.]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[Optional[Blank[]]]\""
    );
    assert_eq!(
        session
            .eval_text("Default[ff] = 0; ff[1, x_.] := {1, x}; ff[1]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 0]"
    );
    assert_eq!(
        session
            .eval_text("FullForm[Hold[x_..]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[Repeated[Pattern[x, Blank[]]]]\""
    );
    assert_eq!(
        session
            .eval_text("FullForm[Hold[x_...]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[RepeatedNull[Pattern[x, Blank[]]]]\""
    );
    assert_eq!(
        session
            .eval_text("FullForm[Hold[1.5]]")
            .unwrap()
            .value
            .to_string(),
        "\"Hold[3/2]\""
    );
}
