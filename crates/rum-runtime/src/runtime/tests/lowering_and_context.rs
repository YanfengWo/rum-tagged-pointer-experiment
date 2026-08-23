use super::*;
use rum_syntax::Span;

#[test]
fn lowers_big_integer_into_core_expr() {
    let lowered = lower_syntax_expr(&SyntaxExpr::integer(
        "123456789012345678901234567890",
        Span::default(),
    ))
    .unwrap();

    assert_eq!(lowered.to_string(), "123456789012345678901234567890");
}

#[test]
fn lowers_simple_decimal_literals_to_exact_rationals() {
    let lowered = lower_syntax_expr(&parse_expr("0.01").unwrap()).unwrap();

    assert_eq!(lowered.to_string(), "1/100");

    let lowered = lower_syntax_expr(&parse_expr(".5").unwrap()).unwrap();
    assert_eq!(lowered.to_string(), "1/2");

    let lowered = lower_syntax_expr(&parse_expr("1.").unwrap()).unwrap();
    assert_eq!(lowered.to_string(), "1");

    let lowered = lower_syntax_expr(&parse_expr("1.5e3").unwrap()).unwrap();
    assert_eq!(lowered.to_string(), "1500");
}

#[test]
fn lowering_rejects_invalid_integer_literals_before_runtime() {
    let error = lower_syntax_expr(&SyntaxExpr::integer("not-a-number", Span::default()))
        .expect_err("invalid integer syntax atoms should fail during lowering");

    assert!(matches!(
        error,
        EvalError::InvalidInteger { ref literal } if literal == "not-a-number"
    ));
}

#[test]
fn lowers_message_name_syntax_into_core_expr() {
    let syntax = parse_expr("FindFile::notfound").unwrap();
    let lowered = lower_syntax_expr(&syntax).unwrap();

    assert_eq!(lowered.to_string(), "FindFile::notfound");
}

#[test]
fn lowers_multiline_string_literals_like_ordinary_strings() {
    let escaped = lower_syntax_expr(&parse_expr("\"\"\"\nab\"\"\"").unwrap()).unwrap();
    let escaped_equivalent = lower_syntax_expr(&parse_expr("\"ab\"").unwrap()).unwrap();
    assert_eq!(escaped, escaped_equivalent);

    let raw = lower_syntax_expr(&parse_expr("'''\nC:\\path \\t raw\n'''").unwrap()).unwrap();
    let raw_equivalent =
        lower_syntax_expr(&parse_expr("\"C:\\\\path \\\\t raw\\n\"").unwrap()).unwrap();
    assert_eq!(raw, raw_equivalent);
}
#[test]
fn lowers_tagset_and_unset_syntax_into_core_expr() {
    let tagged_assignment = parse_expr("f /: g[x_] := rhs").unwrap();
    let lowered_assignment = lower_syntax_expr(&tagged_assignment).unwrap();

    assert_eq!(
        lowered_assignment.to_string(),
        "TagSetDelayed[f, g[Pattern[x, Blank[]]], rhs]"
    );

    let tagged_unset = parse_expr("f /: g[x_] =.").unwrap();
    let lowered_unset = lower_syntax_expr(&tagged_unset).unwrap();

    assert_eq!(
        lowered_unset.to_string(),
        "TagUnset[f, g[Pattern[x, Blank[]]]]"
    );
}

#[test]
fn lowering_preserves_tagset_shape_for_runtime_semantic_validation() {
    let syntax = parse_expr("f /: g[h[f[x_]]] := 1").unwrap();
    let lowered = lower_syntax_expr(&syntax).unwrap();

    assert_eq!(
        lowered.to_string(),
        "TagSetDelayed[f, g[h[f[Pattern[x, Blank[]]]]], 1]"
    );

    let mut session = session_with_test_control_builtins();
    let outcome = session.eval_text("f /: g[h[f[x_]]] := 1").unwrap();

    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "TagSetDelayed" && message.tag == "tagpos")
    );
}

#[test]
fn evaluates_identity_call() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text("Identity[123456789012345678901234567890]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "123456789012345678901234567890");
}

#[test]
fn compound_expression_sugar_flows_through_lowering_and_evaluation() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("x = 1; y = x + 1; y")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn long_statement_chain_lowers_to_flat_nary_compound_expression() {
    // brief-301: the `;` spine lowers iteratively, but the output stays the
    // flat n-ary CompoundExpression the recursive form produced — correct
    // arity, elements in source order.
    let count = 200;
    let mut input = String::new();
    let mut expected = String::from("CompoundExpression[");
    for i in 0..count {
        if i > 0 {
            input.push_str("; ");
            expected.push_str(", ");
        }
        input.push_str(&i.to_string());
        expected.push_str(&i.to_string());
    }
    expected.push(']');

    let lowered = lower_syntax_expr(&parse_expr(&input).unwrap()).unwrap();

    assert_eq!(lowered.to_string(), expected);
}

#[test]
fn over_cap_statement_chain_line_does_not_swallow_later_lines() {
    // brief-302: if StatementChainTooLong were ever classified as
    // "incomplete", split_script_into_forms would keep appending later lines
    // to the offending one, silently swallowing the rest of the script. The
    // over-cap line must become its own (reported) form and the following
    // lines must still split. MAX_STATEMENT_CHAIN_DEPTH is pub(crate) in
    // rum-syntax, so the count is a literal: 300 sits well over the 256 cap.
    let over_cap_line = (0..300)
        .map(|i| format!("x{i} = {i}"))
        .collect::<Vec<_>>()
        .join("; ");
    let script = format!("{over_cap_line}\ny = 1\nz = 2");

    let error = parse_expr(&over_cap_line)
        .expect_err("a 300-statement line should exceed the statement cap");
    assert!(matches!(error, SyntaxError::StatementChainTooLong { .. }));
    assert!(!error.is_incomplete());

    let forms = split_script_into_forms(&script).unwrap();
    assert_eq!(forms.len(), 3);
    assert!(forms[0].starts_with("x0 = 0"));
    assert_eq!(forms[1], "y = 1");
    assert_eq!(forms[2], "z = 2");
}

#[test]
fn attributes_reports_builtin_and_user_defined_attribute_sets() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Attributes[If]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldRest]"
    );
    // HoldAllComplete and Identity resolve to the real catalog and now carry
    // Protected (whole-registry sweep, brief-210): HoldAllComplete has no
    // test-control builtin, and Identity's test-control registration has empty
    // attributes, so both fall through to the catalog default. If and Plus keep
    // their harness attributes — their test registrations set non-empty sets
    // that override the catalog. The sweep does not touch test_builtins.rs.
    assert_eq!(
        session
            .eval_text("Attributes[HoldAllComplete]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldAllComplete, Protected]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[Identity]")
            .unwrap()
            .value
            .to_string(),
        "List[Protected]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[Plus]")
            .unwrap()
            .value
            .to_string(),
        "List[Listable, Orderless, Flat, OneIdentity]"
    );

    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();

    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst]"
    );

    let info = session.inspect("f").unwrap();
    assert_eq!(info.attribute_count, 1);
    assert_eq!(info.attributes, vec!["HoldFirst".to_string()]);
}
