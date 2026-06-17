#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/pair-simp-hotspot-probes.sh [--mma-hol PATH] [options]

Run bounded Pair/Simp hotspot probes against an unmodified mma-hol checkout.
These are diagnostics for localizing the Pair.wl / Simp performance gate, not
acceptance tests and not mma-hol patches.

Options:
  --mma-hol PATH       mma-hol checkout root. Defaults to RUM_MMA_HOL_ROOT,
                       then /private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol.
  --only NAME          Run one probe by file stem, e.g. basic-only-pair4.
  --timeout SECONDS    Per-probe wall timeout. Defaults to 180.
  --output-dir PATH    Directory for summary.tsv and per-probe logs.
  --engine ENGINE      rum or wolfram. Defaults to rum.
  --rum-cli PATH       Existing rum-cli binary. Defaults to RUM_CLI_BIN, then
                       target/release/rum-cli, building it if absent.
  --wolfram PATH       WolframKernel binary for --engine wolfram.
  --keep-going         Continue after a probe fails or times out.
  --list               Print available probes and exit.
  -h, --help           Show this help.

Environment overrides:
  RUM_MMA_HOL_ROOT
  RUM_PAIR_SIMP_TIMEOUT
  RUM_PAIR_SIMP_OUTPUT_DIR
  RUM_PAIR_SIMP_ENGINE
  RUM_CLI_BIN
  WOLFRAM_KERNEL
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
probe_dir="${repo_root}/scripts/perf-probes/pair-simp"
bounded_runner="${repo_root}/scripts/run-bounded.py"

