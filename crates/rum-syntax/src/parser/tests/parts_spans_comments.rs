use super::support::*;

#[test]
fn parses_part_and_span_sugar_into_canonical_forms() {
    assert_eq!(
        parse_expr("expr[[1, 2]]").unwrap().to_string(),
        "Part[expr, 1, 2]"
    );
    assert_eq!(
        parse_expr("expr[[;;, 2 ;; ;; 2]]").unwrap().to_string(),
        "Part[expr, Span[All], Span[2, All, 2]]"
    );
    assert_eq!(parse_expr("a ;; b").unwrap().to_string(), "Span[a, b]");
    assert_eq!(parse_expr("a ;;").unwrap().to_string(), "Span[a, All]");
    assert_eq!(parse_expr(";; b").unwrap().to_string(), "Span[All, b]");
    assert_eq!(parse_expr(";;").unwrap().to_string(), "Span[All]");
    assert_eq!(
        parse_expr("a ;; b ;; c").unwrap().to_string(),
        "Span[a, b, c]"
    );
    assert_eq!(
        parse_expr("a ;; ;; c").unwrap().to_string(),
        "Span[a, All, c]"
    );
    assert_eq!(
        parse_expr(";; ;; c").unwrap().to_string(),
        "Span[All, All, c]"
    );
}

#[test]
fn parses_expressions_with_comments_between_tokens() {
    let parsed = parse_expr("f[(* a *) x, y (* b *)]").unwrap();
    assert_eq!(parsed.to_string(), "f[x, y]");

    let nested = parse_expr("x (* outer (* inner *) *) + y").unwrap();
    assert_eq!(nested.to_string(), "x + y");
}

#[test]
fn attaches_spans_to_root_and_nested_nodes() {
    let parsed = parse_expr("f[{x}, y]").unwrap();

    assert_eq!(parsed.span(), Span::new(0, 9));

    let SyntaxExprKind::Call { head, args } = &parsed.kind else {
        panic!("expected call node");
    };

    assert_eq!(head.span(), Span::new(0, 1));
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].span(), Span::new(2, 5));
    assert_eq!(args[1].span(), Span::new(7, 8));

    let SyntaxExprKind::List(items) = &args[0].kind else {
        panic!("expected list node");
    };

    assert_eq!(items[0].span(), Span::new(3, 4));
}

#[test]
fn keeps_group_and_postfix_delimiters_inside_node_spans() {
    let parsed = parse_expr("(x + 1)&").unwrap();

    assert_eq!(parsed.span(), Span::new(0, 8));

    let SyntaxExprKind::Postfix { expr, .. } = &parsed.kind else {
        panic!("expected postfix node");
    };

    assert_eq!(expr.span(), Span::new(0, 7));
}
