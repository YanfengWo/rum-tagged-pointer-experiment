use super::*;

mod guarded;
mod guarded_plan;
mod parsing;
mod plan;
mod replace_at;
mod replace_part;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplacePartRule {
    position_specs: Vec<ReplacePartPositionSpec>,
    rhs: Expr,
    guards: Box<[Expr]>,
    delayed: bool,
    heads: ReplacePartHeadsMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReplacePartPositionSpec {
    Exact(Vec<PartSelector>),
    Pattern(Expr),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ReplacePlanNode {
    action: Option<ReplacePlanAction>,
    children: BTreeMap<isize, ReplacePlanNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReplacePlanAction {
    Immediate(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplacePartHeadsMode {
    Automatic,
    Include,
    Exclude,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplacePartTarget {
    path: Vec<isize>,
    rhs: Expr,
    delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GuardedReplacePartTarget {
    path: Vec<isize>,
    rule_index: usize,
    bindings: SymbolExprMap,
    option_scope: Option<OptionValueScope>,
    state: GuardedReplacePartTargetState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GuardedReplacePartTargetState {
    PatternCandidate,
    PendingGuard,
    Accepted,
    Inactive,
}
