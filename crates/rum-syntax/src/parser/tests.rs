use crate::ast::{BinaryOp, BlankKind, PostfixOp, SlotSpec, SyntaxExpr, SyntaxExprKind};
use crate::{Span, SyntaxError};

use super::parse_expr;

fn z() -> Span {
    Span::default()
}

#[test]
fn parses_large_integer_literal() {
    let parsed = parse_expr("123456789012345678901234567890").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::integer("123456789012345678901234567890", z())
    );
}

#[test]
fn parses_lists_and_nested_calls() {
    let parsed = parse_expr("f[{1, 2}, g[x]]").unwrap();
    assert_eq!(parsed.to_string(), "f[{1, 2}, g[x]]");
}

#[test]
fn parses_expressions_with_comments_between_tokens() {
    let parsed = parse_expr("f[(* a *) x, y (* b *)]").unwrap();
    assert_eq!(parsed.to_string(), "f[x, y]");

    let nested = parse_expr("x (* outer (* inner *) *) + y").unwrap();
    assert_eq!(nested.to_string(), "x + y");
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
fn parses_prefix_minus_with_power_precedence() {
    let parsed = parse_expr("-x^2").unwrap();
    assert_eq!(parsed.to_string(), "-x ^ 2");
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

#[test]
fn reports_incomplete_call_input() {
    let err = parse_expr("f[1").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnclosedDelimiter {
            expected: "]",
            span: Span::point(3),
            opening_span: Span::new(1, 2),
            opening: "[",
        }
    );
    assert!(err.is_incomplete());
    assert_eq!(err.span(), Some(Span::point(3)));
    assert_eq!(err.related_span(), Some(Span::new(1, 2)));
}

#[test]
fn reports_incomplete_group_input() {
    let err = parse_expr("(x + 1").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnclosedDelimiter {
            expected: ")",
            span: Span::point(6),
            opening_span: Span::new(0, 1),
            opening: "(",
        }
    );
}

#[test]
fn reports_incomplete_rhs_after_binary_operator() {
    let err = parse_expr("x +").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::Incomplete {
            expected: "expression",
            span: Span::point(3),
        }
    );
}

#[test]
fn reports_malformed_call_arguments_without_marking_input_incomplete() {
    let err = parse_expr("f[1 2]").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: ",",
            found: "integer".to_string(),
            span: Span::new(4, 5),
        }
    );
    assert!(!err.is_incomplete());
}

#[test]
fn reports_malformed_list_items_without_marking_input_incomplete() {
    let err = parse_expr("{1,,2}").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: "expression",
            found: ",".to_string(),
            span: Span::new(3, 4),
        }
    );
    assert!(!err.is_incomplete());
}

#[test]
fn reports_trailing_pattern_tokens_as_hard_errors() {
    let err = parse_expr("x_,").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::TrailingInput {
            span: Span::new(2, 3),
        }
    );
    assert!(!err.is_incomplete());
}

#[test]
fn reports_bare_pure_function_marker_as_hard_error() {
    let err = parse_expr("&").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: "expression",
            found: "&".to_string(),
            span: Span::new(0, 1),
        }
    );
    assert!(!err.is_incomplete());
}

#[test]
fn reports_unexpected_closing_delimiters_with_token_labels() {
    let err = parse_expr(")").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: "expression",
            found: ")".to_string(),
            span: Span::new(0, 1),
        }
    );
}

#[test]
fn distinguishes_empty_input_from_incomplete_input() {
    let err = parse_expr("").unwrap_err();

    assert_eq!(err, SyntaxError::EmptyInput);
    assert!(!err.is_incomplete());
    assert_eq!(err.span(), None);
}
