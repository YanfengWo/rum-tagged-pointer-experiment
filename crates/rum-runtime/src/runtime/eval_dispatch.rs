use super::*;

impl Session {
    pub(super) fn build_call_dispatch_plan(&self, head: &Expr) -> CallDispatchPlan {
        let attributes = self.attribute_set_for_head(head);
        let builtin_dispatch = expr_as_symbol(head)
            .and_then(|symbol| self.builtin_dispatch_spec_for_symbol(symbol))
            .map(|spec| spec.dispatch);

        if builtin_dispatch == Some(BuiltinDispatch::SpecialForm) {
            return CallDispatchPlan {
                attributes,
                stages: vec![CallDispatchStage::SpecialForm],
            };
        }

        let mut stages = Vec::with_capacity(6);

        stages.push(CallDispatchStage::PrepareArguments);

        if attributes.contains(Attribute::Listable) {
            stages.push(CallDispatchStage::Listable);
        }

        if !attributes.contains(Attribute::HoldAllComplete) {
            stages.push(CallDispatchStage::UpValues);
        }

        stages.extend([CallDispatchStage::DownValues, CallDispatchStage::SubValues]);

        if builtin_dispatch == Some(BuiltinDispatch::Ordinary) {
            stages.push(CallDispatchStage::OrdinaryBuiltin);
        }

        if parse_function(head).is_some()
            || parse_composition(head).is_some()
            || is_callable_operator_wrapper(head)
        {
            stages.push(CallDispatchStage::Function);
        }

        CallDispatchPlan { attributes, stages }
    }

    pub(super) fn run_call_dispatch_stage(
        &mut self,
        stage: CallDispatchStage,
        head: &Expr,
        attributes: AttributeSet,
        state: &mut CallDispatchState,
    ) -> Result<Option<Expr>, EvalError> {
        match stage {
            CallDispatchStage::SpecialForm => self.try_dispatch_builtin_call(
                BuiltinDispatch::SpecialForm,
                head,
                state.raw_args.as_slice(),
            ),
            CallDispatchStage::PrepareArguments => {
                let raw_args = std::mem::take(&mut state.raw_args);
                let mut evaluated_args =
                    self.prepare_arguments_for_call(head.clone(), attributes, raw_args)?;
                evaluated_args =
                    self.normalize_call_args_by_attributes(head, evaluated_args, attributes);
                state.evaluated_args = Some(evaluated_args);
                Ok(None)
            }
            CallDispatchStage::Listable => {
                match self.dispatch_listable_stage(head, state.evaluated_args(), attributes)? {
                    ThreadingDispatch::NotApplicable => Ok(None),
                    ThreadingDispatch::Threaded(threaded) => {
                        let threaded_result = self.evaluate(threaded);
                        self.suspend_with_frame(threaded_result, |_, value| Ok(value))
                            .map(Some)
                    }
                    ThreadingDispatch::Mismatch => Ok(Some(Expr::call(
                        head.clone(),
                        state.evaluated_args().to_vec(),
                    ))),
                }
            }
            CallDispatchStage::UpValues => self.try_apply_up_values(head, state.evaluated_args()),
            CallDispatchStage::DownValues => {
                self.try_apply_down_values(head, state.evaluated_args())
            }
            CallDispatchStage::SubValues => self.try_apply_sub_values(head, state.evaluated_args()),
            CallDispatchStage::OrdinaryBuiltin => self.try_dispatch_builtin_call(
                BuiltinDispatch::Ordinary,
                head,
                state.evaluated_args(),
            ),
            CallDispatchStage::Function => self.try_apply_function(head, state.evaluated_args()),
        }
    }

    pub(super) fn try_dispatch_builtin_call(
        &mut self,
        dispatch: BuiltinDispatch,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        let Some(spec) = self.builtin_dispatch_spec_for_symbol(symbol) else {
            return Ok(None);
        };

        if spec.dispatch != dispatch {
            return Ok(None);
        }

        let Some(handler) = spec.handler else {
            return Ok(None);
        };

        self.run_builtin_handler(handler, head, args).map(Some)
    }

