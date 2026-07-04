use super::support::*;

#[test]
fn parses_right_associative_apply_sugar() {
    let parsed = parse_expr("f @ g @ x").unwrap();
    assert_eq!(parsed.to_string(), "f @ g @ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::symbol("f", z()),
            SyntaxExpr::binary(
                BinaryOp::Apply,
                SyntaxExpr::symbol("g", z()),
                SyntaxExpr::symbol("x", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_postfix_apply_as_left_associative() {
    let parsed = parse_expr("a // f // g").unwrap();
    assert_eq!(parsed.to_string(), "a // f // g");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::PostfixApply,
            SyntaxExpr::binary(
                BinaryOp::PostfixApply,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("f", z()),
                z(),
            ),
            SyntaxExpr::symbol("g", z()),
            z(),
        )
    );
}

#[test]
fn parses_postfix_apply_between_set_and_replace_all_precedence() {
    let set = parse_expr("a = b // f").unwrap();
    assert_eq!(
        set,
        SyntaxExpr::binary(
            BinaryOp::Set,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::PostfixApply,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("f", z()),
                z(),
            ),
            z(),
        )
    );

    let replace_all = parse_expr("a // f /. r").unwrap();
    assert_eq!(
        replace_all,
        SyntaxExpr::binary(
            BinaryOp::PostfixApply,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::ReplaceAll,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("r", z()),
                z(),
            ),
            z(),
        )
    );

    let plus = parse_expr("a + b // f").unwrap();
    assert_eq!(
        plus,
        SyntaxExpr::binary(
            BinaryOp::PostfixApply,
            SyntaxExpr::binary(
                BinaryOp::Add,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("f", z()),
            z(),
        )
    );
}

#[test]
fn parses_compound_expression_sugar() {
    let parsed = parse_expr("a; b;").unwrap();
    assert_eq!(parsed.to_string(), "a; b; Null");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::CompoundExpression,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::CompoundExpression,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("Null", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_apply_and_composition_operator_precedence() {
    let parsed = parse_expr("f @* g @ x").unwrap();
    assert_eq!(parsed.to_string(), "f @* g @ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::binary(
                BinaryOp::Composition,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("g", z()),
                z(),
            ),
            SyntaxExpr::symbol("x", z()),
            z(),
        )
    );

    let right = parse_expr("f /* g @ x").unwrap();
    assert_eq!(right.to_string(), "f /* g @ x");
    assert_eq!(
        right,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::binary(
                BinaryOp::RightComposition,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("g", z()),
                z(),
            ),
            SyntaxExpr::symbol("x", z()),
            z(),
        )
    );
}

#[test]
fn parses_apply_level_operators_as_right_associative() {
    let parsed = parse_expr("f /@ g @@ h @@@ x").unwrap();
    assert_eq!(parsed.to_string(), "f /@ g @@ h @@@ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Map,
            SyntaxExpr::symbol("f", z()),
            SyntaxExpr::binary(
                BinaryOp::ApplyHead,
                SyntaxExpr::symbol("g", z()),
                SyntaxExpr::binary(
                    BinaryOp::MapApply,
                    SyntaxExpr::symbol("h", z()),
                    SyntaxExpr::symbol("x", z()),
                    z(),
                ),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_function_postfix_as_map_lhs() {
    let parsed = parse_expr("typeSubstImpl[theta, #] & /@ args").unwrap();
    assert_eq!(parsed.to_string(), "(typeSubstImpl[theta, #] &) /@ args");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Map,
            SyntaxExpr::postfix(
                PostfixOp::Function,
                SyntaxExpr::call(
                    SyntaxExpr::symbol("typeSubstImpl", z()),
                    vec![
                        SyntaxExpr::symbol("theta", z()),
                        SyntaxExpr::slot(SlotSpec::Anonymous, z())
                    ],
                    z(),
                ),
                z(),
            ),
            SyntaxExpr::symbol("args", z()),
            z(),
        )
    );
}

#[test]
fn parses_function_postfix_on_set_rhs_before_map() {
    let parsed = parse_expr("ks = #[[1]] & /@ args").unwrap();
    assert_eq!(parsed.to_string(), "ks = (Part[#, 1] &) /@ args");
}
