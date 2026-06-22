#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/snapshot-reliability.sh --mma-hol PATH [options]

Stress-test rum DumpSave/Get snapshot round-trips on fast mma-hol layers.
The source mma-hol checkout is copied to a writable scratch tree before any
snapshot command runs; the source checkout is never written.

Options:
  --mma-hol PATH       Source mma-hol checkout root. Defaults to RUM_MMA_HOL_ROOT.
  --layers "L ..."     Space-separated layers. Defaults to:
                       Bool Drule Tactics Printer Parser
  --cycles N           Restore/re-dump cycles per layer. Defaults to 3.
  --timeout SECONDS    Per-command wall timeout. Defaults to 60.
  --output-dir PATH    Directory for summary.tsv, scratch copy, expressions, logs.
  --rum-cli PATH       Existing rum-cli binary. Defaults to RUM_CLI_BIN, then
                       target/release/rum-cli, building it if needed.
  --list               Print supported layers and exit.
  -h, --help           Show this help.

Exit codes:
  0   all selected layers passed
  1   at least one reliability assertion failed
  2   usage or environment error
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)

default_layers="Bool Drule Tactics Printer Parser"
mma_hol_source=${RUM_MMA_HOL_ROOT:-}
layers_arg=${RUM_SNAPSHOT_LAYERS:-${default_layers}}
cycles=${RUM_SNAPSHOT_CYCLES:-3}
timeout_seconds=${RUM_SNAPSHOT_TIMEOUT:-60}
output_dir=${RUM_SNAPSHOT_OUT:-}
rum_cli_bin=${RUM_CLI_BIN:-}
list_only=0

supported_layers=(ErrorUtil Types Terms Kernel Bootstrap Equal Bool Drule Tactics Printer Parser PropTaut Meson Simp Pair Sum Option Set Num Arith List Finite FTA Int Rat)

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mma-hol)
      [[ $# -ge 2 ]] || { echo "error: --mma-hol expects a path" >&2; exit 2; }
      mma_hol_source=$2
      shift 2
      ;;
    --layers)
      [[ $# -ge 2 ]] || { echo "error: --layers expects a quoted layer list" >&2; exit 2; }
      layers_arg=$2
      shift 2
      ;;
    --cycles)
      [[ $# -ge 2 ]] || { echo "error: --cycles expects a positive integer" >&2; exit 2; }
      cycles=$2
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
    --rum-cli)
      [[ $# -ge 2 ]] || { echo "error: --rum-cli expects a path" >&2; exit 2; }
      rum_cli_bin=$2
      shift 2
      ;;
    --list)
      list_only=1
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
      if [[ -n "${mma_hol_source}" ]]; then
        echo "error: unexpected positional argument $1" >&2
        exit 2
      fi
      mma_hol_source=$1
      shift
      ;;
  esac
done

print_layers() {
  local layer
  printf 'default layers: %s\n' "${default_layers}"
  printf 'supported layers:\n'
  for layer in "${supported_layers[@]}"; do
    printf '  %s\t%s\n' "${layer}" "$(layer_file "${layer}")"
  done
}

require_positive_int() {
  local name=$1
  local value=$2
  if ! [[ "${value}" =~ ^[1-9][0-9]*$ ]]; then
    echo "error: ${name} expects a positive integer" >&2
    exit 2
  fi
}

abs_dir() {
  local path=$1
  (cd "${path}" && pwd -P)
}

abs_file() {
  local path=$1
  local dir
  dir=$(dirname "${path}")
  printf '%s/%s\n' "$(abs_dir "${dir}")" "$(basename "${path}")"
}

wl_string() {
  local value=${1//\\/\\\\}
  value=${value//\"/\\\"}
  printf '"%s"' "${value}"
}

layer_file() {
  case "$1" in
    ErrorUtil) printf 'ErrorUtil.wl' ;;
    Types) printf 'Types.wl' ;;
    Terms) printf 'Terms.wl' ;;
    Kernel) printf 'Kernel.wl' ;;
    Bootstrap) printf 'Bootstrap.wl' ;;
    Equal) printf 'Equal.wl' ;;
    Bool) printf 'Bool.wl' ;;
    Drule) printf 'Drule.wl' ;;
    Tactics) printf 'Tactics.wl' ;;
    Printer) printf 'Printer.wl' ;;
    Parser) printf 'Parser.wl' ;;
    PropTaut) printf 'auto/PropTaut.wl' ;;
    Meson) printf 'auto/Meson.wl' ;;
    Simp) printf 'auto/Simp.wl' ;;
    Pair) printf 'stdlib/Pair.wl' ;;
    Sum) printf 'stdlib/Sum.wl' ;;
    Option) printf 'stdlib/Option.wl' ;;
    Set) printf 'stdlib/Set.wl' ;;
    Num) printf 'stdlib/Num.wl' ;;
    Arith) printf 'auto/Arith.wl' ;;
    List) printf 'stdlib/List.wl' ;;
    Finite) printf 'stdlib/Finite.wl' ;;
    FTA) printf 'stdlib/FTA.wl' ;;
    Int) printf 'stdlib/Int.wl' ;;
    Rat) printf 'stdlib/Rat.wl' ;;
    *) return 1 ;;
  esac
}

