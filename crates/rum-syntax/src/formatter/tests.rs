use super::*;

fn format_with_options(source: &str, options: FormatterOptions) -> String {
    let parsed = parse_with_diagnostics(source).expect("source should parse for formatter test");
    format_parsed_source(&parsed, options)
}

#[test]
fn formatter_canonicalizes_core_spacing() {
    let formatted = format_source("f [ x_Integer,  y]&").unwrap();
    assert_eq!(formatted, "f[x_Integer, y] &");
}

#[test]
fn formatter_preserves_plain_multiline_call_arguments() {
    let formatted = format_source("f[x,\ny]").unwrap();
    assert_eq!(formatted, "f[\n  x,\n  y\n]");
}

#[test]
fn formatter_preserves_plain_multiline_nested_call_arguments() {
    let formatted = format_source("f[g[x,\ny],\nz]").unwrap();
    assert_eq!(formatted, "f[\n  g[\n    x,\n    y\n  ],\n  z\n]");
}

#[test]
fn formatter_preserves_plain_multiline_list_elements() {
    let formatted = format_source("f[{a,\nb},\nc]").unwrap();
    assert_eq!(formatted, "f[\n  {\n    a,\n    b\n  },\n  c\n]");
}

#[test]
fn formatter_preserves_plain_multiline_binary_rhs_inside_calls() {
    let formatted = format_source("f[(x +\ny),\nz]").unwrap();
    assert_eq!(formatted, "f[\n  (x +\n    y),\n  z\n]");
}

#[test]
fn formatter_leaves_top_level_multiline_binary_rhs_flush_left() {
    let formatted = format_source("x +\ny").unwrap();
    assert_eq!(formatted, "x +\ny");
}

#[test]
fn formatter_keeps_max_width_binary_expressions_inline() {
    let formatted = format_source("aaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbb").unwrap();
    assert_eq!(formatted, "aaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbb");
}

#[test]
fn formatter_actively_expands_wide_top_level_binary_expressions() {
    let formatted = format_source("aaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbb").unwrap();
    assert_eq!(formatted, "aaaaaaaaaaaaaaaaaaa +\nbbbbbbbbbbbbbbbbbbb");
}

#[test]
fn formatter_actively_expands_wide_binary_rhs_inside_calls() {
    let formatted = format_source("f[aaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbb]").unwrap();
    assert_eq!(formatted, "f[aaaaaaaaaaaaaaaaaaa +\n  bbbbbbbbbbbbbbbbbbb]");
}

#[test]
fn formatter_actively_expands_binary_expressions_with_long_inline_comments() {
    let formatted = format_with_options(
        "aaaaaaaa + (* very very very long inline comment *) bbbbbbbb",
        FormatterOptions {
            line_width: 30,
            ..FormatterOptions::default()
        },
    );
    assert_eq!(
        formatted,
        "aaaaaaaa +\n(* very very very long inline comment *)\nbbbbbbbb"
    );
}

#[test]
fn formatter_respects_custom_line_width_for_calls() {
    let formatted = format_with_options(
        "f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]",
        FormatterOptions {
            line_width: 60,
            ..FormatterOptions::default()
        },
    );
    assert_eq!(formatted, "f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]");
}

#[test]
fn formatter_actively_expands_calls_with_long_inline_comments() {
    let formatted = format_with_options(
        "f[x (* very very very long inline comment *), y]",
        FormatterOptions {
            line_width: 30,
            ..FormatterOptions::default()
        },
    );
    assert_eq!(
        formatted,
        "f[\n  x (* very very very long inline comment *),\n  y\n]"
    );
}

#[test]
fn formatter_actively_expands_calls_with_escaped_string_literals() {
    let formatted = format_with_options(
        "f[\"aaaaaaaaaa\\\"bbbbbbbbbb\", z]",
        FormatterOptions {
            line_width: 29,
            ..FormatterOptions::default()
        },
    );
    assert_eq!(formatted, "f[\n  \"aaaaaaaaaa\\\"bbbbbbbbbb\",\n  z\n]");
}

