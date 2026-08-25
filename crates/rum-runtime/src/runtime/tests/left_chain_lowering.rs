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
    let syntax = parse_expr("Plus[a, b] + c + d").expect("direct-call chain should parse");
    let lowered = lower_syntax_expr(&syntax).expect("direct-call chain should lower");

    assert_eq!(lowered.to_string(), "Plus[Plus[Plus[a, b], c], d]");
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
    for index in (0..LINK_COUNT).rev() {
        let op = AFFECTED_OPERATORS[index % AFFECTED_OPERATORS.len()].0;
        current = assert_link_shape(current, op, false);
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
    let mut current = expr;
    for index in (0..link_count).rev() {
        current = assert_link_shape(current, op, index == 0);
    }
    assert_global_a(current);
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
