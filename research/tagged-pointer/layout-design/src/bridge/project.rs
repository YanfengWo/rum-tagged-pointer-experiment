use crate::{ArgsRef, AtomRef, HeadRef, IntRef, Tree, View};
use rum_core::{Atom, Expr, IntValue, SharedArgs, SharedExpr};
use std::collections::HashMap;
enum Work<'a, N> {
    Node(&'a N),
    Head(HeadRef<'a, N>),
    Args(ArgsRef<'a, N>),
    SaveHead(usize),
    SaveArgs(usize, usize),
    Call,
}
enum Value {
    Node(Expr),
    Head(SharedExpr),
    Args(SharedArgs),
}
pub fn to_expr<N: Tree>(root: &N) -> Expr {
    let mut pending = vec![Work::Node(root)];
    let mut values = Vec::new();
    let mut heads: HashMap<usize, SharedExpr> = HashMap::new();
    let mut lists: HashMap<usize, SharedArgs> = HashMap::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Node(n) => match n.view() {
                View::Atom(a) => values.push(Value::Node(Expr::Atom(match a {
                    AtomRef::Integer(IntRef::Small(n)) => Atom::Integer(IntValue::Small(n)),
                    AtomRef::Integer(IntRef::Big(n)) => {
                        Atom::Integer(IntValue::Big(Box::new(n.clone())))
                    }
                    AtomRef::Rational(n) => Atom::Rational(Box::new(n.clone())),
                    AtomRef::String(n) => Atom::String(n.to_owned()),
                    AtomRef::Symbol(n) => Atom::Symbol(n),
                }))),
                View::Call { head, args } => {
                    pending.extend([Work::Call, Work::Args(args), Work::Head(head)])
                }
            },
            Work::Head(h) => {
                let key = h.key();
                if let Some(h) = heads.get(&key) {
                    values.push(Value::Head(h.clone()))
                } else {
                    pending.extend([Work::SaveHead(key), Work::Node(h.node())])
                }
            }
            Work::Args(a) => {
                let key = a.key();
                if let Some(a) = lists.get(&key) {
                    values.push(Value::Args(a.clone()))
                } else {
                    pending.push(Work::SaveArgs(key, a.len()));
                    pending.extend(a.slice().iter().rev().map(Work::Node))
                }
            }
            Work::SaveHead(key) => {
                let Some(Value::Node(n)) = values.pop() else {
                    unreachable!()
                };
                let h = SharedExpr::new(n);
                heads.insert(key, h.clone());
                values.push(Value::Head(h));
            }
            Work::SaveArgs(key, count) => {
                let start = values.len() - count;
                let a = SharedArgs::new(
                    values
                        .drain(start..)
                        .map(|v| {
                            let Value::Node(n) = v else { unreachable!() };
                            n
                        })
                        .collect(),
                );
                lists.insert(key, a.clone());
                values.push(Value::Args(a));
            }
            Work::Call => {
                let Some(Value::Args(a)) = values.pop() else {
                    unreachable!()
                };
                let Some(Value::Head(h)) = values.pop() else {
                    unreachable!()
                };
                values.push(Value::Node(Expr::call_shared(h, a)));
            }
        }
    }
    assert_eq!(values.len(), 1);
    let Some(Value::Node(n)) = values.pop() else {
        unreachable!()
    };
    n
}
