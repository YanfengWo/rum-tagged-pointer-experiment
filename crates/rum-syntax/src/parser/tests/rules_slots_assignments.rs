use super::support::*;

#[test]
fn parses_rule_and_replace_all_precedence() {
    let parsed = parse_expr("expr /. x_Integer -> y").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::ReplaceAll,
            SyntaxExpr::symbol("expr", z()),
            SyntaxExpr::binary(
                BinaryOp::Rule,
                SyntaxExpr::pattern(
                    SyntaxExpr::symbol("x", z()),
                    SyntaxExpr::blank(
                        BlankKind::Blank,
                        Some(SyntaxExpr::symbol("Integer", z())),
                        z(),
                    ),
                    z(),
                ),
                SyntaxExpr::symbol("y", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_replace_repeated_precedence() {
    let parsed = parse_expr("expr //. x_Integer -> y").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::ReplaceRepeated,
            SyntaxExpr::symbol("expr", z()),
            SyntaxExpr::binary(
                BinaryOp::Rule,
                SyntaxExpr::pattern(
                    SyntaxExpr::symbol("x", z()),
                    SyntaxExpr::blank(
                        BlankKind::Blank,
                        Some(SyntaxExpr::symbol("Integer", z())),
                        z(),
                    ),
                    z(),
                ),
                SyntaxExpr::symbol("y", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_slots_and_pure_functions() {
    let parsed = parse_expr("#1 + #2 &").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::binary(
                BinaryOp::Add,
                SyntaxExpr::slot(SlotSpec::Number("1".to_string()), z()),
                SyntaxExpr::slot(SlotSpec::Number("2".to_string()), z()),
                z(),
            ),
            z(),
        )
    );

    let sequence_parsed = parse_expr("## + ##2 &").unwrap();
    assert_eq!(
        sequence_parsed,
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::binary(
                BinaryOp::Add,
                SyntaxExpr::slot(SlotSpec::Sequence, z()),
                SyntaxExpr::slot(SlotSpec::SequenceNumber("2".to_string()), z()),
                z(),
            ),
            z(),
        )
    );

    let named_parsed = parse_expr("#x &").unwrap();
    assert_eq!(
        named_parsed,
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::slot(SlotSpec::Named("x".to_string()), z()),
            z(),
        )
    );
}

#[test]
fn parses_rule_expression_as_pure_function_body() {
    let parsed = parse_expr("Map[instType[theta, #[[1]]] -> #[[2]] &, pairs]").unwrap();
    assert_eq!(
        parsed.to_string(),
        "Map[instType[theta, Part[#, 1]] -> Part[#, 2] &, pairs]"
    );
}

#[test]
fn does_not_merge_slots_across_trivia_into_a_sequence_slot() {
    let expected = SyntaxExpr::postfix(
        PostfixOp::Function,
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::slot(SlotSpec::Anonymous, z()),
            SyntaxExpr::slot(SlotSpec::Anonymous, z()),
            z(),
        ),
        z(),
    );

    assert_eq!(parse_expr("# # &").unwrap(), expected);

    assert_eq!(parse_expr("#(* gap *)# &").unwrap(), expected);
}

#[test]
fn does_not_merge_slot_specs_across_trivia() {
    assert_eq!(
        parse_expr("# 6 &").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::slot(SlotSpec::Anonymous, z()),
                SyntaxExpr::integer("6", z()),
                z(),
            ),
            z(),
        )
    );

    assert_eq!(
        parse_expr("# x &").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::slot(SlotSpec::Anonymous, z()),
                SyntaxExpr::symbol("x", z()),
                z(),
            ),
            z(),
        )
    );

    assert_eq!(
        parse_expr("## 2 &").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::slot(SlotSpec::Sequence, z()),
                SyntaxExpr::integer("2", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_message_name_before_assignment() {
    let parsed = parse_expr("f::argx = \"bad arg `1`\"").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Set,
            SyntaxExpr::binary(
                BinaryOp::MessageName,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("argx", z()),
                z(),
            ),
            SyntaxExpr::string("bad arg `1`", z()),
            z(),
        )
    );
}

#[test]
fn parses_message_name_inside_calls() {
    let parsed = parse_expr("Message[f::argx, x]").unwrap();
    assert_eq!(parsed.to_string(), "Message[f :: argx, x]");
}

#[test]
fn parses_tagset_before_assignment_rhs() {
    let parsed = parse_expr("f /: g[x_] := rhs").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::TagSet,
            SyntaxExpr::symbol("f", z()),
            SyntaxExpr::binary(
                BinaryOp::SetDelayed,
                SyntaxExpr::call(
                    SyntaxExpr::symbol("g", z()),
                    vec![SyntaxExpr::pattern(
                        SyntaxExpr::symbol("x", z()),
                        SyntaxExpr::blank(BlankKind::Blank, None, z()),
                        z(),
                    )],
                    z(),
                ),
                SyntaxExpr::symbol("rhs", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_rule_rhs_before_assignment_rhs() {
    let parsed = parse_expr("a -> x = x + 1").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Rule,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::Set,
                SyntaxExpr::symbol("x", z()),
                SyntaxExpr::binary(
                    BinaryOp::Add,
                    SyntaxExpr::symbol("x", z()),
                    SyntaxExpr::integer("1", z()),
                    z(),
                ),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_rule_delayed_rhs_before_assignment_rhs() {
    let parsed = parse_expr("a :> x = x + 1").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::RuleDelayed,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::Set,
                SyntaxExpr::symbol("x", z()),
                SyntaxExpr::binary(
                    BinaryOp::Add,
                    SyntaxExpr::symbol("x", z()),
                    SyntaxExpr::integer("1", z()),
                    z(),
                ),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_unset_postfix() {
    let parsed = parse_expr("f[x_] =.").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::postfix(
            PostfixOp::Unset,
            SyntaxExpr::call(
                SyntaxExpr::symbol("f", z()),
                vec![SyntaxExpr::pattern(
                    SyntaxExpr::symbol("x", z()),
                    SyntaxExpr::blank(BlankKind::Blank, None, z()),
                    z(),
                )],
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_factorial_derivative_and_repeated_postfix_forms() {
    assert_eq!(
        parse_expr("a!^2").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Power,
            SyntaxExpr::postfix(PostfixOp::Factorial, SyntaxExpr::symbol("a", z()), z()),
            SyntaxExpr::integer("2", z()),
            z(),
        )
    );
    assert_eq!(
        parse_expr("a!!").unwrap(),
        SyntaxExpr::postfix(PostfixOp::Factorial2, SyntaxExpr::symbol("a", z()), z())
    );
    assert_eq!(
        parse_expr("f'''").unwrap(),
        SyntaxExpr::postfix(PostfixOp::Derivative(3), SyntaxExpr::symbol("f", z()), z(),)
    );
    assert_eq!(
        parse_expr("{a ..}").unwrap(),
        SyntaxExpr::list(
            vec![SyntaxExpr::postfix(
                PostfixOp::Repeated,
                SyntaxExpr::symbol("a", z()),
                z(),
            )],
            z(),
        )
    );
    assert_eq!(
        parse_expr("{a ...}").unwrap(),
        SyntaxExpr::list(
            vec![SyntaxExpr::postfix(
                PostfixOp::RepeatedNull,
                SyntaxExpr::symbol("a", z()),
                z(),
            )],
            z(),
        )
    );
}
