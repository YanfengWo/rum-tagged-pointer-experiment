# Brief 098 kernel API audit report

Diagnostic only. The compile-error axis used the existing raw logs for `arithmetic`, `associations`, `calculus`, `combinatorics`, `control_flow`, and `functional_ops`, then generated fresh raw logs for `integer_digits`, `logic`, `number_theory`, `polynomials`, `predicates`, `strings`, `structure_ops`, and `rewrite_builtins`.

Parser note: free-item diagnostics are counted only when the primary compiler span is inside the family being audited. Downstream `builtin_catalog/*` fallout from broken family impls is excluded as cascade. Axis 2 is a heuristic source scan for `self.<method>(` / `session.<method>(` where the method has a `Session` impl definition outside the family.

## Per-family counts
| family | distinct items | trivial external | runtime/kernel modules | same-family local | other-family coupling | unknown |
|---|---:|---:|---:|---:|---:|---:|
| `arithmetic` | 11 | 4 | 7 | 0 | 0 | 0 |
| `associations` | 64 | 5 | 12 | 44 | 3 | 0 |
| `calculus` | 12 | 4 | 6 | 0 | 2 | 0 |
| `combinatorics` | 7 | 3 | 3 | 0 | 1 | 0 |
| `control_flow` | 60 | 8 | 25 | 16 | 11 | 0 |
| `functional_ops` | 57 | 6 | 15 | 28 | 8 | 0 |
| `integer_digits` | 10 | 3 | 6 | 0 | 1 | 0 |
| `logic` | 9 | 4 | 5 | 0 | 0 | 0 |
| `number_theory` | 11 | 4 | 5 | 0 | 2 | 0 |
| `polynomials` | 38 | 7 | 7 | 22 | 2 | 0 |
| `predicates` | 30 | 5 | 11 | 14 | 0 | 0 |
| `strings` | 66 | 12 | 13 | 38 | 3 | 0 |
| `structure_ops` | 59 | 7 | 15 | 29 | 8 | 0 |
| `rewrite_builtins` | 33 | 2 | 26 | 3 | 2 | 0 |

## Family to family couplings
| from family | to family | items |
|---|---|---|
| `associations` | `structure_ops` | `exact_expr_at_path`, `exact_position_paths_from_part_selector_exprs`, `replace_exact_path` |
| `calculus` | `arithmetic` | `expr_as_exact_rational` |
| `calculus` | `polynomials` | `parse_indeterminate_expr` |
| `combinatorics` | `arithmetic` | `expr_as_exact_integer` |
| `control_flow` | `arithmetic` | `expr_as_exact_integer` |
| `control_flow` | `associations` | `AssociationEntryForm`, `association_entries_from_expr`, `association_entry_forms_from_expr`, `association_value_by_key`, `is_missing_expr`, `list_items`, `parse_association_entry_form`, `render_association`, `render_association_entry_forms_preserving_heads` |
| `control_flow` | `strings` | `render_input_like_expr` |
| `functional_ops` | `arithmetic` | `expr_as_exact_integer`, `expr_as_exact_rational` |
| `functional_ops` | `associations` | `association_entries_from_expr`, `missing_key_absent_expr`, `render_association` |
| `functional_ops` | `structure_ops` | `exact_expr_at_path`, `exact_position_paths_from_expr`, `replace_exact_path` |
| `integer_digits` | `arithmetic` | `expr_as_exact_integer` |
| `number_theory` | `arithmetic` | `expr_as_exact_integer`, `expr_as_exact_rational` |
| `polynomials` | `arithmetic` | `expr_as_exact_integer`, `expr_as_exact_rational` |
| `strings` | `arithmetic` | `expr_as_exact_rational` |
| `strings` | `associations` | `association_entries_from_expr`, `render_association` |
| `structure_ops` | `arithmetic` | `expr_as_exact_integer` |
| `structure_ops` | `associations` | `association_entries_from_expr`, `association_entry_forms_from_expr`, `association_position_and_value_by_key`, `list_items`, `missing_key_absent_expr`, `render_association`, `render_association_entry_forms_preserving_heads` |
| `rewrite_builtins` | `associations` | `association_entries_from_expr`, `render_association` |

## Per-family details
### arithmetic
- raw log: `/tmp/cdx_brief_098_raw_arithmetic.log`
- trivial external: `Expr`(rum_core), `BigInt`(rum_core), `Rational`(rum_core), `Atom`(rum_core)
- runtime/kernel modules: expr_helpers(2); expr_runtime_helpers(1); metadata(2); model(1); session_state(1)
- `expr_helpers`: `is_negative_one_expr`, `system_expr`
- `expr_runtime_helpers`: `is_system_symbol_name`
- `metadata`: `Attribute`, `AttributeSet`
- `model`: `Session`
- `session_state`: `EvalError`
- same-family local: -
- none
- other-family coupling: none

### associations
- raw log: `/tmp/cdx_brief_098_raw_associations.log`
- trivial external: `Expr`(rum_core), `BigInt`(rum_core), `heads`(rum_core), `Atom`(rum_core), `Symbol`(rum_core)
- runtime/kernel modules: definition_analysis_helpers(1); expr_helpers(4); expr_runtime_helpers(4); message_selector_parsing(1); model(1); session_state(1)
- `definition_analysis_helpers`: `DefinitionAssignmentTarget`
- `expr_helpers`: `expr_as_non_negative_usize`, `expr_as_signed_isize`, `expr_as_symbol`, `system_expr`
- `expr_runtime_helpers`: `call_items_with_head`, `compare_expr_canonical`, `is_known_system_symbol`, `is_system_symbol_name`
- `message_selector_parsing`: `truthiness_symbol`
- `model`: `Session`
- `session_state`: `EvalError`
- same-family local: associations/aggregate(1); associations/data(40); associations/grouping(1); associations/missing(1); associations/resumable(1)
- `associations/aggregate`: `exact_argument_count_message`
- `associations/data`: `AssociationEntryForm`, `KeyedEntriesListError`, `association_entries_from_expr`, `association_entry_form_expr`, `association_entry_forms_from_expr`, `association_rule_exprs_preserving_heads`, `increment_count`, `invalid_association_message`, `invalid_association_or_list_message`, `invalid_association_or_rule_list_message`, `invalid_association_or_rule_message`, `invalid_association_update_spec_message`, `invalid_keyed_collection_list_message`, `invalid_keyed_collection_outer_list_message`, `invalid_rule_message`, `keyed_entries_from_expr`, `keyed_entry_forms_from_expr`, `keyed_operator_form_slot_first`, `keyed_operator_form_slot_second`, `list_expected_message`, `list_items`, `missing_immediate_value_message`, `nonempty_list_message`, `normalize_key_value_pattern_spec`, `parse_association_entry`, `parse_association_entry_form`, `parse_association_entry_forms_deep`, `parse_association_update_spec_forms`, `parse_key_spec`, `parse_keyed_entries_list_expr_with_diagnostics`, `parse_keyed_entry_forms_list_expr_with_diagnostics`, `predicate_bool_expr`, `protected_tag_write_message`, `render_association`, `render_association_entry_forms_preserving_heads`, `render_association_preserving_rule_heads`, `rule_list_entries_from_expr`, `same_length_message`, `single_rule_or_keyed_entries_from_expr`, `single_rule_or_keyed_entry_forms_from_expr`
- `associations/grouping`: `GroupBySpec`
- `associations/missing`: `is_missing_expr`
- `associations/resumable`: `one_or_two_argument_count_message`
- other-family coupling:
  - `structure_ops`: `exact_expr_at_path` (structure_ops), `exact_position_paths_from_part_selector_exprs` (structure_ops), `replace_exact_path` (structure_ops)

### calculus
- raw log: `/tmp/cdx_brief_098_raw_calculus.log`
- trivial external: `Expr`(rum_core), `BigInt`(rum_core), `Rational`(rum_core), `Atom`(rum_core)
- runtime/kernel modules: expr_helpers(3); expr_runtime_helpers(1); model(1); session_state(1)
- `expr_helpers`: `expr_as_non_negative_usize`, `expr_as_symbol`, `system_expr`
- `expr_runtime_helpers`: `is_system_symbol_name`
- `model`: `Session`
- `session_state`: `EvalError`
- same-family local: -
- none
- other-family coupling:
  - `arithmetic`: `expr_as_exact_rational` (arithmetic)
  - `polynomials`: `parse_indeterminate_expr` (polynomials)

