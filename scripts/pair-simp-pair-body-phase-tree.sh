#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/pair-simp-pair-body-phase-tree.sh [--mma-hol PATH] [options]

Build and run a temporary instrumented copy of stdlib/Pair.wl. The canonical
mma-hol checkout is not edited; timing markers are inserted only into a copy in
the output directory.

Options:
  --mma-hol PATH       mma-hol checkout root. Required unless RUM_MMA_HOL_ROOT
                       is set.
  --detail LEVEL       top, mkPairInj, mkPairInjFirstConv,
                       mkPairInjFirstConvRule,
                       mkPairInjFirstSimpFixpoint, or
                       mkPairInjFirstSimpStep,
                       mkPairInjFirstDescendOnce, or
                       mkPairInjFirstDescendF, or
                       mkPairInjFirstDescendFX, or
                       mkPairInjFirstDescendFXDescend,
                       mkPairInjFirstPath,
                       mkPairInjFirstPathFXF, or
                       mkPairInjFirstPathX. Defaults to top.
  --path PATH          Comb path for --detail mkPairInjFirstPath, e.g. f/x/f.
                       Use b to split both comb children at that depth.
  --loop-split-label LABEL[,LABEL...]
                       Split only exact fixpoint loops whose diagnostic labels
                       match this comma-separated set.
  --loop-path-override LABEL=PATH[,LABEL=PATH...]
                       Override the remaining comb path for exact split loop
                       labels, e.g. label=a/a/b. Useful when a loop iteration
                       changes shape before the traced child is reached.
  --timeout SECONDS    Wall timeout. Defaults to 900.
  --output-dir PATH    Directory for generated files and logs.
  --engine ENGINE      rum or wolfram. Defaults to rum.
  --rum-cli PATH       Existing rum-cli binary. Defaults to target/release/rum-cli.
  --wolfram PATH       WolframKernel binary for --engine wolfram.
  -h, --help           Show this help.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
common_wls="${repo_root}/scripts/perf-probes/pair-simp/common.wls"
bounded_runner="${repo_root}/scripts/run-bounded.py"

