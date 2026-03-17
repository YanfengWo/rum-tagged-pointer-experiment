use std::collections::{BTreeMap, BTreeSet};

use rum_core::{Atom, BigInt, Expr, Symbol};
use rum_syntax::{
    BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxError, SyntaxExpr, parse_expr,
};
use thiserror::Error;

const REPLACE_REPEATED_LIMIT: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldAttribute {
    None,
    HoldFirst,
    HoldRest,
    HoldAll,
}

impl HoldAttribute {
    fn holds_argument(self, index: usize) -> bool {
        match self {
            Self::None => false,
            Self::HoldFirst => index == 0,
            Self::HoldRest => index > 0,
            Self::HoldAll => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub hold: HoldAttribute,
}

impl BuiltinSpec {
    pub const fn new(name: &'static str, summary: &'static str) -> Self {
        Self {
            name,
            summary,
            hold: HoldAttribute::None,
        }
    }

    pub const fn with_hold(name: &'static str, summary: &'static str, hold: HoldAttribute) -> Self {
        Self {
            name,
            summary,
            hold,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Session {
    builtins: BTreeMap<String, BuiltinSpec>,
    definitions: BTreeMap<Symbol, SymbolDefinition>,
    next_generated_symbol_id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalOutcome {
    pub value: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolInfo {
    pub qualified_name: String,
    pub summary: Option<String>,
    pub is_builtin: bool,
    pub has_own_value: bool,
    pub has_delayed_value: bool,
    pub has_down_values: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolDefinition {
    pub own_value: Option<Expr>,
    pub delayed_own_value: Option<Expr>,
    pub down_values: Vec<DownValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownValue {
    pub lhs: Expr,
    pub rhs: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RewriteRule {
    lhs: Expr,
    rhs: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CasesSpec {
    Pattern(Expr),
    Rule(RewriteRule),
}

#[derive(Clone, Copy, Debug)]
struct SequencePatternSpec<'a> {
    min_len: usize,
    head_constraint: Option<&'a Expr>,
    name: Option<&'a Symbol>,
    condition: Option<&'a Expr>,
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error("invalid integer literal `{literal}`")]
    InvalidInteger { literal: String },
    #[error("ReplaceRepeated exceeded iteration limit ({limit})")]
    IterationLimitExceeded { limit: usize },
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_builtin(&mut self, spec: BuiltinSpec) {
        self.builtins.insert(spec.name.to_string(), spec);
    }

    pub fn inspect(&self, name: &str) -> Option<SymbolInfo> {
        let symbol = symbol_from_raw_name(name);
        let builtin = self.builtins.get(symbol.name());
        let definition = self.definitions.get(&symbol);

        if builtin.is_none() && definition.is_none() {
            return None;
        }

        Some(SymbolInfo {
            qualified_name: symbol.to_string(),
            summary: builtin.map(|builtin| builtin.summary.to_string()),
            is_builtin: builtin.is_some(),
            has_own_value: definition.and_then(|def| def.own_value.as_ref()).is_some(),
            has_delayed_value: definition
                .and_then(|def| def.delayed_own_value.as_ref())
                .is_some(),
            has_down_values: definition
                .map(|definition| !definition.down_values.is_empty())
                .unwrap_or(false),
        })
    }

    pub fn eval_text(&mut self, input: &str) -> Result<EvalOutcome, EvalError> {
        let syntax = parse_expr(input)?;
        let expr = lower_syntax_expr(&syntax)?;
        let value = self.evaluate(expr)?;
        Ok(EvalOutcome { value })
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => self.evaluate_symbol(symbol),
            Expr::Call { head, args } => self.evaluate_call(*head, args),
            other => Ok(other),
        }
    }

    fn evaluate_symbol(&mut self, symbol: Symbol) -> Result<Expr, EvalError> {
        let definition = self.definitions.get(&symbol).cloned();

        match definition {
            Some(SymbolDefinition {
                own_value: Some(value),
                ..
            }) => self.evaluate(value),
            Some(SymbolDefinition {
                delayed_own_value: Some(value),
                ..
            }) => self.evaluate(value),
            _ => Ok(Expr::Atom(Atom::Symbol(symbol))),
        }
    }

    fn evaluate_call(&mut self, head: Expr, args: Vec<Expr>) -> Result<Expr, EvalError> {
        if let Some(result) = self.try_evaluate_special_form(&head, args.as_slice())? {
            return Ok(result);
        }

        let evaluated_head = self.evaluate(head)?;
        let evaluated_args = self.evaluate_call_args(&evaluated_head, args)?;

        if let Some(result) =
            self.try_apply_down_values(&evaluated_head, evaluated_args.as_slice())?
        {
            return Ok(result);
        }

        if let Some(result) =
            self.try_evaluate_non_held_builtin(&evaluated_head, evaluated_args.as_slice())?
        {
            return Ok(result);
        }

        if let Some(result) = self.try_apply_function(&evaluated_head, evaluated_args.as_slice())? {
            return Ok(result);
        }

        Ok(Expr::call(evaluated_head, evaluated_args))
    }

    fn evaluate_call_args(&mut self, head: &Expr, args: Vec<Expr>) -> Result<Vec<Expr>, EvalError> {
        let hold = self.hold_attribute_for_head(head);
        let mut evaluated_args = Vec::new();

        for (index, arg) in args.into_iter().enumerate() {
            let (evaluated_arg, allow_sequence_splice) = if hold.holds_argument(index) {
                self.evaluate_held_argument(arg)?
            } else {
                (self.evaluate(arg)?, true)
            };

            if allow_sequence_splice {
                splice_sequence_argument(&mut evaluated_args, evaluated_arg);
            } else {
                evaluated_args.push(evaluated_arg);
            }
        }

        Ok(evaluated_args)
    }

    fn evaluate_held_argument(&mut self, arg: Expr) -> Result<(Expr, bool), EvalError> {
        if let Some(inner) = evaluate_wrapper_arg(&arg) {
            return self.evaluate(inner.clone()).map(|value| (value, true));
        }

        Ok((arg, false))
    }

    fn hold_attribute_for_head(&self, head: &Expr) -> HoldAttribute {
        let Some(symbol) = expr_as_symbol(head) else {
            return HoldAttribute::None;
        };

        self.builtins
            .get(symbol.name())
            .map(|spec| spec.hold)
            .unwrap_or_else(|| fallback_hold_attribute(symbol.name()))
    }

    fn try_evaluate_special_form(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        match symbol.name() {
            "Set" => self.eval_set(args).map(Some),
            "SetDelayed" => self.eval_set_delayed(args).map(Some),
            "Rule" => self.eval_rule(args).map(Some),
            "RuleDelayed" => self.eval_rule_delayed(args).map(Some),
            "MatchQ" => self.eval_match_q(args).map(Some),
            "Cases" => self.eval_cases(args).map(Some),
            "Module" => self.eval_module(args).map(Some),
            "ReplaceAll" => self.eval_replace_all(args).map(Some),
            "ReplaceRepeated" => self.eval_replace_repeated(args).map(Some),
            _ => Ok(None),
        }
    }

    fn try_evaluate_non_held_builtin(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        match symbol.name() {
            "Identity" if args.len() == 1 => Ok(Some(args[0].clone())),
            "If" => self.eval_if(args).map(Some),
            "Which" => self.eval_which(args).map(Some),
            "With" => self.eval_with(args).map(Some),
            "Hold" | "HoldFirst" | "HoldRest" | "HoldAll" => {
                Ok(Some(Expr::call(head.clone(), args.to_vec())))
            }
            "Evaluate" if args.len() == 1 => self.evaluate(args[0].clone()).map(Some),
            "Sequence" => Ok(Some(sequence_expr(args.to_vec()))),
            _ => Ok(None),
        }
    }

    fn eval_if(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(Expr::call(Expr::symbol("If"), args.to_vec()));
        }

        match truthiness_symbol(&args[0]) {
            Some(true) => self.evaluate(args[1].clone()),
            Some(false) => {
                if args.len() >= 3 {
                    self.evaluate(args[2].clone())
                } else {
                    Ok(Expr::symbol("Null"))
                }
            }
            None if args.len() == 4 => self.evaluate(args[3].clone()),
            None => Ok(Expr::call(Expr::symbol("If"), args.to_vec())),
        }
    }

    fn eval_which(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() || args.len() % 2 != 0 {
            return Ok(Expr::call(Expr::symbol("Which"), args.to_vec()));
        }

        let mut index = 0;

        while index < args.len() {
            let condition = self.evaluate(args[index].clone())?;

            match truthiness_symbol(&condition) {
                Some(true) => return self.evaluate(args[index + 1].clone()),
                Some(false) => index += 2,
                None => {
                    let mut residual = args.to_vec();
                    residual[index] = condition;
                    return Ok(Expr::call(Expr::symbol("Which"), residual));
                }
            }
        }

        Ok(Expr::symbol("Null"))
    }

    fn eval_with(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("With"), args.to_vec()));
        }

        let Some(bindings) = self.evaluate_with_bindings(&args[0])? else {
            return Ok(Expr::call(Expr::symbol("With"), args.to_vec()));
        };

        let body = substitute_lexical_bindings(&args[1], &bindings, &BTreeSet::new());
        self.evaluate(body)
    }

    fn eval_module(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("Module"), args.to_vec()));
        }

        let Some(items) = parse_module_binding_list(&args[0]) else {
            return Ok(Expr::call(Expr::symbol("Module"), args.to_vec()));
        };

        let mut renames = BTreeMap::new();

        for (symbol, _) in &items {
            renames.insert(
                symbol.clone(),
                Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))),
            );
        }

        for (symbol, init) in items {
            let Some(fresh_expr) = renames.get(&symbol) else {
                continue;
            };
            let Some(fresh_symbol) = expr_as_symbol(fresh_expr).cloned() else {
                continue;
            };

            let definition = self.definitions.entry(fresh_symbol.clone()).or_default();
            definition.own_value = None;
            definition.delayed_own_value = None;

            if let Some(init) = init {
                let renamed_init = substitute_lexical_bindings(&init, &renames, &BTreeSet::new());
                let value = self.evaluate(renamed_init)?;
                let definition = self.definitions.entry(fresh_symbol).or_default();
                definition.own_value = Some(value);
                definition.delayed_own_value = None;
            }
        }

        let body = substitute_lexical_bindings(&args[1], &renames, &BTreeSet::new());
        self.evaluate(body)
    }

    fn try_apply_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(body) = function_body(head) else {
            return Ok(None);
        };

        let applied = substitute_slots(body, args);
        self.evaluate(applied).map(Some)
    }

    fn evaluate_with_bindings(
        &mut self,
        expr: &Expr,
    ) -> Result<Option<BTreeMap<Symbol, Expr>>, EvalError> {
        let Expr::Call { head, args } = expr else {
            return Ok(None);
        };

        if !is_system_symbol_name(head, "List") {
            return Ok(None);
        }

        let mut bindings = BTreeMap::new();

        for item in args {
            let Some((symbol, rhs)) = parse_with_binding_item(item) else {
                return Ok(None);
            };

            let substituted_rhs = substitute_lexical_bindings(&rhs, &bindings, &BTreeSet::new());
            let value = self.evaluate(substituted_rhs)?;
            bindings.insert(symbol, value);
        }

        Ok(Some(bindings))
    }

    fn allocate_fresh_symbol(&mut self, symbol: &Symbol) -> Symbol {
        self.next_generated_symbol_id += 1;
        Symbol::new(
            symbol.context().to_string(),
            format!("{}${}", symbol.name(), self.next_generated_symbol_id),
        )
    }

    fn eval_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("Set"), args.to_vec()));
        }

        let lhs = args[0].clone();
        let rhs = self.evaluate(args[1].clone())?;

        if let Some(symbol) = expr_as_symbol(&lhs).cloned() {
            let definition = self.definitions.entry(symbol).or_default();
            definition.own_value = Some(rhs.clone());
            definition.delayed_own_value = None;
            Ok(rhs)
        } else {
            Ok(Expr::call(Expr::symbol("Set"), vec![lhs, rhs]))
        }
    }

    fn eval_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("SetDelayed"), args.to_vec()));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();

        if let Some(symbol) = expr_as_symbol(&lhs).cloned() {
            let definition = self.definitions.entry(symbol).or_default();
            definition.own_value = None;
            definition.delayed_own_value = Some(rhs);
            Ok(Expr::symbol("Null"))
        } else if let Some(head_symbol) = downvalue_head_symbol(&lhs).cloned() {
            let definition = self.definitions.entry(head_symbol).or_default();
            definition.down_values.push(DownValue { lhs, rhs });
            Ok(Expr::symbol("Null"))
        } else {
            Ok(Expr::call(Expr::symbol("SetDelayed"), vec![lhs, rhs]))
        }
    }

    fn eval_rule(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("Rule"), args.to_vec()));
        }

        Ok(Expr::call(
            Expr::symbol("Rule"),
            vec![args[0].clone(), self.evaluate(args[1].clone())?],
        ))
    }

    fn eval_rule_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("RuleDelayed"), args.to_vec()));
        }

        Ok(Expr::call(Expr::symbol("RuleDelayed"), args.to_vec()))
    }

    fn eval_match_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("MatchQ"), args.to_vec()));
        }

        let value = self.evaluate(args[0].clone())?;
        let pattern = self.normalize_pattern_expr(&args[1])?;
        let mut bindings = BTreeMap::new();

        Ok(Expr::symbol(
            if self.match_expr(&pattern, &value, &mut bindings)? {
                "True"
            } else {
                "False"
            },
        ))
    }

    fn eval_cases(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("Cases"), args.to_vec()));
        }

        let target = self.evaluate(args[0].clone())?;
        let spec = self.normalize_cases_spec(&args[1])?;
        let mut matches = Vec::new();

        self.collect_cases(&target, &spec, &mut matches)?;

        Ok(Expr::call(Expr::symbol("List"), matches))
    }

    fn eval_replace_all(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("ReplaceAll"), args.to_vec()));
        }

        let target = self.evaluate(args[0].clone())?;
        let rules_expr = self.evaluate(args[1].clone())?;

        let Some(rules) = normalize_rewrite_rules(&rules_expr) else {
            return Ok(Expr::call(
                Expr::symbol("ReplaceAll"),
                vec![target, rules_expr],
            ));
        };

        let replaced = self.apply_replace_all(target, &rules)?;
        self.evaluate(replaced)
    }

    fn eval_replace_repeated(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("ReplaceRepeated"), args.to_vec()));
        }

        let mut current = self.evaluate(args[0].clone())?;
        let rules_expr = self.evaluate(args[1].clone())?;

        let Some(rules) = normalize_rewrite_rules(&rules_expr) else {
            return Ok(Expr::call(
                Expr::symbol("ReplaceRepeated"),
                vec![current, rules_expr],
            ));
        };

        for _ in 0..REPLACE_REPEATED_LIMIT {
            let replaced = self.apply_replace_all(current.clone(), &rules)?;
            let next = self.evaluate(replaced)?;

            if next == current {
                return Ok(next);
            }

            current = next;
        }

        Err(EvalError::IterationLimitExceeded {
            limit: REPLACE_REPEATED_LIMIT,
        })
    }

    fn try_apply_down_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        let Some(definition) = self.definitions.get(symbol).cloned() else {
            return Ok(None);
        };

        let candidate = Expr::call(head.clone(), args.to_vec());

        for down_value in definition.down_values {
            let mut bindings = BTreeMap::new();

            if self.match_expr(&down_value.lhs, &candidate, &mut bindings)? {
                let substituted = substitute_rule_bindings(&down_value.rhs, &bindings);
                return self.evaluate(substituted).map(Some);
            }
        }

        Ok(None)
    }

    fn match_expr(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        let snapshot = bindings.clone();
        let matched = self.match_expr_inner(pattern, value, bindings)?;

        if !matched {
            *bindings = snapshot;
        }

        Ok(matched)
    }

    fn match_expr_inner(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if let Some((name, inner)) = pattern_named_pattern(pattern) {
            if !self.match_expr(inner, value, bindings)? {
                return Ok(false);
            }

            return Ok(bind_pattern_name(name, value.clone(), bindings));
        }

        if let Some((inner, condition)) = pattern_condition(pattern) {
            if !self.match_expr(inner, value, bindings)? {
                return Ok(false);
            }

            let condition_expr = substitute_bindings(condition, bindings);
            let result = self.evaluate(condition_expr)?;
            return Ok(is_system_symbol_name(&result, "True"));
        }

        if let Some(blank_head) = pattern_blank_head(pattern) {
            return Ok(match_blank(blank_head, value));
        }

        match (pattern, value) {
            (Expr::Atom(lhs), Expr::Atom(rhs)) => Ok(lhs == rhs),
            (
                Expr::Call {
                    head: lhs_head,
                    args: lhs_args,
                },
                Expr::Call {
                    head: rhs_head,
                    args: rhs_args,
                },
            ) => {
                if !self.match_expr(lhs_head, rhs_head, bindings)? {
                    return Ok(false);
                }

                self.match_call_args(lhs_args, rhs_args, bindings)
            }
            _ => Ok(false),
        }
    }

    fn match_call_args(
        &mut self,
        pattern_args: &[Expr],
        value_args: &[Expr],
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if pattern_args.is_empty() {
            return Ok(value_args.is_empty());
        }

        if let Some(spec) = decompose_sequence_pattern(&pattern_args[0]) {
            let rest = &pattern_args[1..];
            let min_rest = min_required_arg_count(rest);

            if value_args.len() < spec.min_len + min_rest {
                return Ok(false);
            }

            let max_len = value_args.len() - min_rest;

            for consume in spec.min_len..=max_len {
                let snapshot = bindings.clone();

                if self.match_sequence_pattern(spec, &value_args[..consume], bindings)?
                    && self.match_call_args(rest, &value_args[consume..], bindings)?
                {
                    return Ok(true);
                }

                *bindings = snapshot;
            }

            return Ok(false);
        }

        let Some((value, rest)) = value_args.split_first() else {
            return Ok(false);
        };

        if !self.match_expr(&pattern_args[0], value, bindings)? {
            return Ok(false);
        }

        self.match_call_args(&pattern_args[1..], rest, bindings)
    }

    fn match_sequence_pattern(
        &mut self,
        spec: SequencePatternSpec<'_>,
        values: &[Expr],
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if values.len() < spec.min_len {
            return Ok(false);
        }

        if !values
            .iter()
            .all(|value| match_blank(spec.head_constraint, value))
        {
            return Ok(false);
        }

        if let Some(name) = spec.name {
            let bound_value = sequence_expr(values.to_vec());

            if !bind_pattern_name(name, bound_value, bindings) {
                return Ok(false);
            }
        }

        if let Some(condition) = spec.condition {
            let condition_expr = substitute_bindings(condition, bindings);
            let result = self.evaluate(condition_expr)?;
            return Ok(is_system_symbol_name(&result, "True"));
        }

        Ok(true)
    }

    fn normalize_pattern_expr(&mut self, expr: &Expr) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(expr.clone()),
            _ => Ok(expr.clone()),
        }
    }

    fn normalize_cases_spec(&mut self, expr: &Expr) -> Result<CasesSpec, EvalError> {
        let resolved = match expr {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(expr.clone())?,
            Expr::Call { head, .. }
                if is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed") =>
            {
                self.evaluate(expr.clone())?
            }
            _ => expr.clone(),
        };

        Ok(match normalize_single_rewrite_rule(&resolved) {
            Some(rule) => CasesSpec::Rule(rule),
            None => CasesSpec::Pattern(resolved),
        })
    }

    fn collect_cases(
        &mut self,
        expr: &Expr,
        spec: &CasesSpec,
        matches: &mut Vec<Expr>,
    ) -> Result<(), EvalError> {
        if let Some(case) = self.try_collect_case(expr, spec)? {
            matches.push(case);
        }

        if let Expr::Call { args, .. } = expr {
            for arg in args {
                self.collect_cases(arg, spec, matches)?;
            }
        }

        Ok(())
    }

    fn try_collect_case(
        &mut self,
        expr: &Expr,
        spec: &CasesSpec,
    ) -> Result<Option<Expr>, EvalError> {
        let mut bindings = BTreeMap::new();

        match spec {
            CasesSpec::Pattern(pattern) => {
                if self.match_expr(pattern, expr, &mut bindings)? {
                    Ok(Some(expr.clone()))
                } else {
                    Ok(None)
                }
            }
            CasesSpec::Rule(rule) => {
                if self.match_expr(&rule.lhs, expr, &mut bindings)? {
                    let transformed = substitute_rule_bindings(&rule.rhs, &bindings);
                    self.evaluate(transformed).map(Some)
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn apply_replace_all(&mut self, expr: Expr, rules: &[RewriteRule]) -> Result<Expr, EvalError> {
        if let Some(replaced) = self.apply_rules_once(&expr, rules)? {
            return Ok(replaced);
        }

        match expr {
            Expr::Atom(_) => Ok(expr),
            Expr::Call { head, args } => Ok(Expr::call(
                self.apply_replace_all(*head, rules)?,
                args.into_iter()
                    .map(|arg| self.apply_replace_all(arg, rules))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        }
    }

    fn apply_rules_once(
        &mut self,
        expr: &Expr,
        rules: &[RewriteRule],
    ) -> Result<Option<Expr>, EvalError> {
        for rule in rules {
            let mut bindings = BTreeMap::new();

            if self.match_expr(&rule.lhs, expr, &mut bindings)? {
                return Ok(Some(substitute_rule_bindings(&rule.rhs, &bindings)));
            }
        }

        Ok(None)
    }
}

pub fn lower_syntax_expr(expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    match expr {
        SyntaxExpr::Integer(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            Ok(Expr::integer(bigint))
        }
        SyntaxExpr::String(value) => Ok(Expr::string(value.clone())),
        SyntaxExpr::Symbol(value) => Ok(lower_symbol(value)),
        SyntaxExpr::List(items) => {
            let lowered_items = items
                .iter()
                .map(lower_syntax_expr)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::call(Expr::symbol("List"), lowered_items))
        }
        SyntaxExpr::Slot(spec) => lower_slot(spec),
        SyntaxExpr::Blank { kind, head } => lower_blank(*kind, head.as_deref()),
        SyntaxExpr::Pattern { name, pattern } => Ok(Expr::call(
            Expr::symbol("Pattern"),
            vec![lower_syntax_expr(name)?, lower_syntax_expr(pattern)?],
        )),
        SyntaxExpr::Prefix { op, expr } => lower_prefix(*op, expr),
        SyntaxExpr::Binary { op, lhs, rhs } => lower_binary(*op, lhs, rhs),
        SyntaxExpr::Postfix { op, expr } => lower_postfix(*op, expr),
        SyntaxExpr::Call { head, args } => {
            let lowered_head = lower_syntax_expr(head)?;
            let lowered_args = args
                .iter()
                .map(lower_syntax_expr)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::call(lowered_head, lowered_args))
        }
    }
}

fn lower_slot(spec: &SlotSpec) -> Result<Expr, EvalError> {
    let arg = match spec {
        SlotSpec::Anonymous => Expr::integer(BigInt::from(1_u8)),
        SlotSpec::Number(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            Expr::integer(bigint)
        }
        SlotSpec::Named(value) => Expr::string(value.clone()),
    };

    Ok(Expr::call(Expr::symbol("Slot"), vec![arg]))
}

fn lower_blank(kind: BlankKind, head: Option<&SyntaxExpr>) -> Result<Expr, EvalError> {
    let head_symbol = match kind {
        BlankKind::Blank => "Blank",
        BlankKind::BlankSequence => "BlankSequence",
        BlankKind::BlankNullSequence => "BlankNullSequence",
    };

    let args = match head {
        Some(head) => vec![lower_syntax_expr(head)?],
        None => Vec::new(),
    };

    Ok(Expr::call(Expr::symbol(head_symbol), args))
}

fn lower_prefix(op: PrefixOp, expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    let lowered = lower_syntax_expr(expr)?;

    match op {
        PrefixOp::Plus => Ok(lowered),
        PrefixOp::Minus => Ok(Expr::call(
            Expr::symbol("Times"),
            vec![Expr::integer(BigInt::from(-1)), lowered],
        )),
    }
}

fn lower_binary(op: BinaryOp, lhs: &SyntaxExpr, rhs: &SyntaxExpr) -> Result<Expr, EvalError> {
    let lowered_lhs = lower_syntax_expr(lhs)?;
    let lowered_rhs = lower_syntax_expr(rhs)?;

    match op {
        BinaryOp::Set => Ok(Expr::call(
            Expr::symbol("Set"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::SetDelayed => Ok(Expr::call(
            Expr::symbol("SetDelayed"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceAll => Ok(Expr::call(
            Expr::symbol("ReplaceAll"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceRepeated => Ok(Expr::call(
            Expr::symbol("ReplaceRepeated"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Rule => Ok(Expr::call(
            Expr::symbol("Rule"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::RuleDelayed => Ok(Expr::call(
            Expr::symbol("RuleDelayed"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Condition => Ok(Expr::call(
            Expr::symbol("Condition"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Add => Ok(Expr::call(
            Expr::symbol("Plus"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Subtract => Ok(Expr::call(
            Expr::symbol("Plus"),
            vec![
                lowered_lhs,
                Expr::call(
                    Expr::symbol("Times"),
                    vec![Expr::integer(BigInt::from(-1)), lowered_rhs],
                ),
            ],
        )),
        BinaryOp::Times => Ok(Expr::call(
            Expr::symbol("Times"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Divide => Ok(Expr::call(
            Expr::symbol("Times"),
            vec![
                lowered_lhs,
                Expr::call(
                    Expr::symbol("Power"),
                    vec![lowered_rhs, Expr::integer(BigInt::from(-1))],
                ),
            ],
        )),
        BinaryOp::Power => Ok(Expr::call(
            Expr::symbol("Power"),
            vec![lowered_lhs, lowered_rhs],
        )),
    }
}

fn lower_postfix(op: PostfixOp, expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    let lowered = lower_syntax_expr(expr)?;

    match op {
        PostfixOp::Function => Ok(Expr::call(Expr::symbol("Function"), vec![lowered])),
    }
}

fn lower_symbol(raw: &str) -> Expr {
    Expr::Atom(Atom::Symbol(symbol_from_raw_name(raw)))
}

fn symbol_from_raw_name(raw: &str) -> Symbol {
    if let Some(split_at) = raw.rfind('`') {
        let context = &raw[..=split_at];
        let name = &raw[split_at + 1..];
        Symbol::new(context, name)
    } else {
        Symbol::from_name(raw)
    }
}

fn expr_as_symbol(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol),
        _ => None,
    }
}

fn truthiness_symbol(expr: &Expr) -> Option<bool> {
    match expr_as_symbol(expr).map(Symbol::name) {
        Some("True") => Some(true),
        Some("False") => Some(false),
        _ => None,
    }
}

fn fallback_hold_attribute(name: &str) -> HoldAttribute {
    match name {
        "Hold" | "HoldAll" | "Evaluate" => HoldAttribute::HoldAll,
        "Function" | "Which" | "Module" => HoldAttribute::HoldAll,
        "If" => HoldAttribute::HoldRest,
        "With" => HoldAttribute::HoldAll,
        "HoldFirst" => HoldAttribute::HoldFirst,
        "HoldRest" => HoldAttribute::HoldRest,
        _ => HoldAttribute::None,
    }
}

fn evaluate_wrapper_arg(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Evaluate") && args.len() == 1 => {
            Some(&args[0])
        }
        _ => None,
    }
}

fn splice_sequence_argument(args: &mut Vec<Expr>, expr: Expr) {
    if let Some(items) = sequence_items(&expr) {
        args.extend(items.iter().cloned());
    } else {
        args.push(expr);
    }
}

fn function_body(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") && args.len() == 1 => {
            Some(&args[0])
        }
        _ => None,
    }
}

fn parse_with_binding_item(expr: &Expr) -> Option<(Symbol, Expr)> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2 =>
        {
            Some((expr_as_symbol(&args[0])?.clone(), args[1].clone()))
        }
        _ => None,
    }
}

fn parse_module_binding_list(expr: &Expr) -> Option<Vec<(Symbol, Option<Expr>)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(parse_module_binding_item).collect()
}

fn parse_module_binding_item(expr: &Expr) -> Option<(Symbol, Option<Expr>)> {
    if let Some(symbol) = expr_as_symbol(expr) {
        return Some((symbol.clone(), None));
    }

    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2 =>
        {
            Some((expr_as_symbol(&args[0])?.clone(), Some(args[1].clone())))
        }
        _ => None,
    }
}

fn collect_with_binding_symbols(expr: &Expr, locals: &mut BTreeSet<Symbol>) {
    let Expr::Call { head, args } = expr else {
        return;
    };

    if !is_system_symbol_name(head, "List") {
        return;
    }

    for item in args {
        if let Some((symbol, _)) = parse_module_binding_item(item) {
            locals.insert(symbol);
        }
    }
}

fn downvalue_head_symbol(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && !args.is_empty() =>
        {
            downvalue_head_symbol(&args[0])
        }
        Expr::Call { head, .. } => expr_as_symbol(head),
        _ => None,
    }
}

fn normalize_rewrite_rules(expr: &Expr) -> Option<Vec<RewriteRule>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut rules = Vec::new();

            for item in args {
                let rule = normalize_single_rewrite_rule(item)?;
                rules.push(rule);
            }

            Some(rules)
        }
        _ => normalize_single_rewrite_rule(expr).map(|rule| vec![rule]),
    }
}

fn normalize_single_rewrite_rule(expr: &Expr) -> Option<RewriteRule> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            Some(RewriteRule {
                lhs: args[0].clone(),
                rhs: args[1].clone(),
            })
        }
        _ => None,
    }
}

fn pattern_named_pattern(expr: &Expr) -> Option<(&Symbol, &Expr)> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            let name = expr_as_symbol(&args[0])?;
            Some((name, &args[1]))
        }
        _ => None,
    }
}

fn pattern_condition(expr: &Expr) -> Option<(&Expr, &Expr)> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            Some((&args[0], &args[1]))
        }
        _ => None,
    }
}

fn decompose_sequence_pattern(expr: &Expr) -> Option<SequencePatternSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            let mut spec = decompose_sequence_pattern(&args[1])?;
            let name = expr_as_symbol(&args[0])?;

            if spec.name.is_some() {
                return None;
            }

            spec.name = Some(name);
            Some(spec)
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            let mut spec = decompose_sequence_pattern(&args[0])?;

            if spec.condition.is_some() {
                return None;
            }

            spec.condition = Some(&args[1]);
            Some(spec)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "BlankSequence") => {
            if args.len() > 1 {
                return None;
            }

            Some(SequencePatternSpec {
                min_len: 1,
                head_constraint: args.first(),
                name: None,
                condition: None,
            })
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "BlankNullSequence") => {
            if args.len() > 1 {
                return None;
            }

            Some(SequencePatternSpec {
                min_len: 0,
                head_constraint: args.first(),
                name: None,
                condition: None,
            })
        }
        _ => None,
    }
}

fn min_required_arg_count(patterns: &[Expr]) -> usize {
    patterns
        .iter()
        .map(|pattern| {
            decompose_sequence_pattern(pattern)
                .map(|spec| spec.min_len)
                .unwrap_or(1)
        })
        .sum()
}

fn pattern_blank_head(expr: &Expr) -> Option<Option<&Expr>> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Blank")
                || is_system_symbol_name(head, "BlankSequence")
                || is_system_symbol_name(head, "BlankNullSequence") =>
        {
            if args.is_empty() {
                Some(None)
            } else if args.len() == 1 {
                Some(Some(&args[0]))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn match_blank(head_constraint: Option<&Expr>, value: &Expr) -> bool {
    match head_constraint {
        None => true,
        Some(head_constraint) => expr_has_head(value, head_constraint),
    }
}

fn expr_has_head(value: &Expr, head_constraint: &Expr) -> bool {
    let Some(expected_symbol) = expr_as_symbol(head_constraint) else {
        return false;
    };

    match value {
        Expr::Atom(Atom::Integer(_)) => expected_symbol.name() == "Integer",
        Expr::Atom(Atom::String(_)) => expected_symbol.name() == "String",
        Expr::Atom(Atom::Symbol(_)) => expected_symbol.name() == "Symbol",
        Expr::Call { head, .. } => expr_as_symbol(head)
            .map(|actual| actual == expected_symbol)
            .unwrap_or(false),
    }
}

fn bind_pattern_name(name: &Symbol, value: Expr, bindings: &mut BTreeMap<Symbol, Expr>) -> bool {
    match bindings.get(name) {
        Some(existing) => existing == &value,
        None => {
            bindings.insert(name.clone(), value);
            true
        }
    }
}

fn sequence_expr(values: Vec<Expr>) -> Expr {
    Expr::call(Expr::symbol("Sequence"), values)
}

fn substitute_bindings(expr: &Expr, bindings: &BTreeMap<Symbol, Expr>) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => Expr::call(
            substitute_bindings(head, bindings),
            args.iter()
                .map(|arg| substitute_bindings(arg, bindings))
                .collect(),
        ),
    }
}

fn substitute_rule_bindings(expr: &Expr, bindings: &BTreeMap<Symbol, Expr>) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => {
            let substituted_head = substitute_rule_bindings(head, bindings);
            let mut substituted_args = Vec::new();

            for arg in args {
                let substituted_arg = substitute_rule_bindings(arg, bindings);

                if let Some(sequence_items) = sequence_items(&substituted_arg) {
                    substituted_args.extend(sequence_items.iter().cloned());
                } else {
                    substituted_args.push(substituted_arg);
                }
            }

            Expr::call(substituted_head, substituted_args)
        }
    }
}

fn substitute_lexical_bindings(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            if shadowed.contains(symbol) {
                Expr::Atom(Atom::Symbol(symbol.clone()))
            } else {
                bindings
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone())))
            }
        }
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            Expr::call(
                substitute_lexical_bindings(head, bindings, shadowed),
                vec![
                    args[0].clone(),
                    substitute_lexical_bindings(&args[1], bindings, shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed")
                || is_system_symbol_name(head, "SetDelayed"))
                && args.len() == 2 =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_lhs = substitute_lexical_bindings(&args[0], bindings, shadowed);
            let mut rhs_shadowed = shadowed.clone();
            collect_pattern_locals(&substituted_lhs, &mut rhs_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_lhs,
                    substitute_lexical_bindings(&args[1], bindings, &rhs_shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_lhs = substitute_lexical_bindings(&args[0], bindings, shadowed);
            let mut condition_shadowed = shadowed.clone();
            collect_pattern_locals(&substituted_lhs, &mut condition_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_lhs,
                    substitute_lexical_bindings(&args[1], bindings, &condition_shadowed),
                ],
            )
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_bindings = substitute_with_binding_list(&args[0], bindings, shadowed);
            let mut body_shadowed = shadowed.clone();
            collect_with_binding_symbols(&args[0], &mut body_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_bindings,
                    substitute_lexical_bindings(&args[1], bindings, &body_shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Module") || is_system_symbol_name(head, "Block"))
                && !args.is_empty() =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_bindings = substitute_with_binding_list(&args[0], bindings, shadowed);
            let mut body_shadowed = shadowed.clone();
            collect_with_binding_symbols(&args[0], &mut body_shadowed);
            let mut substituted_args = vec![substituted_bindings];

            substituted_args.extend(
                args[1..]
                    .iter()
                    .map(|arg| substitute_lexical_bindings(arg, bindings, &body_shadowed)),
            );

            Expr::call(substituted_head, substituted_args)
        }
        Expr::Call { head, args } => Expr::call(
            substitute_lexical_bindings(head, bindings, shadowed),
            args.iter()
                .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed))
                .collect(),
        ),
    }
}

