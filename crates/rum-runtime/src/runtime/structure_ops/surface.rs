use super::*;

impl Session {
    pub(in crate::runtime) fn eval_head(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => Ok(structural_head_expr(expr)),
            [expr, wrapper] => Ok(Expr::call(
                wrapper.clone(),
                vec![structural_head_expr(expr)],
            )),
            _ => Ok(self.invalid_call_with_arg_range("Head", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_length(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Length", args, 1));
        };

        Ok(Expr::integer(BigInt::from(structural_length(expr))))
    }

    pub(in crate::runtime) fn eval_depth(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let default_include_heads = self.system_option_bool("Depth", "Heads", false)?;
        let (expr, include_heads) = match args {
            [expr] => (expr, default_include_heads),
            [expr, option] => {
                let classification = classify_heads_option_expr(option);
                let HeadsOptionClassification::Value(include_heads) = classification else {
                    return Ok(self.invalid_heads_option_call(
                        "Depth",
                        args,
                        option,
                        1,
                        classification,
                        "expected a Heads -> True/False option as the second argument",
                    ));
                };
                (expr, include_heads.unwrap_or(default_include_heads))
            }
            _ => return Ok(self.invalid_call_with_arg_range("Depth", args, 1, 2)),
        };

        Ok(Expr::integer(BigInt::from(structural_depth(
            expr,
            include_heads,
        ))))
    }

    pub(in crate::runtime) fn eval_full_form(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FullForm", args, 1));
        };

        Ok(Expr::string(expr.to_string()))
    }

    pub(in crate::runtime) fn eval_first(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => Ok(self.resolve_first_like(expr, None, "First", "nofirst", true)),
            [expr, default] => {
                Ok(self.resolve_first_like(expr, Some(default), "First", "nofirst", true))
            }
            _ => Ok(self.invalid_call_with_arg_range("First", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_last(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => Ok(self.resolve_first_like(expr, None, "Last", "nolast", false)),
            [expr, default] => {
                Ok(self.resolve_first_like(expr, Some(default), "Last", "nolast", false))
            }
            _ => Ok(self.invalid_call_with_arg_range("Last", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_rest(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Rest", args, 1));
        };

        if let Some(entries) = association_entry_forms_from_expr(expr) {
            if entries.is_empty() {
                return Ok(self.invalid_empty_sequence_selection_call("Rest", "norest", expr));
            }
            return Ok(render_association_entry_forms_preserving_heads(
                entries.into_iter().skip(1).collect(),
            ));
        }

        Ok(match expr {
            Expr::Atom(_) => self.invalid_surface_normal_expression_call("Rest", args),
            Expr::Call { args, .. } if args.is_empty() => {
                self.invalid_empty_sequence_selection_call("Rest", "norest", expr)
            }
            Expr::Call { head, args } => Expr::call(
                head.as_ref().clone(),
                args.iter().skip(1).cloned().collect::<Vec<_>>(),
            ),
        })
    }

    pub(in crate::runtime) fn eval_most(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Most", args, 1));
        };

        if let Some(entries) = association_entry_forms_from_expr(expr) {
            if entries.is_empty() {
                return Ok(self.invalid_empty_sequence_selection_call("Most", "nomost", expr));
            }
            let retained_len = entries.len().saturating_sub(1);
            return Ok(render_association_entry_forms_preserving_heads(
                entries.into_iter().take(retained_len).collect(),
            ));
        }

        Ok(match expr {
            Expr::Atom(_) => self.invalid_surface_normal_expression_call("Most", args),
            Expr::Call { args, .. } if args.is_empty() => {
                self.invalid_empty_sequence_selection_call("Most", "nomost", expr)
            }
            Expr::Call { head, args } => Expr::call(
                head.as_ref().clone(),
                args.iter()
                    .take(args.len().saturating_sub(1))
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
        })
    }

    pub(in crate::runtime) fn eval_append(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr, item] => Ok(match expr {
                Expr::Atom(_) => self.invalid_surface_normal_expression_call("Append", args),
                Expr::Call { head, args } => {
                    let mut new_args = args.to_vec();
                    new_args.push(item.clone());
                    Expr::call_with_shared_args(head.as_ref().clone(), SharedArgs::new(new_args))
                }
            }),
            [item] => Ok(unary_operator_form("Append", item.clone())),
            _ => Ok(self.invalid_call_with_arg_range("Append", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_prepend(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr, item] => Ok(match expr {
                Expr::Atom(_) => self.invalid_surface_normal_expression_call("Prepend", args),
                Expr::Call { head, args } => {
                    let mut new_args = Vec::with_capacity(args.len() + 1);
                    new_args.push(item.clone());
                    new_args.extend(args.clone());
                    Expr::call(head.as_ref().clone(), new_args)
                }
            }),
            [item] => Ok(unary_operator_form("Prepend", item.clone())),
            _ => Ok(self.invalid_call_with_arg_range("Prepend", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_symbol(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [name_expr] = args else {
            return Ok(self.invalid_call_with_message(
                "Symbol",
                args,
                "argx",
                format!(
                    "Symbol called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            ));
        };

        let Some(name) = expr_as_string(name_expr) else {
            return Ok(self.invalid_call_with_message(
                "Symbol",
                args,
                "string",
                format!(
                    "String expected at position 1 in {}.",
                    Expr::call(head.clone(), args.to_vec())
                ),
            ));
        };

        Ok(self
            .resolve_symbol_name(name, true)
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(in crate::runtime) fn eval_construct(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(self.invalid_call_with_message(
                "Construct",
                args,
                "argx",
                "expected at least 1 argument",
            ));
        }

        Ok(Expr::call(args[0].clone(), args[1..].to_vec()))
    }

    fn resolve_first_like(
        &mut self,
        expr: &Expr,
        default: Option<&Expr>,
        symbol: &str,
        empty_tag: &str,
        take_first: bool,
    ) -> Expr {
        if let Some(entries) = association_entry_forms_from_expr(expr) {
            let selected = if take_first {
                entries.first()
            } else {
                entries.last()
            };
            return selected
                .map(|entry| entry.value.clone())
                .or_else(|| default.cloned())
                .unwrap_or_else(|| {
                    self.invalid_empty_structural_selection_call(symbol, empty_tag, expr)
                });
        }

        match expr {
            Expr::Call { args, .. } if !args.is_empty() => {
                if take_first {
                    args[0].clone()
                } else {
                    args[args.len() - 1].clone()
                }
            }
            Expr::Call { .. } | Expr::Atom(_) if default.is_some() => {
                default.expect("checked above").clone()
            }
            Expr::Call { .. } => {
                self.invalid_empty_structural_selection_call(symbol, empty_tag, expr)
            }
            Expr::Atom(_) => self.invalid_surface_normal_expression_call(symbol, &[expr.clone()]),
        }
    }

    fn invalid_surface_normal_expression_call(&mut self, symbol: &str, args: &[Expr]) -> Expr {
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

    fn invalid_empty_structural_selection_call(
        &mut self,
        symbol: &str,
        tag: &str,
        expr: &Expr,
    ) -> Expr {
        self.push_message(
            symbol,
            tag,
            format!(
                "{} has zero length and no {} element.",
                expr,
                selection_name(symbol)
            ),
        );
        Expr::call(system_expr(symbol), vec![expr.clone()])
    }

    fn invalid_empty_sequence_selection_call(
        &mut self,
        symbol: &str,
        tag: &str,
        expr: &Expr,
    ) -> Expr {
        self.push_message(
            symbol,
            tag,
            format!(
                "Cannot take {} of expression {} with length zero.",
                symbol, expr
            ),
        );
        Expr::call(system_expr(symbol), vec![expr.clone()])
    }
}

fn unary_operator_form(symbol: &str, item: Expr) -> Expr {
    let slot = Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1))]);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), vec![slot, item])],
    )
}

fn selection_name(symbol: &str) -> &'static str {
    match symbol {
        "First" => "first",
        "Last" => "last",
        _ => "requested",
    }
}
