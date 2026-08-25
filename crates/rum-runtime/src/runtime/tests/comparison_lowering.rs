use super::*;
use rum_syntax::{MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS, Span};

#[test]
fn comparison_lowering_preserves_resolver_call_count_and_order() {
    let syntax = parse_expr("(f[a] < b) <= (c == g[d])")
        .expect("resolver-order comparison source should parse");
    let mut calls = Vec::new();
    let lowered = lower_syntax_expr_with(&syntax, &mut |raw| {
        calls.push(raw.to_owned());
        None
    })
    .expect("resolver-order comparison source should lower");

    assert_eq!(calls.len(), 6);
    assert_eq!(
        calls,
        vec![
            "f".to_owned(),
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "g".to_owned(),
            "d".to_owned(),
        ]
    );
    assert_eq!(
        lowered.to_string(),
        "And[Less[f[a], b], LessEqual[b, c], Equal[c, g[d]]]"
    );
}

#[test]
fn comparison_lowering_accepts_a_uniform_chain_at_the_boundary() {
    const LINK_COUNT: usize = MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS;

    let source = comparison_source(LINK_COUNT);
    let syntax = parse_expr(&source).expect("boundary comparison source should parse");
    let lowered = lower_syntax_expr(&syntax).expect("boundary comparison source should lower");

    drop_left_binary_chain(syntax);

    let Expr::Call { head, args } = lowered else {
        panic!("boundary comparison should lower to a call");
    };
    assert!(is_system_symbol_name(&head, "Equal"));
    assert_eq!(args.len(), LINK_COUNT + 1);
    assert!(args.iter().all(is_global_a));
}

#[test]
fn comparison_parser_rejects_a_long_uniform_chain_gracefully() {
    const LINK_COUNT: usize = 50_000;

    let source = comparison_source(LINK_COUNT);
    let error = parse_expr(&source)
        .expect_err("long comparison source should be rejected without aborting");

    assert!(matches!(
        error,
        SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            ..
        }
    ));
    assert!(!error.is_incomplete());
}

#[test]
fn comparison_lowering_bounds_the_total_of_direct_ast_subtrees() {
    let accepted = SyntaxExpr::binary(
        BinaryOp::Equal,
        hand_built_comparison_chain(2_048, 1_000),
        hand_built_comparison_chain(2_047, 10_000),
        Span::new(30_000, 30_001),
    );
    let lowered = lower_syntax_expr(&accepted)
        .expect("4,096 comparison operators across both subtrees should lower");
    let Expr::Call { head, args } = &lowered else {
        panic!("direct comparison tree should lower to a call");
    };
    assert!(is_system_symbol_name(head, "Equal"));
    assert_eq!(args.len(), MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1);
    drop_binary_tree_iteratively(accepted);
    drop(lowered);

    let rejected_rhs_start = 20_000;
    let rejected = SyntaxExpr::binary(
        BinaryOp::Equal,
        hand_built_comparison_chain(2_048, 1_000),
        hand_built_comparison_chain(2_048, rejected_rhs_start),
        Span::new(30_000, 30_001),
    );
    let error = lower_syntax_expr(&rejected)
        .expect_err("4,097 direct comparison operators should be rejected");
    assert!(matches!(
        error,
        EvalError::Syntax(SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            span,
        }) if span == Span::new(rejected_rhs_start, rejected_rhs_start + 1)
    ));
    drop_binary_tree_iteratively(rejected);
}

#[test]
fn comparison_lowering_counts_explicitly_parenthesized_subtrees_together() {
    let side = comparison_source(2_048);
    let source = format!("({side}) == ({side})");
    let syntax =
        parse_expr(&source).expect("independent parenthesized comparison spines should parse");
    let error = lower_syntax_expr(&syntax)
        .expect_err("4,097 total comparison operators should be rejected during lowering");

    assert!(matches!(
        error,
        EvalError::Syntax(SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            ..
        })
    ));
    drop_binary_tree_iteratively(syntax);
}

fn comparison_source(link_count: usize) -> String {
    let mut source = String::with_capacity(link_count * 5 + 1);
    for index in 0..=link_count {
        if index != 0 {
            source.push_str(" == ");
        }
        source.push('a');
    }
    source
}

fn hand_built_comparison_chain(link_count: usize, span_start: usize) -> SyntaxExpr {
    let mut syntax = SyntaxExpr::symbol("a", Span::point(span_start));
    for index in 0..link_count {
        syntax = SyntaxExpr::binary(
            BinaryOp::Equal,
            syntax,
            SyntaxExpr::symbol("a", Span::point(span_start + index)),
            Span::new(span_start + index, span_start + index + 1),
        );
    }
    syntax
}

fn is_global_a(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Atom(Atom::Symbol(symbol))
            if symbol.context() == GLOBAL_CONTEXT && symbol.name() == "a"
    )
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

fn drop_binary_tree_iteratively(expr: SyntaxExpr) {
    let mut work = vec![expr];
    while let Some(expr) = work.pop() {
        if let SyntaxExprKind::Binary { lhs, rhs, .. } = expr.kind {
            work.push(*lhs);
            work.push(*rhs);
        }
    }
}
