use super::*;

impl Session {
    pub(in crate::runtime) fn resolve_argument_exprs_owned(
        &mut self,
        exprs: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        let Some((first, rest)) = exprs.split_first() else {
            return Ok(Vec::new());
        };

        let first_result = self.evaluate(first.clone());
        let rest = rest.to_vec();
        self.drive_evaluated_value(first_result, {
            let rest = rest.clone();
            move |session, first| session.resume_resolve_argument_exprs_after_first(first, rest)
        })
    }

    fn resume_resolve_argument_exprs_after_first(
        &mut self,
        first: Expr,
        rest: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        let rest_result = self.resolve_argument_exprs_owned(rest);
        self.drive_internal_value(rest_result, move |_, mut resolved_rest| {
            let mut resolved = Vec::with_capacity(resolved_rest.len() + 1);
            resolved.push(first);
            resolved.append(&mut resolved_rest);
            Ok(resolved)
        })
    }
}
