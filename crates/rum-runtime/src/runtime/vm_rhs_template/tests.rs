use super::compiler::{compile_body, slot_values_for_code};
use super::executor::execute_compiled_body;
use super::*;
use std::sync::Arc;

#[test]
fn closure_template_matches_plain_rule_rhs() {
    let x = Symbol::from_name("x");
    let y = Symbol::from_name("y");
    let rhs = Expr::call(
        system_expr("f"),
        vec![
            Expr::Atom(Atom::Symbol(x)),
            Expr::call(system_expr("g"), vec![Expr::Atom(Atom::Symbol(y))]),
        ],
    );
    let mut bindings = SymbolExprMap::default();
    bindings.insert(x, Expr::integer(BigInt::from(1_u8)));
    bindings.insert(y, Expr::integer(BigInt::from(2_u8)));

    assert_compiled_body_matches_tree_walk(&rhs, &bindings);
}

#[test]
fn closure_template_splices_bound_and_existing_sequences() {
    let x = Symbol::from_name("x");
    let rhs = Expr::call(
        system_expr("f"),
        vec![
            Expr::Atom(Atom::Symbol(x)),
            Expr::call(
                system_expr("g"),
                vec![sequence_expr(vec![
                    Expr::integer(BigInt::from(3_u8)),
                    Expr::integer(BigInt::from(4_u8)),
                ])],
            ),
        ],
    );
    let mut bindings = SymbolExprMap::default();
    bindings.insert(
        x,
        sequence_expr(vec![
            Expr::integer(BigInt::from(1_u8)),
            Expr::integer(BigInt::from(2_u8)),
        ]),
    );

    assert_compiled_body_matches_tree_walk(&rhs, &bindings);
}

#[test]
fn closure_template_keeps_nested_scopes_on_fallback_path() {
    let bindings = SymbolExprMap::default();
    let module_rhs = Expr::call(
        system_expr("f"),
        vec![Expr::call(
            system_expr("Module"),
            vec![
                Expr::call(system_expr("List"), vec![Expr::symbol("x")]),
                Expr::symbol("x"),
            ],
        )],
    );
    let with_rhs = Expr::call(
        system_expr("f"),
        vec![Expr::call(
            system_expr("With"),
            vec![
                Expr::call(
                    system_expr("List"),
                    vec![Expr::call(
                        system_expr("Set"),
                        vec![Expr::symbol("x"), Expr::integer(BigInt::from(1_u8))],
                    )],
                ),
                Expr::symbol("x"),
            ],
        )],
    );
    let function_rhs = Expr::call(
        system_expr("Function"),
        vec![
            Expr::call(system_expr("List"), vec![Expr::symbol("x")]),
            Expr::symbol("x"),
        ],
    );

    assert!(compile_body(&module_rhs, &bindings).is_none());
    assert!(compile_body(&with_rhs, &bindings).is_none());
    assert!(compile_body(&function_rhs, &bindings).is_none());
}

#[test]
fn closure_module_if_sameq_matches_tree_walk_when_branch_not_taken() {
    let x = Symbol::from_name("x");
    let rhs = module_if_sameq_rhs(
        Expr::call(
            system_expr("If"),
            vec![system_expr("True"), symbol_expr(&x), Expr::symbol("unused")],
        ),
        Expr::symbol("failed"),
        Expr::call(
            system_expr("If"),
            vec![
                system_expr("True"),
                Expr::symbol("fallback"),
                Expr::symbol("unused"),
            ],
        ),
    );
    let mut bindings = SymbolExprMap::default();
    bindings.insert(x, Expr::symbol("ok"));

    assert_compiled_module_matches_tree_walk(&rhs, &bindings, "ok");
}

