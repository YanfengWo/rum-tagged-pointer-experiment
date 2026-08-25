use super::*;
use rum_syntax::Span;

const AFFECTED_OPERATORS: [(BinaryOp, &str); 16] = [
    (BinaryOp::Add, "+"),
    (BinaryOp::Subtract, "-"),
    (BinaryOp::Times, "*"),
    (BinaryOp::Divide, "/"),
    (BinaryOp::Dot, "."),
    (BinaryOp::NonCommutativeMultiply, "**"),
    (BinaryOp::StringJoin, "<>"),
    (BinaryOp::MessageName, "::"),
    (BinaryOp::ReplaceAll, "/."),
    (BinaryOp::ReplaceRepeated, "//."),
    (BinaryOp::Pattern, ":"),
    (BinaryOp::Alternatives, "|"),
    (BinaryOp::PatternTest, "?"),
    (BinaryOp::StringExpression, "~~"),
    (BinaryOp::PostfixApply, "//"),
    (BinaryOp::RightComposition, "/*"),
];

#[test]
fn left_chain_lowering_preserves_cross_family_shape_and_resolver_order() {
    let syntax = parse_expr("a . b * c + d /. r //. s")
        .expect("cross-family left-chain source should parse");
    let mut calls = Vec::new();
    let lowered = lower_syntax_expr_with(&syntax, &mut |raw| {
        calls.push(raw.to_owned());
        None
    })
    .expect("cross-family left-chain source should lower");

    assert_eq!(calls, ["a", "b", "c", "d", "r", "s"]);
    assert_eq!(
        lowered.to_string(),
        "ReplaceRepeated[ReplaceAll[Plus[Times[Dot[a, b], c], d], r], s]"
    );
}

#[test]
fn left_chain_lowering_preserves_message_name_double_resolution_order() {
    let syntax = parse_expr("owner::first::tag[inner]").expect("message-name chain should parse");
    let mut calls = Vec::new();
    let lowered = lower_syntax_expr_with(&syntax, &mut |raw| {
        calls.push(raw.to_owned());
        None
    })
    .expect("message-name chain should lower");

    assert_eq!(calls, ["owner", "first", "tag", "inner", "tag", "inner"]);
    assert_eq!(lowered.to_string(), "MessageName[owner::first, tag[inner]]");
}

#[test]
fn left_chain_lowering_keeps_direct_canonical_calls_opaque() {
    let cases = [
        ("Plus[a, b] + c + d", "Plus[Plus[a, b], c, d]"),
        ("a + Plus[b, c] + d", "Plus[a, Plus[b, c], d]"),
        ("Times[a, b] * c * d", "Times[Times[a, b], c, d]"),
        ("a * Times[b, c] * d", "Times[a, Times[b, c], d]"),
        (
            "RightComposition[f, g] /* h /* k",
            "RightComposition[RightComposition[f, g], h, k]",
        ),
    ];

    for (source, expected) in cases {
        let syntax = parse_expr(source).expect("direct-call chain should parse");
        let lowered = lower_syntax_expr(&syntax).expect("direct-call chain should lower");
        assert_eq!(lowered.to_string(), expected, "{source}");
    }
}

#[test]
fn held_sameq_explicitly_observes_flat_chain_shapes() {
    let mut session = Session::new();
    crate::runtime::test_builtins::register_test_sameq_builtin(&mut session);

    let cases = [
        (
            "SameQ[HoldComplete[a + b + c], HoldComplete[Plus[a, b, c]]]",
            "True",
        ),
        (
            "SameQ[HoldComplete[a + b + c], HoldComplete[Plus[Plus[a, b], c]]]",
            "False",
        ),
        (
            "SameQ[HoldComplete[a * b * c], HoldComplete[Times[a, b, c]]]",
            "True",
        ),
        (
            "SameQ[HoldComplete[a * b * c], HoldComplete[Times[Times[a, b], c]]]",
            "False",
        ),
        (
            "SameQ[HoldComplete[f /* g /* h], HoldComplete[System`RightComposition[f, g, h]]]",
            "True",
        ),
        (
            "SameQ[HoldComplete[f /* g /* h], HoldComplete[System`RightComposition[System`RightComposition[f, g], h]]]",
            "False",
        ),
    ];

    for (source, expected) in cases {
        let value = session
            .eval_text(source)
            .expect("held SameQ case should evaluate");
        assert_eq!(value.value.to_string(), expected, "{source}");
    }
}

