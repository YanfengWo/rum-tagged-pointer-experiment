#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/core-language-microbench.sh [options]

Collect a bottom-up core-language microbenchmark baseline for rum. The suite
runs small core-language statements and writes machine-readable TSV files.

Options:
  --output-dir PATH    Output directory. Defaults to a /tmp directory.
  --timeout SECONDS    Run timeout. Defaults to 180.
  --build-timeout SEC  rum-cli build timeout when the binary is missing.
                       Defaults to 300.
  --scale N            Multiply benchmark iteration counts. Defaults to 1.
  --rum-cli PATH       Existing rum-cli binary. Defaults to target/release/rum-cli.
  -h, --help           Show this help.

Outputs:
  core-language-microbench.wls  Generated benchmark source.
  runs.tsv                      One row for the rum execution.
  measurements.tsv              Parsed rum timing rows.
  timings.tsv                   rum timing by label.
  raw/                          rum stdout/stderr logs.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
bounded_runner="${repo_root}/scripts/run-bounded.py"

output_dir=${RUM_CORE_MICRO_OUTPUT_DIR:-}
timeout_seconds=${RUM_CORE_MICRO_TIMEOUT:-180}
build_timeout_seconds=${RUM_CORE_MICRO_BUILD_TIMEOUT:-300}
scale=${RUM_CORE_MICRO_SCALE:-1}
reps=${RUM_CORE_MICRO_REPS:-7}
rum_cli_bin=${RUM_CLI_BIN:-"${repo_root}/target/release/rum-cli"}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      [[ $# -ge 2 ]] || { echo "error: --output-dir expects a path" >&2; exit 2; }
      output_dir=$2
      shift 2
      ;;
    --timeout)
      [[ $# -ge 2 ]] || { echo "error: --timeout expects seconds" >&2; exit 2; }
      timeout_seconds=$2
      shift 2
      ;;
    --build-timeout)
      [[ $# -ge 2 ]] || { echo "error: --build-timeout expects seconds" >&2; exit 2; }
      build_timeout_seconds=$2
      shift 2
      ;;
    --scale)
      [[ $# -ge 2 ]] || { echo "error: --scale expects a positive integer" >&2; exit 2; }
      scale=$2
      shift 2
      ;;
    --reps)
      [[ $# -ge 2 ]] || { echo "error: --reps expects a positive integer" >&2; exit 2; }
      reps=$2
      shift 2
      ;;
    --rum-cli)
      [[ $# -ge 2 ]] || { echo "error: --rum-cli expects a path" >&2; exit 2; }
      rum_cli_bin=$2
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

for number in "${timeout_seconds}" "${build_timeout_seconds}" "${scale}" "${reps}"; do
  if ! [[ "${number}" =~ ^[1-9][0-9]*$ ]]; then
    echo "error: numeric options expect positive integers" >&2
    exit 2
  fi
done

if [[ ! -x "${bounded_runner}" ]]; then
  echo "error: bounded runner is not executable: ${bounded_runner}" >&2
  exit 2
fi

if [[ ! -x "${rum_cli_bin}" ]]; then
  (
    cd "${repo_root}"
    "${bounded_runner}" --timeout "${build_timeout_seconds}" -- cargo build --release -p rum-cli >/dev/null
  )
fi
if [[ ! -x "${rum_cli_bin}" ]]; then
  echo "error: rum-cli binary is not executable: ${rum_cli_bin}" >&2
  exit 2
fi

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-core-microbench.XXXXXX)
else
  mkdir -p "${output_dir}"
fi
raw_dir="${output_dir}/raw"
mkdir -p "${raw_dir}"

bench_wls="${output_dir}/core-language-microbench.wls"
runs_tsv="${output_dir}/runs.tsv"
measurements_tsv="${output_dir}/measurements.tsv"
timings_tsv="${output_dir}/timings.tsv"

cat > "${bench_wls}" <<EOF
ClearAll[
  rumMicroBench,
  rumMicroOnce,
  rumMicroEmit,
  rumMicroN,
  rumMicroSink,
  fExact,
  fPattern,
  list10,
  rulesSimple
];

rumMicroScale = ${scale};
rumMicroReps = ${reps};
rumMicroN[n_Integer] := n * rumMicroScale;
rumMicroSink = Null;

SetAttributes[rumMicroBench, HoldAll];
rumMicroEmit[label_String, n_, seconds_] := Print[StringJoin[
  "RUM_CORE_MICRO\t",
  label,
  "\t",
  ToString[n],
  "\t",
  ToString[seconds]
]];

(* Min-of-reps: the timed loop runs the bare expression (NO per-iteration
   assignment — a per-iteration result-equals-expr injects one ownvalue Set
   plus cache invalidation per iteration, inflating rum time about 57 percent
   on cheap primitives. We time it
   rumMicroReps times and report the MINIMUM, which rejects OS scheduling
   spikes; sub-millisecond labels otherwise jitter 20-78 percent run-to-run.
   One untimed warm eval precedes the reps so the first timed rep is warm. *)
rumMicroBench[label_String, nExpr_, expr_] := Module[
  {start, i, k, n, seconds, best},
  n = nExpr;
  rumMicroSink = expr;
  best = -1;
  Do[
    start = AbsoluteTime[];
    Do[
      expr,
      {i, 1, n}
    ];
    seconds = AbsoluteTime[] - start;
    If[best < 0 || seconds < best, best = seconds],
    {k, 1, rumMicroReps}
  ];
  rumMicroEmit[label, n, best];
];

SetAttributes[rumMicroOnce, HoldAll];
rumMicroOnce[label_String, nExpr_, expr_] := Module[
  {start, k, n, result, seconds, best},
  n = nExpr;
  result = expr;
  rumMicroSink = result;
  best = -1;
  Do[
    start = AbsoluteTime[];
    result = expr;
    seconds = AbsoluteTime[] - start;
    If[best < 0 || seconds < best, best = seconds],
    {k, 1, rumMicroReps}
  ];
  rumMicroSink = result;
  rumMicroEmit[label, n, best];
];

xOwn = 123;
list10 = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
rulesSimple = {a -> 1, b -> 2, c -> 3};
fExact[1] = 2;
fPattern[x_Integer] := x + 1;

Print["RUM_CORE_MICRO_BEGIN"];

rumMicroBench["loop-empty", rumMicroN[5000], Null];
rumMicroOnce["do-count-null", rumMicroN[10000], Do[Null, {10000 * rumMicroScale}]];
rumMicroOnce["do-iterator-null", rumMicroN[1000], Do[Null, {j, 1000 * rumMicroScale}]];
rumMicroOnce["table-count-null", rumMicroN[5000], Table[Null, {5000 * rumMicroScale}]];
rumMicroOnce["table-iterator-null", rumMicroN[1000], Table[Null, {j, 1000 * rumMicroScale}]];
rumMicroOnce["table-iterator-symbol", rumMicroN[1000], Table[j, {j, 1000 * rumMicroScale}]];

rumMicroBench["literal-integer", rumMicroN[2000], 123];
rumMicroBench["literal-string", rumMicroN[2000], "abc"];
rumMicroBench["symbol-ownvalue-read", rumMicroN[2000], xOwn];
rumMicroBench["set-ownvalue", rumMicroN[1000], xOwn = 123];

rumMicroBench["list-literal-small", rumMicroN[1000], {1, 2, 3, 4}];
rumMicroBench["head-list", rumMicroN[2000], Head[list10]];
rumMicroBench["length-list", rumMicroN[2000], Length[list10]];
rumMicroBench["part-list", rumMicroN[2000], list10[[5]]];
rumMicroBench["sameq-integer", rumMicroN[2000], SameQ[1, 1]];
rumMicroBench["sameq-list", rumMicroN[1000], SameQ[list10, list10]];

rumMicroBench["if-true", rumMicroN[2000], If[True, 1, 2]];
rumMicroBench["module-local", rumMicroN[500], Module[{x = 1}, x + 1]];
rumMicroBench["block-local", rumMicroN[500], Block[{xOwn = 1}, xOwn + 1]];
rumMicroOnce["block-symbol-only-repeat", rumMicroN[1000], Do[Block[{b}, Null], {1000 * rumMicroScale}]];
rumMicroOnce["block-init-null-repeat", rumMicroN[1000], Do[Block[{b = 1}, Null], {1000 * rumMicroScale}]];
rumMicroOnce["block-init-read-repeat", rumMicroN[1000], Do[Block[{b = 1}, b], {1000 * rumMicroScale}]];
rumMicroBench["with-local", rumMicroN[500], With[{x = 1}, x + 1]];
rumMicroBench["table-small", rumMicroN[200], Table[i, {i, 1, 5}]];
rumMicroOnce["table-iterator-plus", rumMicroN[1000], Table[j + 1, {j, 1000 * rumMicroScale}]];
rumMicroOnce["table-iterator-pair-list", rumMicroN[1000], Table[{j, j}, {j, 1000 * rumMicroScale}]];
rumMicroBench["do-small", rumMicroN[200], Module[{s = 0}, Do[s = i, {i, 1, 5}]; s]];

rumMicroBench["downvalue-exact-hit", rumMicroN[2000], fExact[1]];
rumMicroBench["downvalue-exact-miss", rumMicroN[2000], fExact[2]];
rumMicroBench["downvalue-pattern-hit", rumMicroN[1000], fPattern[1]];
rumMicroBench["downvalue-pattern-miss", rumMicroN[1000], fPattern["x"]];
rumMicroOnce["downvalue-exact-miss-repeat", rumMicroN[5000], Do[fExact[2], {5000 * rumMicroScale}]];
rumMicroOnce["downvalue-pattern-miss-repeat", rumMicroN[5000], Do[fPattern["x"], {5000 * rumMicroScale}]];

rumMicroBench["matchq-integer-success", rumMicroN[1000], MatchQ[1, _Integer]];
rumMicroBench["matchq-integer-fail", rumMicroN[1000], MatchQ["x", _Integer]];
rumMicroBench["replaceall-simple", rumMicroN[500], a /. rulesSimple];

rumMicroBench["string-repeat-small", rumMicroN[500], StringRepeat["a", 3]];
rumMicroBench["character-range-small", rumMicroN[200], CharacterRange["a", "z"]];

Print["RUM_CORE_MICRO_END"];
EOF

printf 'engine\tstatus\texit_code\telapsed_seconds\tstdout\tstderr\n' > "${runs_tsv}"
printf 'engine\tlabel\titerations\tseconds\tseconds_raw\tstdout\n' > "${measurements_tsv}"
printf 'label\titerations\trum_seconds\n' > "${timings_tsv}"

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

run_engine() {
  local engine_name=rum
  local stdout_path="${raw_dir}/${engine_name}.stdout"
  local stderr_path="${raw_dir}/${engine_name}.stderr"
  local start elapsed exit_code status

  start=$(perl -MTime::HiRes=time -e 'printf "%.6f\n", time')
  set +e
  env "${disable_profile_env[@]}" \
    "${bounded_runner}" --timeout "${timeout_seconds}" -- \
    "${rum_cli_bin}" --file "${bench_wls}" \
    >"${stdout_path}" 2>"${stderr_path}"
  exit_code=$?
  set -e
  elapsed=$(perl -MTime::HiRes=time -e 'printf "%.6f\n", time - $ARGV[0]' "${start}")

  if [[ "${exit_code}" == "0" ]]; then
    status=ok
  else
    status=failed
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${engine_name}" "${status}" "${exit_code}" "${elapsed}" \
    "${stdout_path}" "${stderr_path}" >> "${runs_tsv}"

  awk -v engine="${engine_name}" -v stdout="${stdout_path}" '
    function trim(x) {
      gsub(/^[[:space:]]+/, "", x);
      gsub(/[[:space:]]+$/, "", x);
      return x;
    }
    function normalize(x, y, parts) {
      y = trim(x);
      gsub(/\*\^/, "e", y);
      if (y ~ /^-?[0-9]+\/[0-9]+$/) {
        split(y, parts, "/");
        return parts[1] / parts[2];
      }
      return y + 0;
    }
    {
      line = $0;
      gsub(/\r/, "", line);
      sub(/^"/, "", line);
      sub(/"$/, "", line);
      gsub(/\\t/, "\t", line);
      split(line, fields, "\t");
    }
    fields[1] == "RUM_CORE_MICRO" {
      print engine "\t" fields[2] "\t" fields[3] "\t" normalize(fields[4]) "\t" trim(fields[4]) "\t" stdout;
    }
  ' "${stdout_path}" >> "${measurements_tsv}"

  if [[ "${exit_code}" != "0" ]]; then
    echo "warning: ${engine_name} exited with ${exit_code}; see ${stderr_path}" >&2
  fi
}

run_engine

awk -F '\t' '
  NR == 1 { next }
  {
    print $2 "\t" $3 "\t" $4;
  }
' "${measurements_tsv}" >> "${timings_tsv}"

echo "core microbench output: ${output_dir}"
