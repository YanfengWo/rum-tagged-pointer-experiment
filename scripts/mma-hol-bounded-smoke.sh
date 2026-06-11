#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/mma-hol-bounded-smoke.sh [--mma-hol PATH] [options]

Run bounded, progress-reporting rum probes against a local mma-hol checkout.
This is an integration triage driver, not a performance benchmark.

Options:
  --mma-hol PATH       mma-hol checkout root. Defaults to RUM_MMA_HOL_ROOT.
  --mode MODE          loads, tests, or all. Defaults to loads.
  --timeout SECONDS    Per-stage wall timeout. Defaults to 60.
  --output-dir PATH    Directory for summary.tsv and per-stage logs.
  --only LABEL         Run one selected stage label, e.g. test/proptaut_tests.
  --limit N            Stop after N selected stages.
  --keep-going         Continue after a failed stage.
  --list               Print selected stages and exit.
  --dry-run            Print selected stages without building or running rum.
  -h, --help           Show this help.

Environment overrides:
  RUM_MMA_HOL_ROOT         Default mma-hol checkout root.
  RUM_MMA_HOL_MODE         Default mode.
  RUM_MMA_HOL_TIMEOUT      Default per-stage timeout.
  RUM_MMA_HOL_OUTPUT_DIR   Default output directory.
  RUM_CLI_BIN              Existing rum-cli binary. If unset, this script
                           builds target/debug/rum-cli once and uses it.

Exit codes:
  0   all selected stages passed
  1   at least one selected stage failed or timed out
  2   usage or environment error
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)