#[test]
fn formatter_respects_custom_line_width_for_binary_chains() {
    let formatted = format_with_options(
        "aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc",
        FormatterOptions {
            line_width: 60,
            ..FormatterOptions::default()
        },
    );
    assert_eq!(
        formatted,
        "aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc"
    );
}

#[test]
fn formatter_accounts_for_current_column_when_fitting_nested_groups() {
    let formatted = format_with_options(
        "f[aaaaaaaa + bbbbbbbb]",
        FormatterOptions {
            line_width: 20,
            ..FormatterOptions::default()
        },
    );
    assert_eq!(formatted, "f[aaaaaaaa +\n  bbbbbbbb]");
}

#[test]
fn formatter_keeps_short_binary_chains_inline() {
    let formatted = format_source("a + b + c + d").unwrap();
    assert_eq!(formatted, "a + b + c + d");
}

#[test]
fn formatter_actively_expands_wide_top_level_binary_chains() {
    let formatted = format_source("aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc").unwrap();
    assert_eq!(
        formatted,
        "aaaaaaaaaaaaaa +\nbbbbbbbbbbbbbb +\ncccccccccccccc"
    );
}

#[test]
fn formatter_actively_expands_wide_binary_chains_inside_calls() {
    let formatted = format_source("f[aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc]").unwrap();
    assert_eq!(
        formatted,
        "f[aaaaaaaaaaaaaa +\n  bbbbbbbbbbbbbb +\n  cccccccccccccc]"
    );
}

#[test]
fn formatter_keeps_parent_calls_inline_when_nested_binary_chains_fit() {
    let formatted = format_source("f[(a +\nb + c), z]").unwrap();
    assert_eq!(formatted, "f[(a +\n    b +\n    c), z]");
}

#[test]
fn formatter_expands_parent_calls_when_nested_binary_chains_make_the_parent_wide() {
    let formatted =
        format_source("f[(aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc), z]").unwrap();
    assert_eq!(
        formatted,
        "f[\n  (aaaaaaaaaaaaaa +\n    bbbbbbbbbbbbbb +\n    cccccccccccccc),\n  z\n]"
    );
}

#[test]
fn formatter_wraps_right_associative_assignment_chains() {
    let formatted = format_source("aaaaaaaaaaaaaa = bbbbbbbbbbbbbb = cccccccccccccc").unwrap();
    assert_eq!(
        formatted,
        "aaaaaaaaaaaaaa =\nbbbbbbbbbbbbbb =\ncccccccccccccc"
    );
}

#[test]
fn formatter_normalizes_partially_multiline_binary_chains() {
    let formatted = format_source("a +\nb + c").unwrap();
    assert_eq!(formatted, "a +\nb +\nc");
}

#[test]
fn formatter_normalizes_binary_chains_with_middle_multiline_comments() {
    let formatted = format_source("a +\n(* note *) b + c").unwrap();
    assert_eq!(formatted, "a +\n(* note *)\nb +\nc");
}

#[test]
fn formatter_normalizes_binary_chains_with_trailing_multiline_comments() {
    let formatted = format_source("a + b +\n(* tail note *) c").unwrap();
    assert_eq!(formatted, "a +\nb +\n(* tail note *)\nc");
}

#[test]
fn formatter_indents_binary_chains_with_multiline_comments_inside_calls() {
    let formatted = format_source("f[a +\n(* note *) b + c]").unwrap();
    assert_eq!(formatted, "f[a +\n  (* note *)\n  b +\n  c]");
}

#[test]
fn formatter_indents_binary_chains_with_multiline_comments_inside_parenthesized_calls() {
    let formatted = format_source("f[(a +\n(* note *) b + c), z]").unwrap();
    assert_eq!(formatted, "f[(a +\n    (* note *)\n    b +\n    c), z]");
}

#[test]
fn formatter_normalizes_binary_chains_with_mixed_inline_and_multiline_comments() {
    let formatted = format_source("a + (* mid inline *) b +\n(* tail block *) c").unwrap();
    assert_eq!(formatted, "a +\n(* mid inline *)\nb +\n(* tail block *)\nc");
}

