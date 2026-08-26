use super::*;
use crate::structure_ops::eval_sort;
use rum_core::{Atom, Expr, GLOBAL_CONTEXT, Rational, SYSTEM_CONTEXT, Symbol};

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn sort_symbols_by_qualified_name(mut symbols: Vec<Symbol>) -> Vec<Symbol> {
    symbols.sort_by(Symbol::cmp_qualified_name);
    symbols
}

fn sort_exprs(session: &mut Session, items: Vec<Expr>) -> Expr {
    let list = Expr::call(Expr::system_symbol("List"), items);
    eval_sort(session, &Expr::system_symbol("Sort"), &[list]).unwrap()
}

#[test]
fn qualified_symbol_name_order_matches_the_allocating_text_order_exactly() {
    let same_a = Symbol::new("Ctx`", "a");
    let same_ab = Symbol::new("Ctx`", "ab");
    let same_b = Symbol::new("Ctx`", "b");
    assert_eq!(
        sort_symbols_by_qualified_name(vec![same_b, same_ab, same_a]),
        vec![same_a, same_ab, same_b]
    );

    let global_z = Symbol::new(GLOBAL_CONTEXT, "z");
    let nested_a = Symbol::new("Global`x`", "a");
    assert_eq!(
        sort_symbols_by_qualified_name(vec![global_z, nested_a]),
        vec![nested_a, global_z]
    );

    let context_underscore = Symbol::new("Ctx`_`", "k");
    let context_a = Symbol::new("Ctx`a`", "k");
    assert_eq!(
        sort_symbols_by_qualified_name(vec![context_a, context_underscore]),
        vec![context_underscore, context_a]
    );

    let name_underscore = Symbol::new("Ctx`", "_");
    let name_backtick = Symbol::new("Ctx`", "`");
    let name_a = Symbol::new("Ctx`", "a");
    assert_eq!(
        sort_symbols_by_qualified_name(vec![name_a, name_backtick, name_underscore]),
        vec![name_underscore, name_backtick, name_a]
    );

    let unicode_ascii = Symbol::new("Ctx`", "a");
    let unicode_latin = Symbol::new("Ctx`", "é");
    let unicode_cjk = Symbol::new("Ctx`", "中");
    assert_eq!(
        sort_symbols_by_qualified_name(vec![unicode_cjk, unicode_latin, unicode_ascii]),
        vec![unicode_ascii, unicode_latin, unicode_cjk]
    );

    let cases = [
        same_a,
        same_ab,
        same_b,
        global_z,
        nested_a,
        context_underscore,
        context_a,
        name_underscore,
        name_backtick,
        unicode_latin,
        unicode_cjk,
    ];
    for lhs in cases {
        for rhs in cases {
            assert_eq!(
                lhs.cmp_qualified_name(&rhs),
                lhs.qualified_name().cmp(&rhs.qualified_name()),
                "qualified-name order differs for {lhs:?} and {rhs:?}"
            );
        }
    }
}

#[test]
fn qualified_symbol_name_ties_ignore_context_elision_and_generated_identity_stably() {
    let system = Symbol::new(SYSTEM_CONTEXT, "same");
    let global = Symbol::new(GLOBAL_CONTEXT, "same");
    assert_eq!(
        system.cmp_qualified_name(&global),
        std::cmp::Ordering::Equal
    );
    assert_eq!(
        sort_symbols_by_qualified_name(vec![system, global]),
        vec![system, global]
    );

    let generated_two = Symbol::with_identity("Generated`", "g", 2);
    let generated_one = Symbol::with_identity("Generated`", "g", 1);
    assert_ne!(generated_two, generated_one);
    assert_eq!(
        generated_two.cmp_qualified_name(&generated_one),
        std::cmp::Ordering::Equal
    );

    let input = vec![
        Expr::Atom(Atom::Symbol(generated_two)),
        Expr::Atom(Atom::Symbol(generated_one)),
    ];
    let mut session = core_session();
    assert_eq!(
        sort_exprs(&mut session, input.clone()),
        Expr::call(Expr::system_symbol("List"), input)
    );
}

