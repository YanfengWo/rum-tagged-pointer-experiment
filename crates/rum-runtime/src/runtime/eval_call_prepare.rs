use super::*;

impl Session {
    pub(super) fn evaluate_call_args_with_attributes(
        &mut self,
        attributes: AttributeSet,
        args: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        let hold = attributes.hold_attribute();
        let sequence_hold = attributes.sequence_hold();
        let blocks_evaluate_override = attributes.blocks_evaluate_override();
        let mut evaluated_args = Vec::new();

        for (index, arg) in args.into_iter().enumerate() {
            let (evaluated_arg, allow_sequence_splice) = if hold.holds_argument(index) {
                if !blocks_evaluate_override {
                    if let Some(inner) = unevaluated_wrapper_arg(&arg) {
                        (inner.clone(), false)
                    } else {
                        self.evaluate_held_argument(arg, blocks_evaluate_override)?
                    }
                } else {
                    self.evaluate_held_argument(arg, blocks_evaluate_override)?
                }
            } else if let Some(inner) = unevaluated_wrapper_arg(&arg) {
                (inner.clone(), true)
            } else {
                (self.evaluate(arg)?, true)
            };

            if allow_sequence_splice && !sequence_hold {
                splice_sequence_argument(&mut evaluated_args, evaluated_arg);
            } else {
                evaluated_args.push(evaluated_arg);
            }
        }

        Ok(evaluated_args)
    }

    pub(super) fn evaluate_held_argument(
        &mut self,
        arg: Expr,
        blocks_evaluate_override: bool,
    ) -> Result<(Expr, bool), EvalError> {
        if blocks_evaluate_override {
            return Ok((arg, false));
        }

        if let Some(inner) = evaluate_wrapper_arg(&arg) {
            return self.evaluate(inner.clone()).map(|value| (value, true));
        }

        Ok((arg, false))
    }

    pub(super) fn attribute_set_for_head(&self, head: &Expr) -> AttributeSet {
        if let Some(function) = parse_function(head) {
            return function.attributes();
        }

        let Some(symbol) = expr_as_symbol(head) else {
            return AttributeSet::empty();
        };

        self.attribute_set_for_symbol(symbol)
    }

    pub(super) fn normalize_call_args_by_attributes(
        &self,
        head: &Expr,
        args: Vec<Expr>,
        attributes: AttributeSet,
    ) -> Vec<Expr> {
        let mut normalized_args = if attributes.contains(Attribute::Flat) {
            flatten_flat_call_args(head, args)
        } else {
            args
        };

        if attributes.contains(Attribute::Orderless) {
            normalized_args.sort_by(compare_expr_canonical);
        }

        normalized_args
    }
}
