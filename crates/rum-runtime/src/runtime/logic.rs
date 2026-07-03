mod comparison;
mod shared;
mod truth;

pub(in crate::runtime) use self::comparison::eval_comparison_operator_wrapper;
pub(in crate::runtime) use self::comparison::eval_equal;
pub(in crate::runtime) use self::comparison::eval_greater;
pub(in crate::runtime) use self::comparison::eval_greater_equal;
pub(in crate::runtime) use self::comparison::eval_less;
pub(in crate::runtime) use self::comparison::eval_less_equal;
pub(in crate::runtime) use self::comparison::eval_unequal;
pub(in crate::runtime) use self::truth::eval_and;
pub(in crate::runtime) use self::truth::eval_equivalent;
pub(in crate::runtime) use self::truth::eval_implies;
pub(in crate::runtime) use self::truth::eval_nand;
pub(in crate::runtime) use self::truth::eval_nor;
pub(in crate::runtime) use self::truth::eval_not;
pub(in crate::runtime) use self::truth::eval_or;
pub(in crate::runtime) use self::truth::eval_xor;
