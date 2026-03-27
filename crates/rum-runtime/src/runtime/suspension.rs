use std::collections::VecDeque;

use super::*;

pub(super) struct SuspendedEvaluation {
    pub(super) active_stack: Vec<Expr>,
    pub(super) frames: VecDeque<Box<dyn SuspensionFrame>>,
}

pub(super) trait SuspensionFrame {
    fn resume(
        self: Box<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError>;
}

pub(super) trait ContinuationValue: Sized {
    fn into_continuation_expr(self) -> Expr;

    fn from_continuation_expr(expr: Expr) -> Self;
}

impl ContinuationValue for Expr {
    fn into_continuation_expr(self) -> Expr {
        self
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        expr
    }
}

impl ContinuationValue for bool {
    fn into_continuation_expr(self) -> Expr {
        system_expr(if self { "True" } else { "False" })
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        is_system_symbol_name(&expr, "True")
    }
}

impl ContinuationValue for String {
    fn into_continuation_expr(self) -> Expr {
        Expr::string(self)
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Atom(Atom::String(text)) => text,
            other => other.to_string(),
        }
    }
}

impl ContinuationValue for Option<Expr> {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Some(expr) => Expr::call(system_expr("InternalContinuationSome"), vec![expr]),
            None => Expr::call(system_expr("InternalContinuationNone"), vec![]),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationSome") =>
            {
                let mut items = args.into_iter();
                let first = items.next();
                debug_assert!(
                    items.next().is_none(),
                    "option continuation payloads use exactly one item for Some"
                );
                first
            }
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationNone") =>
            {
                debug_assert!(args.is_empty(), "none continuation payloads are empty");
                None
            }
            other => Some(other),
        }
    }
}

impl ContinuationValue for Vec<Expr> {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(system_expr("InternalContinuationVec"), self)
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationVec") =>
            {
                args
            }
            other => vec![other],
        }
    }
}

impl ContinuationValue for Option<Vec<Expr>> {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Some(values) => Expr::call(
                system_expr("InternalContinuationSomeVec"),
                vec![Expr::call(system_expr("List"), values)],
            ),
            None => Expr::call(system_expr("InternalContinuationNone"), vec![]),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationSomeVec") =>
            {
                let mut items = args.into_iter();
                let Some(list_expr) = items.next() else {
                    return Some(Vec::new());
                };
                debug_assert!(
                    items.next().is_none(),
                    "some-vec continuation payloads use exactly one list slot"
                );

                match list_expr {
                    Expr::Call { head, args } if is_system_symbol_name(&head, "List") => Some(args),
                    other => Some(vec![other]),
                }
            }
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationNone") =>
            {
                debug_assert!(args.is_empty(), "none continuation payloads are empty");
                None
            }
            Expr::Call { head, args } if is_system_symbol_name(&head, "List") => Some(args),
            other => Some(vec![other]),
        }
    }
}

fn rewrite_rule_info_expr(rule: RewriteRuleInfo) -> Expr {
    Expr::call(
        system_expr(if rule.delayed { "RuleDelayed" } else { "Rule" }),
        vec![rule.lhs, rule.rhs],
    )
}

fn rewrite_rule_info_from_expr(expr: Expr) -> RewriteRuleInfo {
    match expr {
        Expr::Call { head, mut args }
            if (is_system_symbol_name(&head, "Rule")
                || is_system_symbol_name(&head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let rhs = args.pop().expect("rewrite-rule continuation rhs exists");
            let lhs = args.pop().expect("rewrite-rule continuation lhs exists");
            RewriteRuleInfo {
                lhs,
                rhs,
                delayed: is_system_symbol_name(&head, "RuleDelayed"),
            }
        }
        other => RewriteRuleInfo {
            lhs: other,
            rhs: system_expr("Null"),
            delayed: false,
        },
    }
}

impl ContinuationValue for Option<Vec<RewriteRuleInfo>> {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Some(values) => Expr::call(
                system_expr("InternalContinuationSomeRewriteRuleInfos"),
                vec![Expr::call(
                    system_expr("List"),
                    values.into_iter().map(rewrite_rule_info_expr).collect(),
                )],
            ),
            None => Expr::call(system_expr("InternalContinuationNone"), vec![]),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationSomeRewriteRuleInfos") =>
            {
                let Some(list_expr) = args.pop() else {
                    return Some(Vec::new());
                };
                debug_assert!(
                    args.is_empty(),
                    "rewrite-rule continuation payloads use exactly one list slot"
                );

                match list_expr {
                    Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
                        Some(args.into_iter().map(rewrite_rule_info_from_expr).collect())
                    }
                    other => Some(vec![rewrite_rule_info_from_expr(other)]),
                }
            }
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationNone") =>
            {
                debug_assert!(args.is_empty(), "none continuation payloads are empty");
                None
            }
            Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
                Some(args.into_iter().map(rewrite_rule_info_from_expr).collect())
            }
            other => Some(vec![rewrite_rule_info_from_expr(other)]),
        }
    }
}

