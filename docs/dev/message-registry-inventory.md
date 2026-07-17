# Builtin Diagnostic-Message Inventory

Diagnostic audit only. This document records the repository state observed
for brief 243; it does not propose or implement source changes.

## Table A — every direct `push_message` call site

Count command and result (test-path calls excluded):

```text
$ rg -n --glob '*.rs' '\.push_message\s*\(' crates | rg -v '/tests/' | python3 -c 'import sys; print(sum(1 for _ in sys.stdin))'
114
```

Cross-check including test paths:

```text
$ rg -n --glob '*.rs' '\.push_message\s*\(' crates | python3 -c 'import sys; print(sum(1 for _ in sys.stdin))'
115
```

The sole excluded row is the test-only call at
`crates/rum-runtime/src/runtime/tests/session_config.rs:245`. Table A has one
row per one of the 114 non-test matches.

| # | crate | file:line | symbol arg (literal `"X"` / variable / fn parameter) | tag arg (literal `"x"` / computed) | text shape | in-shared-helper? |
|---:|---|---|---|---|---|---|
| 1 | `rum-runtime` | `crates/rum-runtime/src/runtime/attributes.rs:213` | fn parameter `operator` | literal `"wrsym"` | FORMATTED with 1 runtime value | `push_protected_write_message` |
| 2 | `rum-runtime` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:72` | literal `"General"` | literal `"sandbox"` | FORMATTED with 1 runtime value | `ensure_capability_allowed` |
| 3 | `rum-runtime` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:328` | literal `"Needs"` | literal `"load"` | FORMATTED with 1 runtime value | — |
| 4 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:44` | literal `"Remove"` | literal `"rmptc"` | FORMATTED with 1 runtime value | — |
| 5 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:154` | fn parameter `symbol` | literal `"nosym"` | FORMATTED with 1 runtime value | `finish_up_set_assignment` |
| 6 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:120` | literal `"AppendTo"` | literal `"normal"` | FORMATTED with 1 runtime value | — |
| 7 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:154` | literal `"PrependTo"` | literal `"normal"` | FORMATTED with 1 runtime value | — |
| 8 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:325` | fn parameter `operator` | literal `"argform"` | STATIC literal | `current_part_held_update_value` |
| 9 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:354` | literal `"Part"` | literal `"pkspec1"` | PASSTHROUGH variable `detail.clone()` | — |
| 10 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:355` | literal `"Set"` | literal `"pkspec1"` | PASSTHROUGH variable `detail` | — |
| 11 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:359` | fn parameter `operator` | literal `"rvalue"` | FORMATTED with 1 runtime value | `push_held_update_rvalue_message` |
| 12 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/definitions.rs:442` | fn parameter `symbol` | literal `"shape"` | FORMATTED with 2 runtime values | `try_set_list_destructuring_lvalue` |
| 13 | `rum-runtime` | `crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs:77` | computed from fn parameter `kind.inspector_name()` | literal `"setval"` | PASSTHROUGH variable `message` | `replace_value_table_from_expr` |
| 14 | `rum-runtime` | `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:145` | literal `"General"` | literal `"sandbox"` | FORMATTED with 1 runtime value | `sandbox_denied_builtin` |
| 15 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:486` | literal `"Set"` | literal `"kval"` | FORMATTED with 1 runtime value | — |
| 16 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:141` | literal `"CheckAbort"` | literal `"opttfa"` | FORMATTED with 1 runtime value | — |
| 17 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:31` | literal `"Assert"` | variable `message_tag` | PASSTHROUGH variable `text` | — |
| 18 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:39` | literal `"Assert"` | variable `message_tag` | PASSTHROUGH variable `text` | — |
| 19 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:64` | literal `"Assert"` | literal `"asrte"` | FORMATTED with 1 runtime value | — |
| 20 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:12` | computed field `error.symbol` | literal `"range"` | FORMATTED with 1 runtime value | — |
| 21 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:36` | literal `"ReplacePart"` | literal `"span"` | FORMATTED with 1 runtime value | — |
| 22 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:102` | literal `"Part"` | literal `"pkspec1"` | FORMATTED with 1 runtime value | — |
| 23 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:117` | literal `"Part"` | literal `"pmix"` | STATIC literal | — |
| 24 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:161` | literal `"Extract"` | literal `"psl1"` | FORMATTED with 2 runtime values | — |
| 25 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:210` | literal `"Extract"` | literal `"keyw"` | FORMATTED with 2 runtime values | — |
| 26 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:225` | fn parameter `symbol` | literal `"partw"` | FORMATTED with 2 runtime values | `push_part_selection_failure` |
| 27 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:236` | fn parameter `symbol` | literal `"partd"` | FORMATTED with 1 runtime value | `push_part_selection_failure` |
| 28 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:248` | fn parameter `symbol` | literal `"pspec1"` | FORMATTED with 1 runtime value | `push_part_selection_failure` |
| 29 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:268` | fn parameter `symbol` | literal `"take"` | PASSTHROUGH variable `text` | `push_take_selection_failure` |
| 30 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:281` | fn parameter `symbol` | literal `"span"` | PASSTHROUGH variable `text` | `push_invalid_span_selector_messages` |
| 31 | `rum-runtime` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:295` | literal `"UpTo"` | fn parameter `tag` | PASSTHROUGH variable `text` | `push_invalid_upto_message_once` |
| 32 | `rum-runtime` | `crates/rum-runtime/src/runtime/listable_dispatch.rs:114` | literal `"Thread"` | literal `"tdlen"` | PASSTHROUGH variable `detail` | — |
| 33 | `rum-runtime` | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:26` | local variable `&symbol_name` | local variable `&tag` | PASSTHROUGH variable `text` | — |
| 34 | `rum-runtime` | `crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs:482` | fn parameter `symbol` | literal `"level"` | FORMATTED with 1 runtime value | `invalid_level_spec_call` |
| 35 | `rum-runtime` | `crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs:497` | fn parameter `symbol` | literal `"innf"` | FORMATTED with 2 runtime values | `invalid_non_negative_or_infinity_call` |
| 36 | `rum-runtime` | `crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs:518` | fn parameter `symbol` | literal `"optx"` | FORMATTED with 2 runtime values | `invalid_heads_option_call` |
| 37 | `rum-runtime` | `crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs:525` | fn parameter `symbol` | literal `"opttf"` | FORMATTED with 2 runtime values | `invalid_heads_option_call` |
| 38 | `rum-runtime` | `crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs:535` | fn parameter `symbol` | literal `"nonopt"` | FORMATTED with 3 runtime values | `invalid_heads_option_call` |
| 39 | `rum-runtime` | `crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs:544` | fn parameter `symbol` | literal `"nonopt"` | FORMATTED with 3 runtime values | `invalid_heads_option_call` |
| 40 | `rum-runtime` | `crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs:553` | fn parameter `symbol` | literal `"argform"` | PASSTHROUGH variable `fallback` | `invalid_heads_option_call` |
| 41 | `rum-runtime` | `crates/rum-runtime/src/runtime/namespace_tools/names.rs:79` | literal `"Names"` | literal `"optval"` | FORMATTED with 1 runtime value | — |
| 42 | `rum-runtime` | `crates/rum-runtime/src/runtime/namespace_tools/names.rs:96` | literal `"Names"` | literal `"optval"` | FORMATTED with 1 runtime value | — |
| 43 | `rum-runtime` | `crates/rum-runtime/src/runtime/options/arguments.rs:307` | computed local `head_symbol.name()` | local variable `tag` | PASSTHROUGH variable `text` | — |
| 44 | `rum-runtime` | `crates/rum-runtime/src/runtime/options/arguments.rs:354` | computed local `head_symbol.name()` | local variable `tag` | PASSTHROUGH variable `text` | — |
| 45 | `rum-runtime` | `crates/rum-runtime/src/runtime/options/builtins.rs:200` | literal `"SetOptions"` | literal `"optnf"` | FORMATTED with 2 runtime values | — |
| 46 | `rum-runtime` | `crates/rum-runtime/src/runtime/options/builtins.rs:288` | literal `"OptionValue"` | literal `"nodef"` | FORMATTED with 2 runtime values | — |
| 47 | `rum-runtime` | `crates/rum-runtime/src/runtime/options/builtins.rs:305` | literal `"OptionValue"` | literal `"optnf"` | FORMATTED with 2 runtime values | — |
| 48 | `rum-runtime` | `crates/rum-runtime/src/runtime/package_tools.rs:113` | literal `"Once"` | literal `"pers"` | FORMATTED with 2 runtime values | — |
| 49 | `rum-runtime` | `crates/rum-runtime/src/runtime/package_tools.rs:195` | fn parameter `operator` | literal `"optx"` | FORMATTED with 2 runtime values | `eval_persist_package_state` |
| 50 | `rum-runtime` | `crates/rum-runtime/src/runtime/package_tools.rs:217` | literal `"DumpSave"` | literal `"optval"` | FORMATTED with 1 runtime value | `eval_persist_package_state` |
| 51 | `rum-runtime` | `crates/rum-runtime/src/runtime/package_tools.rs:250` | fn parameter `operator` | literal `"io"` | FORMATTED with 2 runtime values | `eval_persist_package_state` |
| 52 | `rum-runtime` | `crates/rum-runtime/src/runtime/package_tools.rs:625` | literal `"OptionValue"` | literal `"nodef"` | FORMATTED with 2 runtime values | — |
| 53 | `rum-runtime` | `crates/rum-runtime/src/runtime/scoping/callable.rs:14` | literal `"Function"` | literal `"argx"` | FORMATTED with 2 runtime values | `try_apply_function` |
| 54 | `rum-runtime` | `crates/rum-runtime/src/runtime/scoping/callable.rs:43` | local variable `symbol` | literal `"argx"` | FORMATTED with 2 runtime values | `try_apply_function` |
| 55 | `rum-runtime` | `crates/rum-runtime/src/runtime/scoping/callable.rs:103` | literal `"Key"` | literal `"argx"` | FORMATTED with 2 runtime values | — |
| 56 | `rum-runtime` | `crates/rum-runtime/src/runtime/scoping/callable.rs:115` | literal `"Key"` | literal `"invak"` | FORMATTED with 1 runtime value via `invalid_association_message` | — |
| 57 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:308` | literal `"Break"` | literal `"nofwd"` | STATIC literal | — |
| 58 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:313` | literal `"Continue"` | literal `"nofwd"` | STATIC literal | — |
| 59 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:323` | literal `"Throw"` | literal `"nocatch"` | FORMATTED with 1 runtime value | — |
| 60 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:341` | fn parameter `symbol` | literal `"notfound"` | PASSTHROUGH variable `detail` | `emit_loader_message` |
| 61 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:344` | fn parameter `symbol` | literal `"io"` | FORMATTED with 2 runtime values | `emit_loader_message` |
| 62 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:347` | fn parameter `symbol` | literal `"syntax"` | FORMATTED with 1 runtime value via `to_string()` | `emit_loader_message` |
| 63 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:353` | fn parameter `symbol` | literal `"load"` | FORMATTED with 1 runtime value via `to_string()` | `emit_loader_message` |
| 64 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:359` | literal `"General"` | literal `"noio"` | STATIC literal | — |
| 65 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:363` | literal `"General"` | literal `"noentropy"` | STATIC literal | — |
| 66 | `rum-runtime` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:378` | fn parameter `symbol` | fn parameter `tag` | PASSTHROUGH fn parameter `text` | `invalid_call_with_message` |
| 67 | `rum-stdlib` | `crates/rum-stdlib/src/associations/basic.rs:143` | literal `"Lookup"` | literal `"invrl"` | FORMATTED with 1 runtime value via `invalid_association_or_rule_list_message` | — |
| 68 | `rum-stdlib` | `crates/rum-stdlib/src/associations/basic.rs:306` | literal `"KeyExistsQ"` | literal `"invrl"` | FORMATTED with 1 runtime value via `invalid_association_or_rule_list_message` | — |
| 69 | `rum-stdlib` | `crates/rum-stdlib/src/associations/mutation.rs:171` | literal `"AssociateTo"` | literal `"wrsym"` | FORMATTED with 1 runtime value | — |
| 70 | `rum-stdlib` | `crates/rum-stdlib/src/associations/mutation.rs:281` | literal `"Set"` | literal `"write"` | PASSTHROUGH variable `message` | — |
| 71 | `rum-stdlib` | `crates/rum-stdlib/src/associations/mutation.rs:342` | literal `"Set"` | literal `"write"` | PASSTHROUGH variable `message` | — |
| 72 | `rum-stdlib` | `crates/rum-stdlib/src/associations/mutation.rs:396` | fn parameter `operator` | literal `"blnoval"` | FORMATTED with 1 runtime value via `missing_immediate_value_message` | `eval_held_association_mutation` |
| 73 | `rum-stdlib` | `crates/rum-stdlib/src/associations/mutation.rs:490` | literal `"AssociateTo"` | literal `"keynx"` | FORMATTED with 2 runtime values | `try_eval_held_association_part_mutation` |
| 74 | `rum-stdlib` | `crates/rum-stdlib/src/associations/mutation.rs:496` | literal `"AssociateTo"` | literal `"partw"` | FORMATTED with 2 runtime values via `partw_message` | `try_eval_held_association_part_mutation` |
| 75 | `rum-stdlib` | `crates/rum-stdlib/src/associations/mutation.rs:504` | literal `"Part"` | literal `"partw"` | FORMATTED with 2 runtime values via `partw_message` | `try_eval_held_association_part_mutation` |
| 76 | `rum-stdlib` | `crates/rum-stdlib/src/associations/mutation.rs:507` | literal `"Part"` | literal `"partd"` | FORMATTED with 1 runtime value via `partd_message` | `try_eval_held_association_part_mutation` |
| 77 | `rum-stdlib` | `crates/rum-stdlib/src/associations/resumable/map.rs:174` | literal `"AssociationMap"` | literal `"invrlf"` | FORMATTED with 3 runtime values | — |
| 78 | `rum-stdlib` | `crates/rum-stdlib/src/bytes/codec.rs:61` | fn parameter `symbol` | fn parameter `tag` | PASSTHROUGH fn parameter `message` | `fail_bytes` |
| 79 | `rum-stdlib` | `crates/rum-stdlib/src/csv.rs:667` | fn parameter `symbol` | literal `"csv"` | PASSTHROUGH fn parameter `message` | `fail_csv` |
| 80 | `rum-stdlib` | `crates/rum-stdlib/src/datetime.rs:218` | fn parameter `symbol` | computed `error.tag()` | FORMATTED with 1 runtime error value via `error.message()` | `fail_datetime` |
| 81 | `rum-stdlib` | `crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:213` | fn parameter `symbol` | fn parameter `tag` | PASSTHROUGH fn parameter `text` | `push_tensor_message_once` |
| 82 | `rum-stdlib` | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:57` | literal `"SubsetMap"` | literal `"path"` | FORMATTED with 1 runtime value via `subset_map_path_message` | — |
| 83 | `rum-stdlib` | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:544` | fn parameter `symbol` | fn parameter `tag` | FORMATTED with 2 runtime values | `push_missing_part_message` |
| 84 | `rum-stdlib` | `crates/rum-stdlib/src/ini.rs:391` | fn parameter `symbol` | literal `"ini"` | PASSTHROUGH fn parameter `message` | `fail_ini` |
| 85 | `rum-stdlib` | `crates/rum-stdlib/src/json.rs:107` | fn parameter `symbol` | fn parameter `tag` | PASSTHROUGH fn parameter `message` | `fail_json` |
| 86 | `rum-stdlib` | `crates/rum-stdlib/src/number_theory.rs:286` | literal `"Prime"` | literal `"intpp"` | FORMATTED with 1 runtime value | — |
| 87 | `rum-stdlib` | `crates/rum-stdlib/src/random/password_policy.rs:275` | literal `"RandomPassword"` | literal `"password"` | PASSTHROUGH fn parameter `message` | — |
| 88 | `rum-stdlib` | `crates/rum-stdlib/src/random.rs:111` | literal `"RandomChoice"` | literal `"empty"` | STATIC literal | — |
| 89 | `rum-stdlib` | `crates/rum-stdlib/src/random.rs:147` | literal `"RandomChoice"` | literal `"weights"` | STATIC literal | — |
| 90 | `rum-stdlib` | `crates/rum-stdlib/src/random.rs:161` | literal `"RandomChoice"` | literal `"weights"` | STATIC literal | — |
| 91 | `rum-stdlib` | `crates/rum-stdlib/src/random.rs:179` | literal `"RandomChoice"` | literal `"weights"` | STATIC literal | — |
| 92 | `rum-stdlib` | `crates/rum-stdlib/src/random.rs:590` | fn parameter `symbol` | literal `"entropy"` | FORMATTED with 1 runtime value | `push_entropy_error` |
| 93 | `rum-stdlib` | `crates/rum-stdlib/src/schema.rs:90` | literal `"ConformSchema"` | literal `"schema"` | FORMATTED with 1 runtime error value via `schema_message_text` | — |
| 94 | `rum-stdlib` | `crates/rum-stdlib/src/schema.rs:477` | fn parameter `symbol` | literal `"schema"` | PASSTHROUGH fn parameter `message` | `fail_schema` |
| 95 | `rum-stdlib` | `crates/rum-stdlib/src/strings/encoding.rs:141` | literal `"Uncompress"` | literal `"env"` | STATIC literal | — |
| 96 | `rum-stdlib` | `crates/rum-stdlib/src/strings/patterns.rs:194` | computed field `error.symbol` | literal `"range"` | FORMATTED with 1 runtime value | — |
| 97 | `rum-stdlib` | `crates/rum-stdlib/src/strings/scripting/mod.rs:26` | fn parameter `symbol` | fn parameter `tag` | PASSTHROUGH fn parameter `message` | `failed_with_message` |
| 98 | `rum-stdlib` | `crates/rum-stdlib/src/strings/similarity/mod.rs:293` | literal `"HammingDistance"` | literal `"len"` | PASSTHROUGH variable `message` | — |
| 99 | `rum-stdlib` | `crates/rum-stdlib/src/strings/template.rs:91` | literal `"TemplateApply"` | literal `"slot"` | PASSTHROUGH variable `message` | — |
| 100 | `rum-stdlib` | `crates/rum-stdlib/src/strings/tokenize.rs:314` | fn parameter `symbol` | literal `"tok"` | PASSTHROUGH fn parameter `message` | `fail_tokenizer` |
| 101 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/join.rs:98` | literal `"Join"` | literal `"heads"` | FORMATTED with 3 runtime values | — |
| 102 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/mod.rs:50` | const literal `heads::messages::GENERAL` (`"General"`) | literal `"normal"` | FORMATTED with 1 runtime call value | `invalid_normal_expression_call` |
| 103 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/sequence/delete.rs:55` | literal `"Delete"` | literal `"partw"` | FORMATTED with 2 runtime values | — |
| 104 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/sequence/delete.rs:68` | literal `"Delete"` | literal `"partw"` | FORMATTED with 2 runtime values | — |
| 105 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/sequence/insert.rs:60` | literal `"Insert"` | literal `"ins"` | FORMATTED with 2 runtime values | — |
| 106 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:124` | fn parameter `symbol` | fn parameter `tag` | FORMATTED with 1 runtime call value | `invalid_take_drop_sequence_call` |
| 107 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:143` | fn parameter `symbol` | fn parameter `tag` | FORMATTED with 4 runtime values | `invalid_take_drop_atom_call` |
| 108 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:155` | literal `"TakeDrop"` | literal `"iseq"` | FORMATTED with 2 runtime values | — |
| 109 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/sequence/take_list.rs:37` | fn parameter `symbol` | literal `"normal"` | FORMATTED with 1 runtime call value | `invalid_sequence_normal_expression_call` |
| 110 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/surface.rs:348` | fn parameter `symbol` | fn parameter `tag` | FORMATTED with 2 runtime values | `invalid_empty_structural_selection_call` |
| 111 | `rum-stdlib` | `crates/rum-stdlib/src/structure_ops/surface.rs:366` | fn parameter `symbol` | fn parameter `tag` | FORMATTED with 2 runtime values | `invalid_empty_sequence_selection_call` |
| 112 | `rum-stdlib` | `crates/rum-stdlib/src/toml.rs:96` | fn parameter `symbol` | literal `"toml"` | PASSTHROUGH fn parameter `message` | `fail_toml` |
| 113 | `rum-stdlib` | `crates/rum-stdlib/src/xml.rs:92` | fn parameter `symbol` | literal `"xml"` | PASSTHROUGH fn parameter `message` | `fail_xml` |
| 114 | `rum-stdlib` | `crates/rum-stdlib/src/yaml.rs:94` | fn parameter `symbol` | literal `"yaml"` | PASSTHROUGH fn parameter `message` | `fail_yaml` |

Text-shape totals for these 114 rows are 11 STATIC literal sites, 72
FORMATTED sites, and 31 PASSTHROUGH sites. Thus direct sites are 9.65% static,
63.16% formatted, and 27.19% passthrough (percentages use 114 as denominator;
passthrough values are already-built strings whose upstream construction is
not represented at the push API).

Manual under-capture cross-checks opened complete call expressions at
`attributes.rs:213`, `selectors.rs:225`, `builtins_messages.rs:26`,
`datetime.rs:218`, `random.rs:590`, and `take_drop.rs:143`; all six are present
above and each corresponds to one grep match.

The row/shape count command was:

```text
$ python3 - <<'PY'
from pathlib import Path
import re
t = Path('docs/dev/message-registry-inventory.md').read_text()
a = t.split('## Table A', 1)[1].split('## Table B', 1)[0]
rows = [line for line in a.splitlines() if re.match(r'^\| \d+ \|', line)]
sequence = [int(re.match(r'^\| (\d+) \|', line).group(1)) for line in rows]
counts = {shape: sum(f'| {shape}' in line for line in rows)
          for shape in ('STATIC', 'FORMATTED', 'PASSTHROUGH')}
