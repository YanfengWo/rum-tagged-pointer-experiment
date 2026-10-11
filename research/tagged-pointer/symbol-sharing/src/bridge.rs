//! Stack-based, alias-preserving bridges. Memo keys refer only to live borrowed
//! inputs; output memo owners keep completed results alive until construction ends.
use rum_core::{Atom, Expr, SharedArgs, SharedExpr};
use rum_recursive_expr::{Args, Head, Tree};
use std::collections::HashMap;

enum Work<'a> {
    Expr(&'a Expr),
    Head(&'a SharedExpr),
    Args(&'a SharedArgs),
    SaveHead(usize),
    SaveArgs(usize, usize),
    Call,
}
enum Value<N> {
    Node(N),
    Head(Head<N>),
    Args(Args<N>),
}

pub(crate) fn convert<N: Tree>(expr: &Expr, atom_node: &mut impl FnMut(&Atom) -> N) -> N {
    let mut pending = vec![Work::Expr(expr)];
    let mut values = Vec::new();
    let mut heads: HashMap<usize, Head<N>> = HashMap::new();
    let mut lists: HashMap<usize, Args<N>> = HashMap::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Expr(Expr::Atom(atom)) => values.push(Value::Node(atom_node(atom))),
            Work::Expr(Expr::Call { head, args }) => {
                pending.extend([Work::Call, Work::Args(args), Work::Head(head)]);
            }
            Work::Head(head) => {
                let key = head.ptr_key();
                if let Some(value) = heads.get(&key) {
                    values.push(Value::Head(value.clone()));
                } else {
                    pending.extend([Work::SaveHead(key), Work::Expr(head)]);
                }
            }
            Work::Args(args) => {
                let key = args.ptr_key();
                if let Some(value) = lists.get(&key) {
                    values.push(Value::Args(value.clone()));
                } else {
                    pending.push(Work::SaveArgs(key, args.len()));
                    pending.extend(args.iter().rev().map(Work::Expr));
                }
            }
            Work::SaveHead(key) => {
                let Some(Value::Node(node)) = values.pop() else {
                    unreachable!()
                };
                let head = Head::new(node);
                heads.insert(key, head.clone());
                values.push(Value::Head(head));
            }
            Work::SaveArgs(key, count) => {
                let start = values.len() - count;
                let args = Args::new(
                    values
                        .drain(start..)
                        .map(|value| {
                            let Value::Node(node) = value else {
                                unreachable!()
                            };
                            node
                        })
                        .collect(),
                );
                lists.insert(key, args.clone());
                values.push(Value::Args(args));
            }
            Work::Call => {
                let Some(Value::Args(args)) = values.pop() else {
                    unreachable!()
                };
                let Some(Value::Head(head)) = values.pop() else {
                    unreachable!()
                };
                values.push(Value::Node(N::call(head, args)));
            }
        }
    }
    assert_eq!(values.len(), 1);
    let Some(Value::Node(node)) = values.pop() else {
        unreachable!()
    };
    node
}