fn substitute_with_binding_list(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let Expr::Call { head, args } = expr else {
        return substitute_lexical_bindings(expr, bindings, shadowed);
    };

    if !is_system_symbol_name(head, "List") {
        return substitute_lexical_bindings(expr, bindings, shadowed);
    }

    Expr::call(
        substitute_lexical_bindings(head, bindings, shadowed),
        args.iter()
            .map(|item| substitute_with_binding_item(item, bindings, shadowed))
            .collect(),
    )
}

fn substitute_with_binding_item(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(_)) => expr.clone(),
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2
                && expr_as_symbol(&args[0]).is_some() =>
        {
            Expr::call(
                substitute_lexical_bindings(head, bindings, shadowed),
                vec![
                    args[0].clone(),
                    substitute_lexical_bindings(&args[1], bindings, shadowed),
                ],
            )
        }
        _ => substitute_lexical_bindings(expr, bindings, shadowed),
    }
}

fn collect_pattern_locals(expr: &Expr, locals: &mut BTreeSet<Symbol>) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            if let Some(symbol) = expr_as_symbol(&args[0]) {
                locals.insert(symbol.clone());
            }
            collect_pattern_locals(&args[1], locals);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_pattern_locals(arg, locals);
            }
        }
        Expr::Atom(_) => {}
    }
}

