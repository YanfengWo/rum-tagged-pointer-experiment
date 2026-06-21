#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/perf-baseline-stage0.sh [--mma-hol PATH] [options]

Collect the Stage 0 performance baseline for the Pair/Simp kernel rewrite
mainline. The suite runs the fixed micro-proxy, mid-level, and root probes
against an unmodified mma-hol checkout and writes machine-readable TSV files.

Options:
  --mma-hol PATH       mma-hol checkout root. Defaults to RUM_MMA_HOL_ROOT,
                       then known local clean-clone paths if present.
  --output-dir PATH    Baseline output directory. Defaults to a /tmp directory.
  --engine ENGINE      rum, wolfram, or both. Defaults to both.
  --timeout SECONDS    Per micro/mid probe timeout. Defaults to 300.
  --root-timeout SEC   Per root probe timeout. Defaults to 900.
  --skip-root          Do not run root probes. Useful for quick local checks.
  --include-wk-wrapper Run the WolframKernel-only conversion wrapper profile.
  --rum-cli PATH       Existing rum-cli binary. Defaults to target/release/rum-cli.
  --wolfram PATH       WolframKernel binary.
  -h, --help           Show this help.

Outputs:
  runs.tsv             One row per probe execution.
  measurements.tsv     AbsoluteTiming label/seconds rows parsed from stdout.
  raw/                 Per-probe logs from the underlying runners.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
hotspot_runner="${repo_root}/scripts/pair-simp-hotspot-probes.sh"
bounded_runner="${repo_root}/scripts/run-bounded.py"

