use super::*;

#[test]
fn timing_and_absolute_timing_return_elapsed_and_value() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[Timing[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[Timing[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("Part[AbsoluteTiming[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[AbsoluteTiming[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("Part[RepeatedTiming[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[RepeatedTiming[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn time_and_memory_constraints_cover_current_first_pass_resource_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("TimeConstrained[Pause[1/100]; 7, 1/1000, timeout]")
            .unwrap()
            .value
            .to_string(),
        "timeout"
    );
    assert_eq!(
        session
            .eval_text("MemoryConstrained[Range[100], 20, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
    assert_eq!(
        session
            .eval_text("IntegerQ[MemoryInUse[]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn monitor_runs_current_display_and_callback_subset() {
    let mut session = Session::new();

    let outcome = session
        .eval_text(
            "Reap[Monitor[1 + 1, progress, {StepMonitor :> Sow[step], EvaluationMonitor :> Sow[eval]}]]",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[2, List[List[step, eval]]]");
    assert_eq!(outcome.prints, vec!["progress"]);

    let changing = session.eval_text("x = 0; Monitor[(x = 1; x), x]").unwrap();
    assert_eq!(changing.value.to_string(), "1");
    assert_eq!(changing.prints, vec!["0", "1"]);

    session
        .eval_text("SetOptions[Monitor, StepMonitor :> Sow[stepDefault]]")
        .unwrap();
    let defaulted = session.eval_text("Reap[Monitor[1 + 1, progress]]").unwrap();
    assert_eq!(
        defaulted.value.to_string(),
        "List[2, List[List[stepDefault]]]"
    );

    session
        .eval_text("SetOptions[Monitor, EvaluationMonitor :> Sow[evalDefault]]")
        .unwrap();
    let default_eval = session.eval_text("Reap[Monitor[1 + 1, progress]]").unwrap();
    assert_eq!(
        default_eval.value.to_string(),
        "List[2, List[List[stepDefault, evalDefault]]]"
    );

    let disabled_defaults = session
        .eval_text(
            "Reap[Monitor[1 + 1, progress, {StepMonitor :> None, EvaluationMonitor :> None}]]",
        )
        .unwrap();
    assert_eq!(disabled_defaults.value.to_string(), "List[2, List[]]");
}

#[test]
fn sow_and_reap_collect_by_tag_and_optional_handler() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Reap[Sow[1]]").unwrap().value.to_string(),
        "List[1, List[List[1]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], a]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[1]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], b]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], _, List]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[a, List[1]]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], {_Symbol}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[List[1]]]]"
    );
}
