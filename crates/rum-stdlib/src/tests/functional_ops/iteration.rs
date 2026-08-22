use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn nest_and_fixed_point_families_cover_current_iteration_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Nest[f, x, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[f[x]]"
    );
    assert_eq!(
        session
            .eval_text("Nest[f, 2][x]")
            .unwrap()
            .value
            .to_string(),
        "f[f[x]]"
    );
    assert_eq!(
        session
            .eval_text("NestList[f, x, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[x, f[x], f[f[x]]]"
    );
    assert_eq!(
        session
            .eval_text("NestList[f, 2][x]")
            .unwrap()
            .value
            .to_string(),
        "List[x, f[x], f[f[x]]]"
    );

    session.eval_text("step[0] = 1").unwrap();
    session.eval_text("step[2] = 1").unwrap();
    session.eval_text("step[1] = 1").unwrap();
    session.eval_text("test1[2] = True").unwrap();
    session.eval_text("test1[1] = False").unwrap();

    assert_eq!(
        session
            .eval_text("NestWhile[step, 0, Function[Unequal[Slot[1], 1]]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("NestWhileList[step, 2, UnsameQ, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, 1]"
    );
    assert_eq!(
        session
            .eval_text("NestWhileList[step, test1][2]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1]"
    );
    assert_eq!(
        session
            .eval_text("FixedPoint[Identity, f[x]]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("FixedPoint[step, 2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("FixedPoint[step][2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("FixedPoint[step, 2, 0]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("FixedPointList[Identity, x]")
            .unwrap()
            .value
            .to_string(),
        "List[x, x]"
    );
    assert_eq!(
        session
            .eval_text("FixedPointList[step, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, 1]"
    );
    assert_eq!(
        session
            .eval_text("FixedPointList[step, 2, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
}

#[test]
fn iteration_lists_preserve_wide_lengths_and_callback_counts() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "fixedCalls = 0; result = FixedPointList[Function[x, CompoundExpression[Increment[fixedCalls], x + 1]], 0, 128]; {Length[result], Last[result], fixedCalls}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[129, 128, 128]"
    );
    assert_eq!(
        session
            .eval_text(
                "stepCalls = 0; testCalls = 0; result = NestWhileList[Function[x, CompoundExpression[Increment[stepCalls], x + 1]], 0, Function[x, CompoundExpression[Increment[testCalls], x < 128]], 1, 128]; {Length[result], Last[result], stepCalls, testCalls}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[129, 128, 128, 129]"
    );
    assert_eq!(
        session
            .eval_text(
                "nestCalls = 0; result = NestList[Function[x, CompoundExpression[Increment[nestCalls], x + 1]], 0, 128]; {Length[result], Last[result], nestCalls}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[129, 128, 128]"
    );
}

#[test]
fn fixed_point_cycle_tracking_covers_value_and_list_results() {
    let mut session = core_session();

    for head in ["FixedPoint", "FixedPointList"] {
        let outcome = session
            .eval_text(&format!("{head}[Function[x, 1 - x], 0]"))
            .unwrap();
        assert_eq!(
            outcome.value.to_string(),
            "TerminatedEvaluation[\"IterationLimit\"]"
        );
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == "General"
                && message.tag == "passlim"
                && message.text.contains("evaluation cycle detected")
        }));
    }

    assert_eq!(
        session
            .eval_text(
                "{FixedPoint[Function[x, 1 - x], 0, 3], FixedPointList[Function[x, 1 - x], 0, 3]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[1, List[0, 1, 0, 1]]"
    );
}

#[test]
fn iteration_builtins_report_current_argument_forms() {
    let mut session = core_session();

    let nest = session.eval_text("Nest[f, expr, bad]").unwrap();
    assert_eq!(nest.value.to_string(), "Nest[f, expr, bad]");
    assert!(
        nest.messages
            .iter()
            .any(|message| message.symbol == "Nest" && message.tag == "argform")
    );

    let nest_while = session.eval_text("NestWhile[f, expr, test, 0]").unwrap();
    assert_eq!(nest_while.value.to_string(), "NestWhile[f, expr, test, 0]");
    assert!(
        nest_while
            .messages
            .iter()
            .any(|message| message.symbol == "NestWhile" && message.tag == "argform")
    );

    let fixed_point = session.eval_text("FixedPoint[f, expr, bad]").unwrap();
    assert_eq!(fixed_point.value.to_string(), "FixedPoint[f, expr, bad]");
    assert!(
        fixed_point
            .messages
            .iter()
            .any(|message| message.symbol == "FixedPoint" && message.tag == "argform")
    );

    let fixed_point_list = session.eval_text("FixedPointList[f, expr, bad]").unwrap();
    assert_eq!(
        fixed_point_list.value.to_string(),
        "FixedPointList[f, expr, bad]"
    );
    assert!(
        fixed_point_list
            .messages
            .iter()
            .any(|message| message.symbol == "FixedPointList" && message.tag == "argform")
    );
}