mma_hol_root=${RUM_MMA_HOL_ROOT:-}
mode=${RUM_MMA_HOL_MODE:-loads}
timeout_seconds=${RUM_MMA_HOL_TIMEOUT:-60}
output_dir=${RUM_MMA_HOL_OUTPUT_DIR:-}
only_label=${RUM_MMA_HOL_ONLY:-}
limit=0
keep_going=0
list_only=0
dry_run=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mma-hol)
      [[ $# -ge 2 ]] || { echo "error: --mma-hol expects a path" >&2; exit 2; }
      mma_hol_root=$2
      shift 2
      ;;
    --mode)
      [[ $# -ge 2 ]] || { echo "error: --mode expects loads, tests, or all" >&2; exit 2; }
      mode=$2
      shift 2
      ;;
    --timeout)
      [[ $# -ge 2 ]] || { echo "error: --timeout expects seconds" >&2; exit 2; }
      timeout_seconds=$2
      shift 2
      ;;
    --output-dir)
      [[ $# -ge 2 ]] || { echo "error: --output-dir expects a path" >&2; exit 2; }
      output_dir=$2
      shift 2
      ;;
    --only)
      [[ $# -ge 2 ]] || { echo "error: --only expects a stage label" >&2; exit 2; }
      only_label=$2
      shift 2
      ;;
    --limit)
      [[ $# -ge 2 ]] || { echo "error: --limit expects a positive integer" >&2; exit 2; }
      limit=$2
      shift 2
      ;;
    --keep-going)
      keep_going=1
      shift
      ;;
    --list)
      list_only=1
      shift
      ;;
    --dry-run)
      dry_run=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "error: unknown option $1" >&2
      exit 2
      ;;
    *)
      if [[ -n "${mma_hol_root}" ]]; then
        echo "error: unexpected positional argument $1" >&2
        exit 2
      fi
      mma_hol_root=$1
      shift
      ;;
  esac
done

case "${mode}" in
  loads|tests|all) ;;
  *) echo "error: --mode expects loads, tests, or all" >&2; exit 2 ;;
esac

if ! [[ "${timeout_seconds}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --timeout expects a positive integer" >&2
  exit 2
fi

if ! [[ "${limit}" =~ ^[0-9]+$ ]]; then
  echo "error: --limit expects a non-negative integer" >&2
  exit 2
fi

if [[ -z "${mma_hol_root}" ]]; then
  echo "error: provide --mma-hol PATH or set RUM_MMA_HOL_ROOT" >&2
  exit 2
fi

if [[ ! -f "${mma_hol_root}/tests/run_all.wls" ]]; then
  echo "error: ${mma_hol_root} does not look like an mma-hol checkout" >&2
  exit 2
fi

load_labels=(
  load/ErrorUtil
  load/Types
  load/Terms
  load/Kernel
  load/Bootstrap
  load/Equal
  load/Bool
  load/Drule
  load/Tactics
  load/Printer
  load/Parser
  load/auto/PropTaut
  load/auto/Meson
  load/auto/Simp
  load/stdlib/Pair
  load/stdlib/Sum
  load/stdlib/Option
  load/stdlib/Set
  load/auto/Set
  load/stdlib/Num
  load/auto/Arith
  load/stdlib/List
  load/stdlib/Finite
  load/stdlib/FTA
  load/stdlib/Int
  load/stdlib/Rat
  load/stdlib/Real/Cut
  load/stdlib/Real/RatAux
  load/stdlib/Real/Field
  load/stdlib/Real/Mul
  load/stdlib/Real/Inv
  load/stdlib/Real/Complete
)

load_files=(
  ErrorUtil.wl
  Types.wl
  Terms.wl
  Kernel.wl
  Bootstrap.wl
  Equal.wl
  Bool.wl
  Drule.wl
  Tactics.wl
  Printer.wl
  Parser.wl
  auto/PropTaut.wl
  auto/Meson.wl
  auto/Simp.wl
  stdlib/Pair.wl
  stdlib/Sum.wl
  stdlib/Option.wl
  stdlib/Set.wl
  auto/Set.wl
  stdlib/Num.wl
  auto/Arith.wl
  stdlib/List.wl
  stdlib/Finite.wl
  stdlib/FTA.wl
  stdlib/Int.wl
  stdlib/Rat.wl
  stdlib/Real/Cut.wl
  stdlib/Real/RatAux.wl
  stdlib/Real/Field.wl
  stdlib/Real/Mul.wl
  stdlib/Real/Inv.wl
  stdlib/Real/Complete.wl
)

test_labels=(
  test/kernel_tests
  test/errorutil_tests
  test/types_tests
  test/terms_tests
  test/bootstrap_tests
  test/equal_tests
  test/bool_tests
  test/drule_tests
  test/tactics_tests
  test/printer_tests
  test/parser_tests
  test/proptaut_tests
  test/meson_tests
  test/meson_proveprop_tests
  test/simp_tests
  test/pair_tests
  test/sum_tests
  test/option_tests
  test/set_tests
  test/auto_set_tests
  test/num_tests
  test/arith_tests
  test/list_tests
  test/finite_tests
  test/fta_tests
  test/int_tests
  test/rat_tests
  test/real_tests
  test/real_complete_tests
)

test_files=(
  tests/kernel_tests.wl
  tests/errorutil_tests.wl
  tests/types_tests.wl
  tests/terms_tests.wl
  tests/bootstrap_tests.wl
  tests/equal_tests.wl
  tests/bool_tests.wl
  tests/drule_tests.wl
  tests/tactics_tests.wl
  tests/printer_tests.wl
  tests/parser_tests.wl
  tests/proptaut_tests.wl
  tests/meson_tests.wl
  tests/meson_proveprop_tests.wl
  tests/simp_tests.wl
  tests/pair_tests.wl
  tests/sum_tests.wl
  tests/option_tests.wl
  tests/set_tests.wl
  tests/auto_set_tests.wl
  tests/num_tests.wl
  tests/arith_tests.wl
  tests/list_tests.wl
  tests/finite_tests.wl
  tests/fta_tests.wl
  tests/int_tests.wl
  tests/rat_tests.wl
  tests/real_tests.wl
  tests/real_complete_tests.wl
)

test_preload_load_indices=(
  3
  0
  1
  2
  4
  5
  6
  7
  8
  9
  10
  11
  12
  12
  13
  14
  15
  16
  17
  18
  19
  20
  21
  22
  23
  24
  25
  31
  31
)

stage_labels=()
stage_kinds=()
stage_indices=()

append_load_stages() {
  local i
  for i in "${!load_labels[@]}"; do
    stage_labels+=("${load_labels[$i]}")
    stage_kinds+=("load")
    stage_indices+=("$i")
  done
}

append_test_stages() {
  local i
  for i in "${!test_labels[@]}"; do
    stage_labels+=("${test_labels[$i]}")
    stage_kinds+=("test")
    stage_indices+=("$i")
  done
}

case "${mode}" in
  loads) append_load_stages ;;
  tests) append_test_stages ;;
  all) append_load_stages; append_test_stages ;;
