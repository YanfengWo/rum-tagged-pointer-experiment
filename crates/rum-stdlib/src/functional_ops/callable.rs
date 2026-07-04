use super::*;

mod composition;
mod curried;
mod head_ops;
mod reverse;

pub(crate) use self::{
    composition::{eval_composition, eval_right_composition},
    curried::{eval_curry_applied, eval_operator_applied},
    head_ops::{eval_operate, eval_through},
    reverse::eval_reverse_applied,
};