### combinatorics
- raw log: `/tmp/cdx_brief_098_raw_combinatorics.log`
- trivial external: `Expr`(rum_core), `BigInt`(rum_core), `Rational`(rum_core)
- runtime/kernel modules: expr_helpers(1); model(1); session_state(1)
- `expr_helpers`: `system_expr`
- `model`: `Session`
- `session_state`: `EvalError`
- same-family local: -
- none
- other-family coupling:
  - `arithmetic`: `expr_as_exact_integer` (arithmetic)

### control_flow
- raw log: `/tmp/cdx_brief_098_raw_control_flow.log`
- trivial external: `Expr`(rum_core), `Symbol`(rum_core), `heads`(rum_core), `Atom`(rum_core), `BigInt`(rum_core), `SharedExpr`(rum_core), `SharedArgs`(rum_core), `SYSTEM_CONTEXT`(rum_core)
- runtime/kernel modules: builtin_catalog(1); expr_helpers(5); expr_runtime_helpers(2); message_catalog(1); message_format(2); message_selector_parsing(3); model(6); profiling(3); session_state(2)
- `builtin_catalog`: `default_system_attribute_set`
- `expr_helpers`: `expr_as_signed_isize`, `expr_as_symbol`, `is_negative_one_expr`, `resolve_unqualified_symbol`, `system_expr`
- `expr_runtime_helpers`: `compare_expr_canonical`, `is_system_symbol_name`
- `message_catalog`: `message_name_key_from_expr`
- `message_format`: `format_message_template_with_named`, `runtime_message_name_expr`
- `message_selector_parsing`: `MessageSelectorNormalizationError`, `parse_message_selectors`, `truthiness_symbol`
- `model`: `MessageNameKey`, `MessageSelector`, `RuntimeMessage`, `Session`, `SymbolDefinition`, `SymbolExprMap`
- `profiling`: `RuntimeProfileOutcome`, `record_profile_runtime_path_timing`, `runtime_path_timing_start`
- `session_state`: `BlockSymbolState`, `EvalError`
- same-family local: control_flow/confirmation(15); control_flow/status(1)
- `control_flow/confirmation`: `ConfirmQuietSelectors`, `build_confirm_assert_failure`, `build_confirm_by_failure`, `build_confirm_failure`, `build_confirm_failure_from_failure`, `build_confirm_match_failure`, `build_confirm_quiet_failure`, `confirm_quiet_matched_message`, `confirmation_tag_from_args`, `evaluate_optional_confirmation_info`, `fresh_internal_enclose_tag_expr`, `is_confirmation_failure_like`, `normalize_confirm_quiet_selectors`, `rewrite_untagged_confirms`, `suppress_confirm_quiet_messages`
- `control_flow/status`: `is_failure_expr`
- other-family coupling:
  - `arithmetic`: `expr_as_exact_integer` (arithmetic)
  - `associations`: `AssociationEntryForm` (associations/data), `association_entries_from_expr` (associations/data), `association_entry_forms_from_expr` (associations/data), `association_value_by_key` (associations/data), `is_missing_expr` (associations/missing), `list_items` (associations/data), `parse_association_entry_form` (associations/data), `render_association` (associations/data), `render_association_entry_forms_preserving_heads` (associations/data)
  - `strings`: `render_input_like_expr` (strings)

### functional_ops
- raw log: `/tmp/cdx_brief_098_raw_functional_ops.log`
- trivial external: `Expr`(rum_core), `BigInt`(rum_core), `Atom`(rum_core), `Symbol`(rum_core), `heads`(rum_core), `Rational`(rum_core)
- runtime/kernel modules: expr_helpers(4); expr_runtime_helpers(2); message_selector_parsing(1); model(2); parse_and_message_helpers(3); session_state(3)
- `expr_helpers`: `expr_as_non_negative_usize`, `expr_as_signed_isize`, `expr_as_symbol`, `system_expr`
- `expr_runtime_helpers`: `call_items_with_head`, `is_system_symbol_name`
- `message_selector_parsing`: `truthiness_symbol`
- `model`: `CasesLevelSpec`, `Session`
- `parse_and_message_helpers`: `parse_heads_option`, `parse_heads_option_with_default`, `parse_standard_level_spec`
- `session_state`: `EvalError`, `ThreadArgSpec`, `ThreadingDispatch`
- same-family local: functional_ops(15); functional_ops/apply_map/normalize(4); functional_ops/arrays/spec(2); functional_ops/arrays/tensor(4); functional_ops/sequence/fold(3)
- `functional_ops`: `NestWhileTestArity`, `expr_as_positive_usize`, `extend_isize_path`, `function_body_operator`, `generated_expr_nodes_memory_proxy_bytes`, `nest_operator_form`, `nest_while_test_args`, `position_expr_from_path`, `slot_expr`, `unary_operator_form`, `unary_operator_form_slot_first`, `unary_operator_form_with_tail`, `unary_operator_form_with_tails`, `variadic_operator_form`, `variadic_operator_form_with_suffix`
- `functional_ops/apply_map/normalize`: `ApplyFlavor`, `MapTraversalFlavor`, `ScanCollection`, `TraversalConfig`
- `functional_ops/arrays/spec`: `ArrayReduceSpec`, `TensorNode`
- `functional_ops/arrays/tensor`: `parse_tensor_node`, `rebuild_tensor_expr`, `tensor_flat_offset`, `tensor_shape`
- `functional_ops/sequence/fold`: `FoldWhileConfig`, `iterable_items`, `parse_fold_while_config`
- other-family coupling:
  - `arithmetic`: `expr_as_exact_integer` (arithmetic), `expr_as_exact_rational` (arithmetic)
  - `associations`: `association_entries_from_expr` (associations/data), `missing_key_absent_expr` (associations/basic), `render_association` (associations/data)
  - `structure_ops`: `exact_expr_at_path` (structure_ops), `exact_position_paths_from_expr` (structure_ops), `replace_exact_path` (structure_ops)

### integer_digits
- raw log: `/tmp/cdx_brief_098_raw_integer_digits.log`
- trivial external: `Expr`(rum_core), `BigInt`(rum_core), `Atom`(rum_core)
- runtime/kernel modules: expr_helpers(1); expr_runtime_helpers(1); model(1); session_state(3)
- `expr_helpers`: `system_expr`
- `expr_runtime_helpers`: `is_system_symbol_name`
- `model`: `Session`
- `session_state`: `EvalError`, `ThreadArgSpec`, `ThreadingDispatch`
- same-family local: -
- none
- other-family coupling:
  - `arithmetic`: `expr_as_exact_integer` (arithmetic)

### logic
- raw log: `/tmp/cdx_brief_098_raw_logic.log`
- trivial external: `Rational`(rum_core), `Expr`(rum_core), `heads`(rum_core), `Atom`(rum_core)
- runtime/kernel modules: expr_helpers(1); expr_runtime_helpers(1); message_selector_parsing(1); model(1); session_state(1)
- `expr_helpers`: `system_expr`
- `expr_runtime_helpers`: `is_system_symbol_name`
- `message_selector_parsing`: `truthiness_symbol`
- `model`: `Session`
- `session_state`: `EvalError`
- same-family local: -
- none
- other-family coupling: none

### number_theory
- raw log: `/tmp/cdx_brief_098_raw_number_theory.log`
- trivial external: `Expr`(rum_core), `BigInt`(rum_core), `Atom`(rum_core), `Rational`(rum_core)
- runtime/kernel modules: expr_helpers(1); model(1); session_state(3)
- `expr_helpers`: `system_expr`
- `model`: `Session`
- `session_state`: `EvalError`, `ThreadArgSpec`, `ThreadingDispatch`
- same-family local: -
- none
- other-family coupling:
  - `arithmetic`: `expr_as_exact_integer` (arithmetic), `expr_as_exact_rational` (arithmetic)

### polynomials
- raw log: `/tmp/cdx_brief_098_raw_polynomials.log`
- trivial external: `Rational`(rum_core), `BigInt`(rum_core), `BTreeSet`(std), `Expr`(rum_core), `heads`(rum_core), `Atom`(rum_core), `SYSTEM_CONTEXT`(rum_core)
- runtime/kernel modules: expr_helpers(3); expr_runtime_helpers(2); model(1); session_state(1)
- `expr_helpers`: `expr_as_non_negative_usize`, `expr_as_symbol`, `system_expr`
- `expr_runtime_helpers`: `compare_expr_canonical`, `is_system_symbol_name`
- `model`: `Session`
- `session_state`: `EvalError`
- same-family local: polynomials(1); polynomials/core(5); polynomials/core/factors(16)
- `polynomials`: `parse_indeterminate_expr`
- `polynomials/core`: `Factor`, `Monomial`, `Polynomial`, `Term`, `VariableBasis`
- `polynomials/core/factors`: `coefficient_after_exact_monomial_power`, `compare_terms`, `constant`, `divide_by_monomial`, `divide_by_term`, `exact_multiplicity_in_term`, `exponent_for_factor`, `from_expr`, `from_polynomials`, `indeterminate`, `is_one`, `multiply`, `normalized`, `one`, `to_expr`, `total_degree`
- other-family coupling:
  - `arithmetic`: `expr_as_exact_integer` (arithmetic), `expr_as_exact_rational` (arithmetic)