esac

if [[ -n "${only_label}" ]]; then
  selected_label=()
  selected_kind=()
  selected_index=()
  for i in "${!stage_labels[@]}"; do
    if [[ "${stage_labels[$i]}" == "${only_label}" ]]; then
      selected_label+=("${stage_labels[$i]}")
      selected_kind+=("${stage_kinds[$i]}")
      selected_index+=("${stage_indices[$i]}")
      break
    fi
  done
  if [[ "${#selected_label[@]}" -eq 0 ]]; then
    echo "error: unknown stage label for --only: ${only_label}" >&2
    exit 2
  fi
  stage_labels=("${selected_label[@]}")
  stage_kinds=("${selected_kind[@]}")
  stage_indices=("${selected_index[@]}")
fi

if [[ "${limit}" -gt 0 && "${limit}" -lt "${#stage_labels[@]}" ]]; then
  stage_labels=("${stage_labels[@]:0:${limit}}")
  stage_kinds=("${stage_kinds[@]:0:${limit}}")
  stage_indices=("${stage_indices[@]:0:${limit}}")
fi

if [[ "${#stage_labels[@]}" -eq 0 ]]; then
  echo "error: no stages selected" >&2
  exit 2
fi

if [[ "${list_only}" == "1" || "${dry_run}" == "1" ]]; then
  printf 'mode=%s timeout=%ss stages=%s\n' "${mode}" "${timeout_seconds}" "${#stage_labels[@]}"
  for i in "${!stage_labels[@]}"; do
    printf '%02d\t%s\n' "$((i + 1))" "${stage_labels[$i]}"
  done
  exit 0
fi

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-mma-hol-smoke.XXXXXX)
else
  mkdir -p "${output_dir}"
fi

summary_file="${output_dir}/summary.tsv"
printf 'stage\tstatus\texit_code\telapsed_seconds\tstdout\tstderr\n' > "${summary_file}"