#[test]
fn flat_chain_evaluated_results_remain_unchanged() {
    let mut session = session_with_test_rewrite_builtins();
    let cases = [
        ("1 + 2 + 3 + 4", "10"),
        ("2 * 3 * 4", "24"),
        ("10 - 3 - 2", "5"),
        ("100 / 5 / 2", "10"),
        ("1 + 2 - 3 + 4 - 5", "-1"),
        ("(f /* g)[x]", "g[f[x]]"),
        ("1/2 + 1/3 + 1/6", "1"),
        ("2 * 3 / 4 * 5", "15/2"),
        ("84 / 7 / 3 / 2 / 2", "1"),
        ("(f /* g /* h /* k)[x]", "k[h[g[f[x]]]]"),
    ];

    for (source, expected) in cases {
        let value = session
            .eval_text(source)
            .expect("flat-chain result case should evaluate");
        assert_eq!(value.value.to_string(), expected, "{source}");
    }
}

#[test]
fn left_chain_lowering_handles_long_source_chains_for_all_operators() {
    const LINK_COUNT: usize = 20_000;

    for (op, token) in AFFECTED_OPERATORS {
        let source = repeated_chain_source(token, LINK_COUNT);
        let syntax = parse_expr(&source)
            .unwrap_or_else(|error| panic!("long {token:?} left chain should parse: {error}"));
        let lowered = match lower_syntax_expr(&syntax) {
            Ok(lowered) => lowered,
            Err(error) => {
                drop_left_binary_chain(syntax);
                panic!("long {token:?} left chain should lower: {error}");
            }
        };

        assert_homogeneous_chain_shape(&lowered, op, LINK_COUNT);
        drop_left_binary_chain(syntax);
        drop_expr_iteratively(lowered);
    }
}

#[test]
fn left_chain_lowering_handles_a_deep_hand_built_alternating_ast() {
    const LINK_COUNT: usize = 20_000;

    let mut syntax = SyntaxExpr::symbol("a", Span::default());
    for index in 0..LINK_COUNT {
        syntax = SyntaxExpr::binary(
            AFFECTED_OPERATORS[index % AFFECTED_OPERATORS.len()].0,
            syntax,
            SyntaxExpr::symbol("a", Span::default()),
            Span::default(),
        );
    }

    let lowered = lower_syntax_expr(&syntax).expect("alternating direct AST should lower");
    let mut current = &lowered;
    for _ in 0..(LINK_COUNT / AFFECTED_OPERATORS.len()) {
        for (op, _) in AFFECTED_OPERATORS[4..].iter().rev() {
            current = assert_link_shape(current, *op, false);
        }
        current = assert_flat_multiplicative_pair(current);
        current = assert_flat_additive_pair(current);
    }
    assert_global_a(current);

    drop_left_binary_chain(syntax);
    drop_expr_iteratively(lowered);
}

fn repeated_chain_source(token: &str, link_count: usize) -> String {
    let mut source = String::with_capacity(1 + link_count * (token.len() + 3));
    source.push('a');
    for _ in 0..link_count {
        source.push(' ');
        source.push_str(token);
        source.push_str(" a");
    }
    source
}

fn assert_homogeneous_chain_shape(expr: &Expr, op: BinaryOp, link_count: usize) {
    if matches!(
        op,
        BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Times
            | BinaryOp::Divide
            | BinaryOp::RightComposition
    ) {
        assert_flat_homogeneous_chain_shape(expr, op, link_count);
        return;
    }

    let mut current = expr;
    for index in (0..link_count).rev() {
        current = assert_link_shape(current, op, index == 0);
    }
    assert_global_a(current);
}

fn assert_flat_homogeneous_chain_shape(expr: &Expr, op: BinaryOp, link_count: usize) {
    let Expr::Call { head, args } = expr else {
        panic!("flat left chain should lower to a call");
    };
    let expected_head = match op {
        BinaryOp::Add | BinaryOp::Subtract => heads::arithmetic::PLUS,
        BinaryOp::Times | BinaryOp::Divide => heads::arithmetic::TIMES,
        BinaryOp::RightComposition => "RightComposition",
        _ => unreachable!("only flat-family operators reach this helper"),
    };
    assert!(is_system_symbol_name(head, expected_head));
    assert_eq!(args.len(), link_count + 1);
    assert_global_a(&args[0]);

    for arg in &args[1..] {
        match op {
            BinaryOp::Add | BinaryOp::Times | BinaryOp::RightComposition => {
                assert_global_a(arg);
            }
            BinaryOp::Subtract => assert_negated_global_a(arg),
            BinaryOp::Divide => assert_reciprocal_global_a(arg),
            _ => unreachable!("only flat-family operators reach this helper"),
        }
    }
}

