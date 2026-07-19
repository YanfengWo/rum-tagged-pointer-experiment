use super::*;

pub(in crate::runtime) const SEALED_IDENTITY_MARKER: u64 = 1 << 62;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct SourceUnit {
    pub(in crate::runtime) package_context: String,
    pub(in crate::runtime) held_wrapper: Expr,
    pub(in crate::runtime) epoch: u64,
    pub(in crate::runtime) public: Vec<Symbol>,
    pub(in crate::runtime) package_scope: Vec<Symbol>,
    pub(in crate::runtime) private_index: BTreeMap<String, Symbol>,
}

#[derive(Clone)]
struct PackageInstallPlan {
    forms: Vec<Expr>,
    source_unit: SourceUnit,
    minted_sealed: Vec<Symbol>,
    replacement_index: Option<usize>,
}

#[derive(Default)]
struct PackageDeclarations {
    public: Vec<Symbol>,
    package_scope: Vec<Symbol>,
}

impl Session {
    pub(in crate::runtime) fn eval_define_package(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let Some(package_context) = args.first().and_then(valid_package_context_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "DefinePackage",
                args,
                "expected a non-empty valid context string with a trailing context separator as the first argument",
            ));
        };

        let held_wrapper = Expr::call(system_expr("DefinePackage"), args.to_vec());
        let (declarations, body_forms) =
            match collect_package_declarations_and_body(&args[1..], &self.current_context) {
                Ok(parts) => parts,
                Err(detail) => return Ok(self.package_declaration_failure(detail)),
            };

        let bound = scan_package_bindings(self, &body_forms, &self.current_context);
        let mut rewrites = SymbolExprMap::default();
        let mut public = Vec::with_capacity(declarations.public.len());
        let mut package_scope = Vec::with_capacity(declarations.package_scope.len());
        let public_names = declarations
            .public
            .iter()
            .map(|symbol| symbol.name().to_string())
            .collect::<BTreeSet<_>>();
        let package_scope_names = declarations
            .package_scope
            .iter()
            .map(|symbol| symbol.name().to_string())
            .collect::<BTreeSet<_>>();

        for source in declarations.public {
            let target = Symbol::new(package_context.clone(), source.name());
            rewrites.insert(source, Expr::Atom(Atom::Symbol(target)));
            public.push(target);
        }

        let package_scope_context = format!("{package_context}PackageScope`");
        for source in declarations.package_scope {
            let target = Symbol::new(package_scope_context.clone(), source.name());
            rewrites.insert(source, Expr::Atom(Atom::Symbol(target)));
            package_scope.push(target);
        }

        let private_context = format!("{package_context}Private`");
        let mut private_index = BTreeMap::new();
        for source in bound {
            if public_names.contains(source.name()) || package_scope_names.contains(source.name()) {
                continue;
            }

            let target = self.allocate_sealed_symbol(&private_context, source.name());
            rewrites.insert(source, Expr::Atom(Atom::Symbol(target)));
            private_index.insert(target.name().to_string(), target);
        }

        let forms = body_forms
            .iter()
            .map(|form| substitute_lexical_bindings(form, &rewrites, &BTreeSet::new()))
            .collect();
        let minted_sealed = private_index.values().copied().collect::<Vec<_>>();
        let replacement_index = self
            .sealed_units
            .iter()
            .position(|unit| unit.package_context == package_context);

        if let Some(index) = replacement_index {
            let old_sealed = self.sealed_units[index]
                .private_index
                .values()
                .copied()
                .collect::<Vec<_>>();
            for symbol in old_sealed {
                self.clear_symbol_state(&symbol, true, true, true, true, true);
            }
        }

        self.resume_package_install(
            PackageInstallPlan {
                forms,
                source_unit: SourceUnit {
                    package_context,
                    held_wrapper,
                    epoch: 0,
                    public,
                    package_scope,
                    private_index,
                },
                minted_sealed,
                replacement_index,
            },
            0,
        )
    }

    fn allocate_sealed_symbol(&mut self, private_context: &str, name: &str) -> Symbol {
        self.next_sealed_symbol_id += 1;
        Symbol::with_identity(
            private_context,
            name,
            SEALED_IDENTITY_MARKER | self.next_sealed_symbol_id,
        )
    }

    fn resume_package_install(
        &mut self,
        plan: PackageInstallPlan,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= plan.forms.len() {
            return Ok(self.finish_package_install(plan));
        }

        let form_result = self.evaluate(plan.forms[index].clone());
        let outcome = self.suspend_with_result_frame_lazy(form_result, || {
            let plan = plan.clone();
            move |session, outcome| session.resume_package_after_form(plan, index, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_package_after_form(plan, index, outcome),
        }
    }

    fn resume_package_after_form(
        &mut self,
        plan: PackageInstallPlan,
        index: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) => self.resume_package_install(plan, index + 1),
            Err(error) => Ok(self.fail_package_install(plan, error)),
        }
    }

    fn finish_package_install(&mut self, mut plan: PackageInstallPlan) -> Expr {
        self.next_source_unit_epoch += 1;
        plan.source_unit.epoch = self.next_source_unit_epoch;
        let package_context = plan.source_unit.package_context.clone();

        if let Some(index) = plan.replacement_index {
            self.sealed_units[index] = plan.source_unit;
        } else {
            self.sealed_units.push(plan.source_unit);
        }

        Expr::string(package_context)
    }

    fn fail_package_install(&mut self, plan: PackageInstallPlan, error: EvalError) -> Expr {
        for symbol in &plan.minted_sealed {
            self.clear_symbol_state(symbol, true, true, true, true, true);
        }

        if let Some(index) = plan.replacement_index {
            self.sealed_units.remove(index);
        }

        self.push_message(
            "DefinePackage",
            "failed",
            format!(
                "Registration of {} failed while evaluating its body: {error}",
                plan.source_unit.package_context
            ),
        );
        system_expr("$Failed")
    }

    fn package_declaration_failure(&mut self, detail: &str) -> Expr {
        self.push_message("DefinePackage", "decl", detail);
        system_expr("$Failed")
    }
}

