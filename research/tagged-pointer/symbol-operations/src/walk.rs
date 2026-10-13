//! Preorder includes Call heads before args, matching kernel symbol walks.
use crate::collector::Observation;
use rum_core::{Atom, Expr, Symbol};
use rum_layout_design::{Tree as OldTree, View as OldView};
use rum_typed_layout::{AtomRef, Inspect, SymbolKey, Tree, View};
use std::hash::{Hash, Hasher};
pub struct Original<'a>(Vec<&'a Expr>);
impl<'a> Original<'a> {
    pub fn new(n: &'a Expr) -> Self {
        Self(vec![n])
    }
}
impl Iterator for Original<'_> {
    type Item = Symbol;
    fn next(&mut self) -> Option<Symbol> {
        while let Some(n) = self.0.pop() {
            match n {
                Expr::Atom(Atom::Symbol(s)) => return Some(*s),
                Expr::Call { head, args } => {
                    self.0.extend(args.iter().rev());
                    self.0.push(head);
                }
                _ => {}
            }
        }
        None
    }
}
pub struct Full<'a, N>(Vec<&'a N>);
impl<'a, N> Full<'a, N> {
    pub fn new(n: &'a N) -> Self {
        Self(vec![n])
    }
}
impl<N: Tree> Iterator for Full<'_, N> {
    type Item = Symbol;
    fn next(&mut self) -> Option<Symbol> {
        while let Some(n) = self.0.pop() {
            match n.view() {
                View::Atom(AtomRef::Symbol(s)) => return Some(s),
                View::Call { head, args } => {
                    self.0.extend(args.slice().iter().rev());
                    self.0.push(head.node());
                }
                _ => {}
            }
        }
        None
    }
}
pub struct Previous<'a, N>(Vec<&'a N>);
impl<'a, N> Previous<'a, N> {
    pub fn new(n: &'a N) -> Self {
        Self(vec![n])
    }
}
impl<N: OldTree> Iterator for Previous<'_, N> {
    type Item = Symbol;
    fn next(&mut self) -> Option<Symbol> {
        while let Some(n) = self.0.pop() {
            match n.view() {
                OldView::Atom(AtomRef::Symbol(s)) => return Some(s),
                OldView::Call { head, args } => {
                    self.0.extend(args.slice().iter().rev());
                    self.0.push(head.node());
                }
                _ => {}
            }
        }
        None
    }
}
pub struct Entry<'a, N, const CORE_HASH: bool> {
    node: &'a N,
    key: SymbolKey,
}
impl<N, const C: bool> Copy for Entry<'_, N, C> {}
impl<N, const C: bool> Clone for Entry<'_, N, C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N, const C: bool> PartialEq for Entry<'_, N, C> {
    fn eq(&self, b: &Self) -> bool {
        self.key == b.key
    }
}
impl<N, const C: bool> Eq for Entry<'_, N, C> {}
impl<N, const C: bool> Hash for Entry<'_, N, C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if C {
            state.write_u64(
                u64::from(self.key.interned_id()).wrapping_mul(0x9e3779b97f4a7c15)
                    ^ self.key.identity().unwrap_or(0),
            );
        } else {
            self.key.hash(state);
        }
    }
}
impl<N: Tree, const C: bool> Observation for Entry<'_, N, C> {
    fn resolve(self) -> Symbol {
        let View::Atom(AtomRef::Symbol(s)) = self.node.view() else {
            unreachable!()
        };
        s
    }
}
pub struct Cached<'a, N, const C: bool>(Vec<&'a N>);
impl<'a, N, const C: bool> Cached<'a, N, C> {
    pub fn new(n: &'a N) -> Self {
        Self(vec![n])
    }
}
impl<'a, N: Tree, const C: bool> Iterator for Cached<'a, N, C> {
    type Item = Entry<'a, N, C>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.0.pop() {
            match node.inspect() {
                Inspect::Symbol(key) => return Some(Entry { node, key }),
                Inspect::Call { head, args } => {
                    self.0.extend(args.slice().iter().rev());
                    self.0.push(head.node());
                }
                _ => {}
            }
        }
        None
    }
}
