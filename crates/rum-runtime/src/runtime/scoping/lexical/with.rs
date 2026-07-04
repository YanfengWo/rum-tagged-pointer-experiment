use super::*;

impl Session {
    pub(in crate::runtime) fn eval_with(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("With", args, 2));
        }

        let config = match self.compile_with_call_from_raw_args(args) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("With", args, error.message()));
            }
        };

        self.resume_with_bindings(config.items, args[1].clone(), SymbolExprMap::default(), 0)
    }

    fn resume_with_bindings(
        &mut self,
        items: Vec<(Symbol, Expr)>,
        body: Expr,
        mut bindings: SymbolExprMap,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= items.len() {
            let body = substitute_lexical_bindings(&body, &bindings, &BTreeSet::new());
            return self.evaluate(body);
        }

        let (symbol, rhs) = items[index].clone();
        let substituted_rhs = substitute_lexical_bindings(&rhs, &bindings, &BTreeSet::new());
        let rhs_result = self.evaluate(substituted_rhs);
        let outcome = self.suspend_with_frame_lazy(rhs_result, || {
            let items = items.clone();
            let body = body.clone();
            let bindings = bindings.clone();
            let symbol = symbol.clone();
            move |session, value| {
                let mut bindings = bindings;
                bindings.insert(symbol, value);
                session.resume_with_bindings(items, body, bindings, index + 1)
            }
        })?;
        bindings.insert(symbol, outcome);
        self.resume_with_bindings(items, body, bindings, index + 1)
    }
}
