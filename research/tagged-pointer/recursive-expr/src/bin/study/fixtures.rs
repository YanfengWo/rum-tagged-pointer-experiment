use rum_core::{Atom, Expr, IntValue, SharedArgs, SharedExpr};
use rum_recursive_expr::{AtomRef, IntRef, Tree, View};

pub const DATASETS: [&str; 5] = [
    "wide-integer",
    "wide-string",
    "balanced",
    "shared-dag",
    "deep-chain",
];
pub fn fixture(name: &str) -> Expr {
    let head = SharedExpr::new(Expr::symbol("f"));
    let width = if cfg!(miri) { 8 } else { 10_000 };
    match name {
        "wide-integer" => Expr::call_shared(
            head,
            SharedArgs::new((0..width).map(Expr::small_integer).collect()),
        ),
        "wide-string" => Expr::call_shared(
            head,
            SharedArgs::new((0..width).map(|_| Expr::string("payload")).collect()),
        ),
        "balanced" => {
            fn build(depth: usize, head: &SharedExpr) -> Expr {
                if depth == 0 {
                    return Expr::small_integer(7);
                }
                Expr::call_shared(
                    head.clone(),
                    SharedArgs::new((0..4).map(|_| build(depth - 1, head)).collect()),
                )
            }
            build(if cfg!(miri) { 2 } else { 6 }, &head)
        }
        "shared-dag" => {
            let args = SharedArgs::new(
                (0..if cfg!(miri) { 4 } else { 1_000 })
                    .map(Expr::small_integer)
                    .collect(),
            );
            let child = Expr::call_shared(head.clone(), args);
            Expr::call_shared(
                head,
                SharedArgs::new(
                    (0..if cfg!(miri) { 3 } else { 200 })
                        .map(|_| child.clone())
                        .collect(),
                ),
            )
        }
        "deep-chain" => {
            let head = SharedExpr::new(Expr::small_integer(0));
            let mut node = Expr::small_integer(7);
            for _ in 0..if cfg!(miri) { 8 } else { 256 } {
                node = Expr::call_shared(head.clone(), SharedArgs::new(vec![node]));
            }
            node
        }
        _ => panic!("unknown fixture"),
    }
}

pub fn checksum<N: Tree>(node: &N) -> u64 {
    let mut pending = vec![node];
    let mut sum = 0_u64;
    while let Some(node) = pending.pop() {
        match node.view() {
            View::Atom(AtomRef::Integer(IntRef::Small(value))) => {
                sum = sum.wrapping_add(value as u64)
            }
            View::Atom(AtomRef::String(text)) => sum += text.len() as u64,
            View::Atom(AtomRef::Symbol(_)) => sum += 13,
            View::Atom(_) => sum += 19,
            View::Call { head, args } => {
                sum += 17;
                pending.push(&**head);
                pending.extend(args.iter());
            }
        }
    }
    sum
}
pub fn original_checksum(expr: &Expr) -> u64 {
    let mut pending = vec![expr];
    let mut sum = 0_u64;
    while let Some(node) = pending.pop() {
        match node {
            Expr::Atom(Atom::Integer(IntValue::Small(value))) => {
                sum = sum.wrapping_add(*value as u64)
            }
            Expr::Atom(Atom::String(text)) => sum += text.len() as u64,
            Expr::Atom(Atom::Symbol(_)) => sum += 13,
            Expr::Atom(_) => sum += 19,
            Expr::Call { head, args } => {
                sum += 17;
                pending.push(head);
                pending.extend(args.iter());
            }
        }
    }
    sum
}
pub fn release_original(expr: Expr) {
    let mut pending = vec![expr];
    while let Some(node) = pending.pop() {
        if let Expr::Call { head, mut args } = node {
            pending.push(head.into_owned());
            // Fixtures have atom heads and no Weak observers. Avoid cloning a
            // shared DAG list just to release this count. Last unique lists can
            // move their children out through rum's safe get_mut interface.
            if let Some(values) = args.get_mut() {
                pending.extend(std::mem::take(values));
            }
        }
    }
}
