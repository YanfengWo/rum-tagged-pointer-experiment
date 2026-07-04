use super::support::*;

#[test]
fn parses_colon_named_pattern_sugar() {
    let parsed = parse_expr("theta : {___Rule}").unwrap();
    assert_eq!(parsed.to_string(), "theta : {___Rule}");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Pattern,
            SyntaxExpr::symbol("theta", z()),
            SyntaxExpr::list(
                vec![SyntaxExpr::blank(
                    BlankKind::BlankNullSequence,
                    Some(SyntaxExpr::symbol("Rule", z())),
                    z(),
                )],
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_pattern_alternatives_sugar() {
    let parsed = parse_expr("MatchQ[t, var[_, _] | const[_, _]]").unwrap();
    assert_eq!(parsed.to_string(), "MatchQ[t, var[_, _] | const[_, _]]");
}

#[test]
fn parses_pattern_test_sugar() {
    let parsed = parse_expr("Cases[t, _?propBoolAtomQ, {0, Infinity}]").unwrap();
    assert_eq!(
        parsed.to_string(),
        "Cases[t, _ ? propBoolAtomQ, {0, Infinity}]"
    );
}

#[test]
fn parses_named_patterns() {
    let parsed = parse_expr("x_Integer").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::pattern(
            SyntaxExpr::symbol("x", z()),
            SyntaxExpr::blank(
                BlankKind::Blank,
                Some(SyntaxExpr::symbol("Integer", z())),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_optional_default_patterns() {
    assert_eq!(
        parse_expr("x_.").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Optional,
            SyntaxExpr::pattern(
                SyntaxExpr::symbol("x", z()),
                SyntaxExpr::blank(BlankKind::Blank, None, z()),
                z(),
            ),
            z(),
        )
    );
    assert_eq!(
        parse_expr("_.").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Optional,
            SyntaxExpr::blank(BlankKind::Blank, None, z()),
            z(),
        )
    );
    assert_eq!(
        parse_expr("x_..").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Repeated,
            SyntaxExpr::pattern(
                SyntaxExpr::symbol("x", z()),
                SyntaxExpr::blank(BlankKind::Blank, None, z()),
                z(),
            ),
            z(),
        )
    );
    assert_eq!(
        parse_expr("x_...").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::RepeatedNull,
            SyntaxExpr::pattern(
                SyntaxExpr::symbol("x", z()),
                SyntaxExpr::blank(BlankKind::Blank, None, z()),
                z(),
            ),
            z(),
        )
    );
}