fn valid_package_context_expr(expr: &Expr) -> Option<String> {
    let Expr::Atom(Atom::String(context)) = expr else {
        return None;
    };

    let body = context.strip_suffix('`')?;
    if body.is_empty() || !body.split('`').all(valid_context_part) {
        return None;
    }

    Some(normalize_context_name(context.clone()))
}

fn valid_context_part(part: &str) -> bool {
    let mut chars = part.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '$')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '$')
}

fn collect_package_declarations_and_body(
    args: &[Expr],
    current_context: &str,
) -> Result<(PackageDeclarations, Vec<Expr>), &'static str> {
    let mut declarations = PackageDeclarations::default();
    let mut saw_public = false;
    let mut saw_package_scope = false;
    let mut body = Vec::new();

    for arg in args {
        let Expr::Call { head, args } = arg else {
            flatten_compound_expression(arg, &mut body);
            continue;
        };

        let target = if is_system_symbol_name(head, "Public") {
            if saw_public {
                return Err("Public may be declared at most once per source unit");
            }
            saw_public = true;
            &mut declarations.public
        } else if is_system_symbol_name(head, "PackageScope") {
            if saw_package_scope {
                return Err("PackageScope may be declared at most once per source unit");
            }
            saw_package_scope = true;
            &mut declarations.package_scope
        } else {
            flatten_compound_expression(arg, &mut body);
            continue;
        };

        let [
            Expr::Call {
                head: list_head,
                args: symbols,
            },
        ] = args.as_slice()
        else {
            return Err("package declarations must have the form Head[{symbols...}]");
        };
        if !is_system_symbol_name(list_head, "List") {
            return Err("package declarations must contain a list of plain symbols");
        }

        let mut seen = target.iter().copied().collect::<BTreeSet<_>>();
        for item in symbols {
            let Some(symbol) = expr_as_symbol(item).copied() else {
                return Err("package declarations must contain only plain symbols");
            };
            if symbol.context() != current_context {
                return Err("package declarations must contain only current-context symbols");
            }
            if seen.insert(symbol) {
                target.push(symbol);
            }
        }
    }

    let public_names = declarations
        .public
        .iter()
        .map(Symbol::name)
        .collect::<BTreeSet<_>>();
    if declarations
        .package_scope
        .iter()
        .any(|symbol| public_names.contains(symbol.name()))
    {
        return Err("a symbol cannot be both Public and PackageScope");
    }

    Ok((declarations, body))
}

fn flatten_compound_expression(expr: &Expr, forms: &mut Vec<Expr>) {
    if let Expr::Call { head, args } = expr {
        if is_system_symbol_name(head, "CompoundExpression") {
            for arg in args {
                flatten_compound_expression(arg, forms);
            }
            return;
        }
    }

    forms.push(expr.clone());
}

fn scan_package_bindings(
    session: &Session,
    forms: &[Expr],
    current_context: &str,
) -> BTreeSet<Symbol> {
    let mut bound = BTreeSet::new();

    for form in forms {
        let Expr::Call { head, args } = form else {
            continue;
        };

        if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "SetDelayed"))
            && args.len() == 2
        {
            if let Some(target) = session.normalize_definition_assignment_target(&args[0]) {
                let symbol = match target {
                    DefinitionAssignmentTarget::Message(key) => key.symbol,
                    DefinitionAssignmentTarget::SyntaxInformation(symbol) => symbol,
                    DefinitionAssignmentTarget::Definition(target) => target.symbol().to_owned(),
                };
                insert_package_local(&mut bound, symbol, current_context);
            }
            continue;
        }

        if (is_system_symbol_name(head, "TagSet") || is_system_symbol_name(head, "TagSetDelayed"))
            && args.len() == 3
        {
            if let Some(symbol) = expr_as_symbol(&args[0]).copied() {
                insert_package_local(&mut bound, symbol, current_context);
            }
            continue;
        }

        if (is_system_symbol_name(head, "UpSet") || is_system_symbol_name(head, "UpSetDelayed"))
            && args.len() == 2
        {
            for symbol in collect_upset_target_symbols(&args[0]) {
                insert_package_local(&mut bound, symbol, current_context);
            }
            continue;
        }

        if (is_system_symbol_name(head, "SetAttributes")
            || is_system_symbol_name(head, "SetOptions"))
            && !args.is_empty()
        {
            collect_package_symbol_targets(&args[0], current_context, &mut bound);
        }
    }

    bound
}

fn collect_package_symbol_targets(
    expr: &Expr,
    current_context: &str,
    bound: &mut BTreeSet<Symbol>,
) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            insert_package_local(bound, *symbol, current_context);
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            for arg in args {
                collect_package_symbol_targets(arg, current_context, bound);
            }
        }
        _ => {}
    }
}

fn insert_package_local(bound: &mut BTreeSet<Symbol>, symbol: Symbol, current_context: &str) {
    if symbol.context() == current_context {
        bound.insert(symbol);
    }
}
