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
fn parses_simple_decimal_literal_as_numeric_atom() {
    let parsed = parse_expr("0.01").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer("0.01", z()));
}

#[test]
fn parses_lists_and_nested_calls() {
    let parsed = parse_expr("f[{1, 2}, g[x]]").unwrap();
    assert_eq!(parsed.to_string(), "f[{1, 2}, g[x]]");
}

#[test]
fn parses_association_literals() {
    let parsed = parse_expr("<|a -> 1, b -> <|c -> 3|>|>").unwrap();
    assert_eq!(parsed.to_string(), "<|a -> 1, b -> <|c -> 3|>|>");
}

#[test]
fn parses_right_associative_apply_sugar() {
    let parsed = parse_expr("f @ g @ x").unwrap();
    assert_eq!(parsed.to_string(), "f @ g @ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::symbol("f", z()),
            SyntaxExpr::binary(
                BinaryOp::Apply,
                SyntaxExpr::symbol("g", z()),
                SyntaxExpr::symbol("x", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_compound_expression_sugar() {
    let parsed = parse_expr("a; b;").unwrap();
    assert_eq!(parsed.to_string(), "a; b; Null");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::CompoundExpression,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::CompoundExpression,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("Null", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_apply_and_composition_operator_precedence() {
    let parsed = parse_expr("f @* g @ x").unwrap();
    assert_eq!(parsed.to_string(), "f @* g @ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::binary(
                BinaryOp::Composition,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("g", z()),
                z(),
            ),
            SyntaxExpr::symbol("x", z()),
            z(),
        )
    );

    let right = parse_expr("f /* g @ x").unwrap();
    assert_eq!(right.to_string(), "f /* g @ x");
    assert_eq!(
        right,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::binary(
                BinaryOp::RightComposition,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("g", z()),
                z(),
            ),
            SyntaxExpr::symbol("x", z()),
            z(),
        )
    );
}

#[test]
fn parses_apply_level_operators_as_right_associative() {
    let parsed = parse_expr("f /@ g @@ h @@@ x").unwrap();
    assert_eq!(parsed.to_string(), "f /@ g @@ h @@@ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Map,
            SyntaxExpr::symbol("f", z()),
            SyntaxExpr::binary(
                BinaryOp::ApplyHead,
                SyntaxExpr::symbol("g", z()),
                SyntaxExpr::binary(
                    BinaryOp::MapApply,
                    SyntaxExpr::symbol("h", z()),
                    SyntaxExpr::symbol("x", z()),
                    z(),
                ),
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

    let mixed_chain = parse_expr("x == y != z").unwrap();
    assert_eq!(mixed_chain.to_string(), "x == y != z");
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
fn parses_function_postfix_as_map_lhs() {
    let parsed = parse_expr("typeSubstImpl[theta, #] & /@ args").unwrap();
    assert_eq!(parsed.to_string(), "(typeSubstImpl[theta, #] &) /@ args");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Map,
            SyntaxExpr::postfix(
                PostfixOp::Function,
                SyntaxExpr::call(
                    SyntaxExpr::symbol("typeSubstImpl", z()),
                    vec![
                        SyntaxExpr::symbol("theta", z()),
                        SyntaxExpr::slot(SlotSpec::Anonymous, z())
                    ],
                    z(),
                ),
                z(),
            ),
            SyntaxExpr::symbol("args", z()),
            z(),
        )
    );
}

#[test]
fn parses_function_postfix_on_set_rhs_before_map() {
    let parsed = parse_expr("ks = #[[1]] & /@ args").unwrap();
    assert_eq!(parsed.to_string(), "ks = (Part[#, 1] &) /@ args");
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
fn parses_rule_expression_as_pure_function_body() {
    let parsed = parse_expr("Map[instType[theta, #[[1]]] -> #[[2]] &, pairs]").unwrap();
    assert_eq!(
        parsed.to_string(),
        "Map[instType[theta, Part[#, 1]] -> Part[#, 2] &, pairs]"
    );
}

#[test]
fn does_not_merge_slots_across_trivia_into_a_sequence_slot() {
    let expected = SyntaxExpr::postfix(
        PostfixOp::Function,
        SyntaxExpr::binary(
            BinaryOp::Times,
            SyntaxExpr::slot(SlotSpec::Anonymous, z()),
            SyntaxExpr::slot(SlotSpec::Anonymous, z()),
            z(),
        ),
        z(),
    );

    assert_eq!(parse_expr("# # &").unwrap(), expected);

    assert_eq!(parse_expr("#(* gap *)# &").unwrap(), expected);
}

#[test]
fn does_not_merge_slot_specs_across_trivia() {
    assert_eq!(
        parse_expr("# 6 &").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::slot(SlotSpec::Anonymous, z()),
                SyntaxExpr::integer("6", z()),
                z(),
            ),
            z(),
        )
    );

    assert_eq!(
        parse_expr("# x &").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::slot(SlotSpec::Anonymous, z()),
                SyntaxExpr::symbol("x", z()),
                z(),
            ),
            z(),
        )
    );

    assert_eq!(
        parse_expr("## 2 &").unwrap(),
        SyntaxExpr::postfix(
            PostfixOp::Function,
            SyntaxExpr::binary(
                BinaryOp::Times,
                SyntaxExpr::slot(SlotSpec::Sequence, z()),
                SyntaxExpr::integer("2", z()),
                z(),
            ),
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
fn parses_rule_rhs_before_assignment_rhs() {
    let parsed = parse_expr("a -> x = x + 1").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Rule,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::Set,
                SyntaxExpr::symbol("x", z()),
                SyntaxExpr::binary(
                    BinaryOp::Add,
                    SyntaxExpr::symbol("x", z()),
                    SyntaxExpr::integer("1", z()),
                    z(),
                ),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_rule_delayed_rhs_before_assignment_rhs() {
    let parsed = parse_expr("a :> x = x + 1").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::RuleDelayed,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::Set,
                SyntaxExpr::symbol("x", z()),
                SyntaxExpr::binary(
                    BinaryOp::Add,
                    SyntaxExpr::symbol("x", z()),
                    SyntaxExpr::integer("1", z()),
                    z(),
                ),
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
fn reports_incomplete_double_bracket_input() {
    let err = parse_expr("expr[[1").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnclosedDelimiter {
            expected: "]]",
            span: Span::point(7),
            opening_span: Span::new(4, 6),
            opening: "[[",
        }
    );
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
    let err = parse_expr("f[1 ! 2]").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: ",",
            found: "!".to_string(),
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

#[test]
fn reports_incomplete_association_input() {
    let err = parse_expr("<|a -> 1").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnclosedDelimiter {
            expected: "|>",
            span: Span::point(8),
            opening_span: Span::new(0, 2),
            opening: "<|",
        }
    );
    assert!(err.is_incomplete());
}
