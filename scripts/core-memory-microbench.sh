#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/core-memory-microbench.sh [options]

Collect a cross-engine core memory microbenchmark baseline for rum and
WolframKernel. Each workload runs in its own fresh process under /usr/bin/time -l.

Options:
  --output-dir PATH    Output directory. Defaults to a /tmp directory.
  --engine ENGINE      rum, wolfram, or both. Defaults to both.
  --rum-cli PATH       Existing rum-cli binary. Defaults to target/release/rum-cli.
  --wolfram PATH       wolframscript binary. Defaults to /usr/local/bin/wolframscript.
  -h, --help           Show this help.

Outputs:
  measurements.tsv     One row per engine/workload measurement.
  comparison.tsv       rum/WolframKernel memory and RSS ratios by label.
  raw/                 Per-engine, per-workload stdout/stderr logs.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)

output_dir=${RUM_CORE_MEMORY_OUTPUT_DIR:-}
engine=${RUM_CORE_MEMORY_ENGINE:-both}
rum_cli_bin=${RUM_CLI_BIN:-"${repo_root}/target/release/rum-cli"}
wolfram_bin=${WOLFRAM_KERNEL:-/usr/local/bin/wolframscript}

while [[ $# -gt 0 ]]; do
  case "$1" in
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

if [[ ! -x /usr/bin/time ]]; then
  echo "error: /usr/bin/time is not executable" >&2
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

if [[ "${engine}" == "wolfram" || "${engine}" == "both" ]]; then
  if [[ ! -x "${wolfram_bin}" ]]; then
    echo "error: wolframscript binary is not executable: ${wolfram_bin}" >&2
    exit 2
  fi
fi

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-core-memory-microbench.XXXXXX)
else
  mkdir -p "${output_dir}"
fi
raw_dir="${output_dir}/raw"
mkdir -p "${raw_dir}"

measurements_tsv="${output_dir}/measurements.tsv"
comparison_tsv="${output_dir}/comparison.tsv"

printf 'engine\tlabel\tmem_before\tmem_after\tmem_delta\tmax_memory\tpeak_rss_bytes\n' > "${measurements_tsv}"
printf 'label\trum_mem_delta\twk_mem_delta\tmem_delta_ratio\trum_max_memory\twk_max_memory\tmax_memory_ratio\trum_rss_delta\twk_rss_delta\trss_delta_ratio\n' > "${comparison_tsv}"

labels=(
  baseline
  module-leak-20000
  retain-range-1000
  retain-range-10000
)

workload_expr() {
  case "$1" in
    baseline) printf '%s' 'Null' ;;
    module-leak-20000) printf '%s' 'Do[Module[{x}, x = i^2; x], {i, 1, 20000}]' ;;
    retain-range-1000) printf '%s' 'memGlobal = Range[1000]' ;;
    retain-range-10000) printf '%s' 'memGlobal = Range[10000]' ;;
    *) echo "error: unknown workload label $1" >&2; exit 2 ;;
  esac
}

parse_peak_rss() {
  local stderr_path=$1
  awk '
    /maximum resident set size/ {
      for (i = 1; i <= NF; i++) {
        if ($i ~ /^[0-9]+$/) {
          print $i;
          exit;
        }
      }
    }
  ' "${stderr_path}" || true
}

