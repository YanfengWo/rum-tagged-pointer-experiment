use super::*;

fn session_with_compound_expression() -> Session {
    crate::runtime::test_builtins::session_with_test_compound_expression_builtin()
}

fn call_parts(expr: &Expr) -> (&SharedExpr, &SharedArgs) {
    let Expr::Call { head, args } = expr else {
        panic!("expected a call expression");
    };
    (head, args)
}

fn cache_stable_call(session: &mut Session, expr: &Expr) -> StabilityBloom {
    let (stable, bloom) = session.cached_expr_stability_with_bloom(expr);
    assert!(stable);
    let (head, args) = call_parts(expr);
    assert!(
        session
            .call_stability_cache
            .contains_key(&CallStabilityCacheKey::new(head, args))
    );
    bloom
}

#[test]
fn append_to_args_become_exclusive_after_definition_take_and_cache_entry_removal() {
    let mut session = Session::default();
    session.eval_text("lst = {}").unwrap();
    session.eval_text("AppendTo[lst, 1]").unwrap();

    let target = Symbol::new(GLOBAL_CONTEXT, "lst");
    let mut current = session
        .definitions
        .get_mut(&target)
        .and_then(|definition| definition.own_value.take())
        .expect("AppendTo must leave its updated List in the definition");
    let Expr::Call { head, args } = &mut current else {
        panic!("AppendTo must store a call expression");
    };
    let key = CallStabilityCacheKey::new(head, args);

    assert!(
        args.get_mut().is_none(),
        "the exact call-stability entry must initially retain the SharedArgs"
    );
    let proof = session
        .take_call_stability_proof(head, args)
        .expect("the real AppendTo result must have an exact call-stability entry");
    assert!(!session.call_stability_cache.contains_key(&key));
    assert_ne!(proof.bloom, StabilityBloom::default());

    assert!(
        args.get_mut().is_some(),
        "cache-entry removal plus definition take must leave exactly one strong SharedArgs owner"
    );
}

#[test]
fn shared_args_make_mut_profiles_only_cow_and_preserves_alias_values() {
    let profiling_enabled = rum_core::repr_profile_enabled();

    let mut unique = SharedArgs::new(vec![Expr::small_integer(1)]);
    let unique_key = unique.ptr_key();
    let before_unique_mutation = rum_core::repr_profile_snapshot();
    assert!(unique.get_mut().is_some());
    unique.make_mut().push(Expr::small_integer(2));
    let after_unique_mutation = rum_core::repr_profile_snapshot();
    assert_eq!(unique.ptr_key(), unique_key);
    assert_eq!(
        after_unique_mutation.shared_args_alloc,
        before_unique_mutation.shared_args_alloc
    );

    let mut aliased = SharedArgs::new(vec![Expr::small_integer(3)]);
    let alias = aliased.clone();
    let alias_key = alias.ptr_key();
    let before_cow = rum_core::repr_profile_snapshot();
    assert!(aliased.get_mut().is_none());
    aliased.make_mut().push(Expr::small_integer(4));
    let after_cow = rum_core::repr_profile_snapshot();

    assert_ne!(aliased.ptr_key(), alias_key);
    assert_eq!(alias.as_ref(), &[Expr::small_integer(3)]);
    assert_eq!(
        aliased.as_ref(),
        &[Expr::small_integer(3), Expr::small_integer(4)]
    );
    assert_eq!(
        after_cow.shared_args_alloc - before_cow.shared_args_alloc,
        u64::from(profiling_enabled)
    );
}

#[test]
fn removing_valid_call_stability_entry_preserves_evaluation_outcome() {
    let mut session = session_with_compound_expression();
    session.eval_text("stableExpr = h[1, 2]").unwrap();
    session.clear_evaluation_caches();
    session.eval_text("stableExpr").unwrap();
    let with_entry = session.eval_text("stableExpr").unwrap();
    let (head, args) = call_parts(&with_entry.value);

    assert!(session.take_call_stability_proof(head, args).is_some());
    let without_entry = session.eval_text("stableExpr").unwrap();

    assert_eq!(without_entry, with_entry);
}

#[test]
fn generation_stale_pointer_matched_entry_is_evicted_without_proof() {
    let mut session = Session::default();
    let expr = Expr::call(system_expr("List"), vec![Expr::small_integer(1)]);
    cache_stable_call(&mut session, &expr);
    let (head, args) = call_parts(&expr);
    let key = CallStabilityCacheKey::new(head, args);
    session.call_stability_generation += 1;

    assert!(session.take_call_stability_proof(head, args).is_none());
    assert!(!session.call_stability_cache.contains_key(&key));
}