prior_layer() {
  case "$1" in
    ErrorUtil) printf '' ;;
    Types) printf 'ErrorUtil' ;;
    Terms) printf 'Types' ;;
    Kernel) printf 'Terms' ;;
    Bootstrap) printf 'Kernel' ;;
    Equal) printf 'Bootstrap' ;;
    Bool) printf 'Equal' ;;
    Drule) printf 'Bool' ;;
    Tactics) printf 'Drule' ;;
    Printer) printf 'Tactics' ;;
    Parser) printf 'Printer' ;;
    PropTaut) printf 'Parser' ;;
    Meson) printf 'PropTaut' ;;
    Simp) printf 'Meson' ;;
    Pair) printf 'Simp' ;;
    Sum) printf 'Pair' ;;
    Option) printf 'Sum' ;;
    Set) printf 'Option' ;;
    Num) printf 'Set' ;;
    Arith) printf 'Num' ;;
    List) printf 'Arith' ;;
    Finite) printf 'List' ;;
    FTA) printf 'Finite' ;;
    Int) printf 'FTA' ;;
    Rat) printf 'Int' ;;
    *) return 1 ;;
  esac
}

layer_pattern() {
  local file stem
  file=$(layer_file "$1") || return 1
  stem=$(basename "${file}" .wl)
  printf '%s' "${stem}"
}

layer_test_pattern() {
  case "$1" in
    PropTaut) printf 'proptaut' ;;
    *) printf '%s' "$1" | tr '[:upper:]' '[:lower:]' ;;
  esac
}

append_script_command_line() {
  local script_rel=$1
  shift
  printf '$ScriptCommandLine = {%s' "$(wl_string "${script_rel}")"
  local arg
  for arg in "$@"; do
    printf ', %s' "$(wl_string "${arg}")"
  done
  printf '}; '
}

append_get_absolute() {
  local path=$1
  printf 'Get[%s]; ' "$(wl_string "${path}")"
}

build_snapshot_expr() {
  local root=$1
  local pattern=$2
  append_script_command_line "tests/build_snapshot.wls" "${pattern}"
  append_get_absolute "${root}/tests/build_snapshot.wls"
  printf '"ok"'
}

extend_snapshot_expr() {
  local root=$1
  local file=$2
  append_script_command_line "tests/extend_snapshot.wls" "${file}"
  append_get_absolute "${root}/tests/extend_snapshot.wls"
  printf '"ok"'
}

run_fast_expr() {
  local root=$1
  local pattern=$2
  append_script_command_line "tests/run_fast.wls" "${pattern}"
  append_get_absolute "${root}/tests/run_fast.wls"
}

count_names_expr() {
  local mx=$1
  printf 'Get[%s]; ' "$(wl_string "${mx}")"
  printf 'Module[{names = Names["HOL`*"], gensyms}, '
  printf 'gensyms = Select[names, StringMatchQ[#, {"*$0*", "*$1*", "*$2*", "*$3*", "*$4*", "*$5*", "*$6*", "*$7*", "*$8*", "*$9*"}] &]; '
  printf 'Print["[snaprel] name_count=", Length[names]]; '
  printf 'Print["[snaprel] gensym_count=", Length[gensyms]]]; '
  printf '"ok"'
}