### predicates
- raw log: `/tmp/cdx_brief_098_raw_predicates.log`
- trivial external: `Expr`(rum_core), `heads`(rum_core), `Atom`(rum_core), `BigInt`(rum_core), `Rational`(rum_core)
- runtime/kernel modules: expr_helpers(2); expr_runtime_helpers(2); message_selector_parsing(1); model(3); parse_and_message_helpers(1); session_state(1); tree_walkers/level(1)
- `expr_helpers`: `expr_as_non_negative_usize`, `system_expr`
- `expr_runtime_helpers`: `compare_expr_canonical`, `is_system_symbol_name`
- `message_selector_parsing`: `truthiness_symbol`
- `model`: `CasesLevelSpec`, `CasesTraversalConfig`, `Session`
- `parse_and_message_helpers`: `parse_standard_level_spec`
- `session_state`: `EvalError`
- `tree_walkers/level`: `collect_level_parts_owned`
- same-family local: predicates(14)
- `predicates`: `array_shape`, `compare_exact_numeric`, `compare_exact_order_subset`, `compare_lexicographic_order_subset`, `exact_integer_value`, `exact_numeric_value`, `expr_counts`, `expr_has_system_head`, `expr_set`, `list_items`, `order_result_expr`, `predicate_bool_expr`, `string_atom_value`, `two_list_args`
- other-family coupling: none

### strings
- raw log: `/tmp/cdx_brief_098_raw_strings.log`
- trivial external: `Expr`(rum_core), `parse_expr`(rum_syntax), `BigInt`(rum_core), `Symbol`(rum_core), `BTreeSet`(std), `heads`(rum_core), `Rational`(rum_core), `fs`(std), `Path`(std), `BTreeMap`(std), `Atom`(rum_core), `PathBuf`(std)
- runtime/kernel modules: expr_helpers(6); expr_runtime_helpers(2); lexical_substitution(1); message_selector_parsing(1); model(2); session_state(1)
- `expr_helpers`: `expr_as_non_negative_usize`, `expr_as_signed_isize`, `expr_as_string`, `expr_as_string_list`, `expr_as_symbol`, `system_expr`
- `expr_runtime_helpers`: `call_items_with_head`, `is_system_symbol_name`
- `lexical_substitution`: `substitute_lexical_bindings`
- `message_selector_parsing`: `truthiness_symbol`
- `model`: `Session`, `SymbolExprMap`
- `session_state`: `EvalError`
- same-family local: strings(8); strings/patterns(22); strings/patterns/matcher(1); strings/patterns/regex(1); strings/similarity(3); strings/similarity/distance(2); strings/template(1)
- `strings`: `bool_expr`, `is_string_whitespace`, `list_expr`, `string_apply_value`, `string_list_expr`, `string_slot_expr`, `string_unary_operator_form`, `string_unary_operator_form_slot_first`
- `strings/patterns`: `ExactStringCaseOutput`, `ExactStringCaseSpec`, `ExactStringReplacementRule`, `ExactStringReplacementValue`, `StringBoundarySpec`, `StringCountConfig`, `StringCountOverlaps`, `StringDeleteConfig`, `StringPattern`, `StringPatternMatchConfig`, `StringSplitLimit`, `parse_exact_string_case_spec`, `parse_exact_string_replacement_rules`, `parse_string_boundary_spec`, `parse_string_count_options`, `parse_string_delete_options`, `parse_string_match_config`, `parse_string_match_limit_and_options`, `parse_string_pattern`, `parse_string_patterns`, `parse_string_position_args`, `parse_string_split_limit`
- `strings/patterns/matcher`: `StringPatternMatchState`
- `strings/patterns/regex`: `regex_pattern_match_ends`
- `strings/similarity`: `NearestDistanceFunction`, `NearestResultSpec`, `NearestSelectionSpec`
- `strings/similarity/distance`: `hamming_distance`, `levenshtein_distance`
- `strings/template`: `string_form_value`
- other-family coupling:
  - `arithmetic`: `expr_as_exact_rational` (arithmetic)
  - `associations`: `association_entries_from_expr` (associations/data), `render_association` (associations/data)

### structure_ops
- raw log: `/tmp/cdx_brief_098_raw_structure_ops.log`
- trivial external: `Expr`(rum_core), `heads`(rum_core), `BigInt`(rum_core), `SharedArgs`(rum_core), `BTreeSet`(std), `BTreeMap`(std), `Atom`(rum_core)
- runtime/kernel modules: binding_substitution(1); expr_helpers(4); expr_runtime_helpers(1); model(4); parse_and_message_helpers(3); session_state(1); suspension/core(1)
- `binding_substitution`: `substitute_rule_bindings`
- `expr_helpers`: `expr_as_non_negative_usize`, `expr_as_signed_isize`, `expr_as_string`, `system_expr`
- `expr_runtime_helpers`: `is_system_symbol_name`
- `model`: `RewriteRule`, `RuleApplicationSource`, `Session`, `SymbolExprMap`
- `parse_and_message_helpers`: `HeadsOptionClassification`, `classify_heads_option_expr`, `parse_heads_option`
- `session_state`: `EvalError`
- `suspension/core`: `ContinuationValue`
- same-family local: structure_ops(9); structure_ops/exact_paths(4); structure_ops/replace(6); structure_ops/selectors(7); structure_ops/selectors/apply(1); structure_ops/selectors/extract(1); structure_ops/selectors/spans(1)
- `structure_ops`: `exact_expr_at_path`, `exact_position_paths_from_expr`, `is_empty_plain_list_expr`, `is_plain_list_expr`, `replace_exact_path`, `structural_depth`, `structural_head_expr`, `structural_leaf_count`, `structural_length`
- `structure_ops/exact_paths`: `collect_exact_paths_for_position_spec`, `exact_part_component`, `normalize_structural_arg_position`, `select_exact_part`
- `structure_ops/replace`: `ReplacePartHeadsMode`, `ReplacePartPositionSpec`, `ReplacePartRule`, `ReplacePartTarget`, `ReplacePlanAction`, `ReplacePlanNode`
- `structure_ops/selectors`: `AtomicPartSpec`, `ExtractPositionSpec`, `PartSelectionFailure`, `PartSelector`, `PartSpanSpec`, `SelectedPartShape`, `SpanEndpoint`
- `structure_ops/selectors/apply`: `apply_part_selectors`
- `structure_ops/selectors/extract`: `parse_single_position_spec`
- `structure_ops/selectors/spans`: `select_positioned_parts_for_span`
- other-family coupling:
  - `arithmetic`: `expr_as_exact_integer` (arithmetic)
  - `associations`: `association_entries_from_expr` (associations/data), `association_entry_forms_from_expr` (associations/data), `association_position_and_value_by_key` (associations/data), `list_items` (associations/data), `missing_key_absent_expr` (associations/basic), `render_association` (associations/data), `render_association_entry_forms_preserving_heads` (associations/data)

### rewrite_builtins
- raw log: `/tmp/cdx_brief_098_raw_rewrite_builtins.log`
- trivial external: `Expr`(rum_core), `BigInt`(rum_core)
- runtime/kernel modules: binding_substitution(1); definition_catalog/traversal(1); expr_helpers(4); expr_runtime_helpers(1); function_and_subst_helpers(1); matcher_and_symbols(3); metadata(2); model(8); parse_and_message_helpers(1); pattern_matching/outcome(1); session_state(2); suspension/core(1)
- `binding_substitution`: `substitute_rule_bindings_for_rhs`
- `definition_catalog/traversal`: `CasesTraversalResolution`
- `expr_helpers`: `expr_as_non_negative_usize`, `expr_as_symbol`, `expr_is_self_evaluating_atom`, `system_expr`
- `expr_runtime_helpers`: `is_system_symbol_name`
- `function_and_subst_helpers`: `strip_hold_pattern`
- `matcher_and_symbols`: `match_blank`, `matcher_head_has_attribute`, `pattern_blank_head`
- `metadata`: `Attribute`, `AttributeSet`
- `model`: `CasesCollectionOutcome`, `CasesLevelSpec`, `CasesSpec`, `CasesTraversalConfig`, `RewriteRule`, `RuleApplicationSource`, `Session`, `SymbolExprMap`
- `parse_and_message_helpers`: `parse_standard_level_spec`
- `pattern_matching/outcome`: `CompiledMatchOutcome`
- `session_state`: `EvalError`, `MatcherNode`
- `suspension/core`: `ContinuationValue`
- same-family local: rewrite_builtins(3)
- `rewrite_builtins`: `DeleteCasesOutcome`, `DeleteCasesResult`, `ReplaceOutcome`
- other-family coupling:
  - `associations`: `association_entries_from_expr` (associations/data), `render_association` (associations/data)

