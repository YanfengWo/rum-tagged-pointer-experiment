#![allow(dead_code)]

use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod apply;
mod spans;

pub(super) use apply::apply_part_selectors;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum AtomicPartSpec {
    Index(isize),
    All,
    Span(PartSpanSpec),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) struct PartSpanSpec {
    pub(super) start: SpanEndpoint,
    pub(super) end: SpanEndpoint,
    pub(super) step: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum SpanEndpoint {
    Index(isize),
    UpTo(Option<isize>),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum PartSelector {
    Single(AtomicPartSpec),
    Key(Expr),
    Multi(Vec<PartSelector>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectedPartShape {
    Leaf(Expr),
    Branch(Vec<SelectedPartShape>),
    AssociationBranch(Vec<(Expr, SelectedPartShape)>),
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<AtomicPartSpec>();
    assert_traces::<PartSelector>();
    assert_traces::<PartSpanSpec>();
    assert_traces::<SpanEndpoint>();
};