    pub(super) fn run_builtin_handler(
        &mut self,
        handler: BuiltinHandler,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match handler {
            BuiltinHandler::Identity => Ok(if args.len() == 1 {
                args[0].clone()
            } else {
                Expr::call(head.clone(), args.to_vec())
            }),
            BuiltinHandler::Plus => self.eval_plus(head, args),
            BuiltinHandler::Times => self.eval_times(head, args),
            BuiltinHandler::Power => self.eval_power(head, args),
            BuiltinHandler::Subtract => self.eval_subtract(head, args),
            BuiltinHandler::Divide => self.eval_divide(head, args),
            BuiltinHandler::Mod => self.eval_mod(head, args),
            BuiltinHandler::Quotient => self.eval_quotient(head, args),
            BuiltinHandler::PowerMod => self.eval_power_mod(head, args),
            BuiltinHandler::GCD => self.eval_gcd(head, args),
            BuiltinHandler::LCM => self.eval_lcm(head, args),
            BuiltinHandler::Expand => self.eval_expand(head, args),
            BuiltinHandler::Coefficient => self.eval_coefficient(head, args),
            BuiltinHandler::CoefficientList => self.eval_coefficient_list(head, args),
            BuiltinHandler::Together => self.eval_together(head, args),
            BuiltinHandler::Factor => self.eval_factor(head, args),
            BuiltinHandler::D => self.eval_d(head, args),
            BuiltinHandler::StringJoin => self.eval_string_join(head, args),
            BuiltinHandler::StringLength => self.eval_string_length(head, args),
            BuiltinHandler::Characters => self.eval_characters(head, args),
            BuiltinHandler::CharacterRange => self.eval_character_range(head, args),
            BuiltinHandler::ToString => self.eval_to_string(head, args),
            BuiltinHandler::StringRepeat => self.eval_string_repeat(head, args),
            BuiltinHandler::StringContainsQ => self.eval_string_contains_q(head, args),
            BuiltinHandler::StringStartsQ => self.eval_string_starts_q(head, args),
            BuiltinHandler::StringEndsQ => self.eval_string_ends_q(head, args),
            BuiltinHandler::StringPosition => self.eval_string_position(head, args),
            BuiltinHandler::StringSplit => self.eval_string_split(head, args),
            BuiltinHandler::StringTrim => self.eval_string_trim(head, args),
            BuiltinHandler::StringReplace => self.eval_string_replace(head, args),
            BuiltinHandler::StringCases => self.eval_string_cases(head, args),
            BuiltinHandler::StringTake => self.eval_string_take(head, args),
            BuiltinHandler::StringDrop => self.eval_string_drop(head, args),
            BuiltinHandler::StringPart => self.eval_string_part(head, args),
            BuiltinHandler::FullForm => self.eval_full_form(head, args),
            BuiltinHandler::Head => self.eval_head(head, args),
            BuiltinHandler::Length => self.eval_length(head, args),
            BuiltinHandler::Depth => self.eval_depth(head, args),
            BuiltinHandler::First => self.eval_first(head, args),
            BuiltinHandler::Last => self.eval_last(head, args),
            BuiltinHandler::Rest => self.eval_rest(head, args),
            BuiltinHandler::Most => self.eval_most(head, args),
            BuiltinHandler::Take => self.eval_take(head, args),
            BuiltinHandler::Drop => self.eval_drop(head, args),
            BuiltinHandler::TakeDrop => self.eval_take_drop(head, args),
            BuiltinHandler::Insert => self.eval_insert(head, args),
            BuiltinHandler::Delete => self.eval_delete(head, args),
            BuiltinHandler::TakeList => self.eval_take_list(head, args),
            BuiltinHandler::Append => self.eval_append(head, args),
            BuiltinHandler::Prepend => self.eval_prepend(head, args),
            BuiltinHandler::Symbol => self.eval_symbol(head, args),
            BuiltinHandler::Construct => self.eval_construct(head, args),
            BuiltinHandler::And => self.eval_and(head, args),
            BuiltinHandler::Or => self.eval_or(head, args),
            BuiltinHandler::Not => self.eval_not(head, args),
            BuiltinHandler::TrueQ => self.eval_true_q(head, args),
            BuiltinHandler::BooleanQ => self.eval_boolean_q(head, args),
            BuiltinHandler::AtomQ => self.eval_atom_q(head, args),
            BuiltinHandler::ListQ => self.eval_list_q(head, args),
            BuiltinHandler::StringQ => self.eval_string_q(head, args),
            BuiltinHandler::IntegerQ => self.eval_integer_q(head, args),
            BuiltinHandler::NumberQ => self.eval_number_q(head, args),
            BuiltinHandler::NumericQ => self.eval_numeric_q(head, args),
            BuiltinHandler::EvenQ => self.eval_even_q(head, args),
            BuiltinHandler::OddQ => self.eval_odd_q(head, args),
            BuiltinHandler::Positive => self.eval_positive(head, args),
            BuiltinHandler::Negative => self.eval_negative(head, args),
            BuiltinHandler::NonPositive => self.eval_non_positive(head, args),
            BuiltinHandler::NonNegative => self.eval_non_negative(head, args),
            BuiltinHandler::PossibleZeroQ => self.eval_possible_zero_q(head, args),
            BuiltinHandler::MachineNumberQ => self.eval_machine_number_q(head, args),
            BuiltinHandler::RealValuedNumberQ => self.eval_real_valued_number_q(head, args),
            BuiltinHandler::RealValuedNumericQ => self.eval_real_valued_numeric_q(head, args),
            BuiltinHandler::PrimeQ => self.eval_prime_q(head, args),
            BuiltinHandler::AlgebraicIntegerQ => self.eval_algebraic_integer_q(head, args),
            BuiltinHandler::DuplicateFreeQ => self.eval_duplicate_free_q(head, args),
            BuiltinHandler::IntersectingQ => self.eval_intersecting_q(head, args),
            BuiltinHandler::DisjointQ => self.eval_disjoint_q(head, args),
            BuiltinHandler::SubsetQ => self.eval_subset_q(head, args),
            BuiltinHandler::Order => self.eval_order(head, args),
            BuiltinHandler::OrderedQ => self.eval_ordered_q(head, args),
            BuiltinHandler::ContainsAll => self.eval_contains_all(head, args),
            BuiltinHandler::ContainsAny => self.eval_contains_any(head, args),
            BuiltinHandler::ContainsExactly => self.eval_contains_exactly(head, args),
            BuiltinHandler::ContainsNone => self.eval_contains_none(head, args),
            BuiltinHandler::ContainsOnly => self.eval_contains_only(head, args),
            BuiltinHandler::Between => self.eval_between(head, args),
            BuiltinHandler::VectorQ => self.eval_vector_q(head, args),
            BuiltinHandler::MatrixQ => self.eval_matrix_q(head, args),
            BuiltinHandler::ArrayQ => self.eval_array_q(head, args),
            BuiltinHandler::AllTrue => self.eval_all_true(head, args),
            BuiltinHandler::AnyTrue => self.eval_any_true(head, args),
            BuiltinHandler::NoneTrue => self.eval_none_true(head, args),
            BuiltinHandler::CoprimeQ => self.eval_coprime_q(head, args),
            BuiltinHandler::Association => self.eval_association(head, args),
            BuiltinHandler::AssociationQ => self.eval_association_q(head, args),
            BuiltinHandler::Lookup => self.eval_lookup(head, args),
            BuiltinHandler::Key => self.eval_key(head, args),
            BuiltinHandler::Keys => self.eval_keys(head, args),
            BuiltinHandler::Values => self.eval_values(head, args),
            BuiltinHandler::KeyExistsQ => self.eval_key_exists_q(head, args),
            BuiltinHandler::KeyTake => self.eval_key_take(head, args),
            BuiltinHandler::KeyDrop => self.eval_key_drop(head, args),
            BuiltinHandler::KeyMap => self.eval_key_map(head, args),
            BuiltinHandler::KeyValueMap => self.eval_key_value_map(head, args),
            BuiltinHandler::KeySelect => self.eval_key_select(head, args),
            BuiltinHandler::KeyMemberQ => self.eval_key_member_q(head, args),
            BuiltinHandler::KeyFreeQ => self.eval_key_free_q(head, args),
            BuiltinHandler::KeySort => self.eval_key_sort(head, args),
            BuiltinHandler::KeySortBy => self.eval_key_sort_by(head, args),
            BuiltinHandler::AssociationMap => self.eval_association_map(head, args),
            BuiltinHandler::AssociationThread => self.eval_association_thread(head, args),
            BuiltinHandler::KeyUnion => self.eval_key_union(head, args),
            BuiltinHandler::KeyIntersection => self.eval_key_intersection(head, args),
            BuiltinHandler::KeyComplement => self.eval_key_complement(head, args),
            BuiltinHandler::Normal => self.eval_normal(head, args),
            BuiltinHandler::AssociateTo => self.eval_associate_to(head, args),
            BuiltinHandler::KeyDropFrom => self.eval_key_drop_from(head, args),
            BuiltinHandler::KeyValuePattern => self.eval_key_value_pattern(head, args),
            BuiltinHandler::Merge => self.eval_merge(head, args),
            BuiltinHandler::Catenate => self.eval_catenate(head, args),
            BuiltinHandler::Counts => self.eval_counts(head, args),
            BuiltinHandler::CountsBy => self.eval_counts_by(head, args),
            BuiltinHandler::GroupBy => self.eval_group_by(head, args),
            BuiltinHandler::PositionIndex => self.eval_position_index(head, args),
            BuiltinHandler::Missing => self.eval_missing(head, args),
            BuiltinHandler::SameQ => self.eval_same_q(head, args),
            BuiltinHandler::UnsameQ => self.eval_unsame_q(head, args),
            BuiltinHandler::Equal => self.eval_equal(head, args),
            BuiltinHandler::Unequal => self.eval_unequal(head, args),
            BuiltinHandler::Less => self.eval_less(head, args),
            BuiltinHandler::Greater => self.eval_greater(head, args),
            BuiltinHandler::LessEqual => self.eval_less_equal(head, args),
            BuiltinHandler::GreaterEqual => self.eval_greater_equal(head, args),
            BuiltinHandler::Part => self.eval_part(head, args),
            BuiltinHandler::Extract => self.eval_extract(head, args),
            BuiltinHandler::ReplacePart => self.eval_replace_part(head, args),
            BuiltinHandler::Set => self.eval_set(args),
            BuiltinHandler::SetDelayed => self.eval_set_delayed(args),
            BuiltinHandler::TagSet => self.eval_tag_set(args),
            BuiltinHandler::TagSetDelayed => self.eval_tag_set_delayed(args),
            BuiltinHandler::TagUnset => self.eval_tag_unset(args),
            BuiltinHandler::Rule => self.eval_rule(args),
            BuiltinHandler::RuleDelayed => self.eval_rule_delayed(args),
            BuiltinHandler::MatchQ => self.eval_match_q(args),
            BuiltinHandler::Cases => self.eval_cases(args),
            BuiltinHandler::Count => self.eval_count(args),
            BuiltinHandler::MemberQ => self.eval_member_q(args),
            BuiltinHandler::FreeQ => self.eval_free_q(args),
            BuiltinHandler::Level => self.eval_level(head, args),
            BuiltinHandler::Position => self.eval_position(args),
            BuiltinHandler::Module => self.eval_module(args),
            BuiltinHandler::Block => self.eval_block(args),
            BuiltinHandler::ReplaceAll => self.eval_replace_all(args),
            BuiltinHandler::ReplaceRepeated => self.eval_replace_repeated(args),
            BuiltinHandler::FindFile => self.eval_find_file(args),
            BuiltinHandler::Get => self.eval_get(args),
            BuiltinHandler::Needs => self.eval_needs(args),
            BuiltinHandler::Context => self.eval_context(args),
            BuiltinHandler::Begin => self.eval_begin(args),
            BuiltinHandler::End => self.eval_end(args),
            BuiltinHandler::BeginPackage => self.eval_begin_package(args),
            BuiltinHandler::EndPackage => self.eval_end_package(args),
            BuiltinHandler::Message => self.eval_message(args),
            BuiltinHandler::MessageName => self.eval_message_name(args),
            BuiltinHandler::Messages => self.eval_messages(args),
            BuiltinHandler::MessageDefinitions => self.eval_message_definitions(args),
            BuiltinHandler::MessageTags => self.eval_message_tags(args),
            BuiltinHandler::OwnValues => self.eval_own_values(args),
            BuiltinHandler::UpValues => self.eval_up_values(args),
            BuiltinHandler::DownValues => self.eval_down_values(args),
            BuiltinHandler::SubValues => self.eval_sub_values(args),
            BuiltinHandler::Attributes => self.eval_attributes(args),
            BuiltinHandler::SetAttributes => self.eval_set_attributes(args),
            BuiltinHandler::ClearAttributes => self.eval_clear_attributes(args),
            BuiltinHandler::Unset => self.eval_unset(args),
            BuiltinHandler::Clear => self.eval_clear(args),
            BuiltinHandler::ClearAll => self.eval_clear_all(args),
            BuiltinHandler::NormalizeRules => self.eval_normalize_rules(args),
            BuiltinHandler::Check => self.eval_check(args),
            BuiltinHandler::Quiet => self.eval_quiet(args),
            BuiltinHandler::CompoundExpression => self.eval_compound_expression(args),
            BuiltinHandler::If => self.eval_if(args),
            BuiltinHandler::Which => self.eval_which(args),
            BuiltinHandler::Switch => self.eval_switch(args),
            BuiltinHandler::Return => self.eval_return(head, args),
            BuiltinHandler::Throw => self.eval_throw(head, args),
            BuiltinHandler::Catch => self.eval_catch(args),
            BuiltinHandler::With => self.eval_with(args),
            BuiltinHandler::BlockMap => self.eval_block_map(head, args),
            BuiltinHandler::SubsetMap => self.eval_subset_map(head, args),
            BuiltinHandler::Comap => self.eval_comap(head, args),
            BuiltinHandler::ComapApply => self.eval_comap_apply(head, args),
            BuiltinHandler::Accumulate => self.eval_accumulate(head, args),
            BuiltinHandler::MapIndexed => self.eval_map_indexed(head, args),
            BuiltinHandler::Map => self.eval_map(head, args),
            BuiltinHandler::MapAll => self.eval_map_all(head, args),
            BuiltinHandler::MapAt => self.eval_map_at(head, args),
            BuiltinHandler::Scan => self.eval_scan(head, args),
            BuiltinHandler::MapThread => self.eval_map_thread(head, args),
            BuiltinHandler::TakeWhile => self.eval_take_while(head, args),
            BuiltinHandler::LengthWhile => self.eval_length_while(head, args),
            BuiltinHandler::Inner => self.eval_inner(head, args),
            BuiltinHandler::Fold => self.eval_fold(head, args),
            BuiltinHandler::FoldList => self.eval_fold_list(head, args),
            BuiltinHandler::FoldPair => self.eval_fold_pair(head, args),
            BuiltinHandler::FoldWhile => self.eval_fold_while(head, args),
            BuiltinHandler::SequenceFold => self.eval_sequence_fold(head, args),
            BuiltinHandler::SequenceFoldList => self.eval_sequence_fold_list(head, args),
            BuiltinHandler::Nest => self.eval_nest(head, args),
            BuiltinHandler::NestWhile => self.eval_nest_while(head, args),
            BuiltinHandler::NestList => self.eval_nest_list(head, args),
            BuiltinHandler::NestWhileList => self.eval_nest_while_list(head, args),
            BuiltinHandler::FixedPoint => self.eval_fixed_point(head, args),
            BuiltinHandler::FixedPointList => self.eval_fixed_point_list(head, args),
            BuiltinHandler::Through => self.eval_through(head, args),
            BuiltinHandler::Operate => self.eval_operate(head, args),
            BuiltinHandler::ArrayFilter => self.eval_array_filter(head, args),
            BuiltinHandler::ArrayReduce => self.eval_array_reduce(head, args),
            BuiltinHandler::Apply => self.eval_apply(head, args),
            BuiltinHandler::MapApply => self.eval_map_apply(head, args),
            BuiltinHandler::OperatorApplied => self.eval_operator_applied(head, args),
            BuiltinHandler::CurryApplied => self.eval_curry_applied(head, args),
            BuiltinHandler::ReverseApplied => self.eval_reverse_applied(head, args),
            BuiltinHandler::Composition => self.eval_composition(head, args),
            BuiltinHandler::RightComposition => self.eval_right_composition(head, args),
            BuiltinHandler::HoldLiteral => Ok(Expr::call(head.clone(), args.to_vec())),
            BuiltinHandler::Unevaluated => Ok(if args.len() == 1 {
                args[0].clone()
            } else {
                Expr::call(head.clone(), args.to_vec())
            }),
            BuiltinHandler::ReleaseHold => self.eval_release_hold(args),
            BuiltinHandler::Evaluate => Ok(if args.len() == 1 {
                let evaluated_result = self.evaluate(args[0].clone());
                self.suspend_with_frame(evaluated_result, |_, value| Ok(value))?
            } else {
                Expr::call(head.clone(), args.to_vec())
            }),
            BuiltinHandler::Sequence => Ok(sequence_expr(args.to_vec())),
            BuiltinHandler::Thread => self.eval_thread(head, args),
            BuiltinHandler::InputString => self.eval_input_string(args),
        }
    }
}
