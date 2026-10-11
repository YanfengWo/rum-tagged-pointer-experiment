//! Complete logical traversal, including call heads, with no owner-count edits.
//! Checksums guard measured work; full semantic validation remains separate.
use rum_core::{Atom, Expr, IntValue, Symbol};
use rum_layout_design::{AtomRef, IntRef, Tree, View};
use rum_recursive_expr::{Compact as OldCompact, View as OldView};

fn integer(n: i64) -> u64 {
    (n as u64).wrapping_mul(31).wrapping_add(7)
}

fn symbol(s: Symbol) -> u64 {
    (s.interned_id() as u64)
        .wrapping_mul(17)
        .wrapping_add(s.identity().unwrap_or(0))
        .wrapping_add(11)
}

fn text(s: &str) -> u64 {
    s.bytes().fold(13_u64, |sum, b| sum.wrapping_add(b as u64))
}

#[inline(always)]
fn atom(value: AtomRef<'_>) -> u64 {
    match value {
        AtomRef::Integer(IntRef::Small(n)) => integer(n),
        // Keep the variant visible in the checksum, including a manual Big(7).
        // These paths allocate decimal text but are absent from the pinned timed
        // corpus. All representations use exactly the same fallback rule.
        AtomRef::Integer(IntRef::Big(n)) => text(&n.to_string()).wrapping_add(23),
        AtomRef::Rational(n) => text(&n.to_string()).wrapping_add(29),
        AtomRef::Symbol(s) => symbol(s),
        AtomRef::String(s) => text(s),
    }
}

pub fn original(root: &Expr) -> u64 {
    let mut work = vec![root];
    let mut sum = 0_u64;
    while let Some(node) = work.pop() {
        let value = match node {
            Expr::Atom(Atom::Integer(IntValue::Small(n))) => integer(*n),
            Expr::Atom(Atom::Integer(IntValue::Big(n))) => text(&n.to_string()).wrapping_add(23),
            Expr::Atom(Atom::Rational(n)) => text(&n.to_string()).wrapping_add(29),
            Expr::Atom(Atom::Symbol(s)) => symbol(*s),
            Expr::Atom(Atom::String(s)) => text(s),
            Expr::Call { head, args } => {
                work.push(head);
                work.extend(args.iter());
                19
            }
        };
        sum = sum.wrapping_add(value);
    }
    sum
}

pub fn tree<N: Tree>(root: &N) -> u64 {
    let mut work = vec![root];
    let mut sum = 0_u64;
    while let Some(node) = work.pop() {
        let value = match node.view() {
            View::Atom(value) => atom(value),
            View::Call { head, args } => {
                // The returned references borrow node for its original lifetime,
                // rather than borrowing the temporary handle-ref wrappers.
                work.push(head.node());
                work.extend(args.slice().iter());
                19
            }
        };
        sum = sum.wrapping_add(value);
    }
    sum
}

pub fn old_tree(root: &OldCompact) -> u64 {
    let mut work = vec![root];
    let mut sum = 0_u64;
    while let Some(node) = work.pop() {
        let value = match node.view() {
            OldView::Atom(value) => atom(value),
            OldView::Call { head, args } => {
                work.push(&**head);
                work.extend(args.iter());
                19
            }
        };
        sum = sum.wrapping_add(value);
    }
    sum
}

pub fn cow_original(root: &Expr) -> Expr {
    let Expr::Call { head, args } = root else {
        panic!("COW measurement requires a call root")
    };
    let mut args = args.clone();
    args.make_mut().push(Expr::small_integer(777));
    Expr::call_shared(head.clone(), args)
}

pub fn cow_tree<N: Tree>(root: &N) -> N {
    let View::Call { head, args } = root.view() else {
        panic!("COW measurement requires a call root")
    };
    let mut args = args.clone_handle();
    args.make_mut()
        .push(N::atom(Atom::Integer(IntValue::Small(777))));
    N::call(head.clone_handle(), args)
}

pub fn cow_old(root: &OldCompact) -> OldCompact {
    let OldView::Call { head, args } = root.view() else {
        panic!("COW measurement requires a call root")
    };
    let mut args = args.clone();
    args.make_mut().push(OldCompact::small(777));
    OldCompact::call(head.clone(), args)
}

pub fn appended_checksum(sum: u64) -> u64 {
    sum.wrapping_add(integer(777))
}