fn cases_level_spec_expr(spec: CasesLevelSpec) -> Expr {
    Expr::call(
        system_expr("InternalContinuationCasesLevelSpec"),
        vec![
            Expr::integer(BigInt::from(spec.min)),
            spec.max
                .map(|value| Expr::integer(BigInt::from(value)))
                .unwrap_or_else(|| system_expr("Infinity")),
        ],
    )
}

fn cases_level_spec_from_expr(expr: Expr) -> CasesLevelSpec {
    match expr {
        Expr::Call { head, mut args }
            if is_system_symbol_name(&head, "InternalContinuationCasesLevelSpec")
                && args.len() == 2 =>
        {
            let max_expr = args.pop().expect("cases-level payload stores max");
            let min_expr = args.pop().expect("cases-level payload stores min");
            let min = expr_as_signed_isize(&min_expr)
                .expect("cases-level payload min should be a signed integer");
            let max = if is_system_symbol_name(&max_expr, "Infinity") {
                None
            } else {
                Some(
                    expr_as_signed_isize(&max_expr)
                        .expect("cases-level payload max should be a signed integer"),
                )
            };

            CasesLevelSpec::range(min, max)
        }
        other => panic!("unexpected continuation payload for CasesLevelSpec: {other:?}"),
    }
}

impl ContinuationValue for CasesTraversalConfig {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationCasesTraversalConfig"),
            vec![
                cases_level_spec_expr(self.levels),
                self.max_matches
                    .map(|value| Expr::integer(BigInt::from(value)))
                    .unwrap_or_else(|| system_expr("InternalContinuationNone")),
                system_expr(if self.include_heads { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationCasesTraversalConfig")
                    && args.len() == 3 =>
            {
                let include_heads_expr = args
                    .pop()
                    .expect("cases-traversal payload stores include-heads");
                let max_matches_expr = args
                    .pop()
                    .expect("cases-traversal payload stores max-matches");
                let levels_expr = args.pop().expect("cases-traversal payload stores levels");

                Self {
                    levels: cases_level_spec_from_expr(levels_expr),
                    max_matches: if is_system_symbol_name(
                        &max_matches_expr,
                        "InternalContinuationNone",
                    ) {
                        None
                    } else {
                        expr_as_non_negative_usize(&max_matches_expr)
                    },
                    include_heads: is_system_symbol_name(&include_heads_expr, "True"),
                }
            }
            other => panic!("unexpected continuation payload for CasesTraversalConfig: {other:?}"),
        }
    }
}

impl ContinuationValue for CasesTraversalResolution {
    fn into_continuation_expr(self) -> Expr {
        match self {
            CasesTraversalResolution::Valid(config) => Expr::call(
                system_expr("InternalContinuationCasesTraversalValid"),
                vec![config.into_continuation_expr()],
            ),
            CasesTraversalResolution::Invalid(error) => Expr::call(
                system_expr("InternalContinuationCasesTraversalInvalid"),
                vec![Expr::integer(BigInt::from(match error {
                    CasesTraversalNormalizationError::InvalidThirdArgument => 3_i32,
                    CasesTraversalNormalizationError::InvalidFourthArgument => 4_i32,
                    CasesTraversalNormalizationError::InvalidFifthArgument => 5_i32,
                }))],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationCasesTraversalValid")
                    && args.len() == 1 =>
            {
                let config_expr = args.pop().expect("valid cases-traversal stores config");
                Self::Valid(CasesTraversalConfig::from_continuation_expr(config_expr))
            }
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationCasesTraversalInvalid")
                    && args.len() == 1 =>
            {
                let code_expr = args.pop().expect("invalid cases-traversal stores code");
                let code = expr_as_non_negative_usize(&code_expr)
                    .expect("invalid cases-traversal code should be an integer");
                let error = match code {
                    3 => CasesTraversalNormalizationError::InvalidThirdArgument,
                    4 => CasesTraversalNormalizationError::InvalidFourthArgument,
                    5 => CasesTraversalNormalizationError::InvalidFifthArgument,
                    other => panic!("unexpected cases-traversal error code: {other}"),
                };
                Self::Invalid(error)
            }
            other => {
                panic!("unexpected continuation payload for CasesTraversalResolution: {other:?}")
            }
        }
    }
}

impl ContinuationValue for CasesCollectionOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationCasesCollection"),
            vec![
                Expr::call(system_expr("List"), self.matches),
                Expr::integer(BigInt::from(self.depth)),
                system_expr(if self.exhausted { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationCasesCollection")
                    && args.len() == 3 =>
            {
                let exhausted_expr = args
                    .pop()
                    .expect("cases-collection payload stores exhausted");
                let depth_expr = args.pop().expect("cases-collection payload stores depth");
                let matches_expr = args.pop().expect("cases-collection payload stores matches");

                let matches = match matches_expr {
                    Expr::Call { head, args } if is_system_symbol_name(&head, "List") => args,
                    other => vec![other],
                };

                Self {
                    matches,
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("cases-collection depth should be non-negative"),
                    exhausted: is_system_symbol_name(&exhausted_expr, "True"),
                }
            }
            other => {
                panic!("unexpected continuation payload for CasesCollectionOutcome: {other:?}")
            }
        }
    }
}

impl ContinuationValue for PositionCollectionOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationPositionCollection"),
            vec![
                Expr::call(system_expr("List"), self.positions),
                Expr::integer(BigInt::from(self.depth)),
                system_expr(if self.exhausted { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationPositionCollection")
                    && args.len() == 3 =>
            {
                let exhausted_expr = args
                    .pop()
                    .expect("position-collection payload stores exhausted");
                let depth_expr = args
                    .pop()
                    .expect("position-collection payload stores depth");
                let positions_expr = args
                    .pop()
                    .expect("position-collection payload stores positions");

                let positions = match positions_expr {
                    Expr::Call { head, args } if is_system_symbol_name(&head, "List") => args,
                    other => vec![other],
                };

                Self {
                    positions,
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("position-collection depth should be non-negative"),
                    exhausted: is_system_symbol_name(&exhausted_expr, "True"),
                }
            }
            other => {
                panic!("unexpected continuation payload for PositionCollectionOutcome: {other:?}")
            }
        }
    }
}

impl ContinuationValue for TraversalCountOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationTraversalCount"),
            vec![
                Expr::integer(BigInt::from(self.count)),
                Expr::integer(BigInt::from(self.depth)),
                system_expr(if self.exhausted { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationTraversalCount")
                    && args.len() == 3 =>
            {
                let exhausted_expr = args
                    .pop()
                    .expect("traversal-count payload stores exhausted");
                let depth_expr = args.pop().expect("traversal-count payload stores depth");
                let count_expr = args.pop().expect("traversal-count payload stores count");

                Self {
                    count: expr_as_non_negative_usize(&count_expr)
                        .expect("traversal-count count should be non-negative"),
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("traversal-count depth should be non-negative"),
                    exhausted: is_system_symbol_name(&exhausted_expr, "True"),
                }
            }
            other => {
                panic!("unexpected continuation payload for TraversalCountOutcome: {other:?}")
            }
        }
    }
}

struct ClosureFrame<F> {
    resume: Option<F>,
}

impl<F> SuspensionFrame for ClosureFrame<F>
where
    F: FnOnce(&mut Session, Result<Expr, EvalError>) -> Result<Expr, EvalError> + 'static,
{
    fn resume(
        mut self: Box<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let resume = self
            .resume
            .take()
            .expect("suspension frames are resumed at most once");
        resume(session, outcome)
    }
}

impl SuspendedEvaluation {
    fn new(active_stack: Vec<Expr>) -> Self {
        Self {
            active_stack,
            frames: VecDeque::new(),
        }
    }
}

impl Session {
    pub(super) fn drive_evaluated_value<T, U, F>(
        &mut self,
        result: Result<Expr, EvalError>,
        cont: F,
    ) -> Result<U, EvalError>
    where
        T: ContinuationValue + 'static,
        U: ContinuationValue + 'static,
        F: FnOnce(&mut Session, T) -> Result<U, EvalError> + 'static,
    {
        match result {
            Ok(value) => cont(self, T::from_continuation_expr(value)),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, outcome| match outcome {
                    Ok(value) => {
                        let resumed = cont(session, T::from_continuation_expr(value))?;
                        Ok(resumed.into_continuation_expr())
                    }
                    Err(error) => Err(error),
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn drive_internal_value<T, U, F>(
        &mut self,
        result: Result<T, EvalError>,
        cont: F,
    ) -> Result<U, EvalError>
    where
        T: ContinuationValue + 'static,
        U: ContinuationValue + 'static,
        F: FnOnce(&mut Session, T) -> Result<U, EvalError> + 'static,
    {
        match result {
            Ok(value) => cont(self, value),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, outcome| match outcome {
                    Ok(value) => {
                        let resumed = cont(session, T::from_continuation_expr(value))?;
                        Ok(resumed.into_continuation_expr())
                    }
                    Err(error) => Err(error),
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn suspend_with_frame<F>(
        &mut self,
        result: Result<Expr, EvalError>,
        frame: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&mut Session, Expr) -> Result<Expr, EvalError> + 'static,
    {
        self.suspend_with_result_frame(result, move |session, outcome| match outcome {
            Ok(value) => frame(session, value),
            Err(error) => Err(error),
        })
    }

    pub(super) fn suspend_with_result_frame<F>(
        &mut self,
        result: Result<Expr, EvalError>,
        frame: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&mut Session, Result<Expr, EvalError>) -> Result<Expr, EvalError> + 'static,
    {
        match result {
            Ok(value) => Ok(value),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(frame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn begin_input_suspension(&mut self, request: RuntimeInputRequest) {
        // The current InputString call and its immediate caller are both
        // resumed from saved suspension frames rather than by restarting
        // evaluate_once. Keeping either expression on the active stack causes
        // false-positive cycle detection when matcher backtracking or other
        // helper-level continuations legitimately re-enter that caller after
        // input arrives.
        let retained_depth = self.evaluation_stack.len().saturating_sub(2);
        let active_stack = self
            .evaluation_stack
            .iter()
            .take(retained_depth)
            .cloned()
            .collect::<Vec<_>>();

        self.pending_input_request = Some(request);
        self.pending_suspension = Some(SuspendedEvaluation::new(active_stack));
    }

    pub(super) fn push_suspension_frame<F>(&mut self, frame: F)
    where
        F: FnOnce(&mut Session, Result<Expr, EvalError>) -> Result<Expr, EvalError> + 'static,
    {
        let suspension = self
            .pending_suspension
            .as_mut()
            .expect("input suspension should be initialized before frames are pushed");
        suspension.frames.push_back(Box::new(ClosureFrame {
            resume: Some(frame),
        }));
    }

    pub(super) fn clear_suspension_state(&mut self) {
        self.pending_input_request = None;
        self.pending_suspension = None;
    }

    pub(super) fn resume_suspended_evaluation(
        &mut self,
        input: impl Into<String>,
    ) -> Result<Expr, EvalError> {
        let mut suspension = self
            .pending_suspension
            .take()
            .expect("resume requires a pending suspended evaluation");
        let saved_stack = std::mem::replace(&mut self.evaluation_stack, suspension.active_stack);
        let saved_depth =
            std::mem::replace(&mut self.evaluation_depth, self.evaluation_stack.len());
        self.pending_input_request = None;
        let mut outcome = Ok(Expr::string(input.into()));

        let result = (|| -> Result<Expr, EvalError> {
            while let Some(frame) = suspension.frames.pop_front() {
                outcome = frame.resume(self, outcome);

                if let Err(EvalError::InputRequired { request }) = &outcome {
                    let mut nested = self
                        .pending_suspension
                        .take()
                        .expect("nested input request should establish a fresh suspension");
                    nested.frames.append(&mut suspension.frames);
                    self.pending_input_request = Some(request.clone());
                    self.pending_suspension = Some(nested);
                    return Err(EvalError::InputRequired {
                        request: request.clone(),
                    });
                }
            }

            outcome
        })();

        self.evaluation_stack = saved_stack;
        self.evaluation_depth = saved_depth;

        match result {
            Ok(value) => {
                self.pending_input_request = None;
                Ok(value)
            }
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            Err(error) => {
                self.clear_suspension_state();
                Err(error)
            }
        }
    }
}
