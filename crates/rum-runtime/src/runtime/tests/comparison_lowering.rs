use super::*;

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
fn comparison_lowering_handles_a_long_uniform_chain_iteratively() {
    const LINK_COUNT: usize = 50_000;

    let mut source = String::with_capacity(LINK_COUNT * 5 + 1);
    for index in 0..=LINK_COUNT {
        if index != 0 {
            source.push_str(" == ");
        }
        source.push('a');
    }

    let syntax = parse_expr(&source).expect("long comparison source should parse");
    let lowered = lower_syntax_expr(&syntax).expect("long comparison source should lower");

    // SyntaxExpr destruction is outside this brief. Consume the known left
    // spine iteratively so this test isolates collection and output shape.
    drop_left_binary_chain(syntax);

    let Expr::Call { head, args } = lowered else {
        panic!("long comparison should lower to a call");
    };
    assert!(is_system_symbol_name(&head, "Equal"));
    assert_eq!(args.len(), LINK_COUNT + 1);
    assert!(args.iter().all(|arg| {
        matches!(
            arg,
            Expr::Atom(Atom::Symbol(symbol))
                if symbol.context() == GLOBAL_CONTEXT && symbol.name() == "a"
        )
    }));
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
