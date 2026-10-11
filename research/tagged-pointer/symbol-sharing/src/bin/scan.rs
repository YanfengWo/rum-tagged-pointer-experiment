use rum_core::{Atom, Expr, IntValue, Symbol};
use rum_recursive_expr::{AtomRef, IntRef, Tree, View};

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
pub fn original(root: &Expr) -> u64 {
    let mut work = vec![root];
    let mut sum = 0_u64;
    while let Some(node) = work.pop() {
        let value = match node {
            Expr::Atom(Atom::Integer(IntValue::Small(n))) => integer(*n),
            Expr::Atom(Atom::Symbol(s)) => symbol(*s),
            Expr::Atom(Atom::String(s)) => text(s),
            Expr::Atom(_) => panic!("this pinned corpus contains no Big/Rational atoms"),
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
            View::Atom(AtomRef::Integer(IntRef::Small(n))) => integer(n),
            View::Atom(AtomRef::Symbol(s)) => symbol(s),
            View::Atom(AtomRef::String(s)) => text(s),
            View::Atom(_) => panic!("this pinned corpus contains no Big/Rational atoms"),
            View::Call { head, args } => {
                work.push(head);
                work.extend(args.iter());
                19
            }
        };
        sum = sum.wrapping_add(value);
    }
    sum
}
pub fn cow_original(root: &Expr) -> Expr {
    let Expr::Call { head, mut args } = root.clone() else {
        panic!()
    };
    args.make_mut().push(Expr::small_integer(777));
    Expr::call_shared(head, args)
}
pub fn cow_tree<N: Tree>(root: &N) -> N {
    let View::Call { head, args } = root.view() else {
        panic!()
    };
    let mut args = args.clone();
    args.make_mut()
        .push(N::atom(Atom::Integer(IntValue::Small(777))));
    N::call(head.clone(), args)
}
pub fn appended_checksum(sum: u64) -> u64 {
    sum.wrapping_add(integer(777))
}