#[test]
fn formatter_keeps_lhs_inline_comments_while_expanding_binary_chains() {
    let formatted = format_source("a (* lhs inline *) + b +\n(* tail block *) c").unwrap();
    assert_eq!(formatted, "a (* lhs inline *) +\nb +\n(* tail block *)\nc");
}

#[test]
fn formatter_indents_binary_chains_with_mixed_inline_and_multiline_comments_inside_calls() {
    let formatted = format_source("f[a + (* mid inline *) b +\n(* tail block *) c]").unwrap();
    assert_eq!(
        formatted,
        "f[a +\n  (* mid inline *)\n  b +\n  (* tail block *)\n  c]"
    );
}

#[test]
fn formatter_preserves_outer_comment_trivia() {
    let formatted = format_source("(* header *)\n f[x] (* tail *)").unwrap();
    assert_eq!(formatted, "(* header *)\nf[x] (* tail *)");
}

#[test]
fn formatter_preserves_multiline_string_literals_verbatim() {
    let source = "escaped =\n\"\"\"\na,\"b\"\n\"\"\";\n(* note *)\nraw =\n'''\nC:\\path \\t raw\n''';\nplain = \"ok\"";

    let formatted = format_source(source).unwrap();

    assert_eq!(formatted, source);
}

#[test]
fn formatter_preserves_inline_comments_inside_calls_and_binary_ops() {
    let formatted = format_source("f[x(* inner *)+y,(* note *)z]").unwrap();
    assert_eq!(formatted, "f[x (* inner *) + y, (* note *) z]");
}

#[test]
fn formatter_keeps_multiline_comments_attached_in_place() {
    let formatted = format_source("f[x,\n(* note *) y]").unwrap();
    assert_eq!(formatted, "f[\n  x,\n  (* note *)\n  y\n]");
}

#[test]
fn formatter_indents_multiline_comments_inside_nested_calls() {
    let formatted = format_source("f[g[x,\n(* note *) y], z]").unwrap();
    assert_eq!(formatted, "f[g[\n    x,\n    (* note *)\n    y\n  ], z]");
}

#[test]
fn formatter_dedents_closing_delimiters_after_multiline_comments() {
    let formatted = format_source("f[x\n(* note *)]").unwrap();
    assert_eq!(formatted, "f[\n  x\n  (* note *)\n]");
}

#[test]
fn formatter_indents_multiline_comments_before_binary_rhs_inside_calls() {
    let formatted = format_source("f[x +\n(* note *) y]").unwrap();
    assert_eq!(formatted, "f[x +\n  (* note *)\n  y]");
}

#[test]
fn formatter_keeps_stacked_multiline_comments_grouped_with_the_following_arg() {
    let formatted = format_source("f[x,\n(* first *)\n(* second *) y]").unwrap();
    assert_eq!(formatted, "f[\n  x,\n  (* first *)\n  (* second *)\n  y\n]");
}

#[test]
fn formatter_indents_multiline_comments_inside_nested_lists() {
    let formatted = format_source("f[{a,\n(* list note *) b}, c]").unwrap();
    assert_eq!(
        formatted,
        "f[{\n    a,\n    (* list note *)\n    b\n  }, c]"
    );
}

#[test]
fn formatter_indents_multiline_comments_before_nested_outer_arguments() {
    let formatted = format_source("f[g[x],\n(* outer *) h[z]]").unwrap();
    assert_eq!(formatted, "f[\n  g[x],\n  (* outer *)\n  h[z]\n]");
}

#[test]
fn formatter_dedents_nested_closing_delimiters_after_multiline_comments() {
    let formatted = format_source("f[g[x\n(* inner close *)], z]").unwrap();
    assert_eq!(formatted, "f[g[\n    x\n    (* inner close *)\n  ], z]");
}

#[test]
fn formatter_indents_multiline_comments_inside_parenthesized_binary_rhs() {
    let formatted = format_source("f[(x +\n(* rhs note *) g[y]), z]").unwrap();
    assert_eq!(formatted, "f[(x +\n    (* rhs note *)\n    g[y]), z]");
}