mma_hol_root=${RUM_MMA_HOL_ROOT:-}
output_dir=${RUM_PERF_BASELINE_OUTPUT_DIR:-}
engine=${RUM_PERF_BASELINE_ENGINE:-both}
timeout_seconds=${RUM_PERF_BASELINE_TIMEOUT:-300}
root_timeout_seconds=${RUM_PERF_BASELINE_ROOT_TIMEOUT:-900}
skip_root=0
include_wk_wrapper=0
rum_cli_bin=${RUM_CLI_BIN:-"${repo_root}/target/release/rum-cli"}
wolfram_bin=${WOLFRAM_KERNEL:-/Applications/Wolfram.app/Contents/MacOS/WolframKernel}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mma-hol)
      [[ $# -ge 2 ]] || { echo "error: --mma-hol expects a path" >&2; exit 2; }
      mma_hol_root=$2
      shift 2
      ;;
    --output-dir)
      [[ $# -ge 2 ]] || { echo "error: --output-dir expects a path" >&2; exit 2; }
      output_dir=$2
      shift 2
      ;;
    --engine)
      [[ $# -ge 2 ]] || { echo "error: --engine expects rum, wolfram, or both" >&2; exit 2; }
      engine=$2
      shift 2
      ;;
    --timeout)
      [[ $# -ge 2 ]] || { echo "error: --timeout expects seconds" >&2; exit 2; }
      timeout_seconds=$2
      shift 2
      ;;
    --root-timeout)
      [[ $# -ge 2 ]] || { echo "error: --root-timeout expects seconds" >&2; exit 2; }
      root_timeout_seconds=$2
      shift 2
      ;;
    --skip-root)
      skip_root=1
      shift
      ;;
    --include-wk-wrapper)
      include_wk_wrapper=1
      shift
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
  rum|wolfram|both) ;;
  *) echo "error: --engine expects rum, wolfram, or both" >&2; exit 2 ;;
esac

if ! [[ "${timeout_seconds}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --timeout expects a positive integer" >&2
  exit 2
fi
if ! [[ "${root_timeout_seconds}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --root-timeout expects a positive integer" >&2
  exit 2
fi

if [[ -z "${mma_hol_root}" ]]; then
  for candidate in \
    /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol \
    /private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol \
    /private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol
  do
    if [[ -f "${candidate}/auto/Simp.wl" && -f "${candidate}/stdlib/Pair.wl" ]]; then
      mma_hol_root=${candidate}
      break
    fi
  done
fi

if [[ -z "${mma_hol_root}" || ! -f "${mma_hol_root}/auto/Simp.wl" || ! -f "${mma_hol_root}/stdlib/Pair.wl" ]]; then
  echo "error: --mma-hol is required and must point at an mma-hol checkout" >&2
  exit 2
fi

if [[ ! -x "${hotspot_runner}" ]]; then
  echo "error: hotspot runner is not executable: ${hotspot_runner}" >&2
  exit 2
fi
if [[ ! -x "${bounded_runner}" ]]; then
  echo "error: bounded runner is not executable: ${bounded_runner}" >&2
  exit 2
fi

if [[ "${engine}" == "rum" || "${engine}" == "both" ]]; then
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
fi

if [[ "${engine}" == "wolfram" || "${engine}" == "both" || "${include_wk_wrapper}" == "1" ]]; then
  if [[ ! -x "${wolfram_bin}" ]]; then
    echo "error: WolframKernel binary is not executable: ${wolfram_bin}" >&2
    exit 2
  fi
fi

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-stage0-baseline.XXXXXX)
else
  mkdir -p "${output_dir}"
fi
raw_dir="${output_dir}/raw"
mkdir -p "${raw_dir}"

runs_tsv="${output_dir}/runs.tsv"
measurements_tsv="${output_dir}/measurements.tsv"

printf 'layer\tprobe\tengine\tstatus\texit_code\telapsed_seconds\tstdout\tstderr\n' > "${runs_tsv}"
printf 'layer\tprobe\tengine\tlabel\tseconds\tseconds_raw\tstdout\n' > "${measurements_tsv}"

micro_probes=(
  single-basic-failure-split
  runtime-profile-var-basic-baseconv-chain
)
mid_probes=(
  basic-chain-orelsec-split
  repeatc-tryconv-control-split
  pair4-stock-repeat8
)
root_probes=(
  pair-load-root-phase
)

engines=()
case "${engine}" in
  both) engines=(rum wolfram) ;;
  *) engines=("${engine}") ;;
esac

disable_profile_env=(
  RUM_PROFILE_LEXICAL_SUBSTITUTION=0
  RUM_PROFILE_LEXICAL_DEFER=0
  RUM_PROFILE_LEXICAL_DETAIL=0
  RUM_PROFILE_LEXICAL_SHAPES=0
  RUM_PROFILE_LEXICAL_BINDINGS=0
  RUM_PROFILE_LEXICAL_SOURCES=0
  RUM_PROFILE_LEXICAL_SOURCE_FINGERPRINTS=0
  RUM_PROFILE_LEXICAL_PROVENANCE=0
  RUM_PROFILE_EVALUATION_CACHE=0
  RUM_PROFILE_EVALUATION_CACHE_DEFER=0
  RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=0
  RUM_PROFILE_RULE_RHS_TIMING=0
  RUM_PROFILE_DEFINITION_RULE_TIMING=0
  RUM_PROFILE_BUILTIN_TIMING=0
)

extract_measurements() {
  local layer=$1
  local probe=$2
  local run_engine=$3
  local stdout_file=$4

  if [[ ! -f "${stdout_file}" ]]; then
    return
  fi

  awk -v layer="${layer}" -v probe="${probe}" -v engine="${run_engine}" -v stdout_file="${stdout_file}" '
    /RUM_PAIR_PROBE label=/ {
      label = $0
      sub(/^.*RUM_PAIR_PROBE label=/, "", label)
      gsub(/"/, "", label)
      next
    }
    /RUM_PAIR_PROBE seconds=/ {
      seconds_raw = $0
      sub(/^.*RUM_PAIR_PROBE seconds=/, "", seconds_raw)
      gsub(/"/, "", seconds_raw)
      seconds = seconds_raw
      if (seconds_raw ~ /^[0-9]+\/[0-9]+$/) {
        split(seconds_raw, parts, "/")
        seconds = sprintf("%.9f", parts[1] / parts[2])
      }
      if (label != "" && seconds_raw != "") {
        print layer "\t" probe "\t" engine "\t" label "\t" seconds "\t" seconds_raw "\t" stdout_file
      }
    }
  ' "${stdout_file}" >> "${measurements_tsv}"
}

append_run_summary() {
  local layer=$1
  local summary_file=$2
  if [[ ! -f "${summary_file}" ]]; then
    return
  fi
  awk -v layer="${layer}" 'NR > 1 { print layer "\t" $0 }' "${summary_file}" >> "${runs_tsv}"
}

run_hotspot_probe() {
  local layer=$1
  local probe=$2
  local run_engine=$3
  local timeout=$4
  local probe_output="${raw_dir}/${layer}.${probe}.${run_engine}"

  echo "[stage0] ${layer} ${probe} ${run_engine}"
  set +e
  env "${disable_profile_env[@]}" \
    "${hotspot_runner}" \
      --mma-hol "${mma_hol_root}" \
      --engine "${run_engine}" \
      --only "${probe}" \
      --timeout "${timeout}" \
      --output-dir "${probe_output}" \
      --rum-cli "${rum_cli_bin}" \
      --wolfram "${wolfram_bin}" \
      > "${probe_output}.runner.stdout" \
      2> "${probe_output}.runner.stderr"
  local exit_code=$?
  set -e

  append_run_summary "${layer}" "${probe_output}/summary.tsv"
  extract_measurements \
    "${layer}" \
    "${probe}" \
    "${run_engine}" \
    "${probe_output}/${probe}.${run_engine}.stdout"

  if [[ "${exit_code}" != "0" ]]; then
    echo "[stage0] probe failed: layer=${layer} probe=${probe} engine=${run_engine} exit=${exit_code}" >&2
  fi
}

run_wk_wrapper() {
  local probe=wk-conversion-wrapper-profile
  local layer=oracle-wrapper
  local stdout_file="${raw_dir}/${layer}.${probe}.wolfram.stdout"
  local stderr_file="${raw_dir}/${layer}.${probe}.wolfram.stderr"
  local probe_file="${repo_root}/scripts/perf-probes/pair-simp/wolfram/${probe}.wls"

  echo "[stage0] ${layer} ${probe} wolfram"
  local start end elapsed exit_code status
  start=$(date +%s)
  set +e
  (
    cd "${mma_hol_root}"
    "${bounded_runner}" --timeout "${root_timeout_seconds}" -- \
      "${wolfram_bin}" -noprompt -script "${probe_file}"
  ) > "${stdout_file}" 2> "${stderr_file}"
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

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${layer}" "${probe}" wolfram "${status}" "${exit_code}" "${elapsed}" \
    "${stdout_file}" "${stderr_file}" >> "${runs_tsv}"
  extract_measurements "${layer}" "${probe}" wolfram "${stdout_file}"
}

echo "[stage0] mma-hol: ${mma_hol_root}"
echo "[stage0] output: ${output_dir}"
echo "[stage0] engines: ${engines[*]}"

for run_engine in "${engines[@]}"; do
  for probe in "${micro_probes[@]}"; do
    run_hotspot_probe micro "${probe}" "${run_engine}" "${timeout_seconds}"
  done
  for probe in "${mid_probes[@]}"; do
    run_hotspot_probe mid "${probe}" "${run_engine}" "${timeout_seconds}"
  done
  if [[ "${skip_root}" != "1" ]]; then
    for probe in "${root_probes[@]}"; do
      run_hotspot_probe root "${probe}" "${run_engine}" "${root_timeout_seconds}"
    done
  fi
done

if [[ "${include_wk_wrapper}" == "1" ]]; then
  run_wk_wrapper
fi

echo "[stage0] runs: ${runs_tsv}"
echo "[stage0] measurements: ${measurements_tsv}"
