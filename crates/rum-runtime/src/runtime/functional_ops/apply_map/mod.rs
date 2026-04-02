use super::*;

mod normalize;
mod traversal;

use self::normalize::*;
use self::traversal::{
    apply_heads_with_config,
    collect_scan_parts_with_config,
    map_indexed_parts_with_config,
    map_parts_with_config,
    map_parts_with_mode,
};

impl Session {
    pub(in crate::runtime) fn eval_comap(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_comap_call(head, args) {
            CommonArgMapNormalization::OperatorForm(expr)
            | CommonArgMapNormalization::Immediate(expr) => Ok(expr),
            CommonArgMapNormalization::Call {
                functions,
                common_expr,
                config,
            } => {
                let operator =
                    function_body_operator(Expr::call(slot_expr(), vec![common_expr.clone()]));
                let (mapped, _) = map_parts_with_config(&functions, &operator, config);
                self.evaluate(mapped)
            }
        }
    }

    pub(in crate::runtime) fn eval_comap_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_comap_apply_call(head, args) {
            CommonArgMapNormalization::OperatorForm(expr)
            | CommonArgMapNormalization::Immediate(expr) => Ok(expr),
            CommonArgMapNormalization::Call {
                functions,
                common_expr,
                config,
            } => {
                let operator = function_body_operator(Expr::call(
                    system_expr("Apply"),
                    vec![slot_expr(), common_expr.clone()],
                ));
                let (mapped, _) = map_parts_with_config(&functions, &operator, config);
                self.evaluate(mapped)
            }
        }
    }

    pub(in crate::runtime) fn eval_map_indexed(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::MapIndexed) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_indexed_parts_with_config(&expr, &operator, config).0),
        }
    }

    pub(in crate::runtime) fn eval_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::Map) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_parts_with_config(&expr, &operator, config).0),
        }
    }

    pub(in crate::runtime) fn eval_map_all(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_all_call(head, args) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(map_parts_with_mode(&expr, &operator, config, MapTraversalFlavor::MapAll).0),
        }
    }

    pub(in crate::runtime) fn eval_scan(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_map_like_call(head, args, MapLikeFlavor::Scan) {
            MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
                Ok(expr)
            }
            MapLikeNormalization::Call {
                operator,
                expr,
                config,
            } => {
                let parts = collect_scan_parts_with_config(&expr, config);
                self.evaluate_scan_parts(&operator, parts)
            }
        }
    }

    pub(in crate::runtime) fn eval_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_apply_call(head, args, ApplyFlavor::Apply) {
            ApplyNormalization::OperatorForm(expr) | ApplyNormalization::Immediate(expr) => {
                Ok(expr)
            }
            ApplyNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(apply_heads_with_config(&expr, &operator, config, ApplyFlavor::Apply).0),
        }
    }

    pub(in crate::runtime) fn eval_map_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match self.normalize_apply_call(head, args, ApplyFlavor::MapApply) {
            ApplyNormalization::OperatorForm(expr) | ApplyNormalization::Immediate(expr) => {
                Ok(expr)
            }
            ApplyNormalization::Call {
                operator,
                expr,
                config,
            } => Ok(apply_heads_with_config(&expr, &operator, config, ApplyFlavor::MapApply).0),
        }
    }
}
