use super::*;

impl Session {
    pub(in crate::runtime) fn eval_delete_duplicates(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_delete_duplicates_normal_call("DeleteDuplicates", args));
        };

        Ok(Expr::call(
            expr_head.as_ref().clone(),
            stable_unique_exprs(expr_args),
        ))
    }

    pub(in crate::runtime) fn eval_delete_duplicates_by(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, function] = args else {
            return Ok(match args.len() {
                1 => Expr::call(head.clone(), args.to_vec()),
                _ => self.invalid_call_with_arg_range("DeleteDuplicatesBy", args, 1, 2),
            });
        };

        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_delete_duplicates_normal_call("DeleteDuplicatesBy", args));
        };

        self.resume_delete_duplicates_by_from_index(
            args.to_vec(),
            expr_head.as_ref().clone(),
            function.clone(),
            expr_args.to_vec(),
            0,
            Vec::new(),
            Vec::new(),
        )
    }

    fn resume_delete_duplicates_by_from_index(
        &mut self,
        call_args: Vec<Expr>,
        expr_head: Expr,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        keys: Vec<Expr>,
        kept: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            return Ok(Expr::call(expr_head, kept));
        };

        let key_result = self.evaluate(Expr::call(function.clone(), vec![item.clone()]));
        let suspended_call_args = call_args.clone();
        let suspended_expr_head = expr_head.clone();
        let suspended_function = function.clone();
        let suspended_items = items.clone();
        let suspended_keys = keys.clone();
        let suspended_kept = kept.clone();
        let suspended_item = item.clone();
        let key = self.suspend_with_frame(key_result, move |session, key| {
            session.resume_delete_duplicates_by_with_key(
                suspended_call_args,
                suspended_expr_head,
                suspended_function,
                suspended_items,
                index,
                suspended_keys,
                suspended_kept,
                suspended_item,
                key,
            )
        })?;

        self.resume_delete_duplicates_by_with_key(
            call_args, expr_head, function, items, index, keys, kept, item, key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_delete_duplicates_by_with_key(
        &mut self,
        call_args: Vec<Expr>,
        expr_head: Expr,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        mut keys: Vec<Expr>,
        mut kept: Vec<Expr>,
        item: Expr,
        key: Expr,
    ) -> Result<Expr, EvalError> {
        if !keys.iter().any(|existing| existing == &key) {
            keys.push(key);
            kept.push(item);
        }
        self.resume_delete_duplicates_by_from_index(
            call_args,
            expr_head,
            function,
            items,
            index + 1,
            keys,
            kept,
        )
    }

    fn invalid_delete_duplicates_normal_call(&mut self, symbol: &str, args: &[Expr]) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "normal",
            format!(
                "Nonatomic expression expected at position 1 in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        )
    }
}

fn stable_unique_exprs(items: &[Expr]) -> Vec<Expr> {
    let mut seen = Vec::new();
    let mut kept = Vec::new();

    for item in items {
        if seen.iter().any(|existing| existing == item) {
            continue;
        }
        seen.push(item.clone());
        kept.push(item.clone());
    }

    kept
}
