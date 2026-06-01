use super::*;

mod names;
mod regex;
mod syntax;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SyntaxInformationNormalizationError {
    InvalidTarget,
    InvalidRules,
}

impl SyntaxInformationNormalizationError {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::InvalidTarget => "expected a symbol or symbol-name string argument",
            Self::InvalidRules => {
                "expected a list of rules whose left-hand sides are string property names"
            }
        }
    }
}

fn string_list_expr(items: Vec<String>) -> Expr {
    Expr::call(
        system_expr("List"),
        items.into_iter().map(Expr::string).collect(),
    )
}