#[test]
fn formatter_handles_mixed_inline_and_multiline_comments_inside_nested_calls() {
    let formatted = format_source("f[g[x, (* inline *) y,\n(* block *) h[z]], w]").unwrap();
    assert_eq!(
        formatted,
        "f[\n  g[\n    x,\n    (* inline *)\n    y,\n    (* block *)\n    h[z]\n  ],\n  w\n]"
    );
}

#[test]
fn formatter_handles_multiline_comments_at_multiple_nesting_levels() {
    let formatted =
        format_source("f[g[x,\n(* inner block *) h[y, (* tail *) z]],\n(* outer block *) q[r]]")
            .unwrap();
    assert_eq!(
        formatted,
        "f[\n  g[\n    x,\n    (* inner block *)\n    h[y, (* tail *) z]\n  ],\n  (* outer block *)\n  q[r]\n]"
    );
}

#[test]
fn formatter_handles_mixed_inline_and_multiline_comments_inside_nested_lists() {
    let formatted =
        format_source("f[{a, (* list inline *) b,\n(* list block *) {c, d}}, t]").unwrap();
    assert_eq!(
        formatted,
        "f[\n  {\n    a,\n    (* list inline *)\n    b,\n    (* list block *)\n    {c, d}\n  },\n  t\n]"
    );
}

#[test]
fn formatter_handles_multiline_comments_in_parenthesized_binary_expressions_with_inline_comments() {
    let formatted =
        format_source("f[(x (* left inline *) +\n(* rhs block *) g[y, (* arg inline *) z]), u]")
            .unwrap();
    assert_eq!(
        formatted,
        "f[\n  (x (* left inline *) +\n    (* rhs block *)\n    g[y, (* arg inline *) z]),\n  u\n]"
    );
}

#[test]
fn formatter_handles_deeply_nested_multiline_comments_with_following_inline_comments() {
    let formatted = format_source(
        "f[g[(x +\n(* deep block *) y), (* after paren *) z],\n(* outer note *) {m, n}]",
    )
    .unwrap();
    assert_eq!(
        formatted,
        "f[\n  g[\n    (x +\n      (* deep block *)\n      y),\n    (* after paren *)\n    z\n  ],\n  (* outer note *)\n  {m, n}\n]"
    );
}

#[test]
fn formatter_handles_stacked_comments_before_nested_closing_delimiters() {
    let formatted = format_source("f[g[x\n(* first *)\n(* second *)],\n(* outer *) y]").unwrap();
    assert_eq!(
        formatted,
        "f[\n  g[\n    x\n    (* first *)\n    (* second *)\n  ],\n  (* outer *)\n  y\n]"
    );
}

#[test]
fn formatter_handles_stacked_rhs_comments_inside_parenthesized_calls() {
    let formatted =
        format_source("f[(x +\n(* first rhs *)\n(* second rhs *) h[y]),\n(* outer *) z]").unwrap();
    assert_eq!(
        formatted,
        "f[\n  (x +\n    (* first rhs *)\n    (* second rhs *)\n    h[y]),\n  (* outer *)\n  z\n]"
    );
}

#[test]
fn formatter_handles_nested_lists_followed_by_outer_comment_blocks() {
    let formatted =
        format_source("f[g[{a,\n(* list block *) b}, c],\n(* outer *) q[r, (* inline *) s]]")
            .unwrap();
    assert_eq!(
        formatted,
        "f[\n  g[{\n      a,\n      (* list block *)\n      b\n    }, c],\n  (* outer *)\n  q[r, (* inline *) s]\n]"
    );
}

#[test]
fn formatter_actively_expands_long_call_argument_lists() {
    let formatted = format_source("f[a, b, c, d]").unwrap();
    assert_eq!(formatted, "f[\n  a,\n  b,\n  c,\n  d\n]");
}

#[test]
fn formatter_actively_expands_long_list_literals() {
    let formatted = format_source("f[{a, b, c, d}, z]").unwrap();
    assert_eq!(formatted, "f[{\n    a,\n    b,\n    c,\n    d\n  }, z]");
}