#[test]
fn dirty_pointer_matched_entry_is_evicted_without_proof() {
    let mut session = Session::default();
    let observed = Symbol::new(GLOBAL_CONTEXT, "observed");
    let expr = Expr::call(
        Expr::Atom(Atom::Symbol(observed)),
        vec![Expr::small_integer(1)],
    );
    cache_stable_call(&mut session, &expr);
    let (head, args) = call_parts(&expr);
    let key = CallStabilityCacheKey::new(head, args);
    session.bump_evaluation_epoch_for_symbol(&observed);

    assert!(session.take_call_stability_proof(head, args).is_none());
    assert!(!session.call_stability_cache.contains_key(&key));
}

#[test]
fn pointer_mismatched_entry_is_left_untouched() {
    let mut session = Session::default();
    let requested = Expr::call(system_expr("List"), vec![Expr::small_integer(1)]);
    let mismatched = Expr::call(system_expr("List"), vec![Expr::small_integer(2)]);
    let (requested_head, requested_args) = call_parts(&requested);
    let (mismatched_head, mismatched_args) = call_parts(&mismatched);
    let key = CallStabilityCacheKey::new(requested_head, requested_args);
    session.call_stability_cache.insert(
        key,
        CallStabilityCacheEntry {
            generation: session.call_stability_generation,
            head: mismatched_head.clone(),
            args: mismatched_args.clone(),
            stable: true,
            bloom: StabilityBloom::default(),
            dirty_snapshot: session.stability_dirty_bloom,
        },
    );

    assert!(
        session
            .take_call_stability_proof(requested_head, requested_args)
            .is_none()
    );
    assert!(session.take_call_stability_proof_for_key(key).is_none());
    let remaining = session
        .call_stability_cache
        .get(&key)
        .expect("a pointer mismatch must leave the cache entry untouched");
    assert!(SharedExpr::ptr_eq(&remaining.head, mismatched_head));
    assert!(SharedArgs::ptr_eq(&remaining.args, mismatched_args));
}

#[test]
fn proof_reinsertion_uses_current_generation_dirty_snapshot_and_bloom() {
    let mut session = Session::default();
    let expr = Expr::call(system_expr("List"), vec![Expr::small_integer(1)]);
    cache_stable_call(&mut session, &expr);
    let (head, args) = call_parts(&expr);
    let mut proof = session
        .take_call_stability_proof(head, args)
        .expect("the stable List must yield a proof");

    let newly_dirty = Symbol::new(GLOBAL_CONTEXT, "newlyDirty");
    session.bump_evaluation_epoch_for_symbol(&newly_dirty);
    session.call_stability_generation += 1;
    proof.bloom.union_assign(StabilityBloom::regular(
        super::stability::symbol_stability_bloom_bit(&newly_dirty),
    ));
    let expected_bloom = proof.bloom;
    let current_generation = session.call_stability_generation;
    let current_dirty_snapshot = session.stability_dirty_bloom;

    session.reinsert_call_stability_proof(head, args, proof);

    let entry = session
        .call_stability_cache
        .get(&CallStabilityCacheKey::new(head, args))
        .expect("reinsertion must seed the exact entry");
    assert_eq!(entry.generation, current_generation);
    assert_eq!(entry.dirty_snapshot, current_dirty_snapshot);
    assert_eq!(entry.bloom, expected_bloom);
    assert!(entry.stable);
    let (stable, hit_bloom) = session.cached_expr_stability_with_bloom(&expr);
    assert!(stable);
    assert_eq!(hit_bloom, expected_bloom);
}