## Promotion candidates
Runtime-owned non-trivial items used by at least one audited family, sorted by family fan-out.
| item | kind | defining module | families | fan-out |
|---|---|---|---:|---:|
| `EvalError` | type | `session_state` | `arithmetic`, `associations`, `calculus`, `combinatorics`, `control_flow`, `functional_ops`, `integer_digits`, `logic`, `number_theory`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 14 |
| `Session` | type | `model` | `arithmetic`, `associations`, `calculus`, `combinatorics`, `control_flow`, `functional_ops`, `integer_digits`, `logic`, `number_theory`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 14 |
| `system_expr` | fn | `expr_helpers` | `arithmetic`, `associations`, `calculus`, `combinatorics`, `control_flow`, `functional_ops`, `integer_digits`, `logic`, `number_theory`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 14 |
| `is_system_symbol_name` | fn | `expr_runtime_helpers` | `arithmetic`, `associations`, `calculus`, `control_flow`, `functional_ops`, `integer_digits`, `logic`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 12 |
| `expr_as_non_negative_usize` | fn | `expr_helpers` | `associations`, `calculus`, `functional_ops`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 8 |
| `expr_as_exact_integer` | fn | `arithmetic` | `combinatorics`, `control_flow`, `functional_ops`, `integer_digits`, `number_theory`, `polynomials`, `structure_ops` | 7 |
| `expr_as_symbol` | fn | `expr_helpers` | `associations`, `calculus`, `control_flow`, `functional_ops`, `polynomials`, `rewrite_builtins`, `strings` | 7 |
| `association_entries_from_expr` | fn | `associations/data` | `associations`, `control_flow`, `functional_ops`, `rewrite_builtins`, `strings`, `structure_ops` | 6 |
| `render_association` | fn | `associations/data` | `associations`, `control_flow`, `functional_ops`, `rewrite_builtins`, `strings`, `structure_ops` | 6 |
| `truthiness_symbol` | fn | `message_selector_parsing` | `associations`, `control_flow`, `functional_ops`, `logic`, `predicates`, `strings` | 6 |
| `expr_as_exact_rational` | fn | `arithmetic` | `calculus`, `functional_ops`, `number_theory`, `polynomials`, `strings` | 5 |
| `expr_as_signed_isize` | fn | `expr_helpers` | `associations`, `control_flow`, `functional_ops`, `strings`, `structure_ops` | 5 |
| `SymbolExprMap` | type | `model` | `control_flow`, `rewrite_builtins`, `strings`, `structure_ops` | 4 |
| `compare_expr_canonical` | fn | `expr_runtime_helpers` | `associations`, `control_flow`, `polynomials`, `predicates` | 4 |
| `list_items` | fn | `associations/data` | `associations`, `control_flow`, `predicates`, `structure_ops` | 4 |
| `CasesLevelSpec` | type | `model` | `functional_ops`, `predicates`, `rewrite_builtins` | 3 |
| `ThreadArgSpec` | type | `session_state` | `functional_ops`, `integer_digits`, `number_theory` | 3 |
| `ThreadingDispatch` | type | `session_state` | `functional_ops`, `integer_digits`, `number_theory` | 3 |
| `association_entry_forms_from_expr` | fn | `associations/data` | `associations`, `control_flow`, `structure_ops` | 3 |
| `call_items_with_head` | fn | `expr_runtime_helpers` | `associations`, `functional_ops`, `strings` | 3 |
| `exact_expr_at_path` | fn | `structure_ops` | `associations`, `functional_ops`, `structure_ops` | 3 |
| `parse_standard_level_spec` | fn | `parse_and_message_helpers` | `functional_ops`, `predicates`, `rewrite_builtins` | 3 |
| `render_association_entry_forms_preserving_heads` | fn | `associations/data` | `associations`, `control_flow`, `structure_ops` | 3 |
| `replace_exact_path` | fn | `structure_ops` | `associations`, `functional_ops`, `structure_ops` | 3 |
| `AssociationEntryForm` | type | `associations/data` | `associations`, `control_flow` | 2 |
| `Attribute` | type | `metadata` | `arithmetic`, `rewrite_builtins` | 2 |
| `AttributeSet` | type | `metadata` | `arithmetic`, `rewrite_builtins` | 2 |
| `CasesTraversalConfig` | type | `model` | `predicates`, `rewrite_builtins` | 2 |
| `ContinuationValue` | type | `suspension/core` | `rewrite_builtins`, `structure_ops` | 2 |
| `RewriteRule` | type | `model` | `rewrite_builtins`, `structure_ops` | 2 |
| `RuleApplicationSource` | type | `model` | `rewrite_builtins`, `structure_ops` | 2 |
| `exact_position_paths_from_expr` | fn | `structure_ops` | `functional_ops`, `structure_ops` | 2 |
| `expr_as_string` | fn | `expr_helpers` | `strings`, `structure_ops` | 2 |
| `is_missing_expr` | fn | `associations/missing` | `associations`, `control_flow` | 2 |
| `is_negative_one_expr` | fn | `expr_helpers` | `arithmetic`, `control_flow` | 2 |
| `missing_key_absent_expr` | fn | `associations/basic` | `functional_ops`, `structure_ops` | 2 |
| `parse_association_entry_form` | fn | `associations/data` | `associations`, `control_flow` | 2 |
| `parse_heads_option` | fn | `parse_and_message_helpers` | `functional_ops`, `structure_ops` | 2 |
| `parse_indeterminate_expr` | fn | `polynomials` | `calculus`, `polynomials` | 2 |
| `predicate_bool_expr` | fn | `associations/data` | `associations`, `predicates` | 2 |
| `ApplyFlavor` | type | `functional_ops/apply_map/normalize` | `functional_ops` | 1 |
| `ArrayReduceSpec` | type | `functional_ops/arrays/spec` | `functional_ops` | 1 |
| `AtomicPartSpec` | type | `structure_ops/selectors` | `structure_ops` | 1 |
| `BlockSymbolState` | type | `session_state` | `control_flow` | 1 |
| `CasesCollectionOutcome` | type | `model` | `rewrite_builtins` | 1 |
| `CasesSpec` | type | `model` | `rewrite_builtins` | 1 |
| `CasesTraversalResolution` | type | `definition_catalog/traversal` | `rewrite_builtins` | 1 |
| `CompiledMatchOutcome` | type | `pattern_matching/outcome` | `rewrite_builtins` | 1 |
| `ConfirmQuietSelectors` | type | `control_flow/confirmation` | `control_flow` | 1 |
| `DefinitionAssignmentTarget` | type | `definition_analysis_helpers` | `associations` | 1 |
| `DeleteCasesOutcome` | type | `rewrite_builtins` | `rewrite_builtins` | 1 |
| `DeleteCasesResult` | type | `rewrite_builtins` | `rewrite_builtins` | 1 |
| `ExactStringCaseOutput` | type | `strings/patterns` | `strings` | 1 |
| `ExactStringCaseSpec` | type | `strings/patterns` | `strings` | 1 |
| `ExactStringReplacementRule` | type | `strings/patterns` | `strings` | 1 |
| `ExactStringReplacementValue` | type | `strings/patterns` | `strings` | 1 |
| `ExtractPositionSpec` | type | `structure_ops/selectors` | `structure_ops` | 1 |
| `Factor` | type | `polynomials/core` | `polynomials` | 1 |
| `FoldWhileConfig` | type | `functional_ops/sequence/fold` | `functional_ops` | 1 |
| `GroupBySpec` | type | `associations/grouping` | `associations` | 1 |
| `HeadsOptionClassification` | type | `parse_and_message_helpers` | `structure_ops` | 1 |
| `KeyedEntriesListError` | type | `associations/data` | `associations` | 1 |
| `MapTraversalFlavor` | type | `functional_ops/apply_map/normalize` | `functional_ops` | 1 |
| `MatcherNode` | type | `session_state` | `rewrite_builtins` | 1 |
| `MessageNameKey` | type | `model` | `control_flow` | 1 |
| `MessageSelector` | type | `model` | `control_flow` | 1 |
| `MessageSelectorNormalizationError` | type | `message_selector_parsing` | `control_flow` | 1 |
| `Monomial` | type | `polynomials/core` | `polynomials` | 1 |
| `NearestDistanceFunction` | type | `strings/similarity` | `strings` | 1 |
| `NearestResultSpec` | type | `strings/similarity` | `strings` | 1 |
| `NearestSelectionSpec` | type | `strings/similarity` | `strings` | 1 |
| `NestWhileTestArity` | type | `functional_ops` | `functional_ops` | 1 |
| `PartSelectionFailure` | type | `structure_ops/selectors` | `structure_ops` | 1 |
| `PartSelector` | type | `structure_ops/selectors` | `structure_ops` | 1 |
| `PartSpanSpec` | type | `structure_ops/selectors` | `structure_ops` | 1 |
| `Polynomial` | type | `polynomials/core` | `polynomials` | 1 |
| `ReplaceOutcome` | type | `rewrite_builtins` | `rewrite_builtins` | 1 |
| `ReplacePartHeadsMode` | type | `structure_ops/replace` | `structure_ops` | 1 |
| `ReplacePartPositionSpec` | type | `structure_ops/replace` | `structure_ops` | 1 |
| `ReplacePartRule` | type | `structure_ops/replace` | `structure_ops` | 1 |
| `ReplacePartTarget` | type | `structure_ops/replace` | `structure_ops` | 1 |
| `ReplacePlanAction` | type | `structure_ops/replace` | `structure_ops` | 1 |
| `ReplacePlanNode` | type | `structure_ops/replace` | `structure_ops` | 1 |
| `RuntimeMessage` | type | `model` | `control_flow` | 1 |
| `RuntimeProfileOutcome` | type | `profiling` | `control_flow` | 1 |
| `ScanCollection` | type | `functional_ops/apply_map/normalize` | `functional_ops` | 1 |
| `SelectedPartShape` | type | `structure_ops/selectors` | `structure_ops` | 1 |
| `SpanEndpoint` | type | `structure_ops/selectors` | `structure_ops` | 1 |
| `StringBoundarySpec` | type | `strings/patterns` | `strings` | 1 |
| `StringCountConfig` | type | `strings/patterns` | `strings` | 1 |
| `StringCountOverlaps` | type | `strings/patterns` | `strings` | 1 |
| `StringDeleteConfig` | type | `strings/patterns` | `strings` | 1 |
| `StringPattern` | type | `strings/patterns` | `strings` | 1 |
| `StringPatternMatchConfig` | type | `strings/patterns` | `strings` | 1 |
| `StringPatternMatchState` | type | `strings/patterns/matcher` | `strings` | 1 |
| `StringSplitLimit` | type | `strings/patterns` | `strings` | 1 |
| `SymbolDefinition` | type | `model` | `control_flow` | 1 |
| `TensorNode` | type | `functional_ops/arrays/spec` | `functional_ops` | 1 |
| `Term` | type | `polynomials/core` | `polynomials` | 1 |
| `TraversalConfig` | type | `functional_ops/apply_map/normalize` | `functional_ops` | 1 |
| `VariableBasis` | type | `polynomials/core` | `polynomials` | 1 |
| `apply_part_selectors` | fn | `structure_ops/selectors/apply` | `structure_ops` | 1 |
| `array_shape` | fn | `predicates` | `predicates` | 1 |
| `association_entry_form_expr` | fn | `associations/data` | `associations` | 1 |
| `association_position_and_value_by_key` | fn | `associations/data` | `structure_ops` | 1 |
| `association_rule_exprs_preserving_heads` | fn | `associations/data` | `associations` | 1 |
| `association_value_by_key` | fn | `associations/data` | `control_flow` | 1 |
| `bool_expr` | fn | `strings` | `strings` | 1 |
| `build_confirm_assert_failure` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `build_confirm_by_failure` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `build_confirm_failure` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `build_confirm_failure_from_failure` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `build_confirm_match_failure` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `build_confirm_quiet_failure` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `classify_heads_option_expr` | fn | `parse_and_message_helpers` | `structure_ops` | 1 |
| `coefficient_after_exact_monomial_power` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `collect_exact_paths_for_position_spec` | fn | `structure_ops/exact_paths` | `structure_ops` | 1 |
| `collect_level_parts_owned` | fn | `tree_walkers/level` | `predicates` | 1 |
| `compare_exact_numeric` | fn | `predicates` | `predicates` | 1 |
| `compare_exact_order_subset` | fn | `predicates` | `predicates` | 1 |
| `compare_lexicographic_order_subset` | fn | `predicates` | `predicates` | 1 |
| `compare_terms` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `confirm_quiet_matched_message` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `confirmation_tag_from_args` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `constant` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `default_system_attribute_set` | fn | `builtin_catalog` | `control_flow` | 1 |
| `divide_by_monomial` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `divide_by_term` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `evaluate_optional_confirmation_info` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `exact_argument_count_message` | fn | `associations/aggregate` | `associations` | 1 |
| `exact_integer_value` | fn | `predicates` | `predicates` | 1 |
| `exact_multiplicity_in_term` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `exact_numeric_value` | fn | `predicates` | `predicates` | 1 |
| `exact_part_component` | fn | `structure_ops/exact_paths` | `structure_ops` | 1 |
| `exact_position_paths_from_part_selector_exprs` | fn | `structure_ops` | `associations` | 1 |
| `exponent_for_factor` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `expr_as_positive_usize` | fn | `functional_ops` | `functional_ops` | 1 |
| `expr_as_string_list` | fn | `expr_helpers` | `strings` | 1 |
| `expr_counts` | fn | `predicates` | `predicates` | 1 |
| `expr_has_system_head` | fn | `predicates` | `predicates` | 1 |
| `expr_is_self_evaluating_atom` | fn | `expr_helpers` | `rewrite_builtins` | 1 |
| `expr_set` | fn | `predicates` | `predicates` | 1 |
| `extend_isize_path` | fn | `functional_ops` | `functional_ops` | 1 |
| `format_message_template_with_named` | fn | `message_format` | `control_flow` | 1 |
| `fresh_internal_enclose_tag_expr` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `from_expr` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `from_polynomials` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `function_body_operator` | fn | `functional_ops` | `functional_ops` | 1 |
| `generated_expr_nodes_memory_proxy_bytes` | fn | `functional_ops` | `functional_ops` | 1 |
| `hamming_distance` | fn | `strings/similarity/distance` | `strings` | 1 |
| `increment_count` | fn | `associations/data` | `associations` | 1 |
| `indeterminate` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `invalid_association_message` | fn | `associations/data` | `associations` | 1 |
| `invalid_association_or_list_message` | fn | `associations/data` | `associations` | 1 |
| `invalid_association_or_rule_list_message` | fn | `associations/data` | `associations` | 1 |
| `invalid_association_or_rule_message` | fn | `associations/data` | `associations` | 1 |
| `invalid_association_update_spec_message` | fn | `associations/data` | `associations` | 1 |
| `invalid_keyed_collection_list_message` | fn | `associations/data` | `associations` | 1 |
| `invalid_keyed_collection_outer_list_message` | fn | `associations/data` | `associations` | 1 |
| `invalid_rule_message` | fn | `associations/data` | `associations` | 1 |
| `is_confirmation_failure_like` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `is_empty_plain_list_expr` | fn | `structure_ops` | `structure_ops` | 1 |
| `is_failure_expr` | fn | `control_flow/status` | `control_flow` | 1 |
| `is_known_system_symbol` | fn | `expr_runtime_helpers` | `associations` | 1 |
| `is_one` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `is_plain_list_expr` | fn | `structure_ops` | `structure_ops` | 1 |
| `is_string_whitespace` | fn | `strings` | `strings` | 1 |
| `iterable_items` | fn | `functional_ops/sequence/fold` | `functional_ops` | 1 |
| `keyed_entries_from_expr` | fn | `associations/data` | `associations` | 1 |
| `keyed_entry_forms_from_expr` | fn | `associations/data` | `associations` | 1 |
| `keyed_operator_form_slot_first` | fn | `associations/data` | `associations` | 1 |
| `keyed_operator_form_slot_second` | fn | `associations/data` | `associations` | 1 |
| `levenshtein_distance` | fn | `strings/similarity/distance` | `strings` | 1 |
| `list_expected_message` | fn | `associations/data` | `associations` | 1 |
| `list_expr` | fn | `strings` | `strings` | 1 |
| `match_blank` | fn | `matcher_and_symbols` | `rewrite_builtins` | 1 |
| `matcher_head_has_attribute` | fn | `matcher_and_symbols` | `rewrite_builtins` | 1 |
| `message_name_key_from_expr` | fn | `message_catalog` | `control_flow` | 1 |
| `missing_immediate_value_message` | fn | `associations/data` | `associations` | 1 |
| `multiply` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `nest_operator_form` | fn | `functional_ops` | `functional_ops` | 1 |
| `nest_while_test_args` | fn | `functional_ops` | `functional_ops` | 1 |
| `nonempty_list_message` | fn | `associations/data` | `associations` | 1 |
| `normalize_confirm_quiet_selectors` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `normalize_key_value_pattern_spec` | fn | `associations/data` | `associations` | 1 |
| `normalize_structural_arg_position` | fn | `structure_ops/exact_paths` | `structure_ops` | 1 |
| `normalized` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `one` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `one_or_two_argument_count_message` | fn | `associations/resumable` | `associations` | 1 |
| `order_result_expr` | fn | `predicates` | `predicates` | 1 |
| `parse_association_entry` | fn | `associations/data` | `associations` | 1 |
| `parse_association_entry_forms_deep` | fn | `associations/data` | `associations` | 1 |
| `parse_association_update_spec_forms` | fn | `associations/data` | `associations` | 1 |
| `parse_exact_string_case_spec` | fn | `strings/patterns` | `strings` | 1 |
| `parse_exact_string_replacement_rules` | fn | `strings/patterns` | `strings` | 1 |
| `parse_fold_while_config` | fn | `functional_ops/sequence/fold` | `functional_ops` | 1 |
| `parse_heads_option_with_default` | fn | `parse_and_message_helpers` | `functional_ops` | 1 |
| `parse_key_spec` | fn | `associations/data` | `associations` | 1 |
| `parse_keyed_entries_list_expr_with_diagnostics` | fn | `associations/data` | `associations` | 1 |
| `parse_keyed_entry_forms_list_expr_with_diagnostics` | fn | `associations/data` | `associations` | 1 |
| `parse_message_selectors` | fn | `message_selector_parsing` | `control_flow` | 1 |
| `parse_single_position_spec` | fn | `structure_ops/selectors/extract` | `structure_ops` | 1 |
| `parse_string_boundary_spec` | fn | `strings/patterns` | `strings` | 1 |
| `parse_string_count_options` | fn | `strings/patterns` | `strings` | 1 |
| `parse_string_delete_options` | fn | `strings/patterns` | `strings` | 1 |
| `parse_string_match_config` | fn | `strings/patterns` | `strings` | 1 |
| `parse_string_match_limit_and_options` | fn | `strings/patterns` | `strings` | 1 |
| `parse_string_pattern` | fn | `strings/patterns` | `strings` | 1 |
| `parse_string_patterns` | fn | `strings/patterns` | `strings` | 1 |
| `parse_string_position_args` | fn | `strings/patterns` | `strings` | 1 |
| `parse_string_split_limit` | fn | `strings/patterns` | `strings` | 1 |
| `parse_tensor_node` | fn | `functional_ops/arrays/tensor` | `functional_ops` | 1 |
| `pattern_blank_head` | fn | `matcher_and_symbols` | `rewrite_builtins` | 1 |
| `position_expr_from_path` | fn | `functional_ops` | `functional_ops` | 1 |
| `protected_tag_write_message` | fn | `associations/data` | `associations` | 1 |
| `rebuild_tensor_expr` | fn | `functional_ops/arrays/tensor` | `functional_ops` | 1 |
| `record_profile_runtime_path_timing` | fn | `profiling` | `control_flow` | 1 |
| `regex_pattern_match_ends` | fn | `strings/patterns/regex` | `strings` | 1 |
| `render_association_preserving_rule_heads` | fn | `associations/data` | `associations` | 1 |
| `render_input_like_expr` | fn | `strings` | `control_flow` | 1 |
| `resolve_unqualified_symbol` | fn | `expr_helpers` | `control_flow` | 1 |
| `rewrite_untagged_confirms` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `rule_list_entries_from_expr` | fn | `associations/data` | `associations` | 1 |
| `runtime_message_name_expr` | fn | `message_format` | `control_flow` | 1 |
| `runtime_path_timing_start` | fn | `profiling` | `control_flow` | 1 |
| `same_length_message` | fn | `associations/data` | `associations` | 1 |
| `select_exact_part` | fn | `structure_ops/exact_paths` | `structure_ops` | 1 |
| `select_positioned_parts_for_span` | fn | `structure_ops/selectors/spans` | `structure_ops` | 1 |
| `single_rule_or_keyed_entries_from_expr` | fn | `associations/data` | `associations` | 1 |
| `single_rule_or_keyed_entry_forms_from_expr` | fn | `associations/data` | `associations` | 1 |
| `slot_expr` | fn | `functional_ops` | `functional_ops` | 1 |
| `string_apply_value` | fn | `strings` | `strings` | 1 |
| `string_atom_value` | fn | `predicates` | `predicates` | 1 |
| `string_form_value` | fn | `strings/template` | `strings` | 1 |
| `string_list_expr` | fn | `strings` | `strings` | 1 |
| `string_slot_expr` | fn | `strings` | `strings` | 1 |
| `string_unary_operator_form` | fn | `strings` | `strings` | 1 |
| `string_unary_operator_form_slot_first` | fn | `strings` | `strings` | 1 |
| `strip_hold_pattern` | fn | `function_and_subst_helpers` | `rewrite_builtins` | 1 |
| `structural_depth` | fn | `structure_ops` | `structure_ops` | 1 |
| `structural_head_expr` | fn | `structure_ops` | `structure_ops` | 1 |
| `structural_leaf_count` | fn | `structure_ops` | `structure_ops` | 1 |
| `structural_length` | fn | `structure_ops` | `structure_ops` | 1 |
| `substitute_lexical_bindings` | fn | `lexical_substitution` | `strings` | 1 |
| `substitute_rule_bindings` | fn | `binding_substitution` | `structure_ops` | 1 |
| `substitute_rule_bindings_for_rhs` | fn | `binding_substitution` | `rewrite_builtins` | 1 |
| `suppress_confirm_quiet_messages` | fn | `control_flow/confirmation` | `control_flow` | 1 |
| `tensor_flat_offset` | fn | `functional_ops/arrays/tensor` | `functional_ops` | 1 |
| `tensor_shape` | fn | `functional_ops/arrays/tensor` | `functional_ops` | 1 |
| `to_expr` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `total_degree` | fn | `polynomials/core/factors` | `polynomials` | 1 |
| `two_list_args` | fn | `predicates` | `predicates` | 1 |
| `unary_operator_form` | fn | `functional_ops` | `functional_ops` | 1 |
| `unary_operator_form_slot_first` | fn | `functional_ops` | `functional_ops` | 1 |
| `unary_operator_form_with_tail` | fn | `functional_ops` | `functional_ops` | 1 |
| `unary_operator_form_with_tails` | fn | `functional_ops` | `functional_ops` | 1 |
| `variadic_operator_form` | fn | `functional_ops` | `functional_ops` | 1 |
| `variadic_operator_form_with_suffix` | fn | `functional_ops` | `functional_ops` | 1 |