#[test]
fn closure_module_if_sameq_matches_tree_walk_when_branch_taken() {
    let x = Symbol::from_name("x");
    let rhs = module_if_sameq_rhs(
        Expr::call(
            system_expr("If"),
            vec![system_expr("True"), symbol_expr(&x), Expr::symbol("unused")],
        ),
        Expr::symbol("failed"),
        Expr::call(
            system_expr("If"),
            vec![
                system_expr("True"),
                Expr::symbol("fallback"),
                Expr::symbol("unused"),
            ],
        ),
    );
    let mut bindings = SymbolExprMap::default();
    bindings.insert(x, Expr::symbol("failed"));

    assert_compiled_module_matches_tree_walk(&rhs, &bindings, "fallback");
}

#[test]
fn closure_module_rejects_held_local_escape() {
    let x = Symbol::from_name("x");
    let res = Symbol::from_name("res");
    let rhs = module_if_sameq_rhs(
        Expr::call(system_expr("Hold"), vec![symbol_expr(&res)]),
        Expr::symbol("failed"),
        Expr::call(Expr::symbol("fallback"), vec![symbol_expr(&x)]),
    );
    let mut bindings = SymbolExprMap::default();
    bindings.insert(x, Expr::symbol("value"));

    assert!(compile_body(&rhs, &bindings).is_none());
}

#[test]
fn closure_repeatc_matches_tree_walk_when_failed_return_taken() {
    let (rhs, mut bindings) = repeatc_rhs_and_bindings("failConv", "start");
    assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "refl[start]");

    bindings.insert(Symbol::from_name("c"), Expr::symbol("stableConv"));
    assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "thm[start, start]");
}

#[test]
fn closure_repeatc_matches_tree_walk_when_stable_return_taken() {
    let (rhs, bindings) = repeatc_rhs_and_bindings("stableConv", "start");
    assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "thm[start, start]");
}

#[test]
fn closure_repeatc_matches_tree_walk_when_recursive_fallthrough_runs() {
    let (rhs, bindings) = repeatc_rhs_and_bindings("stepConv", "start");
    assert_compiled_repeatc_matches_tree_walk(
        &rhs,
        &bindings,
        "transResult[thm[start, mid], refl[mid]]",
    );
}

#[test]
fn closure_repeatc_rejects_held_local_read() {
    let c = Symbol::from_name("c");
    let t = Symbol::from_name("t");
    let mut bindings = SymbolExprMap::default();
    bindings.insert(c, Expr::symbol("stepConv"));
    bindings.insert(t, Expr::symbol("start"));
    let rhs = repeatc_rhs_with_final(
        &c,
        &t,
        Expr::call(system_expr("Hold"), vec![Expr::symbol("res")]),
    );

    assert!(compile_body(&rhs, &bindings).is_none());
}

#[test]
fn closure_compiles_general_control_skeleton() {
    let x = Symbol::from_name("x");
    let rhs = Expr::call(
        system_expr("CompoundExpression"),
        vec![
            Expr::call(
                system_expr("If"),
                vec![
                    Expr::call(
                        system_expr("SameQ"),
                        vec![symbol_expr(&x), Expr::symbol("a")],
                    ),
                    Expr::symbol("Null"),
                ],
            ),
            Expr::call(Expr::symbol("h"), vec![symbol_expr(&x)]),
        ],
    );
    let mut bindings = SymbolExprMap::default();
    bindings.insert(x, Expr::symbol("a"));

    assert_compiled_body_matches_tree_walk(&rhs, &bindings);
}

fn assert_compiled_body_matches_tree_walk(rhs: &Expr, bindings: &SymbolExprMap) {
    let code = Arc::new(compile_body(rhs, bindings).expect("RHS should compile"));
    let slot_values = slot_values_for_code(&code, bindings).expect("slots should prepare");
    let mut actual_session = vm_test_session();
    let actual = finish_compiled_definition_rhs_result(execute_compiled_body(
        &mut actual_session,
        PreparedCompiledRhsTemplate { code, slot_values },
        None,
    ))
    .expect("compiled body should evaluate");

    let substituted =
        substitute_rule_bindings_for_rhs(rhs, bindings, rule_rhs_may_splice_without_bindings(rhs));
    let mut expected_session = vm_test_session();
    let expected = expected_session
        .evaluate_definition_rhs(None, substituted)
        .expect("tree-walk RHS should evaluate");

    assert_eq!(actual, expected);
}

