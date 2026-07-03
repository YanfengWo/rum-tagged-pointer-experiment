use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn arithmetic_builtins_thread_over_lists() {
    let mut session = core_session();

    let cases = [
        ("Abs[{-1,-2,3}]", "List[1, 2, 3]"),
        ("GCD[{12,18},6]", "List[6, 6]"),
        ("LCM[{4,6},3]", "List[12, 6]"),
        ("Mod[{5,7},3]", "List[2, 1]"),
        ("Quotient[{17,9},5]", "List[3, 1]"),
        ("PowerMod[{2,3},10,7]", "List[2, 4]"),
        ("Numerator[{3/4,5/6}]", "List[3, 5]"),
        ("Denominator[{3/4,5/6}]", "List[4, 6]"),
        ("Factorial[{3,4}]", "List[6, 24]"),
        ("Factorial2[{4,5}]", "List[8, 15]"),
        ("CoprimeQ[{4,9},3]", "List[True, False]"),
        ("Round[{3/2,5/2,7/3}]", "List[2, 2, 2]"),
        ("Abs[-5]", "5"),
        ("GCD[12,18]", "6"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn hold_form_holds_argument_and_fullform_shows_wrapper() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Attributes[HoldForm]"),
        "List[HoldAll, Protected]"
    );
    assert_eq!(
        eval_to_string(&mut session, "FullForm[HoldForm[1 + 1]]"),
        "\"HoldForm[Plus[1, 1]]\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FullForm[HoldForm[1 + 1]] === \"HoldForm[2]\""
        ),
        "False"
    );
}

#[test]
fn fullform_evaluates_its_argument_but_preserves_inner_hold() {
    let mut session = core_session();

    assert_eq!(eval_to_string(&mut session, "FullForm[1+1]"), "\"2\"");
    assert_eq!(
        eval_to_string(&mut session, "FullForm[Hold[1+1]]"),
        "\"Hold[Plus[1, 1]]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Hold[Evaluate[1+1]] === Hold[2]"),
        "True"
    );
}