## Top 20 promotion candidates
| item | kind | defining module | families | fan-out |
|---|---|---|---:|---:|
| `EvalError` | type | `session_state` | `arithmetic`, `associations`, `calculus`, `combinatorics`, `control_flow`, `functional_ops`, `integer_digits`, `logic`, `number_theory`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 14 |
| `Session` | type | `model` | `arithmetic`, `associations`, `calculus`, `combinatorics`, `control_flow`, `functional_ops`, `integer_digits`, `logic`, `number_theory`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 14 |
| `system_expr` | fn | `expr_helpers` | `arithmetic`, `associations`, `calculus`, `combinatorics`, `control_flow`, `functional_ops`, `integer_digits`, `logic`, `number_theory`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 14 |
| `is_system_symbol_name` | fn | `expr_runtime_helpers` | `arithmetic`, `associations`, `calculus`, `control_flow`, `functional_ops`, `integer_digits`, `logic`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 12 |
| `expr_as_non_negative_usize` | fn | `expr_helpers` | `associations`, `calculus`, `functional_ops`, `polynomials`, `predicates`, `rewrite_builtins`, `strings`, `structure_ops` | 8 |
| `expr_as_exact_integer` | fn | `arithmetic` | `combinatorics`, `control_flow`, `functional_ops`, `integer_digits`, `number_theory`, `polynomials`, `structure_ops` | 7 |
| `expr_as_symbol` | fn | `expr_helpers` | `associations`, `calculus`, `control_flow`, `functional_ops`, `polynomials`, `rewrite_builtins`, `strings` | 7 |
| `association_entries_from_expr` | fn | `associations/data` | `associations`, `control_flow`, `functional_ops`, `rewrite_builtins`, `strings`, `structure_ops` | 6 |
| `render_association` | fn | `associations/data` | `associations`, `control_flow`, `functional_ops`, `rewrite_builtins`, `strings`, `structure_ops` | 6 |
| `truthiness_symbol` | fn | `message_selector_parsing` | `associations`, `control_flow`, `functional_ops`, `logic`, `predicates`, `strings` | 6 |
| `expr_as_exact_rational` | fn | `arithmetic` | `calculus`, `functional_ops`, `number_theory`, `polynomials`, `strings` | 5 |
| `expr_as_signed_isize` | fn | `expr_helpers` | `associations`, `control_flow`, `functional_ops`, `strings`, `structure_ops` | 5 |
| `SymbolExprMap` | type | `model` | `control_flow`, `rewrite_builtins`, `strings`, `structure_ops` | 4 |
| `compare_expr_canonical` | fn | `expr_runtime_helpers` | `associations`, `control_flow`, `polynomials`, `predicates` | 4 |
| `list_items` | fn | `associations/data` | `associations`, `control_flow`, `predicates`, `structure_ops` | 4 |
| `CasesLevelSpec` | type | `model` | `functional_ops`, `predicates`, `rewrite_builtins` | 3 |
| `ThreadArgSpec` | type | `session_state` | `functional_ops`, `integer_digits`, `number_theory` | 3 |
| `ThreadingDispatch` | type | `session_state` | `functional_ops`, `integer_digits`, `number_theory` | 3 |
| `association_entry_forms_from_expr` | fn | `associations/data` | `associations`, `control_flow`, `structure_ops` | 3 |
| `call_items_with_head` | fn | `expr_runtime_helpers` | `associations`, `functional_ops`, `strings` | 3 |

