use super::*;

#[test]
fn evaluate_forces_evaluation_inside_held_arguments() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Evaluate[Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Hold[Evaluate[Identity[1]]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[1]"
    );
}

#[test]
fn sequence_splices_in_non_held_calls_but_not_in_raw_held_ones() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("f[Sequence[1, 2], 3]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Sequence[1, Sequence[2, 3]]")
            .unwrap()
            .value
            .to_string(),
        "Sequence[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Hold[Sequence[1, 2], 3]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Sequence[1, 2], 3]"
    );
    assert_eq!(
        session
            .eval_text("Hold[Evaluate[Sequence[1, 2]], 3]")
            .unwrap()
            .value
            .to_string(),
        "Hold[1, 2, 3]"
    );
}
#[test]
fn setattributes_and_clearattributes_control_argument_evaluation() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("SetAttributes[f, HoldFirst]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("f[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "f[Identity[1], 2]"
    );

    assert_eq!(
        session
            .eval_text("ClearAttributes[f, HoldFirst]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("f[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2]"
    );
}
#[test]
fn setattributes_accepts_symbol_lists_and_combined_hold_attributes() {
    let mut session = Session::new();

    session
        .eval_text("SetAttributes[{f, g}, {HoldFirst, HoldRest}]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, HoldRest]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[g]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, HoldRest]"
    );
    assert_eq!(
        session
            .eval_text("g[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "g[Identity[1], Identity[2]]"
    );
}

#[test]
fn setattributes_accepts_resolved_attribute_specs() {
    let mut session = Session::new();

    session.eval_text("attrs = {HoldFirst, HoldRest}").unwrap();
    session.eval_text("SetAttributes[f, attrs]").unwrap();

    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, HoldRest]"
    );

    session.eval_text("ClearAttributes[f, attrs]").unwrap();

    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn protect_and_unprotect_round_trip_and_leave_attribute_updates_available() {
    let mut session = Session::new();

    let protected = session.eval_text("Protect[{f, \"g\"}]").unwrap();
    assert_eq!(protected.value.to_string(), "List[\"f\", \"g\"]");
    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Protected]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[g]")
            .unwrap()
            .value
            .to_string(),
        "List[Protected]"
    );

    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();
    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, Protected]"
    );

    let unprotected = session.eval_text("Unprotect[f]").unwrap();
    assert_eq!(unprotected.value.to_string(), "List[\"f\"]");
    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst]"
    );
}

#[test]
fn sequencehold_prevents_sequence_splicing() {
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, SequenceHold]").unwrap();

    assert_eq!(
        session
            .eval_text("f[Sequence[1, 2], 3]")
            .unwrap()
            .value
            .to_string(),
        "f[Sequence[1, 2], 3]"
    );
}

#[test]
fn holdallcomplete_prevents_evaluate_override_and_sequence_splicing() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text(
                "HoldAllComplete[Evaluate[Identity[1]], Unevaluated[Identity[2]], Sequence[3, 4]]",
            )
            .unwrap()
            .value
            .to_string(),
        "HoldAllComplete[Evaluate[Identity[1]], Unevaluated[Identity[2]], Sequence[3, 4]]"
    );
}

#[test]
fn holdallcomplete_blocks_upvalue_application() {
    let mut session = Session::new();

    session
        .eval_text("SetAttributes[gate, HoldAllComplete]")
        .unwrap();
    session.eval_text("f /: gate[f[x_Integer]] := x").unwrap();

    assert_eq!(
        session.eval_text("gate[f[7]]").unwrap().value.to_string(),
        "gate[f[7]]"
    );
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[RuleDelayed[gate[f[Pattern[x, Blank[Integer]]]], x]]"
    );
}

#[test]
fn user_defined_holdallcomplete_attribute_holds_all_arguments_completely() {
    let mut session = Session::new();

    session
        .eval_text("SetAttributes[f, HoldAllComplete]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("f[Evaluate[Identity[1]], Identity[2], Sequence[3, 4]]")
            .unwrap()
            .value
            .to_string(),
        "f[Evaluate[Identity[1]], Identity[2], Sequence[3, 4]]"
    );
}

#[test]
fn unevaluated_temporarily_preserves_direct_function_arguments() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Unevaluated[Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Hold[Unevaluated[Identity[1]]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Identity[1]]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[Unevaluated]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldAllComplete]"
    );
}

#[test]
fn holdpattern_preserves_patterns_without_prior_evaluation() {
    let mut session = Session::new();
    session.eval_text("f = 1").unwrap();

    assert_eq!(
        session
            .eval_text("MatchQ[Hold[f[7]], Hold[HoldPattern[f[x_Integer]]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Hold[f[7]] /. HoldPattern[f[x_Integer]] :> x")
            .unwrap()
            .value
            .to_string(),
        "Hold[7]"
    );
}

#[test]
fn releasehold_releases_supported_hold_wrappers() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReleaseHold[Hold[Identity[1]]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("ReleaseHold[HoldAllComplete[Identity[1]]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("ReleaseHold[HoldPattern[f[x_Integer]]]")
            .unwrap()
            .value
            .to_string(),
        "f[Pattern[x, Blank[Integer]]]"
    );
}

#[test]
fn if_evaluates_only_the_selected_branch() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("If[True, Identity[1], x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("If[False, x, Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session.eval_text("If[False, x]").unwrap().value.to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("If[maybe, Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "If[maybe, Identity[1], Identity[2]]"
    );
    assert_eq!(
        session
            .eval_text("If[maybe, x, y, Identity[3]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn which_evaluates_conditions_in_order() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Which[False, x, True, Identity[2], True, Identity[3]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Which[False, x, False, y]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("Which[maybe, Identity[1], True, Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "Which[maybe, Identity[1], True, Identity[2]]"
    );
}