#[test]
fn formatter_indexed_break_points_preserve_every_wide_list_item() {
    let items = (1..=20)
        .map(|index| format!("x{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("{{{items}}}");
    let formatted = format_source(&source).unwrap();

    assert_eq!(formatted.lines().count(), 22);
    assert_eq!(formatted.matches("\n  x").count(), 20);
    assert_eq!(format_source(&formatted).unwrap(), formatted);
}

#[test]
fn formatter_handles_association_literals() {
    let formatted = format_source("<|a->1,b-><|c->3,d->4|>|>").unwrap();
    assert_eq!(formatted, "<|a -> 1, b -> <|c -> 3, d -> 4|>|>");
}

#[test]
fn formatter_spaces_apply_sugar() {
    let formatted = format_source("f@g@x").unwrap();
    assert_eq!(formatted, "f @ g @ x");
}

#[test]
fn formatter_spaces_compound_expression_sugar() {
    assert_eq!(format_source("a;b").unwrap(), "a; b");
    assert_eq!(format_source("a;b;").unwrap(), "a; b;");
}

#[test]
fn formatter_spaces_functional_operator_sugars() {
    assert_eq!(format_source("f@@g").unwrap(), "f @@ g");
    assert_eq!(format_source("f@@@xs").unwrap(), "f @@@ xs");
    assert_eq!(format_source("f@*g@x").unwrap(), "f @* g @ x");
    assert_eq!(format_source("f/*g@x").unwrap(), "f /* g @ x");
}

#[test]
fn formatter_expands_long_nested_calls_without_expanding_the_parent() {
    let formatted = format_source("f[g[a, b, c, d], z]").unwrap();
    assert_eq!(formatted, "f[g[\n    a,\n    b,\n    c,\n    d\n  ], z]");
}

#[test]
fn formatter_keeps_max_width_calls_inline() {
    let formatted = format_source("f[aaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]").unwrap();
    assert_eq!(formatted, "f[aaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]");
}

#[test]
fn formatter_actively_expands_wide_two_argument_calls() {
    let formatted = format_source("f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]").unwrap();
    assert_eq!(
        formatted,
        "f[\n  aaaaaaaaaaaaaaaaaa,\n  bbbbbbbbbbbbbbbbbb\n]"
    );
}

#[test]
fn formatter_actively_expands_wide_top_level_lists() {
    let formatted = format_source("{aaaaaaaaaaaa, bbbbbbbbbbbb, cccccccccccc}").unwrap();
    assert_eq!(
        formatted,
        "{\n  aaaaaaaaaaaa,\n  bbbbbbbbbbbb,\n  cccccccccccc\n}"
    );
}

#[test]
fn formatter_expands_wide_parent_calls_around_nested_lists() {
    let formatted = format_source("f[{aaaaaaaaaaaa, bbbbbbbbbbbb, cccccccccccc}, z]").unwrap();
    assert_eq!(
        formatted,
        "f[\n  {\n    aaaaaaaaaaaa,\n    bbbbbbbbbbbb,\n    cccccccccccc\n  },\n  z\n]"
    );
}

#[test]
fn formatter_is_idempotent_on_regression_corpus() {
    let corpus = [
        "f [ x_Integer,  y]&",
        "f[g[x,\ny],\nz]",
        "f[(x +\ny),\nz]",
        "aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc",
        "a + (* mid inline *) b +\n(* tail block *) c",
        "f[g[x, (* inline *) y,\n(* block *) h[z]], w]",
        "f[{a, (* list inline *) b,\n(* list block *) {c, d}}, t]",
        "f[{aaaaaaaaaaaa, bbbbbbbbbbbb, cccccccccccc}, z]",
        "aaaaaaaaaaaaaa = bbbbbbbbbbbbbb = cccccccccccccc",
        "(* header *)\n f[x] (* tail *)",
    ];

    for source in corpus {
        let once = format_source(source).expect("corpus case should format");
        let twice = format_source(&once).expect("formatted corpus case should reformat");
        assert_eq!(twice, once, "formatter should be idempotent for {source:?}");
    }
}