fn assert_compiled_module_matches_tree_walk(
    rhs: &Expr,
    bindings: &SymbolExprMap,
    expected_text: &str,
) {
    assert_compiled_body_matches_tree_walk(rhs, bindings);
    let actual = eval_compiled_to_string(rhs, bindings);
    assert_eq!(actual, expected_text);
}

fn assert_compiled_repeatc_matches_tree_walk(
    rhs: &Expr,
    bindings: &SymbolExprMap,
    expected_text: &str,
) {
    let actual = eval_compiled_repeatc_to_string(rhs, bindings);
    let substituted =
        substitute_rule_bindings_for_rhs(rhs, bindings, rule_rhs_may_splice_without_bindings(rhs));
    let mut expected_session = repeatc_test_session();
    let expected = expected_session
        .evaluate_definition_rhs(None, substituted)
        .expect("tree-walk REPEATC Module should evaluate");

    assert_eq!(actual, expected.to_string());
    assert_eq!(actual, expected_text);
}

fn eval_compiled_to_string(rhs: &Expr, bindings: &SymbolExprMap) -> String {
    let code = Arc::new(compile_body(rhs, bindings).expect("RHS should compile"));
    let slot_values = slot_values_for_code(&code, bindings).expect("slots should prepare");
    let mut session = vm_test_session();
    finish_compiled_definition_rhs_result(execute_compiled_body(
        &mut session,
        PreparedCompiledRhsTemplate { code, slot_values },
        None,
    ))
    .expect("compiled body should evaluate")
    .to_string()
}

fn eval_compiled_repeatc_to_string(rhs: &Expr, bindings: &SymbolExprMap) -> String {
    let code = Arc::new(compile_body(rhs, bindings).expect("REPEATC RHS should compile"));
    let slot_values = slot_values_for_code(&code, bindings).expect("slots should prepare");
    let mut session = repeatc_test_session();
    finish_compiled_definition_rhs_result(execute_compiled_body(
        &mut session,
        PreparedCompiledRhsTemplate { code, slot_values },
        None,
    ))
    .expect("compiled REPEATC body should evaluate")
    .to_string()
}

fn module_if_sameq_rhs(eval_expr: Expr, const_expr: Expr, alt_expr: Expr) -> Expr {
    let res = Symbol::from_name("res");
    Expr::call(
        system_expr("Module"),
        vec![
            Expr::call(system_expr("List"), vec![symbol_expr(&res)]),
            Expr::call(
                system_expr("CompoundExpression"),
                vec![
                    Expr::call(system_expr("Set"), vec![symbol_expr(&res), eval_expr]),
                    Expr::call(
                        system_expr("If"),
                        vec![
                            Expr::call(system_expr("SameQ"), vec![symbol_expr(&res), const_expr]),
                            alt_expr,
                            symbol_expr(&res),
                        ],
                    ),
                ],
            ),
        ],
    )
}

fn repeatc_rhs_and_bindings(conv: &str, term: &str) -> (Expr, SymbolExprMap) {
    let c = Symbol::from_name("c");
    let t = Symbol::from_name("t");
    let mut bindings = SymbolExprMap::default();
    bindings.insert(c, Expr::symbol(conv));
    bindings.insert(t, Expr::symbol(term));
    (repeatc_rhs(&c, &t), bindings)
}

fn repeatc_rhs(c: &Symbol, t: &Symbol) -> Expr {
    let res = Symbol::from_name("res");
    let rhs = Symbol::from_name("rhs");
    let final_expr = Expr::call(
        Expr::symbol("trans"),
        vec![
            symbol_expr(&res),
            Expr::call(
                Expr::call(Expr::symbol("repeatc"), vec![symbol_expr(c)]),
                vec![symbol_expr(&rhs)],
            ),
        ],
    );
    repeatc_rhs_with_final(c, t, final_expr)
}

