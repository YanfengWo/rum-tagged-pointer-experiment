use super::*;

impl Session {
    pub(super) fn eval_release_hold(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("ReleaseHold", args, 1));
        }

        let mut released = args[0].clone();

        while let Some(next) = release_hold_once(&released) {
            released = next;
        }

        let released_result = self.evaluate(released);
        self.suspend_with_frame(released_result, |_, value| Ok(value))
    }

    pub(super) fn eval_thread(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (expr, thread_head, arg_spec) = match args {
            [expr] => (expr.clone(), system_expr("List"), ThreadArgSpec::All),
            [expr, thread_head] => (expr.clone(), thread_head.clone(), ThreadArgSpec::All),
            [expr, thread_head, spec] => {
                let Some(arg_spec) = parse_thread_arg_spec(spec) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Thread",
                        args,
                        "expected the third argument to be a standard sequence specification such as All, None, n, -n, {n}, {m, n}, or {m, n, s}",
                    ));
                };

                (expr.clone(), thread_head.clone(), arg_spec)
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "Thread",
                    args,
                    "argx",
                    format!("expected 1, 2, or 3 argument(s), got {}", args.len()),
                ));
            }
        };

        let Expr::Call {
            head: threaded_head,
            args: threaded_args,
        } = &expr
        else {
            return Ok(expr);
        };

        match self.thread_call_args(
            threaded_head.as_ref(),
            threaded_args,
            &thread_head,
            &thread_head,
            &arg_spec,
        )? {
            ThreadingDispatch::NotApplicable => Ok(expr),
            ThreadingDispatch::Threaded(threaded) => Ok(threaded),
            ThreadingDispatch::Mismatch => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }
}
