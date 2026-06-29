use super::*;

#[test]
fn configurable_replace_repeated_limit_is_respected() {
    let mut session = Session::new();
    session.set_replace_repeated_limit(8);

    let error = session
        .eval_text("1 //. {1 -> 2, 2 -> 1}")
        .expect_err("expected iteration limit error");

    assert!(matches!(
        error,
        EvalError::IterationLimitExceeded { limit: 8 }
    ));
    assert_eq!(session.resource_limits().replace_repeated_limit, 8);
}

#[test]
fn with_evaluates_bindings_and_substitutes_lexically() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("With[{x = Identity[1], y = x}, y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("With[{x = 1}, Hold[x]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[1]"
    );
}

#[test]
fn with_supports_rule_style_bindings_and_nested_shadowing() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("With[{x -> 1}, x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("With[{x = 1}, With[{x = 2}, x]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn scoping_builtins_accept_canonical_raw_binding_lists() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("With[List[Set[x, 1], Rule[y, x]], y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Module[List[Set[x, 1], Set[y, x]], y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Block[List[Set[x, 1], Rule[y, x]], y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
}

#[test]
fn with_respects_pattern_local_names_but_substitutes_pattern_metadata() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("With[{x = 1}, 7 /. x_Integer :> x]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("With[{t = Integer}, MatchQ[7, x_t]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn module_generates_fresh_lexical_symbols_and_initializes_sequentially() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Module[{x = 1, y = x}, y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );

    let first = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;
    let second = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;

    let first_text = first.to_string();
    let second_text = second.to_string();

    assert!(first_text.starts_with("Hold[x$"));
    assert!(second_text.starts_with("Hold[x$"));
    assert_ne!(first_text, second_text);
}

#[test]
fn module_local_assignment_invalidates_after_prior_read() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Module[{x}, x; x = 1; x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
}

#[test]
fn module_leaf_definition_gc_reclaims_loop_local_gensyms() {
    let mut session = Session::new();

    let before = eval_usize(&mut session, "MemoryInUse[]");
    session
        .eval_text("Do[Module[{x}, x = i^2; x], {i, 1, 2000}]")
        .unwrap();
    let after = eval_usize(&mut session, "MemoryInUse[]");
    let delta = after.saturating_sub(before);

    assert!(
        delta < 100_000,
        "expected Module leaf-definition GC to keep MemoryInUse delta small, got {delta}"
    );
}

#[test]
fn module_result_escape_retains_gensym_definition() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("held049 = Module[{x}, x = 7; Hold[x]]; First[held049]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn module_global_definition_escape_retains_gensym_definition() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Module[{x}, x = 7; gg049 := x; Null]; gg049")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn module_cross_gensym_definition_retains_transitive_dependency() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("held049 = Module[{a, b}, b = 5; a := b; Hold[a]]; First[held049]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
}

#[test]
fn module_options_and_sow_escapes_retain_gensym_definitions() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text(
                "Module[{x}, x = 7; Options[ggOpts049] = {o -> Hold[x]}; Null]; First[OptionValue[ggOpts049, o]]",
            )
            .unwrap()
            .value
            .to_string(),
        "7"
    );

    assert_eq!(
        session
            .eval_text("First[Part[Reap[Module[{x}, x = 7; Sow[Hold[x]]; Null]], 2, 1, 1]]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn module_nested_and_error_paths_remain_conservative() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Module[{r}, r = Module[{t}, t = 1; t]; r]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );

    assert_eq!(
        session
            .eval_text("First[Catch[Module[{x}, x = 7; Throw[Hold[x]]]]]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn module_respects_nested_shadowing_and_pattern_locals() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Module[{x = 1}, Module[{x = 2}, x]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    let nested = session
        .eval_text("Module[{x = 1}, Module[{x}, Hold[x]]]")
        .unwrap()
        .value
        .to_string();
    assert!(nested.starts_with("Hold[x$"));
    assert_ne!(nested, "Hold[x]");
    assert_ne!(nested, "Hold[1]");
    assert_eq!(
        session
            .eval_text("Module[{x = 1}, 7 /. x_Integer :> x]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

fn eval_usize(session: &mut Session, source: &str) -> usize {
    session
        .eval_text(source)
        .unwrap()
        .value
        .to_string()
        .parse()
        .unwrap()
}

#[test]
fn block_temporarily_rebinds_and_restores_symbol_values() {
    let mut session = Session::new();

    session.eval_text("x = 10").unwrap();

    assert_eq!(
        session
            .eval_text("Block[{x = 1}, x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
    assert_eq!(
        session
            .eval_text("Block[{x}, x]")
            .unwrap()
            .value
            .to_string(),
        "10"
    );
    assert_eq!(
        session
            .eval_text("Block[{x}, Hold[x]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[x]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
}

#[test]
fn block_applies_dynamic_scope_across_called_definitions() {
    let mut session = Session::new();

    session.eval_text("x = 10").unwrap();
    session.eval_text("f[] := x").unwrap();

    assert_eq!(
        session
            .eval_text("Block[{x = 1}, f[]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("f[]").unwrap().value.to_string(), "10");
    assert_eq!(
        session
            .eval_text("Block[{x}, x = 3]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
}

#[test]
fn block_supports_sequential_initializers_and_nested_shadowing() {
    let mut session = Session::new();

    session.eval_text("x = 10").unwrap();

    assert_eq!(
        session
            .eval_text("Block[{x = 1, y = x}, y]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Block[{x -> 1}, x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Block[{x = 1}, Block[{x = 2}, x]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
}

#[test]
fn compound_expression_evaluates_sequentially_and_returns_the_last_value() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("CompoundExpression[x = 1, y = x + 1, y]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("CompoundExpression[]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    assert_eq!(session.eval_text("y").unwrap().value.to_string(), "2");
}

#[test]
fn switch_matches_against_held_forms_and_residualizes_when_nothing_matches() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Switch[1 + 1, _String, no, _Integer, yes]")
            .unwrap()
            .value
            .to_string(),
        "yes"
    );
    assert_eq!(
        session
            .eval_text("Switch[1 + 1, _String, no]")
            .unwrap()
            .value
            .to_string(),
        "Switch[2, Blank[String], no]"
    );
}

#[test]
fn catch_throw_and_return_support_first_pass_nonlocal_control_flow() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Catch[Throw[1]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Catch[Throw[1, tag], tag, List]")
            .unwrap()
            .value
            .to_string(),
        "List[1, tag]"
    );
    assert_eq!(
        session
            .eval_text("Function[x, CompoundExpression[0, Return[x + 1], 9]][2]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session.eval_text("Return[7]").unwrap().value.to_string(),
        "7"
    );
}

#[test]
fn return_is_consumed_at_definition_application_boundaries() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ClearAll[f, g]; g[] := Return[1]; f[] := (g[]; 2); {g[], f[]}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("ClearAll[f, h]; h := Return[1]; f[] := (h; 2); {h, f[]}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("ClearAll[a, f, g]; a /: f[a] := Return[5]; g[] := (f[a]; 6); {f[a], g[]}")
            .unwrap()
            .value
            .to_string(),
        "List[5, 6]"
    );
    assert_eq!(
        session
            .eval_text(
                "ClearAll[f, g]; f[x_][y_] := Return[7]; g[] := (f[1][2]; 8); {f[1][2], g[]}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[7, 8]"
    );
}

#[test]
fn throw_through_block_restores_dynamic_scope_before_catch_handles_it() {
    let mut session = Session::new();

    session.eval_text("x = 10").unwrap();

    assert_eq!(
        session
            .eval_text("Catch[Block[{x = 1}, Throw[x, tag]], tag]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
}

#[test]
fn scoping_builtins_require_raw_binding_lists_instead_of_resolved_variables() {
    let mut session = Session::new();

    session.eval_text("bindings = {x = 1}").unwrap();

    let with_outcome = session.eval_text("With[bindings, x]").unwrap();
    assert_eq!(with_outcome.value.to_string(), "With[bindings, x]");
    assert!(
        with_outcome
            .messages
            .iter()
            .any(|message| message.symbol == "With" && message.tag == "argform")
    );

    let module_outcome = session.eval_text("Module[bindings, x]").unwrap();
    assert_eq!(module_outcome.value.to_string(), "Module[bindings, x]");
    assert!(
        module_outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Module" && message.tag == "argform")
    );

    let block_outcome = session.eval_text("Block[bindings, x]").unwrap();
    assert_eq!(block_outcome.value.to_string(), "Block[bindings, x]");
    assert!(
        block_outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Block" && message.tag == "argform")
    );
}