## Split totals
- Runtime-owned non-trivial items used by audited families: 258
- Used by >=3 families: 24
- Used by exactly 2 families: 16
- Used by exactly 1 family: 218
- Current-source exact `pub(in crate::runtime)` item names found: 651 unique (653 definitions).
- Exact `pub(in crate::runtime)` item names not used by any audited family free-item diagnostic: 600
- Note: the brief text says 528 `pub(in crate::runtime)` symbols; this checkout scans as above. The unused count uses the measured current source, not the stale denominator.

## Axis 2: Session method calls
| method | defining module(s) | total calls | per-family counts |
|---|---|---:|---|
| `evaluate` | `resolution` | 148 | `arithmetic`:1, `associations`:28, `control_flow`:54, `functional_ops`:30, `integer_digits`:1, `logic`:4, `number_theory`:1, `predicates`:1, `rewrite_builtins`:15, `strings`:5, `structure_ops`:8 |
| `invalid_call_with_message` | `session_helpers` | 131 | `associations`:78, `calculus`:1, `control_flow`:11, `functional_ops`:1, `predicates`:3, `structure_ops`:37 |
| `invalid_call_with_exact_arg_count` | `session_helpers` | 113 | `arithmetic`:11, `combinatorics`:9, `control_flow`:5, `functional_ops`:1, `logic`:2, `number_theory`:8, `polynomials`:4, `predicates`:32, `rewrite_builtins`:3, `strings`:29, `structure_ops`:9 |
| `invalid_call_with_arg_range` | `session_helpers` | 112 | `arithmetic`:1, `control_flow`:15, `functional_ops`:36, `integer_digits`:5, `number_theory`:1, `polynomials`:3, `predicates`:3, `rewrite_builtins`:4, `strings`:25, `structure_ops`:19 |
| `suspend_with_frame` | `suspension/core` | 100 | `associations`:31, `control_flow`:18, `functional_ops`:25, `logic`:4, `predicates`:1, `rewrite_builtins`:13, `strings`:4, `structure_ops`:4 |
| `invalid_call_with_arg_form` | `session_helpers` | 67 | `control_flow`:7, `functional_ops`:48, `polynomials`:2, `predicates`:1, `rewrite_builtins`:6, `structure_ops`:3 |
| `drive_internal_value` | `suspension/core` | 41 | `control_flow`:12, `rewrite_builtins`:24, `structure_ops`:5 |
| `push_message` | `session_helpers` | 37 | `associations`:11, `control_flow`:4, `number_theory`:1, `structure_ops`:21 |
| `suspend_with_result_frame` | `suspension/core` | 17 | `control_flow`:17 |
| `enforce_active_memory_limit_for_projected_bytes` | `debug_tools/resources/constraints` | 12 | `control_flow`:1, `functional_ops`:2, `strings`:2, `structure_ops`:7 |
| `enforce_generated_element_limit_for_count` | `debug_tools/resources/constraints` | 10 | `functional_ops`:1, `integer_digits`:4, `strings`:2, `structure_ops`:3 |
| `match_expr_owned` | `pattern_matching/core` | 9 | `control_flow`:5, `rewrite_builtins`:3, `structure_ops`:1 |
| `combine_multiplicative_args` | `arithmetic` | 7 | `calculus`:3, `combinatorics`:1, `polynomials`:3 |
| `merged_option_exprs_for_system_symbol` | `options/core` | 7 | `strings`:7 |
| `enforce_generated_element_limit_for_big_count` | `debug_tools/resources/constraints` | 6 | `control_flow`:1, `functional_ops`:1, `structure_ops`:4 |
| `bump_evaluation_epoch_for_symbol` | `evaluation_cache` | 5 | `associations`:3, `control_flow`:2 |
| `combine_additive_args` | `arithmetic` | 5 | `calculus`:3, `combinatorics`:1, `polynomials`:1 |
| `normalize_symbol_target_expr` | `message_catalog` | 5 | `associations`:5 |
| `push_suspension_frame` | `suspension/core` | 5 | `control_flow`:5 |
| `drive_rewrite_rule_exprs_owned` | `definition_catalog/rewrite_rules` | 4 | `rewrite_builtins`:3, `structure_ops`:1 |
| `drive_evaluated_value` | `suspension/core` | 3 | `rewrite_builtins`:2, `structure_ops`:1 |
| `enforce_iteration_work_limit_for_big_count` | `debug_tools/resources/constraints` | 3 | `control_flow`:2, `functional_ops`:1 |
| `ensure_symbol_not_protected_for_write` | `attributes` | 3 | `associations`:3 |
| `match_compiled_owned` | `pattern_matching/core` | 3 | `rewrite_builtins`:3 |
| `normalize_call_args_by_attributes` | `eval_call_prepare` | 3 | `arithmetic`:3 |
| `normalize_match_expr` | `resolution` | 3 | `rewrite_builtins`:3 |
| `resolve_matcher_pattern_expr_owned` | `rewrite_builtins/cases/match_q` | 3 | `control_flow`:2, `structure_ops`:1 |
| `suspend_with_frame_lazy` | `suspension/core` | 3 | `associations`:1, `control_flow`:2 |
| `suspend_with_result_frame_lazy` | `suspension/core` | 3 | `control_flow`:3 |
| `system_option_bool` | `options/core` | 3 | `rewrite_builtins`:2, `structure_ops`:1 |
| `thread_call_args` | `listable_dispatch` | 3 | `functional_ops`:1, `integer_digits`:1, `number_theory`:1 |
| `apply_replace_all` | `definition_application` | 2 | `rewrite_builtins`:2 |
| `attribute_set_for_head` | `eval_call_prepare` | 2 | `arithmetic`:2 |
| `attribute_set_for_symbol` | `resolution` | 2 | `control_flow`:1, `rewrite_builtins`:1 |
| `bump_attribute_revision` | `resolution` | 2 | `control_flow`:2 |
| `collect_compiled_args_matches_owned` | `pattern_matching/sequence/entry` | 2 | `rewrite_builtins`:2 |
| `collect_compiled_orderless_arg_matches_owned` | `pattern_matching/sequence/entry` | 2 | `rewrite_builtins`:2 |
| `compile_cases_spec_from_resolved_expr` | `definition_catalog/rewrite_rules` | 2 | `rewrite_builtins`:2 |
| `emit_non_loader_eval_message` | `session_helpers` | 2 | `control_flow`:2 |
| `enforce_active_resource_limits_for_expr` | `debug_tools/resources/constraints` | 2 | `arithmetic`:2 |
| `invalid_cases_traversal_call` | `definition_catalog/traversal` | 2 | `rewrite_builtins`:2 |
| `lower_syntax_expr` | `resolution` | 2 | `strings`:2 |
| `messages_since_match` | `session_helpers` | 2 | `control_flow`:2 |
| `normalize_definition_assignment_target` | `definition_analysis_helpers` | 2 | `associations`:2 |
| `normalize_optional_message_selectors` | `message_selector_parsing` | 2 | `control_flow`:2 |
| `option_rule_expr_for_symbol` | `options/core` | 2 | `control_flow`:1, `functional_ops`:1 |
| `rebuild_message_symbol_counts` | `resolution` | 2 | `control_flow`:2 |
| `refresh_has_up_value_definitions` | `definition_clear` | 2 | `control_flow`:2 |
| `resolve_cases_spec_expr_for_matching_owned` | `definition_catalog/rewrite_rules` | 2 | `rewrite_builtins`:2 |
| `resolve_cases_traversal_owned` | `definition_catalog/traversal` | 2 | `rewrite_builtins`:2 |
| `suppress_messages_since` | `session_helpers` | 2 | `control_flow`:2 |
| `try_apply_rewrite_rules_once_owned` | `definition_application` | 2 | `rewrite_builtins`:1, `structure_ops`:1 |
| `allocate_fresh_symbol` | `scoping/unique` | 1 | `control_flow`:1 |
| `clear_dynamic_scope_symbol_side_effects` | `eval_core` | 1 | `control_flow`:1 |
| `collect_compiled_args_matches_with_flat_head_owned` | `pattern_matching/sequence/entry` | 1 | `rewrite_builtins`:1 |
| `drive_rewrite_rules_owned` | `definition_catalog/rewrite_rules` | 1 | `rewrite_builtins`:1 |
| `dynamic_scope_symbol_state_is_clean` | `eval_core` | 1 | `control_flow`:1 |
| `emit_thread_length_message` | `listable_dispatch` | 1 | `functional_ops`:1 |
| `invalid_call` | `session_helpers` | 1 | `control_flow`:1 |
| `invalid_heads_option_call` | `parse_and_message_helpers` | 1 | `structure_ops`:1 |
| `push_dynamic_own_value_binding` | `eval_core` | 1 | `control_flow`:1 |
| `push_protected_write_message` | `attributes` | 1 | `associations`:1 |
| `push_rule_applied_trace` | `definition_application` | 1 | `rewrite_builtins`:1 |
| `render_message_text_with_named` | `message_catalog` | 1 | `control_flow`:1 |
| `resume_call_from_prepared_args` | `eval_call_prepare` | 1 | `rewrite_builtins`:1 |
| `store_definition_target` | `definition_tags` | 1 | `associations`:1 |
| `symbol_is_protected` | `attributes` | 1 | `associations`:1 |
| `truncate_dynamic_own_value_bindings` | `eval_core` | 1 | `control_flow`:1 |

