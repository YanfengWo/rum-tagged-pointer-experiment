use super::*;

mod delete;
mod insert;
mod range;
mod take_drop;
mod take_list;

pub(crate) use self::{
    delete::eval_delete,
    insert::eval_insert,
    range::eval_range,
    take_drop::{eval_drop, eval_take, eval_take_drop},
    take_list::eval_take_list,
};
