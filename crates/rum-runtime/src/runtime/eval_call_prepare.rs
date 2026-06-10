use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct CallArgumentPlan {
    expr: Expr,
    should_evaluate: bool,
    allow_sequence_splice: bool,
}

fn plan_call_argument_preparation(
    attributes: AttributeSet,
    index: usize,
    arg: Expr,
) -> CallArgumentPlan {
    let hold = attributes.hold_attribute();
    let blocks_evaluate_override = attributes.blocks_evaluate_override();

    if hold.holds_argument(index) {
        if blocks_evaluate_override {
            return CallArgumentPlan {
                expr: arg,
                should_evaluate: false,
                allow_sequence_splice: false,
            };
        }

        if let Some(inner) = unevaluated_wrapper_arg(&arg) {
            return CallArgumentPlan {
                expr: inner.clone(),
                should_evaluate: false,
                allow_sequence_splice: false,
            };
        }

        if let Some(inner) = evaluate_wrapper_arg(&arg) {
            return CallArgumentPlan {
                expr: inner.clone(),
                should_evaluate: true,
                allow_sequence_splice: true,
            };
        }

        return CallArgumentPlan {
            expr: arg,
            should_evaluate: false,
            allow_sequence_splice: false,
        };
    }

    if let Some(inner) = unevaluated_wrapper_arg(&arg) {
        return CallArgumentPlan {
            expr: inner.clone(),
            should_evaluate: false,
            allow_sequence_splice: true,
        };
    }

    CallArgumentPlan {
        expr: arg,
        should_evaluate: true,
        allow_sequence_splice: true,
    }
}

fn append_prepared_argument(
    prepared_args: &mut Vec<Expr>,
    value: Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) {
    if allow_sequence_splice && !sequence_hold {
        splice_sequence_argument(prepared_args, value);
    } else {
        prepared_args.push(value);
    }
}

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
        let sequence_hold = attributes.sequence_hold();
        let mut remaining_args = remaining_args.into_iter().enumerate();

        while let Some((offset, arg)) = remaining_args.next() {
            let index = start_index + offset;

            let plan = plan_call_argument_preparation(attributes, index, arg);
            let prepared_arg = if plan.should_evaluate {
                let allow_sequence_splice = plan.allow_sequence_splice;
                let evaluated_result = self.evaluate(plan.expr);
                match evaluated_result {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        let head_for_resume = head.clone();
                        let attributes_for_resume = attributes;
                        let evaluated_for_resume = evaluated_args.clone();
                        let remaining_for_resume = remaining_args
                            .by_ref()
                            .map(|(_, arg)| arg)
                            .collect::<Vec<_>>();
                        self.suspend_with_frame(
                            Err(EvalError::InputRequired { request }),
                            move |session, value| {
                                session.resume_call_after_argument(
                                    head_for_resume,
                                    attributes_for_resume,
                                    evaluated_for_resume,
                                    remaining_for_resume,
                                    index + 1,
                                    value,
                                    allow_sequence_splice,
                                )
                            },
                        )?
                    }
                    Err(error) => return Err(error),
                }
            } else {
                plan.expr
            };

            append_prepared_argument(
                &mut evaluated_args,
                prepared_arg,
                plan.allow_sequence_splice,
                sequence_hold,
            );
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
        append_prepared_argument(
            &mut evaluated_args,
            value,
            allow_sequence_splice,
            attributes.sequence_hold(),
        );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_keeps_held_arguments_verbatim_under_holdallcomplete() {
        let arg = Expr::call(system_expr("Evaluate"), vec![Expr::integer(1_i64.into())]);
        let attributes = AttributeSet::from_attribute(Attribute::HoldAllComplete);
        let plan = plan_call_argument_preparation(attributes, 0, arg.clone());

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: arg,
                should_evaluate: false,
                allow_sequence_splice: false,
            }
        );
    }

    #[test]
    fn planner_respects_evaluate_wrapper_for_held_arguments() {
        let inner = Expr::Atom(Atom::Symbol(Symbol::from_name("x")));
        let arg = Expr::call(system_expr("Evaluate"), vec![inner.clone()]);
        let attributes = AttributeSet::from_attribute(Attribute::HoldAll);
        let plan = plan_call_argument_preparation(attributes, 0, arg);

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: inner,
                should_evaluate: true,
                allow_sequence_splice: true,
            }
        );
    }

    #[test]
    fn planner_respects_unevaluated_wrapper_for_nonheld_arguments() {
        let inner = Expr::Atom(Atom::Symbol(Symbol::from_name("x")));
        let arg = Expr::call(system_expr("Unevaluated"), vec![inner.clone()]);
        let plan = plan_call_argument_preparation(AttributeSet::empty(), 0, arg);

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: inner,
                should_evaluate: false,
                allow_sequence_splice: true,
            }
        );
    }
}
