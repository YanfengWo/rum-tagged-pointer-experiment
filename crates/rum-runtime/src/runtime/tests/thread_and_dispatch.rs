use super::*;

#[test]
fn evaluated_head_function_attributes_control_argument_evaluation() {
    let mut session = Session::new();

    session
        .eval_text("gate = Function[Hold[Slot[1]], HoldAll]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("gate[Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Identity[1]]"
    );
}

#[test]
fn block_localizes_definitions_attributes_and_messages() {
    let mut session = Session::new();

    session.eval_text("f[x_] := x").unwrap();
    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();
    session.eval_text("f::tag = \"outer\"").unwrap();

    assert_eq!(
        session
            .eval_text("Block[{f}, {Attributes[f], MessageDefinitions[f], f[Identity[1]]}]",)
            .unwrap()
            .value
            .to_string(),
        "List[List[], List[], 1]"
    );

    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst]"
    );
    assert_eq!(
        session
            .eval_text("MessageDefinitions[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[f::tag, \"outer\"]]"
    );
    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "1");
}

#[test]
fn module_skips_existing_generated_symbol_names() {
    let mut session = Session::new();

    session.eval_text("x$1 = occupied").unwrap();
    session.eval_text("x$2 = occupied").unwrap();

    let value = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;
    assert_eq!(value.to_string(), "Hold[x$3]");
}

#[test]
fn orderless_attribute_canonicalizes_calls_and_rules() {
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, Orderless]").unwrap();

    assert_eq!(
        session.eval_text("f[b, a, c]").unwrap().value.to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[b, a], f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("f[2, 1] /. f[a_, b_] -> {a, b}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
}

#[test]
fn held_downvalue_matching_normalizes_nested_attribute_heads() {
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, HoldAll]").unwrap();
    session.eval_text("SetAttributes[h, Orderless]").unwrap();
    session.eval_text("f[h[a_, b_]] := {a, b}").unwrap();

    assert_eq!(
        session.eval_text("f[h[2, 1]]").unwrap().value.to_string(),
        "List[1, 2]"
    );
}

#[test]
fn stable_call_shortcut_preserves_symbol_and_upvalue_evaluation() {
    let mut session = Session::new();

    session.eval_text("x = 1").unwrap();
    assert_eq!(
        session.eval_text("inert[x]").unwrap().value.to_string(),
        "inert[1]"
    );

    session.eval_text("u /: inert[u] := hit").unwrap();
    assert_eq!(
        session.eval_text("inert[u]").unwrap().value.to_string(),
        "hit"
    );
}

#[test]
fn downvalue_miss_stability_shortcut_invalidates_on_later_definition() {
    let mut session = Session::new();

    session.eval_text("f[1] = 2").unwrap();
    assert_eq!(session.eval_text("f[2]").unwrap().value.to_string(), "f[2]");

    session.eval_text("f[2] = 3").unwrap();
    assert_eq!(session.eval_text("f[2]").unwrap().value.to_string(), "3");
}