## Top 15 Axis 2 methods
| method | defining module(s) | total calls | per-family counts |
|---|---|---:|---|
| `evaluate` | `resolution` | 148 | `arithmetic`:1, `associations`:28, `control_flow`:54, `functional_ops`:30, `integer_digits`:1, `logic`:4, `number_theory`:1, `predicates`:1, `rewrite_builtins`:15, `strings`:5, `structure_ops`:8 |
| `invalid_call_with_message` | `session_helpers` | 131 | `associations`:78, `calculus`:1, `control_flow`:11, `functional_ops`:1, `predicates`:3, `structure_ops`:37 |
| `invalid_call_with_exact_arg_count` | `session_helpers` | 113 | `arithmetic`:11, `combinatorics`:9, `control_flow`:5, `functional_ops`:1, `logic`:2, `number_theory`:8, `polynomials`:4, `predicates`:32, `rewrite_builtins`:3, `strings`:29, `structure_ops`:9 |
| `invalid_call_with_arg_range` | `session_helpers` | 112 | `arithmetic`:1, `control_flow`:15, `functional_ops`:36, `integer_digits`:5, `number_theory`:1, `polynomials`:3, `predicates`:3, `rewrite_builtins`:4, `strings`:25, `structure_ops`:19 |
| `suspend_with_frame` | `suspension/core` | 100 | `associations`:31, `control_flow`:18, `functional_ops`:25, `logic`:4, `predicates`:1, `rewrite_builtins`:13, `strings`:4, `structure_ops`:4 |
| `invalid_call_with_arg_form` | `session_helpers` | 67 | `control_flow`:7, `functional_ops`:48, `polynomials`:2, `predicates`:1, `rewrite_builtins`:6, `structure_ops`:3 |
| `drive_internal_value` | `suspension/core` | 41 | `control_flow`:12, `rewrite_builtins`:24, `structure_ops`:5 |
| `push_message` | `session_helpers` | 37 | `associations`:11, `control_flow`:4, `number_theory`:1, `structure_ops`:21 |
| `suspend_with_result_frame` | `suspension/core` | 17 | `control_flow`:17 |
| `enforce_active_memory_limit_for_projected_bytes` | `debug_tools/resources/constraints` | 12 | `control_flow`:1, `functional_ops`:2, `strings`:2, `structure_ops`:7 |
| `enforce_generated_element_limit_for_count` | `debug_tools/resources/constraints` | 10 | `functional_ops`:1, `integer_digits`:4, `strings`:2, `structure_ops`:3 |
| `match_expr_owned` | `pattern_matching/core` | 9 | `control_flow`:5, `rewrite_builtins`:3, `structure_ops`:1 |
| `combine_multiplicative_args` | `arithmetic` | 7 | `calculus`:3, `combinatorics`:1, `polynomials`:3 |
| `merged_option_exprs_for_system_symbol` | `options/core` | 7 | `strings`:7 |
| `enforce_generated_element_limit_for_big_count` | `debug_tools/resources/constraints` | 6 | `control_flow`:1, `functional_ops`:1, `structure_ops`:4 |

