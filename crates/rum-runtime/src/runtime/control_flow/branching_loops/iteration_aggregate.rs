use super::iteration::{
    DoSpec, IterationConsumer, build_iteration_body_or_nested, evaluate_do_spec_expr, parse_do_spec,
};
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IterationAggregate {
    Product,
    Sum,
}

impl IterationAggregate {
    fn name(self) -> &'static str {
        match self {
            Self::Product => "Product",
            Self::Sum => "Sum",
        }
    }

    fn combiner_name(self) -> &'static str {
        match self {
            Self::Product => "Times",
            Self::Sum => "Plus",
        }
    }

    fn consumer(self) -> IterationConsumer {
        match self {
            Self::Product => IterationConsumer::Product,
            Self::Sum => IterationConsumer::Sum,
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_product(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_iteration_aggregate(IterationAggregate::Product, args)
    }

    pub(in crate::runtime) fn eval_sum(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_iteration_aggregate(IterationAggregate::Sum, args)
    }

    fn eval_iteration_aggregate(
        &mut self,
        aggregate: IterationAggregate,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                aggregate.name(),
                args,
                "argx",
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        self.resume_iteration_aggregate_after_spec_eval(aggregate, args.to_vec(), 0)
    }

    fn resume_iteration_aggregate_after_spec_eval(
        &mut self,
        aggregate: IterationAggregate,
        args: Vec<Expr>,
        spec_index: usize,
    ) -> Result<Expr, EvalError> {
        let spec_result = evaluate_do_spec_expr(self, args[spec_index + 1].clone());
        let spec_expr = self.suspend_with_frame(spec_result, {
            let args = args.clone();
            move |session, spec_expr| {
                session.finish_iteration_aggregate_spec_eval(aggregate, args, spec_index, spec_expr)
            }
        })?;

        self.finish_iteration_aggregate_spec_eval(aggregate, args, spec_index, spec_expr)
    }

    fn finish_iteration_aggregate_spec_eval(
        &mut self,
        aggregate: IterationAggregate,
        args: Vec<Expr>,
        spec_index: usize,
        spec_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let spec = match parse_do_spec(self, &spec_expr, aggregate.consumer())? {
            Some(spec) => spec,
            None => return Ok(Expr::call(system_expr(aggregate.name()), args)),
        };

        let capacity = spec.iteration_count();
        self.resume_iteration_aggregate_iterations(
            aggregate,
            args,
            spec_index,
            spec,
            0,
            Vec::with_capacity(capacity),
        )
    }

    fn resume_iteration_aggregate_iterations(
        &mut self,
        aggregate: IterationAggregate,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        mut iteration: usize,
        mut collected: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &spec {
                DoSpec::Count(count) => {
                    if iteration >= *count {
                        return self.finish_iteration_aggregate(aggregate, collected);
                    }
                    let body = build_iteration_body_or_nested(&args, spec_index, aggregate.name());
                    self.evaluate(body)
                }
                DoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(iteration).cloned() else {
                        return self.finish_iteration_aggregate(aggregate, collected);
                    };
                    let body = build_iteration_body_or_nested(&args, spec_index, aggregate.name());
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(value) => {
                    collected.push(value);
                    iteration += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, outcome| {
                        session.resume_iteration_aggregate_after_iteration(
                            aggregate, args, spec_index, spec, iteration, collected, outcome,
                        )
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_iteration_aggregate_after_iteration(
        &mut self,
        aggregate: IterationAggregate,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        iteration: usize,
        mut collected: Vec<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                collected.push(value);
                self.resume_iteration_aggregate_iterations(
                    aggregate,
                    args,
                    spec_index,
                    spec,
                    iteration + 1,
                    collected,
                )
            }
            Err(error) => Err(error),
        }
    }

    fn finish_iteration_aggregate(
        &mut self,
        aggregate: IterationAggregate,
        values: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.evaluate(Expr::call(system_expr(aggregate.combiner_name()), values))
    }
}