#[test]
fn dispatch_stage_guards_preserve_value_and_builtin_semantics() {
    let mut session = Session::new();

    assert_eq!(session.eval_text("k[1]").unwrap().value.to_string(), "k[1]");
    session.eval_text("k[1] = 9").unwrap();
    assert_eq!(session.eval_text("k[1]").unwrap().value.to_string(), "9");

    session.eval_text("m /: f[m] := ok").unwrap();
    assert_eq!(session.eval_text("f[m]").unwrap().value.to_string(), "ok");

    session.eval_text("d[1][2] := 12").unwrap();
    assert_eq!(
        session.eval_text("d[1][2]").unwrap().value.to_string(),
        "12"
    );

    assert_eq!(
        session.eval_text("SameQ[1, 1]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("If[True, 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Length[{1, 2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn argument_preparation_cow_preserves_dispatch_semantics() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("g[1 + 1]").unwrap().value.to_string(),
        "g[2]"
    );

    session.eval_text("x = 5").unwrap();
    assert_eq!(session.eval_text("h[x]").unwrap().value.to_string(), "h[5]");

    assert_eq!(
        session.eval_text("Hold[1 + 1]").unwrap().value.to_string(),
        "Hold[Plus[1, 1]]"
    );
    assert_eq!(
        session
            .eval_text("If[True, a, b]")
            .unwrap()
            .value
            .to_string(),
        "a"
    );
    assert_eq!(
        session
            .eval_text("f[Sequence[1, 2], 3]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 3]"
    );

    session.eval_text("SetAttributes[ff, Orderless]").unwrap();
    assert_eq!(
        session.eval_text("ff[2, 1]").unwrap().value.to_string(),
        "ff[1, 2]"
    );

    session.eval_text("SetAttributes[flatf, Flat]").unwrap();
    assert_eq!(
        session
            .eval_text("flatf[a, flatf[b, c], d]")
            .unwrap()
            .value
            .to_string(),
        "flatf[a, b, c, d]"
    );

    let failure = session
        .eval_request("wrap[InputString[\"arg: \"], Identity[2]]")
        .expect_err("argument evaluation should suspend on InputString");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "arg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("argument evaluation should resume and finish the call");
    assert_eq!(response.value.to_string(), "wrap[\"done\", 2]");
}

#[test]
fn flat_attribute_flattens_calls_and_matches_flat_patterns() {
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, Flat]").unwrap();

    assert_eq!(
        session
            .eval_text("f[a, f[b, c], d]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c, d]"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[a, f[b, c]], f[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("f[a, f[b, c]] /. f[x_, y_, z_] -> {x, y, z}")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
}

#[test]
fn one_identity_allows_singleton_head_patterns_to_match_atoms() {
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, OneIdentity]").unwrap();

    assert_eq!(
        session
            .eval_text("MatchQ[7, f[x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("7 /. f[x_Integer] -> x")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session.eval_text("Plus[b, a]").unwrap().value.to_string(),
        "Plus[a, b]"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[7, Plus[x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn listable_threads_user_defined_calls_and_broadcasts_scalars() {
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, Listable]").unwrap();

    assert_eq!(
        session.eval_text("f[{a, b}, c]").unwrap().value.to_string(),
        "List[f[a, c], f[b, c]]"
    );
    assert_eq!(
        session
            .eval_text("f[{a, b}, {c, d}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, c], f[b, d]]"
    );
    assert_eq!(
        session.eval_text("f[{}, x]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("f[{{a, b}, {c, d}}, x]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[a, x], f[b, x]], List[f[c, x], f[d, x]]]"
    );
}

#[test]
fn listable_mismatched_lengths_emit_thread_message_and_leave_call_unevaluated() {
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, Listable]").unwrap();

    let outcome = session.eval_text("f[{a, b}, {c}]").unwrap();

    assert_eq!(outcome.value.to_string(), "f[List[a, b], List[c]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Thread" && message.tag == "tdlen")
    );
    assert_eq!(
        session
            .eval_text("Check[f[{a, b}, {c}], fallback, Thread]")
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );
}

#[test]
fn listable_interacts_with_builtin_and_function_attributes() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Plus[{b, a}, c]")
            .unwrap()
            .value
            .to_string(),
        "List[Plus[b, c], Plus[a, c]]"
    );
    assert_eq!(
        session
            .eval_text("Function[f[Slot[1]], Listable][{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a], f[b]]"
    );
}

#[test]
fn thread_builtin_supports_default_and_custom_thread_heads() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Thread[f[{a, b}, c]]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, c], f[b, c]]"
    );
    assert_eq!(
        session
            .eval_text("Thread[f[g[a, b], g[c, d]], g]")
            .unwrap()
            .value
            .to_string(),
        "g[f[a, c], f[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("Thread[f[g[a, b], c], g]")
            .unwrap()
            .value
            .to_string(),
        "g[f[a, c], f[b, c]]"
    );
}

#[test]
fn thread_builtin_supports_standard_sequence_spec_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Thread[f[{a, b}, {c, d}, {e, f}], List, All]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, c, e], f[b, d, f]]"
    );
    assert_eq!(
        session
            .eval_text("Thread[f[{a, b}, {c, d}], List, None]")
            .unwrap()
            .value
            .to_string(),
        "f[List[a, b], List[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("Thread[f[{a, b}, {c, d}, {e, f}], List, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, c, List[e, f]], f[b, d, List[e, f]]]"
    );
    assert_eq!(
        session
            .eval_text("Thread[f[{a, b}, {c, d}, {e, f}], List, -2]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, b], c, e], f[List[a, b], d, f]]"
    );
    assert_eq!(
        session
            .eval_text("Thread[f[{a, b}, {c, d}, {e, f}], List, {2}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, b], c, List[e, f]], f[List[a, b], d, List[e, f]]]"
    );
    assert_eq!(
        session
            .eval_text("Thread[f[{a, b}, {c, d}, {e, f}, {g, h}], List, {2, 4, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, b], c, List[e, f], g], f[List[a, b], d, List[e, f], h]]"
    );
}

#[test]
fn thread_builtin_accepts_resolved_head_and_spec_values() {
    let mut session = Session::new();

    session.eval_text("threadHead = g").unwrap();
    session.eval_text("threadSpec = {2}").unwrap();

    assert_eq!(
        session
            .eval_text("Thread[f[g[a, b], g[c, d]], threadHead]")
            .unwrap()
            .value
            .to_string(),
        "g[f[a, c], f[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("Thread[f[{a, b}, {c, d}, {e, f}], List, threadSpec]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, b], c, List[e, f]], f[List[a, b], d, List[e, f]]]"
    );
}

#[test]
fn thread_evaluates_its_first_argument_before_threading() {
    let mut session = Session::new();

    session.eval_text("x = f[{a, b}, c]").unwrap();

    assert_eq!(
        session.eval_text("Thread[x]").unwrap().value.to_string(),
        "List[f[a, c], f[b, c]]"
    );
}