#[test]
fn symbol_sort_order_and_mixed_atom_fallback_match_the_pinned_lists() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Sort[{Global`ab, Global`a, Global`b}]"),
        "List[a, ab, b]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Sort[{Global`z, Global`x`a}]"),
        "List[Global`x`a, z]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Sort[{a, \"x\", 2, 1/2}]"),
        "List[1/2, 2, \"x\", a]"
    );

    let rational = Expr::rational(Rational::new(1.into(), 2.into()).unwrap());
    let expected = vec![
        rational.clone(),
        Expr::small_integer(2),
        Expr::string("x"),
        Expr::symbol("a"),
    ];
    assert_eq!(
        sort_exprs(
            &mut session,
            vec![
                Expr::symbol("a"),
                Expr::string("x"),
                Expr::small_integer(2),
                rational,
            ],
        ),
        Expr::call(Expr::system_symbol("List"), expected)
    );
}

#[test]
fn every_default_structural_order_consumer_keeps_qualified_symbol_order() {
    let mut session = core_session();

    for (source, expected) in [
        (
            "SortBy[{{Global`a, 1}, {Ctx`z, 2}}, First]",
            "List[List[Ctx`z, 2], List[a, 1]]",
        ),
        ("Union[{Global`a, Ctx`z}]", "List[Ctx`z, a]"),
        (
            "Intersection[{Global`a, Ctx`z}, {Ctx`z, Global`a}]",
            "List[Ctx`z, a]",
        ),
        ("Complement[{Global`a, Ctx`z}, {}]", "List[Ctx`z, a]"),
        (
            "Union[{Global`a, Ctx`z}, SameTest -> SameQ]",
            "List[Ctx`z, a]",
        ),
        (
            "Intersection[{Global`a, Ctx`z}, {Ctx`z, Global`a}, SameTest -> SameQ]",
            "List[Ctx`z, a]",
        ),
        (
            "Complement[{Global`a, Ctx`z}, {}, SameTest -> SameQ]",
            "List[Ctx`z, a]",
        ),
        ("ReverseSort[{Global`a, Ctx`z}]", "List[a, Ctx`z]"),
        (
            "ReverseSortBy[{{Global`a, 1}, {Ctx`z, 2}}, First]",
            "List[List[a, 1], List[Ctx`z, 2]]",
        ),
        ("MaxBy[{{Global`a, 1}, {Ctx`z, 2}}, First]", "List[a, 1]"),
        (
            "MinBy[{{Global`a, 1}, {Ctx`z, 2}}, First]",
            "List[Ctx`z, 2]",
        ),
        (
            "MaxBy[{{Global`a, 1}, {Ctx`z, 2}}, First, 2]",
            "List[List[a, 1], List[Ctx`z, 2]]",
        ),
        (
            "MinBy[{{Global`a, 1}, {Ctx`z, 2}}, First, 2]",
            "List[List[Ctx`z, 2], List[a, 1]]",
        ),
        (
            "MaximalBy[{{Global`a, 1}, {Ctx`z, 2}}, First]",
            "List[List[a, 1]]",
        ),
        (
            "MinimalBy[{{Global`a, 1}, {Ctx`z, 2}}, First]",
            "List[List[Ctx`z, 2]]",
        ),
        (
            "TakeLargestBy[{{Global`a, 1}, {Ctx`z, 2}}, First, 2]",
            "List[List[a, 1], List[Ctx`z, 2]]",
        ),
        (
            "TakeSmallestBy[{{Global`a, 1}, {Ctx`z, 2}}, First, 2]",
            "List[List[Ctx`z, 2], List[a, 1]]",
        ),
    ] {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn separate_canonical_and_numeric_order_consumers_keep_their_base_outputs() {
    let mut session = core_session();

    for (source, expected) in [
        (
            "Keys[KeySort[Association[Global`a -> 1, Ctx`z -> 2]]]",
            "List[a, Ctx`z]",
        ),
        (
            "Keys[KeySortBy[Association[Global`a -> 1, Ctx`z -> 2], Identity]]",
            "List[a, Ctx`z]",
        ),
        ("Order[Global`a, Ctx`z]", "1"),
        ("OrderedQ[{Global`a, Ctx`z}]", "True"),
        (
            "SetAttributes[brief360Orderless, Orderless]; brief360Orderless[Ctx`z, Global`a]",
            "brief360Orderless[a, Ctx`z]",
        ),
        ("Min[Global`a, Ctx`z]", "Min[a, Ctx`z]"),
        ("Max[Global`a, Ctx`z]", "Max[a, Ctx`z]"),
        ("Ordering[{Ctx`z, Global`a}]", "Ordering[List[Ctx`z, a]]"),
    ] {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn wide_symbol_sort_matches_the_full_list_and_pinned_base_rendering() {
    let mut input = (1..=16_000)
        .rev()
        .map(|index| Symbol::from_name(format!("brief360Wide{index}")))
        .collect::<Vec<_>>();
    let mut expected = input.clone();
    expected.sort_by(|lhs, rhs| lhs.name().as_bytes().cmp(rhs.name().as_bytes()));

    let input = input
        .drain(..)
        .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
        .collect();
    let expected = expected
        .into_iter()
        .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
        .collect();
    let mut session = core_session();
    assert_eq!(
        sort_exprs(&mut session, input),
        Expr::call(Expr::system_symbol("List"), expected)
    );

    // Captured from the unmodified release CLI before replacing the comparator.
    assert_eq!(
        eval_to_string(
            &mut session,
            "Hash[ToString[Sort[Reverse[Table[Symbol[StringJoin[\"brief360Wide\", ToString[i]]], {i, 1, 16000}]]]], \"SHA256\", \"HexString\"]",
        ),
        "\"b6cb5c1ec67c0fba1eeb7ff1f43f9e8bbe2e699631296f458b5984d2a0d50fae\""
    );
}

#[test]
fn set_operation_attributes_match_wk_anchors() {
    let mut session = core_session();

    let union_attributes = eval_to_string(&mut session, "Attributes[Union]");
    let intersection_attributes = eval_to_string(&mut session, "Attributes[Intersection]");

    assert_eq!(
        union_attributes,
        "List[Flat, OneIdentity, Protected, ReadProtected]"
    );
    assert_eq!(intersection_attributes, union_attributes);
}

#[test]
fn sort_and_sortby_cover_current_top_level_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Sort[{b, a, c, a}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Sort[f[2, 10, 1, a, \"x\"]]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 10, \"x\", a]"
    );
    assert_eq!(
        session
            .eval_text("SortBy[{bb, a, ccc}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "List[a, bb, ccc]"
    );
    assert_eq!(
        session
            .eval_text("SortBy[f[bb, a, ccc], StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "f[a, bb, ccc]"
    );
    assert_eq!(
        session
            .eval_text("SortBy[StringLength[ToString[#]] &][{bb, a, ccc}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, bb, ccc]"
    );
    assert_eq!(
        session
            .eval_text("SortBy[{3, 1, 2}, Identity, Greater]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 2, 1]"
    );
}

#[test]
fn custom_sort_comparators_obey_the_merge_bound_without_growing_the_stack() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"n = 4097;
               calls = 0;
               ordering = Function[{a, b}, calls = calls + 1; Less[a, b]];
               sorted = Sort[Reverse[Range[n]], ordering];
               sortCalls = calls;
               calls = 0;
               sortedBy = SortBy[Reverse[Range[n]], Identity, ordering];
               SameQ[{sorted, sortedBy}, {Range[n], Range[n]}] &&
                 sortCalls > 0 && sortCalls <= n * 13 &&
                 calls > 0 && calls <= n * 13"#,
        ),
        "True"
    );
}

#[test]
fn custom_sort_and_sort_by_cover_boundaries_and_stable_tagged_ties() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"input = {{2, a}, {1, b}, {2, c}, {1, d}};
               expected = {{1, b}, {1, d}, {2, a}, {2, c}};
               keyTrace = {};
               sortedBy = SortBy[
                 input,
                 Function[item, keyTrace = Append[keyTrace, item]; First[item]],
                 Less
               ];
               SameQ[
                 {
                   Sort[{}, Less],
                   Sort[{1}, Less],
                   Sort[{1, 2, 3, 4}, Less],
                   Sort[{4, 3, 2, 1}, Less],
                   Sort[{3, 1, 4, 2}, Less],
                   Sort[input, Function[{left, right}, Less[First[left], First[right]]]],
                   SortBy[{}, Identity, Less],
                   SortBy[{1}, Identity, Less],
                   sortedBy,
                   keyTrace
                 },
                 {
                   {},
                   {1},
                   {1, 2, 3, 4},
                   {1, 2, 3, 4},
                   {1, 2, 3, 4},
                   expected,
                   {},
                   {1},
                   expected,
                   input
                 }
               ]"#,
        ),
        "True"
    );
}

#[test]
fn sort_by_supports_stable_multi_key_specs_and_tuple_ordering() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SortBy[{<|\"i\"->3,\"d\"->\"x\"|>, <|\"i\"->1,\"d\"->\"y\"|>, <|\"i\"->2,\"d\"->\"x\"|>}, {Key[\"d\"], Key[\"i\"]}]"
        ),
        "List[Association[Rule[\"i\", 2], Rule[\"d\", \"x\"]], Association[Rule[\"i\", 3], Rule[\"d\", \"x\"]], Association[Rule[\"i\", 1], Rule[\"d\", \"y\"]]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "SortBy[{{2, b}, {1, z}, {2, a}}, {First}]"),
        "List[List[1, z], List[2, b], List[2, a]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "SortBy[{{2, 1}, {1, 9}, {1, 5}}, {Last, First}, (Less[First[#1], First[#2]] &)]"
        ),
        "List[List[2, 1], List[1, 5], List[1, 9]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "SortBy[{{2, 1}, {1, 9}, {1, 5}}, {}]"),
        "List[List[2, 1], List[1, 9], List[1, 5]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "SortBy[{{2, 1}, {1, 9}, {1, 5}}, First]"),
        "List[List[1, 9], List[1, 5], List[2, 1]]"
    );
}

#[test]
fn sort_by_multi_key_evaluation_supports_user_downvalues() {
    let mut session = core_session();
    session.eval_text("sortPrimary[x_] := First[x]").unwrap();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SortBy[{{2, 1}, {1, 9}, {1, 5}}, {sortPrimary, Last}]"
        ),
        "List[List[1, 5], List[1, 9], List[2, 1]]"
    );
}

#[test]
fn union_and_complement_cover_current_same_head_subset() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("Union[]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Union[{b, a, c, a}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Union[f[b, a], f[c, a]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Complement[{a, b, c, b}, {b}, {d}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Complement[f[a, b, c, b], f[b], f[d]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Complement[{b, a, c, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
}

#[test]
fn set_operations_accept_same_test_option() {
    let mut session = core_session();

    let union = session
        .eval_text("Union[{1, -1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)]")
        .unwrap();
    assert_eq!(union.value.to_string(), "List[-1]");
    assert!(union.messages.is_empty());

    let union_option_list = session
        .eval_text("Union[{1, -1}, {SameTest -> (SameQ[Abs[#1], Abs[#2]] &)}]")
        .unwrap();
    assert_eq!(union_option_list.value.to_string(), "List[-1]");
    assert!(union_option_list.messages.is_empty());

    let intersection = session
        .eval_text("Intersection[{1, 2}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)]")
        .unwrap();
    assert_eq!(intersection.value.to_string(), "List[1]");
    assert!(intersection.messages.is_empty());

    let complement = session
        .eval_text("Complement[{1, 2}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)]")
        .unwrap();
    assert_eq!(complement.value.to_string(), "List[2]");
    assert!(complement.messages.is_empty());

    let options = session.eval_text("Options[Union]").unwrap();
    assert_eq!(options.value.to_string(), "List[Rule[SameTest, Automatic]]");
}

#[test]
fn wide_custom_same_test_set_operations_preserve_comparison_counts() {
    let mut session = core_session();

    let outcome = session
        .eval_text(
            "never[x_, y_] := (calls = calls + 1; False); \
             calls = 0; union = Union[Range[200], SameTest -> never]; unionCalls = calls; \
             calls = 0; intersection = Intersection[Range[80], Range[80], SameTest -> never]; intersectionCalls = calls; \
             calls = 0; complement = Complement[Range[80], Range[80], SameTest -> never]; \
             {Length[union], unionCalls, Length[intersection], intersectionCalls, Length[complement], calls}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[200, 19900, 0, 6400, 80, 9560]"
    );
}

#[test]
fn intersection_covers_wk_anchored_same_head_subset() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Intersection[{1, 2, 3}, {2, 3, 4}]"),
        "List[2, 3]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Intersection[{1, 2, 3, 4}, {2, 3, 4}, {3, 4, 5}]"
        ),
        "List[3, 4]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Intersection[{3, 1, 2, 1}]"),
        "List[1, 2, 3]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Intersection[f[1, 2], f[2, 3], f[2, 4]]"),
        "f[2]"
    );
}

