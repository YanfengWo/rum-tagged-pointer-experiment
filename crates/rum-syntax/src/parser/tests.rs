use crate::ast::{BinaryOp, BlankKind, PostfixOp, SlotSpec, SyntaxExpr};

use super::parse_expr;

#[test]
fn parses_large_integer_literal() {
    let parsed = parse_expr("123456789012345678901234567890").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::Integer("123456789012345678901234567890".to_string())
    );
}

#[test]
fn parses_lists_and_nested_calls() {
    let parsed = parse_expr("f[{1, 2}, g[x]]").unwrap();
    assert_eq!(parsed.to_string(), "f[{1, 2}, g[x]]");
}

#[test]
fn parses_named_patterns() {
    let parsed = parse_expr("x_Integer").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::Pattern {
            name: Box::new(SyntaxExpr::Symbol("x".to_string())),
            pattern: Box::new(SyntaxExpr::Blank {
                kind: BlankKind::Blank,
                head: Some(Box::new(SyntaxExpr::Symbol("Integer".to_string()))),
            }),
        }
    );
}

#[test]
fn parses_rule_and_replace_all_precedence() {
    let parsed = parse_expr("expr /. x_Integer -> y").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::Binary {
            op: BinaryOp::ReplaceAll,
            lhs: Box::new(SyntaxExpr::Symbol("expr".to_string())),
            rhs: Box::new(SyntaxExpr::Binary {
                op: BinaryOp::Rule,
                lhs: Box::new(SyntaxExpr::Pattern {
                    name: Box::new(SyntaxExpr::Symbol("x".to_string())),
                    pattern: Box::new(SyntaxExpr::Blank {
                        kind: BlankKind::Blank,
                        head: Some(Box::new(SyntaxExpr::Symbol("Integer".to_string()))),
                    }),
                }),
                rhs: Box::new(SyntaxExpr::Symbol("y".to_string())),
            }),
        }
    );
}

#[test]
fn parses_replace_repeated_precedence() {
    let parsed = parse_expr("expr //. x_Integer -> y").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::Binary {
            op: BinaryOp::ReplaceRepeated,
            lhs: Box::new(SyntaxExpr::Symbol("expr".to_string())),
            rhs: Box::new(SyntaxExpr::Binary {
                op: BinaryOp::Rule,
                lhs: Box::new(SyntaxExpr::Pattern {
                    name: Box::new(SyntaxExpr::Symbol("x".to_string())),
                    pattern: Box::new(SyntaxExpr::Blank {
                        kind: BlankKind::Blank,
                        head: Some(Box::new(SyntaxExpr::Symbol("Integer".to_string()))),
                    }),
                }),
                rhs: Box::new(SyntaxExpr::Symbol("y".to_string())),
            }),
        }
    );
}

#[test]
fn parses_slots_and_pure_functions() {
    let parsed = parse_expr("#1 + #2 &").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::Postfix {
            op: PostfixOp::Function,
            expr: Box::new(SyntaxExpr::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(SyntaxExpr::Slot(SlotSpec::Number("1".to_string()))),
                rhs: Box::new(SyntaxExpr::Slot(SlotSpec::Number("2".to_string()))),
            }),
        }
    );

    let sequence_parsed = parse_expr("## + ##2 &").unwrap();
    assert_eq!(
        sequence_parsed,
        SyntaxExpr::Postfix {
            op: PostfixOp::Function,
            expr: Box::new(SyntaxExpr::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(SyntaxExpr::Slot(SlotSpec::Sequence)),
                rhs: Box::new(SyntaxExpr::Slot(SlotSpec::SequenceNumber("2".to_string()))),
            }),
        }
    );

    let named_parsed = parse_expr("#x &").unwrap();
    assert_eq!(
        named_parsed,
        SyntaxExpr::Postfix {
            op: PostfixOp::Function,
            expr: Box::new(SyntaxExpr::Slot(SlotSpec::Named("x".to_string()))),
        }
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
        SyntaxExpr::Binary {
            op: BinaryOp::Set,
            lhs: Box::new(SyntaxExpr::Binary {
                op: BinaryOp::MessageName,
                lhs: Box::new(SyntaxExpr::Symbol("f".to_string())),
                rhs: Box::new(SyntaxExpr::Symbol("argx".to_string())),
            }),
            rhs: Box::new(SyntaxExpr::String("bad arg `1`".to_string())),
        }
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
        SyntaxExpr::Binary {
            op: BinaryOp::TagSet,
            lhs: Box::new(SyntaxExpr::Symbol("f".to_string())),
            rhs: Box::new(SyntaxExpr::Binary {
                op: BinaryOp::SetDelayed,
                lhs: Box::new(SyntaxExpr::Call {
                    head: Box::new(SyntaxExpr::Symbol("g".to_string())),
                    args: vec![SyntaxExpr::Pattern {
                        name: Box::new(SyntaxExpr::Symbol("x".to_string())),
                        pattern: Box::new(SyntaxExpr::Blank {
                            kind: BlankKind::Blank,
                            head: None,
                        }),
                    }],
                }),
                rhs: Box::new(SyntaxExpr::Symbol("rhs".to_string())),
            }),
        }
    );
}

#[test]
fn parses_unset_postfix() {
    let parsed = parse_expr("f[x_] =.").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::Postfix {
            op: PostfixOp::Unset,
            expr: Box::new(SyntaxExpr::Call {
                head: Box::new(SyntaxExpr::Symbol("f".to_string())),
                args: vec![SyntaxExpr::Pattern {
                    name: Box::new(SyntaxExpr::Symbol("x".to_string())),
                    pattern: Box::new(SyntaxExpr::Blank {
                        kind: BlankKind::Blank,
                        head: None,
                    }),
                }],
            }),
        }
    );
}
