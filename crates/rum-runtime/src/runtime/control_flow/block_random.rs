use super::*;

impl Session {
    pub(in crate::runtime) fn eval_block_random(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("BlockRandom", args, 1, 2));
        }

        let body_result = self.evaluate(args[0].clone());
        self.suspend_with_frame(body_result, |_, value| Ok(value))
    }
}