## Parse coverage notes
Files without an exact `use super::*;` line in the restored tree (therefore no temporary comment-out was possible there):
- `functional_ops`: `crates/rum-runtime/src/runtime/functional_ops/arrays/filter.rs`, `crates/rum-runtime/src/runtime/functional_ops/arrays/spec.rs`
- `polynomials`: `crates/rum-runtime/src/runtime/polynomials/candidates.rs`, `crates/rum-runtime/src/runtime/polynomials/core.rs`, `crates/rum-runtime/src/runtime/polynomials/factorization.rs`, `crates/rum-runtime/src/runtime/polynomials/rational_function.rs`
- `strings`: `crates/rum-runtime/src/runtime/strings/patterns/regex.rs`, `crates/rum-runtime/src/runtime/strings/similarity/lexical.rs`, `crates/rum-runtime/src/runtime/strings/text/diacritics.rs`

Excluded downstream spans:
| family | excluded downstream error spans (top) |
|---|---|
| `arithmetic` | `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs`:26, `crates/rum-runtime/src/runtime/calculus.rs`:6, `crates/rum-runtime/src/runtime/combinatorics.rs`:2 |
| `associations` | `crates/rum-runtime/src/runtime/builtin_catalog/associations.rs`:33, `crates/rum-runtime/src/runtime/definitions.rs`:1 |
| `calculus` | `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs`:1 |
| `combinatorics` | `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs`:11 |
| `control_flow` | `crates/rum-runtime/src/runtime/builtin_adapters.rs`:26, `crates/rum-runtime/src/runtime/builtin_catalog/control.rs`:10, `crates/rum-runtime/src/runtime/scoping/callable.rs`:1 |
| `functional_ops` | `crates/rum-runtime/src/runtime/builtin_catalog/functional.rs`:42 |
| `integer_digits` | `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs`:5 |
| `logic` | `crates/rum-runtime/src/runtime/builtin_catalog/logic.rs`:19, `crates/rum-runtime/src/runtime/builtin_adapters.rs`:2 |
| `number_theory` | `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs`:9 |
| `polynomials` | `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs`:5, `crates/rum-runtime/src/runtime/builtin_catalog/logic.rs`:3 |
| `predicates` | `crates/rum-runtime/src/runtime/builtin_catalog/logic.rs`:47, `crates/rum-runtime/src/runtime/builtin_catalog/associations.rs`:1 |
| `strings` | `crates/rum-runtime/src/runtime/builtin_catalog/strings.rs`:61 |
| `structure_ops` | `crates/rum-runtime/src/runtime/builtin_catalog/structure.rs`:38, `crates/rum-runtime/src/runtime/builtin_catalog/rules.rs`:4 |
| `rewrite_builtins` | - |

## Clean-tree proof

Final command outputs:

```text
$ cargo check -p rum-runtime
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
```

```text
$ git status --short -- crates/
```

The scoped status output above is empty.

