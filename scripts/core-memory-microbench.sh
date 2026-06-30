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
  comparison.tsv       rum/WolframKernel four-C memory and RSS ratios by label.
  aggregate.tsv        C_max/C_p95/C_geomean plus retained N-drift verdict.
  raw/                 Per-engine, per-workload stdout/stderr logs.

MemoryInUse[] and MaxMemoryUsed[] are compared only as baseline-subtracted
proxy deltas: retained, peak, transient, and post-cleanup. Post-cleanup is
measured after each workload's portable cleanup form; this script does not clear
engine-specific caches. The rum-local alloc_bytes column is empty unless the
provided rum-cli already supports the alloc-count profile and emits it.

INTERPRETATION (important - measured 2026-06-30):
  * retained_ratio is the ONLY clean cross-engine constant-factor metric.
    MemoryInUse[] deltas are workload-attributable in BOTH engines, so the
    retained ratio is meaningful: retain-range gives ~8.1-8.3x, STABLE across
    N (the dense-representation overhead Phase 3 targets). This is the number
    the memory line claims "constant factor" against.
  * peak / transient / post_cleanup / peak_rss / rss_delta WK columns are
    CONFOUNDED; read them RUM-LOCAL, not as WK ratios. WK's MaxMemoryUsed[] is
    a sticky high-water set at kernel startup (~15.4M above current, NOT reset
    per workload), and its process RSS carries ~86M pre-allocated kernel; both
    swamp these small workloads, so the WK side is a near-constant baseline and
    the ratios (peak ~0.00004, rss_delta ~10^3x) are noise. Use rum's own
    peak/transient (mid-eval bloat / TransientOverhead) and post_cleanup (leak /
    cache / intern residual) as rum-local regression + diagnosis. (Sanity:
    module-leak post_cleanup ~879 B = the Phase-1 gensym GC works; the transient
    workload shows rum transient >> retained.)
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
aggregate_tsv="${output_dir}/aggregate.tsv"

printf 'engine\tlabel\tmem_before\tmem_after\tmem_delta\tmax_memory\tmem_postcleanup\tpeak_rss_bytes\talloc_bytes\n' > "${measurements_tsv}"
printf 'label\trum_retained\twk_retained\tretained_ratio\trum_peak\twk_peak\tpeak_ratio\trum_transient\twk_transient\ttransient_ratio\trum_post_cleanup\twk_post_cleanup\tpost_cleanup_ratio\trum_peak_rss\twk_peak_rss\tpeak_rss_ratio\trum_rss_delta\twk_rss_delta\trss_delta_ratio\trum_alloc_bytes\n' > "${comparison_tsv}"
printf 'statistic\tmetric\tvalue\tdetail\n' > "${aggregate_tsv}"

labels=(
  baseline
  module-leak-20000
  retain-range-1000
  retain-range-10000
  retain-range-100000
  transient-total-200000
)

workload_expr() {
  case "$1" in
    baseline) printf '%s' 'Null' ;;
    module-leak-20000) printf '%s' 'Do[Module[{x}, x = i^2; x], {i, 1, 20000}]' ;;
    retain-range-1000) printf '%s' 'memGlobal = Range[1000]' ;;
    retain-range-10000) printf '%s' 'memGlobal = Range[10000]' ;;
    retain-range-100000) printf '%s' 'memGlobal = Range[100000]' ;;
    transient-total-200000) printf '%s' 'memTransient = Join[Range[100000], Range[100001, 200000]]; mxProbe = MaxMemoryUsed[]; Total[memTransient]; ClearAll[memTransient, mxProbe]' ;;
    *) echo "error: unknown workload label $1" >&2; exit 2 ;;
  esac
}

workload_cleanup() {
  case "$1" in
    baseline) printf '%s' 'Null' ;;
    module-leak-20000) printf '%s' 'Null' ;;
    retain-range-1000) printf '%s' 'ClearAll[memGlobal]' ;;
    retain-range-10000) printf '%s' 'ClearAll[memGlobal]' ;;
    retain-range-100000) printf '%s' 'ClearAll[memGlobal]' ;;
    transient-total-200000) printf '%s' 'ClearAll[memTransient, mxProbe]' ;;
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

