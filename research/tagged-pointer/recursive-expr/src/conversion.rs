//! Stack-based, alias-preserving bridges. Memo keys refer only to live borrowed
//! inputs; output memo owners keep completed results alive until construction ends.
use crate::{Args, AtomRef, Head, IntRef, Tree, View};
use rum_core::{Atom, Expr, IntValue, SharedArgs, SharedExpr};
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

pub(crate) fn from_expr<N: Tree>(expr: &Expr) -> N {
    let mut pending = vec![Work::Expr(expr)];
    let mut values = Vec::new();
    let mut heads: HashMap<usize, Head<N>> = HashMap::new();
    let mut lists: HashMap<usize, Args<N>> = HashMap::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Expr(Expr::Atom(atom)) => values.push(Value::Node(N::atom(atom.clone()))),
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

enum Back<'a, N> {
    Node(&'a N),
    Head(&'a Head<N>),
    Args(&'a Args<N>),
    SaveHead(usize),
    SaveArgs(usize, usize),
    Call,
}
enum Original {
    Node(Expr),
    Head(SharedExpr),
    Args(SharedArgs),
}

pub(crate) fn to_expr<N: Tree>(expr: &N) -> Expr {
    let mut pending = vec![Back::Node(expr)];
    let mut values = Vec::new();
    let mut heads: HashMap<usize, SharedExpr> = HashMap::new();
    let mut lists: HashMap<usize, SharedArgs> = HashMap::new();
    while let Some(work) = pending.pop() {
        match work {
            Back::Node(node) => match node.view() {
                View::Atom(atom) => values.push(Original::Node(Expr::Atom(match atom {
                    AtomRef::Integer(IntRef::Small(value)) => Atom::Integer(IntValue::Small(value)),
                    AtomRef::Integer(IntRef::Big(value)) => {
                        Atom::Integer(IntValue::Big(Box::new(value.clone())))
                    }
                    AtomRef::String(value) => Atom::String(value.to_owned()),
                    AtomRef::Symbol(value) => Atom::Symbol(value),
                    AtomRef::Rational(value) => Atom::Rational(Box::new(value.clone())),
                }))),
                View::Call { head, args } => {
                    pending.extend([Back::Call, Back::Args(args), Back::Head(head)])
                }
            },
            Back::Head(head) => {
                let key = head.key();
                if let Some(value) = heads.get(&key) {
                    values.push(Original::Head(value.clone()));
                } else {
                    pending.extend([Back::SaveHead(key), Back::Node(&**head)]);
                }
            }
            Back::Args(args) => {
                let key = args.key();
                if let Some(value) = lists.get(&key) {
                    values.push(Original::Args(value.clone()));
                } else {
                    pending.push(Back::SaveArgs(key, args.len()));
                    pending.extend(args.iter().rev().map(Back::Node));
                }
            }
            Back::SaveHead(key) => {
                let Some(Original::Node(node)) = values.pop() else {
                    unreachable!()
                };
                let head = SharedExpr::new(node);
                heads.insert(key, head.clone());
                values.push(Original::Head(head));
            }
            Back::SaveArgs(key, count) => {
                let start = values.len() - count;
                let args = SharedArgs::new(
                    values
                        .drain(start..)
                        .map(|value| {
                            let Original::Node(node) = value else {
                                unreachable!()
                            };
                            node
                        })
                        .collect(),
                );
                lists.insert(key, args.clone());
                values.push(Original::Args(args));
            }
            Back::Call => {
                let Some(Original::Args(args)) = values.pop() else {
                    unreachable!()
                };
                let Some(Original::Head(head)) = values.pop() else {
                    unreachable!()
                };
                values.push(Original::Node(Expr::call_shared(head, args)));
            }
        }
    }
    assert_eq!(values.len(), 1);
    let Some(Original::Node(node)) = values.pop() else {
        unreachable!()
    };
    node
}