fn assert_flat_additive_pair(expr: &Expr) -> &Expr {
    let Expr::Call { head, args } = expr else {
        panic!("additive pair should lower to a call");
    };
    assert!(is_system_symbol_name(head, heads::arithmetic::PLUS));
    assert_eq!(args.len(), 3);
    assert_global_a(&args[1]);
    assert_negated_global_a(&args[2]);
    &args[0]
}

fn assert_flat_multiplicative_pair(expr: &Expr) -> &Expr {
    let Expr::Call { head, args } = expr else {
        panic!("multiplicative pair should lower to a call");
    };
    assert!(is_system_symbol_name(head, heads::arithmetic::TIMES));
    assert_eq!(args.len(), 3);
    assert_global_a(&args[1]);
    assert_reciprocal_global_a(&args[2]);
    &args[0]
}

fn assert_negated_global_a(expr: &Expr) {
    let Expr::Call { head, args } = expr else {
        panic!("subtracted term should lower to Times[-1, a]");
    };
    assert!(is_system_symbol_name(head, heads::arithmetic::TIMES));
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], Expr::integer(BigInt::from(-1)));
    assert_global_a(&args[1]);
}

fn assert_reciprocal_global_a(expr: &Expr) {
    let Expr::Call { head, args } = expr else {
        panic!("divisor should lower to Power[a, -1]");
    };
    assert!(is_system_symbol_name(head, heads::arithmetic::POWER));
    assert_eq!(args.len(), 2);
    assert_global_a(&args[0]);
    assert_eq!(args[1], Expr::integer(BigInt::from(-1)));
}

fn assert_link_shape(expr: &Expr, op: BinaryOp, pattern_lhs_was_symbol: bool) -> &Expr {
    let Expr::Call { head, args } = expr else {
        panic!("left-chain link should lower to a call");
    };

    if op == BinaryOp::PostfixApply {
        assert_global_a(head);
        assert_eq!(args.len(), 1);
        return &args[0];
    }

    let expected_head = match op {
        BinaryOp::Add | BinaryOp::Subtract => heads::arithmetic::PLUS,
        BinaryOp::Times | BinaryOp::Divide => heads::arithmetic::TIMES,
        BinaryOp::Dot => "Dot",
        BinaryOp::NonCommutativeMultiply => "NonCommutativeMultiply",
        BinaryOp::StringJoin => "StringJoin",
        BinaryOp::MessageName => heads::messages::MESSAGE_NAME,
        BinaryOp::ReplaceAll => heads::rules::REPLACE_ALL,
        BinaryOp::ReplaceRepeated => heads::rules::REPLACE_REPEATED,
        BinaryOp::Pattern if pattern_lhs_was_symbol => heads::patterns::PATTERN,
        BinaryOp::Pattern => "Optional",
        BinaryOp::Alternatives => "Alternatives",
        BinaryOp::PatternTest => "PatternTest",
        BinaryOp::StringExpression => "StringExpression",
        BinaryOp::RightComposition => "RightComposition",
        _ => panic!("unexpected left-chain operator {op:?}"),
    };

    assert!(is_system_symbol_name(head, expected_head));
    assert_eq!(args.len(), 2);
    &args[0]
}

fn assert_global_a(expr: &Expr) {
    assert!(matches!(
        expr,
        Expr::Atom(Atom::Symbol(symbol))
            if symbol.context() == GLOBAL_CONTEXT && symbol.name() == "a"
    ));
}

fn drop_left_binary_chain(mut expr: SyntaxExpr) {
    loop {
        match expr.kind {
            SyntaxExprKind::Binary { lhs, rhs, .. } => {
                drop(rhs);
                expr = *lhs;
            }
            _ => return,
        }
    }
}

fn drop_expr_iteratively(expr: Expr) {
    let mut work = vec![expr];
    while let Some(expr) = work.pop() {
        if let Some((head, args)) = expr.into_call_parts() {
            work.push(head);
            work.extend(args);
        }
    }
}
