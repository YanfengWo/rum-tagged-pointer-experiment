use super::super::*;
use super::common::*;

#[test]
fn host_eval_exposes_typed_value_variants() {
    let mut session = HostSession::new();
    let cases = vec![
        ("2 + 3", "5", HostTyped::Integer(5)),
        (
            "2^200",
            TWO_TO_200,
            HostTyped::BigInteger(TWO_TO_200.to_string()),
        ),
        (
            "1/3",
            "1/3",
            HostTyped::Rational {
                numerator: "1".to_string(),
                denominator: "3".to_string(),
            },
        ),
        ("True", "True", HostTyped::Bool(true)),
        ("False", "False", HostTyped::Bool(false)),
        ("\"hi\"", "\"hi\"", HostTyped::Str("hi".to_string())),
        (
            "{1, 2, 3}",
            "List[1, 2, 3]",
            HostTyped::List(vec![
                HostTyped::Integer(1),
                HostTyped::Integer(2),
                HostTyped::Integer(3),
            ]),
        ),
        (
            "f[1]",
            "f[1]",
            HostTyped::Other {
                head: "f".to_string(),
                text: "f[1]".to_string(),
            },
        ),
    ];

    for (input, expected_text, expected_typed) in cases {
        match session.eval(input) {
            HostEvalOutcome::Success(success) => {
                assert_eq!(success.value.text, expected_text, "input {input}");
                assert_eq!(success.value.typed, expected_typed, "input {input}");
                eprintln!(
                    "typed_value input={input:?} text={:?} typed={:?}",
                    success.value.text, success.value.typed
                );
            }
            other => panic!("expected success for {input}, got {other:?}"),
        }
    }
}