#[test]
fn ordering_and_set_helpers_report_current_invalid_boundaries() {
    let mut session = core_session();

    let sort_atom = session.eval_text("Sort[a]").unwrap();
    assert_eq!(sort_atom.value.to_string(), "Sort[a]");
    assert!(
        sort_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Sort" && message.tag == "normal")
    );

    let sort_comparator_atom = session.eval_text("Sort[a, Greater]").unwrap();
    assert_eq!(sort_comparator_atom.value.to_string(), "Sort[a, Greater]");
    assert!(
        sort_comparator_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Sort" && message.tag == "normal")
    );

    let sort_by_atom = session.eval_text("SortBy[a, f]").unwrap();
    assert_eq!(sort_by_atom.value.to_string(), "SortBy[a, f]");
    assert!(
        sort_by_atom
            .messages
            .iter()
            .any(|message| message.symbol == "SortBy" && message.tag == "normal")
    );

    let union_atom = session.eval_text("Union[{a}, b]").unwrap();
    assert_eq!(union_atom.value.to_string(), "Union[List[a], b]");
    assert!(
        union_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Union" && message.tag == "normal")
    );

    let union_heads = session.eval_text("Union[{a}, f[b]]").unwrap();
    assert_eq!(union_heads.value.to_string(), "Union[List[a], f[b]]");
    assert!(
        union_heads
            .messages
            .iter()
            .any(|message| message.symbol == "Union" && message.tag == "heads")
    );

    let complement_heads = session.eval_text("Complement[f[a, b], g[b]]").unwrap();
    assert_eq!(
        complement_heads.value.to_string(),
        "Complement[f[a, b], g[b]]"
    );
    assert!(
        complement_heads
            .messages
            .iter()
            .any(|message| message.symbol == "Complement" && message.tag == "heads")
    );

    let complement_empty = session.eval_text("Complement[]").unwrap();
    assert_eq!(complement_empty.value.to_string(), "Complement[]");
    assert!(
        complement_empty
            .messages
            .iter()
            .any(|message| message.symbol == "Complement" && message.tag == "argm")
    );

    let intersection_empty = session.eval_text("Intersection[]").unwrap();
    assert_eq!(intersection_empty.value.to_string(), "Intersection[]");
    assert!(
        intersection_empty
            .messages
            .iter()
            .any(|message| message.symbol == "Intersection" && message.tag == "argm")
    );

    let intersection_atom = session.eval_text("Intersection[{a}, b]").unwrap();
    assert_eq!(
        intersection_atom.value.to_string(),
        "Intersection[List[a], b]"
    );
    assert!(
        intersection_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Intersection" && message.tag == "normal")
    );

    let intersection_heads = session.eval_text("Intersection[{a}, f[a]]").unwrap();
    assert_eq!(
        intersection_heads.value.to_string(),
        "Intersection[List[a], f[a]]"
    );
    assert!(
        intersection_heads
            .messages
            .iter()
            .any(|message| message.symbol == "Intersection" && message.tag == "heads")
    );
}