redump_expr() {
  local mx=$1
  printf 'Module[{mx = %s, holContexts}, ' "$(wl_string "${mx}")"
  printf 'Get[mx]; '
  printf 'holContexts = Select[Contexts[], StringMatchQ[#, "HOL`*"] &]; '
  printf 'DumpSave[mx, Evaluate[holContexts]]; '
  printf 'Print["[snaprel] redump_bytes=", FileByteCount[mx]]; '
  printf '"ok"]'
}

stage_slug() {
  local slug=${1//\//_}
  slug=${slug//[^A-Za-z0-9_.-]/_}
  printf '%s' "${slug}"
}

run_expr_file() {
  local label=$1
  local expr_file=$2
  local stdout_file=$3
  local stderr_file=$4
  set +e
  "${bounded_runner}" --timeout "${timeout_seconds}" -- \
    "${rum_cli_bin}" --eval "$(cat "${expr_file}")" \
    > "${stdout_file}" 2> "${stderr_file}"
  local exit_code=$?
  set -e
  printf '%s' "${exit_code}"
  if [[ "${exit_code}" != "0" ]]; then
    echo "[snaprel] ${label} failed with exit ${exit_code}" >&2
    echo "[snaprel] stdout: ${stdout_file}" >&2
    echo "[snaprel] stderr: ${stderr_file}" >&2
  fi
}

run_generated_expr() {
  local label=$1
  local expr_file=$2
  local stdout_file=$3
  local stderr_file=$4
  shift 4
  "$@" > "${expr_file}"
  run_expr_file "${label}" "${expr_file}" "${stdout_file}" "${stderr_file}"
}

file_bytes() {
  wc -c < "$1" | tr -d '[:space:]'
}

file_sha() {
  shasum -a 256 "$1" | awk '{print $1}'
}

extract_name_count() {
  local file=$1
  awk -F= '/^\[snaprel\] name_count=/ { value = $2 } END { print value }' "${file}"
}

extract_gensym_count() {
  local file=$1
  awk -F= '/^\[snaprel\] gensym_count=/ { value = $2 } END { print value }' "${file}"
}

extract_test_summary() {
  local file=$1
  awk '
    /^[[:space:]]*passed:[[:space:]]*[0-9]+[[:space:]]*$/ { p = $2 }
    /^[[:space:]]*failed:[[:space:]]*[0-9]+[[:space:]]*$/ { f = $2 }
    /^List\[[0-9]+,[[:space:]]*[0-9]+\]$/ {
      gsub(/^List\[/, "", $0);
      gsub(/\]$/, "", $0);
      split($0, parts, ",");
      p = parts[1];
      f = parts[2];
      gsub(/[[:space:]]/, "", p);
      gsub(/[[:space:]]/, "", f);
    }
    END {
      if (p != "" && f != "") {
        printf "%s/%s", p, f
      }
    }' "${file}"
}

record_failure() {
  local layer=$1
  local check=$2
  echo "[snaprel] FAIL ${layer}: ${check}" >&2
  failures="${failures}${layer}:${check}; "
}

if [[ "${list_only}" == "1" ]]; then
  print_layers
  exit 0
fi

require_positive_int "--cycles" "${cycles}"
require_positive_int "--timeout" "${timeout_seconds}"

if [[ -z "${mma_hol_source}" ]]; then
  echo "error: provide --mma-hol PATH or set RUM_MMA_HOL_ROOT" >&2
  exit 2
fi

if [[ ! -f "${mma_hol_source}/tests/build_snapshot.wls" ||
      ! -f "${mma_hol_source}/tests/extend_snapshot.wls" ||
      ! -f "${mma_hol_source}/tests/run_fast.wls" ]]; then
  echo "error: ${mma_hol_source} does not look like an mma-hol checkout" >&2
  exit 2
fi

mma_hol_source=$(abs_dir "${mma_hol_source}")

selected_layers=()
read -r -a selected_layers <<< "${layers_arg}"
if [[ "${#selected_layers[@]}" -eq 0 ]]; then
  echo "error: --layers selected no layers" >&2
  exit 2
fi

for layer in "${selected_layers[@]}"; do
  if ! layer_file "${layer}" >/dev/null; then
    echo "error: unsupported layer: ${layer}" >&2
    echo "hint: use --list to see supported layers" >&2
    exit 2
  fi
done

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-snapshot-reliability.XXXXXX)
else
  mkdir -p "${output_dir}"
  output_dir=$(abs_dir "${output_dir}")
fi

logs_dir="${output_dir}/logs"
expr_dir="${output_dir}/expr"
mkdir -p "${logs_dir}" "${expr_dir}"

scratch_parent=$(mktemp -d "${output_dir}/scratch.XXXXXX")
scratch_root="${scratch_parent}/mma-hol"
mkdir -p "${scratch_root}"
rsync -a --delete --exclude '.git/' --exclude '*.mx' \
  "${mma_hol_source}/" "${scratch_root}/"

if [[ -z "${rum_cli_bin}" ]]; then
  rum_cli_bin="${repo_root}/target/release/rum-cli"
  (
    cd "${repo_root}"
    cargo build --release -p rum-cli >/dev/null
  )
fi
rum_cli_bin=$(abs_file "${rum_cli_bin}")

if [[ ! -x "${rum_cli_bin}" ]]; then
  echo "error: rum-cli binary is not executable: ${rum_cli_bin}" >&2
  exit 2
fi

bounded_runner="${repo_root}/scripts/run-bounded.py"
if [[ ! -x "${bounded_runner}" ]]; then
  echo "error: bounded runner is not executable: ${bounded_runner}" >&2
  exit 2
fi

summary_file="${output_dir}/summary.tsv"
printf 'layer\tcold_bytes\tgensym_count\tcycle_byte_stable(yes/no)\ttests(P/F)\ttest_stable(yes/no)\tfidelity(ok/na/FAIL)\n' > "${summary_file}"

echo "[snaprel] source: ${mma_hol_source}"
echo "[snaprel] scratch: ${scratch_root}"
echo "[snaprel] output: ${output_dir}"
echo "[snaprel] rum-cli: ${rum_cli_bin}"
echo "[snaprel] cycles: ${cycles}; timeout: ${timeout_seconds}s; layers: ${selected_layers[*]}"

failures=""

for index in "${!selected_layers[@]}"; do
  layer=${selected_layers[$index]}
  pattern=$(layer_pattern "${layer}")
  test_pattern=$(layer_test_pattern "${layer}")
  file=$(layer_file "${layer}")
  prior=$(prior_layer "${layer}")
  mx="${scratch_root}/bootstrap.mx"
  slug=$(stage_slug "${layer}")

  echo "[$((index + 1))/${#selected_layers[@]}] ${layer}"

  build_expr_file="${expr_dir}/${slug}.cold-build.expr"
  build_stdout="${logs_dir}/${slug}.cold-build.stdout"
  build_stderr="${logs_dir}/${slug}.cold-build.stderr"
  printf '  cold build through %s ... ' "${pattern}"
  exit_code=$(run_generated_expr "${layer} cold build" "${build_expr_file}" "${build_stdout}" "${build_stderr}" build_snapshot_expr "${scratch_root}" "${pattern}")
  if [[ "${exit_code}" != "0" || ! -f "${mx}" ]]; then
    printf 'FAIL\n'
    record_failure "${layer}" "cold-build"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "${layer}" "FAIL" "FAIL" "no" "FAIL" "no" "FAIL" >> "${summary_file}"
    continue
  fi

  cold_bytes=$(file_bytes "${mx}")
  cold_sha=$(file_sha "${mx}")
  count_expr_file="${expr_dir}/${slug}.cold-count.expr"
  count_stdout="${logs_dir}/${slug}.cold-count.stdout"
  count_stderr="${logs_dir}/${slug}.cold-count.stderr"
  exit_code=$(run_generated_expr "${layer} cold count" "${count_expr_file}" "${count_stdout}" "${count_stderr}" count_names_expr "${mx}")
  cold_name_count=$(extract_name_count "${count_stdout}")
  cold_gensym_count=$(extract_gensym_count "${count_stdout}")
  if [[ "${exit_code}" != "0" || -z "${cold_name_count}" || -z "${cold_gensym_count}" ]]; then
    printf 'FAIL\n'
    record_failure "${layer}" "cold-name-count"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "${layer}" "${cold_bytes}" "${cold_gensym_count:-FAIL}" "no" "FAIL" "no" "FAIL" >> "${summary_file}"
    continue
  fi
  printf 'ok bytes=%s names=%s gensyms=%s\n' "${cold_bytes}" "${cold_name_count}" "${cold_gensym_count}"

  printf '  dimension B gensym hygiene ... '
  if [[ "${cold_gensym_count}" == "0" ]]; then
    printf 'ok\n'
  else
    printf 'FAIL gensym_count=%s\n' "${cold_gensym_count}"
    record_failure "${layer}" "gensym-count-${cold_gensym_count}"
  fi

  cold_test_expr_file="${expr_dir}/${slug}.cold-test.expr"
  cold_test_stdout="${logs_dir}/${slug}.cold-test.stdout"
  cold_test_stderr="${logs_dir}/${slug}.cold-test.stderr"
  printf '  cold run_fast %s ... ' "${test_pattern}"
  exit_code=$(run_generated_expr "${layer} cold run_fast" "${cold_test_expr_file}" "${cold_test_stdout}" "${cold_test_stderr}" run_fast_expr "${scratch_root}" "${test_pattern}")
  cold_tests=$(extract_test_summary "${cold_test_stdout}")
  if [[ "${exit_code}" != "0" || -z "${cold_tests}" ]]; then
    printf 'FAIL\n'
    record_failure "${layer}" "cold-tests"
    cold_tests="FAIL"
  else
    printf 'ok tests=%s\n' "${cold_tests}"
  fi

  cycle_stable="yes"
  for cycle in $(seq 1 "${cycles}"); do
    cycle_expr_file="${expr_dir}/${slug}.cycle-${cycle}.expr"
    cycle_stdout="${logs_dir}/${slug}.cycle-${cycle}.stdout"
    cycle_stderr="${logs_dir}/${slug}.cycle-${cycle}.stderr"
    printf '  cycle %s/%s restore-redump ... ' "${cycle}" "${cycles}"
    exit_code=$(run_generated_expr "${layer} cycle ${cycle}" "${cycle_expr_file}" "${cycle_stdout}" "${cycle_stderr}" redump_expr "${mx}")
    if [[ "${exit_code}" != "0" ]]; then
      cycle_stable="no"
      printf 'FAIL\n'
      record_failure "${layer}" "cycle-${cycle}"
      break
    fi
    cycle_bytes=$(file_bytes "${mx}")
    cycle_sha=$(file_sha "${mx}")
    if [[ "${cycle_bytes}" == "${cold_bytes}" && "${cycle_sha}" == "${cold_sha}" ]]; then
      printf 'ok bytes=%s\n' "${cycle_bytes}"
    else
      cycle_stable="no"
      printf 'FAIL bytes=%s expected=%s\n' "${cycle_bytes}" "${cold_bytes}"
      record_failure "${layer}" "cycle-${cycle}-bytes-${cycle_bytes}-expected-${cold_bytes}"
      break
    fi
  done

  restored_test_expr_file="${expr_dir}/${slug}.restored-test.expr"
  restored_test_stdout="${logs_dir}/${slug}.restored-test.stdout"
  restored_test_stderr="${logs_dir}/${slug}.restored-test.stderr"
  printf '  restored run_fast %s ... ' "${test_pattern}"
  exit_code=$(run_generated_expr "${layer} restored run_fast" "${restored_test_expr_file}" "${restored_test_stdout}" "${restored_test_stderr}" run_fast_expr "${scratch_root}" "${test_pattern}")
  restored_tests=$(extract_test_summary "${restored_test_stdout}")
  if [[ "${exit_code}" == "0" && -n "${restored_tests}" && "${restored_tests}" == "${cold_tests}" ]]; then
    test_stable="yes"
    printf 'ok tests=%s\n' "${restored_tests}"
  else
    test_stable="no"
    printf 'FAIL tests=%s expected=%s\n' "${restored_tests:-missing}" "${cold_tests}"
    record_failure "${layer}" "restored-tests"
  fi

  fidelity="na"
  if [[ -n "${prior}" ]]; then
    prior_pattern=$(layer_pattern "${prior}")
    fidelity_build_expr_file="${expr_dir}/${slug}.fidelity-build-prior.expr"
    fidelity_build_stdout="${logs_dir}/${slug}.fidelity-build-prior.stdout"
    fidelity_build_stderr="${logs_dir}/${slug}.fidelity-build-prior.stderr"
    printf '  fidelity cold %s then extend %s ... ' "${prior_pattern}" "${file}"
    exit_code=$(run_generated_expr "${layer} fidelity build prior" "${fidelity_build_expr_file}" "${fidelity_build_stdout}" "${fidelity_build_stderr}" build_snapshot_expr "${scratch_root}" "${prior_pattern}")
    if [[ "${exit_code}" != "0" ]]; then
      fidelity="FAIL"
      printf 'FAIL prior-build\n'
      record_failure "${layer}" "fidelity-prior-build"
    else
      fidelity_extend_expr_file="${expr_dir}/${slug}.fidelity-extend.expr"
      fidelity_extend_stdout="${logs_dir}/${slug}.fidelity-extend.stdout"
      fidelity_extend_stderr="${logs_dir}/${slug}.fidelity-extend.stderr"
      exit_code=$(run_generated_expr "${layer} fidelity extend" "${fidelity_extend_expr_file}" "${fidelity_extend_stdout}" "${fidelity_extend_stderr}" extend_snapshot_expr "${scratch_root}" "${file}")
      if [[ "${exit_code}" != "0" ]]; then
        fidelity="FAIL"
        printf 'FAIL extend\n'
        record_failure "${layer}" "fidelity-extend"
      else
        fidelity_count_expr_file="${expr_dir}/${slug}.fidelity-count.expr"
        fidelity_count_stdout="${logs_dir}/${slug}.fidelity-count.stdout"
        fidelity_count_stderr="${logs_dir}/${slug}.fidelity-count.stderr"
        exit_code=$(run_generated_expr "${layer} fidelity count" "${fidelity_count_expr_file}" "${fidelity_count_stdout}" "${fidelity_count_stderr}" count_names_expr "${mx}")
        fidelity_name_count=$(extract_name_count "${fidelity_count_stdout}")

        fidelity_test_expr_file="${expr_dir}/${slug}.fidelity-test.expr"
        fidelity_test_stdout="${logs_dir}/${slug}.fidelity-test.stdout"
        fidelity_test_stderr="${logs_dir}/${slug}.fidelity-test.stderr"
        test_exit_code=$(run_generated_expr "${layer} fidelity test" "${fidelity_test_expr_file}" "${fidelity_test_stdout}" "${fidelity_test_stderr}" run_fast_expr "${scratch_root}" "${test_pattern}")
        fidelity_tests=$(extract_test_summary "${fidelity_test_stdout}")

        if [[ "${exit_code}" == "0" && "${test_exit_code}" == "0" &&
              "${fidelity_name_count}" == "${cold_name_count}" &&
              "${fidelity_tests}" == "${cold_tests}" ]]; then
          fidelity="ok"
          printf 'ok names=%s tests=%s\n' "${fidelity_name_count}" "${fidelity_tests}"
        else
          fidelity="FAIL"
          printf 'FAIL names=%s expected=%s tests=%s expected=%s\n' \
            "${fidelity_name_count:-missing}" "${cold_name_count}" \
            "${fidelity_tests:-missing}" "${cold_tests}"
          record_failure "${layer}" "fidelity-mismatch"
        fi
      fi
    fi
  else
    printf '  fidelity ... n/a (no prior layer)\n'
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${layer}" "${cold_bytes}" "${cold_gensym_count}" "${cycle_stable}" \
    "${cold_tests}" "${test_stable}" "${fidelity}" >> "${summary_file}"
done

echo "[snaprel] summary: ${summary_file}"
if [[ -n "${failures}" ]]; then
  echo "[snaprel] failures: ${failures}" >&2
  exit 1
fi

exit 0
