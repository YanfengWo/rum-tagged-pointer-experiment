use super::*;

impl Session {
    pub(super) fn capture_loader_snapshot(&self) -> LoaderSnapshot {
        LoaderSnapshot {
            attributes: self.attributes.clone(),
            current_context: self.current_context.clone(),
            context_path: self.context_path.clone(),
            context_stack: self.context_stack.clone(),
            loaded_packages: self.loaded_packages.clone(),
            definitions: self.definitions.clone(),
            message_definitions: self.message_definitions.clone(),
            next_generated_symbol_id: self.next_generated_symbol_id,
            trace_events: self.trace_events.clone(),
            messages: self.messages.clone(),
            last_message_list: self.last_message_list.clone(),
        }
    }

    pub(super) fn restore_loader_snapshot(&mut self, snapshot: LoaderSnapshot) {
        self.attributes = snapshot.attributes;
        self.current_context = snapshot.current_context;
        self.context_path = snapshot.context_path;
        self.context_stack = snapshot.context_stack;
        self.loaded_packages = snapshot.loaded_packages;
        self.definitions = snapshot.definitions;
        self.message_definitions = snapshot.message_definitions;
        self.next_generated_symbol_id = snapshot.next_generated_symbol_id;
        self.trace_events = snapshot.trace_events;
        self.messages = snapshot.messages;
        self.last_message_list = snapshot.last_message_list;
    }

    pub(super) fn push_message(&mut self, symbol: &str, tag: &str, text: impl Into<String>) {
        self.messages.push(RuntimeMessage {
            symbol: symbol.to_string(),
            tag: tag.to_string(),
            text: text.into(),
        });
    }

    pub(super) fn runtime_message_for_non_loader_eval_error(
        error: &EvalError,
    ) -> Option<RuntimeMessage> {
        let tag = match error {
            EvalError::EvaluationCycleDetected { .. } => "cycle",
            EvalError::RecursionLimitExceeded { .. } => "reclim",
            EvalError::IterationLimitExceeded { .. } => "iterlim",
            EvalError::EvaluationPassLimitExceeded { .. } => "passlim",
            _ => return None,
        };

        Some(RuntimeMessage {
            symbol: "General".to_string(),
            tag: tag.to_string(),
            text: error.to_string(),
        })
    }

    pub(super) fn emit_non_loader_eval_message(
        &mut self,
        checkpoint: usize,
        error: &EvalError,
    ) -> bool {
        let Some(message) = Self::runtime_message_for_non_loader_eval_error(error) else {
            return false;
        };

        if self.handled_non_loader_eval_errors[checkpoint..]
            .iter()
            .any(|existing| existing == &message)
        {
            return true;
        }

        self.handled_non_loader_eval_errors.push(message.clone());
        self.messages.push(message);
        true
    }

    pub(super) fn emit_loader_message(&mut self, symbol: &str, error: &EvalError) {
        match error {
            EvalError::LoadPathNotFound { target, searched } => {
                let detail = if searched.is_empty() {
                    format!("could not locate `{target}`")
                } else {
                    format!("could not locate `{target}` in {}", searched.join(", "))
                };
                self.push_message(symbol, "notfound", detail);
            }
            EvalError::Io { path, source } => {
                self.push_message(symbol, "io", format!("could not read `{path}`: {source}"));
            }
            EvalError::Syntax(err) => {
                self.push_message(symbol, "syntax", err.to_string());
            }
            other => {
                self.push_message(symbol, "load", other.to_string());
            }
        }
    }

    pub(super) fn invalid_call(&self, symbol: &str, args: &[Expr]) -> Expr {
        Expr::call(system_expr(symbol), args.to_vec())
    }

    pub(super) fn invalid_call_with_message(
        &mut self,
        symbol: &str,
        args: &[Expr],
        tag: &str,
        text: impl Into<String>,
    ) -> Expr {
        self.push_message(symbol, tag, text);
        self.invalid_call(symbol, args)
    }

    pub(super) fn invalid_call_with_exact_arg_count(
        &mut self,
        symbol: &str,
        args: &[Expr],
        expected: usize,
    ) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "argx",
            format!(
                "expected exactly {expected} argument(s), got {}",
                args.len()
            ),
        )
    }

    pub(super) fn invalid_call_with_arg_range(
        &mut self,
        symbol: &str,
        args: &[Expr],
        min: usize,
        max: usize,
    ) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "argtu",
            format!(
                "expected between {min} and {max} argument(s), got {}",
                args.len()
            ),
        )
    }

    pub(super) fn invalid_call_with_arg_form(
        &mut self,
        symbol: &str,
        args: &[Expr],
        detail: impl Into<String>,
    ) -> Expr {
        self.invalid_call_with_message(symbol, args, "argform", detail)
    }

    pub(super) fn messages_since_match(
        &self,
        checkpoint: usize,
        selectors: &[MessageSelector],
    ) -> bool {
        self.messages[checkpoint..]
            .iter()
            .any(|message| selectors.iter().any(|selector| selector.matches(message)))
    }

    pub(super) fn suppress_messages_since(
        &mut self,
        checkpoint: usize,
        selectors: &[MessageSelector],
    ) {
        let trailing_messages = self.messages.split_off(checkpoint);

        for message in trailing_messages {
            if !selectors.iter().any(|selector| selector.matches(&message)) {
                self.messages.push(message);
            }
        }
    }
}
