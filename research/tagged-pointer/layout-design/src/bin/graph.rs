use super::scan;
use rum_core::Expr;
use rum_layout_design::{Safe24, Tagged8, Tagged16};
use rum_recursive_expr::Compact;
use std::hint::black_box;
#[derive(Clone, Copy)]
pub enum Case {
    Original,
    Safe24,
    Tagged16,
    Tagged8,
    Previous,
}
impl Case {
    pub const ALL: [Self; 5] = [
        Self::Original,
        Self::Safe24,
        Self::Tagged16,
        Self::Tagged8,
        Self::Previous,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Safe24 => "safe24",
            Self::Tagged16 => "tagged16",
            Self::Tagged8 => "tagged8",
            Self::Previous => "previous-shared8",
        }
    }
    #[cfg(feature = "allocation-count")]
    pub fn root_bytes(self) -> usize {
        match self {
            Self::Original => std::mem::size_of::<Expr>(),
            Self::Safe24 => std::mem::size_of::<Safe24>(),
            Self::Tagged16 => std::mem::size_of::<Tagged16>(),
            Self::Tagged8 => std::mem::size_of::<Tagged8>(),
            Self::Previous => std::mem::size_of::<Compact>(),
        }
    }
}
#[derive(Clone)]
pub enum Graph {
    Original(Expr),
    Safe24(Safe24),
    Tagged16(Tagged16),
    Tagged8(Tagged8),
    Previous(Compact),
}
impl Graph {
    pub fn build(case: Case, source: &Expr) -> Self {
        match case {
            Case::Original => Self::Original(rum_layout_design::original_copy(source)),
            Case::Safe24 => Self::Safe24(rum_layout_design::from_expr(source)),
            Case::Tagged16 => Self::Tagged16(rum_layout_design::from_expr(source)),
            Case::Tagged8 => Self::Tagged8(rum_layout_design::from_expr(source)),
            Case::Previous => {
                let (n, stats) = rum_symbol_sharing::shared(source, 128);
                black_box(stats);
                Self::Previous(n)
            }
        }
    }
    pub fn project(&self) -> Expr {
        match self {
            Self::Original(n) => n.clone(),
            Self::Safe24(n) => rum_layout_design::to_expr(n),
            Self::Tagged16(n) => rum_layout_design::to_expr(n),
            Self::Tagged8(n) => rum_layout_design::to_expr(n),
            Self::Previous(n) => n.to_expr(),
        }
    }
    pub fn scan(&self) -> u64 {
        match self {
            Self::Original(n) => scan::original(n),
            Self::Safe24(n) => scan::tree(n),
            Self::Tagged16(n) => scan::tree(n),
            Self::Tagged8(n) => scan::tree(n),
            Self::Previous(n) => scan::old_tree(n),
        }
    }
    pub fn cow(&self) -> Self {
        match self {
            Self::Original(n) => Self::Original(scan::cow_original(n)),
            Self::Safe24(n) => Self::Safe24(scan::cow_tree(n)),
            Self::Tagged16(n) => Self::Tagged16(scan::cow_tree(n)),
            Self::Tagged8(n) => Self::Tagged8(scan::cow_tree(n)),
            Self::Previous(n) => Self::Previous(scan::cow_old(n)),
        }
    }
}
