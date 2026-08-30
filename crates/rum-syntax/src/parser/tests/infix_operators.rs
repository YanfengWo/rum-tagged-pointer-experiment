use super::support::*;

#[test]
fn parses_new_infix_binary_operators() {
    let upset = parse_expr("a ^= b ^= c").unwrap();
    assert_eq!(
        upset,
        SyntaxExpr::binary(
            BinaryOp::UpSet,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::UpSet,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("c", z()),
                z(),
            ),
            z(),
        )
    );

    let upset_delayed = parse_expr("a ^:= b ^:= c").unwrap();
    assert_eq!(
        upset_delayed,
        SyntaxExpr::binary(
            BinaryOp::UpSetDelayed,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::UpSetDelayed,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("c", z()),
                z(),
            ),
            z(),
        )
    );

    let upset_precedence = parse_expr("a ^= b // f").unwrap();
    assert_eq!(
        upset_precedence,
        SyntaxExpr::binary(
            BinaryOp::UpSet,
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

    let non_commutative_chain = parse_expr("a ** b ** c").unwrap();
    assert_eq!(
        non_commutative_chain,
        SyntaxExpr::binary(
            BinaryOp::NonCommutativeMultiply,
            SyntaxExpr::binary(
                BinaryOp::NonCommutativeMultiply,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("c", z()),
            z(),
        )
    );

    let times = parse_expr("a ** b * c").unwrap();
    assert_eq!(
        times,
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::binary(
                BinaryOp::NonCommutativeMultiply,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("c", z()),
            z(),
        )
    );

    let power = parse_expr("a ** b ^ c").unwrap();
    assert_eq!(
        power,
        SyntaxExpr::binary(
            BinaryOp::NonCommutativeMultiply,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::Power,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("c", z()),
                z(),
            ),
            z(),
        )
    );

    let dot_chain = parse_expr("a . b . c").unwrap();
    assert_eq!(
        dot_chain,
        SyntaxExpr::binary(
            BinaryOp::Dot,
            SyntaxExpr::binary(
                BinaryOp::Dot,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("c", z()),
            z(),
        )
    );

    let dot_tighter_than_times = parse_expr("a . b * c").unwrap();
    assert_eq!(
        dot_tighter_than_times,
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::binary(
                BinaryOp::Dot,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("c", z()),
            z(),
        )
    );

    let power_tighter_than_dot = parse_expr("a . b ^ c").unwrap();
    assert_eq!(
        power_tighter_than_dot,
        SyntaxExpr::binary(
            BinaryOp::Dot,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::Power,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("c", z()),
                z(),
            ),
            z(),
        )
    );

    let non_commutative_tighter_than_dot = parse_expr("a . b ** c").unwrap();
    assert_eq!(
        non_commutative_tighter_than_dot,
        SyntaxExpr::binary(
            BinaryOp::Dot,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::NonCommutativeMultiply,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("c", z()),
                z(),
            ),
            z(),
        )
    );

    let map_all = parse_expr("f //@ g //@ x").unwrap();
    assert_eq!(
        map_all,
        SyntaxExpr::binary(
            BinaryOp::MapAll,
            SyntaxExpr::symbol("f", z()),
            SyntaxExpr::binary(
                BinaryOp::MapAll,
                SyntaxExpr::symbol("g", z()),
                SyntaxExpr::symbol("x", z()),
                z(),
            ),
            z(),
        )
    );

    let function = parse_expr("x |-> a + b").unwrap();
    assert_eq!(
        function,
        SyntaxExpr::binary(
            BinaryOp::Function,
            SyntaxExpr::symbol("x", z()),
            SyntaxExpr::binary(
                BinaryOp::Add,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            z(),
        )
    );

    let function_chain = parse_expr("x |-> y |-> y").unwrap();
    assert_eq!(
        function_chain,
        SyntaxExpr::binary(
            BinaryOp::Function,
            SyntaxExpr::symbol("x", z()),
            SyntaxExpr::binary(
                BinaryOp::Function,
                SyntaxExpr::symbol("y", z()),
                SyntaxExpr::symbol("y", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_logical_and_comparison_operator_precedence() {
    let parsed = parse_expr("! MatchQ[k, pat] && 0 <= i < depth || done").unwrap();
    assert_eq!(
        parsed.to_string(),
        "!MatchQ[k, pat] && 0 <= i < depth || done"
    );

    let same_chain = parse_expr("x === y === z").unwrap();
    assert_eq!(same_chain.to_string(), "x === y === z");
    let SyntaxExprKind::ComparisonChain {
        operands,
        operators,
    } = &same_chain.kind
    else {
        panic!("same-operator comparisons should parse as one chain");
    };
    assert_eq!(operands.len(), 3);
    assert_eq!(operators, &[BinaryOp::SameQ, BinaryOp::SameQ]);

    let mixed_chain = parse_expr("x == y != z").unwrap();
    assert_eq!(mixed_chain.to_string(), "x == y != z");
    let SyntaxExprKind::ComparisonChain {
        operands,
        operators,
    } = &mixed_chain.kind
    else {
        panic!("mixed comparisons should parse as one chain");
    };
    assert_eq!(operands.len(), 3);
    assert_eq!(operators, &[BinaryOp::Equal, BinaryOp::Unequal]);
}

#[test]
fn comparison_chains_do_not_cross_parenthesis_boundaries() {
    let parsed = parse_expr("(a == b) == c").unwrap();
    let SyntaxExprKind::ComparisonChain {
        operands,
        operators,
    } = &parsed.kind
    else {
        panic!("outer comparison should parse as a chain");
    };
    assert_eq!(operators, &[BinaryOp::Equal]);
    assert!(matches!(
        operands[0].kind,
        SyntaxExprKind::ComparisonChain { .. }
    ));

    let parsed = parse_expr("a == (b == c)").unwrap();
    let SyntaxExprKind::ComparisonChain { operands, .. } = &parsed.kind else {
        panic!("outer comparison should parse as a chain");
    };
    assert!(matches!(
        operands[1].kind,
        SyntaxExprKind::ComparisonChain { .. }
    ));
}

#[test]
fn parses_string_join_infix_sugar() {
    let parsed = parse_expr("\"a\" <> ToString[i]").unwrap();
    assert_eq!(parsed.to_string(), "\"a\" <> ToString[i]");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::StringJoin,
            SyntaxExpr::string("a", z()),
            SyntaxExpr::call(
                SyntaxExpr::symbol("ToString", z()),
                vec![SyntaxExpr::symbol("i", z())],
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_string_expression_infix_sugar() {
    let parsed = parse_expr("a ~~ b ~~ c").unwrap();
    assert_eq!(parsed.to_string(), "a ~~ b ~~ c");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::StringExpression,
            SyntaxExpr::binary(
                BinaryOp::StringExpression,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("c", z()),
            z(),
        )
    );

    let string_join_binds_tighter = parse_expr("a ~~ b <> c").unwrap();
    assert_eq!(
        string_join_binds_tighter,
        SyntaxExpr::binary(
            BinaryOp::StringExpression,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::StringJoin,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("c", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_tilde_infix_application_as_call() {
    let parsed = parse_expr("a ~ g ~ b").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::call(
            SyntaxExpr::symbol("g", z()),
            vec![SyntaxExpr::symbol("a", z()), SyntaxExpr::symbol("b", z())],
            z(),
        )
    );

    let plus = parse_expr("a ~ g ~ b + c").unwrap();
    assert_eq!(
        plus,
        SyntaxExpr::binary(
            BinaryOp::Add,
            SyntaxExpr::call(
                SyntaxExpr::symbol("g", z()),
                vec![SyntaxExpr::symbol("a", z()), SyntaxExpr::symbol("b", z())],
                z(),
            ),
            SyntaxExpr::symbol("c", z()),
            z(),
        )
    );

    let chain = parse_expr("x ~ f ~ y ~ g ~ z").unwrap();
    assert_eq!(
        chain,
        SyntaxExpr::call(
            SyntaxExpr::symbol("g", z()),
            vec![
                SyntaxExpr::call(
                    SyntaxExpr::symbol("f", z()),
                    vec![SyntaxExpr::symbol("x", z()), SyntaxExpr::symbol("y", z())],
                    z(),
                ),
                SyntaxExpr::symbol("z", z()),
            ],
            z(),
        )
    );

    let context_head = parse_expr("a ~ HOL`Bool`foo ~ b").unwrap();
    assert_eq!(
        context_head,
        SyntaxExpr::call(
            SyntaxExpr::symbol("HOL`Bool`foo", z()),
            vec![SyntaxExpr::symbol("a", z()), SyntaxExpr::symbol("b", z())],
            z(),
        )
    );
}

#[test]
fn parses_implicit_multiplication_juxtaposition() {
    assert_eq!(
        parse_expr("a b").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::symbol("b", z()),
            z(),
        )
    );
    assert_eq!(
        parse_expr("2 x").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::integer("2", z()),
            SyntaxExpr::symbol("x", z()),
            z(),
        )
    );
    assert_eq!(
        parse_expr("g #").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::symbol("g", z()),
            SyntaxExpr::slot(SlotSpec::Anonymous, z()),
            z(),
        )
    );
    assert_eq!(
        parse_expr("a (b + c)").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::Add,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("c", z()),
                z(),
            ),
            z(),
        )
    );
    assert_eq!(
        parse_expr("a + b c").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Add,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("c", z()),
                z(),
            ),
            z(),
        )
    );
    assert_eq!(
        parse_expr("a b + c").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Add,
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("c", z()),
            z(),
        )
    );
    assert_eq!(
        parse_expr("a b c").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("c", z()),
            z(),
        )
    );
    assert_eq!(
        parse_expr("2 x^2").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::integer("2", z()),
            SyntaxExpr::binary(
                BinaryOp::Power,
                SyntaxExpr::symbol("x", z()),
                SyntaxExpr::integer("2", z()),
                z(),
            ),
            z(),
        )
    );
    assert_eq!(
        parse_expr("Map[g # &, a]").unwrap(),
        SyntaxExpr::call(
            SyntaxExpr::symbol("Map", z()),
            vec![
                SyntaxExpr::postfix(
                    PostfixOp::Function,
                    SyntaxExpr::binary(
                        BinaryOp::Times,
                        SyntaxExpr::symbol("g", z()),
                        SyntaxExpr::slot(SlotSpec::Anonymous, z()),
                        z(),
                    ),
                    z(),
                ),
                SyntaxExpr::symbol("a", z()),
            ],
            z(),
        )
    );
    assert_eq!(parse_expr("a - b").unwrap().to_string(), "a - b");
    assert_eq!(parse_expr("a -> b").unwrap().to_string(), "a -> b");
}

#[test]
fn parses_prefix_minus_with_power_precedence() {
    let parsed = parse_expr("-x^2").unwrap();
    assert_eq!(parsed.to_string(), "-x ^ 2");
}
