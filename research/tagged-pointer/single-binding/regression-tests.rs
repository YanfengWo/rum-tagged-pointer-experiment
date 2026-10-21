use super::*;

fn sb_symbol(symbol: Symbol) -> Expr {
    Expr::Atom(Atom::Symbol(symbol))
}

fn sb_call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::call(system_expr(name), args)
}

fn sb_arg(expr: &Expr, index: usize) -> &Expr {
    let Expr::Call { args, .. } = expr else {
        panic!("expected a call, got {expr:?}");
    };
    &args[index]
}

fn sb_as_symbol(expr: &Expr) -> Symbol {
    let Expr::Atom(Atom::Symbol(symbol)) = expr else {
        panic!("expected a symbol, got {expr:?}");
    };
    *symbol
}

// Exercise the actual scanner in every implementation, before later evaluation
// can hide substitution hygiene or Sequence reconstruction differences.
fn sb_rewrite(name: Symbol, value: Expr, rhs: Expr) -> Expr {
    let mut session = session_with_test_control_builtins();
    let lhs = sb_call("Pattern", vec![sb_symbol(name), sb_call("Blank", vec![])]);
    let rule = session
        .compile_rewrite_rule_for_matching(&sb_call("RuleDelayed", vec![lhs, rhs]))
        .unwrap();
    let plan = Session::shared_rewrite_rule_plan(vec![rule]);
    session
        .apply_rewrite_rule_plan_once(value, plan, RuleApplicationSource::Replace)
        .unwrap()
        .expect("an untyped named blank must match")
}

fn sb_assert_shared(expected: &Expr, actual: &Expr) {
    let (
        Expr::Call { head: eh, args: ea },
        Expr::Call { head: ah, args: aa },
    ) = (expected, actual)
    else {
        panic!("expected two shared calls");
    };
    assert!(SharedExpr::ptr_eq(eh, ah));
    assert!(SharedArgs::ptr_eq(ea, aa));
}

#[test]
fn single_binding_compares_complete_symbol_identity() {
    let plain = Symbol::from_name("same");
    let bound = Symbol::with_identity(plain.context(), plain.name(), 41);
    let other = Symbol::with_identity(plain.context(), plain.name(), 42);
    let value = Expr::small_integer(7);
    let rhs = sb_call(
        "Hold",
        vec![sb_symbol(plain), sb_symbol(bound), sb_symbol(other)],
    );
    let result = sb_rewrite(bound, value.clone(), rhs);
    assert_eq!(sb_as_symbol(sb_arg(&result, 0)), plain);
    assert_eq!(sb_arg(&result, 1), &value);
    assert_eq!(sb_as_symbol(sb_arg(&result, 2)), other);
}

#[test]
fn single_binding_keeps_scoped_hygiene_and_inserted_values_outside_renaming() {
    let local = Symbol::from_name("local");
    let name = Symbol::from_name("body");
    let value = sb_call("Hold", vec![sb_symbol(local)]);
    for kind in ["Module", "With", "Function"] {
        let locals = match kind {
            "Module" => sb_call("List", vec![sb_symbol(local)]),
            "With" => sb_call(
                "List",
                vec![sb_call("Set", vec![sb_symbol(local), Expr::small_integer(1)])],
            ),
            "Function" => sb_symbol(local),
            _ => unreachable!(),
        };
        let rhs = sb_call(
            kind,
            vec![locals, sb_call("Hold", vec![sb_symbol(name), sb_symbol(local)])],
        );
        let result = sb_rewrite(name, value.clone(), rhs);
        let fresh = match kind {
            "Module" => sb_as_symbol(sb_arg(sb_arg(&result, 0), 0)),
            "With" => sb_as_symbol(sb_arg(sb_arg(sb_arg(&result, 0), 0), 0)),
            "Function" => sb_as_symbol(sb_arg(&result, 0)),
            _ => unreachable!(),
        };
        assert_eq!(fresh.context(), local.context());
        assert_eq!(fresh.name(), local.name());
        assert!(fresh.identity().is_some());
        assert_ne!(fresh, local);
        let body = sb_arg(&result, 1);
        assert_eq!(sb_as_symbol(sb_arg(body, 1)), fresh);
        let inserted = sb_arg(body, 0);
        sb_assert_shared(&value, inserted);
        assert_eq!(sb_as_symbol(sb_arg(inserted, 0)), local);
    }
}

