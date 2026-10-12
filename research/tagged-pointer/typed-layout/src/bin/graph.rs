use super::scan;
use rum_core::Expr;
use rum_layout_design::Tagged16 as Previous;
use rum_typed_layout::{Safe24, Typed16};
#[derive(Clone, Copy)]
pub enum Case {
    Original,
    Previous,
    Safe24,
    Moved,
    Borrowed,
}
impl Case {
    pub const ALL: [Self; 5] = [
        Self::Original,
        Self::Previous,
        Self::Safe24,
        Self::Moved,
        Self::Borrowed,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Previous => "previous16",
            Self::Safe24 => "safe24-borrow",
            Self::Moved => "typed16-move",
            Self::Borrowed => "typed16-borrow",
        }
    }
    #[cfg(feature = "allocation-count")]
    pub fn root_bytes(self) -> usize {
        match self {
            Self::Original => 32,
            Self::Safe24 => 24,
            _ => 16,
        }
    }
}
#[derive(Clone)]
pub enum Graph {
    Original(Expr),
    Previous(Previous),
    Safe24(Safe24),
    Moved(Typed16),
    Borrowed(Typed16),
}
impl Graph {
    pub fn build(case: Case, e: &Expr) -> Self {
        match case {
            Case::Original => Self::Original(rum_typed_layout::original_copy(e)),
            Case::Previous => Self::Previous(rum_layout_design::from_expr(e)),
            Case::Safe24 => Self::Safe24(rum_typed_layout::from_expr(e)),
            Case::Moved => Self::Moved(rum_typed_layout::from_expr_moved(e)),
            Case::Borrowed => Self::Borrowed(rum_typed_layout::from_expr(e)),
        }
    }
    pub fn project(&self) -> Expr {
        match self {
            Self::Original(n) => n.clone(),
            Self::Previous(n) => rum_layout_design::to_expr(n),
            Self::Safe24(n) => rum_typed_layout::to_expr(n),
            Self::Moved(n) | Self::Borrowed(n) => rum_typed_layout::to_expr(n),
        }
    }
    pub fn scan(&self) -> u64 {
        match self {
            Self::Original(n) => scan::original(n),
            Self::Previous(n) => scan::previous(n),
            Self::Safe24(n) => scan::tree(n),
            Self::Moved(n) | Self::Borrowed(n) => scan::tree(n),
        }
    }
    pub fn scan_key(&self) -> u64 {
        match self {
            Self::Original(n) => scan::original(n),
            Self::Previous(n) => scan::previous(n),
            Self::Safe24(n) => scan::inspect(n),
            Self::Moved(n) | Self::Borrowed(n) => scan::inspect(n),
        }
    }
    pub fn cow(&self) -> Self {
        match self {
            Self::Original(n) => Self::Original(scan::cow_original(n)),
            Self::Previous(n) => Self::Previous(scan::cow_previous(n)),
            Self::Safe24(n) => Self::Safe24(scan::cow_tree(n)),
            Self::Moved(n) => Self::Moved(scan::cow_tree(n)),
            Self::Borrowed(n) => Self::Borrowed(scan::cow_tree(n)),
        }
    }
}