mma_hol_root=${RUM_MMA_HOL_ROOT:-/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol}
timeout_seconds=${RUM_PAIR_SIMP_TIMEOUT:-180}
output_dir=${RUM_PAIR_SIMP_OUTPUT_DIR:-}
engine=${RUM_PAIR_SIMP_ENGINE:-rum}
rum_cli_bin=${RUM_CLI_BIN:-}
wolfram_bin=${WOLFRAM_KERNEL:-/Applications/Wolfram.app/Contents/MacOS/WolframKernel}
only_name=
keep_going=0
list_only=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mma-hol)
      [[ $# -ge 2 ]] || { echo "error: --mma-hol expects a path" >&2; exit 2; }
      mma_hol_root=$2
      shift 2
      ;;
    --only)
      [[ $# -ge 2 ]] || { echo "error: --only expects a probe name" >&2; exit 2; }
      only_name=$2
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
    --engine)
      [[ $# -ge 2 ]] || { echo "error: --engine expects rum or wolfram" >&2; exit 2; }
      engine=$2
      shift 2
      ;;
    --rum-cli)
      [[ $# -ge 2 ]] || { echo "error: --rum-cli expects a path" >&2; exit 2; }
      rum_cli_bin=$2
      shift 2
      ;;
    --wolfram)
      [[ $# -ge 2 ]] || { echo "error: --wolfram expects a path" >&2; exit 2; }
      wolfram_bin=$2
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
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "error: unknown option $1" >&2
      exit 2
      ;;
    *)
      echo "error: unexpected positional argument $1" >&2
      exit 2
      ;;
  esac
done

case "${engine}" in
  rum|wolfram) ;;
  *) echo "error: --engine expects rum or wolfram" >&2; exit 2 ;;
esac

if ! [[ "${timeout_seconds}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --timeout expects a positive integer" >&2
  exit 2
fi

if [[ ! -d "${mma_hol_root}" || ! -f "${mma_hol_root}/auto/Simp.wl" ]]; then
  echo "error: ${mma_hol_root} does not look like an mma-hol checkout" >&2
  exit 2
fi

if [[ ! -x "${bounded_runner}" ]]; then
  echo "error: bounded runner is not executable: ${bounded_runner}" >&2
  exit 2
fi

probe_files=()
while IFS= read -r probe_file; do
  probe_files+=("${probe_file}")
done < <(find "${probe_dir}" -maxdepth 1 -type f -name '*.wls' ! -name 'common.wls' | sort)

if [[ "${#probe_files[@]}" -eq 0 ]]; then
  echo "error: no probes found in ${probe_dir}" >&2
  exit 2
fi

if [[ -n "${only_name}" ]]; then
  selected=()
  for probe_file in "${probe_files[@]}"; do
    stem=$(basename "${probe_file}" .wls)
    if [[ "${stem}" == "${only_name}" ]]; then
      selected+=("${probe_file}")
    fi
  done
  if [[ "${#selected[@]}" -eq 0 ]]; then
    echo "error: unknown probe for --only: ${only_name}" >&2
    exit 2
  fi
  probe_files=("${selected[@]}")
fi

if [[ "${list_only}" == "1" ]]; then
  for probe_file in "${probe_files[@]}"; do
    basename "${probe_file}" .wls
  done
  exit 0
fi

if [[ "${engine}" == "rum" ]]; then
  if [[ -z "${rum_cli_bin}" ]]; then
    rum_cli_bin="${repo_root}/target/release/rum-cli"
  fi
  if [[ ! -x "${rum_cli_bin}" ]]; then
    (
      cd "${repo_root}"
      cargo build --release -p rum-cli >/dev/null
    )
  fi
  if [[ ! -x "${rum_cli_bin}" ]]; then
    echo "error: rum-cli binary is not executable: ${rum_cli_bin}" >&2
    exit 2
  fi
else
  if [[ ! -x "${wolfram_bin}" ]]; then
    echo "error: WolframKernel binary is not executable: ${wolfram_bin}" >&2
    exit 2
  fi
fi

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-pair-simp-probes.XXXXXX)
else
  mkdir -p "${output_dir}"
fi

summary_file="${output_dir}/summary.tsv"
printf 'probe\tengine\tstatus\texit_code\telapsed_seconds\tstdout\tstderr\n' > "${summary_file}"

echo "[rum-pair-simp] mma-hol: ${mma_hol_root}"
echo "[rum-pair-simp] engine: ${engine}; timeout: ${timeout_seconds}s; probes: ${#probe_files[@]}"
echo "[rum-pair-simp] logs: ${output_dir}"

overall_status=0

for index in "${!probe_files[@]}"; do
  probe_file=${probe_files[$index]}
  probe_name=$(basename "${probe_file}" .wls)
  stdout_file="${output_dir}/${probe_name}.${engine}.stdout"
  stderr_file="${output_dir}/${probe_name}.${engine}.stderr"

  printf '[%02d/%02d] %s ... ' "$((index + 1))" "${#probe_files[@]}" "${probe_name}"
  start=$(date +%s)
  set +e
  if [[ "${engine}" == "rum" ]]; then
    (
      cd "${mma_hol_root}"
      if [[ "${probe_name}" == runtime-profile-* ]]; then
        env \
          RUM_PROFILE_LEXICAL_SUBSTITUTION="${RUM_PROFILE_LEXICAL_SUBSTITUTION:-1}" \
          RUM_PROFILE_LEXICAL_DEFER="${RUM_PROFILE_LEXICAL_DEFER:-1}" \
          RUM_PROFILE_LEXICAL_DETAIL="${RUM_PROFILE_LEXICAL_DETAIL:-1}" \
          RUM_PROFILE_LEXICAL_SHAPES="${RUM_PROFILE_LEXICAL_SHAPES:-1}" \
          RUM_PROFILE_LEXICAL_BINDINGS="${RUM_PROFILE_LEXICAL_BINDINGS:-1}" \
          RUM_PROFILE_LEXICAL_SOURCES="${RUM_PROFILE_LEXICAL_SOURCES:-1}" \
          RUM_PROFILE_EVALUATION_CACHE="${RUM_PROFILE_EVALUATION_CACHE:-1}" \
          RUM_PROFILE_EVALUATION_CACHE_DEFER="${RUM_PROFILE_EVALUATION_CACHE_DEFER:-1}" \
          RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL="${RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL:-0}" \
          "${bounded_runner}" --timeout "${timeout_seconds}" -- \
            "${rum_cli_bin}" --file "${probe_file}"
      else
        "${bounded_runner}" --timeout "${timeout_seconds}" -- \
          "${rum_cli_bin}" --file "${probe_file}"
      fi
    ) > "${stdout_file}" 2> "${stderr_file}"
  else
    (
      cd "${mma_hol_root}"
      "${bounded_runner}" --timeout "${timeout_seconds}" -- \
        "${wolfram_bin}" -noprompt -script "${probe_file}"
    ) > "${stdout_file}" 2> "${stderr_file}"
  fi
  exit_code=$?
  set -e
  end=$(date +%s)
  elapsed=$((end - start))

  if [[ "${exit_code}" == "0" ]]; then
    status=ok
  elif [[ "${exit_code}" == "124" ]]; then
    status=timeout
  else
    status=error
  fi

  printf '%s (%ss)\n' "${status}" "${elapsed}"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${probe_name}" "${engine}" "${status}" "${exit_code}" "${elapsed}" \
    "${stdout_file}" "${stderr_file}" >> "${summary_file}"

  if [[ "${status}" != "ok" ]]; then
    overall_status=1
    echo "[rum-pair-simp] failing probe: ${probe_name}" >&2
    echo "[rum-pair-simp] stdout: ${stdout_file}" >&2
    echo "[rum-pair-simp] stderr: ${stderr_file}" >&2
    if [[ "${keep_going}" != "1" ]]; then
      break
    fi
  fi
done

echo "[rum-pair-simp] summary: ${summary_file}"
exit "${overall_status}"
