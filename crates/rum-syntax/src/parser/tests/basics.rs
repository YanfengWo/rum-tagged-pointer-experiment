use super::support::*;

#[test]
fn parses_large_integer_literal() {
    let parsed = parse_expr("123456789012345678901234567890").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::integer("123456789012345678901234567890", z())
    );
}

#[test]
fn parses_simple_decimal_literal_as_numeric_atom() {
    let parsed = parse_expr("0.01").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer("0.01", z()));

    let parsed = parse_expr(".5").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer(".5", z()));

    let parsed = parse_expr("1.").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer("1.", z()));

    let parsed = parse_expr("1.5e3").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer("1.5e3", z()));
}

#[test]
fn parses_lists_and_nested_calls() {
    let parsed = parse_expr("f[{1, 2}, g[x]]").unwrap();
    assert_eq!(parsed.to_string(), "f[{1, 2}, g[x]]");
}

#[test]
fn parses_multiline_string_literals_as_string_atoms() {
    assert_eq!(
        parse_expr("\"\"\"\nab\"\"\"").unwrap(),
        SyntaxExpr::string("ab", z())
    );
    assert_eq!(
        parse_expr("\"\"\"a \" b \"\" c\"\"\"").unwrap(),
        SyntaxExpr::string("a \" b \"\" c", z())
    );
    assert_eq!(
        parse_expr("x = '''\nC:\\path \\t raw\n'''").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Set,
            SyntaxExpr::symbol("x", z()),
            SyntaxExpr::string("C:\\path \\t raw\n", z()),
            z(),
        )
    );
}

#[test]
fn rejects_too_deep_nested_call_without_panicking() {
    let depth = MAX_PARSE_DEPTH + 1;
    let mut input = String::with_capacity(depth * 3 + 1);
    for _ in 0..depth {
        input.push_str("f[");
    }
    input.push('x');
    for _ in 0..depth {
        input.push(']');
    }

    let error = parse_expr(&input).expect_err("too-deep call should be a parse error");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_PARSE_DEPTH,
            ..
        }
    ));
}

#[test]
fn pins_nested_call_parse_depth_boundary() {
    fn nested_call(depth: usize) -> String {
        let mut input = String::with_capacity(depth * 3 + 1);
        for _ in 0..depth {
            input.push_str("f[");
        }
        input.push('x');
        for _ in 0..depth {
            input.push(']');
        }
        input
    }

    let accepted = nested_call(127);
    assert_eq!(parse_expr(&accepted).unwrap().to_string(), accepted);

    let rejected = nested_call(129);
    let error = parse_expr(&rejected).expect_err("depth 129 should exceed the parser guard");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_PARSE_DEPTH,
            ..
        }
    ));
}

#[test]
fn rejects_deep_right_associative_chain_without_panicking() {
    let mut input = String::with_capacity(32_001);
    input.push('a');
    for _ in 0..16_000 {
        input.push_str("^a");
    }

    let error = parse_expr(&input).expect_err("deep power chain should exceed the parser guard");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_PARSE_DEPTH,
            ..
        }
    ));
}

#[test]
fn rejects_deep_prefix_chain_without_panicking() {
    let input = format!("{}x", "!".repeat(16_000));

    let error = parse_expr(&input).expect_err("deep prefix chain should exceed the parser guard");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_PARSE_DEPTH,
            ..
        }
    ));
}

#[test]
fn parses_association_literals() {
    let parsed = parse_expr("<|a -> 1, b -> <|c -> 3|>|>").unwrap();
    assert_eq!(parsed.to_string(), "<|a -> 1, b -> <|c -> 3|>|>");
}
