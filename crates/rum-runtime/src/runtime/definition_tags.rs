use super::*;

impl Session {
    pub(super) fn eval_tag_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("TagSet", args, 3));
        }

        let Some(tag) = self.lookup_attribute_target(&args[0], true) else {
            return Ok(self.invalid_call_with_arg_form(
                "TagSet",
                args,
                "expected a symbol or symbol-name string tag as the first argument",
            ));
        };

        let lhs = args[1].clone();
        let rhs_result = self.evaluate(args[2].clone());
        let rhs = self.suspend_with_frame(rhs_result, {
            let tag = tag.clone();
            let lhs = lhs.clone();
            let args = args.to_vec();
            move |session, rhs| {
                session.store_tagged_definition("TagSet", &args, tag, lhs, rhs, false)
            }
        })?;
        self.store_tagged_definition("TagSet", args, tag, lhs, rhs, false)
    }

    pub(super) fn eval_tag_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("TagSetDelayed", args, 3));
        }

        let Some(tag) = self.lookup_attribute_target(&args[0], true) else {
            return Ok(self.invalid_call_with_arg_form(
                "TagSetDelayed",
                args,
                "expected a symbol or symbol-name string tag as the first argument",
            ));
        };

        let lhs = args[1].clone();
        let rhs = args[2].clone();
        self.store_tagged_definition("TagSetDelayed", args, tag, lhs, rhs, true)
    }

    pub(super) fn eval_tag_unset(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("TagUnset", args, 2));
        }

        let Some(tag) = self.lookup_attribute_target(&args[0], true) else {
            return Ok(self.invalid_call_with_arg_form(
                "TagUnset",
                args,
                "expected a symbol or symbol-name string tag as the first argument",
            ));
        };

        match self.unset_tagged_definition(&tag, &args[1]) {
            Ok(_) => {
                self.prune_definition_if_empty(&tag);
                Ok(system_expr("Null"))
            }
            Err(error) => Ok(self.invalid_tagged_definition_call("TagUnset", args, error)),
        }
    }

    pub(super) fn invalid_tagged_definition_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        error: TagDefinitionError,
    ) -> Expr {
        match error {
            TagDefinitionError::TagNotFound => self.invalid_call_with_message(
                symbol,
                args,
                "tagnf",
                "tag symbol does not occur in the assignment lhs",
            ),
            TagDefinitionError::TagPositionInvalid => self.invalid_call_with_message(
                symbol,
                args,
                "tagpos",
                "tag symbol must appear within the outer two levels of the assignment lhs",
            ),
        }
    }

    pub(super) fn store_tagged_definition(
        &mut self,
        symbol: &str,
        args: &[Expr],
        tag: Symbol,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        match classify_tagged_definition(&tag, &lhs) {
            Ok(target) => {
                self.store_definition_target(target, lhs, rhs.clone(), delayed);
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            Err(error) => Ok(self.invalid_tagged_definition_call(symbol, args, error)),
        }
    }

    pub(super) fn store_definition_target(
        &mut self,
        target: TaggedDefinitionTarget,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) {
        match target {
            TaggedDefinitionTarget::OwnValue(symbol) => {
                let definition = self.definitions.entry(symbol.clone()).or_default();

                if delayed {
                    definition.own_value = None;
                    definition.delayed_own_value = Some(rhs.clone());
                    self.push_trace(TraceEvent::DefinitionStored {
                        symbol: symbol.to_string(),
                        kind: DefinitionKind::DelayedOwnValue,
                        value: rhs.to_string(),
                    });
                } else {
                    definition.own_value = Some(rhs.clone());
                    definition.delayed_own_value = None;
                    self.push_trace(TraceEvent::DefinitionStored {
                        symbol: symbol.to_string(),
                        kind: DefinitionKind::OwnValue,
                        value: rhs.to_string(),
                    });
                }
            }
            TaggedDefinitionTarget::DownValue(symbol) => {
                let definition = self.definitions.entry(symbol.clone()).or_default();
                definition.down_values.push(DownValue {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    delayed,
                });
                self.push_trace(TraceEvent::DefinitionStored {
                    symbol: symbol.to_string(),
                    kind: DefinitionKind::DownValue,
                    value: format!("{} {} {}", lhs, if delayed { ":=" } else { "=" }, rhs),
                });
            }
            TaggedDefinitionTarget::SubValue(symbol) => {
                let definition = self.definitions.entry(symbol.clone()).or_default();
                definition.sub_values.push(SubValue {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    delayed,
                });
                self.push_trace(TraceEvent::DefinitionStored {
                    symbol: symbol.to_string(),
                    kind: DefinitionKind::SubValue,
                    value: format!("{} {} {}", lhs, if delayed { ":=" } else { "=" }, rhs),
                });
            }
            TaggedDefinitionTarget::UpValue(symbol) => {
                let definition = self.definitions.entry(symbol.clone()).or_default();
                definition.up_values.push(UpValue {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    delayed,
                });
                self.push_trace(TraceEvent::DefinitionStored {
                    symbol: symbol.to_string(),
                    kind: DefinitionKind::UpValue,
                    value: format!("{} {} {}", lhs, if delayed { ":=" } else { "=" }, rhs),
                });
            }
        }
    }
}