#[test]
fn path_eviction_removes_only_the_named_exact_call_nodes() {
    let mut session = Session::default();
    let leaf = Expr::call(Expr::symbol("leafHead"), vec![Expr::small_integer(1)]);
    let middle = Expr::call(Expr::symbol("middleHead"), vec![leaf]);
    let nested = Expr::call(Expr::symbol("outerHead"), vec![middle]);
    let unrelated = Expr::call(Expr::symbol("unrelatedHead"), vec![Expr::small_integer(2)]);
    cache_stable_call(&mut session, &nested);
    cache_stable_call(&mut session, &unrelated);

    let (outer_head, outer_args) = call_parts(&nested);
    let (middle_head, middle_args) = call_parts(&outer_args[0]);
    let (leaf_head, leaf_args) = call_parts(&middle_args[0]);
    let (unrelated_head, unrelated_args) = call_parts(&unrelated);
    let outer_key = CallStabilityCacheKey::new(outer_head, outer_args);
    let middle_key = CallStabilityCacheKey::new(middle_head, middle_args);
    let leaf_key = CallStabilityCacheKey::new(leaf_head, leaf_args);
    let unrelated_key = CallStabilityCacheKey::new(unrelated_head, unrelated_args);

    session.evict_call_stability_path(&[(outer_head, outer_args), (middle_head, middle_args)]);

    assert!(!session.call_stability_cache.contains_key(&outer_key));
    assert!(!session.call_stability_cache.contains_key(&middle_key));
    assert!(session.call_stability_cache.contains_key(&leaf_key));
    assert!(session.call_stability_cache.contains_key(&unrelated_key));
}

#[test]
fn append_to_preserves_an_aliased_list_value() {
    let mut session = session_with_compound_expression();
    let outcome = session
        .eval_text("x = {1, 2}; y = x; AppendTo[y, 3]; {x, y}")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[List[1, 2], List[1, 2, 3]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn association_lvalue_update_preserves_an_aliased_value() {
    let mut session = session_with_compound_expression();
    let outcome = session
        .eval_text("a = <|1 -> 2|>; b = a; b[3] = 4; {a, b}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[Association[Rule[1, 2]], Association[Rule[1, 2], Rule[3, 4]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn evaluation_cache_is_invalidated_by_new_down_value() {
    let mut session = Session::default();

    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "f[1]");
    assert!(!session.symbol_call_effect_cache.is_empty());

    session.eval_text("f[x_] := x + 1").unwrap();

    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "2");
}

#[test]
fn evaluation_cache_is_invalidated_by_attribute_change() {
    let mut session = Session::default();

    assert_eq!(
        session.eval_text("f[{a, b}]").unwrap().value.to_string(),
        "f[List[a, b]]"
    );
    assert!(!session.symbol_call_effect_cache.is_empty());

    session.eval_text("SetAttributes[f, Listable]").unwrap();

    assert_eq!(
        session.eval_text("f[{a, b}]").unwrap().value.to_string(),
        "List[f[a], f[b]]"
    );
}

#[test]
fn repeated_writes_to_one_symbol_keep_invalidation_sound() {
    let mut session = session_with_compound_expression();

    session.eval_text("g[] := (s = s + 1; s)").unwrap();
    session.eval_text("s = 0").unwrap();
    assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "1");
    assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "2");
    assert_eq!(session.eval_text("s").unwrap().value.to_string(), "2");
}

#[test]
fn stability_cache_survives_unrelated_symbol_writes() {
    let mut session = Session::default();

    // Populate the call-stability cache with a stable expression.
    assert_eq!(
        session.eval_text("h[1, 2]").unwrap().value.to_string(),
        "h[1, 2]"
    );

    // Writing an unrelated symbol must not change the stable verdict...
    session.eval_text("unrelated = 5").unwrap();
    assert_eq!(
        session.eval_text("h[1, 2]").unwrap().value.to_string(),
        "h[1, 2]"
    );

    // ...while a write to the observed head must flip it.
    session.eval_text("h[x_, y_] := x + y").unwrap();
    assert_eq!(session.eval_text("h[1, 2]").unwrap().value.to_string(), "3");
}

#[test]
fn repeated_unrelated_writes_do_not_bump_call_stability_generation() {
    let mut session = Session::default();

    session.eval_text("stableExpr = h[1, 2, 3]").unwrap();
    session.clear_evaluation_caches();
    assert_eq!(
        session.eval_text("stableExpr").unwrap().value.to_string(),
        "h[1, 2, 3]"
    );
    assert!(!session.call_stability_cache.is_empty());

    let generation = session.call_stability_generation;
    session.eval_text("unrelatedWrite = 1").unwrap();
    session.eval_text("unrelatedWrite = 2").unwrap();
    session.eval_text("unrelatedWrite = 3").unwrap();

    assert_eq!(session.call_stability_generation, generation);
    assert_eq!(
        session.eval_text("stableExpr").unwrap().value.to_string(),
        "h[1, 2, 3]"
    );
}
