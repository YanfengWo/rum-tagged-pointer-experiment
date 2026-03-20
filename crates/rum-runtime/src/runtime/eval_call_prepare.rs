use super::*;

impl Session {
    pub(super) fn prepare_arguments_for_call(
        &mut self,
        head: Expr,
        attributes: AttributeSet,
        args: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        self.resume_prepared_call_arguments(head, attributes, Vec::new(), args, 0)
    }

    pub(super) fn resume_prepared_call_arguments(
        &mut self,
        head: Expr,
        attributes: AttributeSet,
        mut evaluated_args: Vec<Expr>,
        remaining_args: Vec<Expr>,
        start_index: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        let hold = attributes.hold_attribute();
        let sequence_hold = attributes.sequence_hold();
        let blocks_evaluate_override = attributes.blocks_evaluate_override();

        for offset in 0..remaining_args.len() {
            let arg = remaining_args[offset].clone();
            let index = start_index + offset;
            let remaining_for_resume = remaining_args
                .iter()
                .skip(offset + 1)
                .cloned()
                .collect::<Vec<_>>();

            let (evaluated_arg, allow_sequence_splice) = if hold.holds_argument(index) {
                if blocks_evaluate_override {
                    (arg, false)
                } else if let Some(inner) = unevaluated_wrapper_arg(&arg) {
                    (inner.clone(), false)
                } else if let Some(inner) = evaluate_wrapper_arg(&arg) {
                    let head_for_resume = head.clone();
                    let attributes_for_resume = attributes;
                    let evaluated_for_resume = evaluated_args.clone();
                    let evaluated_result = self.evaluate(inner.clone());
                    (
                        self.suspend_with_frame(evaluated_result, move |session, value| {
                            session.resume_call_after_argument(
                                head_for_resume,
                                attributes_for_resume,
                                evaluated_for_resume,
                                remaining_for_resume,
                                index + 1,
                                value,
                                true,
                            )
                        })?,
                        true,
                    )
                } else {
                    (arg, false)
                }
            } else if let Some(inner) = unevaluated_wrapper_arg(&arg) {
                (inner.clone(), true)
            } else {
                let head_for_resume = head.clone();
                let attributes_for_resume = attributes;
                let evaluated_for_resume = evaluated_args.clone();
                let evaluated_result = self.evaluate(arg);
                (
                    self.suspend_with_frame(evaluated_result, move |session, value| {
                        session.resume_call_after_argument(
                            head_for_resume,
                            attributes_for_resume,
                            evaluated_for_resume,
                            remaining_for_resume,
                            index + 1,
                            value,
                            true,
                        )
                    })?,
                    true,
                )
            };

            if allow_sequence_splice && !sequence_hold {
                splice_sequence_argument(&mut evaluated_args, evaluated_arg);
            } else {
                evaluated_args.push(evaluated_arg);
            }
        }

        Ok(evaluated_args)
    }

    pub(super) fn resume_call_after_argument(
        &mut self,
        head: Expr,
        attributes: AttributeSet,
        mut evaluated_args: Vec<Expr>,
        remaining_args: Vec<Expr>,
        next_index: usize,
        value: Expr,
        allow_sequence_splice: bool,
    ) -> Result<Expr, EvalError> {
        if allow_sequence_splice && !attributes.sequence_hold() {
            splice_sequence_argument(&mut evaluated_args, value);
        } else {
            evaluated_args.push(value);
        }

        let prepared = self.resume_prepared_call_arguments(
            head.clone(),
            attributes,
            evaluated_args,
            remaining_args,
            next_index,
        )?;
        self.resume_call_from_prepared_args(head, prepared)
    }

    pub(super) fn resume_call_from_prepared_args(
        &mut self,
        head: Expr,
        prepared_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let plan = self.build_call_dispatch_plan(&head);
        let state = CallDispatchState {
            raw_args: Vec::new(),
            evaluated_args: Some(self.normalize_call_args_by_attributes(
                &head,
                prepared_args,
                plan.attributes,
            )),
        };

        self.resume_call_with_dispatch_plan(head, plan, state, 0)
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
