use super::*;
use rum_core::{BigInt, Rational, SharedArgs, Symbol};

fn equivalent(expr: Expr) {
    let compact = CompactExpr::new(expr.clone());
    let safe = SafeExpr::new(expr.clone());
    assert_eq!(compact.to_expr(), expr);
    assert_eq!(safe.to_expr(), expr);
    assert_eq!(compact.clone().into_expr(), expr);
    assert_eq!(safe.clone().into_expr(), expr);
    assert_eq!(compact.into_expr(), expr);
    assert_eq!(safe.into_expr(), expr);
}

#[test]
fn layout_and_thread_bounds() {
    fn thread_safe<T: Send + Sync>() {}
    thread_safe::<Expr>();
    thread_safe::<CompactExpr>();
    assert_eq!(std::mem::size_of::<CompactExpr>(), 8);
    assert_eq!(std::mem::size_of::<Option<CompactExpr>>(), 8);
    assert_eq!(std::mem::align_of::<Payload>(), 8);
    assert_eq!(std::mem::size_of::<SafeExpr>(), 16);
    assert_eq!(std::mem::size_of::<Expr>(), 32);
}

#[test]
fn integer_boundaries_and_full_i64() {
    for value in [
        i64::MIN,
        MIN_INLINE - 1,
        MIN_INLINE,
        -1,
        0,
        1,
        MAX_INLINE,
        MAX_INLINE + 1,
        i64::MAX,
    ] {
        let compact = CompactExpr::new(Expr::small_integer(value));
        assert_eq!(
            compact.is_inline(),
            (MIN_INLINE..=MAX_INLINE).contains(&value)
        );
        if compact.is_inline() {
            assert_eq!(compact.integer(), value);
        }
        equivalent(Expr::small_integer(value));
    }
}

#[test]
fn heap_tag_zero_and_noncanonical_big_variant_preserved() {
    let value = CompactExpr::new(Expr::string("unmodified heap address"));
    assert_eq!(value.tagged.addr().get() & MASK, 0);
    assert_eq!(value.tagged.as_ptr().cast_const(), value.heap_ptr());
    // Directly constructed Big(7) must stay Big, rather than being normalized.
    let big = Expr::Atom(Atom::Integer(IntValue::Big(Box::new(BigInt::from(7)))));
    let packed = CompactExpr::new(big);
    assert!(!packed.is_inline());
    assert!(matches!(
        packed.into_expr(),
        Expr::Atom(Atom::Integer(IntValue::Big(_)))
    ));
}

#[test]
fn deterministic_integer_round_trips() {
    let mut state = 0x1234_5678_9abc_def0_u64;
    for _ in 0..256 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        equivalent(Expr::small_integer(state as i64));
        equivalent(Expr::small_integer((state as i64) >> 3));
    }
}

#[test]
fn original_variants_and_generated_identity() {
    equivalent(Expr::integer(BigInt::from(1_u64) << 100));
    equivalent(Expr::rational(
        Rational::new(BigInt::from(-7), BigInt::from(3)).unwrap(),
    ));
    equivalent(Expr::string("owned unicode: λ 中文"));
    equivalent(Expr::Atom(Atom::Symbol(Symbol::with_identity(
        "Global`", "x", 17,
    ))));
    equivalent(Expr::call(
        Expr::system_symbol("List"),
        vec![
            Expr::small_integer(3),
            Expr::string("child"),
            Expr::call(Expr::system_symbol("f"), vec![]),
        ],
    ));
}

// Only tests temporarily reconstruct an Arc to obtain a Weak observer. This
// ManuallyDrop Arc borrows the owner's count; it must not decrement it.
fn observe(value: &CompactExpr) -> std::sync::Weak<Payload> {
    assert!(!value.is_inline());
    let arc = ManuallyDrop::new(unsafe { Arc::from_raw(value.heap_ptr()) });
    Arc::downgrade(&arc)
}

#[test]
fn counts_and_final_release() {
    let original = CompactExpr::new(Expr::string("unique payload"));
    let weak = observe(&original);
    assert_eq!(weak.strong_count(), 1);
    let clones: Vec<_> = (0..16).map(|_| original.clone()).collect();
    assert_eq!(weak.strong_count(), 17);
    drop(original);
    for clone in &clones {
        assert_eq!(clone.to_expr(), Expr::string("unique payload"));
    }
    assert_eq!(weak.strong_count(), 16);
    drop(clones);
    assert!(weak.upgrade().is_none());
}

#[test]
fn consuming_unique_and_shared_payloads() {
    let unique = CompactExpr::new(Expr::string("unique"));
    let weak = observe(&unique);
    assert_eq!(unique.into_expr(), Expr::string("unique"));
    assert!(weak.upgrade().is_none());
    let original = CompactExpr::new(Expr::string("shared"));
    let weak = observe(&original);
    assert_eq!(original.clone().into_expr(), Expr::string("shared"));
    assert_eq!(weak.strong_count(), 1);
    assert_eq!(original.into_expr(), Expr::string("shared"));
    assert!(weak.upgrade().is_none());
}

#[test]
fn nested_args_keep_weak_and_copy_on_write_behavior() {
    let args = SharedArgs::new(vec![Expr::small_integer(7)]);
    let weak = args.downgrade();
    let expr = Expr::call_with_shared_args(Expr::system_symbol("List"), args.clone());
    let packed = CompactExpr::new(expr);
    let Expr::Call {
        args: mut edited, ..
    } = packed.clone().into_expr()
    else {
        panic!()
    };
    assert!(SharedArgs::ptr_eq(&args, &edited));
    edited.make_mut().push(Expr::small_integer(8));
    assert!(!SharedArgs::ptr_eq(&args, &edited));
    assert_eq!(args.len(), 1);
    assert_eq!(edited.len(), 2);
    assert!(weak.is_alive());
    drop(args);
    assert!(weak.is_alive());
    drop(packed);
    assert!(!weak.is_alive());
}

#[test]
fn threads_borrow_clone_and_release() {
    let value = CompactExpr::new(Expr::string("thread shared"));
    let weak = observe(&value);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let owner = value.clone();
            let borrowed = &value;
            scope.spawn(move || {
                for _ in 0..8 {
                    assert_eq!(owner.clone().into_expr(), borrowed.to_expr());
                }
            });
        }
    });
    assert_eq!(weak.strong_count(), 1);
    drop(value);
    assert!(weak.upgrade().is_none());
}

#[test]
fn forgotten_inline_value_is_safe() {
    // Heap forget is also safe but intentionally leaks; Miri leak checking stays on.
    std::mem::forget(CompactExpr::new(Expr::small_integer(-17)));
}
