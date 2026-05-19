use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum HeldUpdateReturn {
    Updated,
    Previous(Expr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum HeldUpdateCurrent {
    Ready { target: Symbol, current: Expr },
    Immediate(Expr),
}

impl Session {
    pub(super) fn eval_add_to(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr, increment] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AddTo", args, 2));
        };

        let (target, current) = match self.current_held_update_value("AddTo", target_expr, args) {
            HeldUpdateCurrent::Ready { target, current } => (target, current),
            HeldUpdateCurrent::Immediate(value) => return Ok(value),
        };

        let update = Expr::call(system_expr("Plus"), vec![current, increment.clone()]);
        self.finish_held_update(target, update, HeldUpdateReturn::Updated)
    }

    pub(super) fn eval_subtract_from(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr, decrement] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("SubtractFrom", args, 2));
        };

        let (target, current) =
            match self.current_held_update_value("SubtractFrom", target_expr, args) {
                HeldUpdateCurrent::Ready { target, current } => (target, current),
                HeldUpdateCurrent::Immediate(value) => return Ok(value),
            };

        let update = Expr::call(system_expr("Subtract"), vec![current, decrement.clone()]);
        self.finish_held_update(target, update, HeldUpdateReturn::Updated)
    }

    pub(super) fn eval_times_by(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr, factor] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("TimesBy", args, 2));
        };

        let (target, current) = match self.current_held_update_value("TimesBy", target_expr, args) {
            HeldUpdateCurrent::Ready { target, current } => (target, current),
            HeldUpdateCurrent::Immediate(value) => return Ok(value),
        };

        let update = Expr::call(system_expr("Times"), vec![current, factor.clone()]);
        self.finish_held_update(target, update, HeldUpdateReturn::Updated)
    }

    pub(super) fn eval_divide_by(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr, divisor] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("DivideBy", args, 2));
        };

        let (target, current) = match self.current_held_update_value("DivideBy", target_expr, args)
        {
            HeldUpdateCurrent::Ready { target, current } => (target, current),
            HeldUpdateCurrent::Immediate(value) => return Ok(value),
        };

        let update = Expr::call(system_expr("Divide"), vec![current, divisor.clone()]);
        self.finish_held_update(target, update, HeldUpdateReturn::Updated)
    }

    pub(super) fn eval_apply_to(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr, function] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("ApplyTo", args, 2));
        };

        let (target, current) = match self.current_held_update_value("ApplyTo", target_expr, args) {
            HeldUpdateCurrent::Ready { target, current } => (target, current),
            HeldUpdateCurrent::Immediate(value) => return Ok(value),
        };

        let update = Expr::call(function.clone(), vec![current]);
        self.finish_held_update(target, update, HeldUpdateReturn::Updated)
    }

    pub(super) fn eval_append_to(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr, item] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AppendTo", args, 2));
        };

        let (target, current) = match self.current_held_update_value("AppendTo", target_expr, args)
        {
            HeldUpdateCurrent::Ready { target, current } => (target, current),
            HeldUpdateCurrent::Immediate(value) => return Ok(value),
        };

        let update = match current {
            Expr::Atom(_) => {
                let held_call = Expr::call(system_expr("AppendTo"), args.to_vec());
                self.push_message(
                    "AppendTo",
                    "normal",
                    format!("Nonatomic expression expected at position 1 in {held_call}."),
                );
                return Ok(held_call);
            }
            Expr::Call { head, mut args } => {
                args.push(item.clone());
                Expr::call(head.as_ref().clone(), args)
            }
        };

        self.finish_held_update(target, update, HeldUpdateReturn::Updated)
    }

    pub(super) fn eval_prepend_to(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr, item] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("PrependTo", args, 2));
        };

        let (target, current) = match self.current_held_update_value("PrependTo", target_expr, args)
        {
            HeldUpdateCurrent::Ready { target, current } => (target, current),
            HeldUpdateCurrent::Immediate(value) => return Ok(value),
        };

        let update = match current {
            Expr::Atom(_) => {
                let held_call = Expr::call(system_expr("PrependTo"), args.to_vec());
                self.push_message(
                    "PrependTo",
                    "normal",
                    format!("Nonatomic expression expected at position 1 in {held_call}."),
                );
                return Ok(held_call);
            }
            Expr::Call { head, mut args } => {
                args.insert(0, item.clone());
                Expr::call(head.as_ref().clone(), args)
            }
        };

        self.finish_held_update(target, update, HeldUpdateReturn::Updated)
    }

    pub(super) fn eval_increment(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Increment", args, 1));
        };

        let (target, current) = match self.current_held_update_value("Increment", target_expr, args)
        {
            HeldUpdateCurrent::Ready { target, current } => (target, current),
            HeldUpdateCurrent::Immediate(value) => return Ok(value),
        };
        let update = Expr::call(
            system_expr("Plus"),
            vec![current.clone(), Expr::integer(BigInt::from(1))],
        );

        self.finish_held_update(target, update, HeldUpdateReturn::Previous(current))
    }

    pub(super) fn eval_decrement(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_unary_increment_update("Decrement", args, Expr::integer(BigInt::from(-1)), true)
    }

    pub(super) fn eval_pre_increment(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_unary_increment_update(
            "PreIncrement",
            args,
            Expr::integer(BigInt::from(1)),
            false,
        )
    }

    pub(super) fn eval_pre_decrement(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_unary_increment_update(
            "PreDecrement",
            args,
            Expr::integer(BigInt::from(-1)),
            false,
        )
    }

    fn eval_unary_increment_update(
        &mut self,
        operator: &'static str,
        args: &[Expr],
        delta: Expr,
        return_previous: bool,
    ) -> Result<Expr, EvalError> {
        let [target_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count(operator, args, 1));
        };

        let (target, current) = match self.current_held_update_value(operator, target_expr, args) {
            HeldUpdateCurrent::Ready { target, current } => (target, current),
            HeldUpdateCurrent::Immediate(value) => return Ok(value),
        };
        let update = Expr::call(system_expr("Plus"), vec![current.clone(), delta]);
        let return_value = if return_previous {
            HeldUpdateReturn::Previous(current)
        } else {
            HeldUpdateReturn::Updated
        };

        self.finish_held_update(target, update, return_value)
    }

    fn current_held_update_value(
        &mut self,
        operator: &'static str,
        target_expr: &Expr,
        args: &[Expr],
    ) -> HeldUpdateCurrent {
        let Some(target) = self.normalize_symbol_target_expr(target_expr, true) else {
            return HeldUpdateCurrent::Immediate(self.invalid_call_with_arg_form(
                operator,
                args,
                "expected a symbol or symbol-name string as the first argument",
            ));
        };

        if !self.ensure_symbol_not_protected_for_write(operator, &target) {
            return HeldUpdateCurrent::Immediate(Expr::call(system_expr(operator), args.to_vec()));
        }

        let Some(current) = self
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.clone())
        else {
            self.push_message(
                operator,
                "rvalue",
                format!("{target} is not a variable with a value, so its value cannot be changed."),
            );
            return HeldUpdateCurrent::Immediate(Expr::call(system_expr(operator), args.to_vec()));
        };

        HeldUpdateCurrent::Ready { target, current }
    }

    fn finish_held_update(
        &mut self,
        target: Symbol,
        update: Expr,
        return_value: HeldUpdateReturn,
    ) -> Result<Expr, EvalError> {
        let update_result = self.evaluate(update);
        let updated = self.suspend_with_frame(update_result, {
            let target = target.clone();
            let return_value = return_value.clone();
            move |session, updated| session.store_held_update(target, updated, return_value)
        })?;

        self.store_held_update(target, updated, return_value)
    }

    fn store_held_update(
        &mut self,
        target: Symbol,
        updated: Expr,
        return_value: HeldUpdateReturn,
    ) -> Result<Expr, EvalError> {
        let definition = self.definitions.entry(target).or_default();
        definition.own_value = Some(updated.clone());
        definition.delayed_own_value = None;

        Ok(match return_value {
            HeldUpdateReturn::Updated => updated,
            HeldUpdateReturn::Previous(value) => value,
        })
    }
}