fn substitute_slots(expr: &Expr, args: &[Expr]) -> Expr {
    if let Some(substituted) = substitute_single_slot(expr, args) {
        return substituted;
    }

    match expr {
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call {
            head,
            args: call_args,
        } => Expr::call(
            substitute_slots(head, args),
            call_args
                .iter()
                .map(|arg| substitute_slots(arg, args))
                .collect(),
        ),
    }
}

fn substitute_single_slot(expr: &Expr, args: &[Expr]) -> Option<Expr> {
    let Expr::Call {
        head,
        args: slot_args,
    } = expr
    else {
        return None;
    };

    if !is_system_symbol_name(head, "Slot") || slot_args.len() != 1 {
        return None;
    }

    match &slot_args[0] {
        Expr::Atom(Atom::Integer(index)) => {
            let index = index.to_string().parse::<usize>().ok()?;
            if index == 0 {
                None
            } else {
                args.get(index - 1).cloned()
            }
        }
        _ => None,
    }
}

fn sequence_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Sequence") => Some(args),
        _ => None,
    }
}

fn is_system_symbol_name(expr: &Expr, name: &str) -> bool {
    expr_as_symbol(expr)
        .map(|symbol| symbol.name() == name)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_big_integer_into_core_expr() {
        let lowered = lower_syntax_expr(&SyntaxExpr::Integer(
            "123456789012345678901234567890".to_string(),
        ))
        .unwrap();

        assert_eq!(lowered.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn evaluates_identity_call() {
        let mut session = Session::new();
        let outcome = session
            .eval_text("Identity[123456789012345678901234567890]")
            .unwrap();

        assert_eq!(outcome.value.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn hold_family_respects_builtin_hold_attributes() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Hold[Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[Identity[1], Identity[2]]"
        );
        assert_eq!(
            session
                .eval_text("HoldFirst[Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "HoldFirst[Identity[1], 2]"
        );
        assert_eq!(
            session
                .eval_text("HoldRest[Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "HoldRest[1, Identity[2]]"
        );
        assert_eq!(
            session
                .eval_text("HoldAll[Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "HoldAll[Identity[1], Identity[2]]"
        );
    }

    #[test]
    fn evaluate_forces_evaluation_inside_held_arguments() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Evaluate[Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("Hold[Evaluate[Identity[1]]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[1]"
        );
    }

    #[test]
    fn sequence_splices_in_non_held_calls_but_not_in_raw_held_ones() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[Sequence[1, 2], 3]")
                .unwrap()
                .value
                .to_string(),
            "f[1, 2, 3]"
        );
        assert_eq!(
            session
                .eval_text("Sequence[1, Sequence[2, 3]]")
                .unwrap()
                .value
                .to_string(),
            "Sequence[1, 2, 3]"
        );
        assert_eq!(
            session
                .eval_text("Hold[Sequence[1, 2], 3]")
                .unwrap()
                .value
                .to_string(),
            "Hold[Sequence[1, 2], 3]"
        );
        assert_eq!(
            session
                .eval_text("Hold[Evaluate[Sequence[1, 2]], 3]")
                .unwrap()
                .value
                .to_string(),
            "Hold[1, 2, 3]"
        );
    }

    #[test]
    fn if_evaluates_only_the_selected_branch() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("If[True, Identity[1], x]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("If[False, x, Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
        assert_eq!(
            session.eval_text("If[False, x]").unwrap().value.to_string(),
            "Null"
        );
        assert_eq!(
            session
                .eval_text("If[maybe, Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "If[maybe, Identity[1], Identity[2]]"
        );
        assert_eq!(
            session
                .eval_text("If[maybe, x, y, Identity[3]]")
                .unwrap()
                .value
                .to_string(),
            "3"
        );
    }

    #[test]
    fn which_evaluates_conditions_in_order() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Which[False, x, True, Identity[2], True, Identity[3]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
        assert_eq!(
            session
                .eval_text("Which[False, x, False, y]")
                .unwrap()
                .value
                .to_string(),
            "Null"
        );
        assert_eq!(
            session
                .eval_text("Which[maybe, Identity[1], True, Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "Which[maybe, Identity[1], True, Identity[2]]"
        );
    }

    #[test]
    fn function_applies_slots_positionally() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Function[Slot[1]][Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("Function[Plus[Slot[1], Slot[2]]][1, 2]")
                .unwrap()
                .value
                .to_string(),
            "Plus[1, 2]"
        );
    }

    #[test]
    fn pure_function_syntax_uses_slots_and_evaluated_arguments() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("(#1 &)[Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("(#2 &)[10, Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
    }

    #[test]
    fn with_evaluates_bindings_and_substitutes_lexically() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("With[{x = Identity[1], y = x}, y]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("With[{x = 1}, Hold[x]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[1]"
        );
    }

    #[test]
    fn with_supports_rule_style_bindings_and_nested_shadowing() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("With[{x -> 1}, x]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("With[{x = 1}, With[{x = 2}, x]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
    }

    #[test]
    fn with_respects_pattern_local_names_but_substitutes_pattern_metadata() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("With[{x = 1}, 7 /. x_Integer :> x]")
                .unwrap()
                .value
                .to_string(),
            "7"
        );
        assert_eq!(
            session
                .eval_text("With[{t = Integer}, MatchQ[7, x_t]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
    }

    #[test]
    fn module_generates_fresh_lexical_symbols_and_initializes_sequentially() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Module[{x = 1, y = x}, y]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );

        let first = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;
        let second = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;

        let first_text = first.to_string();
        let second_text = second.to_string();

        assert!(first_text.starts_with("Hold[x$"));
        assert!(second_text.starts_with("Hold[x$"));
        assert_ne!(first_text, second_text);
    }

    #[test]
    fn module_respects_nested_shadowing_and_pattern_locals() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Module[{x = 1}, Module[{x = 2}, x]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
        let nested = session
            .eval_text("Module[{x = 1}, Module[{x}, Hold[x]]]")
            .unwrap()
            .value
            .to_string();
        assert!(nested.starts_with("Hold[x$"));
        assert_ne!(nested, "Hold[x]");
        assert_ne!(nested, "Hold[1]");
        assert_eq!(
            session
                .eval_text("Module[{x = 1}, 7 /. x_Integer :> x]")
                .unwrap()
                .value
                .to_string(),
            "7"
        );
    }

    #[test]
    fn set_assigns_simple_symbol_and_lookup_works() {
        let mut session = Session::new();

        let assigned = session.eval_text("x = 42").unwrap();
        assert_eq!(assigned.value.to_string(), "42");

        let looked_up = session.eval_text("x").unwrap();
        assert_eq!(looked_up.value.to_string(), "42");

        let info = session.inspect("x").unwrap();
        assert!(info.has_own_value);
        assert!(!info.has_delayed_value);
    }

    #[test]
    fn setdelayed_stores_rhs_without_immediate_evaluation() {
        let mut session = Session::new();

        assert_eq!(
            session.eval_text("y := x").unwrap().value.to_string(),
            "Null"
        );
        assert_eq!(session.eval_text("x = 9").unwrap().value.to_string(), "9");
        assert_eq!(session.eval_text("y").unwrap().value.to_string(), "9");

        let info = session.inspect("y").unwrap();
        assert!(!info.has_own_value);
        assert!(info.has_delayed_value);
        assert!(!info.has_down_values);
    }

    #[test]
    fn set_with_non_symbol_lhs_remains_unevaluated_for_now() {
        let mut session = Session::new();
        let outcome = session.eval_text("f[x] = 1").unwrap();

        assert_eq!(outcome.value.to_string(), "Set[f[x], 1]");
    }

    #[test]
    fn lowers_named_pattern_assignment() {
        let syntax = parse_expr("x_Integer := f[x]").unwrap();
        let lowered = lower_syntax_expr(&syntax).unwrap();

        assert_eq!(
            lowered.to_string(),
            "SetDelayed[Pattern[x, Blank[Integer]], f[x]]"
        );
    }

    #[test]
    fn lowers_slots_and_pure_functions() {
        let syntax = parse_expr("#1 + #2 &").unwrap();
        let lowered = lower_syntax_expr(&syntax).unwrap();

        assert_eq!(lowered.to_string(), "Function[Plus[Slot[1], Slot[2]]]");
    }

    #[test]
    fn setdelayed_stores_and_applies_simple_downvalue() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[x_Integer] := x")
                .unwrap()
                .value
                .to_string(),
            "Null"
        );
        assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");

        let info = session.inspect("f").unwrap();
        assert!(info.has_down_values);
    }

    #[test]
    fn matchq_reports_true_for_matching_pattern() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[7, x_Integer]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
    }

    #[test]
    fn matchq_reports_false_for_non_matching_pattern() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[\"hi\", x_Integer]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    }

    #[test]
    fn matchq_supports_blanksequence_in_argument_lists() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[f[1, 2], f[x__]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
        assert_eq!(
            session
                .eval_text("MatchQ[f[], f[x__]]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    }

    #[test]
    fn matchq_supports_blanknullsequence_in_argument_lists() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[f[], f[x___]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
        assert_eq!(
            session
                .eval_text("MatchQ[f[1, 2, 3], f[x__, y_Integer]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
    }

    #[test]
    fn matchq_supports_conditions_on_sequence_patterns() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[f[1, 2], f[x__ /; MatchQ[x, Sequence[1, 2]]]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
        assert_eq!(
            session
                .eval_text("MatchQ[f[1, 2], f[x__ /; MatchQ[x, Sequence[2, 1]]]]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    }

    #[test]
    fn downvalue_matches_after_argument_evaluation() {
        let mut session = Session::new();

        session.eval_text("f[x_Integer] := x").unwrap();
        session.eval_text("y = 11").unwrap();

        assert_eq!(session.eval_text("f[y]").unwrap().value.to_string(), "11");
    }

    #[test]
    fn downvalue_can_substitute_multiple_named_patterns() {
        let mut session = Session::new();

        session.eval_text("pair[a_, b_] := {a, b}").unwrap();

        assert_eq!(
            session.eval_text("pair[1, 2]").unwrap().value.to_string(),
            "List[1, 2]"
        );
    }

    #[test]
    fn downvalue_splices_named_blanksequence_into_rhs_calls() {
        let mut session = Session::new();

        session.eval_text("echo[x__] := g[x]").unwrap();

        assert_eq!(
            session
                .eval_text("echo[1, 2, 3]")
                .unwrap()
                .value
                .to_string(),
            "g[1, 2, 3]"
        );
    }

    #[test]
    fn downvalue_splices_named_blanknullsequence_into_empty_lists() {
        let mut session = Session::new();

        session.eval_text("pack[x___] := {x}").unwrap();

        assert_eq!(
            session.eval_text("pack[]").unwrap().value.to_string(),
            "List[]"
        );
        assert_eq!(
            session.eval_text("pack[1, 2]").unwrap().value.to_string(),
            "List[1, 2]"
        );
    }

    #[test]
    fn unmatched_downvalue_leaves_call_in_place() {
        let mut session = Session::new();

        session.eval_text("f[x_Integer] := x").unwrap();

        assert_eq!(
            session.eval_text("f[\"hi\"]").unwrap().value.to_string(),
            "f[\"hi\"]"
        );
    }

    #[test]
    fn replaceall_applies_single_rule() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("7 /. x_Integer -> x")
                .unwrap()
                .value
                .to_string(),
            "7"
        );
    }

    #[test]
    fn replaceall_applies_rule_list_in_order() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("7 /. {x_String -> 0, x_Integer -> 1}")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
    }

    #[test]
    fn replaceall_traverses_nested_expressions() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[g[1]] /. g[x_Integer] -> h[x]")
                .unwrap()
                .value
                .to_string(),
            "f[h[1]]"
        );
    }

    #[test]
    fn replaceall_splices_named_blanksequence_into_rhs_calls() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[1, 2] /. f[x__] -> g[x]")
                .unwrap()
                .value
                .to_string(),
            "g[1, 2]"
        );
    }

    #[test]
    fn replacerepeated_reaches_a_fixed_point() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[1] //. {f[x_Integer] -> g[x], g[x_Integer] -> x}")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
    }

    #[test]
    fn replacerepeated_errors_on_non_terminating_rewrite_cycle() {
        let mut session = Session::new();

        let error = session
            .eval_text("1 //. {1 -> 2, 2 -> 1}")
            .expect_err("expected iteration limit error");

        assert!(matches!(
            error,
            EvalError::IterationLimitExceeded {
                limit: REPLACE_REPEATED_LIMIT
            }
        ));
    }

    #[test]
    fn immediate_rule_captures_current_rhs_value() {
        let mut session = Session::new();

        session.eval_text("n = 1").unwrap();
        session.eval_text("rule = x_Integer -> n").unwrap();
        session.eval_text("n = 2").unwrap();

        assert_eq!(
            session.eval_text("7 /. rule").unwrap().value.to_string(),
            "1"
        );
    }

    #[test]
    fn delayed_rule_uses_current_rhs_value_at_application_time() {
        let mut session = Session::new();

        session.eval_text("n = 1").unwrap();
        session.eval_text("rule = x_Integer :> n").unwrap();
        session.eval_text("n = 2").unwrap();

        assert_eq!(
            session.eval_text("7 /. rule").unwrap().value.to_string(),
            "2"
        );
    }

    #[test]
    fn matchq_honors_condition_patterns() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[7, x_Integer /; MatchQ[x, 7]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
        assert_eq!(
            session
                .eval_text("MatchQ[8, x_Integer /; MatchQ[x, 7]]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    }

    #[test]
    fn cases_collects_nested_matches() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[g[1], 2, \"hi\"], x_Integer]")
                .unwrap()
                .value
                .to_string(),
            "List[1, 2]"
        );
    }

    #[test]
    fn cases_can_match_the_root_expression() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[1], f[x_Integer]]")
                .unwrap()
                .value
                .to_string(),
            "List[f[1]]"
        );
    }

    #[test]
    fn cases_does_not_treat_heads_as_separate_candidates() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[1, g[2]], x_Symbol]")
                .unwrap()
                .value
                .to_string(),
            "List[]"
        );
    }

    #[test]
    fn cases_applies_rule_transformations() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[g[1], 2], g[x_Integer] :> h[x]]")
                .unwrap()
                .value
                .to_string(),
            "List[h[1]]"
        );
    }

    #[test]
    fn cases_preserves_immediate_and_delayed_rule_behavior() {
        let mut session = Session::new();

        session.eval_text("n = 1").unwrap();
        session.eval_text("immediate = x_Integer -> n").unwrap();
        session.eval_text("delayed = x_Integer :> n").unwrap();
        session.eval_text("n = 2").unwrap();

        assert_eq!(
            session
                .eval_text("Cases[{7}, immediate]")
                .unwrap()
                .value
                .to_string(),
            "List[1]"
        );
        assert_eq!(
            session
                .eval_text("Cases[{7}, delayed]")
                .unwrap()
                .value
                .to_string(),
            "List[2]"
        );
    }

    #[test]
    fn cases_honors_condition_patterns() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[7, 8, 9], x_Integer /; MatchQ[x, 8]]")
                .unwrap()
                .value
                .to_string(),
            "List[8]"
        );
    }

    #[test]
    fn cases_rule_rhs_splices_named_blanksequence_matches() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[g[1, 2], g[3]], g[x__] :> {x}]")
                .unwrap()
                .value
                .to_string(),
            "List[List[1, 2], List[3]]"
        );
    }

    #[test]
    fn replaceall_honors_condition_patterns() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("7 /. (x_Integer /; MatchQ[x, 7]) -> y")
                .unwrap()
                .value
                .to_string(),
            "y"
        );
        assert_eq!(
            session
                .eval_text("8 /. (x_Integer /; MatchQ[x, 7]) -> y")
                .unwrap()
                .value
                .to_string(),
            "8"
        );
    }

    #[test]
    fn conditioned_downvalue_matches_only_when_test_passes() {
        let mut session = Session::new();

        session
            .eval_text("f[x_Integer /; MatchQ[x, 7]] := x")
            .unwrap();

        assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
        assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
    }

    #[test]
    fn outer_conditioned_downvalue_is_stored_and_applied() {
        let mut session = Session::new();

        session
            .eval_text("(f[x_Integer] /; MatchQ[x, 7]) := x")
            .unwrap();

        assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
        assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
    }
}