parse_measurement() {
  local engine_name=$1
  local stdout_path=$2
  local peak_rss=$3

  awk -v engine="${engine_name}" -v peak_rss="${peak_rss}" '
    function trim(x) {
      gsub(/^[[:space:]]+/, "", x);
      gsub(/[[:space:]]+$/, "", x);
      return x;
    }
    {
      line = $0;
      gsub(/\r/, "", line);
      sub(/^"/, "", line);
      sub(/"$/, "", line);
      gsub(/\\t/, "\t", line);
      split(line, fields, "\t");
    }
    fields[1] == "RUM_MEM" {
      before = trim(fields[3]);
      after = trim(fields[4]);
      max_memory = trim(fields[5]);
      print engine "\t" trim(fields[2]) "\t" before "\t" after "\t" (after - before) "\t" max_memory "\t" peak_rss;
    }
  ' "${stdout_path}"
}

run_workload() {
  local engine_name=$1
  local label=$2
  local expr
  local stdout_path="${raw_dir}/${engine_name}.${label}.stdout"
  local stderr_path="${raw_dir}/${engine_name}.${label}.stderr"
  local program parsed peak_rss exit_code

  expr=$(workload_expr "${label}")

  # The b0/b1/mx ownvalues add a tiny constant in both engines; compare deltas.
  program="b0 = MemoryInUse[]; ${expr}; b1 = MemoryInUse[]; mx = MaxMemoryUsed[]; Print[\"RUM_MEM\t${label}\t\", b0, \"\t\", b1, \"\t\", mx]"

  set +e
  if [[ "${engine_name}" == "rum" ]]; then
    /usr/bin/time -l "${rum_cli_bin}" --eval "${program}" >"${stdout_path}" 2>"${stderr_path}"
  else
    /usr/bin/time -l "${wolfram_bin}" -code "${program}" >"${stdout_path}" 2>"${stderr_path}"
  fi
  exit_code=$?
  set -e

  peak_rss=$(parse_peak_rss "${stderr_path}")
  if [[ "${exit_code}" != "0" ]]; then
    echo "error: ${engine_name} ${label} exited with ${exit_code}; see ${stderr_path}" >&2
    exit "${exit_code}"
  fi

  parsed=$(parse_measurement "${engine_name}" "${stdout_path}" "${peak_rss}")
  if [[ -z "${parsed}" ]]; then
    echo "error: ${engine_name} ${label} did not emit a RUM_MEM row; see ${stdout_path}" >&2
    exit 1
  fi
  printf '%s\n' "${parsed}" >> "${measurements_tsv}"
}

engines=()
case "${engine}" in
  both) engines=(rum wolfram) ;;
  *) engines=("${engine}") ;;
esac

for engine_name in "${engines[@]}"; do
  for label in "${labels[@]}"; do
    run_workload "${engine_name}" "${label}"
  done
done

label_order="${labels[*]}"
awk -F '\t' -v label_order="${label_order}" '
  BEGIN {
    label_count = split(label_order, labels, " ");
  }
  NR == 1 { next }
  {
    label = $2;
    engine = $1;
    mem_delta[label, engine] = $5;
    max_memory[label, engine] = $6;
    peak_rss[label, engine] = $7;
    if (label == "baseline" && $7 != "") {
      baseline_rss[engine] = $7;
    }
  }
  function ratio(num, den) {
    if (num != "" && den != "" && den + 0 != 0) {
      return sprintf("%.6f", (num + 0) / (den + 0));
    }
    return "";
  }
  function rss_delta(label, engine, value) {
    value = peak_rss[label, engine];
    if (value == "" || baseline_rss[engine] == "") {
      return "";
    }
    return (value + 0) - (baseline_rss[engine] + 0);
  }
  END {
    for (i = 1; i <= label_count; i++) {
      label = labels[i];
      if (label == "") {
        continue;
      }
      rum_mem = mem_delta[label, "rum"];
      wk_mem = mem_delta[label, "wolfram"];
      rum_max = max_memory[label, "rum"];
      wk_max = max_memory[label, "wolfram"];
      rum_rss = rss_delta(label, "rum");
      wk_rss = rss_delta(label, "wolfram");
      print label "\t" rum_mem "\t" wk_mem "\t" ratio(rum_mem, wk_mem) "\t" rum_max "\t" wk_max "\t" ratio(rum_max, wk_max) "\t" rum_rss "\t" wk_rss "\t" ratio(rum_rss, wk_rss);
    }
  }
' "${measurements_tsv}" >> "${comparison_tsv}"

echo "core memory microbench output: ${output_dir}"