#[test]
fn host_eval_exposes_ordered_typed_association_entries() {
    let mut session = HostSession::new();

    let success = successful_eval(session.eval(r#"<|"a" -> 1, "b" -> {True}|>"#));

    assert_eq!(
        success.value.typed,
        HostTyped::Association(vec![
            (HostTyped::Str("a".to_string()), HostTyped::Integer(1)),
            (
                HostTyped::Str("b".to_string()),
                HostTyped::List(vec![HostTyped::Bool(true)]),
            ),
        ])
    );
}

#[test]
fn host_eval_keeps_delayed_association_out_of_typed_data_view() {
    let mut session = HostSession::new();

    let success = successful_eval(session.eval(r#"<|"a" :> 1|>"#));

    assert_eq!(
        success.value.typed,
        HostTyped::Other {
            head: "Association".to_string(),
            text: "Association[RuleDelayed[\"a\", 1]]".to_string(),
        }
    );
}

#[test]
fn host_eval_exposes_immediate_rule_and_rejects_delayed_rule_data_view() {
    let mut session = HostSession::new();

    let immediate = successful_eval(session.eval("a -> b"));
    assert_eq!(
        immediate.value.typed,
        HostTyped::Rule {
            lhs: Box::new(HostTyped::Symbol("a".to_string())),
            rhs: Box::new(HostTyped::Symbol("b".to_string())),
        }
    );

    let delayed = successful_eval(session.eval("a :> b"));
    assert_eq!(
        delayed.value.typed,
        HostTyped::Other {
            head: "RuleDelayed".to_string(),
            text: "RuleDelayed[a, b]".to_string(),
        }
    );
}

#[test]
fn host_call_resolves_short_system_head_for_string_join() {
    let mut session = HostSession::new();
    let args = [
        HostTyped::Str("a".to_string()),
        HostTyped::Str("b".to_string()),
    ];

    let success = successful_eval(session.call("StringJoin", &args));

    assert_eq!(success.value.text, "\"ab\"");
    assert_eq!(success.value.typed, HostTyped::Str("ab".to_string()));
}

#[test]
fn host_call_evaluates_integer_arguments() {
    let mut session = HostSession::new();
    let args = [HostTyped::Integer(1), HostTyped::Integer(2)];

    let success = successful_eval(session.call("Plus", &args));

    assert_eq!(success.value.typed, HostTyped::Integer(3));
}

#[test]
fn host_call_treats_string_arguments_as_injection_safe_data() {
    let mut session = HostSession::new();
    let payload = "\"], Print[666], x = \"\nsecond line";
    let args = [HostTyped::Str(payload.to_string())];

    let success = successful_eval(session.call("Identity", &args));

    assert_eq!(success.value.typed, HostTyped::Str(payload.to_string()));
    assert!(success.prints.is_empty());
    let x = successful_eval(session.eval("x"));
    assert_eq!(x.value.typed, HostTyped::Symbol("x".to_string()));
}

#[test]
fn host_call_round_trips_typed_association() {
    let mut session = HostSession::new();
    let association = HostTyped::Association(vec![
        (HostTyped::Str("a".to_string()), HostTyped::Integer(1)),
        (
            HostTyped::Str("b".to_string()),
            HostTyped::List(vec![HostTyped::Bool(true)]),
        ),
    ]);

    let success = successful_eval(session.call("Identity", std::slice::from_ref(&association)));

    assert_eq!(success.value.typed, association);
}

#[test]
fn host_call_resolves_short_and_fully_qualified_symbols() {
    let mut session = HostSession::new();
    successful_eval(session.eval("shortValue = 11; Global`qualifiedValue = 13"));

    let short =
        successful_eval(session.call("Identity", &[HostTyped::Symbol("shortValue".to_string())]));
    assert_eq!(short.value.typed, HostTyped::Integer(11));

    let qualified = successful_eval(session.call(
        "System`Identity",
        &[HostTyped::Symbol("Global`qualifiedValue".to_string())],
    ));
    assert_eq!(qualified.value.typed, HostTyped::Integer(13));

    let assigned = successful_eval(session.call(
        "Set",
        &[
            HostTyped::Symbol("assignedByCall".to_string()),
            HostTyped::Integer(7),
        ],
    ));
    assert_eq!(assigned.value.typed, HostTyped::Integer(7));
    assert_eq!(
        successful_eval(session.eval("assignedByCall")).value.typed,
        HostTyped::Integer(7)
    );
}

#[test]
fn host_call_round_trips_large_integer_and_immediate_rule() {
    let mut session = HostSession::new();
    let expected = HostTyped::List(vec![
        HostTyped::BigInteger(TWO_TO_200.to_string()),
        HostTyped::Rule {
            lhs: Box::new(HostTyped::Str("key".to_string())),
            rhs: Box::new(HostTyped::Bool(true)),
        },
    ]);
    let HostTyped::List(args) = &expected else {
        unreachable!("expected test list")
    };

    let success = successful_eval(session.call("List", args));

    assert_eq!(success.value.typed, expected);
}

#[test]
fn host_call_rejects_output_only_other_argument() {
    let mut session = HostSession::new();
    let failure = failed_eval(session.call(
        "Identity",
        &[HostTyped::Other {
            head: "f".to_string(),
            text: "f[1]".to_string(),
        }],
    ));

    assert_eq!(failure.code, HostFailureCode::Other);
    assert!(failure.message.contains("index 0"));
    assert!(failure.message.contains("output-only lossy view"));
}

#[test]
fn host_call_rejects_malformed_big_integer() {
    let mut session = HostSession::new();
    let failure = failed_eval(session.call(
        "Identity",
        &[HostTyped::BigInteger("not-an-integer".to_string())],
    ));

    assert_eq!(failure.code, HostFailureCode::Other);
    assert!(failure.message.contains("index 0"));
    assert!(failure.message.contains("invalid decimal BigInteger"));
}

#[test]
fn host_call_rejects_zero_denominator_rational() {
    let mut session = HostSession::new();
    let failure = failed_eval(session.call(
        "Identity",
        &[HostTyped::Rational {
            numerator: "1".to_string(),
            denominator: "0".to_string(),
        }],
    ));

    assert_eq!(failure.code, HostFailureCode::Other);
    assert!(failure.message.contains("index 0"));
    assert!(failure.message.contains("denominator must not be zero"));
}

#[test]
fn host_call_enforces_typed_input_depth_cap() {
    let mut session = HostSession::new();
    let mut at_cap = HostTyped::Integer(1);
    for _ in 0..64 {
        at_cap = HostTyped::List(vec![at_cap]);
    }

    let success = successful_eval(session.call("Identity", std::slice::from_ref(&at_cap)));
    assert_eq!(success.value.typed, at_cap);

    let over_cap = HostTyped::List(vec![at_cap]);
    let failure = failed_eval(session.call("Identity", &[over_cap]));
    assert_eq!(failure.code, HostFailureCode::Other);
    assert!(failure.message.contains("index 0"));
    assert!(failure.message.contains("maximum input depth of 64"));
}

#[test]
fn host_call_normalizes_rational_input() {
    let mut session = HostSession::new();
    let args = [HostTyped::Rational {
        numerator: "2".to_string(),
        denominator: "4".to_string(),
    }];

    let success = successful_eval(session.call("Identity", &args));

    assert_eq!(
        success.value.typed,
        HostTyped::Rational {
            numerator: "1".to_string(),
            denominator: "2".to_string(),
        }
    );
}

#[test]
fn host_call_rejects_empty_head() {
    let mut session = HostSession::new();

    let failure = failed_eval(session.call("  ", &[]));

    assert_eq!(failure.code, HostFailureCode::Other);
    assert_eq!(failure.message, "call head must not be empty");
}