print('rows:', len(rows), 'sequence-ok:', sequence == list(range(1, 115)))
print(counts)
for shape, count in counts.items():
    print(shape, f'{100 * count / len(rows):.2f}%')
PY
rows: 114 sequence-ok: True
{'STATIC': 11, 'FORMATTED': 72, 'PASSTHROUGH': 31}
STATIC 9.65%
FORMATTED 63.16%
PASSTHROUGH 27.19%
```

## Table B — helper fan-in

The 56 Table-A sites marked as shared live in 41 unique helpers. This first
block covers the 40 helpers other than `invalid_call_with_message`: 130
direct caller rows plus 81 one-level rows for helper-nested callers. A
nested expansion stops after that additional level, as labelled.

| helper | caller file:line | builtin symbol passed |
|---|---|---|
| `push_protected_write_message` | `crates/rum-runtime/src/runtime/attributes.rs:128` (`finish_update_attributes_after_resolved_attributes`) | `symbol` — **HELPER-NESTED `finish_update_attributes_after_resolved_attributes`; expanded below** |
| `push_protected_write_message` | `crates/rum-runtime/src/runtime/attributes.rs:223` (`ensure_symbol_not_protected_for_write`) | `operator` — **HELPER-NESTED `ensure_symbol_not_protected_for_write`; expanded below** |
| ↳ `finish_update_attributes_after_resolved_attributes` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/attributes.rs:85` (`eval_update_attributes`) | `&symbol_name` |
| ↳ `finish_update_attributes_after_resolved_attributes` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/attributes.rs:95` (`eval_update_attributes`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:15` (`eval_unset`) | `"Unset"` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:74` (`eval_clear_like`) | `name` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:89` (`eval_tag_unset`) | `"TagUnset"` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:135` (`store_tagged_definition`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:163` (`finish_up_set_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:274` (`current_symbol_held_update_value`) | `operator` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:303` (`current_part_held_update_value`) | `operator` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:395` (`store_held_update`) | `operator` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:208` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:225` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:242` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:268` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:288` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:297` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:361` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:368` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:374` (`finish_definition_assignment`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:404` (`try_set_symbol_own_value_lvalue`) | `symbol` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:379` (`try_set_association_lvalue`) | `"Set"` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:192` (`apply_set_options_updates`) | `"SetOptions"` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-stdlib/src/associations/mutation.rs:391` (`eval_held_association_mutation`) | `operator` |
| ↳ `ensure_symbol_not_protected_for_write` (one-level expansion; stop) | `crates/rum-stdlib/src/associations/mutation.rs:471` (`try_eval_held_association_part_mutation`) | `operator` |
| `ensure_capability_allowed` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:171` (`eval_file_names`) | `"FileNames"` (message is attributed to `General`) |
| `ensure_capability_allowed` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:237` (`eval_get`) | `"Get"` (message is attributed to `General`) |
| `ensure_capability_allowed` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:287` (`eval_needs`) | `"Needs"` (message is attributed to `General`) |
| `finish_up_set_assignment` | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:13` (`eval_up_set`) | `"UpSet"` |
| `finish_up_set_assignment` | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:16` (`eval_up_set`) | `"UpSet"` |
| `finish_up_set_assignment` | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:29` (`eval_up_set_delayed`) | `"UpSetDelayed"` |
| `current_part_held_update_value` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:261` (`current_held_update_value`) | `operator` — **HELPER-NESTED `current_held_update_value`; expanded below** |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:36` (`eval_add_to`) | `"AddTo"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:54` (`eval_subtract_from`) | `"SubtractFrom"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:68` (`eval_times_by`) | `"TimesBy"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:82` (`eval_divide_by`) | `"DivideBy"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:97` (`eval_apply_to`) | `"ApplyTo"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:111` (`eval_append_to`) | `"AppendTo"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:145` (`eval_prepend_to`) | `"PrependTo"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:179` (`eval_increment`) | `"Increment"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:236` (`eval_unary_increment_update`) | `operator` |
| `push_held_update_rvalue_message` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:264` (`current_held_update_value`) | `operator` — **HELPER-NESTED `current_held_update_value`; expanded below** |
| `push_held_update_rvalue_message` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:283` (`current_symbol_held_update_value`) | `operator` — **HELPER-NESTED `current_symbol_held_update_value`; expanded below** |
| `push_held_update_rvalue_message` | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:312` (`current_part_held_update_value`) | `operator` — **HELPER-NESTED `current_part_held_update_value`; expanded below** |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:36` (`eval_add_to`) | `"AddTo"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:54` (`eval_subtract_from`) | `"SubtractFrom"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:68` (`eval_times_by`) | `"TimesBy"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:82` (`eval_divide_by`) | `"DivideBy"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:97` (`eval_apply_to`) | `"ApplyTo"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:111` (`eval_append_to`) | `"AppendTo"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:145` (`eval_prepend_to`) | `"PrependTo"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:179` (`eval_increment`) | `"Increment"` |
| ↳ `current_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:236` (`eval_unary_increment_update`) | `operator` |
| ↳ `current_symbol_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:257` (`current_held_update_value`) | `operator` |
| ↳ `current_part_held_update_value` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:261` (`current_held_update_value`) | `operator` |
| `try_set_list_destructuring_lvalue` | `crates/rum-runtime/src/runtime/definitions/definitions.rs:321` (`finish_definition_assignment`) | `symbol` — **HELPER-NESTED `finish_definition_assignment`; expanded below** |
| ↳ `finish_definition_assignment` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:64` (`eval_set_delayed`) | `"SetDelayed"` |
| ↳ `finish_definition_assignment` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:116` (`resume_set_with_rhs`) | `"Set"` |
| ↳ `finish_definition_assignment` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:468` (`store_list_destructuring_assignments`) | `symbol` |
| `replace_value_table_from_expr` | `crates/rum-runtime/src/runtime/definitions/definitions.rs:292` (`finish_definition_assignment`) | `kind.inspector_name()` → `OwnValues` / `DownValues` / `UpValues` / `SubValues` — **HELPER-NESTED `finish_definition_assignment`; expanded below** |
| ↳ `finish_definition_assignment` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:64` (`eval_set_delayed`) | `"SetDelayed"` |
| ↳ `finish_definition_assignment` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:116` (`resume_set_with_rhs`) | `"Set"` |
| ↳ `finish_definition_assignment` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:468` (`store_list_destructuring_assignments`) | `symbol` |
| `sandbox_denied_builtin` | `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:127` (`try_dispatch_builtin_call`) | dynamic registered `spec.name` / call `head` (message is attributed to `General`) — **HELPER-NESTED `try_dispatch_builtin_call`; expanded below** |
| ↳ `try_dispatch_builtin_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:71` (`run_call_dispatch_stage`) | dynamic registered `spec.name` / call `head` (message is attributed to `General`) |
| ↳ `try_dispatch_builtin_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:105` (`run_call_dispatch_stage`) | dynamic registered `spec.name` / call `head` (message is attributed to `General`) |
| `push_part_selection_failure` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:128` (`eval_part`) | `"Part"` |
| `push_part_selection_failure` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:180` (`eval_extract`) | `"Extract"` |
| `push_part_selection_failure` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:195` (`eval_extract`) | `"Extract"` |
| `push_take_selection_failure` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:246` (`push_part_selection_failure`) | `symbol` — **HELPER-NESTED `push_part_selection_failure`; expanded below** |
| ↳ `push_part_selection_failure` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:128` (`eval_part`) | `"Part"` |
| ↳ `push_part_selection_failure` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:180` (`eval_extract`) | `"Extract"` |
| ↳ `push_part_selection_failure` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:195` (`eval_extract`) | `"Extract"` |
| `push_invalid_span_selector_messages` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:100` (`eval_part`) | `"Part"` |
| `push_invalid_span_selector_messages` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:159` (`eval_extract`) | `"Extract"` |
| `push_invalid_upto_message_once` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:306` (`push_invalid_upto_messages_in_span`) | `UpTo` — **HELPER-NESTED `push_invalid_upto_messages_in_span`; expanded below** |
| `push_invalid_upto_message_once` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:313` (`push_invalid_upto_messages_in_span`) | `UpTo` — **HELPER-NESTED `push_invalid_upto_messages_in_span`; expanded below** |
| ↳ `push_invalid_upto_messages_in_span` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:272` (`push_invalid_span_selector_messages`) | `UpTo` |
| ↳ `push_invalid_upto_messages_in_span` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:325` (`push_invalid_upto_messages_in_span`) | `UpTo` |
| `invalid_level_spec_call` | `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:237` (`invalid_cases_traversal_call`) | `symbol` — **HELPER-NESTED `invalid_cases_traversal_call`; expanded below** |
| `invalid_level_spec_call` | `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:269` (`invalid_traversal_without_limit_call`) | `symbol` — **HELPER-NESTED `invalid_traversal_without_limit_call`; expanded below** |
| `invalid_level_spec_call` | `crates/rum-runtime/src/runtime/tree_walkers/level.rs:15` (`eval_level`) | `"Level"` |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:82` (`finish_cases_after_resolved_traversal`) | `"Cases"` |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:91` (`finish_delete_cases_after_resolved_traversal`) | `"DeleteCases"` |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/position.rs:77` (`finish_position_after_resolved_traversal`) | `"Position"` |
| ↳ `invalid_traversal_without_limit_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:214` (`finish_count_like_after_pattern`) | `builtin_name` |
| `invalid_non_negative_or_infinity_call` | `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:240` (`invalid_cases_traversal_call`) | `symbol` — **HELPER-NESTED `invalid_cases_traversal_call`; expanded below** |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:82` (`finish_cases_after_resolved_traversal`) | `"Cases"` |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:91` (`finish_delete_cases_after_resolved_traversal`) | `"DeleteCases"` |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/position.rs:77` (`finish_position_after_resolved_traversal`) | `"Position"` |
| `invalid_heads_option_call` | `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:246` (`invalid_cases_traversal_call`) | `symbol` — **HELPER-NESTED `invalid_cases_traversal_call`; expanded below** |
| `invalid_heads_option_call` | `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:275` (`invalid_traversal_without_limit_call`) | `symbol` — **HELPER-NESTED `invalid_traversal_without_limit_call`; expanded below** |
| `invalid_heads_option_call` | `crates/rum-runtime/src/runtime/tree_walkers/level.rs:27` (`eval_level`) | `"Level"` |
| `invalid_heads_option_call` | `crates/rum-runtime/src/runtime/tree_walkers/level.rs:40` (`eval_level`) | `"Level"` |
| `invalid_heads_option_call` | `crates/rum-stdlib/src/structure_ops/surface.rs:41` (`eval_depth`) | `"Depth"` |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:82` (`finish_cases_after_resolved_traversal`) | `"Cases"` |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:91` (`finish_delete_cases_after_resolved_traversal`) | `"DeleteCases"` |
| ↳ `invalid_cases_traversal_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/position.rs:77` (`finish_position_after_resolved_traversal`) | `"Position"` |
| ↳ `invalid_traversal_without_limit_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:214` (`finish_count_like_after_pattern`) | `builtin_name` |
| `eval_persist_package_state` | `crates/rum-runtime/src/runtime/package_tools.rs:80` (`eval_save`) | `"Save"` |
| `eval_persist_package_state` | `crates/rum-runtime/src/runtime/package_tools.rs:84` (`eval_dump_save`) | `"DumpSave"` |
| `try_apply_function` | `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:107` (`run_call_dispatch_stage`) | dynamic callable `head` (`Function` or a named callable) — **HELPER-NESTED `run_call_dispatch_stage`; expanded below** |
| ↳ `run_call_dispatch_stage` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/eval_core/call.rs:148` (`resume_call_with_dispatch_plan`) | dynamic evaluated call head / `builtin_spec` |
| `emit_loader_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:224` (`eval_find_file`) | `"FindFile"` |
| `emit_loader_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:265` (`eval_get`) | `"Get"` |
| `emit_loader_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:280` (`eval_get`) | `"Get"` |
| `eval_held_association_mutation` | `crates/rum-stdlib/src/associations/mutation.rs:136` (`eval_associate_to`) | `"AssociateTo"` |
| `eval_held_association_mutation` | `crates/rum-stdlib/src/associations/mutation.rs:243` (`eval_key_drop_from`) | `"KeyDropFrom"` |
| `try_eval_held_association_part_mutation` | `crates/rum-stdlib/src/associations/mutation.rs:147` (`eval_associate_to`) | `"AssociateTo"` |
| `try_eval_held_association_part_mutation` | `crates/rum-stdlib/src/associations/mutation.rs:255` (`eval_key_drop_from`) | `"KeyDropFrom"` |
| `fail_bytes` | `crates/rum-stdlib/src/bytes/digest.rs:53` (`eval_crc32`) | `"Crc32"` |
| `fail_bytes` | `crates/rum-stdlib/src/bytes/digest.rs:83` (`eval_digest`) | `symbol` — **HELPER-NESTED `eval_digest`; expanded below** |
| `fail_bytes` | `crates/rum-stdlib/src/bytes/digest.rs:97` (`eval_digest`) | `symbol` — **HELPER-NESTED `eval_digest`; expanded below** |
| `fail_bytes` | `crates/rum-stdlib/src/bytes.rs:26` (`eval_to_hex_string`) | `"ToHexString"` |
| `fail_bytes` | `crates/rum-stdlib/src/bytes.rs:50` (`eval_from_hex_string`) | `"FromHexString"` |
| `fail_bytes` | `crates/rum-stdlib/src/bytes.rs:69` (`eval_to_base64`) | `"ToBase64"` |
| `fail_bytes` | `crates/rum-stdlib/src/bytes.rs:93` (`eval_from_base64`) | `"FromBase64"` |
| `fail_bytes` | `crates/rum-stdlib/src/bytes.rs:127` (`eval_byte_array_to_string`) | `"ByteArrayToString"` |
| `fail_bytes` | `crates/rum-stdlib/src/bytes.rs:137` (`eval_byte_array_to_string`) | `"ByteArrayToString"` |
| ↳ `eval_digest` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes/digest.rs:15` (`eval_sha256`) | `"Sha256"` |
| ↳ `eval_digest` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes/digest.rs:23` (`eval_sha512`) | `"Sha512"` |
| ↳ `eval_digest` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes/digest.rs:31` (`eval_sha1`) | `"Sha1"` |
| ↳ `eval_digest` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes/digest.rs:39` (`eval_md5`) | `"Md5"` |
| `fail_csv` | `crates/rum-stdlib/src/csv.rs:95` (`eval_from_csv`) | `"FromCsv"` |
| `fail_csv` | `crates/rum-stdlib/src/csv.rs:100` (`eval_from_csv`) | `"FromCsv"` |
| `fail_csv` | `crates/rum-stdlib/src/csv.rs:111` (`eval_to_csv`) | `"ToCsv"` |
| `fail_csv` | `crates/rum-stdlib/src/csv.rs:116` (`eval_to_csv`) | `"ToCsv"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:19` (`eval_date_list`) | `"DateList"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:33` (`eval_from_date_list`) | `"FromDateList"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:54` (`eval_date_string`) | `"DateString"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:59` (`eval_date_string`) | `"DateString"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:62` (`eval_date_string`) | `"DateString"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:86` (`eval_date_string`) | `"DateString"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:99` (`eval_from_date_string`) | `"FromDateString"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:108` (`eval_from_date_string`) | `"FromDateString"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:126` (`eval_date_plus`) | `"DatePlus"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:141` (`eval_date_difference`) | `"DateDifference"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:153` (`eval_day_of_week`) | `"DayOfWeek"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:157` (`eval_day_of_week`) | `"DayOfWeek"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:168` (`eval_day_of_week`) | `"DayOfWeek"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:172` (`eval_day_of_week`) | `"DayOfWeek"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:174` (`eval_day_of_week`) | `"DayOfWeek"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:198` (`eval_leap_year_q`) | `"LeapYearQ"` |
| `fail_datetime` | `crates/rum-stdlib/src/datetime.rs:213` (`eval_days_in_month`) | `"DaysInMonth"` |
| `push_tensor_message_once` | `crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:166` (`compute_inner`) | `caller` — **HELPER-NESTED `compute_inner`; expanded below** |
| ↳ `compute_inner` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:24` (`eval_inner`) | `"Inner"` |
| ↳ `compute_inner` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:47` (`eval_dot`) | `"Dot"` |
| `push_missing_part_message` | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:89` (`eval_map_at`) | `"MapAt"` |
| `fail_ini` | `crates/rum-stdlib/src/ini.rs:57` (`eval_from_ini`) | `"FromIni"` |
| `fail_ini` | `crates/rum-stdlib/src/ini.rs:65` (`eval_from_ini`) | `"FromIni"` |
| `fail_ini` | `crates/rum-stdlib/src/ini.rs:74` (`eval_from_ini`) | `"FromIni"` |
| `fail_ini` | `crates/rum-stdlib/src/ini.rs:84` (`eval_to_ini`) | `"ToIni"` |
| `fail_ini` | `crates/rum-stdlib/src/ini.rs:93` (`eval_to_ini`) | `"ToIni"` |
| `fail_json` | `crates/rum-stdlib/src/json.rs:16` (`eval_from_json`) | `"FromJson"` |
| `fail_json` | `crates/rum-stdlib/src/json.rs:25` (`eval_from_json`) | `"FromJson"` |
| `fail_json` | `crates/rum-stdlib/src/json.rs:35` (`eval_from_json`) | `"FromJson"` |
| `fail_json` | `crates/rum-stdlib/src/json.rs:52` (`eval_to_json`) | `"ToJson"` |
| `fail_json` | `crates/rum-stdlib/src/json.rs:58` (`eval_to_json`) | `"ToJson"` |
| `push_entropy_error` | `crates/rum-stdlib/src/random/password_policy.rs:62` (`eval_random_password_with_policy`) | `"RandomPassword"` |
| `push_entropy_error` | `crates/rum-stdlib/src/random.rs:37` (`eval_reseed_random`) | `"ReseedRandom"` |
| `push_entropy_error` | `crates/rum-stdlib/src/random.rs:296` (`eval_random_password`) | `"RandomPassword"` |
| `push_entropy_error` | `crates/rum-stdlib/src/random.rs:360` (`eval_random_bytes`) | `"RandomBytes"` |
| `fail_schema` | `crates/rum-stdlib/src/schema.rs:56` (`eval_schema_errors`) | `"SchemaErrors"` |
| `fail_schema` | `crates/rum-stdlib/src/schema.rs:74` (`eval_conform_schema`) | `"ConformSchema"` |
| `fail_schema` | `crates/rum-stdlib/src/schema.rs:487` (`finish_abort`) | `symbol` — **HELPER-NESTED `finish_abort`; expanded below** |
| ↳ `finish_abort` (one-level expansion; stop) | `crates/rum-stdlib/src/schema.rs:61` (`eval_schema_errors`) | `"SchemaErrors"` |
| ↳ `finish_abort` (one-level expansion; stop) | `crates/rum-stdlib/src/schema.rs:80` (`eval_conform_schema`) | `"ConformSchema"` |
| `failed_with_message` | `crates/rum-stdlib/src/strings/scripting/html.rs:36` (`eval_html_unescape`) | `"HtmlUnescape"` |
| `failed_with_message` | `crates/rum-stdlib/src/strings/scripting/shell.rs:24` (`eval_shell_split`) | `"ShellSplit"` |
| `failed_with_message` | `crates/rum-stdlib/src/strings/scripting/url.rs:40` (`eval_url_decode`) | `"UrlDecode"` |
| `failed_with_message` | `crates/rum-stdlib/src/strings/scripting/url.rs:61` (`eval_url_parse`) | `"UrlParse"` |
| `failed_with_message` | `crates/rum-stdlib/src/strings/scripting/url.rs:67` (`eval_url_parse`) | `"UrlParse"` |
| `fail_tokenizer` | `crates/rum-stdlib/src/strings/tokenize.rs:34` (`eval_tokenize_with`) | `"TokenizeWith"` |
| `fail_tokenizer` | `crates/rum-stdlib/src/strings/tokenize.rs:43` (`eval_tokenize_with`) | `"TokenizeWith"` |
| `fail_tokenizer` | `crates/rum-stdlib/src/strings/tokenize.rs:47` (`eval_tokenize_with`) | `"TokenizeWith"` |
| `fail_tokenizer` | `crates/rum-stdlib/src/strings/tokenize.rs:63` (`eval_rum_tokens`) | `"RumTokens"` |
| `fail_tokenizer` | `crates/rum-stdlib/src/strings/tokenize.rs:71` (`eval_rum_tokens`) | `"RumTokens"` |
| `invalid_normal_expression_call` | `crates/rum-stdlib/src/structure_ops/sequence/delete.rs:25` (`eval_delete`) | "Delete" (message is attributed to `General`) |
| `invalid_normal_expression_call` | `crates/rum-stdlib/src/structure_ops/sequence/delete.rs:33` (`eval_delete`) | "Delete" (message is attributed to `General`) |
| `invalid_normal_expression_call` | `crates/rum-stdlib/src/structure_ops/sequence/insert.rs:37` (`eval_insert`) | "Insert" (message is attributed to `General`) |
| `invalid_take_drop_sequence_call` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:42` (`eval_take`) | `"Take"` |
| `invalid_take_drop_sequence_call` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:74` (`eval_drop`) | `"Drop"` |
| `invalid_take_drop_atom_call` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:39` (`eval_take`) | `"Take"` |
| `invalid_take_drop_atom_call` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:71` (`eval_drop`) | `"Drop"` |
| `invalid_sequence_normal_expression_call` | `crates/rum-stdlib/src/structure_ops/sequence/take_list.rs:16` (`eval_take_list`) | `"TakeList"` |
| `invalid_empty_structural_selection_call` | `crates/rum-stdlib/src/structure_ops/surface.rs:304` (`resolve_first_like`) | `symbol` — **HELPER-NESTED `resolve_first_like`; expanded below** |
| `invalid_empty_structural_selection_call` | `crates/rum-stdlib/src/structure_ops/surface.rs:320` (`resolve_first_like`) | `symbol` — **HELPER-NESTED `resolve_first_like`; expanded below** |
| ↳ `resolve_first_like` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:91` (`eval_first`) | `"First"` |
| ↳ `resolve_first_like` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:94` (`eval_first`) | `"First"` |
| ↳ `resolve_first_like` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:112` (`eval_last`) | `"Last"` |
| ↳ `resolve_first_like` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:115` (`eval_last`) | `"Last"` |
| `invalid_empty_sequence_selection_call` | `crates/rum-stdlib/src/structure_ops/surface.rs:138` (`eval_rest`) | `"Rest"` |
| `invalid_empty_sequence_selection_call` | `crates/rum-stdlib/src/structure_ops/surface.rs:150` (`eval_rest`) | `"Rest"` |
| `invalid_empty_sequence_selection_call` | `crates/rum-stdlib/src/structure_ops/surface.rs:170` (`eval_most`) | `"Most"` |
| `invalid_empty_sequence_selection_call` | `crates/rum-stdlib/src/structure_ops/surface.rs:183` (`eval_most`) | `"Most"` |
| `fail_toml` | `crates/rum-stdlib/src/toml.rs:55` (`eval_from_toml`) | `"FromToml"` |
| `fail_toml` | `crates/rum-stdlib/src/toml.rs:63` (`eval_from_toml`) | `"FromToml"` |
| `fail_toml` | `crates/rum-stdlib/src/toml.rs:72` (`eval_from_toml`) | `"FromToml"` |
| `fail_toml` | `crates/rum-stdlib/src/toml.rs:82` (`eval_to_toml`) | `"ToToml"` |
| `fail_toml` | `crates/rum-stdlib/src/toml.rs:91` (`eval_to_toml`) | `"ToToml"` |
| `fail_xml` | `crates/rum-stdlib/src/xml.rs:16` (`eval_from_xml`) | `"FromXml"` |
| `fail_xml` | `crates/rum-stdlib/src/xml.rs:24` (`eval_from_xml`) | `"FromXml"` |
| `fail_xml` | `crates/rum-stdlib/src/xml.rs:33` (`eval_from_xml`) | `"FromXml"` |
| `fail_xml` | `crates/rum-stdlib/src/xml.rs:44` (`eval_to_xml`) | `"ToXml"` |
| `fail_xml` | `crates/rum-stdlib/src/xml.rs:49` (`eval_to_xml`) | `"ToXml"` |
| `fail_yaml` | `crates/rum-stdlib/src/yaml.rs:53` (`eval_from_yaml`) | `"FromYaml"` |
| `fail_yaml` | `crates/rum-stdlib/src/yaml.rs:61` (`eval_from_yaml`) | `"FromYaml"` |
| `fail_yaml` | `crates/rum-stdlib/src/yaml.rs:70` (`eval_from_yaml`) | `"FromYaml"` |
| `fail_yaml` | `crates/rum-stdlib/src/yaml.rs:80` (`eval_to_yaml`) | `"ToYaml"` |
| `fail_yaml` | `crates/rum-stdlib/src/yaml.rs:89` (`eval_to_yaml`) | `"ToYaml"` |

### Generic `invalid_call_with_message` fan-in

There are 173 direct callers. Eighteen direct-caller functions are themselves
helpers; their callers are expanded once and then stopped. The three standard
wrappers account for 517 of those one-level rows (196 exact-count, 162 range,
159 argument-form); their rows are grouped after the direct-caller table.

| helper | caller file:line | builtin symbol passed |
|---|---|---|
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:113` (`eval_power_mod`) | `"PowerMod"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:93` (`eval_context`) | `"Context"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:114` (`eval_contexts`) | `"Contexts"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:294` (`eval_needs`) | `"Needs"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:364` (`eval_end`) | `"End"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:397` (`eval_begin_package`) | `"BeginPackage"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:442` (`eval_end_package`) | `"EndPackage"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_input.rs:114` (`eval_pause`) | `"Pause"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_input.rs:243` (`invalid_input_options_call`) | `symbol_name` — **HELPER-NESTED `invalid_input_options_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_input.rs:251` (`invalid_input_options_call`) | `symbol_name` — **HELPER-NESTED `invalid_input_options_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/builtins_input.rs:259` (`invalid_input_options_call`) | `symbol_name` — **HELPER-NESTED `invalid_input_options_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/debug_tools/message_switch.rs:41` (`eval_message_switch`) | `symbol` — **HELPER-NESTED `eval_message_switch`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:109` (`invalid_tagged_definition_call`) | `symbol` — **HELPER-NESTED `invalid_tagged_definition_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:115` (`invalid_tagged_definition_call`) | `symbol` — **HELPER-NESTED `invalid_tagged_definition_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/hold_thread.rs:36` (`eval_thread`) | `"Thread"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:53` (`eval_exit`) | `"Exit"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:63` (`eval_exit`) | `"Exit"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:84` (`eval_check_abort`) | `"CheckAbort"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:344` (`invalid_check_abort_option_call`) | `"CheckAbort"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:352` (`invalid_check_abort_option_call`) | `"CheckAbort"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:360` (`invalid_check_abort_option_call`) | `"CheckAbort"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:10` (`eval_do`) | `"Do"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:23` (`eval_table`) | `"Table"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:50` (`eval_iteration_aggregate`) | `aggregate.name()` — **HELPER-NESTED `eval_iteration_aggregate`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:6` (`eval_check`) | `"Check"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:53` (`eval_quiet`) | `"Quiet"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:82` (`eval_part`) | `"Part"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:146` (`eval_extract`) | `"Extract"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:6` (`eval_message`) | `"Message"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:69` (`eval_messages`) | `"Messages"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:95` (`eval_message_definitions`) | `"MessageDefinitions"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:119` (`eval_message_tags`) | `"MessageTags"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/scoping/unique.rs:200` (`invalid_unique_arg`) | `"Unique"` |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:389` (`invalid_call_with_exact_arg_count`) | `symbol` — **HELPER-NESTED `invalid_call_with_exact_arg_count`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:408` (`invalid_call_with_arg_range`) | `symbol` — **HELPER-NESTED `invalid_call_with_arg_range`; expanded below** |
| `invalid_call_with_message` | `crates/rum-runtime/src/runtime/session/session_helpers.rs:426` (`invalid_call_with_arg_form`) | `symbol` — **HELPER-NESTED `invalid_call_with_arg_form`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/arithmetic/stats.rs:20` (`eval_median`) | `"Median"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/arithmetic/stats.rs:110` (`eval_quantile`) | `"Quantile"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/arithmetic/stats.rs:122` (`eval_quantile`) | `"Quantile"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/arithmetic/stats.rs:130` (`eval_quantile`) | `"Quantile"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/arithmetic/stats.rs:158` (`eval_quantile`) | `"Quantile"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/arithmetic.rs:168` (`eval_mean`) | `"Mean"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:16` (`eval_catenate`) | `"Catenate"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:58` (`eval_counts`) | `"Counts"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:71` (`eval_counts`) | `"Counts"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:90` (`eval_counts`) | `"Counts"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:101` (`eval_counts`) | `"Counts"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:119` (`eval_position_index`) | `"PositionIndex"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:137` (`eval_position_index`) | `"PositionIndex"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:290` (`finish_catenate_direct_association_values`) | `"Catenate"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/aggregate.rs:347` (`resume_catenate_steps_from_index`) | `"Catenate"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic/key_predicates.rs:12` (`eval_key_member_q`) | `"KeyMemberQ"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic/key_predicates.rs:24` (`eval_key_member_q`) | `"KeyMemberQ"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic/key_predicates.rs:45` (`eval_key_free_q`) | `"KeyFreeQ"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic/key_predicates.rs:57` (`eval_key_free_q`) | `"KeyFreeQ"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:68` (`eval_association_q`) | `"AssociationQ"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:87` (`eval_lookup`) | `"Lookup"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:224` (`eval_key`) | `"Key"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:241` (`eval_keys`) | `"Keys"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:252` (`eval_keys`) | `"Keys"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:267` (`eval_values`) | `"Values"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:278` (`eval_values`) | `"Values"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:296` (`eval_key_exists_q`) | `"KeyExistsQ"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:329` (`eval_key_take`) | `"KeyTake"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:337` (`eval_key_take`) | `"KeyTake"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:356` (`eval_key_drop`) | `"KeyDrop"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:364` (`eval_key_drop`) | `"KeyDrop"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/basic.rs:379` (`eval_normal`) | `"Normal"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/grouping.rs:75` (`eval_group_by`) | `"GroupBy"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/grouping.rs:132` (`eval_group_by`) | `"GroupBy"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/grouping.rs:162` (`eval_group_by`) | `"GroupBy"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/missing.rs:23` (`eval_delete_missing`) | `"DeleteMissing"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/missing.rs:34` (`eval_delete_missing`) | `"DeleteMissing"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/missing.rs:42` (`eval_delete_missing`) | `"DeleteMissing"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/missing.rs:52` (`eval_delete_missing`) | `"DeleteMissing"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/missing.rs:62` (`eval_delete_missing`) | `"DeleteMissing"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:118` (`eval_associate_to`) | `"AssociateTo"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:127` (`eval_associate_to`) | `"AssociateTo"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:162` (`eval_associate_to`) | `"AssociateTo"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:204` (`finish_associate_to_value`) | `"AssociateTo"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:234` (`eval_key_drop_from`) | `"KeyDropFrom"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:269` (`eval_key_drop_from`) | `"KeyDropFrom"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:289` (`eval_key_drop_from`) | `"KeyDropFrom"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:298` (`eval_key_drop_from`) | `"KeyDropFrom"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:333` (`finish_key_drop_from_value`) | `"KeyDropFrom"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:427` (`finish_held_association_mutation`) | `operator` — **HELPER-NESTED `finish_held_association_mutation`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/mutation.rs:527` (`try_eval_held_association_part_mutation`) | `operator` — **HELPER-NESTED `try_eval_held_association_part_mutation`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/aggregate.rs:15` (`eval_merge`) | `"Merge"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/aggregate.rs:23` (`eval_merge`) | `"Merge"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/aggregate.rs:59` (`eval_merge`) | `"Merge"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/aggregate.rs:110` (`eval_counts_by`) | `"CountsBy"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/aggregate.rs:128` (`eval_counts_by`) | `"CountsBy"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/key_select.rs:15` (`eval_key_select`) | `"KeySelect"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/key_select.rs:33` (`eval_key_select`) | `"KeySelect"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/key_sort.rs:27` (`eval_key_sort_by`) | `"KeySortBy"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/key_sort.rs:47` (`eval_key_sort_by`) | `"KeySortBy"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/key_sort.rs:66` (`eval_key_sort_by`) | `"KeySortBy"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/map.rs:31` (`eval_association_map`) | `"AssociationMap"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/resumable/map.rs:49` (`eval_association_map`) | `"AssociationMap"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:12` (`eval_key_union`) | `"KeyUnion"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:24` (`eval_key_union`) | `"KeyUnion"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:32` (`eval_key_union`) | `"KeyUnion"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:66` (`eval_key_intersection`) | `"KeyIntersection"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:77` (`eval_key_intersection`) | `"KeyIntersection"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:85` (`eval_key_intersection`) | `"KeyIntersection"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:139` (`eval_key_complement`) | `"KeyComplement"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:149` (`eval_key_complement`) | `"KeyComplement"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:155` (`eval_key_complement`) | `"KeyComplement"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:167` (`eval_key_complement`) | `"KeyComplement"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:175` (`eval_key_complement`) | `"KeyComplement"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:185` (`eval_key_complement`) | `"KeyComplement"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:18` (`eval_key_map`) | `"KeyMap"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:37` (`eval_key_map`) | `"KeyMap"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:58` (`eval_key_value_map`) | `"KeyValueMap"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:74` (`eval_key_value_map`) | `"KeyValueMap"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:91` (`eval_key_sort`) | `"KeySort"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:104` (`eval_key_sort`) | `"KeySort"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:122` (`eval_key_sort`) | `"KeySort"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:139` (`eval_association_thread`) | `"AssociationThread"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:150` (`eval_association_thread`) | `"AssociationThread"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/associations/transforms.rs:166` (`eval_association_thread`) | `"AssociationThread"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/calculus.rs:17` (`eval_d`) | `"D"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/calculus.rs:49` (`invalid_derivative_spec_call`) | `"D"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/calculus.rs:74` (`invalid_derivative_spec_call`) | `"D"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/datetime.rs:43` (`eval_date_string`) | `"DateString"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/functional_ops/arrays/generators.rs:251` (`invalid_outer_heads_call`) | `"Outer"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/integer_digits.rs:264` (`parse_integer_digits_base`) | `symbol` — **HELPER-NESTED `parse_integer_digits_base`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/integer_digits.rs:283` (`parse_integer_digits_length`) | `"IntegerDigits"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/predicates/ordering.rs:58` (`eval_ordered_q_default`) | `"OrderedQ"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/predicates/ordering.rs:85` (`eval_ordered_q_with_comparator`) | `"OrderedQ"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/predicates/ordering.rs:311` (`eval_alphabetic_order`) | `"AlphabeticOrder"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/predicates/ordering.rs:319` (`eval_alphabetic_order`) | `"AlphabeticOrder"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/random.rs:228` (`eval_random_sample`) | `"RandomSample"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/strings/template.rs:25` (`eval_string_form`) | `"StringForm"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:731` (`invalid_collection_normal_call`) | `symbol` — **HELPER-NESTED `invalid_collection_normal_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:743` (`invalid_empty_collection_call`) | `symbol` — **HELPER-NESTED `invalid_empty_collection_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:752` (`invalid_riffle_position_call`) | `"Riffle"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:243` (`invalid_subsets_normal_call`) | `"Subsets"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:255` (`invalid_subsets_size_spec_call`) | `"Subsets"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:267` (`invalid_tuples_normal_call`) | `"Tuples"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:279` (`invalid_tuples_length_call`) | `"Tuples"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:291` (`invalid_partition_normal_call`) | `"Partition"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:304` (`invalid_partition_positive_integer_call`) | `"Partition"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:316` (`invalid_partition_alignment_call`) | `"Partition"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/dedup.rs:251` (`invalid_delete_duplicates_normal_call`) | `symbol` — **HELPER-NESTED `invalid_delete_duplicates_normal_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/flatten.rs:101` (`invalid_flatten_level_call`) | `"Flatten"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/flatten.rs:116` (`invalid_flatten_positive_integer_list_call`) | `"Flatten"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/flatten.rs:127` (`invalid_flatten_repeated_level_call`) | `"Flatten"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/flatten.rs:142` (`invalid_flatten_depth_call`) | `"Flatten"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/flatten.rs:153` (`invalid_flatten_normal_expression_call`) | `"Flatten"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/join.rs:112` (`invalid_join_normal_call`) | `"Join"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/join.rs:124` (`invalid_join_normal1_call`) | `"Join"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/join.rs:142` (`invalid_join_headsd_call`) | `"Join"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/join.rs:154` (`invalid_join_incompatible_call`) | `"Join"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:133` (`invalid_differences_order_call`) | `"Differences"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:145` (`invalid_pad_normal_call`) | `symbol` — **HELPER-NESTED `invalid_pad_normal_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:157` (`invalid_pad_length_call`) | `symbol` — **HELPER-NESTED `invalid_pad_length_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:153` (`eval_intersection`) | `"Intersection"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:202` (`eval_complement`) | `"Complement"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:868` (`invalid_ordering_normal_call`) | `symbol` — **HELPER-NESTED `invalid_ordering_normal_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:887` (`invalid_ordering_heads_call`) | `symbol` — **HELPER-NESTED `invalid_ordering_heads_call`; expanded below** |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/reordering.rs:76` (`invalid_reverse_normal_call`) | `"Reverse"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/reordering.rs:88` (`invalid_reverse_level_call`) | `"Reverse"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/reordering.rs:100` (`invalid_transpose_matrix_call`) | `"Transpose"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/reordering.rs:109` (`invalid_transpose_permutation_call`) | `"Transpose"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:25` (`eval_take`) | `"Take"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:57` (`eval_drop`) | `"Drop"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:86` (`eval_take_drop`) | `"TakeDrop"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:106` (`eval_take_drop`) | `"TakeDrop"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/surface.rs:240` (`eval_symbol`) | `"Symbol"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/surface.rs:252` (`eval_symbol`) | `"Symbol"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/surface.rs:275` (`eval_construct`) | `"Construct"` |
| `invalid_call_with_message` | `crates/rum-stdlib/src/structure_ops/surface.rs:331` (`invalid_surface_normal_expression_call`) | `symbol` — **HELPER-NESTED `invalid_surface_normal_expression_call`; expanded below** |
| ↳ `invalid_input_options_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_input.rs:77` (`eval_input_string`) | `"InputString"` |
| ↳ `invalid_input_options_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_input.rs:86` (`eval_input`) | `"Input"` |
| ↳ `eval_message_switch` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/message_switch.rs:12` (`eval_on`) | `"On"` |
| ↳ `eval_message_switch` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/message_switch.rs:23` (`eval_off`) | `"Off"` |
| ↳ `invalid_tagged_definition_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:98` (`eval_tag_unset`) | `"TagUnset"` |
| ↳ `invalid_tagged_definition_call` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:141` (`store_tagged_definition`) | `symbol` |
| ↳ `eval_iteration_aggregate` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:37` (`eval_product`) | `"Product"` |
| ↳ `eval_iteration_aggregate` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:41` (`eval_sum`) | `"Sum"` |
| ↳ `finish_held_association_mutation` (one-level expansion; stop) | `crates/rum-stdlib/src/associations/mutation.rs:404` (`eval_held_association_mutation`) | `operator` |
| ↳ `try_eval_held_association_part_mutation` (one-level expansion; stop) | `crates/rum-stdlib/src/associations/mutation.rs:147` (`eval_associate_to`) | `"AssociateTo"` |
| ↳ `try_eval_held_association_part_mutation` (one-level expansion; stop) | `crates/rum-stdlib/src/associations/mutation.rs:255` (`eval_key_drop_from`) | `"KeyDropFrom"` |
| ↳ `parse_integer_digits_base` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:23` (`eval_integer_length`) | `"IntegerLength"` |
| ↳ `parse_integer_digits_base` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:56` (`eval_integer_digits`) | `"IntegerDigits"` |
| ↳ `parse_integer_digits_base` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:115` (`eval_integer_reverse`) | `"IntegerReverse"` |
| ↳ `parse_integer_digits_base` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:191` (`eval_digit_count`) | `"DigitCount"` |
| ↳ `invalid_collection_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:65` (`eval_tally`) | `"Tally"` |
| ↳ `invalid_collection_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:207` (`eval_reverse_sort`) | `"ReverseSort"` |
| ↳ `invalid_collection_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:241` (`eval_reverse_sort_by`) | `"ReverseSortBy"` |
| ↳ `invalid_collection_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:274` (`eval_gather_by`) | `"GatherBy"` |
| ↳ `invalid_collection_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:325` (`eval_riffle`) | `"Riffle"` |
| ↳ `invalid_collection_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:542` (`eval_extreme_by`) | `symbol` |
| ↳ `invalid_collection_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:589` (`eval_take_by`) | `symbol` |
| ↳ `invalid_empty_collection_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:545` (`eval_extreme_by`) | `symbol` |
| ↳ `invalid_delete_duplicates_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/dedup.rs:34` (`eval_delete_duplicates_structural`) | `"DeleteDuplicates"` |
| ↳ `invalid_delete_duplicates_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/dedup.rs:58` (`eval_delete_duplicates_with_test`) | `"DeleteDuplicates"` |
| ↳ `invalid_delete_duplicates_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/dedup.rs:94` (`eval_delete_duplicates_by`) | `"DeleteDuplicatesBy"` |
| ↳ `invalid_pad_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:110` (`eval_pad`) | `symbol` |
| ↳ `invalid_pad_length_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:106` (`eval_pad`) | `symbol` |
| ↳ `invalid_ordering_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:42` (`eval_sort`) | `"Sort"` |
| ↳ `invalid_ordering_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:75` (`eval_sort_by`) | `"SortBy"` |
| ↳ `invalid_ordering_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:94` (`eval_sort_by`) | `"SortBy"` |
| ↳ `invalid_ordering_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:253` (`sort_normal_expression`) | `symbol` |
| ↳ `invalid_ordering_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:267` (`same_head_argument_groups`) | `symbol` |
| ↳ `invalid_ordering_normal_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:273` (`same_head_argument_groups`) | `symbol` |
| ↳ `invalid_ordering_heads_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:281` (`same_head_argument_groups`) | `symbol` |
| ↳ `invalid_surface_normal_expression_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:148` (`eval_rest`) | `"Rest"` |
| ↳ `invalid_surface_normal_expression_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:181` (`eval_most`) | `"Most"` |
| ↳ `invalid_surface_normal_expression_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:202` (`eval_append`) | `"Append"` |
| ↳ `invalid_surface_normal_expression_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:221` (`eval_prepend`) | `"Prepend"` |
| ↳ `invalid_surface_normal_expression_call` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:322` (`resolve_first_like`) | `symbol` |

#### `invalid_call_with_exact_arg_count` expansion (196 rows)

| helper | caller file:line | builtin symbol passed |
|---|---|---|
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/canonical.rs:20` (`eval_power`) | `"Power"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/canonical.rs:97` (`eval_subtract`) | `"Subtract"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/canonical.rs:109` (`eval_divide`) | `"Divide"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:7` (`eval_factorial`) | `"Factorial"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:43` (`eval_factorial2`) | `"Factorial2"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:7` (`eval_mod`) | `"Mod"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:28` (`eval_quotient`) | `"Quotient"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:96` (`eval_power_mod`) | `"PowerMod"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/parts.rs:31` (`eval_denominator`) | `"Denominator"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/parts.rs:43` (`eval_numerator`) | `"Numerator"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/rounding.rs:8` (`eval_abs`) | `"Abs"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/attributes.rs:42` (`eval_attributes`) | `"Attributes"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/attributes.rs:69` (`eval_update_attributes`) | `symbol` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:9` (`eval_file_name_join`) | `"FileNameJoin"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:61` (`eval_parent_directory`) | `"ParentDirectory"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:203` (`eval_find_file`) | `"FindFile"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:244` (`eval_get`) | `"Get"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:343` (`eval_begin`) | `"Begin"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:360` (`eval_end`) | `"End"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:438` (`eval_end_package`) | `"EndPackage"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_input.rs:109` (`eval_pause`) | `"Pause"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/basic.rs:98` (`eval_stack`) | `"Stack"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/basic.rs:116` (`eval_rum_profile_reset`) | `"RumProfileReset"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/timing.rs:17` (`eval_absolute_time`) | `"AbsoluteTime"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/timing.rs:39` (`eval_repeated_timing`) | `"RepeatedTiming"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/timing.rs:52` (`eval_timing_like`) | `symbol` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:22` (`eval_unset`) | `"Unset"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:6` (`eval_up_set`) | `"UpSet"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:24` (`eval_up_set_delayed`) | `"UpSetDelayed"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:34` (`eval_tag_set`) | `"TagSet"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:62` (`eval_tag_set_delayed`) | `"TagSetDelayed"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:79` (`eval_tag_unset`) | `"TagUnset"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:33` (`eval_add_to`) | `"AddTo"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:50` (`eval_subtract_from`) | `"SubtractFrom"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:65` (`eval_times_by`) | `"TimesBy"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:79` (`eval_divide_by`) | `"DivideBy"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:94` (`eval_apply_to`) | `"ApplyTo"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:108` (`eval_append_to`) | `"AppendTo"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:142` (`eval_prepend_to`) | `"PrependTo"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:176` (`eval_increment`) | `"Increment"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:233` (`eval_unary_increment_update`) | `operator` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:10` (`eval_normalize_rules`) | `"NormalizeRules"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:16` (`eval_set`) | `"Set"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:59` (`eval_set_delayed`) | `"SetDelayed"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:69` (`eval_rule`) | `"Rule"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:87` (`eval_rule_delayed`) | `"RuleDelayed"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/hold_thread.rs:19` (`eval_release_hold`) | `"ReleaseHold"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:36` (`eval_abort`) | `"Abort"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:181` (`eval_abort_protect`) | `"AbortProtect"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:58` (`eval_break`) | `"Break"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:70` (`eval_continue`) | `"Continue"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:26` (`eval_failure_q`) | `"FailureQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:9` (`eval_match_q`) | `"MatchQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:12` (`eval_replace_all`) | `"ReplaceAll"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:16` (`eval_replace_repeated`) | `"ReplaceRepeated"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:36` (`eval_message_name`) | `"MessageName"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:12` (`eval_definition`) | `"Definition"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:34` (`eval_value_q`) | `"ValueQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:59` (`eval_own_values`) | `"OwnValues"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:71` (`eval_up_values`) | `"UpValues"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:86` (`eval_down_values`) | `"DownValues"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:98` (`eval_sub_values`) | `"SubValues"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:113` (`eval_default_values`) | `"DefaultValues"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/mod.rs:81` (`eval_missing_q`) | `"MissingQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:20` (`eval_syntax_information`) | `"SyntaxInformation"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:23` (`eval_options`) | `"Options"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:62` (`eval_declare_package`) | `"DeclarePackage"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/lexical/block.rs:6` (`eval_block`) | `"Block"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:11` (`eval_module`) | `"Module"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/lexical/with.rs:9` (`eval_with`) | `"With"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/arithmetic/stats.rs:13` (`eval_median`) | `"Median"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/arithmetic/stats.rs:46` (`eval_commonest`) | `"Commonest"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/arithmetic/stats.rs:103` (`eval_quantile`) | `"Quantile"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/arithmetic.rs:161` (`eval_mean`) | `"Mean"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes/digest.rs:48` (`eval_crc32`) | `"Crc32"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes.rs:22` (`eval_to_hex_string`) | `"ToHexString"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes.rs:42` (`eval_from_hex_string`) | `"FromHexString"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes.rs:65` (`eval_to_base64`) | `"ToBase64"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes.rs:85` (`eval_from_base64`) | `"FromBase64"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes.rs:108` (`eval_byte_array_q`) | `"ByteArrayQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes.rs:121` (`eval_byte_array_to_string`) | `"ByteArrayToString"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:12` (`eval_binomial`) | `"Binomial"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:32` (`eval_fibonacci`) | `"Fibonacci"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:48` (`eval_partitions_p`) | `"PartitionsP"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:73` (`eval_partitions_q`) | `"PartitionsQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:98` (`eval_bernoulli_b`) | `"BernoulliB"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:120` (`eval_subfactorial`) | `"Subfactorial"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:145` (`eval_stirling_s1`) | `"StirlingS1"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:171` (`eval_stirling_s2`) | `"StirlingS2"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:197` (`eval_catalan_number`) | `"CatalanNumber"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/combinatorics.rs:243` (`eval_pochhammer`) | `"Pochhammer"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:14` (`eval_date_list`) | `"DateList"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:29` (`eval_from_date_list`) | `"FromDateList"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:96` (`eval_from_date_string`) | `"FromDateString"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:121` (`eval_date_plus`) | `"DatePlus"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:136` (`eval_date_difference`) | `"DateDifference"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:193` (`eval_leap_year_q`) | `"LeapYearQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:208` (`eval_days_in_month`) | `"DaysInMonth"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/window.rs:84` (`eval_accumulate`) | `"Accumulate"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/logic/truth.rs:84` (`eval_implies`) | `"Implies"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/logic/truth.rs:105` (`eval_not`) | `"Not"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:20` (`eval_divisors`) | `"Divisors"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:46` (`eval_euler_phi`) | `"EulerPhi"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:74` (`eval_moebius_mu`) | `"MoebiusMu"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:105` (`eval_divisor_sigma`) | `"DivisorSigma"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:129` (`eval_prime`) | `"Prime"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:158` (`eval_prime_pi`) | `"PrimePi"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:183` (`eval_prime_nu`) | `"PrimeNu"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:208` (`eval_prime_omega`) | `"PrimeOmega"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/polynomials/mod.rs:26` (`eval_expand`) | `"Expand"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/polynomials/mod.rs:86` (`eval_coefficient_list`) | `"CoefficientList"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/polynomials/mod.rs:111` (`eval_together`) | `"Together"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/polynomials/mod.rs:127` (`eval_factor`) | `"Factor"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/basic.rs:9` (`eval_atom_q`) | `"AtomQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/basic.rs:21` (`eval_list_q`) | `"ListQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/basic.rs:35` (`eval_string_q`) | `"StringQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/basic.rs:50` (`eval_date_object_q`) | `"DateObjectQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/basic.rs:65` (`eval_time_object_q`) | `"TimeObjectQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/collections.rs:41` (`eval_duplicate_free_q`) | `"DuplicateFreeQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/collections.rs:231` (`eval_list_containment_relation`) | `name` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:9` (`eval_integer_q`) | `"IntegerQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:24` (`eval_number_q`) | `"NumberQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:39` (`eval_numeric_q`) | `"NumericQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:51` (`eval_even_q`) | `"EvenQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:70` (`eval_odd_q`) | `"OddQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:137` (`eval_machine_number_q`) | `"MachineNumberQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:149` (`eval_real_valued_number_q`) | `"RealValuedNumberQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:161` (`eval_real_valued_numeric_q`) | `"RealValuedNumericQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:173` (`eval_prime_q`) | `"PrimeQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:187` (`eval_algebraic_integer_q`) | `"AlgebraicIntegerQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:228` (`eval_exact_numeric_sign_predicate`) | `name` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/numeric.rs:246` (`eval_exact_numeric_sign_predicate_or_inert`) | `name` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/ordering.rs:31` (`eval_order`) | `"Order"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/ordering.rs:295` (`eval_numerical_order`) | `"NumericalOrder"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/ordering.rs:340` (`eval_between`) | `"Between"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/truth.rs:16` (`eval_true_q`) | `"TrueQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/truth.rs:31` (`eval_boolean_q`) | `"BooleanQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/truth.rs:43` (`eval_boole`) | `"Boole"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:17` (`eval_seed_random`) | `"SeedRandom"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:32` (`eval_reseed_random`) | `"ReseedRandom"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:310` (`eval_block_random`) | `"BlockRandom"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:322` (`eval_random_uuid`) | `"RandomUuid"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:338` (`eval_uuid_q`) | `"UuidQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:350` (`eval_random_bytes`) | `"RandomBytes"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/encoding.rs:89` (`eval_compress`) | `"Compress"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/parts.rs:15` (`eval_string_take`) | `"StringTake"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/parts.rs:46` (`eval_string_drop`) | `"StringDrop"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/parts.rs:81` (`eval_string_part`) | `"StringPart"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/parts.rs:110` (`eval_string_take_drop`) | `"StringTakeDrop"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/html.rs:9` (`eval_html_escape`) | `"HtmlEscape"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/html.rs:24` (`eval_html_unescape`) | `"HtmlUnescape"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/html.rs:30` (`eval_html_unescape`) | `"HtmlUnescape"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/paths.rs:9` (`eval_file_extension`) | `"FileExtension"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/paths.rs:24` (`eval_file_base_name`) | `"FileBaseName"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/paths.rs:39` (`eval_file_name_split`) | `"FileNameSplit"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/shell.rs:16` (`eval_shell_split`) | `"ShellSplit"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/shell.rs:39` (`eval_shell_quote`) | `"ShellQuote"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/text_shape.rs:68` (`eval_string_dedent`) | `"StringDedent"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:230` (`eval_edit_distance`) | `"EditDistance"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:259` (`eval_hamming_distance`) | `"HammingDistance"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:304` (`eval_longest_common_subsequence`) | `"LongestCommonSubsequence"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:330` (`eval_sequence_alignment`) | `"SequenceAlignment"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:357` (`eval_diff`) | `"Diff"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/template.rs:35` (`eval_string_template`) | `"StringTemplate"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:45` (`eval_string_length`) | `"StringLength"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:61` (`eval_to_upper_case`) | `"ToUpperCase"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:313` (`eval_string_lines`) | `"StringLines"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:329` (`eval_text_words`) | `"TextWords"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:448` (`eval_syntax_q`) | `"SyntaxQ"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:464` (`eval_text_string`) | `"TextString"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:483` (`eval_character_counts`) | `"CharacterCounts"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:499` (`eval_remove_diacritics`) | `"RemoveDiacritics"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:524` (`eval_characters`) | `"Characters"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:545` (`eval_character_range`) | `"CharacterRange"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:604` (`eval_string_repeat`) | `"StringRepeat"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:659` (`eval_string_value_predicate`) | `symbol` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/case.rs:9` (`eval_string_reverse`) | `"StringReverse"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/case.rs:25` (`eval_to_lower_case`) | `"ToLowerCase"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/case.rs:92` (`eval_string_scalar_predicate`) | `symbol` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/case.rs:110` (`eval_string_map`) | `symbol` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/decimal.rs:9` (`eval_decimal_string`) | `"DecimalString"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:61` (`eval_tally`) | `"Tally"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:203` (`eval_reverse_sort`) | `"ReverseSort"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:237` (`eval_reverse_sort_by`) | `"ReverseSortBy"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:270` (`eval_gather_by`) | `"GatherBy"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:537` (`eval_extreme_by`) | `symbol` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:585` (`eval_take_by`) | `symbol` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:41` (`eval_constant_array`) | `"ConstantArray"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/sequence/delete.rs:18` (`eval_delete`) | `"Delete"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/sequence/insert.rs:17` (`eval_insert`) | `"Insert"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/sequence/take_list.rs:12` (`eval_take_list`) | `"TakeList"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:24` (`eval_length`) | `"Length"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:67` (`eval_leaf_count`) | `"LeafCount"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:79` (`eval_full_form`) | `"FullForm"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:133` (`eval_rest`) | `"Rest"` |
| ↳ `invalid_call_with_exact_arg_count` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:165` (`eval_most`) | `"Most"` |

#### `invalid_call_with_arg_range` expansion (162 rows)

| helper | caller file:line | builtin symbol passed |
|---|---|---|
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/arithmetic/rounding.rs:110` (`eval_round`) | `"Round"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:39` (`eval_directory_name`) | `"DirectoryName"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:85` (`eval_file_name_take`) | `"FileNameTake"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:181` (`eval_file_names`) | `"FileNames"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/basic.rs:132` (`eval_rum_profile_snapshot`) | `"RumProfileSnapshot"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:11` (`eval_evaluation_data`) | `"EvaluationData"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:6` (`eval_monitor`) | `"Monitor"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/reap.rs:54` (`eval_reap`) | `"Reap"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:19` (`eval_time_constrained`) | `"TimeConstrained"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:80` (`eval_memory_constrained`) | `"MemoryConstrained"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:70` (`eval_trace`) | `"Trace"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:138` (`eval_trace_print`) | `"TracePrint"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:16` (`eval_if`) | `"If"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:68` (`eval_while`) | `"While"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:76` (`eval_until`) | `"Until"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:84` (`eval_for`) | `"For"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:32` (`eval_with_cleanup`) | `"WithCleanup"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:6` (`eval_enclose`) | `"Enclose"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:39` (`eval_confirm`) | `"Confirm"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:53` (`eval_confirm_by`) | `"ConfirmBy"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:70` (`eval_confirm_match`) | `"ConfirmMatch"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:87` (`eval_confirm_quiet`) | `"ConfirmQuiet"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:128` (`eval_confirm_assert`) | `"ConfirmAssert"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:17` (`eval_return`) | `"Return"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:43` (`eval_throw`) | `"Throw"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:79` (`eval_catch`) | `"Catch"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:9` (`eval_cases`) | `"Cases"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:12` (`eval_delete_cases`) | `"DeleteCases"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:33` (`eval_replace_list`) | `"ReplaceList"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:9` (`eval_replace`) | `"Replace"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:13` (`eval_replace`) | `"Replace"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:38` (`eval_replace_at`) | `"ReplaceAt"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:16` (`eval_replace_part`) | `"ReplacePart"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:22` (`eval_information`) | `"Information"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/arguments.rs:265` (`eval_arguments_options`) | `"ArgumentsOptions"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/arguments.rs:321` (`eval_check_arguments`) | `"CheckArguments"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:32` (`eval_set_options`) | `"SetOptions"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:86` (`eval_option_value`) | `"OptionValue"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:149` (`eval_option_value`) | `"OptionValue"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:106` (`eval_once_builtin`) | `"Once"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:176` (`eval_persist_package_state`) | `operator` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/unique.rs:61` (`eval_unique`) | `"Unique"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:9` (`eval_count`) | `"Count"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:31` (`eval_member_q`) | `"MemberQ"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:53` (`eval_free_q`) | `"FreeQ"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/level.rs:10` (`eval_level`) | `"Level"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/tree_walkers/position.rs:9` (`eval_position`) | `"Position"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/bytes/digest.rs:77` (`eval_digest`) | `symbol` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:72` (`eval_date_string`) | `"DateString"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/datetime.rs:183` (`eval_day_of_week`) | `"DayOfWeek"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:106` (`normalize_apply_call`) | `symbol` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:190` (`normalize_map_like_call`) | `symbol` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:259` (`normalize_map_all_call`) | `"MapAll"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:307` (`normalize_comap_call`) | `"Comap"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:394` (`normalize_comap_apply_call`) | `"ComapApply"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/generators.rs:46` (`eval_array`) | `"Array"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/generators.rs:92` (`eval_outer`) | `"Outer"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/generators.rs:98` (`eval_outer`) | `"Outer"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:21` (`eval_inner`) | `"Inner"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:66` (`eval_array_filter`) | `"ArrayFilter"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:113` (`eval_array_reduce`) | `"ArrayReduce"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/curried.rs:27` (`eval_operator_applied`) | `"OperatorApplied"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/curried.rs:66` (`eval_curry_applied`) | `"CurryApplied"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:9` (`eval_through`) | `"Through"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:38` (`eval_operate`) | `"Operate"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/reverse.rs:22` (`eval_reverse_applied`) | `"ReverseApplied"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:70` (`eval_nest`) | `"Nest"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:115` (`eval_nest_while`) | `"NestWhile"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:147` (`eval_nest_list`) | `"NestList"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:192` (`eval_nest_while_list`) | `"NestWhileList"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:215` (`eval_fixed_point_list`) | `"FixedPointList"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:238` (`eval_fixed_point`) | `"FixedPoint"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:30` (`eval_fold`) | `"Fold"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:45` (`eval_fold_list`) | `"FoldList"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:76` (`eval_fold_pair`) | `"FoldPair"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:127` (`eval_fold_while`) | `"FoldWhile"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:150` (`eval_sequence_fold`) | `"SequenceFold"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:173` (`eval_sequence_fold_list`) | `"SequenceFoldList"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/select.rs:69` (`eval_select`) | `"Select"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/select.rs:100` (`eval_select_first`) | `"SelectFirst"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/window.rs:74` (`eval_block_map`) | `"BlockMap"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/window.rs:108` (`eval_take_while`) | `"TakeWhile"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/window.rs:126` (`eval_length_while`) | `"LengthWhile"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:63` (`eval_subset_map`) | `"SubsetMap"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:96` (`eval_map_at`) | `"MapAt"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:143` (`eval_map_thread`) | `"MapThread"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:15` (`eval_integer_length`) | `"IntegerLength"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:48` (`eval_integer_digits`) | `"IntegerDigits"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:107` (`eval_integer_reverse`) | `"IntegerReverse"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:142` (`eval_from_digits`) | `"FromDigits"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/integer_digits.rs:183` (`eval_digit_count`) | `"DigitCount"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/number_theory.rs:238` (`eval_next_prime`) | `"NextPrime"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/polynomials/mod.rs:42` (`eval_coefficient`) | `"Coefficient"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/arrays.rs:30` (`eval_vector_q`) | `"VectorQ"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/arrays.rs:53` (`eval_matrix_q`) | `"MatrixQ"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/arrays.rs:63` (`eval_array_q`) | `"ArrayQ"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/ordering.rs:45` (`eval_ordered_q`) | `"OrderedQ"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/ordering.rs:160` (`eval_lexicographic_order`) | `"LexicographicOrder"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/truth.rs:89` (`eval_truth_aggregate`) | `symbol` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates.rs:200` (`eval_polynomial_q`) | `"PolynomialQ"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates.rs:254` (`eval_polynomial_like_expression_q`) | `symbol` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:77` (`eval_random_integer`) | `"RandomInteger"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:89` (`eval_random_choice`) | `"RandomChoice"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:217` (`eval_random_sample`) | `"RandomSample"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:256` (`eval_random_string`) | `"RandomString"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/encoding.rs:18` (`eval_string_to_byte_array`) | `"StringToByteArray"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/encoding.rs:52` (`eval_hash`) | `"Hash"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/encoding.rs:101` (`eval_uncompress`) | `"Uncompress"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/encoding.rs:109` (`eval_uncompress`) | `"Uncompress"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:45` (`eval_string_position`) | `"StringPosition"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:80` (`eval_string_split`) | `"StringSplit"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:156` (`eval_string_trim`) | `"StringTrim"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:192` (`eval_string_replace`) | `"StringReplace"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:235` (`eval_string_cases`) | `"StringCases"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:296` (`eval_string_count`) | `"StringCount"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:338` (`eval_string_delete`) | `"StringDelete"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:385` (`eval_string_apply`) | `"StringApply"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:416` (`eval_string_pattern_predicate`) | `symbol` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/patterns/search.rs:458` (`eval_string_patterns_predicate`) | `symbol` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/args.rs:9` (`eval_parse_args`) | `"ParseArgs"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/text_shape.rs:9` (`eval_string_wrap`) | `"StringWrap"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/text_shape.rs:39` (`eval_string_indent`) | `"StringIndent"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/text_shape.rs:90` (`eval_string_expand_tabs`) | `"StringExpandTabs"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/text_shape.rs:112` (`eval_align_columns`) | `"AlignColumns"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/url.rs:11` (`eval_url_encode`) | `"UrlEncode"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/url.rs:29` (`eval_url_decode`) | `"UrlDecode"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/url.rs:50` (`eval_url_parse`) | `"UrlParse"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:62` (`eval_nearest`) | `"Nearest"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:172` (`eval_pluralize`) | `"Pluralize"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:191` (`eval_alphabet`) | `"Alphabet"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/similarity/mod.rs:204` (`eval_alphabetic_sort`) | `"AlphabeticSort"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/template.rs:16` (`eval_string_form`) | `"StringForm"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/template.rs:70` (`eval_template_apply`) | `"TemplateApply"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/template.rs:83` (`render_template_apply`) | `"TemplateApply"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:82` (`eval_string_riffle`) | `"StringRiffle"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:177` (`eval_row`) | `"Row"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:206` (`eval_string_pad_left`) | `"StringPadLeft"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:240` (`eval_string_pad_right`) | `"StringPadRight"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:364` (`eval_to_character_code`) | `"ToCharacterCode"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:413` (`eval_to_expression`) | `"ToExpression"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/text/builtins.rs:582` (`eval_to_string`) | `"ToString"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/collection_stats.rs:320` (`eval_riffle`) | `"Riffle"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:11` (`eval_subsets`) | `"Subsets"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:57` (`eval_tuples`) | `"Tuples"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:87` (`eval_partition`) | `"Partition"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/dedup.rs:85` (`eval_delete_duplicates_by`) | `"DeleteDuplicatesBy"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/flatten.rs:29` (`eval_flatten`) | `"Flatten"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_lookup.rs:72` (`eval_select_positions`) | `"SelectPositions"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_lookup.rs:82` (`eval_duplicates`) | `"Duplicates"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_lookup.rs:109` (`eval_binary_search`) | `"BinarySearch"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:16` (`eval_differences`) | `"Differences"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:102` (`eval_pad`) | `symbol` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:58` (`eval_sort`) | `"Sort"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:108` (`eval_sort_by`) | `"SortBy"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/reordering.rs:16` (`eval_reverse`) | `"Reverse"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/reordering.rs:39` (`eval_transpose`) | `"Transpose"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:14` (`eval_head`) | `"Head"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:52` (`eval_depth`) | `"Depth"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:102` (`eval_first`) | `"First"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:123` (`eval_last`) | `"Last"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:210` (`eval_append`) | `"Append"` |
| ↳ `invalid_call_with_arg_range` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/surface.rs:230` (`eval_prepend`) | `"Prepend"` |

#### `invalid_call_with_arg_form` expansion (159 rows)

| helper | caller file:line | builtin symbol passed |
|---|---|---|
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/attributes.rs:35` (`eval_attributes`) | `"Attributes"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/attributes.rs:75` (`eval_update_attributes`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/attributes.rs:116` (`finish_update_attributes_after_resolved_attributes`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/attributes.rs:156` (`eval_protect_like`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:13` (`eval_file_name_join`) | `"FileNameJoin"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:43` (`eval_directory_name`) | `"DirectoryName"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:65` (`eval_parent_directory`) | `"ParentDirectory"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:89` (`eval_file_name_take`) | `"FileNameTake"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:88` (`eval_context`) | `"Context"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:107` (`eval_contexts`) | `"Contexts"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:206` (`eval_find_file`) | `"FindFile"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:247` (`eval_get`) | `"Get"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:302` (`eval_needs`) | `"Needs"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:347` (`eval_begin`) | `"Begin"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_context_loader.rs:405` (`eval_begin_package`) | `"BeginPackage"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/builtins_input.rs:44` (`eval_dialog`) | `"Dialog"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/basic.rs:16` (`eval_short`) | `"Short"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/message_switch.rs:33` (`eval_message_switch`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:14` (`eval_monitor`) | `"Monitor"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:29` (`eval_monitor`) | `"Monitor"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/properties.rs:33` (`project_association_properties`) | `head_name` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/properties.rs:44` (`project_association_properties`) | `head_name` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:47` (`resume_time_constrained_with_limit`) | `"TimeConstrained"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:108` (`resume_memory_constrained_with_limit`) | `"MemoryConstrained"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:37` (`eval_trace`) | `"Trace"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:53` (`eval_trace`) | `"Trace"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:62` (`eval_trace`) | `"Trace"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:105` (`eval_trace_print`) | `"TracePrint"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:121` (`eval_trace_print`) | `"TracePrint"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:130` (`eval_trace_print`) | `"TracePrint"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:255` (`invalid_cases_traversal_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:284` (`invalid_traversal_without_limit_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:20` (`eval_unset`) | `"Unset"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:38` (`eval_remove`) | `"Remove"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:70` (`eval_clear_like`) | `name` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:40` (`eval_tag_set`) | `"TagSet"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:68` (`eval_tag_set_delayed`) | `"TagSetDelayed"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definition_tags.rs:85` (`eval_tag_unset`) | `"TagUnset"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:106` (`resume_normalize_rules`) | `"NormalizeRules"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:212` (`finish_definition_assignment`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:229` (`finish_definition_assignment`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:246` (`finish_definition_assignment`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:275` (`finish_definition_assignment`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/definitions/definitions.rs:305` (`finish_definition_assignment`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/hold_thread.rs:44` (`eval_thread`) | `"Thread"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:57` (`finish_do_spec_eval`) | `"Do"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:158` (`finish_table_spec_eval`) | `"Table"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:88` (`finish_iteration_aggregate_spec_eval`) | `aggregate.name()` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:38` (`eval_which`) | `"Which"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:50` (`eval_switch`) | `"Switch"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:17` (`eval_check`) | `"Check"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:64` (`eval_quiet`) | `"Quiet"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:93` (`eval_confirm_quiet`) | `"ConfirmQuiet"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:42` (`resume_replace_all_after_rules`) | `"ReplaceAll"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:51` (`resume_replace_list_after_target`) | `"ReplaceList"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:82` (`resume_replace_list_after_rule_exprs`) | `"ReplaceList"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:71` (`resume_replace_after_rule_exprs`) | `"Replace"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:87` (`resume_replace_after_rule_exprs`) | `"Replace"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:98` (`resume_replace_after_rule_exprs`) | `"Replace"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:70` (`resume_replace_repeated_after_rule_exprs`) | `"ReplaceRepeated"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:84` (`resume_replace_repeated_after_rule_exprs`) | `"ReplaceRepeated"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:51` (`resume_replace_at_after_target`) | `"ReplaceAt"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:82` (`resume_replace_at_after_rule_exprs`) | `"ReplaceAt"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:21` (`eval_replace_part`) | `"ReplacePart"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:17` (`eval_message`) | `"Message"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:40` (`eval_message_name`) | `"MessageName"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:66` (`eval_messages`) | `"Messages"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:89` (`eval_message_definitions`) | `"MessageDefinitions"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:116` (`eval_message_tags`) | `"MessageTags"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:9` (`eval_definition`) | `"Definition"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:56` (`eval_own_values`) | `"OwnValues"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:68` (`eval_up_values`) | `"UpValues"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:83` (`eval_down_values`) | `"DownValues"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:95` (`eval_sub_values`) | `"SubValues"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:110` (`eval_default_values`) | `"DefaultValues"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/namespace_tools/names.rs:70` (`eval_names`) | `"Names"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:11` (`eval_syntax_information`) | `"SyntaxInformation"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/arguments.rs:269` (`eval_arguments_options`) | `"ArgumentsOptions"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/arguments.rs:277` (`eval_arguments_options`) | `"ArgumentsOptions"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/arguments.rs:286` (`eval_arguments_options`) | `"ArgumentsOptions"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/arguments.rs:325` (`eval_check_arguments`) | `"CheckArguments"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/arguments.rs:333` (`eval_check_arguments`) | `"CheckArguments"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/arguments.rs:342` (`eval_check_arguments`) | `"CheckArguments"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:14` (`eval_options`) | `"Options"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:36` (`eval_set_options`) | `"SetOptions"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:78` (`eval_option_value`) | `"OptionValue"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:93` (`eval_option_value`) | `"OptionValue"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:102` (`eval_option_value`) | `"OptionValue"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:114` (`eval_option_value`) | `"OptionValue"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:123` (`eval_option_value`) | `"OptionValue"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:163` (`finish_set_options_from_resolved`) | `"SetOptions"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/options/builtins.rs:234` (`finish_option_value_from_explicit`) | `"OptionValue"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:49` (`eval_declare_package`) | `"DeclarePackage"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:56` (`eval_declare_package`) | `"DeclarePackage"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:101` (`eval_once_builtin`) | `"Once"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:167` (`eval_persist_package_state`) | `operator` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:180` (`eval_persist_package_state`) | `operator` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:190` (`eval_persist_package_state`) | `operator` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/package_tools.rs:238` (`eval_persist_package_state`) | `operator` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/lexical/block.rs:12` (`eval_block`) | `"Block"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:27` (`eval_module`) | `"Module"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/lexical/with.rs:15` (`eval_with`) | `"With"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/lexical/with.rs:27` (`eval_staged_with`) | `"With"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-runtime/src/runtime/scoping/unique.rs:52` (`eval_unique`) | `"Unique"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:135` (`normalize_apply_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:143` (`normalize_apply_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:152` (`normalize_apply_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:159` (`normalize_apply_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:213` (`normalize_map_like_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:224` (`normalize_map_like_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:231` (`normalize_map_like_call`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:279` (`normalize_map_all_call`) | `"MapAll"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:341` (`normalize_comap_call`) | `"Comap"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:353` (`normalize_comap_call`) | `"Comap"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:360` (`normalize_comap_call`) | `"Comap"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:421` (`normalize_comap_apply_call`) | `"ComapApply"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/generators.rs:50` (`eval_array`) | `"Array"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/generators.rs:57` (`eval_array`) | `"Array"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/generators.rs:69` (`eval_array`) | `"Array"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:70` (`eval_array_filter`) | `"ArrayFilter"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:77` (`eval_array_filter`) | `"ArrayFilter"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:94` (`eval_array_filter`) | `"ArrayFilter"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:117` (`eval_array_reduce`) | `"ArrayReduce"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:124` (`eval_array_reduce`) | `"ArrayReduce"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/curried.rs:22` (`eval_operator_applied`) | `"OperatorApplied"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/curried.rs:41` (`eval_curry_applied`) | `"CurryApplied"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/curried.rs:61` (`eval_curry_applied`) | `"CurryApplied"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:29` (`eval_operate`) | `"Operate"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/callable/reverse.rs:17` (`eval_reverse_applied`) | `"ReverseApplied"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:50` (`eval_nest`) | `"Nest"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:61` (`eval_nest`) | `"Nest"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:95` (`eval_nest_while`) | `"NestWhile"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:106` (`eval_nest_while`) | `"NestWhile"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:127` (`eval_nest_list`) | `"NestList"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:138` (`eval_nest_list`) | `"NestList"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:172` (`eval_nest_while_list`) | `"NestWhileList"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:183` (`eval_nest_while_list`) | `"NestWhileList"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:206` (`eval_fixed_point_list`) | `"FixedPointList"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/iteration.rs:229` (`eval_fixed_point`) | `"FixedPoint"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/sequence_fold.rs:86` (`apply_sequence_fold`) | `if collect_all { "SequenceFoldList" } else { "SequenceFold" }` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/sequence_fold.rs:102` (`apply_sequence_fold`) | `if collect_all { "SequenceFoldList" } else { "SequenceFold" }` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs:159` (`apply_fold_while`) | `"FoldWhile"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/select.rs:33` (`eval_select`) | `"Select"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/select.rs:60` (`eval_select`) | `"Select"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/window.rs:47` (`eval_block_map`) | `"BlockMap"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/window.rs:58` (`eval_block_map`) | `"BlockMap"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/sequence/window.rs:65` (`eval_block_map`) | `"BlockMap"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:47` (`eval_subset_map`) | `"SubsetMap"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:80` (`eval_map_at`) | `"MapAt"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:118` (`eval_map_thread`) | `"MapThread"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:126` (`eval_map_thread`) | `"MapThread"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/functional_ops/targeted_map.rs:157` (`finish_map_thread`) | `"MapThread"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates/truth.rs:94` (`eval_truth_aggregate`) | `truth_aggregate_name(kind)` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates.rs:190` (`eval_polynomial_q`) | `"PolynomialQ"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/predicates.rs:245` (`eval_polynomial_like_expression_q`) | `symbol` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/random.rs:275` (`eval_random_password`) | `"RandomPassword"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/text_shape.rs:16` (`eval_string_wrap`) | `"StringWrap"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/strings/scripting/text_shape.rs:96` (`eval_string_expand_tabs`) | `"StringExpandTabs"` |
| ↳ `invalid_call_with_arg_form` (one-level expansion; stop) | `crates/rum-stdlib/src/structure_ops/list_lookup.rs:55` (`eval_select_positions`) | `"SelectPositions"` |

Table-B total: **941 all-rows entries**: 303 direct caller sites (130 for the
40 non-generic helpers plus 173 for `invalid_call_with_message`) and 638
explicit one-level HELPER-NESTED entries. Repeated nested call sites are kept
when they fan into different Table-A helpers; this table counts fan-in rows,
not unique source locations.

The helper/fan-in count command was:

```text
$ python3 - <<'PY'
from pathlib import Path
import re
t = Path('docs/dev/message-registry-inventory.md').read_text()
a = t.split('## Table A', 1)[1].split('## Table B', 1)[0]
a_rows = [line for line in a.splitlines() if re.match(r'^\| \d+ \|', line)]
shared = [line.rsplit('|', 2)[1].strip() for line in a_rows
          if line.rsplit('|', 2)[1].strip() != '—']
b = t.split('## Table B', 1)[1].split('## Table C', 1)[0]
first, generic = b.split('### Generic `invalid_call_with_message` fan-in', 1)
def rows(section):
    return [line for line in section.splitlines()
            if line.startswith('| ') and not line.startswith('|---')
            and 'helper | caller' not in line]
first_rows, generic_rows = rows(first), rows(generic)
first_nested = [line for line in first_rows if line.startswith('| ↳')]
generic_nested = [line for line in generic_rows if line.startswith('| ↳')]
print('Table-A shared sites/helpers:', len(shared), len(set(shared)))
print('non-generic direct/nested:', len(first_rows)-len(first_nested), len(first_nested))
print('generic direct/nested:', len(generic_rows)-len(generic_nested), len(generic_nested))
for helper in ('invalid_call_with_exact_arg_count', 'invalid_call_with_arg_range',
               'invalid_call_with_arg_form'):
    print(helper + ':', sum(helper in line for line in generic_nested))
print('Table-B direct/nested/total:',
      len(first_rows)+len(generic_rows)-len(first_nested)-len(generic_nested),
      len(first_nested)+len(generic_nested), len(first_rows)+len(generic_rows))
PY
Table-A shared sites/helpers: 56 41
non-generic direct/nested: 130 81
generic direct/nested: 173 557
invalid_call_with_exact_arg_count: 196
invalid_call_with_arg_range: 162
invalid_call_with_arg_form: 159
Table-B direct/nested/total: 303 638 941
```

## Table C — emission paths that do not go through `push_message`

### C1. `runtime_message_for_non_loader_eval_error`

| `EvalError` variant | tag | message text | attributed symbol |
|---|---|---|---|
| `EvaluationCycleDetected { expr }` | `passlim` | `evaluation cycle detected while re-entering \`{expr}\`` | `General` |
| `RecursionLimitExceeded { limit }` | `reclim` | `evaluation exceeded recursion limit ({limit})` | `General` |
| `MatchRecursionLimitExceeded { limit }` | `matchreclim` | `evaluation exceeded pattern-match recursion limit ({limit})` | `General` |
| `Cancelled` | `cancel` | `evaluation cancelled by host` | `General` |
| `TimeConstraintExceeded` | `timelim` | `evaluation exceeded TimeConstrained limit` | `General` |
| `MemoryConstraintExceeded { limit }` | `memlim` | `evaluation exceeded MemoryConstrained limit ({limit})` | `General` |
| `StackLimitExceeded { limit }` | `reslim` | `evaluation exceeded native stack limit ({limit} bytes)` | `General` |
| `IntegerWorkLimitExceeded { limit }` | `reslim` | `evaluation exceeded integer work limit ({limit})` | `General` |
| `GeneratedElementLimitExceeded { limit }` | `reslim` | `evaluation exceeded generated element limit ({limit})` | `General` |
| `IterationWorkLimitExceeded { limit }` | `reslim` | `evaluation exceeded iteration work limit ({limit})` | `General` |
| `IterationLimitExceeded { limit }` | `iterlim` | `ReplaceRepeated exceeded iteration limit ({limit})` | `General` |
| `EvaluationPassLimitExceeded { limit }` | `passlim` | `evaluation exceeded fixed-point pass limit ({limit})` | `General` |

The mapping is at
`crates/rum-runtime/src/runtime/session/session_helpers.rs:187-209`; the exact
text comes from the `thiserror` display declarations at
`crates/rum-runtime/src/runtime/session/session_state.rs:301-324` because the
constructor stores `error.to_string()`. `emit_non_loader_eval_message`
(`session_helpers.rs:267-285`) deduplicates the constructed value within its
checkpoint, checks `message_enabled`, and then appends directly to
`self.messages`; it does not call `push_message`.

### C2. Other `RuntimeMessage {` construction sites

Search command and result:

```text
$ rg -n --glob '*.rs' 'RuntimeMessage\s*\{' crates
crates/rum-runtime/src/runtime/session/session_helpers.rs:171:        self.messages.push(RuntimeMessage {
crates/rum-runtime/src/runtime/session/session_helpers.rs:206:        Some(RuntimeMessage {
crates/rum-runtime/src/runtime/model/messages.rs:44:pub struct RuntimeMessage {
crates/rum-runtime/src/runtime/model/messages.rs:106:impl fmt::Display for RuntimeMessage {
crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:121:            normalized.push(runtime_message_name_expr(RuntimeMessage {
crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:330:            Some(RuntimeMessage {
```

The six textual matches classify as follows:

| file:line | classification | fields / effect |
|---|---|---|
| `session_helpers.rs:171` | The ordinary `push_message` constructor | `symbol`, `tag`, and `text` are copied/converted from its three arguments and appended. |
| `session_helpers.rs:206` | The C1 non-loader constructor | Always `symbol = "General"`; tag is the mapping above; text is `error.to_string()`. It is emitted later by the direct append at line 283. |
| `model/messages.rs:44` | Type declaration, not a construction | Declares the three owned `String` fields. |
| `model/messages.rs:106` | `Display` impl header, not a construction | Renders `{symbol}::{tag}: {text}`. |
| `debug_tools/test_report/evaluation.rs:121` | Synthetic normalization value, not an emission | Converts an expected `MessageName` to `RuntimeMessage { symbol: key.symbol.to_string(), tag: key.tag, text: String::new() }`, immediately converts it back to a message-name expression, and never appends it. |
| `debug_tools/test_report/evaluation.rs:330` | Continuation decoding, not a new emission | Reconstructs all three fields from a serialized three-string list; the value restores already-recorded test-report messages. |

The append-side cross-check was:

```text
$ rg -n --glob '*.rs' '\.messages\.push\s*\(' crates
crates/rum-runtime/src/runtime/session/session_helpers.rs:171:        self.messages.push(RuntimeMessage {
crates/rum-runtime/src/runtime/session/session_helpers.rs:283:            self.messages.push(message);
crates/rum-runtime/src/runtime/session/session_helpers.rs:448:                self.messages.push(message);
crates/rum-jupyter/src/transport/local.rs:15:        self.messages.push(message);
```

Line 283 is the C1 emission. `session_helpers.rs:448` only re-inserts existing
trailing messages that survive `suppress_messages_since`; it neither
constructs nor originates a diagnostic. `rum-jupyter`'s line 15 appends a
Jupyter transport message, not `runtime::RuntimeMessage`.

### C3. Loader-side sibling

No `runtime_message_for_loader_eval_error` (or other loader-side
`RuntimeMessage` constructor) exists. The search was:

```text
$ rg -n --glob '*.rs' 'runtime_message_for_.*loader|loader.*RuntimeMessage|RuntimeMessage.*loader' crates/rum-runtime/src
crates/rum-runtime/src/runtime/session/session_helpers.rs:14:    handled_non_loader_eval_errors: Vec<RuntimeMessage>,
crates/rum-runtime/src/runtime/session/session_helpers.rs:187:    pub(in crate::runtime) fn runtime_message_for_non_loader_eval_error(
crates/rum-runtime/src/runtime/session/session_helpers.rs:270:        let Some(message) = Self::runtime_message_for_non_loader_eval_error(error) else {
crates/rum-runtime/src/runtime/model/session.rs:81:    pub(in crate::runtime) handled_non_loader_eval_errors: Vec<RuntimeMessage>,
```

The loader-side path instead is `emit_loader_message(symbol, error)` at
`session_helpers.rs:332-355`, and all of its emissions go through the Table-A
sites:

| loader error | tag / text | attributed symbol |
|---|---|---|
| `LoadPathNotFound { target, searched }` | `notfound`; `could not locate \`{target}\`` when `searched` is empty, otherwise `could not locate \`{target}\` in {searched.join(", ")}` | fn parameter `symbol` (`FindFile` or `Get` at current callers) |
| `Io { path, source }` | `io`; `could not read \`{path}\`: {source}` | fn parameter `symbol` |
| `Syntax(err)` | `syntax`; `err.to_string()` | fn parameter `symbol` |
| `NoFileIoProvider` | delegates to `emit_no_file_io_message`: `noio`; `no file provider installed` | `General` |
| every other `EvalError` | `load`; `other.to_string()` | fn parameter `symbol` |

`eval_get` also extends the parent session with messages already produced by
its child file-evaluation outcome (`builtins_context_loader.rs:272`); that is
a transfer of existing messages, not another construction path.

## Section D — tag vocabulary

The word “overall” has two materially different answers:

- The runtime-wide vocabulary is **open/unbounded**. `eval_message` accepts an
  arbitrary normalized `MessageName` symbol/tag pair and calls `push_message`
  (`builtins_messages.rs:13-27`). `ArgumentsOptions` and `CheckArguments` also
  attribute `argx`, `argt`, `optx`, or `nonopt` to the arbitrary symbolic head
  of the held call they inspect (`options/arguments.rs:258-355`).
- The **closed current-source builtin diagnostic set**—the useful registry
  baseline after excluding those two user-selected/open families—contains
  **144 distinct tags and 966 distinct `(symbol, tag)` pairs**.

Counting rule for the closed set: start with literal symbol/tag arguments in
Table A, direct `invalid_call_with_message` callers, and the three standard
wrapper caller sets; expand fn-parameter symbols through Table B; inspect and
expand closed enums/name functions (`DateTimeError::tag`, JSON parse tags,
assert tags, key-selection tags, `UpTo`, tensor, operator-wrapper symbols, and
`EvalError` mappings); then deduplicate exact pairs. The loader expansion is
path-sensitive: `FindFile` contributes only `notfound`, while `Get` contributes
`notfound`, `io`, `syntax`, and `load`. Unresolved user symbols are not replaced
with every installed builtin, because doing so would falsely turn an open
runtime mechanism into declarations owned by each builtin.

The literal-only source baseline was 93 tags / 604 pairs with 133 dynamic
records. Closed helper/tag expansion adds 51 new tags and 362 net-new pairs.
Counts below are **distinct-symbol usage counts**: the number of closed
`(symbol, tag)` pairs bearing each tag, not the number of branches or duplicate
call sites.

Manual expansion cross-checks opened four unlike dynamic cases rather than
inferring them from call-line text: `builtins_messages.rs:13-27` confirms the
arbitrary/open `MessageName` pair; `options/arguments.rs:258-355` confirms the
arbitrary held-call head with its finite four-tag set; `datetime.rs:150-218`
confirms the finite `DateTimeError::tag()` expansion; and
`session_helpers.rs:332-355` together with the callers in
`builtins_context_loader.rs:124-278` confirms the path-sensitive loader pairs.
Those checks are reflected in the open-family exclusion, the `date` count of
9, and the `FindFile`/`Get` loader pairs respectively.

### Closed tag vocabulary and per-tag pair usage

| tag | distinct symbol usages |
|---|---:|
| `align` | 1 |
| `arg2` | 1 |
| `arg3` | 1 |
| `argb` | 4 |
| `argform` | 137 |
| `argm` | 3 |
| `argr` | 3 |
| `argrx` | 5 |
| `argt` | 18 |
| `argtu` | 171 |
| `argx` | 273 |
| `asrte` | 1 |
| `asrtf` | 1 |
| `asrtfe` | 1 |
| `asrttf` | 1 |
| `base64` | 1 |
| `blnoval` | 2 |
| `bytearray` | 8 |
| `cancel` | 1 |
| `context` | 2 |
| `csv` | 2 |
| `date` | 9 |
| `depth` | 1 |
| `dim` | 1 |
| `dot` | 1 |
| `drop` | 1 |
| `dvar` | 1 |
| `dvord` | 1 |
| `empt` | 1 |
| `empty` | 6 |
| `entity` | 1 |
| `entropy` | 3 |
| `env` | 1 |
| `escape` | 2 |
| `fldep` | 1 |
| `flev` | 1 |
| `flpi` | 1 |
| `flrep` | 1 |
| `format` | 4 |
| `heads` | 5 |
| `headsd` | 1 |
| `hex` | 1 |
| `ibase` | 4 |
| `idim` | 1 |
| `ilsm` | 2 |
| `ilsmn` | 2 |
| `ilsmp` | 2 |
| `incpt` | 1 |
| `ini` | 2 |
| `innf` | 4 |
| `ins` | 1 |
| `intm` | 1 |
| `intnn` | 1 |
| `intpp` | 1 |
| `invak` | 5 |
| `invar` | 3 |
| `invas` | 4 |
| `invl` | 1 |
| `invlb` | 1 |
| `invrk` | 3 |
| `invrl` | 9 |
| `invrlf` | 1 |
| `invrp` | 5 |
| `invru` | 3 |
| `io` | 3 |
| `iseq` | 1 |
| `iterlim` | 1 |
| `json` | 2 |
| `keynx` | 1 |
| `keyw` | 1 |
| `ksamp` | 1 |
| `kval` | 1 |
| `len` | 1 |
| `level` | 7 |
| `list` | 1 |
| `list1` | 2 |
| `load` | 2 |
| `matchreclim` | 1 |
| `memlim` | 1 |
| `ninv` | 1 |
| `nmtx` | 1 |
| `nninfseq` | 1 |
| `nocatch` | 1 |
| `nodef` | 1 |
| `noentropy` | 1 |
| `nofirst` | 1 |
| `nofwd` | 2 |
| `noio` | 1 |
| `nolast` | 1 |
| `nomost` | 1 |
| `nonopt` | 12 |
| `norest` | 1 |
| `normal` | 34 |
| `normal1` | 1 |
| `nosym` | 2 |
| `notfound` | 2 |
| `npart` | 1 |
| `nstr` | 1 |
| `numnm` | 1 |
| `optnf` | 2 |
| `optrs` | 3 |
| `opttf` | 8 |
| `opttfa` | 1 |
| `optval` | 2 |
| `optx` | 14 |
| `partd` | 2 |
| `partw` | 5 |
| `passlim` | 1 |
| `password` | 1 |
| `path` | 1 |
| `permspec` | 1 |
| `pers` | 1 |
| `pkspec1` | 2 |
| `pmix` | 1 |
| `psl1` | 1 |
| `pspec1` | 2 |
| `range` | 3 |
| `reclim` | 1 |
| `reslim` | 1 |
| `rmptc` | 1 |
| `rvalue` | 13 |
| `sandbox` | 1 |
| `schema` | 2 |
| `seqs` | 1 |
| `setval` | 4 |
| `shape` | 1 |
| `slot` | 2 |
| `span` | 3 |
| `string` | 1 |
| `syntax` | 3 |
| `tagnf` | 3 |
| `tagpos` | 3 |
| `take` | 4 |
| `tdlen` | 1 |
| `timelim` | 1 |
| `tok` | 2 |
| `toml` | 2 |
| `usym` | 1 |
| `utf8` | 1 |
| `weights` | 1 |
| `write` | 1 |
| `wrsym` | 26 |
| `xml` | 2 |
| `yaml` | 2 |

### Top shared-tag text uniformity

“Uniform” means the same text template after substituting runtime values;
literal values and placeholder values may differ. The cutoff here is at least
five distinct-symbol usages in the closed pair set.

| tag | usages | uniform across emitting sites? | factual text-shape note |
|---|---:|---|---|
| `argx` | 273 | No | The exact-count helper is uniform, but numerous direct/custom arity diagnostics use different wording. |
| `argtu` | 171 | Yes | All closed uses flow through `invalid_call_with_arg_range`: `expected between {min} and {max} argument(s), got {actual}`. |
| `argform` | 137 | No | The helper deliberately accepts caller-supplied detail text. |
| `normal` | 34 | No | Most say a nonatomic expression was expected, but call/position wording and direct family messages vary. |
| `wrsym` | 26 | Yes | All use `Symbol {symbol} is Protected.`; the stdlib `AssociateTo` site matches the runtime helper. |
| `argt` | 18 | No | Several one-or-two/range and type diagnostics use the same tag with different wording. |
| `optx` | 14 | No | Unknown-option diagnostics render different calls, option syntaxes, and family-specific wording. |
| `rvalue` | 13 | Yes | All use `{target} is not a variable with a value, so its value cannot be changed.` |
| `nonopt` | 12 | No | The common option-suffix form is reused, but Input/CheckAbort and association-option paths vary. |
| `date` | 9 | No | `DateTimeError` stores caller-specific preformatted messages under one tag. |
| `invrl` | 9 | No | Association/rule-list validation failures describe different accepted shapes and offenders. |
| `opttf` | 8 | Yes | All closed uses come from the Heads-option helper: `Value of option {option} -> {value} should be True or False.` |
| `bytearray` | 8 | Yes | Every site passes the literal `invalid ByteArray payload`. |
| `level` | 7 | Yes | All use the one `invalid_level_spec_call` template. |
| `empty` | 6 | No | Empty statistical, random, and collection-selection cases use different text. |
| `argrx` | 5 | No | Exact-two mutation/key predicates and CheckAbort’s at-least-two case differ. |
| `heads` | 5 | No | Join, ordering-set, and Outer-style head mismatch diagnostics do not share one template. |
| `invak` | 5 | No | Invalid-association/key mutation contexts render different calls/targets. |
| `invrp` | 5 | No | Invalid rule/rule-pair diagnostics vary by aggregation and association operation. |
| `partw` | 5 | Yes | All use `Part {part} of {target} does not exist.` |

### Count verification command

```text
$ python3 - <<'PY'
from pathlib import Path
import re
text = Path("docs/dev/message-registry-inventory.md").read_text()
section = text.split("### Closed tag vocabulary and per-tag pair usage", 1)[1]
section = section.split("### Top shared-tag text uniformity", 1)[0]
rows = re.findall(r"^\| `([^`]+)` \| (\d+) \|$", section, re.M)
print(f"distinct tags: {len(rows)}")
print(f"distinct closed pairs: {sum(int(count) for _, count in rows)}")
PY
distinct tags: 144
distinct closed pairs: 966
```

## Section E — what the existing catalog machinery reads today

There are two separate stores in `Session` today
(`session_state.rs:150-156`):

- `message_definitions: BTreeMap<MessageNameKey, MessageDefinition>` stores
  templates created through message-name assignment. `define_message` updates
  this map (`message_catalog.rs:72-92`); the assignment path reaches it at
  `definitions.rs:364`.
- `messages: Vec<RuntimeMessage>` and `last_message_list` store emitted,
  already-rendered instances. `push_message` checks
  `disabled_message_selectors`, then appends only to `messages`
  (`session_helpers.rs:151-174`). It never updates `message_definitions`.

| surface | store/path consulted | do builtin `push_message` emissions enter what this surface reads? |
|---|---|---|
| `Messages[f]` | `eval_messages` delegates to `message_names_expr_for_symbol`; both the symbol and exact-key branches inspect `message_definitions` keys (`builtins_messages.rs:50-67`, `message_catalog.rs:239-251`). | **No.** It lists defined templates, not emitted runtime instances. |
| bare `f::tag` (`MessageName` read) | Syntax lowering constructs `MessageName[f, "tag"]` (`lowering.rs:296-311`); `eval_message_name` validates/normalizes and reconstructs that expression (`builtins_messages.rs:31-47`). A bare read consults no template or emitted-message store. Assignment to that expression is the separate path that calls `define_message`; `Message[f::tag, ...]` later reads `message_definitions` in `render_message_text` (`message_catalog.rs:94-127`). | **No.** A bare `MessageName` is structural; assignment can create a definition, while a push cannot. |
| `On` / `Off` message selectors | Selector expressions/strings are normalized into `MessageSelector` values and mutate only `disabled_message_selectors` (`debug_tools/message_switch.rs:18-60`; parsing is in `messages/message_selector_parsing.rs`). `push_message` consults this vector through `message_enabled`. | **No.** Selectors can suppress/reenable a push but do not catalog its symbol/tag/text. |
| `Information`'s `"Messages"` field | `information_data_expr_for_symbol` calls `message_tags_expr_for_symbol`, which derives tags from `message_definitions` keys (`builtins_messages_inspect.rs:245-251`, `message_catalog.rs:326-344`). | **No.** Only assigned templates contribute tags. |
| `Session::inspect_message_catalog` and host `HostMessageTemplate` | Query normalization selects `message_catalog_entries_for_symbol` or `_for_key`, both reading `message_definitions` (`session_eval.rs:122-137`, `message_catalog.rs:302-324`). `rum-host` maps each returned entry to `HostMessageTemplate { message_name, qualified_symbol_name, tag, template, delayed }` (`rum-host/src/session.rs:462-475`; fields at `rum-host/src/model/inspection.rs:79-91`). | **No.** The host seam exposes defined templates only. |

Headline gap: builtin diagnostics are constructed at the point of failure as
rendered `RuntimeMessage` values, while every catalog/listing surface that has
template or tag data reads the independent user-definition map. No current
path copies builtin push metadata into that map.

### Release-CLI probe transcripts

The release CLI was built once. The command and complete build output were:

```text
$ scripts/run-bounded.py --timeout 600 -- cargo build --release -p rum-cli
warning: field `source_template` is never read
  --> crates/rum-runtime/src/runtime/vm_rhs_template/model.rs:12:16
   |
9  | pub(super) struct CodeObject {
   |                   ---------- field in this struct
...
12 |     pub(super) source_template: Option<TemplateNode>,
   |                ^^^^^^^^^^^^^^^
   |
   = note: `CodeObject` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead code analysis
   = note: `#[warn(dead_code)]` on by default

warning: `rum-runtime` (lib) generated 1 warning
   Compiling rum-host v0.1.0 (/Users/fft/Developments/rum/crates/rum-host)
   Compiling rum-terminal v0.1.0 (/Users/fft/Developments/rum/crates/rum-terminal)
   Compiling rum-repl v0.1.0 (/Users/fft/Developments/rum/crates/rum-repl)
   Compiling rum-cli v0.1.0 (/Users/fft/Developments/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 2.41s
```

The requested failing `Set` call:

```text
$ target/release/rum-cli --eval 'Set[]' </dev/null
Set[]
! Set::argx: expected exactly 2 argument(s), got 0
```

The requested catalog query:

```text
$ target/release/rum-cli --eval 'Messages[Set]' </dev/null
List[]
```

A same-session compound probe confirms that the just-emitted message still
does not enter the template catalog:

```text
$ target/release/rum-cli --eval 'Set[]; Messages[Set]' </dev/null
List[]
! Set::argx: expected exactly 2 argument(s), got 0
```

The generic catalog query is also empty:

```text
$ target/release/rum-cli --eval 'Messages[General]' </dev/null
List[]
```

The user-defined template round-trip does populate the catalog:

```text
$ target/release/rum-cli --eval 'f::tag = "user template"; Messages[f]' </dev/null
List[f::tag]
```

The observable surprise is formatting, not semantics: this CLI prints lists
as `List[...]` rather than brace syntax, and sends the emitted diagnostic after
the evaluated result in the same-session compound probe. The output above is
left verbatim.

## Section F — registry attachment facts (no design)

### `BuiltinSpec` and declaration idiom

`BuiltinSpec` is declared at `crates/rum-runtime/src/runtime/metadata.rs:287`.
Its current fields are:

| field | type |
|---|---|
| `name` | `&'static str` |
| `summary` | `&'static str` |
| `attributes` | `AttributeSet` |
| `dispatch` | `BuiltinDispatch` |
| `handler` | `Option<BuiltinFn>` |
| `capability` | `Capability` |

The complete const-builder/constructor set in that impl is `new`, `with_hold`,
`with_attributes`, `special`, `special_with_hold`,
`special_with_attributes`, `ordinary`, `ordinary_with_hold`,
`ordinary_with_attributes`, `protected`, and `with_capability`
(`metadata.rs:297-430`). `with_fallback_dispatch` at `metadata.rs:432` is an
additional non-const, crate-internal merge helper, not a declaration builder.
Every constructor initializes all six fields; modifier builders preserve the
rest of the copied spec.

Catalog declarations are static `&[BuiltinSpec]` arrays built with those
constructors and modifiers. One stdlib example is `FromCsv` at
`crates/rum-stdlib/src/catalog/csv.rs:5-11`, declared with
`BuiltinSpec::ordinary_with_attributes(...)`. One kernel example is `FindFile`
at `crates/rum-runtime/src/runtime/builtin_catalog/namespace.rs:94-100`,
declared with `BuiltinSpec::ordinary(...)`, then `.protected()` and
`.with_capability(Capability::FileRead)`.

### Installation, docgen, and drift

`rum_stdlib::install_core` registers the kernel `core_builtin_specs()` followed
by each stdlib catalog slice (`crates/rum-stdlib/src/lib.rs:38-121`).
`Session::register_builtin` stores the complete spec keyed by its System
symbol; if a spec overlays an existing entry it uses
`with_fallback_dispatch`, then installs the spec's attributes
(`session_api.rs:149-176`). `registered_builtin_specs` exposes the stored
values at `session_api.rs:178-181`.

Docgen's `installed_builtin_specs` creates a session, calls `install_core`,
collects `registered_builtin_specs`, and sorts by name
(`crates/rum-stdlib/src/docgen.rs:301-311`). `push_entry` currently consumes
`name`, `summary` (subject to `SUMMARY_OVERRIDES`), `attributes`, and
`capability`, plus the separate name-keyed `LIMITS` table
(`docgen.rs:340-365`). It reads no message-definition, message-tag, handler, or
dispatch data. `generate_builtin_reference` renders those entries, and
`crates/rum-stdlib/src/bin/generate-builtin-reference.rs:3-17` writes
`docs/reference/builtins.md`.

The drift test is `builtin_reference_is_in_sync` at
`crates/rum-stdlib/src/tests/builtin_reference_docgen.rs:6`; it compares a
fresh `generate_builtin_reference()` result to the committed reference. The
module is wired at `crates/rum-stdlib/src/tests.rs:11-12`, and the test is
compiled only with the `hash` feature because both its module import and test
carry `#[cfg(feature = "hash")]`.

### Brief-210 mirror-lockstep facts

Brief 210's Protected sweep had three mechanically coupled representations:

1. stdlib `BuiltinSpec` catalog entries in
   `crates/rum-stdlib/src/catalog/*.rs`;
2. the kernel-resident name/attribute mirror in
   `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`, whose
   public row shape is only `BuiltinAttributeSpec { name, attributes }`
   (`builtin_catalog.rs:19-28`); and
3. the generated builtin reference produced from the installed full specs.

`catalog_attributes_match_kernel_vocabulary` at
`crates/rum-stdlib/src/tests.rs:178` constructs name-to-attribute maps from the
stdlib catalog and `stdlib_builtin_attribute_specs()`, rejects duplicate names,
and asserts matching attributes for every stdlib catalog row. The generated
reference has the separate drift test above. The installation/docgen path
carries the full `BuiltinSpec`, but the explicit catalog-versus-kernel mirror
lockstep covers **name + attributes only**; there is no existing mirrored
message field or message-tag assertion. These are the current attachment and
verification points only, not a proposed registry shape.

## Stop-loss / open questions

No execution blocker or out-of-scope change was needed. One semantic boundary
is intentionally left explicit rather than forced into a false finite count:
`Message[arbitrary::tag]` and the held-call inspection builtins make the total
runtime-wide `(symbol, tag)` vocabulary open. Section D therefore reports both
that fact and the finite 966-pair current-source builtin diagnostic baseline.
