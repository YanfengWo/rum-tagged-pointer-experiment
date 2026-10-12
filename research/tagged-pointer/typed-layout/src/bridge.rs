//! One alias-preserving construction algorithm for original and new layouts.
use crate::{Args, Head, Tree};
use rum_core::{Atom, Expr, SharedArgs, SharedExpr};
use std::{collections::HashMap, marker::PhantomData};
mod project;
pub use project::to_expr;

trait Builder {
    type Node;
    type Head: Clone;
    type Args: Clone;
    fn atom(a: &Atom) -> Self::Node;
    fn head(n: Self::Node) -> Self::Head;
    fn args(v: Vec<Self::Node>) -> Self::Args;
    fn call(h: Self::Head, a: Self::Args) -> Self::Node;
}
struct NodeBuilder<N, const BORROW: bool>(PhantomData<N>);
impl<N: Tree, const BORROW: bool> Builder for NodeBuilder<N, BORROW> {
    type Node = N;
    type Head = Head<N>;
    type Args = Args<N>;
    #[inline]
    fn atom(a: &Atom) -> N {
        if BORROW {
            N::borrowed_atom(a)
        } else {
            N::atom(a.clone())
        }
    }
    #[inline]
    fn head(n: N) -> Head<N> {
        Head::new(n)
    }
    #[inline]
    fn args(v: Vec<N>) -> Args<N> {
        Args::new(v)
    }
    #[inline]
    fn call(h: Head<N>, a: Args<N>) -> N {
        N::call(h, a)
    }
}
struct Original;
impl Builder for Original {
    type Node = Expr;
    type Head = SharedExpr;
    type Args = SharedArgs;
    #[inline]
    fn atom(a: &Atom) -> Expr {
        Expr::Atom(a.clone())
    }
    #[inline]
    fn head(n: Expr) -> SharedExpr {
        SharedExpr::new(n)
    }
    #[inline]
    fn args(v: Vec<Expr>) -> SharedArgs {
        SharedArgs::new(v)
    }
    #[inline]
    fn call(h: SharedExpr, a: SharedArgs) -> Expr {
        Expr::call_shared(h, a)
    }
}
enum Work<'a> {
    Expr(&'a Expr),
    Head(&'a SharedExpr),
    Args(&'a SharedArgs),
    SaveHead(usize),
    SaveArgs(usize, usize),
    Call,
}
enum Value<B: Builder> {
    Node(B::Node),
    Head(B::Head),
    Args(B::Args),
}
fn convert<B: Builder>(expr: &Expr) -> B::Node {
    let mut pending = vec![Work::Expr(expr)];
    let mut values: Vec<Value<B>> = Vec::new();
    let mut heads: HashMap<usize, B::Head> = HashMap::new();
    let mut lists: HashMap<usize, B::Args> = HashMap::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Expr(Expr::Atom(a)) => values.push(Value::Node(B::atom(a))),
            Work::Expr(Expr::Call { head, args }) => {
                pending.extend([Work::Call, Work::Args(args), Work::Head(head)])
            }
            Work::Head(head) => {
                let key = head.ptr_key();
                if let Some(h) = heads.get(&key) {
                    values.push(Value::Head(h.clone()))
                } else {
                    pending.extend([Work::SaveHead(key), Work::Expr(head)])
                }
            }
            Work::Args(args) => {
                let key = args.ptr_key();
                if let Some(a) = lists.get(&key) {
                    values.push(Value::Args(a.clone()))
                } else {
                    pending.push(Work::SaveArgs(key, args.len()));
                    pending.extend(args.iter().rev().map(Work::Expr))
                }
            }
            Work::SaveHead(key) => {
                let Some(Value::Node(n)) = values.pop() else {
                    unreachable!()
                };
                let h = B::head(n);
                heads.insert(key, h.clone());
                values.push(Value::Head(h));
            }
            Work::SaveArgs(key, count) => {
                let start = values.len() - count;
                let a = B::args(
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
                values.push(Value::Node(B::call(h, a)));
            }
        }
    }
    assert_eq!(values.len(), 1);
    let Some(Value::Node(n)) = values.pop() else {
        unreachable!()
    };
    n
}
pub fn from_expr<N: Tree>(expr: &Expr) -> N {
    convert::<NodeBuilder<N, true>>(expr)
}
/// Fresh original graph built in one pass using exactly the same work/memo
/// algorithm as the new layouts. No double roundtrip or wrapper is needed.
pub fn original_copy(expr: &Expr) -> Expr {
    convert::<Original>(expr)
}

pub fn from_expr_moved<N: Tree>(expr: &Expr) -> N {
    convert::<NodeBuilder<N, false>>(expr)
}