parse_alloc_bytes() {
  local stderr_path=$1
  awk '
    /rum-profile alloc / {
      for (i = 1; i <= NF; i++) {
        if ($i ~ /^bytes_total=/) {
          sub(/^bytes_total=/, "", $i);
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
  local alloc_bytes=$4

  awk -v engine="${engine_name}" -v peak_rss="${peak_rss}" -v alloc_bytes="${alloc_bytes}" '
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
      postcleanup = trim(fields[6]);
      print engine "\t" trim(fields[2]) "\t" before "\t" after "\t" (after - before) "\t" max_memory "\t" postcleanup "\t" peak_rss "\t" alloc_bytes;
    }
  ' "${stdout_path}"
}

run_workload() {
  local engine_name=$1
  local label=$2
  local expr
  local cleanup
  local stdout_path="${raw_dir}/${engine_name}.${label}.stdout"
  local stderr_path="${raw_dir}/${engine_name}.${label}.stderr"
  local program parsed peak_rss alloc_bytes exit_code

  expr=$(workload_expr "${label}")
  cleanup=$(workload_cleanup "${label}")

  # The b0/b1/mx ownvalues add a tiny constant in both engines; compare deltas.
  program="b0 = MemoryInUse[]; ${expr}; b1 = MemoryInUse[]; mx = MaxMemoryUsed[]; ${cleanup}; bp = MemoryInUse[]; Print[\"RUM_MEM\t${label}\t\", b0, \"\t\", b1, \"\t\", mx, \"\t\", bp]"

  set +e
  if [[ "${engine_name}" == "rum" ]]; then
    /usr/bin/time -l "${rum_cli_bin}" --eval "${program}" >"${stdout_path}" 2>"${stderr_path}"
  else
    /usr/bin/time -l "${wolfram_bin}" -code "${program}" >"${stdout_path}" 2>"${stderr_path}"
  fi
  exit_code=$?
  set -e

  peak_rss=$(parse_peak_rss "${stderr_path}")
  alloc_bytes=
  if [[ "${engine_name}" == "rum" ]]; then
    alloc_bytes=$(parse_alloc_bytes "${stderr_path}")
  fi
  if [[ "${exit_code}" != "0" ]]; then
    echo "error: ${engine_name} ${label} exited with ${exit_code}; see ${stderr_path}" >&2
    exit "${exit_code}"
  fi

  parsed=$(parse_measurement "${engine_name}" "${stdout_path}" "${peak_rss}" "${alloc_bytes}")
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
awk -F '\t' -v label_order="${label_order}" -v aggregate_tsv="${aggregate_tsv}" '
  BEGIN {
    label_count = split(label_order, labels, " ");
  }
  NR == 1 { next }
  {
    label = $2;
    engine = $1;
    mem_before[label, engine] = $3;
    mem_after[label, engine] = $4;
    mem_delta[label, engine] = $5;
    max_memory[label, engine] = $6;
    mem_postcleanup[label, engine] = $7;
    peak_rss[label, engine] = $8;
    alloc_bytes[label, engine] = $9;
    if (label == "baseline" && $8 != "") {
      baseline_rss[engine] = $8;
    }
  }
  function ratio(num, den) {
    if (num != "" && den != "" && den + 0 != 0) {
      return sprintf("%.6f", (num + 0) / (den + 0));
    }
    return "";
  }
  function abs(value) {
    return value < 0 ? -value : value;
  }
  function rss_delta(label, engine, value) {
    value = peak_rss[label, engine];
    if (value == "" || baseline_rss[engine] == "") {
      return "";
    }
    return (value + 0) - (baseline_rss[engine] + 0);
  }
  function metric_peak(label, engine) {
    if (max_memory[label, engine] == "" || mem_before[label, engine] == "") {
      return "";
    }
    return (max_memory[label, engine] + 0) - (mem_before[label, engine] + 0);
  }
  function metric_transient(label, engine) {
    if (max_memory[label, engine] == "" || mem_after[label, engine] == "") {
      return "";
    }
    return (max_memory[label, engine] + 0) - (mem_after[label, engine] + 0);
  }
  function metric_post_cleanup(label, engine) {
    if (mem_postcleanup[label, engine] == "" || mem_before[label, engine] == "") {
      return "";
    }
    return (mem_postcleanup[label, engine] + 0) - (mem_before[label, engine] + 0);
  }
  function add_headline(metric, value, n) {
    if (value == "" || value + 0 <= 0) {
      return;
    }
    n = ++headline_count[metric];
    headline_values[metric, n] = value + 0;
    headline_log_sum[metric] += log(value + 0);
  }
  function emit_aggregate(metric, n, i, j, tmp, pidx, cmax, p95, geomean) {
    n = headline_count[metric] + 0;
    if (n == 0) {
      print "C_max\t" metric "\t\tnon_baseline_positive_ratios=0" >> aggregate_tsv;
      print "C_p95\t" metric "\t\tnon_baseline_positive_ratios=0" >> aggregate_tsv;
      print "C_geomean\t" metric "\t\tnon_baseline_positive_ratios=0" >> aggregate_tsv;
      return;
    }
    for (i = 1; i <= n; i++) {
      sorted_values[i] = headline_values[metric, i];
    }
    for (i = 1; i <= n; i++) {
      for (j = i + 1; j <= n; j++) {
        if (sorted_values[j] < sorted_values[i]) {
          tmp = sorted_values[i];
          sorted_values[i] = sorted_values[j];
          sorted_values[j] = tmp;
        }
      }
    }
    pidx = int(0.95 * n);
    while (pidx < 0.95 * n) {
      pidx++;
    }
    if (pidx < 1) {
      pidx = 1;
    }
    cmax = sorted_values[n];
    p95 = sorted_values[pidx];
    geomean = exp(headline_log_sum[metric] / n);
    print "C_max\t" metric "\t" sprintf("%.6f", cmax) "\tnon_baseline_positive_ratios=" n >> aggregate_tsv;
    print "C_p95\t" metric "\t" sprintf("%.6f", p95) "\tnon_baseline_positive_ratios=" n >> aggregate_tsv;
    print "C_geomean\t" metric "\t" sprintf("%.6f", geomean) "\tnon_baseline_positive_ratios=" n >> aggregate_tsv;
  }
  function observe_drift(label, retained_ratio, n_label, n_value) {
    if (label !~ /^retain-range-[0-9]+$/ || retained_ratio == "") {
      return;
    }
    n_label = label;
    sub(/^retain-range-/, "", n_label);
    n_value = n_label + 0;
    if (small_n == "" || n_value < small_n) {
      small_n = n_value;
      small_label = label;
      small_ratio = retained_ratio + 0;
    }
    if (big_n == "" || n_value > big_n) {
      big_n = n_value;
      big_label = label;
      big_ratio = retained_ratio + 0;
    }
  }
  function emit_drift(rel_change, verdict) {
    if (small_n == "" || big_n == "" || small_ratio == 0) {
      print "N_drift\tretained\t\tinsufficient retain-range ratios" >> aggregate_tsv;
      return;
    }
    rel_change = abs(big_ratio - small_ratio) / abs(small_ratio);
    verdict = rel_change > 0.15 ? "DRIFT (ratio not constant - suspect order gap)" : "STABLE";
    print "N_drift\tretained\t" verdict "\tsmall_label=" small_label " small_ratio=" sprintf("%.6f", small_ratio) " big_label=" big_label " big_ratio=" sprintf("%.6f", big_ratio) " relative_change=" sprintf("%.6f", rel_change) >> aggregate_tsv;
  }
  END {
    for (i = 1; i <= label_count; i++) {
      label = labels[i];
      if (label == "") {
        continue;
      }
      rum_retained = mem_delta[label, "rum"];
      wk_retained = mem_delta[label, "wolfram"];
      retained_ratio = ratio(rum_retained, wk_retained);
      rum_peak = metric_peak(label, "rum");
      wk_peak = metric_peak(label, "wolfram");
      peak_ratio = ratio(rum_peak, wk_peak);
      rum_transient = metric_transient(label, "rum");
      wk_transient = metric_transient(label, "wolfram");
      transient_ratio = ratio(rum_transient, wk_transient);
      rum_post_cleanup = metric_post_cleanup(label, "rum");
      wk_post_cleanup = metric_post_cleanup(label, "wolfram");
      post_cleanup_ratio = ratio(rum_post_cleanup, wk_post_cleanup);
      rum_peak_rss = peak_rss[label, "rum"];
      wk_peak_rss = peak_rss[label, "wolfram"];
      rum_rss = rss_delta(label, "rum");
      wk_rss = rss_delta(label, "wolfram");
      peak_rss_ratio = ratio(rum_peak_rss, wk_peak_rss);
      rss_delta_ratio = ratio(rum_rss, wk_rss);

      print label "\t" rum_retained "\t" wk_retained "\t" retained_ratio "\t" rum_peak "\t" wk_peak "\t" peak_ratio "\t" rum_transient "\t" wk_transient "\t" transient_ratio "\t" rum_post_cleanup "\t" wk_post_cleanup "\t" post_cleanup_ratio "\t" rum_peak_rss "\t" wk_peak_rss "\t" peak_rss_ratio "\t" rum_rss "\t" wk_rss "\t" rss_delta_ratio "\t" alloc_bytes[label, "rum"];

      if (label != "baseline") {
        add_headline("retained", retained_ratio);
        add_headline("peak", peak_ratio);
        observe_drift(label, retained_ratio);
      }
    }
    emit_aggregate("retained");
    emit_aggregate("peak");
    emit_drift();
  }
' "${measurements_tsv}" >> "${comparison_tsv}"

echo "core memory microbench output: ${output_dir}"
