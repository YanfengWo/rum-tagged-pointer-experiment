//! Memoize both nodes and argument vectors: work is linear in physical slots.
use crate::graph::Counts;
use rum_core::{Expr, SharedArgs};
use std::collections::HashMap;

type Summary = (Counts, usize, bool);
enum Work<'a> {
    Node(&'a Expr, bool),
    Args(&'a SharedArgs, bool),
}

pub fn logical(root: &Expr) -> Summary {
    let mut nodes: HashMap<usize, Summary> = HashMap::new();
    let mut lists: HashMap<usize, Summary> = HashMap::new();
    let mut work = vec![Work::Node(root, false)];
    while let Some(task) = work.pop() {
        match task {
            Work::Node(node, finish) => {
                let key = std::ptr::from_ref(node).addr();
                if nodes.contains_key(&key) {
                    continue;
                }
                if !finish {
                    work.push(Work::Node(node, true));
                    if let Expr::Call { head, args } = node {
                        work.push(Work::Node(head, false));
                        work.push(Work::Args(args, false));
                    }
                } else {
                    let mut result = (Counts::one(node), 1, false);
                    if let Expr::Call { head, args } = node {
                        merge(
                            &mut result,
                            &nodes[&std::ptr::from_ref(head.as_ref()).addr()],
                            1,
                        );
                        merge(&mut result, &lists[&args.ptr_key()], 1);
                    }
                    result.2 |= result.0.total() == u64::MAX;
                    nodes.insert(key, result);
                }
            }
            Work::Args(args, finish) => {
                let key = args.ptr_key();
                if lists.contains_key(&key) {
                    continue;
                }
                if !finish {
                    work.push(Work::Args(args, true));
                    work.extend(args.iter().map(|arg| Work::Node(arg, false)));
                } else {
                    let mut result = (Counts::default(), 0, false);
                    for arg in args.iter() {
                        merge(&mut result, &nodes[&std::ptr::from_ref(arg).addr()], 0);
                    }
                    result.2 |= result.0.total() == u64::MAX;
                    lists.insert(key, result);
                }
            }
        }
    }
    nodes.remove(&std::ptr::from_ref(root).addr()).unwrap()
}

fn merge(target: &mut Summary, source: &Summary, depth_increment: usize) {
    target.0.add(&source.0, &mut target.2);
    target.1 = target.1.max(source.1 + depth_increment);
    target.2 |= source.2;
}