#[test]
fn single_binding_preserves_binding_priority_over_scoped_local_renaming() {
    let name = Symbol::from_name("boundLocal");
    let value = sb_symbol(Symbol::from_name("replacement"));
    for kind in ["Module", "With", "Function"] {
        let locals = match kind {
            "Module" => sb_call("List", vec![sb_symbol(name)]),
            "With" => sb_call(
                "List",
                vec![sb_call("Set", vec![sb_symbol(name), Expr::small_integer(1)])],
            ),
            "Function" => sb_symbol(name),
            _ => unreachable!(),
        };
        let rhs = sb_call(kind, vec![locals, sb_call("Hold", vec![sb_symbol(name)])]);
        let result = sb_rewrite(name, value.clone(), rhs);
        let local_result = match kind {
            "Module" => sb_arg(sb_arg(&result, 0), 0),
            "With" => sb_arg(sb_arg(sb_arg(&result, 0), 0), 0),
            "Function" => sb_arg(&result, 0),
            _ => unreachable!(),
        };
        assert_eq!(local_result, &value);
        assert_eq!(sb_arg(sb_arg(&result, 1), 0), &value);
    }
}

#[test]
fn single_binding_splices_direct_sequence_values_but_keeps_literal_sequence_heads() {
    let name = Symbol::from_name("sequenceValue");
    let value = sb_call("Sequence", vec![Expr::small_integer(1), Expr::small_integer(2)]);
    let original = value.clone();
    let rhs = sb_call(
        "Hold",
        vec![sb_symbol(name), sb_call("Sequence", vec![sb_symbol(name)])],
    );
    let result = sb_rewrite(name, value.clone(), rhs);
    assert_eq!(result.to_string(), "Hold[1, 2, Sequence[1, 2]]");
    assert_eq!(value, original);
    sb_assert_shared(&value, &original);
}

#[test]
fn single_binding_retains_shared_children_and_copy_on_write_aliases() {
    let name = Symbol::from_name("value");
    let value = sb_call("g", vec![sb_call("child", vec![Expr::small_integer(1)])]);
    let sibling = sb_call("sibling", vec![Expr::small_integer(2)]);
    let rhs = sb_call(
        "f",
        vec![sb_symbol(name), sb_symbol(name), sibling.clone()],
    );
    let mut result = sb_rewrite(name, value.clone(), rhs);
    sb_assert_shared(&value, sb_arg(&result, 0));
    sb_assert_shared(&value, sb_arg(&result, 1));
    sb_assert_shared(&sibling, sb_arg(&result, 2));
    let alias = result.clone();
    let Expr::Call { args, .. } = &mut result else { unreachable!() };
    let Expr::Call { args: inner, .. } = &mut args.make_mut()[0] else { unreachable!() };
    inner.make_mut()[0] = Expr::small_integer(99);
    assert_eq!(value.to_string(), "g[child[1]]");
    assert_eq!(sb_arg(&alias, 0), &value);
    assert_eq!(sb_arg(&result, 1), &value);
    assert_eq!(sb_arg(&result, 0).to_string(), "g[99]");
}

#[test]
fn single_binding_guard_fallback_resumes_without_replaying_effects() {
    let source = "seen = 0; result = Replace[{1, 2}, { \
        x_Integer :> bad /; (seen = seen + 1; \
            MatchQ[InputString[\"guard: \"], \"yes\"]), \
        x_Integer :> Hold[x]} , {1}]; {result, seen}";
    let mut queued = session_with_test_control_builtins();
    queued.queue_input_response("no");
    queued.queue_input_response("no");
    let expected = queued.eval_request(source).unwrap().value;
    assert_eq!(expected.to_string(), "List[List[Hold[1], Hold[2]], 2]");
    let mut resumed = session_with_test_control_builtins();
    let first = resumed.eval_request(source).unwrap_err();
    assert!(matches!(first.error, SessionRequestError::InputRequired { .. }));
    let second = resumed.resume_eval_request_with_input("no").unwrap_err();
    assert!(matches!(second.error, SessionRequestError::InputRequired { .. }));
    let actual = resumed.resume_eval_request_with_input("no").unwrap();
    assert_eq!(actual.value, expected);
    assert!(actual.messages.is_empty());
}
