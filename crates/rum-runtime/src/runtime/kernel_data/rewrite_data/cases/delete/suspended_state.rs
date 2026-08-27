use super::*;

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedDeleteCallChildren {
    pub(super) children: FrozenRewriteQueue<(bool, Expr, bool)>,
    pub(super) child_index: usize,
    pub(super) pattern: Expr,
    pub(super) traversal: CasesTraversalConfig,
    pub(super) level: usize,
    pub(super) subtree_start: usize,
    pub(super) deleted_before: usize,
    pub(super) rebuilt_args: PersistentRewriteExprs,
    pub(super) max_child_depth: usize,
    pub(super) rebuilt_head: Option<Expr>,
    pub(super) head_deleted: bool,
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedDeleteAssociationChildren {
    pub(super) children: FrozenRewriteQueue<(bool, Option<Expr>, Expr, bool)>,
    pub(super) child_index: usize,
    pub(super) pattern: Expr,
    pub(super) traversal: CasesTraversalConfig,
    pub(super) level: usize,
    pub(super) subtree_start: usize,
    pub(super) deleted_before: usize,
    pub(super) rebuilt_entries: PersistentRewriteExprs,
    pub(super) max_child_depth: usize,
    pub(super) rebuilt_head: Option<Expr>,
    pub(super) head_deleted: bool,
}
