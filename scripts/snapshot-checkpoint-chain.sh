#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/snapshot-checkpoint-chain.sh --mma-hol PATH [options]

Build a cumulative mma-hol snapshot checkpoint chain on a scratch copy.
The source mma-hol checkout is never written; build_snapshot/extend_snapshot
run only inside the scratch tree.

Options:
  --mma-hol PATH       Source mma-hol checkout root. Defaults to RUM_MMA_HOL_ROOT.
  --output-dir PATH    Directory for checkpoints, profiles, manifest, logs.
  --base PATTERN       Cold base layer pattern. Defaults to Simp.
  --timeout SECONDS    Per-step wall timeout. Defaults to 1800.
  --limit N            Stop after N extend steps. Defaults to all.
  --rum-cli PATH       Existing rum-cli binary. Defaults to RUM_CLI_BIN, then
                       target/release/rum-cli, building it if needed.
  --list               Print the build_snapshot library order and exit.
  -h, --help           Show this help.

Exit codes:
  0   selected checkpoint chain completed
  1   a checkpoint step failed or timed out
  2   usage or environment error
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)

mma_hol_source=${RUM_MMA_HOL_ROOT:-}
output_dir=${RUM_SNAPSHOT_CHAIN_OUT:-}
base_pattern=${RUM_SNAPSHOT_CHAIN_BASE:-Simp}
timeout_seconds=${RUM_SNAPSHOT_CHAIN_TIMEOUT:-1800}
limit=0
rum_cli_bin=${RUM_CLI_BIN:-}
list_only=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mma-hol)
      [[ $# -ge 2 ]] || { echo "error: --mma-hol expects a path" >&2; exit 2; }
      mma_hol_source=$2
      shift 2
      ;;
    --output-dir)
      [[ $# -ge 2 ]] || { echo "error: --output-dir expects a path" >&2; exit 2; }
      output_dir=$2
      shift 2
      ;;
    --base)
      [[ $# -ge 2 ]] || { echo "error: --base expects a pattern" >&2; exit 2; }
      base_pattern=$2
      shift 2
      ;;
    --timeout)
      [[ $# -ge 2 ]] || { echo "error: --timeout expects seconds" >&2; exit 2; }
      timeout_seconds=$2
      shift 2
      ;;
    --limit)
      [[ $# -ge 2 ]] || { echo "error: --limit expects a non-negative integer" >&2; exit 2; }
      limit=$2
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

require_positive_int() {
  local name=$1
  local value=$2
  if ! [[ "${value}" =~ ^[1-9][0-9]*$ ]]; then
    echo "error: ${name} expects a positive integer" >&2
    exit 2
  fi
}

require_nonnegative_int() {
  local name=$1
  local value=$2
  if ! [[ "${value}" =~ ^[0-9]+$ ]]; then
    echo "error: ${name} expects a non-negative integer" >&2
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

is_under_dir() {
  local child=$1
  local parent=$2
  case "${child}" in
    "${parent}"|"${parent}"/*) return 0 ;;
    *) return 1 ;;
  esac
}

wl_string() {
  local value=${1//\\/\\\\}
  value=${value//\"/\\\"}
  printf '"%s"' "${value}"
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

count_snapshot_expr() {
  local mx=$1
  printf 'Get[%s]; ' "$(wl_string "${mx}")"
  printf 'Module[{holContexts, names}, '
  printf 'holContexts = Select[Contexts[], StringMatchQ[#, "HOL`*"] &]; '
  printf 'names = Names["HOL`*"]; '
  printf 'Print["[checkpoint] num_contexts=", Length[holContexts]]; '
  printf 'Print["[checkpoint] num_names=", Length[names]]]; '
  printf '"ok"'
}

stage_slug() {
  local slug=${1//\//_}
  slug=${slug//./_}
  slug=${slug//[^A-Za-z0-9_-]/_}
  printf '%s' "${slug}"
}

file_bytes() {
  wc -c < "$1" | tr -d '[:space:]'
}

elapsed_now() {
  date +%s
}

load_library_list() {
  local build_script=$1
  python3 - "${build_script}" <<'PY'
import re
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()
match = re.search(r"\blibs\s*=\s*\{(?P<body>.*?)^\s*\};", text, re.S | re.M)
if not match:
    raise SystemExit("error: could not find libs list in build_snapshot.wls")

index = 0
for line in match.group("body").splitlines():
    parts = re.findall(r'"([^"]+)"', line)
    if parts:
        index += 1
        print(f"{index}\t{'/'.join(parts)}")
PY
}

extract_context_count() {
  local file=$1
  python3 - "${file}" <<'PY'
import re
import sys

text = open(sys.argv[1], encoding="utf-8", errors="replace").read()
matches = re.findall(r"(?:re-)?dumping\s+([0-9]+)\s+HOL contexts", text)
print(matches[-1] if matches else "")
PY
}

extract_checkpoint_count() {
  local key=$1
  local file=$2
  awk -F= -v key="${key}" '$0 ~ "^\\[checkpoint\\] " key "=" { value = $2 } END { print value }' "${file}"
}

classify_step() {
  local exit_code=$1
  local stdout_file=$2
  local stderr_file=$3

  if [[ "${exit_code}" == "124" ]]; then
    printf 'timeout'
    return
  fi
  if [[ "${exit_code}" != "0" ]]; then
    printf 'error'
    return
  fi
  if grep -q 'TerminatedEvaluation' "${stdout_file}" "${stderr_file}"; then
    printf 'error'
    return
  fi
  if grep -q '\$Failed' "${stdout_file}" "${stderr_file}"; then
    printf 'error'
    return
  fi
  printf 'ok'
}

write_profile() {
  local profile_file=$1
  local index=$2
  local module=$3
  local load_seconds=$4
  local cumulative_seconds=$5
  local size_bytes=$6
  local num_contexts=$7
  local num_names=$8
  shift 8
  local libs=("$@")
  local lib

  {
    printf 'index: %s\n' "${index}"
    printf 'module: %s\n' "${module}"
    printf 'load_seconds: %s\n' "${load_seconds}"
    printf 'cumulative_seconds: %s\n' "${cumulative_seconds}"
    printf 'size_bytes: %s\n' "${size_bytes}"
    printf 'num_contexts: %s\n' "${num_contexts}"
    printf 'num_names: %s\n' "${num_names}"
    printf 'libraries:\n'
    for lib in "${libs[@]}"; do
      printf '  %s\n' "${lib}"
    done
  } > "${profile_file}"
}

RUN_EXIT_CODE=0
RUN_ELAPSED=0

run_expr_file() {
  local expr_file=$1
  local stdout_file=$2
  local stderr_file=$3
  local start
  local end

  start=$(elapsed_now)
  set +e
  "${bounded_runner}" --timeout "${timeout_seconds}" -- \
    "${rum_cli_bin}" --eval "$(< "${expr_file}")" \
    > "${stdout_file}" 2> "${stderr_file}"
  RUN_EXIT_CODE=$?
  set -e
  end=$(elapsed_now)
  RUN_ELAPSED=$((end - start))
}

run_count_for_snapshot() {
  local label=$1
  local snapshot_file=$2
  local expr_file=$3
  local stdout_file=$4
  local stderr_file=$5

  count_snapshot_expr "${snapshot_file}" > "${expr_file}"
  run_expr_file "${expr_file}" "${stdout_file}" "${stderr_file}"
  if [[ "$(classify_step "${RUN_EXIT_CODE}" "${stdout_file}" "${stderr_file}")" != "ok" ]]; then
    echo "[checkpoint-chain] count failed for ${label}" >&2
    echo "[checkpoint-chain] stdout: ${stdout_file}" >&2
    echo "[checkpoint-chain] stderr: ${stderr_file}" >&2
    return 1
  fi
  return 0
}

require_positive_int "--timeout" "${timeout_seconds}"
require_nonnegative_int "--limit" "${limit}"

if [[ -z "${mma_hol_source}" ]]; then
  echo "error: provide --mma-hol PATH or set RUM_MMA_HOL_ROOT" >&2
  exit 2
fi

if [[ ! -f "${mma_hol_source}/tests/build_snapshot.wls" ||
      ! -f "${mma_hol_source}/tests/extend_snapshot.wls" ]]; then
  echo "error: ${mma_hol_source} does not look like an mma-hol checkout" >&2
  exit 2
fi

mma_hol_source=$(abs_dir "${mma_hol_source}")

source_libs=()
source_lib_indices=()
while IFS=$'\t' read -r index rel; do
  source_lib_indices+=("${index}")
  source_libs+=("${rel}")
done < <(load_library_list "${mma_hol_source}/tests/build_snapshot.wls")

if [[ "${#source_libs[@]}" -eq 0 ]]; then
  echo "error: no libraries parsed from ${mma_hol_source}/tests/build_snapshot.wls" >&2
  exit 2
fi

base_array_index=-1
for i in "${!source_libs[@]}"; do
  filename=$(basename "${source_libs[$i]}")
  if [[ "${filename}" == *"${base_pattern}"* ]]; then
    base_array_index=${i}
    break
  fi
done

if [[ "${base_array_index}" -lt 0 ]]; then
  echo "error: no library filename contains base pattern '${base_pattern}'" >&2
  exit 2
fi

if [[ "${list_only}" == "1" ]]; then
  printf 'base pattern: %s\n' "${base_pattern}"
  for i in "${!source_libs[@]}"; do
    marker=""
    if [[ "${i}" -eq "${base_array_index}" ]]; then
      marker=" base"
    fi
    printf '%02d\t%s%s\n' "${source_lib_indices[$i]}" "${source_libs[$i]}" "${marker}"
  done
  exit 0
fi

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-snapshot-chain.XXXXXX)
else
  mkdir -p "${output_dir}"
  output_dir=$(abs_dir "${output_dir}")
fi

if is_under_dir "${output_dir}" "${mma_hol_source}"; then
  echo "error: --output-dir must not be inside the source mma-hol checkout" >&2
  exit 2
fi

logs_dir="${output_dir}/logs"
expr_dir="${output_dir}/expr"
mkdir -p "${logs_dir}" "${expr_dir}"

scratch_parent=$(mktemp -d "${output_dir}/scratch.XXXXXX")
scratch_root="${scratch_parent}/mma-hol"
mkdir -p "${scratch_root}"
rsync -a --delete --exclude '.git/' --exclude '*.mx' \
  "${mma_hol_source}/" "${scratch_root}/"

scratch_libs=()
scratch_lib_indices=()
while IFS=$'\t' read -r index rel; do
  scratch_lib_indices+=("${index}")
  scratch_libs+=("${rel}")
done < <(load_library_list "${scratch_root}/tests/build_snapshot.wls")

if [[ "${source_libs[*]}" != "${scratch_libs[*]}" ]]; then
  echo "error: scratch build_snapshot library order differs from source" >&2
  exit 2
fi

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

manifest_file="${output_dir}/manifest.tsv"
printf 'index\tmodule\tsnapshot_file\tload_seconds\tcumulative_seconds\tsize_bytes\tnum_contexts\tnum_names\tstatus\n' > "${manifest_file}"

base_module=${source_libs[$base_array_index]}
base_slug=$(stage_slug "${base_module}")
base_checkpoint="${output_dir}/cp00-${base_slug}.mx"
base_profile="${output_dir}/cp00-${base_slug}.profile.txt"
mx="${scratch_root}/bootstrap.mx"
cumulative_seconds=0
deepest_module=""
stuck_module=""
stuck_status=""
stuck_seconds=""
loaded_modules=()

echo "[checkpoint-chain] source: ${mma_hol_source}"
echo "[checkpoint-chain] scratch: ${scratch_root}"
echo "[checkpoint-chain] output: ${output_dir}"
echo "[checkpoint-chain] rum-cli: ${rum_cli_bin}"
echo "[checkpoint-chain] base: ${base_pattern} -> ${base_module}; timeout: ${timeout_seconds}s; limit: ${limit}"

base_expr="${expr_dir}/cp00-${base_slug}.expr"
base_stdout="${logs_dir}/cp00-${base_slug}.stdout"
base_stderr="${logs_dir}/cp00-${base_slug}.stderr"
build_snapshot_expr "${scratch_root}" "${base_pattern}" > "${base_expr}"

printf '[00] base %s ... ' "${base_module}"
run_expr_file "${base_expr}" "${base_stdout}" "${base_stderr}"
base_status=$(classify_step "${RUN_EXIT_CODE}" "${base_stdout}" "${base_stderr}")
base_seconds=${RUN_ELAPSED}
if [[ "${base_status}" == "ok" && ! -f "${mx}" ]]; then
  base_status="error"
fi

if [[ "${base_status}" != "ok" ]]; then
  printf '%s (%ss)\n' "${base_status}" "${base_seconds}"
  printf '00\t%s\tstderr:%s\t%s\t%s\t\t\t\t%s\n' \
    "${base_module}" "${base_stderr}" "${base_seconds}" "${cumulative_seconds}" \
    "${base_status}" >> "${manifest_file}"
  echo "STUCK AT: ${base_module} (${base_status}, ${base_seconds}s)"
  echo "[checkpoint-chain] stdout: ${base_stdout}" >&2
  echo "[checkpoint-chain] stderr: ${base_stderr}" >&2
  exit 1
fi

rsync -a "${mx}" "${base_checkpoint}"
cumulative_seconds=$((cumulative_seconds + base_seconds))
base_size=$(file_bytes "${base_checkpoint}")
base_contexts=$(extract_context_count "${base_stdout}")

base_count_expr="${expr_dir}/cp00-${base_slug}.count.expr"
base_count_stdout="${logs_dir}/cp00-${base_slug}.count.stdout"
base_count_stderr="${logs_dir}/cp00-${base_slug}.count.stderr"
if ! run_count_for_snapshot "cp00-${base_slug}" "${base_checkpoint}" \
  "${base_count_expr}" "${base_count_stdout}" "${base_count_stderr}"; then
  printf 'error (%ss)\n' "${base_seconds}"
  printf '00\t%s\tstderr:%s\t%s\t%s\t%s\t%s\t\t%s\n' \
    "${base_module}" "${base_count_stderr}" "${base_seconds}" "${cumulative_seconds}" \
    "${base_size}" "${base_contexts}" "error" >> "${manifest_file}"
  echo "STUCK AT: ${base_module} (error, ${base_seconds}s)"
  exit 1
fi

base_contexts=$(extract_checkpoint_count "num_contexts" "${base_count_stdout}")
base_names=$(extract_checkpoint_count "num_names" "${base_count_stdout}")
loaded_modules=("${source_libs[@]:0:$((base_array_index + 1))}")
write_profile "${base_profile}" "00" "${base_module}" "${base_seconds}" \
  "${cumulative_seconds}" "${base_size}" "${base_contexts}" "${base_names}" \
  "${loaded_modules[@]}"
printf 'ok (%ss, %s bytes, %s contexts, %s names)\n' \
  "${base_seconds}" "${base_size}" "${base_contexts}" "${base_names}"
printf '00\t%s\t%s\t%s\t%s\t%s\t%s\t%s\tok\n' \
  "${base_module}" "${base_checkpoint}" "${base_seconds}" "${cumulative_seconds}" \
  "${base_size}" "${base_contexts}" "${base_names}" >> "${manifest_file}"
deepest_module="${base_module}"

extend_done=0
for ((i = base_array_index + 1; i < ${#source_libs[@]}; i++)); do
  if [[ "${limit}" -gt 0 && "${extend_done}" -ge "${limit}" ]]; then
    break
  fi

  module=${source_libs[$i]}
  display_index=${source_lib_indices[$i]}
  index_label=$(printf '%02d' "${display_index}")
  slug=$(stage_slug "${module}")
  checkpoint="${output_dir}/cp${index_label}-${slug}.mx"
  profile="${output_dir}/cp${index_label}-${slug}.profile.txt"
  expr_file="${expr_dir}/cp${index_label}-${slug}.expr"
  stdout_file="${logs_dir}/cp${index_label}-${slug}.stdout"
  stderr_file="${logs_dir}/cp${index_label}-${slug}.stderr"

  extend_snapshot_expr "${scratch_root}" "${module}" > "${expr_file}"

  printf '[%s] extend %s ... ' "${index_label}" "${module}"
  run_expr_file "${expr_file}" "${stdout_file}" "${stderr_file}"
  status=$(classify_step "${RUN_EXIT_CODE}" "${stdout_file}" "${stderr_file}")
  step_seconds=${RUN_ELAPSED}
  if [[ "${status}" == "ok" && ! -f "${mx}" ]]; then
    status="error"
  fi

  if [[ "${status}" != "ok" ]]; then
    printf '%s (%ss)\n' "${status}" "${step_seconds}"
    printf '%s\t%s\tstderr:%s\t%s\t%s\t\t\t\t%s\n' \
      "${index_label}" "${module}" "${stderr_file}" "${step_seconds}" \
      "${cumulative_seconds}" "${status}" >> "${manifest_file}"
    stuck_module="${module}"
    stuck_status="${status}"
    stuck_seconds="${step_seconds}"
    echo "STUCK AT: ${module} (${status}, ${step_seconds}s)"
    echo "[checkpoint-chain] stdout: ${stdout_file}" >&2
    echo "[checkpoint-chain] stderr: ${stderr_file}" >&2
    break
  fi

  rsync -a "${mx}" "${checkpoint}"
  cumulative_seconds=$((cumulative_seconds + step_seconds))
  size_bytes=$(file_bytes "${checkpoint}")
  num_contexts=$(extract_context_count "${stdout_file}")

  count_expr="${expr_dir}/cp${index_label}-${slug}.count.expr"
  count_stdout="${logs_dir}/cp${index_label}-${slug}.count.stdout"
  count_stderr="${logs_dir}/cp${index_label}-${slug}.count.stderr"
  if ! run_count_for_snapshot "cp${index_label}-${slug}" "${checkpoint}" \
    "${count_expr}" "${count_stdout}" "${count_stderr}"; then
    printf 'error (%ss)\n' "${step_seconds}"
    printf '%s\t%s\tstderr:%s\t%s\t%s\t%s\t%s\t\t%s\n' \
      "${index_label}" "${module}" "${count_stderr}" "${step_seconds}" \
      "${cumulative_seconds}" "${size_bytes}" "${num_contexts}" "error" \
      >> "${manifest_file}"
    stuck_module="${module}"
    stuck_status="error"
    stuck_seconds="${step_seconds}"
    echo "STUCK AT: ${module} (error, ${step_seconds}s)"
    break
  fi

  num_contexts=$(extract_checkpoint_count "num_contexts" "${count_stdout}")
  num_names=$(extract_checkpoint_count "num_names" "${count_stdout}")
  loaded_modules+=("${module}")
  write_profile "${profile}" "${index_label}" "${module}" "${step_seconds}" \
    "${cumulative_seconds}" "${size_bytes}" "${num_contexts}" "${num_names}" \
    "${loaded_modules[@]}"
  printf 'ok (%ss, %s bytes, %s contexts, %s names)\n' \
    "${step_seconds}" "${size_bytes}" "${num_contexts}" "${num_names}"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\tok\n' \
    "${index_label}" "${module}" "${checkpoint}" "${step_seconds}" \
    "${cumulative_seconds}" "${size_bytes}" "${num_contexts}" "${num_names}" \
    >> "${manifest_file}"
  deepest_module="${module}"
  extend_done=$((extend_done + 1))
done

echo "[checkpoint-chain] manifest: ${manifest_file}"
echo "[checkpoint-chain] deepest module reached: ${deepest_module:-none}"
echo "[checkpoint-chain] cumulative seconds: ${cumulative_seconds}"

if [[ -n "${stuck_module}" ]]; then
  echo "[checkpoint-chain] stuck point: ${stuck_module} (${stuck_status}, ${stuck_seconds}s)"
  exit 1
fi

echo "[checkpoint-chain] completed all ${extend_done} selected extend steps"
exit 0