#[test]
fn head_length_and_depth_cover_atoms_calls_and_compound_heads() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("FullForm[1 + 2]")
            .unwrap()
            .value
            .to_string(),
        "\"3\""
    );
    assert_eq!(
        session.eval_text("Head[1 + 2]").unwrap().value.to_string(),
        "Integer"
    );
    assert_eq!(
        session
            .eval_text("Head[f[a], Hold]")
            .unwrap()
            .value
            .to_string(),
        "Hold[f]"
    );
    assert_eq!(
        session.eval_text("Length[5]").unwrap().value.to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("Length[f[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Attributes[LeafCount]")
            .unwrap()
            .value
            .to_string(),
        "List[Protected]"
    );
    assert_eq!(
        session
            .eval_text("LeafCount[f[x, y]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("LeafCount[1 + 2 x]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
    assert_eq!(
        session.eval_text("LeafCount[5]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("LeafCount[f[g[x], h[1, 2]]]")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
    assert_eq!(
        session.eval_text("Depth[5]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Depth[f[]]").unwrap().value.to_string(),
        "2"
    );
    assert_eq!(
        session.eval_text("Depth[{}]").unwrap().value.to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[a, g[b]]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[f[1] -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[], Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[a -> 1, b -> f[2]]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[a -> Association[b -> 1]], Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], {Heads -> True}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], \"Heads\" -> True]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], {Heads -> False, Heads -> True}]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], {}]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], {Heads -> True, Heads -> x}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    let unknown_option = session.eval_text("Depth[f[a], Bad -> 1]").unwrap();
    assert_eq!(
        unknown_option.value.to_string(),
        "Depth[f[a], Rule[Bad, 1]]"
    );
    assert!(
        unknown_option
            .messages
            .iter()
            .any(|message| message.symbol == "Depth" && message.tag == "optx")
    );
    let invalid_heads_value = session.eval_text("Depth[f[a], Heads -> x]").unwrap();
    assert_eq!(
        invalid_heads_value.value.to_string(),
        "Depth[f[a], Rule[Heads, x]]"
    );
    assert!(
        invalid_heads_value
            .messages
            .iter()
            .any(|message| message.symbol == "Depth" && message.tag == "opttf")
    );
    let non_option = session.eval_text("Depth[f[a], {1}]").unwrap();
    assert_eq!(non_option.value.to_string(), "Depth[f[a], List[1]]");
    assert!(
        non_option
            .messages
            .iter()
            .any(|message| message.symbol == "Depth" && message.tag == "nonopt")
    );
}

#[test]
fn first_last_rest_and_most_support_defaults_and_empty_calls() {
    let mut session = core_session();

    session.eval_text("n = 1").unwrap();
    assert_eq!(
        session
            .eval_text("First[f[a], n = 2]")
            .unwrap()
            .value
            .to_string(),
        "a"
    );
    assert_eq!(session.eval_text("n").unwrap().value.to_string(), "1");
    assert_eq!(
        session
            .eval_text("First[f[], n = 3]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(session.eval_text("n").unwrap().value.to_string(), "3");
    assert_eq!(
        session
            .eval_text("Last[f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "b"
    );
    assert_eq!(
        session.eval_text("Last[5, x]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Rest[f[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "f[b, c]"
    );
    assert_eq!(
        session
            .eval_text("Most[f[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("First[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Last[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Rest[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Most[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {First[Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 1]"
    );
}

#[test]
fn flatten_reports_current_invalid_boundaries() {
    let mut session = core_session();

    let atomic = session.eval_text("Flatten[x]").unwrap();
    assert_eq!(atomic.value.to_string(), "Flatten[x]");
    assert!(
        atomic
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "normal")
    );

    let association = session
        .eval_text("Flatten[Association[a -> Association[b -> 1], c -> 2]]")
        .unwrap();
    assert_eq!(
        association.value.to_string(),
        "Flatten[Association[Rule[a, Association[Rule[b, 1]]], Rule[c, 2]]]"
    );
    assert!(
        association
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "normal")
    );

    let invalid_level = session.eval_text("Flatten[{a}, x]").unwrap();
    assert_eq!(invalid_level.value.to_string(), "Flatten[List[a], x]");
    assert!(
        invalid_level
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "flev")
    );

    let invalid_level_with_head = session.eval_text("Flatten[{a}, x, List]").unwrap();
    assert_eq!(
        invalid_level_with_head.value.to_string(),
        "Flatten[List[a], x, List]"
    );
    assert!(
        invalid_level_with_head
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "flev")
    );

    let excess_level = session.eval_text("Flatten[{{a, b}, c}, {{2}}]").unwrap();
    assert_eq!(
        excess_level.value.to_string(),
        "Flatten[List[List[a, b], c], List[List[2]]]"
    );
    assert!(
        excess_level
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "fldep")
    );

    let repeated_level = session.eval_text("Flatten[{{a, b}}, {{1}, {1}}]").unwrap();
    assert_eq!(
        repeated_level.value.to_string(),
        "Flatten[List[List[a, b]], List[List[1], List[1]]]"
    );
    assert!(
        repeated_level
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "flrep")
    );

    let non_positive_level = session.eval_text("Flatten[{{a, b}}, {{0}}]").unwrap();
    assert_eq!(
        non_positive_level.value.to_string(),
        "Flatten[List[List[a, b]], List[List[0]]]"
    );
    assert!(
        non_positive_level
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "flpi")
    );

    let atomic_with_invalid_spec = session.eval_text("Flatten[x, y]").unwrap();
    assert_eq!(atomic_with_invalid_spec.value.to_string(), "Flatten[x, y]");
    assert!(
        atomic_with_invalid_spec
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "normal")
    );
}