fn repeatc_rhs_with_final(c: &Symbol, t: &Symbol, final_expr: Expr) -> Expr {
    let res = Symbol::from_name("res");
    let rhs = Symbol::from_name("rhs");
    Expr::call(
        system_expr("Module"),
        vec![
            Expr::call(
                system_expr("List"),
                vec![symbol_expr(&res), symbol_expr(&rhs)],
            ),
            Expr::call(
                system_expr("CompoundExpression"),
                vec![
                    Expr::call(
                        system_expr("Set"),
                        vec![
                            symbol_expr(&res),
                            Expr::call(
                                Expr::symbol("try"),
                                vec![Expr::call(symbol_expr(c), vec![symbol_expr(t)])],
                            ),
                        ],
                    ),
                    Expr::call(
                        system_expr("If"),
                        vec![
                            Expr::call(
                                system_expr("SameQ"),
                                vec![symbol_expr(&res), Expr::symbol("failed")],
                            ),
                            Expr::call(
                                system_expr("Return"),
                                vec![Expr::call(Expr::symbol("refl"), vec![symbol_expr(t)])],
                            ),
                        ],
                    ),
                    Expr::call(
                        system_expr("Set"),
                        vec![
                            symbol_expr(&rhs),
                            Expr::call(Expr::symbol("rhsOf"), vec![symbol_expr(&res)]),
                        ],
                    ),
                    Expr::call(
                        system_expr("If"),
                        vec![
                            Expr::call(
                                system_expr("SameQ"),
                                vec![symbol_expr(&rhs), symbol_expr(t)],
                            ),
                            Expr::call(system_expr("Return"), vec![symbol_expr(&res)]),
                        ],
                    ),
                    final_expr,
                ],
            ),
        ],
    )
}

fn repeatc_test_session() -> Session {
    let mut session = vm_test_session();
    session
        .eval_text(
            "\
            ClearAll[try, failConv, stableConv, stepConv, rhsOf, trans, repeatc];\
            try[x_] := x;\
            failConv[x_] := failed;\
            stableConv[x_] := thm[x, x];\
            stepConv[start] := thm[start, mid];\
            stepConv[mid] := failed;\
            rhsOf[thm[_, r_]] := r;\
            trans[a_, b_] := transResult[a, b];\
            repeatc[c_][t_] := Module[{res, rhs},\
              res = try[c[t]];\
              If[res === failed, Return[refl[t]]];\
              rhs = rhsOf[res];\
              If[SameQ[rhs, t], Return[res]];\
              trans[res, repeatc[c][rhs]]\
            ]",
        )
        .expect("repeatc test definitions should install");
    session
}

fn vm_test_session() -> Session {
    let mut session = Session::new();
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "CompoundExpression",
        "Test-local control builtin for VM template probes.",
        HoldAttribute::HoldAll,
        test_eval_compound_expression,
    ));
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "If",
        "Test-local control builtin for VM template probes.",
        HoldAttribute::HoldRest,
        test_eval_if,
    ));
    session.register_builtin(BuiltinSpec::ordinary(
        "Return",
        "Test-local control builtin for VM template probes.",
        test_eval_return,
    ));
    session.register_builtin(BuiltinSpec::ordinary(
        "SameQ",
        "Tests whether all arguments are exactly identical after evaluation.",
        test_eval_same_q,
    ));
    session
}

fn test_eval_compound_expression(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    session.eval_control_data_builtin(ControlDataBuiltin::CompoundExpression, head, args)
}

fn test_eval_if(session: &mut Session, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
    session.eval_control_data_builtin(ControlDataBuiltin::If, head, args)
}

fn test_eval_return(session: &mut Session, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
    session.eval_control_data_builtin(ControlDataBuiltin::Return, head, args)
}

fn test_eval_same_q(
    _session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    Ok(system_expr(
        if args.windows(2).all(|pair| pair[0] == pair[1]) {
            heads::structure::TRUE
        } else {
            heads::structure::FALSE
        },
    ))
}

fn symbol_expr(symbol: &Symbol) -> Expr {
    Expr::Atom(Atom::Symbol(*symbol))
}