wl_string() {
  local value=${1//\\/\\\\}
  value=${value//\"/\\\"}
  printf '"%s"' "${value}"
}

append_path_setup() {
  local root_string tests_string
  root_string=$(wl_string "${mma_hol_root}")
  tests_string=$(wl_string "${mma_hol_root}/tests")
  printf '$Path = Prepend[$Path, %s]; ' "${root_string}"
  printf '$Path = Prepend[$Path, %s]; ' "${tests_string}"
}

append_get() {
  local rel=$1
  local abs_string
  abs_string=$(wl_string "${mma_hol_root}/${rel}")
  printf 'Print["[rum-mma-hol] GET %s"]; Get[%s]; ' "${rel}" "${abs_string}"
}

load_expr_through() {
  local index=$1
  local i
  append_path_setup
  for ((i = 0; i <= index; i++)); do
    append_get "${load_files[$i]}"
  done
  printf '"ok"'
}

test_expr() {
  local index=$1
  local rel=${test_files[$index]}
  local preload_index=${test_preload_load_indices[$index]}
  local i
  append_path_setup
  for ((i = 0; i <= preload_index; i++)); do
    append_get "${load_files[$i]}"
  done
  append_get "tests/harness.wl"
  printf 'HOLTest`resetCounters[]; '
  append_get "${rel}"
  printf 'HOLTest`testSummary[]'
}

classify_stage() {
  local kind=$1
  local exit_code=$2
  local stdout_file=$3
  local stderr_file=$4
  local last_line

  if [[ "${exit_code}" == "124" ]]; then
    printf 'timeout'
    return
  fi
  if [[ "${exit_code}" != "0" ]]; then
    printf 'error'
    return
  fi
  if grep -q 'TerminatedEvaluation' "${stdout_file}" "${stderr_file}"; then
    printf 'terminated'
    return
  fi
  if grep -q '\$Failed' "${stdout_file}" "${stderr_file}"; then
    printf 'failed'
    return
  fi

  last_line=$(awk 'NF { line = $0 } END { print line }' "${stdout_file}")
  if [[ "${kind}" == "load" ]]; then
    if [[ "${last_line}" == '"ok"' ]]; then
      printf 'ok'
    else
      printf 'unexpected-output'
    fi
  else
    if [[ "${last_line}" =~ ^List\[[0-9]+,\ 0\]$ ]]; then
      printf 'ok'
    else
      printf 'failed-tests'
    fi
  fi
}

rum_cli_bin=${RUM_CLI_BIN:-}
if [[ -z "${rum_cli_bin}" ]]; then
  (
    cd "${repo_root}"
    cargo build -p rum-cli >/dev/null
  )
  rum_cli_bin="${repo_root}/target/debug/rum-cli"
fi

if [[ ! -x "${rum_cli_bin}" ]]; then
  echo "error: rum-cli binary is not executable: ${rum_cli_bin}" >&2
  exit 2
fi

bounded_runner="${repo_root}/scripts/run-bounded.py"
if [[ ! -x "${bounded_runner}" ]]; then
  echo "error: bounded runner is not executable: ${bounded_runner}" >&2
  exit 2
fi

stage_slug() {
  local slug=${1//\//_}
  slug=${slug//[^A-Za-z0-9_.-]/_}
  printf '%s' "${slug}"
}

echo "[rum-mma-hol] root: ${mma_hol_root}"
echo "[rum-mma-hol] mode: ${mode}; timeout: ${timeout_seconds}s; stages: ${#stage_labels[@]}"
echo "[rum-mma-hol] logs: ${output_dir}"

overall_status=0

for i in "${!stage_labels[@]}"; do
  label=${stage_labels[$i]}
  kind=${stage_kinds[$i]}
  index=${stage_indices[$i]}
  slug=$(stage_slug "${label}")
  stdout_file="${output_dir}/${slug}.stdout"
  stderr_file="${output_dir}/${slug}.stderr"
  expr_file="${output_dir}/${slug}.expr"

  if [[ "${kind}" == "load" ]]; then
    load_expr_through "${index}" > "${expr_file}"
  else
    test_expr "${index}" > "${expr_file}"
  fi

  printf '[%02d/%02d] %s ... ' "$((i + 1))" "${#stage_labels[@]}" "${label}"
  start=$(date +%s)
  set +e
  "${bounded_runner}" --timeout "${timeout_seconds}" -- \
    "${rum_cli_bin}" --eval "$(cat "${expr_file}")" \
    > "${stdout_file}" 2> "${stderr_file}"
  exit_code=$?
  set -e
  end=$(date +%s)
  elapsed=$((end - start))
  status=$(classify_stage "${kind}" "${exit_code}" "${stdout_file}" "${stderr_file}")

  printf '%s (%ss)\n' "${status}" "${elapsed}"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${label}" "${status}" "${exit_code}" "${elapsed}" "${stdout_file}" "${stderr_file}" \
    >> "${summary_file}"

  if [[ "${status}" != "ok" ]]; then
    overall_status=1
    echo "[rum-mma-hol] first failing stage: ${label}" >&2
    echo "[rum-mma-hol] stdout: ${stdout_file}" >&2
    echo "[rum-mma-hol] stderr: ${stderr_file}" >&2
    if [[ "${keep_going}" != "1" ]]; then
      break
    fi
  fi
done

echo "[rum-mma-hol] summary: ${summary_file}"
exit "${overall_status}"
