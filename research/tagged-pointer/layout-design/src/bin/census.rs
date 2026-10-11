use rum_core::{Atom, Expr, IntValue};
use std::collections::HashSet;
pub fn count(root: &Expr) -> serde_json::Value {
    let mut work = vec![root];
    let mut heads = HashSet::new();
    let mut lists = HashSet::new();
    let (mut nodes, mut owned_atoms, mut tag8_heap, mut plain_symbols, mut generated_symbols) =
        (0, 0, 0, 0, 0);
    while let Some(e) = work.pop() {
        nodes += 1;
        match e {
            Expr::Call { head, args } => {
                tag8_heap += 1;
                if heads.insert(head.ptr_key()) {
                    work.push(head)
                }
                if lists.insert(args.ptr_key()) {
                    work.extend(args.iter())
                }
            }
            Expr::Atom(Atom::Integer(IntValue::Small(n))) => {
                if !(-(1_i64 << 60)..=(1_i64 << 60) - 1).contains(n) {
                    tag8_heap += 1
                }
            }
            Expr::Atom(Atom::Symbol(s)) => {
                if s.identity().is_some() {
                    generated_symbols += 1;
                    tag8_heap += 1
                } else {
                    plain_symbols += 1
                }
            }
            Expr::Atom(_) => {
                owned_atoms += 1;
                tag8_heap += 1
            }
        }
    }
    serde_json::json!({"nodes":nodes,"slots":nodes-1,"owned_atoms":owned_atoms,
        "tagged8_heap_nodes":tag8_heap,"plain_symbols":plain_symbols,"generated_symbols":generated_symbols,
        "unique_heads":heads.len(),"unique_args":lists.len()})
}