mma_hol_root=${RUM_MMA_HOL_ROOT:-}
timeout_seconds=${RUM_PAIR_BODY_PHASE_TIMEOUT:-900}
output_dir=${RUM_PAIR_BODY_PHASE_OUTPUT_DIR:-}
engine=${RUM_PAIR_BODY_PHASE_ENGINE:-rum}
detail=${RUM_PAIR_BODY_PHASE_DETAIL:-top}
focus_path=${RUM_PAIR_BODY_PHASE_PATH:-}
loop_split_label=${RUM_PAIR_BODY_PHASE_LOOP_SPLIT_LABEL:-}
loop_path_override=${RUM_PAIR_BODY_PHASE_LOOP_PATH_OVERRIDE:-}
rum_cli_bin=${RUM_CLI_BIN:-"${repo_root}/target/release/rum-cli"}
wolfram_bin=${WOLFRAM_KERNEL:-/Applications/Wolfram.app/Contents/MacOS/WolframKernel}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mma-hol)
      [[ $# -ge 2 ]] || { echo "error: --mma-hol expects a path" >&2; exit 2; }
      mma_hol_root=$2
      shift 2
      ;;
    --timeout)
      [[ $# -ge 2 ]] || { echo "error: --timeout expects seconds" >&2; exit 2; }
      timeout_seconds=$2
      shift 2
      ;;
    --detail)
      [[ $# -ge 2 ]] || { echo "error: --detail expects top, mkPairInj, mkPairInjFirstConv, mkPairInjFirstConvRule, mkPairInjFirstSimpFixpoint, mkPairInjFirstSimpStep, mkPairInjFirstDescendOnce, mkPairInjFirstDescendF, mkPairInjFirstDescendFX, mkPairInjFirstDescendFXDescend, mkPairInjFirstPath, mkPairInjFirstPathFXF, or mkPairInjFirstPathX" >&2; exit 2; }
      detail=$2
      shift 2
      ;;
    --path)
      [[ $# -ge 2 ]] || { echo "error: --path expects a slash-separated path" >&2; exit 2; }
      focus_path=$2
      shift 2
      ;;
    --loop-split-label)
      [[ $# -ge 2 ]] || { echo "error: --loop-split-label expects a diagnostic label" >&2; exit 2; }
      loop_split_label=$2
      shift 2
      ;;
    --loop-path-override)
      [[ $# -ge 2 ]] || { echo "error: --loop-path-override expects LABEL=PATH[,LABEL=PATH...]" >&2; exit 2; }
      loop_path_override=$2
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

case "${detail}" in
  top|mkPairInj|mkPairInjFirstConv|mkPairInjFirstConvRule|mkPairInjFirstSimpFixpoint|mkPairInjFirstSimpStep|mkPairInjFirstDescendOnce|mkPairInjFirstDescendF|mkPairInjFirstDescendFX|mkPairInjFirstDescendFXDescend|mkPairInjFirstPath|mkPairInjFirstPathFXF|mkPairInjFirstPathX) ;;
  *) echo "error: --detail expects top, mkPairInj, mkPairInjFirstConv, mkPairInjFirstConvRule, mkPairInjFirstSimpFixpoint, mkPairInjFirstSimpStep, mkPairInjFirstDescendOnce, mkPairInjFirstDescendF, mkPairInjFirstDescendFX, mkPairInjFirstDescendFXDescend, mkPairInjFirstPath, mkPairInjFirstPathFXF, or mkPairInjFirstPathX" >&2; exit 2 ;;
esac

path_wl="{}"
path_label="custom"
if [[ "${detail}" == "mkPairInjFirstPath" ]]; then
  if [[ -z "${focus_path}" ]]; then
    echo "error: --detail mkPairInjFirstPath requires --path" >&2
    exit 2
  fi
  IFS='/' read -r -a path_parts <<< "${focus_path}"
  path_wl="{"
  path_label=""
  sep=""
  for part in "${path_parts[@]}"; do
    case "${part}" in
      f|x|b|a) ;;
      *) echo "error: --path entries must be f, x, b, or a: ${focus_path}" >&2; exit 2 ;;
    esac
    path_wl="${path_wl}${sep}\"${part}\""
    path_label="${path_label}${part}"
    sep=", "
  done
  path_wl="${path_wl}}"
fi

loop_path_override_wl="<||>"
if [[ -n "${loop_path_override}" ]]; then
  loop_path_override_wl="<|"
  assoc_sep=""
  IFS=',' read -r -a override_specs <<< "${loop_path_override}"
  for spec in "${override_specs[@]}"; do
    if [[ "${spec}" != *=* ]]; then
      echo "error: --loop-path-override entries must be LABEL=PATH: ${spec}" >&2
      exit 2
    fi
    override_label=${spec%%=*}
    override_path=${spec#*=}
    if [[ -z "${override_label}" || -z "${override_path}" ]]; then
      echo "error: --loop-path-override entries must have non-empty label and path: ${spec}" >&2
      exit 2
    fi
    IFS='/' read -r -a override_path_parts <<< "${override_path}"
    override_path_wl="{"
    path_sep=""
    for part in "${override_path_parts[@]}"; do
      case "${part}" in
        f|x|b|a) ;;
        *) echo "error: --loop-path-override path entries must be f, x, b, or a: ${override_path}" >&2; exit 2 ;;
      esac
      override_path_wl="${override_path_wl}${path_sep}\"${part}\""
      path_sep=", "
    done
    override_path_wl="${override_path_wl}}"
    loop_path_override_wl="${loop_path_override_wl}${assoc_sep}\"${override_label}\" -> ${override_path_wl}"
    assoc_sep=", "
  done
  loop_path_override_wl="${loop_path_override_wl}|>"
fi

if ! [[ "${timeout_seconds}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --timeout expects a positive integer" >&2
  exit 2
fi

if [[ -z "${mma_hol_root}" ]]; then
  echo "error: --mma-hol is required when RUM_MMA_HOL_ROOT is unset" >&2
  exit 2
fi

pair_src="${mma_hol_root}/stdlib/Pair.wl"
if [[ ! -f "${pair_src}" || ! -f "${mma_hol_root}/auto/Simp.wl" ]]; then
  echo "error: ${mma_hol_root} does not look like an mma-hol checkout" >&2
  exit 2
fi

if [[ ! -f "${common_wls}" ]]; then
  echo "error: missing common probe helper: ${common_wls}" >&2
  exit 2
fi

if [[ ! -x "${bounded_runner}" ]]; then
  echo "error: bounded runner is not executable: ${bounded_runner}" >&2
  exit 2
fi

if [[ "${engine}" == "rum" ]]; then
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
  output_dir=$(mktemp -d /tmp/rum-pair-body-phase.XXXXXX)
else
  mkdir -p "${output_dir}"
fi

instrumented_pair="${output_dir}/Pair.instrumented.wl"
driver_wls="${output_dir}/pair-body-phase-tree.wls"
stdout_file="${output_dir}/pair-body-phase-tree.${engine}.stdout"
stderr_file="${output_dir}/pair-body-phase-tree.${engine}.stderr"
summary_file="${output_dir}/summary.tsv"

awk -v detail="${detail}" \
    -v focus_path_wl="${path_wl}" \
    -v focus_path_label="${path_label}" '
function mark(label) {
  print "Global`rumPairProbeStageNext[\"" label "\"];"
}
{
  if ($0 ~ /^mkPairDefThm =/) mark("mkPairDefThm")
  if ($0 ~ /^isPairPredicateTerm\[\] :=/) mark("mkPair-helper-definitions")
  if ($0 ~ /^isPairWitnessThm =/) mark("isPairWitnessThm")
  if ($0 ~ /^\{absRepProdThm, repAbsProdThm\} =/) mark("newBasicTypeDefinition")
  if ($0 ~ /^pairConsDefThm =/) mark("pairConsDefThm")
  if ($0 ~ /^repPairThm =/) mark("repPairThm")
  if ($0 ~ /^mkPairInjThm =/) {
    mark("mkPairInjThm")
    inMkPairInj = 1
  }
  if ((detail == "mkPairInjFirstPath" ||
       detail == "mkPairInjFirstPathFXF" ||
       detail == "mkPairInjFirstPathX") && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    path = detail == "mkPairInjFirstPathFXF" ? "{\"f\", \"x\", \"f\"}" : "{\"x\"}";
    label = detail == "mkPairInjFirstPathFXF" ? "mkPairInj-first-path-fxf" : "mkPairInj-first-path-x";
    if (detail == "mkPairInjFirstPath") {
      path = focus_path_wl;
      label = "mkPairInj-first-path-" focus_path_label;
    }
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-target-concl")
    print "    rumPairProbeMkPairFirstTarget = HOL`Kernel`concl[lhsAppXY];"
    mark("mkPairInj-first-simp-withBasic")
    print "    rumPairProbeFirstRuleThms = HOL`Auto`Simp`Private`withBasic[{mkPairDefThm}];"
    mark(label)
    print "    rumPairProbeFirstCur = Global`rumPairProbeSplitSimpFixpoint[\"" label "\", rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit, rumPairProbeMkPairFirstTarget, " path "];"
    mark("mkPairInj-first-simp-EQMP")
    print "    simplifiedEq = HOL`Kernel`EQMP[rumPairProbeFirstCur, lhsAppXY];"
    getline
    next
  }
  if (detail == "mkPairInjFirstDescendFXDescend" && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-target-concl")
    print "    rumPairProbeMkPairFirstTarget = HOL`Kernel`concl[lhsAppXY];"
    mark("mkPairInj-first-simp-withBasic")
    print "    rumPairProbeFirstRuleThms = HOL`Auto`Simp`Private`withBasic[{mkPairDefThm}];"
    mark("mkPairInj-first-simp-buildBaseConv")
    print "    rumPairProbeFirstBaseConv = HOL`Auto`Simp`Private`buildBaseConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-simp-build-stepConv")
    print "    rumPairProbeFirstStepConv = HOL`Auto`Simp`Private`simpStepConv[rumPairProbeFirstBaseConv, rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-step-top-repeatc")
    print "    rumPairProbeFirstTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeMkPairFirstTarget];"
    mark("mkPairInj-first-step-rhs1")
    print "    rumPairProbeFirstRhs1 = HOL`Kernel`concl[rumPairProbeFirstTopTh][[2]];"
    print "    rumPairProbeFirstF = rumPairProbeFirstRhs1[[1]];"
    print "    rumPairProbeFirstX = rumPairProbeFirstRhs1[[2]];"
    mark("mkPairInj-first-descend-f-top-repeatc")
    print "    rumPairProbeFirstFTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeFirstF];"
    mark("mkPairInj-first-descend-f-rhs1")
    print "    rumPairProbeFirstFRhs1 = HOL`Kernel`concl[rumPairProbeFirstFTopTh][[2]];"
    print "    rumPairProbeFirstFF = rumPairProbeFirstFRhs1[[1]];"
    print "    rumPairProbeFirstFX = rumPairProbeFirstFRhs1[[2]];"
    mark("mkPairInj-first-descend-f-comb-f")
    print "    rumPairProbeFirstFFEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstFF];"
    mark("mkPairInj-first-descend-fx-top-repeatc")
    print "    rumPairProbeFirstFXTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeFirstFX];"
    mark("mkPairInj-first-descend-fx-rhs1")
    print "    rumPairProbeFirstFXRhs1 = HOL`Kernel`concl[rumPairProbeFirstFXTopTh][[2]];"
    print "    rumPairProbeFirstFXF = rumPairProbeFirstFXRhs1[[1]];"
    print "    rumPairProbeFirstFXX = rumPairProbeFirstFXRhs1[[2]];"
    mark("mkPairInj-first-descend-fx-comb-f")
    print "    rumPairProbeFirstFXFEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstFXF];"
    mark("mkPairInj-first-descend-fx-comb-x")
    print "    rumPairProbeFirstFXXEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstFXX];"
    mark("mkPairInj-first-descend-fx-comb-MKCOMB")
    print "    rumPairProbeFirstFXDescTh = HOL`Kernel`MKCOMB[rumPairProbeFirstFXFEq, rumPairProbeFirstFXXEq];"
    mark("mkPairInj-first-descend-fx-TRANS")
    print "    rumPairProbeFirstFXEq = HOL`Kernel`TRANS[rumPairProbeFirstFXTopTh, rumPairProbeFirstFXDescTh];"
    mark("mkPairInj-first-descend-fx-init-seen")
    print "    rumPairProbeFirstFXRhs = HOL`Kernel`concl[rumPairProbeFirstFXEq][[2]];"
    print "    rumPairProbeFirstFXSeen = <|rumPairProbeFirstFX -> True, rumPairProbeFirstFXRhs -> True|>;"
    print "    rumPairProbeFirstFXLoopIterations = 0;"
    mark("mkPairInj-first-descend-fx-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstFXLoopIterations++;"
    print "      rumPairProbeFirstFXNext = rumPairProbeFirstStepConv[rumPairProbeFirstFXRhs];"
    print "      rumPairProbeFirstFXNewRhs = HOL`Kernel`concl[rumPairProbeFirstFXNext][[2]];"
    print "      If[rumPairProbeFirstFXNewRhs === rumPairProbeFirstFXRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstFXSeen, rumPairProbeFirstFXNewRhs], Break[]];"
    print "      rumPairProbeFirstFXEq = HOL`Kernel`TRANS[rumPairProbeFirstFXEq, rumPairProbeFirstFXNext];"
    print "      rumPairProbeFirstFXRhs = rumPairProbeFirstFXNewRhs;"
    print "      rumPairProbeFirstFXSeen[rumPairProbeFirstFXNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-fx-loop-iterations=\", rumPairProbeFirstFXLoopIterations];"
    mark("mkPairInj-first-descend-f-comb-MKCOMB")
    print "    rumPairProbeFirstFDescTh = HOL`Kernel`MKCOMB[rumPairProbeFirstFFEq, rumPairProbeFirstFXEq];"
    mark("mkPairInj-first-descend-f-TRANS")
    print "    rumPairProbeFirstFEq = HOL`Kernel`TRANS[rumPairProbeFirstFTopTh, rumPairProbeFirstFDescTh];"
    mark("mkPairInj-first-descend-f-init-seen")
    print "    rumPairProbeFirstFRhs = HOL`Kernel`concl[rumPairProbeFirstFEq][[2]];"
    print "    rumPairProbeFirstFSeen = <|rumPairProbeFirstF -> True, rumPairProbeFirstFRhs -> True|>;"
    print "    rumPairProbeFirstFLoopIterations = 0;"
    mark("mkPairInj-first-descend-f-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstFLoopIterations++;"
    print "      rumPairProbeFirstFNext = rumPairProbeFirstStepConv[rumPairProbeFirstFRhs];"
    print "      rumPairProbeFirstFNewRhs = HOL`Kernel`concl[rumPairProbeFirstFNext][[2]];"
    print "      If[rumPairProbeFirstFNewRhs === rumPairProbeFirstFRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstFSeen, rumPairProbeFirstFNewRhs], Break[]];"
    print "      rumPairProbeFirstFEq = HOL`Kernel`TRANS[rumPairProbeFirstFEq, rumPairProbeFirstFNext];"
    print "      rumPairProbeFirstFRhs = rumPairProbeFirstFNewRhs;"
    print "      rumPairProbeFirstFSeen[rumPairProbeFirstFNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-f-loop-iterations=\", rumPairProbeFirstFLoopIterations];"
    mark("mkPairInj-first-descend-comb-x")
    print "    rumPairProbeFirstXEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstX];"
    mark("mkPairInj-first-descend-comb-MKCOMB")
    print "    rumPairProbeFirstDescTh = HOL`Kernel`MKCOMB[rumPairProbeFirstFEq, rumPairProbeFirstXEq];"
    mark("mkPairInj-first-step-TRANS")
    print "    rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstTopTh, rumPairProbeFirstDescTh];"
    mark("mkPairInj-first-simp-init-seen")
    print "    rumPairProbeFirstRhs = HOL`Kernel`concl[rumPairProbeFirstCur][[2]];"
    print "    rumPairProbeFirstSeen = <|rumPairProbeMkPairFirstTarget -> True, rumPairProbeFirstRhs -> True|>;"
    print "    rumPairProbeFirstLoopIterations = 0;"
    mark("mkPairInj-first-simp-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstLoopIterations++;"
    print "      rumPairProbeFirstNext = rumPairProbeFirstStepConv[rumPairProbeFirstRhs];"
    print "      rumPairProbeFirstNewRhs = HOL`Kernel`concl[rumPairProbeFirstNext][[2]];"
    print "      If[rumPairProbeFirstNewRhs === rumPairProbeFirstRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstSeen, rumPairProbeFirstNewRhs], Break[]];"
    print "      rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstCur, rumPairProbeFirstNext];"
    print "      rumPairProbeFirstRhs = rumPairProbeFirstNewRhs;"
    print "      rumPairProbeFirstSeen[rumPairProbeFirstNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-simp-loop-iterations=\", rumPairProbeFirstLoopIterations];"
    mark("mkPairInj-first-simp-EQMP")
    print "    simplifiedEq = HOL`Kernel`EQMP[rumPairProbeFirstCur, lhsAppXY];"
    getline
    next
  }
  if (detail == "mkPairInjFirstDescendFX" && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-target-concl")
    print "    rumPairProbeMkPairFirstTarget = HOL`Kernel`concl[lhsAppXY];"
    mark("mkPairInj-first-simp-withBasic")
    print "    rumPairProbeFirstRuleThms = HOL`Auto`Simp`Private`withBasic[{mkPairDefThm}];"
    mark("mkPairInj-first-simp-buildBaseConv")
    print "    rumPairProbeFirstBaseConv = HOL`Auto`Simp`Private`buildBaseConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-simp-build-stepConv")
    print "    rumPairProbeFirstStepConv = HOL`Auto`Simp`Private`simpStepConv[rumPairProbeFirstBaseConv, rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-step-top-repeatc")
    print "    rumPairProbeFirstTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeMkPairFirstTarget];"
    mark("mkPairInj-first-step-rhs1")
    print "    rumPairProbeFirstRhs1 = HOL`Kernel`concl[rumPairProbeFirstTopTh][[2]];"
    print "    rumPairProbeFirstF = rumPairProbeFirstRhs1[[1]];"
    print "    rumPairProbeFirstX = rumPairProbeFirstRhs1[[2]];"
    mark("mkPairInj-first-descend-f-top-repeatc")
    print "    rumPairProbeFirstFTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeFirstF];"
    mark("mkPairInj-first-descend-f-rhs1")
    print "    rumPairProbeFirstFRhs1 = HOL`Kernel`concl[rumPairProbeFirstFTopTh][[2]];"
    print "    rumPairProbeFirstFF = rumPairProbeFirstFRhs1[[1]];"
    print "    rumPairProbeFirstFX = rumPairProbeFirstFRhs1[[2]];"
    mark("mkPairInj-first-descend-f-comb-f")
    print "    rumPairProbeFirstFFEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstFF];"
    mark("mkPairInj-first-descend-fx-top-repeatc")
    print "    rumPairProbeFirstFXTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeFirstFX];"
    mark("mkPairInj-first-descend-fx-rhs1")
    print "    rumPairProbeFirstFXRhs1 = HOL`Kernel`concl[rumPairProbeFirstFXTopTh][[2]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-fx-isImp=\", HOL`Auto`Simp`Private`isImp[rumPairProbeFirstFXRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-fx-isAnd=\", HOL`Auto`Simp`Private`isAndForm[rumPairProbeFirstFXRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-fx-isOr=\", HOL`Auto`Simp`Private`isOrForm[rumPairProbeFirstFXRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-fx-isComb=\", MatchQ[rumPairProbeFirstFXRhs1, HOL`Terms`comb[_, _]]];"
    mark("mkPairInj-first-descend-fx-descendOnce")
    print "    rumPairProbeFirstFXDescTh = HOL`Auto`Simp`Private`descendOnce[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstFXRhs1];"
    mark("mkPairInj-first-descend-fx-TRANS")
    print "    rumPairProbeFirstFXEq = HOL`Kernel`TRANS[rumPairProbeFirstFXTopTh, rumPairProbeFirstFXDescTh];"
    mark("mkPairInj-first-descend-fx-init-seen")
    print "    rumPairProbeFirstFXRhs = HOL`Kernel`concl[rumPairProbeFirstFXEq][[2]];"
    print "    rumPairProbeFirstFXSeen = <|rumPairProbeFirstFX -> True, rumPairProbeFirstFXRhs -> True|>;"
    print "    rumPairProbeFirstFXLoopIterations = 0;"
    mark("mkPairInj-first-descend-fx-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstFXLoopIterations++;"
    print "      rumPairProbeFirstFXNext = rumPairProbeFirstStepConv[rumPairProbeFirstFXRhs];"
    print "      rumPairProbeFirstFXNewRhs = HOL`Kernel`concl[rumPairProbeFirstFXNext][[2]];"
    print "      If[rumPairProbeFirstFXNewRhs === rumPairProbeFirstFXRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstFXSeen, rumPairProbeFirstFXNewRhs], Break[]];"
    print "      rumPairProbeFirstFXEq = HOL`Kernel`TRANS[rumPairProbeFirstFXEq, rumPairProbeFirstFXNext];"
    print "      rumPairProbeFirstFXRhs = rumPairProbeFirstFXNewRhs;"
    print "      rumPairProbeFirstFXSeen[rumPairProbeFirstFXNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-fx-loop-iterations=\", rumPairProbeFirstFXLoopIterations];"
    mark("mkPairInj-first-descend-f-comb-MKCOMB")
    print "    rumPairProbeFirstFDescTh = HOL`Kernel`MKCOMB[rumPairProbeFirstFFEq, rumPairProbeFirstFXEq];"
    mark("mkPairInj-first-descend-f-TRANS")
    print "    rumPairProbeFirstFEq = HOL`Kernel`TRANS[rumPairProbeFirstFTopTh, rumPairProbeFirstFDescTh];"
    mark("mkPairInj-first-descend-f-init-seen")
    print "    rumPairProbeFirstFRhs = HOL`Kernel`concl[rumPairProbeFirstFEq][[2]];"
    print "    rumPairProbeFirstFSeen = <|rumPairProbeFirstF -> True, rumPairProbeFirstFRhs -> True|>;"
    print "    rumPairProbeFirstFLoopIterations = 0;"
    mark("mkPairInj-first-descend-f-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstFLoopIterations++;"
    print "      rumPairProbeFirstFNext = rumPairProbeFirstStepConv[rumPairProbeFirstFRhs];"
    print "      rumPairProbeFirstFNewRhs = HOL`Kernel`concl[rumPairProbeFirstFNext][[2]];"
    print "      If[rumPairProbeFirstFNewRhs === rumPairProbeFirstFRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstFSeen, rumPairProbeFirstFNewRhs], Break[]];"
    print "      rumPairProbeFirstFEq = HOL`Kernel`TRANS[rumPairProbeFirstFEq, rumPairProbeFirstFNext];"
    print "      rumPairProbeFirstFRhs = rumPairProbeFirstFNewRhs;"
    print "      rumPairProbeFirstFSeen[rumPairProbeFirstFNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-f-loop-iterations=\", rumPairProbeFirstFLoopIterations];"
    mark("mkPairInj-first-descend-comb-x")
    print "    rumPairProbeFirstXEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstX];"
    mark("mkPairInj-first-descend-comb-MKCOMB")
    print "    rumPairProbeFirstDescTh = HOL`Kernel`MKCOMB[rumPairProbeFirstFEq, rumPairProbeFirstXEq];"
    mark("mkPairInj-first-step-TRANS")
    print "    rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstTopTh, rumPairProbeFirstDescTh];"
    mark("mkPairInj-first-simp-init-seen")
    print "    rumPairProbeFirstRhs = HOL`Kernel`concl[rumPairProbeFirstCur][[2]];"
    print "    rumPairProbeFirstSeen = <|rumPairProbeMkPairFirstTarget -> True, rumPairProbeFirstRhs -> True|>;"
    print "    rumPairProbeFirstLoopIterations = 0;"
    mark("mkPairInj-first-simp-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstLoopIterations++;"
    print "      rumPairProbeFirstNext = rumPairProbeFirstStepConv[rumPairProbeFirstRhs];"
    print "      rumPairProbeFirstNewRhs = HOL`Kernel`concl[rumPairProbeFirstNext][[2]];"
    print "      If[rumPairProbeFirstNewRhs === rumPairProbeFirstRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstSeen, rumPairProbeFirstNewRhs], Break[]];"
    print "      rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstCur, rumPairProbeFirstNext];"
    print "      rumPairProbeFirstRhs = rumPairProbeFirstNewRhs;"
    print "      rumPairProbeFirstSeen[rumPairProbeFirstNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-simp-loop-iterations=\", rumPairProbeFirstLoopIterations];"
    mark("mkPairInj-first-simp-EQMP")
    print "    simplifiedEq = HOL`Kernel`EQMP[rumPairProbeFirstCur, lhsAppXY];"
    getline
    next
  }
  if (detail == "mkPairInjFirstDescendF" && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-target-concl")
    print "    rumPairProbeMkPairFirstTarget = HOL`Kernel`concl[lhsAppXY];"
    mark("mkPairInj-first-simp-withBasic")
    print "    rumPairProbeFirstRuleThms = HOL`Auto`Simp`Private`withBasic[{mkPairDefThm}];"
    mark("mkPairInj-first-simp-buildBaseConv")
    print "    rumPairProbeFirstBaseConv = HOL`Auto`Simp`Private`buildBaseConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-simp-build-stepConv")
    print "    rumPairProbeFirstStepConv = HOL`Auto`Simp`Private`simpStepConv[rumPairProbeFirstBaseConv, rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-step-top-repeatc")
    print "    rumPairProbeFirstTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeMkPairFirstTarget];"
    mark("mkPairInj-first-step-rhs1")
    print "    rumPairProbeFirstRhs1 = HOL`Kernel`concl[rumPairProbeFirstTopTh][[2]];"
    print "    rumPairProbeFirstF = rumPairProbeFirstRhs1[[1]];"
    print "    rumPairProbeFirstX = rumPairProbeFirstRhs1[[2]];"
    mark("mkPairInj-first-descend-f-top-repeatc")
    print "    rumPairProbeFirstFTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeFirstF];"
    mark("mkPairInj-first-descend-f-rhs1")
    print "    rumPairProbeFirstFRhs1 = HOL`Kernel`concl[rumPairProbeFirstFTopTh][[2]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-f-isImp=\", HOL`Auto`Simp`Private`isImp[rumPairProbeFirstFRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-f-isAnd=\", HOL`Auto`Simp`Private`isAndForm[rumPairProbeFirstFRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-f-isOr=\", HOL`Auto`Simp`Private`isOrForm[rumPairProbeFirstFRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-f-isComb=\", MatchQ[rumPairProbeFirstFRhs1, HOL`Terms`comb[_, _]]];"
    mark("mkPairInj-first-descend-f-comb-f")
    print "    If[MatchQ[rumPairProbeFirstFRhs1, HOL`Terms`comb[_, _]],"
    print "      rumPairProbeFirstFF = rumPairProbeFirstFRhs1[[1]];"
    print "      rumPairProbeFirstFX = rumPairProbeFirstFRhs1[[2]];"
    print "      rumPairProbeFirstFFEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstFF],"
    print "      rumPairProbeFirstFFEq = Missing[\"not-comb\"]"
    print "    ];"
    mark("mkPairInj-first-descend-f-comb-x")
    print "    If[MatchQ[rumPairProbeFirstFRhs1, HOL`Terms`comb[_, _]],"
    print "      rumPairProbeFirstFXEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstFX],"
    print "      rumPairProbeFirstFXEq = Missing[\"not-comb\"]"
    print "    ];"
    mark("mkPairInj-first-descend-f-comb-MKCOMB")
    print "    If[MatchQ[rumPairProbeFirstFRhs1, HOL`Terms`comb[_, _]],"
    print "      rumPairProbeFirstFDescTh = HOL`Kernel`MKCOMB[rumPairProbeFirstFFEq, rumPairProbeFirstFXEq],"
    print "      rumPairProbeFirstFDescTh = HOL`Auto`Simp`Private`descendOnce[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstFRhs1]"
    print "    ];"
    mark("mkPairInj-first-descend-f-TRANS")
    print "    rumPairProbeFirstFEq = HOL`Kernel`TRANS[rumPairProbeFirstFTopTh, rumPairProbeFirstFDescTh];"
    mark("mkPairInj-first-descend-f-init-seen")
    print "    rumPairProbeFirstFRhs = HOL`Kernel`concl[rumPairProbeFirstFEq][[2]];"
    print "    rumPairProbeFirstFSeen = <|rumPairProbeFirstF -> True, rumPairProbeFirstFRhs -> True|>;"
    print "    rumPairProbeFirstFLoopIterations = 0;"
    mark("mkPairInj-first-descend-f-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstFLoopIterations++;"
    print "      rumPairProbeFirstFNext = rumPairProbeFirstStepConv[rumPairProbeFirstFRhs];"
    print "      rumPairProbeFirstFNewRhs = HOL`Kernel`concl[rumPairProbeFirstFNext][[2]];"
    print "      If[rumPairProbeFirstFNewRhs === rumPairProbeFirstFRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstFSeen, rumPairProbeFirstFNewRhs], Break[]];"
    print "      rumPairProbeFirstFEq = HOL`Kernel`TRANS[rumPairProbeFirstFEq, rumPairProbeFirstFNext];"
    print "      rumPairProbeFirstFRhs = rumPairProbeFirstFNewRhs;"
    print "      rumPairProbeFirstFSeen[rumPairProbeFirstFNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-f-loop-iterations=\", rumPairProbeFirstFLoopIterations];"
    mark("mkPairInj-first-descend-comb-x")
    print "    rumPairProbeFirstXEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstX];"
    mark("mkPairInj-first-descend-comb-MKCOMB")
    print "    rumPairProbeFirstDescTh = HOL`Kernel`MKCOMB[rumPairProbeFirstFEq, rumPairProbeFirstXEq];"
    mark("mkPairInj-first-step-TRANS")
    print "    rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstTopTh, rumPairProbeFirstDescTh];"
    mark("mkPairInj-first-simp-init-seen")
    print "    rumPairProbeFirstRhs = HOL`Kernel`concl[rumPairProbeFirstCur][[2]];"
    print "    rumPairProbeFirstSeen = <|rumPairProbeMkPairFirstTarget -> True, rumPairProbeFirstRhs -> True|>;"
    print "    rumPairProbeFirstLoopIterations = 0;"
    mark("mkPairInj-first-simp-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstLoopIterations++;"
    print "      rumPairProbeFirstNext = rumPairProbeFirstStepConv[rumPairProbeFirstRhs];"
    print "      rumPairProbeFirstNewRhs = HOL`Kernel`concl[rumPairProbeFirstNext][[2]];"
    print "      If[rumPairProbeFirstNewRhs === rumPairProbeFirstRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstSeen, rumPairProbeFirstNewRhs], Break[]];"
    print "      rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstCur, rumPairProbeFirstNext];"
    print "      rumPairProbeFirstRhs = rumPairProbeFirstNewRhs;"
    print "      rumPairProbeFirstSeen[rumPairProbeFirstNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-simp-loop-iterations=\", rumPairProbeFirstLoopIterations];"
    mark("mkPairInj-first-simp-EQMP")
    print "    simplifiedEq = HOL`Kernel`EQMP[rumPairProbeFirstCur, lhsAppXY];"
    getline
    next
  }
  if (detail == "mkPairInjFirstDescendOnce" && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-target-concl")
    print "    rumPairProbeMkPairFirstTarget = HOL`Kernel`concl[lhsAppXY];"
    mark("mkPairInj-first-simp-withBasic")
    print "    rumPairProbeFirstRuleThms = HOL`Auto`Simp`Private`withBasic[{mkPairDefThm}];"
    mark("mkPairInj-first-simp-buildBaseConv")
    print "    rumPairProbeFirstBaseConv = HOL`Auto`Simp`Private`buildBaseConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-simp-build-stepConv")
    print "    rumPairProbeFirstStepConv = HOL`Auto`Simp`Private`simpStepConv[rumPairProbeFirstBaseConv, rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-step-top-repeatc")
    print "    rumPairProbeFirstTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeMkPairFirstTarget];"
    mark("mkPairInj-first-step-rhs1")
    print "    rumPairProbeFirstRhs1 = HOL`Kernel`concl[rumPairProbeFirstTopTh][[2]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-isImp=\", HOL`Auto`Simp`Private`isImp[rumPairProbeFirstRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-isAnd=\", HOL`Auto`Simp`Private`isAndForm[rumPairProbeFirstRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-isOr=\", HOL`Auto`Simp`Private`isOrForm[rumPairProbeFirstRhs1]];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-descend-isComb=\", MatchQ[rumPairProbeFirstRhs1, HOL`Terms`comb[_, _]]];"
    mark("mkPairInj-first-descend-comb-f")
    print "    If[MatchQ[rumPairProbeFirstRhs1, HOL`Terms`comb[_, _]],"
    print "      rumPairProbeFirstF = rumPairProbeFirstRhs1[[1]];"
    print "      rumPairProbeFirstX = rumPairProbeFirstRhs1[[2]];"
    print "      rumPairProbeFirstFEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstF],"
    print "      rumPairProbeFirstFEq = Missing[\"not-comb\"]"
    print "    ];"
    mark("mkPairInj-first-descend-comb-x")
    print "    If[MatchQ[rumPairProbeFirstRhs1, HOL`Terms`comb[_, _]],"
    print "      rumPairProbeFirstXEq = HOL`Auto`Simp`Private`simpFixpointConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstX],"
    print "      rumPairProbeFirstXEq = Missing[\"not-comb\"]"
    print "    ];"
    mark("mkPairInj-first-descend-comb-MKCOMB")
    print "    If[MatchQ[rumPairProbeFirstRhs1, HOL`Terms`comb[_, _]],"
    print "      rumPairProbeFirstDescTh = HOL`Kernel`MKCOMB[rumPairProbeFirstFEq, rumPairProbeFirstXEq],"
    print "      rumPairProbeFirstDescTh = HOL`Auto`Simp`Private`descendOnce[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstRhs1]"
    print "    ];"
    mark("mkPairInj-first-step-TRANS")
    print "    rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstTopTh, rumPairProbeFirstDescTh];"
    mark("mkPairInj-first-simp-init-seen")
    print "    rumPairProbeFirstRhs = HOL`Kernel`concl[rumPairProbeFirstCur][[2]];"
    print "    rumPairProbeFirstSeen = <|rumPairProbeMkPairFirstTarget -> True, rumPairProbeFirstRhs -> True|>;"
    print "    rumPairProbeFirstLoopIterations = 0;"
    mark("mkPairInj-first-simp-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstLoopIterations++;"
    print "      rumPairProbeFirstNext = rumPairProbeFirstStepConv[rumPairProbeFirstRhs];"
    print "      rumPairProbeFirstNewRhs = HOL`Kernel`concl[rumPairProbeFirstNext][[2]];"
    print "      If[rumPairProbeFirstNewRhs === rumPairProbeFirstRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstSeen, rumPairProbeFirstNewRhs], Break[]];"
    print "      rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstCur, rumPairProbeFirstNext];"
    print "      rumPairProbeFirstRhs = rumPairProbeFirstNewRhs;"
    print "      rumPairProbeFirstSeen[rumPairProbeFirstNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-simp-loop-iterations=\", rumPairProbeFirstLoopIterations];"
    mark("mkPairInj-first-simp-EQMP")
    print "    simplifiedEq = HOL`Kernel`EQMP[rumPairProbeFirstCur, lhsAppXY];"
    getline
    next
  }
  if (detail == "mkPairInjFirstSimpStep" && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-target-concl")
    print "    rumPairProbeMkPairFirstTarget = HOL`Kernel`concl[lhsAppXY];"
    mark("mkPairInj-first-simp-withBasic")
    print "    rumPairProbeFirstRuleThms = HOL`Auto`Simp`Private`withBasic[{mkPairDefThm}];"
    mark("mkPairInj-first-simp-buildBaseConv")
    print "    rumPairProbeFirstBaseConv = HOL`Auto`Simp`Private`buildBaseConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-simp-build-stepConv")
    print "    rumPairProbeFirstStepConv = HOL`Auto`Simp`Private`simpStepConv[rumPairProbeFirstBaseConv, rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-step-top-repeatc")
    print "    rumPairProbeFirstTopTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[rumPairProbeFirstBaseConv]][rumPairProbeMkPairFirstTarget];"
    mark("mkPairInj-first-step-rhs1")
    print "    rumPairProbeFirstRhs1 = HOL`Kernel`concl[rumPairProbeFirstTopTh][[2]];"
    mark("mkPairInj-first-step-descendOnce")
    print "    rumPairProbeFirstDescTh = HOL`Auto`Simp`Private`descendOnce[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeFirstRhs1];"
    mark("mkPairInj-first-step-TRANS")
    print "    rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstTopTh, rumPairProbeFirstDescTh];"
    mark("mkPairInj-first-simp-init-seen")
    print "    rumPairProbeFirstRhs = HOL`Kernel`concl[rumPairProbeFirstCur][[2]];"
    print "    rumPairProbeFirstSeen = <|rumPairProbeMkPairFirstTarget -> True, rumPairProbeFirstRhs -> True|>;"
    print "    rumPairProbeFirstLoopIterations = 0;"
    mark("mkPairInj-first-simp-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstLoopIterations++;"
    print "      rumPairProbeFirstNext = rumPairProbeFirstStepConv[rumPairProbeFirstRhs];"
    print "      rumPairProbeFirstNewRhs = HOL`Kernel`concl[rumPairProbeFirstNext][[2]];"
    print "      If[rumPairProbeFirstNewRhs === rumPairProbeFirstRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstSeen, rumPairProbeFirstNewRhs], Break[]];"
    print "      rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstCur, rumPairProbeFirstNext];"
    print "      rumPairProbeFirstRhs = rumPairProbeFirstNewRhs;"
    print "      rumPairProbeFirstSeen[rumPairProbeFirstNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-simp-loop-iterations=\", rumPairProbeFirstLoopIterations];"
    mark("mkPairInj-first-simp-EQMP")
    print "    simplifiedEq = HOL`Kernel`EQMP[rumPairProbeFirstCur, lhsAppXY];"
    getline
    next
  }
  if (detail == "mkPairInjFirstSimpFixpoint" && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-target-concl")
    print "    rumPairProbeMkPairFirstTarget = HOL`Kernel`concl[lhsAppXY];"
    mark("mkPairInj-first-simp-withBasic")
    print "    rumPairProbeFirstRuleThms = HOL`Auto`Simp`Private`withBasic[{mkPairDefThm}];"
    mark("mkPairInj-first-simp-buildBaseConv")
    print "    rumPairProbeFirstBaseConv = HOL`Auto`Simp`Private`buildBaseConv[rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-simp-build-stepConv")
    print "    rumPairProbeFirstStepConv = HOL`Auto`Simp`Private`simpStepConv[rumPairProbeFirstBaseConv, rumPairProbeFirstRuleThms, {}, HOL`Auto`Simp`Private`$simpDepthLimit];"
    mark("mkPairInj-first-simp-first-step")
    print "    rumPairProbeFirstCur = rumPairProbeFirstStepConv[rumPairProbeMkPairFirstTarget];"
    mark("mkPairInj-first-simp-init-seen")
    print "    rumPairProbeFirstRhs = HOL`Kernel`concl[rumPairProbeFirstCur][[2]];"
    print "    rumPairProbeFirstSeen = <|rumPairProbeMkPairFirstTarget -> True, rumPairProbeFirstRhs -> True|>;"
    print "    rumPairProbeFirstLoopIterations = 0;"
    mark("mkPairInj-first-simp-fixpoint-loop")
    print "    While[True,"
    print "      rumPairProbeFirstLoopIterations++;"
    print "      rumPairProbeFirstNext = rumPairProbeFirstStepConv[rumPairProbeFirstRhs];"
    print "      rumPairProbeFirstNewRhs = HOL`Kernel`concl[rumPairProbeFirstNext][[2]];"
    print "      If[rumPairProbeFirstNewRhs === rumPairProbeFirstRhs, Break[]];"
    print "      If[KeyExistsQ[rumPairProbeFirstSeen, rumPairProbeFirstNewRhs], Break[]];"
    print "      rumPairProbeFirstCur = HOL`Kernel`TRANS[rumPairProbeFirstCur, rumPairProbeFirstNext];"
    print "      rumPairProbeFirstRhs = rumPairProbeFirstNewRhs;"
    print "      rumPairProbeFirstSeen[rumPairProbeFirstNewRhs] = True"
    print "    ];"
    print "    Print[\"RUM_PAIR_PHASE mkPairInj-first-simp-loop-iterations=\", rumPairProbeFirstLoopIterations];"
    mark("mkPairInj-first-simp-EQMP")
    print "    simplifiedEq = HOL`Kernel`EQMP[rumPairProbeFirstCur, lhsAppXY];"
    getline
    next
  }
  if (detail == "mkPairInjFirstConvRule" && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-build-simpConv")
    print "    rumPairProbeMkPairFirstConv = HOL`Auto`Simp`simpConv[{mkPairDefThm}];"
    mark("mkPairInj-first-simp-target-concl")
    print "    rumPairProbeMkPairFirstTarget = HOL`Kernel`concl[lhsAppXY];"
    mark("mkPairInj-first-simp-call-conv")
    print "    rumPairProbeMkPairFirstEqTh = rumPairProbeMkPairFirstConv[rumPairProbeMkPairFirstTarget];"
    mark("mkPairInj-first-simp-EQMP")
    print "    simplifiedEq = HOL`Kernel`EQMP[rumPairProbeMkPairFirstEqTh, lhsAppXY];"
    getline
    next
  }
  if (detail == "mkPairInjFirstConv" && inMkPairInj &&
      $0 ~ /^    simplifiedEq = HOL`Drule`CONVRULE\[$/) {
    mark("mkPairInj-first-simp-CONVRULE")
    mark("mkPairInj-first-simp-build-simpConv")
    print "    rumPairProbeMkPairFirstConv = HOL`Auto`Simp`simpConv[{mkPairDefThm}];"
    mark("mkPairInj-first-simp-apply-CONVRULE")
    print "    simplifiedEq = HOL`Drule`CONVRULE[rumPairProbeMkPairFirstConv, lhsAppXY];"
    getline
    next
  }
  if ((detail == "mkPairInj" || detail == "mkPairInjFirstConv" ||
       detail == "mkPairInjFirstConvRule" ||
       detail == "mkPairInjFirstSimpFixpoint" ||
       detail == "mkPairInjFirstSimpStep" ||
       detail == "mkPairInjFirstDescendOnce" ||
       detail == "mkPairInjFirstDescendF" ||
       detail == "mkPairInjFirstDescendFX" ||
       detail == "mkPairInjFirstDescendFXDescend" ||
       detail == "mkPairInjFirstPath" ||
       detail == "mkPairInjFirstPathFXF" ||
       detail == "mkPairInjFirstPathX") && inMkPairInj) {
    if ($0 ~ /^    xV = /) mark("mkPairInj-init-vars")
    if ($0 ~ /^    hypEq = /) mark("mkPairInj-assume-hyp-eq")
    if ($0 ~ /^    lhsAppX = /) mark("mkPairInj-lhsAppX-MKCOMB")
    if ($0 ~ /^    lhsAppXY = /) mark("mkPairInj-lhsAppXY-MKCOMB")
    if ($0 ~ /^    simplifiedEq = /) mark("mkPairInj-first-simp-CONVRULE")
    if ($0 ~ /^    xEqxTh = /) mark("mkPairInj-reflexive-EQTINTROs")
    if ($0 ~ /^    conjPart = /) mark("mkPairInj-second-simp-CONVRULE")
    if ($0 ~ /^    dischargedTh = /) mark("mkPairInj-DISCH")
    if ($0 ~ /^    dischargedTh$/) mark("mkPairInj-return")
  }
  if ($0 ~ /^pairInjThm =/) {
    mark("pairInjThm")
    inMkPairInj = 0
  }
  if ($0 ~ /^fstDefThm =/) mark("fst-snd-definitions")
  if ($0 ~ /^fstPairEqThm =/) mark("fstPairEqThm")
  if ($0 ~ /^sndPairEqThm =/) mark("sndPairEqThm")
  if ($0 ~ /^pairSurjThm =/) mark("pairSurjThm")
  if ($0 ~ /^End\[\];/) mark("package-close")
  print
}
' "${pair_src}" > "${instrumented_pair}"

cat > "${driver_wls}" <<EOF
Get["${common_wls}"];
rumPairProbeLoopSplitLabels = Select[StringSplit["${loop_split_label}", ","], # =!= "" &];
rumPairProbeLoopPathOverrides = ${loop_path_override_wl};

Print["RUM_PAIR_PHASE probe=pair-body-phase-tree"];
Print["RUM_PAIR_PHASE detail=${detail}"];
Print["RUM_PAIR_PHASE source=${pair_src}"];
Print["RUM_PAIR_PHASE instrumented=${instrumented_pair}"];

rumPairProbeMeasure[
  "root-preload-simp-stack",
  rumPairProbeLoadSimpStack[],
  Function[{value}, Null]];

rumPairProbeStageStarted = AbsoluteTime[];
rumPairProbeStageLast = rumPairProbeStageStarted;
rumPairProbeStageCurrent = "package-prefix";
rumPairProbeStageTotal = 0;

rumPairProbeStageNext[next_String] :=
  Module[{now, seconds},
    now = AbsoluteTime[];
    seconds = now - rumPairProbeStageLast;
    rumPairProbeStageTotal = rumPairProbeStageTotal + seconds;
    Print["RUM_PAIR_PHASE label=", rumPairProbeStageCurrent];
    Print["RUM_PAIR_PHASE seconds=", seconds];
    rumPairProbeStageLast = now;
    rumPairProbeStageCurrent = next;
    Null
  ];

rumPairProbeSplitSimpStep[
    label_String, baseConv_, ruleThms_, ctxThms_, depth_, t_, path_List] :=
  Module[{topTh, rhs1, descTh, f, x, fEq, xEq, first, rest,
          bty, body, origin, forbidden, name, v, openTh, opened, innerEq},
    Print["RUM_PAIR_PHASE ", label, "-path=", path];

    Global\`rumPairProbeStageNext[label <> "-top-repeatc"];
    topTh = HOL\`Drule\`TRYCONV[HOL\`Drule\`REPEATC[baseConv]][t];

    Global\`rumPairProbeStageNext[label <> "-rhs1"];
    rhs1 = HOL\`Kernel\`concl[topTh][[2]];
    Print["RUM_PAIR_PHASE ", label, "-isImp=",
      HOL\`Auto\`Simp\`Private\`isImp[rhs1]];
    Print["RUM_PAIR_PHASE ", label, "-isAnd=",
      HOL\`Auto\`Simp\`Private\`isAndForm[rhs1]];
    Print["RUM_PAIR_PHASE ", label, "-isOr=",
      HOL\`Auto\`Simp\`Private\`isOrForm[rhs1]];
    Print["RUM_PAIR_PHASE ", label, "-isComb=",
      MatchQ[rhs1, HOL\`Terms\`comb[_, _]]];
    Print["RUM_PAIR_PHASE ", label, "-isAbs=",
      MatchQ[rhs1, HOL\`Terms\`abs[HOL\`Terms\`bvar[0, _], _, _]]];

    Which[
      Length[path] > 0 && HOL\`Auto\`Simp\`Private\`isAndForm[rhs1],
      descTh = Global\`rumPairProbeSplitAndCong[
        label, ruleThms, ctxThms, depth, rhs1, path],

      Length[path] > 0 && MatchQ[rhs1, HOL\`Terms\`comb[_, _]],
      first = First[path];
      rest = Rest[path];
      f = rhs1[[1]];
      x = rhs1[[2]];
      Which[
        first === "f",
        Global\`rumPairProbeStageNext[label <> "-comb-f-split"];
        fEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-f", ruleThms, ctxThms, depth, f, rest];
        Global\`rumPairProbeStageNext[label <> "-comb-x-sibling"];
        xEq = HOL\`Auto\`Simp\`Private\`simpFixpointConv[
          ruleThms, ctxThms, depth][x],

        first === "x",
        Global\`rumPairProbeStageNext[label <> "-comb-f-sibling"];
        fEq = HOL\`Auto\`Simp\`Private\`simpFixpointConv[
          ruleThms, ctxThms, depth][f];
        Global\`rumPairProbeStageNext[label <> "-comb-x-split"];
        xEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-x", ruleThms, ctxThms, depth, x, rest],

        True,
        Global\`rumPairProbeStageNext[label <> "-comb-f-split"];
        fEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-f", ruleThms, ctxThms, depth, f, rest];
        Global\`rumPairProbeStageNext[label <> "-comb-x-split"];
        xEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-x", ruleThms, ctxThms, depth, x, rest]
      ];
      Global\`rumPairProbeStageNext[label <> "-comb-MKCOMB"];
      descTh = HOL\`Kernel\`MKCOMB[fEq, xEq],

      Length[path] > 0 &&
        MatchQ[rhs1, HOL\`Terms\`abs[HOL\`Terms\`bvar[0, _], _, _]],
      first = First[path];
      rest = Rest[path];
      If[first === "a" || first === "b",
        bty = rhs1[[1, 2]];
        body = rhs1[[2]];
        origin = rhs1[[3]];
        Global\`rumPairProbeStageNext[label <> "-abs-freesIn"];
        forbidden = First /@ HOL\`Terms\`freesIn[body];
        Global\`rumPairProbeStageNext[label <> "-abs-pickFresh"];
        name = HOL\`Auto\`Simp\`Private\`pickFreshNm[origin, forbidden];
        v = HOL\`Terms\`mkVar[name, bty];
        Global\`rumPairProbeStageNext[label <> "-abs-BETA-open"];
        openTh = HOL\`Kernel\`BETA[
          HOL\`Terms\`mkComb[
            HOL\`Terms\`abs[HOL\`Terms\`bvar[0, bty], body, name],
            v]];
        opened = HOL\`Kernel\`concl[openTh][[2]];
        Global\`rumPairProbeStageNext[label <> "-abs-inner-split"];
        innerEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-abs", ruleThms, ctxThms, depth, opened, rest];
        Global\`rumPairProbeStageNext[label <> "-abs-ABS"];
        descTh = HOL\`Kernel\`ABS[v, innerEq],

        Global\`rumPairProbeStageNext[label <> "-descendOnce"];
        descTh = HOL\`Auto\`Simp\`Private\`descendOnce[
          ruleThms, ctxThms, depth][rhs1]
      ],

      True,
      Global\`rumPairProbeStageNext[label <> "-descendOnce"];
      descTh = HOL\`Auto\`Simp\`Private\`descendOnce[
        ruleThms, ctxThms, depth][rhs1]
    ];

    Global\`rumPairProbeStageNext[label <> "-TRANS"];
    HOL\`Kernel\`TRANS[topTh, descTh]
  ];

rumPairProbeSplitAndCong[
    label_String, ruleThms_, ctxThms_, depth_, t_, path_List] :=
  Module[{first, rest, p1Tm, q1Tm, eq1, p2Tm, eq2, q2Tm,
          dischEq2, conjAx, p1V, p2V, q1V, q2V, lemma, instLemma},
    first = First[path];
    rest = Rest[path];
    p1Tm = t[[1, 2]];
    q1Tm = t[[2]];

    Which[
      first === "f",
      Global\`rumPairProbeStageNext[label <> "-and-p-split"];
      eq1 = Global\`rumPairProbeSplitSimpFixpoint[
        label <> "-and-p", ruleThms, ctxThms, depth, p1Tm, rest];
      p2Tm = HOL\`Kernel\`concl[eq1][[2]];
      Global\`rumPairProbeStageNext[label <> "-and-q-sibling"];
      eq2 = HOL\`Auto\`Simp\`Private\`simpFixpointConv[
        ruleThms, Append[ctxThms, HOL\`Kernel\`ASSUME[p2Tm]], depth][q1Tm],

      first === "x",
      Global\`rumPairProbeStageNext[label <> "-and-p-sibling"];
      eq1 = HOL\`Auto\`Simp\`Private\`simpFixpointConv[
        ruleThms, ctxThms, depth][p1Tm];
      p2Tm = HOL\`Kernel\`concl[eq1][[2]];
      Global\`rumPairProbeStageNext[label <> "-and-q-split"];
      eq2 = Global\`rumPairProbeSplitSimpFixpoint[
        label <> "-and-q", ruleThms,
        Append[ctxThms, HOL\`Kernel\`ASSUME[p2Tm]], depth, q1Tm, rest],

      True,
      Global\`rumPairProbeStageNext[label <> "-and-p-split"];
      eq1 = Global\`rumPairProbeSplitSimpFixpoint[
        label <> "-and-p", ruleThms, ctxThms, depth, p1Tm, rest];
      p2Tm = HOL\`Kernel\`concl[eq1][[2]];
      Global\`rumPairProbeStageNext[label <> "-and-q-split"];
      eq2 = Global\`rumPairProbeSplitSimpFixpoint[
        label <> "-and-q", ruleThms,
        Append[ctxThms, HOL\`Kernel\`ASSUME[p2Tm]], depth, q1Tm, rest]
    ];

    Global\`rumPairProbeStageNext[label <> "-and-q-rhs"];
    q2Tm = HOL\`Kernel\`concl[eq2][[2]];
    Global\`rumPairProbeStageNext[label <> "-and-DISCH"];
    dischEq2 = HOL\`Bool\`DISCH[p2Tm, eq2];
    Global\`rumPairProbeStageNext[label <> "-and-CONJ"];
    conjAx = HOL\`Bool\`CONJ[eq1, dischEq2];
    Global\`rumPairProbeStageNext[label <> "-and-vars"];
    p1V = HOL\`Terms\`mkVar["p1", HOL\`Kernel\`boolTy];
    p2V = HOL\`Terms\`mkVar["p2", HOL\`Kernel\`boolTy];
    q1V = HOL\`Terms\`mkVar["q1", HOL\`Kernel\`boolTy];
    q2V = HOL\`Terms\`mkVar["q2", HOL\`Kernel\`boolTy];
    Global\`rumPairProbeStageNext[label <> "-and-lemma"];
    lemma = HOL\`Auto\`Simp\`Private\`andCongLemma[];
    Global\`rumPairProbeStageNext[label <> "-and-INST"];
    instLemma = HOL\`Kernel\`INST[
      {p1V -> p1Tm, p2V -> p2Tm, q1V -> q1Tm, q2V -> q2Tm},
      lemma];
    Global\`rumPairProbeStageNext[label <> "-and-MP"];
    HOL\`Bool\`MP[instLemma, conjAx]
  ];

rumPairProbeSplitSimpFixpoint[
    label_String, ruleThms_, ctxThms_, depth_, t_, path_List] :=
  Module[{baseConv, stepConv, topTh, rhs1, descTh, f, x, fEq, xEq,
          first, rest, bty, body, origin, forbidden, name, v, openTh,
          opened, innerEq, cur, rhs, next, newRhs, seen, loopIterations,
          loopPath},
    Print["RUM_PAIR_PHASE ", label, "-path=", path];
    Global\`rumPairProbeStageNext[label <> "-buildBaseConv"];
    baseConv = HOL\`Auto\`Simp\`Private\`buildBaseConv[ruleThms, ctxThms, depth];

    Global\`rumPairProbeStageNext[label <> "-build-stepConv"];
    stepConv = HOL\`Auto\`Simp\`Private\`simpStepConv[
      baseConv, ruleThms, ctxThms, depth];

    Global\`rumPairProbeStageNext[label <> "-top-repeatc"];
    topTh = HOL\`Drule\`TRYCONV[HOL\`Drule\`REPEATC[baseConv]][t];

    Global\`rumPairProbeStageNext[label <> "-rhs1"];
    rhs1 = HOL\`Kernel\`concl[topTh][[2]];
    Print["RUM_PAIR_PHASE ", label, "-isImp=",
      HOL\`Auto\`Simp\`Private\`isImp[rhs1]];
    Print["RUM_PAIR_PHASE ", label, "-isAnd=",
      HOL\`Auto\`Simp\`Private\`isAndForm[rhs1]];
    Print["RUM_PAIR_PHASE ", label, "-isOr=",
      HOL\`Auto\`Simp\`Private\`isOrForm[rhs1]];
    Print["RUM_PAIR_PHASE ", label, "-isComb=",
      MatchQ[rhs1, HOL\`Terms\`comb[_, _]]];
    Print["RUM_PAIR_PHASE ", label, "-isAbs=",
      MatchQ[rhs1, HOL\`Terms\`abs[HOL\`Terms\`bvar[0, _], _, _]]];

    Which[
      Length[path] > 0 && HOL\`Auto\`Simp\`Private\`isAndForm[rhs1],
      descTh = Global\`rumPairProbeSplitAndCong[
        label, ruleThms, ctxThms, depth, rhs1, path],

      Length[path] > 0 && MatchQ[rhs1, HOL\`Terms\`comb[_, _]],
      first = First[path];
      rest = Rest[path];
      f = rhs1[[1]];
      x = rhs1[[2]];
      Which[
        first === "f",
        Global\`rumPairProbeStageNext[label <> "-comb-f-split"];
        fEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-f", ruleThms, ctxThms, depth, f, rest];
        Global\`rumPairProbeStageNext[label <> "-comb-x-sibling"];
        xEq = HOL\`Auto\`Simp\`Private\`simpFixpointConv[
          ruleThms, ctxThms, depth][x],

        first === "x",
        Global\`rumPairProbeStageNext[label <> "-comb-f-sibling"];
        fEq = HOL\`Auto\`Simp\`Private\`simpFixpointConv[
          ruleThms, ctxThms, depth][f];
        Global\`rumPairProbeStageNext[label <> "-comb-x-split"];
        xEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-x", ruleThms, ctxThms, depth, x, rest],

        True,
        Global\`rumPairProbeStageNext[label <> "-comb-f-split"];
        fEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-f", ruleThms, ctxThms, depth, f, rest];
        Global\`rumPairProbeStageNext[label <> "-comb-x-split"];
        xEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-x", ruleThms, ctxThms, depth, x, rest]
      ];
      Global\`rumPairProbeStageNext[label <> "-comb-MKCOMB"];
      descTh = HOL\`Kernel\`MKCOMB[fEq, xEq],

      Length[path] > 0 &&
        MatchQ[rhs1, HOL\`Terms\`abs[HOL\`Terms\`bvar[0, _], _, _]],
      first = First[path];
      rest = Rest[path];
      If[first === "a" || first === "b",
        bty = rhs1[[1, 2]];
        body = rhs1[[2]];
        origin = rhs1[[3]];
        Global\`rumPairProbeStageNext[label <> "-abs-freesIn"];
        forbidden = First /@ HOL\`Terms\`freesIn[body];
        Global\`rumPairProbeStageNext[label <> "-abs-pickFresh"];
        name = HOL\`Auto\`Simp\`Private\`pickFreshNm[origin, forbidden];
        v = HOL\`Terms\`mkVar[name, bty];
        Global\`rumPairProbeStageNext[label <> "-abs-BETA-open"];
        openTh = HOL\`Kernel\`BETA[
          HOL\`Terms\`mkComb[
            HOL\`Terms\`abs[HOL\`Terms\`bvar[0, bty], body, name],
            v]];
        opened = HOL\`Kernel\`concl[openTh][[2]];
        Global\`rumPairProbeStageNext[label <> "-abs-inner-split"];
        innerEq = Global\`rumPairProbeSplitSimpFixpoint[
          label <> "-abs", ruleThms, ctxThms, depth, opened, rest];
        Global\`rumPairProbeStageNext[label <> "-abs-ABS"];
        descTh = HOL\`Kernel\`ABS[v, innerEq],

        Global\`rumPairProbeStageNext[label <> "-descendOnce"];
        descTh = HOL\`Auto\`Simp\`Private\`descendOnce[
          ruleThms, ctxThms, depth][rhs1]
      ],

      True,
      Global\`rumPairProbeStageNext[label <> "-descendOnce"];
      descTh = HOL\`Auto\`Simp\`Private\`descendOnce[
        ruleThms, ctxThms, depth][rhs1]
    ];

    Global\`rumPairProbeStageNext[label <> "-TRANS"];
    cur = HOL\`Kernel\`TRANS[topTh, descTh];

    Global\`rumPairProbeStageNext[label <> "-init-seen"];
    rhs = HOL\`Kernel\`concl[cur][[2]];
    seen = <|t -> True, rhs -> True|>;
    loopIterations = 0;

    Global\`rumPairProbeStageNext[label <> "-fixpoint-loop"];
    While[True,
      loopIterations++;
      next = If[
        Length[path] > 0 && MemberQ[Global\`rumPairProbeLoopSplitLabels, label],
        loopPath = If[
          KeyExistsQ[Global\`rumPairProbeLoopPathOverrides, label],
          Global\`rumPairProbeLoopPathOverrides[label],
          path];
        If[loopPath =!= path,
          Print["RUM_PAIR_PHASE ", label, "-loop-path-override=", loopPath]];
        Global\`rumPairProbeSplitSimpStep[
          label <> "-loop" <> ToString[loopIterations],
          baseConv, ruleThms, ctxThms, depth, rhs, loopPath],
        stepConv[rhs]];
      newRhs = HOL\`Kernel\`concl[next][[2]];
      If[newRhs === rhs, Break[]];
      If[KeyExistsQ[seen, newRhs], Break[]];
      cur = HOL\`Kernel\`TRANS[cur, next];
      rhs = newRhs;
      seen[newRhs] = True
    ];
    Print["RUM_PAIR_PHASE ", label, "-loop-iterations=", loopIterations];
    cur
  ];

Get["${instrumented_pair}"];
rumPairProbeStageNext["file-end"];

Print["RUM_PAIR_PHASE pair_body_phase_sum=", rumPairProbeStageTotal];
Print[
  "RUM_PAIR_PHASE pair_mkPairInjThm_is_thm=",
  HOL\`Kernel\`isThm[HOL\`Stdlib\`Pair\`mkPairInjThm]];
Print[
  "RUM_PAIR_PHASE pair_pairInjThm_is_thm=",
  HOL\`Kernel\`isThm[HOL\`Stdlib\`Pair\`pairInjThm]];
Null
EOF

printf 'probe\tengine\tstatus\texit_code\telapsed_seconds\tstdout\tstderr\n' > "${summary_file}"

echo "[rum-pair-body-phase] mma-hol: ${mma_hol_root}"
echo "[rum-pair-body-phase] engine: ${engine}; detail: ${detail}; timeout: ${timeout_seconds}s"
echo "[rum-pair-body-phase] logs: ${output_dir}"

start=$(date +%s)
set +e
if [[ "${engine}" == "rum" ]]; then
  (
    cd "${mma_hol_root}"
    "${bounded_runner}" --timeout "${timeout_seconds}" -- \
      "${rum_cli_bin}" --file "${driver_wls}"
  ) > "${stdout_file}" 2> "${stderr_file}"
else
  (
    cd "${mma_hol_root}"
    "${bounded_runner}" --timeout "${timeout_seconds}" -- \
      "${wolfram_bin}" -noprompt -script "${driver_wls}"
  ) > "${stdout_file}" 2> "${stderr_file}"
fi
exit_code=$?
set -e
end=$(date +%s)
elapsed=$((end - start))

if [[ "${exit_code}" -eq 0 ]]; then
  status=ok
else
  status=fail
fi
printf 'pair-body-phase-tree\t%s\t%s\t%s\t%s\t%s\t%s\n' \
  "${engine}" "${status}" "${exit_code}" "${elapsed}" \
  "${stdout_file}" "${stderr_file}" >> "${summary_file}"

echo "${status} (${elapsed}s)"
echo "[rum-pair-body-phase] summary: ${summary_file}"

exit "${exit_code}"
