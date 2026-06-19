#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/pair-simp-repetition-probe.sh [--mma-hol PATH] [options]

Build and run temporary instrumented copies of auto/Simp.wl and, by default,
stdlib/Pair.wl. The canonical mma-hol checkout is not edited.

The probe records term fingerprints at simpFixpointConv/descendOnce boundaries
and emits aggregate rows grouped by kind, depth, rule count, context count,
term shape, and SHA256 term fingerprint.

Options:
  --mma-hol PATH       mma-hol checkout root. Required unless RUM_MMA_HOL_ROOT
                       is set.
  --target TARGET      first-convrule or full-load. Defaults to first-convrule.
  --track-step         Also record simpStepConv calls. Off by default because it
                       can add substantial probe overhead.
  --split-kind KIND    Optional targeted split: simpFixpointConv, simpStepConv,
                       descendOnce, topRepeatc, topRepeatcTrace, or
                       topRepeatcNativeTrace.
  --split-fingerprint FP
                       SHA256 term fingerprint to split for --split-kind.
  --split-shape SHAPE  Optional shape filter, e.g. abs, and, comb, eq.
  --split-depth N      Optional depth filter.
  --split-rule-count N Optional rule-count filter.
  --split-context-count N
                       Optional context-count filter.
  --split-match-index N
                       Split the Nth matching call. Defaults to 1.
  --split-limit N      Number of matching calls to split. Defaults to 1 when a
                       split fingerprint is provided, otherwise 0.
  --prototype KIND     Temporary diagnostic prototype: none, flat-baseconv,
                       failure-cache, or flat-failure-cache. Defaults to none.
  --timeout SECONDS    Wall timeout. Defaults to 1200.
  --output-dir PATH    Directory for generated files and logs.
  --engine ENGINE      rum or wolfram. Defaults to rum.
  --rum-cli PATH       Existing rum-cli binary. Defaults to target/release/rum-cli.
  --wolfram PATH       WolframKernel binary for --engine wolfram.
  -h, --help           Show this help.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
bounded_runner="${repo_root}/scripts/run-bounded.py"

mma_hol_root=${RUM_MMA_HOL_ROOT:-}
timeout_seconds=${RUM_PAIR_REPEAT_TIMEOUT:-1200}
output_dir=${RUM_PAIR_REPEAT_OUTPUT_DIR:-}
engine=${RUM_PAIR_REPEAT_ENGINE:-rum}
target=${RUM_PAIR_REPEAT_TARGET:-first-convrule}
track_step=${RUM_PAIR_REPEAT_TRACK_STEP:-0}
split_kind=${RUM_PAIR_REPEAT_SPLIT_KIND:-}
split_fingerprint=${RUM_PAIR_REPEAT_SPLIT_FINGERPRINT:-}
split_shape=${RUM_PAIR_REPEAT_SPLIT_SHAPE:-}
split_depth=${RUM_PAIR_REPEAT_SPLIT_DEPTH:--1}
split_rule_count=${RUM_PAIR_REPEAT_SPLIT_RULE_COUNT:--1}
split_context_count=${RUM_PAIR_REPEAT_SPLIT_CONTEXT_COUNT:--1}
split_match_index=${RUM_PAIR_REPEAT_SPLIT_MATCH_INDEX:-1}
split_limit=${RUM_PAIR_REPEAT_SPLIT_LIMIT:-}
prototype=${RUM_PAIR_REPEAT_PROTOTYPE:-none}
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
    --target)
      [[ $# -ge 2 ]] || { echo "error: --target expects first-convrule or full-load" >&2; exit 2; }
      target=$2
      shift 2
      ;;
    --track-step)
      track_step=1
      shift
      ;;
    --split-kind)
      [[ $# -ge 2 ]] || { echo "error: --split-kind expects a split kind" >&2; exit 2; }
      split_kind=$2
      shift 2
      ;;
    --split-fingerprint)
      [[ $# -ge 2 ]] || { echo "error: --split-fingerprint expects a hash string" >&2; exit 2; }
      split_fingerprint=$2
      shift 2
      ;;
    --split-shape)
      [[ $# -ge 2 ]] || { echo "error: --split-shape expects a shape label" >&2; exit 2; }
      split_shape=$2
      shift 2
      ;;
    --split-depth)
      [[ $# -ge 2 ]] || { echo "error: --split-depth expects an integer" >&2; exit 2; }
      split_depth=$2
      shift 2
      ;;
    --split-rule-count)
      [[ $# -ge 2 ]] || { echo "error: --split-rule-count expects an integer" >&2; exit 2; }
      split_rule_count=$2
      shift 2
      ;;
    --split-context-count)
      [[ $# -ge 2 ]] || { echo "error: --split-context-count expects an integer" >&2; exit 2; }
      split_context_count=$2
      shift 2
      ;;
    --split-match-index)
      [[ $# -ge 2 ]] || { echo "error: --split-match-index expects a positive integer" >&2; exit 2; }
      split_match_index=$2
      shift 2
      ;;
    --split-limit)
      [[ $# -ge 2 ]] || { echo "error: --split-limit expects a non-negative integer" >&2; exit 2; }
      split_limit=$2
      shift 2
      ;;
    --prototype)
      [[ $# -ge 2 ]] || { echo "error: --prototype expects none, flat-baseconv, failure-cache, or flat-failure-cache" >&2; exit 2; }
      prototype=$2
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

case "${target}" in
  first-convrule|full-load) ;;
  *) echo "error: --target expects first-convrule or full-load" >&2; exit 2 ;;
esac

case "${track_step}" in
  0|1) ;;
  *) echo "error: --track-step flag or RUM_PAIR_REPEAT_TRACK_STEP must be 0 or 1" >&2; exit 2 ;;
esac

case "${split_kind}" in
  ""|simpFixpointConv|simpStepConv|descendOnce|topRepeatc|topRepeatcTrace|topRepeatcNativeTrace) ;;
  *) echo "error: --split-kind expects simpFixpointConv, simpStepConv, descendOnce, topRepeatc, topRepeatcTrace, or topRepeatcNativeTrace" >&2; exit 2 ;;
esac

case "${prototype}" in
  none|flat-baseconv|failure-cache|flat-failure-cache) ;;
  *) echo "error: --prototype expects none, flat-baseconv, failure-cache, or flat-failure-cache" >&2; exit 2 ;;
esac

if [[ -n "${split_kind}" && -z "${split_fingerprint}" ]]; then
  echo "error: --split-kind requires --split-fingerprint" >&2
  exit 2
fi

if [[ -z "${split_limit}" ]]; then
  if [[ -n "${split_fingerprint}" ]]; then
    split_limit=1
  else
    split_limit=0
  fi
fi

for numeric_option in split_depth split_rule_count split_context_count; do
  numeric_value=${!numeric_option}
  if ! [[ "${numeric_value}" =~ ^-?[0-9]+$ ]]; then
    echo "error: ${numeric_option} expects an integer" >&2
    exit 2
  fi
done

if ! [[ "${split_match_index}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --split-match-index expects a positive integer" >&2
  exit 2
fi

if ! [[ "${split_limit}" =~ ^[0-9]+$ ]]; then
  echo "error: --split-limit expects a non-negative integer" >&2
  exit 2
fi

if ! [[ "${timeout_seconds}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --timeout expects a positive integer" >&2
  exit 2
fi

if [[ -z "${mma_hol_root}" ]]; then
  echo "error: --mma-hol is required when RUM_MMA_HOL_ROOT is unset" >&2
  exit 2
fi

simp_src="${mma_hol_root}/auto/Simp.wl"
pair_src="${mma_hol_root}/stdlib/Pair.wl"
if [[ ! -f "${simp_src}" || ! -f "${pair_src}" ]]; then
  echo "error: ${mma_hol_root} does not look like an mma-hol checkout" >&2
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
  output_dir=$(mktemp -d /tmp/rum-pair-repeat.XXXXXX)
else
  mkdir -p "${output_dir}"
fi

instrumented_simp="${output_dir}/Simp.instrumented.wl"
instrumented_pair="${output_dir}/Pair.instrumented.wl"
driver_wls="${output_dir}/pair-simp-repetition-probe.wls"
stdout_file="${output_dir}/pair-simp-repetition.${engine}.stdout"
stderr_file="${output_dir}/pair-simp-repetition.${engine}.stderr"
summary_file="${output_dir}/summary.tsv"
rows_file="${output_dir}/repeat-rows.tsv"
top_time_file="${output_dir}/repeat-top-time.tsv"
top_calls_file="${output_dir}/repeat-top-calls.tsv"

awk -v track_step="${track_step}" \
    -v split_kind="${split_kind}" \
    -v split_fingerprint="${split_fingerprint}" \
    -v split_shape="${split_shape}" \
    -v split_depth="${split_depth}" \
    -v split_rule_count="${split_rule_count}" \
    -v split_context_count="${split_context_count}" \
    -v split_match_index="${split_match_index}" \
    -v split_limit="${split_limit}" \
    -v prototype="${prototype}" '
function emit_wrappers() {
  print "";
  print "(* rum temporary repetition probe wrappers. *)";
  print "Global`rumPairRepeatStats = <||>;";
  print "Global`rumPairRepeatPrototype = \"" prototype "\";";
  print "Global`rumPairRepeatTrackStep = " (track_step == "1" ? "True" : "False") ";";
  print "Global`rumPairRepeatSplitKind = \"" split_kind "\";";
  print "Global`rumPairRepeatSplitFingerprint = \"" split_fingerprint "\";";
  print "Global`rumPairRepeatSplitShapeFilter = \"" split_shape "\";";
  print "Global`rumPairRepeatSplitDepthFilter = " split_depth ";";
  print "Global`rumPairRepeatSplitRuleCountFilter = " split_rule_count ";";
  print "Global`rumPairRepeatSplitContextCountFilter = " split_context_count ";";
  print "Global`rumPairRepeatSplitMatchIndex = " split_match_index ";";
  print "Global`rumPairRepeatSplitLimit = " split_limit ";";
  print "Global`rumPairRepeatSplitSeen = 0;";
  print "Global`rumPairRepeatSplitDone = 0;";
  print "Global`rumPairRepeatSplitActive = False;";
  print "Global`rumPairRepeatBaseFailed = Unique[\"rumPairRepeatBaseFailed\"];";
  print "Global`rumPairRepeatTopCache = <||>;";
  print "Global`rumPairRepeatTopCacheHits = 0;";
  print "Global`rumPairRepeatTopCacheStores = 0;";
  print "Global`rumPairRepeatTopCacheChanged = 0;";
  print "Global`rumPairRepeatTopCacheSkipped = 0;";
  print "Global`rumPairNativeTraceActive = False;";
  print "Global`rumPairNativeTraceLabel = \"\";";
  print "Global`rumPairNativeOrelseDepth = 0;";
  print "Global`rumPairRepeatShape[t_] := Which[";
  print "  isImp[t], \"imp\",";
  print "  isAndForm[t], \"and\",";
  print "  isOrForm[t], \"or\",";
  print "  isEquation[t], \"eq\",";
  print "  MatchQ[t, comb[_, _]], \"comb\",";
  print "  MatchQ[t, abs[bvar[0, _], _, _]], \"abs\",";
  print "  True, ToString[Head[t], InputForm]";
  print "];";
  print "Global`rumPairRepeatFingerprint[t_] :=";
  print "  Hash[t, \"SHA256\", \"HexString\"];";
  print "Global`rumPairRepeatAtomicLeafQ[t_] :=";
  print "  MatchQ[t, var[_, _] | const[_, _]];";
  print "Global`rumPairRepeatFailureCachePrototypeQ[] :=";
  print "  Global`rumPairRepeatPrototype === \"failure-cache\" ||";
  print "  Global`rumPairRepeatPrototype === \"flat-failure-cache\";";
  print "Global`rumPairRepeatFlatBasePrototypeQ[] :=";
  print "  Global`rumPairRepeatPrototype === \"flat-baseconv\" ||";
  print "  Global`rumPairRepeatPrototype === \"flat-failure-cache\";";
  print "Global`rumPairRepeatTopCacheKey[ruleThms_, ctxThms_, depth_, t_] :=";
  print "  StringJoin[\"d=\", ToString[depth], \"\\tr=\",";
  print "    ToString[Length[ruleThms]], \"\\tc=\",";
  print "    ToString[Length[ctxThms]], \"\\trh=\",";
  print "    Hash[concl /@ ruleThms, \"SHA256\", \"HexString\"],";
  print "    \"\\tch=\", Hash[concl /@ ctxThms, \"SHA256\",";
  print "      \"HexString\"], \"\\tt=\",";
  print "    Global`rumPairRepeatFingerprint[t]];";
  print "Global`rumPairRepeatRunTopRepeatc[baseConv_, ruleThms_, ctxThms_, depth_, t_] :=";
  print "  Module[{useCache, key, topTh, rhs},";
  print "    useCache = Global`rumPairRepeatFailureCachePrototypeQ[] &&";
  print "      Global`rumPairRepeatAtomicLeafQ[t];";
  print "    If[useCache,";
  print "      key = Global`rumPairRepeatTopCacheKey[ruleThms, ctxThms,";
  print "        depth, t];";
  print "      If[KeyExistsQ[Global`rumPairRepeatTopCache, key],";
  print "        Global`rumPairRepeatTopCacheHits++;";
  print "        Return[REFL[t]]";
  print "      ],";
  print "      Global`rumPairRepeatTopCacheSkipped++";
  print "    ];";
  print "    topTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[baseConv]][t];";
  print "    If[useCache,";
  print "      rhs = concl[topTh][[2]];";
  print "      If[rhs === t,";
  print "        Global`rumPairRepeatTopCache[key] = True;";
  print "        Global`rumPairRepeatTopCacheStores++,";
  print "        Global`rumPairRepeatTopCacheChanged++";
  print "      ]";
  print "    ];";
  print "    topTh";
  print "  ];";
  print "Global`rumPairRepeatTryFlatPiece[conv_, t_] :=";
  print "  Catch[conv[t], HOL`Error`holErrorTag];";
  print "Global`rumPairRepeatFlatCombine[convs_List][t_] :=";
  print "  Module[{all, i, res, last},";
  print "    all = Join[convs, {BETACONV}];";
  print "    last = Null;";
  print "    For[i = 1, i <= Length[all], i++,";
  print "      res = Global`rumPairRepeatTryFlatPiece[all[[i]], t];";
  print "      If[!MatchQ[res, _Failure], Return[res]];";
  print "      last = res";
  print "    ];";
  print "    If[last === Null,";
  print "      Throw[HOL`Error`holError[\"conv\",";
  print "        \"flat combineSimpConvs: no conversion\", <||>],";
  print "        HOL`Error`holErrorTag],";
  print "      Throw[last, HOL`Error`holErrorTag]";
  print "    ]";
  print "  ];";
  print "If[Global`rumPairRepeatFlatBasePrototypeQ[],";
  print "  Clear[combineSimpConvs];";
  print "  combineSimpConvs[convs_List] :=";
  print "    Global`rumPairRepeatFlatCombine[convs];";
  print "];";
  print "Global`rumPairRepeatRecord[kind_, ruleThms_, ctxThms_, depth_, t_, seconds_] :=";
  print "  Module[{shape, fp, key, old},";
  print "    shape = Global`rumPairRepeatShape[t];";
  print "    fp = Global`rumPairRepeatFingerprint[t];";
  print "    key = StringJoin[kind, \"\\t\", ToString[depth], \"\\t\",";
  print "      ToString[Length[ruleThms]], \"\\t\", ToString[Length[ctxThms]],";
  print "      \"\\t\", shape, \"\\t\", fp];";
  print "    If[KeyExistsQ[Global`rumPairRepeatStats, key],";
  print "      old = Global`rumPairRepeatStats[key];";
  print "      Global`rumPairRepeatStats[key] = {old[[1]] + 1, old[[2]] + seconds},";
  print "      Global`rumPairRepeatStats[key] = {1, seconds}";
  print "    ];";
  print "    Null";
  print "  ];";
  print "SetAttributes[Global`rumPairRepeatTimed, HoldRest];";
  print "Global`rumPairRepeatTimed[label_, expr_] :=";
  print "  Module[{seconds, result},";
  print "    seconds = AbsoluteTiming[result = expr;][[1]];";
  print "    Print[\"RUM_PAIR_SPLIT\\t\", label, \"\\t\", seconds];";
  print "    result";
  print "  ];";
  print "Global`rumPairRepeatBuildBasePieces[ruleThms_, ctxThms_, depth_] :=";
  print "  Module[{prepared, productive, prover, ctxPieces, rulePieces},";
  print "    prepared = HOL`Auto`Simp`simpPrepareRule /@ ruleThms;";
  print "    productive = Select[prepared, productiveSimpRule];";
  print "    prover = makeProver[ruleThms, ctxThms, depth];";
  print "    ctxPieces = MapIndexed[";
  print "      {\"ctx-\" <> ToString[#2[[1]]], ctxRewrConv[#1], concl[#1]} &,";
  print "      ctxThms];";
  print "    rulePieces = MapIndexed[";
  print "      {\"rule-\" <> ToString[#2[[1]]], simpRuleConv[#1, prover],";
  print "        concl[#1]} &, productive];";
  print "    Join[ctxPieces, rulePieces, {{\"beta\", BETACONV, Null}}]";
  print "  ];";
  print "Global`rumPairRepeatPrintBasePieces[label_, pieces_] :=";
  print "  Module[{i, meta, shape, fp},";
  print "    Do[";
  print "      meta = pieces[[i, 3]];";
  print "      If[meta === Null,";
  print "        shape = \"none\"; fp = \"none\",";
  print "        shape = Global`rumPairRepeatShape[meta];";
  print "        fp = Global`rumPairRepeatFingerprint[meta]";
  print "      ];";
  print "      Print[\"RUM_PAIR_BASECONV_PIECE\\t\", label, \"\\t\", i,";
  print "        \"\\t\", pieces[[i, 1]], \"\\t\", shape, \"\\t\", fp];";
  print "      , {i, Length[pieces]}]";
  print "  ];";
  print "Global`rumPairRepeatTryBasePiece[label_, index_, piece_, t_] :=";
  print "  Module[{seconds, result, status},";
  print "    seconds = AbsoluteTiming[";
  print "      result = Catch[piece[[2]][t], HOL`Error`holErrorTag];";
  print "    ][[1]];";
  print "    status = If[MatchQ[result, _Failure], \"fail\", \"hit\"];";
  print "    Print[\"RUM_PAIR_BASECONV\\t\", label, \"\\t\", index,";
  print "      \"\\t\", piece[[1]], \"\\t\", status, \"\\t\", seconds];";
  print "    If[MatchQ[result, _Failure],";
  print "      Global`rumPairRepeatBaseFailed, result]";
  print "  ];";
  print "Global`rumPairRepeatRunBaseOnce[label_, pieces_, t_] :=";
  print "  Module[{i, result},";
  print "    For[i = 1, i <= Length[pieces], i++,";
  print "      result = Global`rumPairRepeatTryBasePiece[label, i,";
  print "        pieces[[i]], t];";
  print "      If[result =!= Global`rumPairRepeatBaseFailed, Return[result]]";
  print "    ];";
  print "    Global`rumPairRepeatBaseFailed";
  print "  ];";
  print "Global`rumPairRepeatRunRepeatBase[label_, pieces_, t_] :=";
  print "  Module[{result, rhs, next},";
  print "    result = Global`rumPairRepeatRunBaseOnce[label <> \"-iter1\",";
  print "      pieces, t];";
  print "    If[result === Global`rumPairRepeatBaseFailed, Return[REFL[t]]];";
  print "    rhs = concl[result][[2]];";
  print "    If[aconv[rhs, t], Return[result]];";
  print "    next = Global`rumPairRepeatRunRepeatBase[label <> \"-next\",";
  print "      pieces, rhs];";
  print "    TRANS[result, next]";
  print "  ];";
  print "Global`rumPairRepeatRunRightAssocOnce[label_, pieces_, index_, t_] :=";
  print "  Module[{piece, seconds, result, status, restSeconds, rest},";
  print "    piece = pieces[[index]];";
  print "    seconds = AbsoluteTiming[";
  print "      result = Catch[piece[[2]][t], HOL`Error`holErrorTag];";
  print "    ][[1]];";
  print "    status = If[MatchQ[result, _Failure], \"fail\", \"hit\"];";
  print "    Print[\"RUM_PAIR_ORELSEC_TRY\\t\", label, \"\\t\", index,";
  print "      \"\\t\", piece[[1]], \"\\t\", status, \"\\t\", seconds];";
  print "    If[!MatchQ[result, _Failure], Return[result]];";
  print "    If[index >= Length[pieces],";
  print "      Print[\"RUM_PAIR_ORELSEC_REST\\t\", label, \"\\t\", index,";
  print "        \"\\t\", \"end\", \"\\t\", 0];";
  print "      Return[Global`rumPairRepeatBaseFailed]";
  print "    ];";
  print "    restSeconds = AbsoluteTiming[";
  print "      rest = Global`rumPairRepeatRunRightAssocOnce[label, pieces,";
  print "        index + 1, t];";
  print "    ][[1]];";
  print "    Print[\"RUM_PAIR_ORELSEC_REST\\t\", label, \"\\t\", index,";
  print "      \"\\t\", If[rest === Global`rumPairRepeatBaseFailed, \"fail\",";
  print "        \"hit\"], \"\\t\", restSeconds];";
  print "    rest";
  print "  ];";
  print "Global`rumPairRepeatRunRepeatRightAssoc[label_, pieces_, t_] :=";
  print "  Module[{result, rhs, next},";
  print "    result = Global`rumPairRepeatRunRightAssocOnce[";
  print "      label <> \"-iter1\", pieces, 1, t];";
  print "    If[result === Global`rumPairRepeatBaseFailed, Return[REFL[t]]];";
  print "    rhs = concl[result][[2]];";
  print "    If[aconv[rhs, t], Return[result]];";
  print "    next = Global`rumPairRepeatRunRepeatRightAssoc[label <> \"-next\",";
  print "      pieces, rhs];";
  print "    TRANS[result, next]";
  print "  ];";
  print "Clear[HOL`Drule`ORELSEC];";
  print "HOL`Drule`ORELSEC[c1_, c2_][t_] :=";
  print "  Module[{res, seconds, rest, restSeconds, level, failed},";
  print "    If[!TrueQ[Global`rumPairNativeTraceActive],";
  print "      res = HOL`Drule`Private`tryConv[c1[t]];";
  print "      Return[If[res === HOL`Drule`Private`$convFailed, c2[t], res]]";
  print "    ];";
  print "    Global`rumPairNativeOrelseDepth++;";
  print "    level = Global`rumPairNativeOrelseDepth;";
  print "    seconds = AbsoluteTiming[";
  print "      res = HOL`Drule`Private`tryConv[c1[t]];";
  print "    ][[1]];";
  print "    Print[\"RUM_PAIR_NATIVE_ORELSEC_TRY\\t\",";
  print "      Global`rumPairNativeTraceLabel, \"\\t\", level, \"\\t\",";
  print "      If[res === HOL`Drule`Private`$convFailed, \"fail\", \"hit\"],";
  print "      \"\\t\", seconds];";
  print "    If[res === HOL`Drule`Private`$convFailed,";
  print "      restSeconds = AbsoluteTiming[";
  print "        rest = Catch[c2[t], HOL`Error`holErrorTag];";
  print "      ][[1]];";
  print "      failed = MatchQ[rest, _Failure];";
  print "      Print[\"RUM_PAIR_NATIVE_ORELSEC_REST\\t\",";
  print "        Global`rumPairNativeTraceLabel, \"\\t\", level, \"\\t\",";
  print "        If[failed, \"fail\", \"hit\"], \"\\t\", restSeconds];";
  print "      Global`rumPairNativeOrelseDepth--;";
  print "      If[failed, Throw[rest, HOL`Error`holErrorTag], rest],";
  print "      Global`rumPairNativeOrelseDepth--;";
  print "      res";
  print "    ]";
  print "  ];";
  print "Clear[HOL`Drule`REPEATC];";
  print "HOL`Drule`REPEATC[c_][t_] :=";
  print "  Module[{res, rhs, next, seconds},";
  print "    seconds = AbsoluteTiming[";
  print "      res = HOL`Drule`Private`tryConv[c[t]];";
  print "    ][[1]];";
  print "    If[TrueQ[Global`rumPairNativeTraceActive],";
  print "      Print[\"RUM_PAIR_NATIVE_REPEATC_TRY\\t\",";
  print "        Global`rumPairNativeTraceLabel, \"\\t\",";
  print "        If[res === HOL`Drule`Private`$convFailed, \"fail\", \"hit\"],";
  print "        \"\\t\", seconds]";
  print "    ];";
  print "    If[res === HOL`Drule`Private`$convFailed, Return[REFL[t]]];";
  print "    rhs = concl[res][[2]];";
  print "    If[aconv[rhs, t], Return[res]];";
  print "    next = HOL`Drule`REPEATC[c][rhs];";
  print "    TRANS[res, next]";
  print "  ];";
  print "Clear[HOL`Drule`TRYCONV];";
  print "HOL`Drule`TRYCONV[c_][t_] :=";
  print "  Module[{res, seconds},";
  print "    seconds = AbsoluteTiming[";
  print "      res = HOL`Drule`Private`tryConv[c[t]];";
  print "    ][[1]];";
  print "    If[TrueQ[Global`rumPairNativeTraceActive],";
  print "      Print[\"RUM_PAIR_NATIVE_TRYCONV\\t\",";
  print "        Global`rumPairNativeTraceLabel, \"\\t\",";
  print "        If[res === HOL`Drule`Private`$convFailed, \"fail\", \"hit\"],";
  print "        \"\\t\", seconds]";
  print "    ];";
  print "    If[res === HOL`Drule`Private`$convFailed, REFL[t], res]";
  print "  ];";
  print "Global`rumPairRepeatShouldSplit[kind_, ruleThms_, ctxThms_, depth_, t_] :=";
  print "  Module[{shape, fp, ruleCount, ctxCount, match},";
  print "    If[Global`rumPairRepeatSplitActive, Return[False]];";
  print "    If[Global`rumPairRepeatSplitLimit <= 0, Return[False]];";
  print "    If[kind =!= Global`rumPairRepeatSplitKind, Return[False]];";
  print "    If[Global`rumPairRepeatSplitDone >= Global`rumPairRepeatSplitLimit,";
  print "      Return[False]];";
  print "    shape = Global`rumPairRepeatShape[t];";
  print "    fp = Global`rumPairRepeatFingerprint[t];";
  print "    ruleCount = Length[ruleThms];";
  print "    ctxCount = Length[ctxThms];";
  print "    match = fp === Global`rumPairRepeatSplitFingerprint &&";
  print "      (Global`rumPairRepeatSplitShapeFilter === \"\" ||";
  print "        shape === Global`rumPairRepeatSplitShapeFilter) &&";
  print "      (Global`rumPairRepeatSplitDepthFilter < 0 ||";
  print "        depth === Global`rumPairRepeatSplitDepthFilter) &&";
  print "      (Global`rumPairRepeatSplitRuleCountFilter < 0 ||";
  print "        ruleCount === Global`rumPairRepeatSplitRuleCountFilter) &&";
  print "      (Global`rumPairRepeatSplitContextCountFilter < 0 ||";
  print "        ctxCount === Global`rumPairRepeatSplitContextCountFilter);";
  print "    If[!match, Return[False]];";
  print "    Global`rumPairRepeatSplitSeen++;";
  print "    If[Global`rumPairRepeatSplitSeen < Global`rumPairRepeatSplitMatchIndex,";
  print "      Return[False]];";
  print "    Global`rumPairRepeatSplitDone++;";
  print "    Print[\"RUM_PAIR_SPLIT_MATCH\\t\", kind, \"\\t\",";
  print "      Global`rumPairRepeatSplitSeen, \"\\t\", shape, \"\\t\", depth,";
  print "      \"\\t\", ruleCount, \"\\t\", ctxCount, \"\\t\", fp];";
  print "    True";
  print "  ];";
  print "Global`rumPairRepeatSplitTopRepeatcStep[label_, baseConv_, ruleThms_, ctxThms_, depth_, t_] :=";
  print "  Module[{pieces, topTh, detailTh, rhs1, descTh, result},";
  print "    Global`rumPairRepeatSplitActive = True;";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-input\", \"\\t\",";
  print "      Global`rumPairRepeatShape[t], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[t]];";
  print "    topTh = Global`rumPairRepeatTimed[label <> \"-top-repeatc-original\",";
  print "      HOL`Drule`TRYCONV[HOL`Drule`REPEATC[baseConv]][t]];";
  print "    pieces = Global`rumPairRepeatTimed[label <> \"-buildBasePieces\",";
  print "      Global`rumPairRepeatBuildBasePieces[ruleThms, ctxThms, depth]];";
  print "    Global`rumPairRepeatPrintBasePieces[label, pieces];";
  print "    detailTh = Global`rumPairRepeatTimed[label <> \"-top-repeatc-detail\",";
  print "      Global`rumPairRepeatRunRepeatBase[label, pieces, t]];";
  print "    rhs1 = Global`rumPairRepeatTimed[label <> \"-rhs1\",";
  print "      HOL`Kernel`concl[topTh][[2]]];";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-rhs1\", \"\\t\",";
  print "      Global`rumPairRepeatShape[rhs1], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[rhs1]];";
  print "    descTh = Global`rumPairRepeatTimed[label <> \"-descendOnce\",";
  print "      descendOnce[ruleThms, ctxThms, depth][rhs1]];";
  print "    result = Global`rumPairRepeatTimed[label <> \"-TRANS\",";
  print "      HOL`Kernel`TRANS[topTh, descTh]];";
  print "    Global`rumPairRepeatSplitActive = False;";
  print "    result";
  print "  ];";
  print "Global`rumPairRepeatSplitTopRepeatcTraceStep[label_, baseConv_, ruleThms_, ctxThms_, depth_, t_] :=";
  print "  Module[{pieces, topTh, detailTh, rightAssocTh, rhs1, descTh, result},";
  print "    Global`rumPairRepeatSplitActive = True;";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-input\", \"\\t\",";
  print "      Global`rumPairRepeatShape[t], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[t]];";
  print "    topTh = Global`rumPairRepeatTimed[label <> \"-top-repeatc-original\",";
  print "      HOL`Drule`TRYCONV[HOL`Drule`REPEATC[baseConv]][t]];";
  print "    pieces = Global`rumPairRepeatTimed[label <> \"-buildBasePieces\",";
  print "      Global`rumPairRepeatBuildBasePieces[ruleThms, ctxThms, depth]];";
  print "    Global`rumPairRepeatPrintBasePieces[label, pieces];";
  print "    detailTh = Global`rumPairRepeatTimed[label <> \"-top-repeatc-detail\",";
  print "      Global`rumPairRepeatRunRepeatBase[label, pieces, t]];";
  print "    rightAssocTh = Global`rumPairRepeatTimed[";
  print "      label <> \"-top-repeatc-rightassoc\",";
  print "      Global`rumPairRepeatRunRepeatRightAssoc[label, pieces, t]];";
  print "    rhs1 = Global`rumPairRepeatTimed[label <> \"-rhs1\",";
  print "      HOL`Kernel`concl[topTh][[2]]];";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-rhs1\", \"\\t\",";
  print "      Global`rumPairRepeatShape[rhs1], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[rhs1]];";
  print "    descTh = Global`rumPairRepeatTimed[label <> \"-descendOnce\",";
  print "      descendOnce[ruleThms, ctxThms, depth][rhs1]];";
  print "    result = Global`rumPairRepeatTimed[label <> \"-TRANS\",";
  print "      HOL`Kernel`TRANS[topTh, descTh]];";
  print "    Global`rumPairRepeatSplitActive = False;";
  print "    result";
  print "  ];";
  print "Global`rumPairRepeatSplitTopRepeatcNativeTraceStep[label_, baseConv_, ruleThms_, ctxThms_, depth_, t_] :=";
  print "  Module[{pieces, topTh, rhs1, descTh, result},";
  print "    Global`rumPairRepeatSplitActive = True;";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-input\", \"\\t\",";
  print "      Global`rumPairRepeatShape[t], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[t]];";
  print "    pieces = Global`rumPairRepeatTimed[label <> \"-buildBasePieces\",";
  print "      Global`rumPairRepeatBuildBasePieces[ruleThms, ctxThms, depth]];";
  print "    Global`rumPairRepeatPrintBasePieces[label, pieces];";
  print "    Global`rumPairNativeTraceActive = True;";
  print "    Global`rumPairNativeTraceLabel = label;";
  print "    Global`rumPairNativeOrelseDepth = 0;";
  print "    topTh = Global`rumPairRepeatTimed[label <> \"-top-repeatc-native\",";
  print "      HOL`Drule`TRYCONV[HOL`Drule`REPEATC[baseConv]][t]];";
  print "    Global`rumPairNativeTraceActive = False;";
  print "    Global`rumPairNativeTraceLabel = \"\";";
  print "    Global`rumPairNativeOrelseDepth = 0;";
  print "    rhs1 = Global`rumPairRepeatTimed[label <> \"-rhs1\",";
  print "      HOL`Kernel`concl[topTh][[2]]];";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-rhs1\", \"\\t\",";
  print "      Global`rumPairRepeatShape[rhs1], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[rhs1]];";
  print "    descTh = Global`rumPairRepeatTimed[label <> \"-descendOnce\",";
  print "      descendOnce[ruleThms, ctxThms, depth][rhs1]];";
  print "    result = Global`rumPairRepeatTimed[label <> \"-TRANS\",";
  print "      HOL`Kernel`TRANS[topTh, descTh]];";
  print "    Global`rumPairRepeatSplitActive = False;";
  print "    result";
  print "  ];";
  print "Global`rumPairRepeatSplitSimpStep[label_, baseConv_, ruleThms_, ctxThms_, depth_, t_] :=";
  print "  Module[{topTh, rhs1, descTh, result},";
  print "    Global`rumPairRepeatSplitActive = True;";
  print "    topTh = Global`rumPairRepeatTimed[label <> \"-top-repeatc\",";
  print "      Global`rumPairRepeatRunTopRepeatc[baseConv, ruleThms,";
  print "        ctxThms, depth, t]];";
  print "    rhs1 = Global`rumPairRepeatTimed[label <> \"-rhs1\",";
  print "      HOL`Kernel`concl[topTh][[2]]];";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-rhs1\", \"\\t\",";
  print "      Global`rumPairRepeatShape[rhs1], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[rhs1]];";
  print "    descTh = Global`rumPairRepeatTimed[label <> \"-descendOnce\",";
  print "      descendOnce[ruleThms, ctxThms, depth][rhs1]];";
  print "    result = Global`rumPairRepeatTimed[label <> \"-TRANS\",";
  print "      HOL`Kernel`TRANS[topTh, descTh]];";
  print "    Global`rumPairRepeatSplitActive = False;";
  print "    result";
  print "  ];";
  print "Global`rumPairRepeatPrototypeSimpStep[baseConv_, ruleThms_, ctxThms_, depth_][t_] :=";
  print "  Module[{topTh, rhs1, descTh},";
  print "    topTh = Global`rumPairRepeatRunTopRepeatc[baseConv, ruleThms,";
  print "      ctxThms, depth, t];";
  print "    rhs1 = concl[topTh][[2]];";
  print "    descTh = descendOnce[ruleThms, ctxThms, depth][rhs1];";
  print "    TRANS[topTh, descTh]";
  print "  ];";
  print "Global`rumPairRepeatSplitSimpFixpoint[label_, ruleThms_, ctxThms_, depth_, t_] :=";
  print "  Module[{baseConv, stepConv, cur, rhs, next, newRhs, seen,";
  print "          loopIterations, termination},";
  print "    Global`rumPairRepeatSplitActive = True;";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-input\", \"\\t\",";
  print "      Global`rumPairRepeatShape[t], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[t]];";
  print "    baseConv = Global`rumPairRepeatTimed[label <> \"-buildBaseConv\",";
  print "      buildBaseConv[ruleThms, ctxThms, depth]];";
  print "    stepConv = simpStepConv[baseConv, ruleThms, ctxThms, depth];";
  print "    cur = Global`rumPairRepeatTimed[label <> \"-initial-step\",";
  print "      stepConv[t]];";
  print "    rhs = Global`rumPairRepeatTimed[label <> \"-initial-rhs\",";
  print "      HOL`Kernel`concl[cur][[2]]];";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-initial-rhs\",";
  print "      \"\\t\", Global`rumPairRepeatShape[rhs], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[rhs]];";
  print "    seen = <|t -> True, rhs -> True|>;";
  print "    loopIterations = 0;";
  print "    termination = \"unknown\";";
  print "    While[True,";
  print "      loopIterations++;";
  print "      next = Global`rumPairRepeatTimed[";
  print "        label <> \"-loop\" <> ToString[loopIterations] <> \"-step\",";
  print "        stepConv[rhs]];";
  print "      newRhs = Global`rumPairRepeatTimed[";
  print "        label <> \"-loop\" <> ToString[loopIterations] <> \"-rhs\",";
  print "        HOL`Kernel`concl[next][[2]]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\",";
  print "        label <> \"-loop\" <> ToString[loopIterations] <> \"-rhs\",";
  print "        \"\\t\", Global`rumPairRepeatShape[newRhs], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[newRhs]];";
  print "      If[newRhs === rhs, termination = \"same\"; Break[]];";
  print "      If[KeyExistsQ[seen, newRhs], termination = \"cycle\"; Break[]];";
  print "      cur = Global`rumPairRepeatTimed[";
  print "        label <> \"-loop\" <> ToString[loopIterations] <> \"-TRANS\",";
  print "        HOL`Kernel`TRANS[cur, next]];";
  print "      rhs = newRhs;";
  print "      seen[newRhs] = True";
  print "    ];";
  print "    Print[\"RUM_PAIR_SPLIT_LOOP\\t\", label, \"\\t\",";
  print "      loopIterations, \"\\t\", termination];";
  print "    Global`rumPairRepeatSplitActive = False;";
  print "    cur";
  print "  ];";
  print "Global`rumPairRepeatSplitDescendOnce[label_, ruleThms_, ctxThms_, depth_, t_] :=";
  print "  Module[{result, bty, body, origin, forbidden, name, v, openTh,";
  print "          opened, innerEq, p1Tm, q1Tm, eq1, p2Tm, eq2, q2Tm,";
  print "          dischEq2, conjAx, p1V, p2V, q1V, q2V, instLemma,";
  print "          fTm, xTm, fEq, xEq, f2Tm, x2Tm},";
  print "    Global`rumPairRepeatSplitActive = True;";
  print "    Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-input\", \"\\t\",";
  print "      Global`rumPairRepeatShape[t], \"\\t\",";
  print "      Global`rumPairRepeatFingerprint[t]];";
  print "    Which[";
  print "    isAndForm[t],";
  print "      p1Tm = t[[1, 2]];";
  print "      q1Tm = t[[2]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-and-p1\",";
  print "        \"\\t\", Global`rumPairRepeatShape[p1Tm], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[p1Tm]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-and-q1\",";
  print "        \"\\t\", Global`rumPairRepeatShape[q1Tm], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[q1Tm]];";
  print "      eq1 = Global`rumPairRepeatTimed[label <> \"-and-p-simpFixpoint\",";
  print "        simpFixpointConv[ruleThms, ctxThms, depth][p1Tm]];";
  print "      p2Tm = Global`rumPairRepeatTimed[label <> \"-and-p-rhs\",";
  print "        concl[eq1][[2]]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-and-p2\",";
  print "        \"\\t\", Global`rumPairRepeatShape[p2Tm], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[p2Tm]];";
  print "      eq2 = Global`rumPairRepeatTimed[label <> \"-and-q-simpFixpoint\",";
  print "        simpFixpointConv[ruleThms, Append[ctxThms, ASSUME[p2Tm]],";
  print "          depth][q1Tm]];";
  print "      q2Tm = Global`rumPairRepeatTimed[label <> \"-and-q-rhs\",";
  print "        concl[eq2][[2]]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-and-q2\",";
  print "        \"\\t\", Global`rumPairRepeatShape[q2Tm], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[q2Tm]];";
  print "      dischEq2 = Global`rumPairRepeatTimed[label <> \"-and-DISCH\",";
  print "        HOL`Bool`DISCH[p2Tm, eq2]];";
  print "      conjAx = Global`rumPairRepeatTimed[label <> \"-and-CONJ\",";
  print "        HOL`Bool`CONJ[eq1, dischEq2]];";
  print "      p1V = Global`rumPairRepeatTimed[label <> \"-and-var-p1\",";
  print "        mkVar[\"p1\", boolTy]];";
  print "      p2V = Global`rumPairRepeatTimed[label <> \"-and-var-p2\",";
  print "        mkVar[\"p2\", boolTy]];";
  print "      q1V = Global`rumPairRepeatTimed[label <> \"-and-var-q1\",";
  print "        mkVar[\"q1\", boolTy]];";
  print "      q2V = Global`rumPairRepeatTimed[label <> \"-and-var-q2\",";
  print "        mkVar[\"q2\", boolTy]];";
  print "      instLemma = Global`rumPairRepeatTimed[label <> \"-and-INST\",";
  print "        INST[{p1V -> p1Tm, p2V -> p2Tm, q1V -> q1Tm, q2V -> q2Tm},";
  print "          andCongLemma[]]];";
  print "      result = Global`rumPairRepeatTimed[label <> \"-and-MP\",";
  print "        HOL`Bool`MP[instLemma, conjAx]],";
  print "";
  print "    MatchQ[t, comb[_, _]],";
  print "      fTm = t[[1]];";
  print "      xTm = t[[2]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-comb-f1\",";
  print "        \"\\t\", Global`rumPairRepeatShape[fTm], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[fTm]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-comb-x1\",";
  print "        \"\\t\", Global`rumPairRepeatShape[xTm], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[xTm]];";
  print "      fEq = Global`rumPairRepeatTimed[label <> \"-comb-f-simpFixpoint\",";
  print "        simpFixpointConv[ruleThms, ctxThms, depth][fTm]];";
  print "      f2Tm = Global`rumPairRepeatTimed[label <> \"-comb-f-rhs\",";
  print "        concl[fEq][[2]]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-comb-f2\",";
  print "        \"\\t\", Global`rumPairRepeatShape[f2Tm], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[f2Tm]];";
  print "      xEq = Global`rumPairRepeatTimed[label <> \"-comb-x-simpFixpoint\",";
  print "        simpFixpointConv[ruleThms, ctxThms, depth][xTm]];";
  print "      x2Tm = Global`rumPairRepeatTimed[label <> \"-comb-x-rhs\",";
  print "        concl[xEq][[2]]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-comb-x2\",";
  print "        \"\\t\", Global`rumPairRepeatShape[x2Tm], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[x2Tm]];";
  print "      result = Global`rumPairRepeatTimed[label <> \"-comb-MKCOMB\",";
  print "        MKCOMB[fEq, xEq]],";
  print "";
  print "    MatchQ[t, abs[bvar[0, _], _, _]],";
  print "      bty = t[[1, 2]];";
  print "      body = t[[2]];";
  print "      origin = t[[3]];";
  print "      forbidden = Global`rumPairRepeatTimed[label <> \"-abs-freesIn\",";
  print "        First /@ freesIn[body]];";
  print "      name = Global`rumPairRepeatTimed[label <> \"-abs-pickFresh\",";
  print "        pickFreshNm[origin, forbidden]];";
  print "      v = Global`rumPairRepeatTimed[label <> \"-abs-mkVar\",";
  print "        mkVar[name, bty]];";
  print "      openTh = Global`rumPairRepeatTimed[label <> \"-abs-BETA-open\",";
  print "        BETA[mkComb[abs[bvar[0, bty], body, name], v]]];";
  print "      opened = Global`rumPairRepeatTimed[label <> \"-abs-opened\",";
  print "        concl[openTh][[2]]];";
  print "      Print[\"RUM_PAIR_SPLIT_SHAPE\\t\", label <> \"-abs-opened\",";
  print "        \"\\t\", Global`rumPairRepeatShape[opened], \"\\t\",";
  print "        Global`rumPairRepeatFingerprint[opened]];";
  print "      innerEq = Global`rumPairRepeatTimed[";
  print "        label <> \"-abs-inner-simpFixpoint\",";
  print "        simpFixpointConv[ruleThms, ctxThms, depth][opened]];";
  print "      result = Global`rumPairRepeatTimed[label <> \"-abs-ABS\",";
  print "        ABS[v, innerEq]],";
  print "";
  print "    True,";
  print "      result = Global`rumPairRepeatTimed[label <> \"-original\",";
  print "        descendOnceOriginal[ruleThms, ctxThms, depth][t]]";
  print "    ];";
  print "    Global`rumPairRepeatSplitActive = False;";
  print "    result";
  print "  ];";
  print "Global`rumPairRepeatPrintSummary[] :=";
  print "  Module[{keys, k, v},";
  print "    keys = Keys[Global`rumPairRepeatStats];";
  print "    Print[\"RUM_PAIR_TOP_CACHE\\tprototype\\t\",";
  print "      Global`rumPairRepeatPrototype, \"\\tentries\\t\",";
  print "      Length[Keys[Global`rumPairRepeatTopCache]], \"\\thits\\t\",";
  print "      Global`rumPairRepeatTopCacheHits, \"\\tstores\\t\",";
  print "      Global`rumPairRepeatTopCacheStores, \"\\tchanged\\t\",";
  print "      Global`rumPairRepeatTopCacheChanged, \"\\tskipped\\t\",";
  print "      Global`rumPairRepeatTopCacheSkipped];";
  print "    Print[\"RUM_PAIR_REPEAT rows=\", Length[keys]];";
  print "    Do[";
  print "      v = Global`rumPairRepeatStats[k];";
  print "      Print[\"RUM_PAIR_REPEAT_ROW\\t\", k, \"\\t\", v[[1]], \"\\t\", v[[2]]];";
  print "      , {k, keys}]";
  print "  ];";
  print "";
  print "descendOnce[ruleThms_, ctxThms_, depth_][t_] :=";
  print "  Module[{seconds, result},";
  print "    If[Global`rumPairRepeatShouldSplit[\"descendOnce\",";
  print "        ruleThms, ctxThms, depth, t],";
  print "      seconds = AbsoluteTiming[";
  print "        result = Global`rumPairRepeatSplitDescendOnce[";
  print "          \"descendOnce-\" <> ToString[Global`rumPairRepeatSplitDone],";
  print "          ruleThms, ctxThms, depth, t];";
  print "        ][[1]],";
  print "      seconds = AbsoluteTiming[";
  print "        result = descendOnceOriginal[ruleThms, ctxThms, depth][t];";
  print "        ][[1]]";
  print "    ];";
  print "    Global`rumPairRepeatRecord[\"descendOnce\", ruleThms, ctxThms, depth, t, seconds];";
  print "    result";
  print "  ];";
  print "simpStepConv[baseConv_, ruleThms_, ctxThms_, depth_][t_] :=";
  print "  Which[";
  print "    Global`rumPairRepeatShouldSplit[\"topRepeatcNativeTrace\",";
  print "        ruleThms, ctxThms, depth, t],";
  print "      Global`rumPairRepeatSplitTopRepeatcNativeTraceStep[";
  print "        \"topRepeatcNativeTrace-\" <> ToString[Global`rumPairRepeatSplitDone],";
  print "        baseConv, ruleThms, ctxThms, depth, t],";
  print "    Global`rumPairRepeatShouldSplit[\"topRepeatcTrace\",";
  print "        ruleThms, ctxThms, depth, t],";
  print "      Global`rumPairRepeatSplitTopRepeatcTraceStep[";
  print "        \"topRepeatcTrace-\" <> ToString[Global`rumPairRepeatSplitDone],";
  print "        baseConv, ruleThms, ctxThms, depth, t],";
  print "    Global`rumPairRepeatShouldSplit[\"topRepeatc\",";
  print "        ruleThms, ctxThms, depth, t],";
  print "      Global`rumPairRepeatSplitTopRepeatcStep[";
  print "        \"topRepeatc-\" <> ToString[Global`rumPairRepeatSplitDone],";
  print "        baseConv, ruleThms, ctxThms, depth, t],";
  print "    Global`rumPairRepeatShouldSplit[\"simpStepConv\",";
  print "        ruleThms, ctxThms, depth, t],";
  print "      Global`rumPairRepeatSplitSimpStep[";
  print "        \"simpStepConv-\" <> ToString[Global`rumPairRepeatSplitDone],";
  print "        baseConv, ruleThms, ctxThms, depth, t],";
  print "    Global`rumPairRepeatTrackStep,";
  print "      Module[{seconds, result},";
  print "        seconds = AbsoluteTiming[";
  print "          result = If[Global`rumPairRepeatFailureCachePrototypeQ[],";
  print "            Global`rumPairRepeatPrototypeSimpStep[baseConv,";
  print "              ruleThms, ctxThms, depth][t],";
  print "            simpStepConvOriginal[baseConv, ruleThms, ctxThms, depth][t]];";
  print "          ][[1]];";
  print "        Global`rumPairRepeatRecord[\"simpStepConv\", ruleThms, ctxThms, depth, t, seconds];";
  print "        result";
  print "      ],";
  print "    True,";
  print "      If[Global`rumPairRepeatFailureCachePrototypeQ[],";
  print "        Global`rumPairRepeatPrototypeSimpStep[baseConv,";
  print "          ruleThms, ctxThms, depth][t],";
  print "        simpStepConvOriginal[baseConv, ruleThms, ctxThms, depth][t]]";
  print "  ];";
  print "simpFixpointConv[ruleThms_, ctxThms_, depth_][t_] :=";
  print "  Module[{seconds, result},";
  print "    If[Global`rumPairRepeatShouldSplit[\"simpFixpointConv\",";
  print "        ruleThms, ctxThms, depth, t],";
  print "      seconds = AbsoluteTiming[";
  print "        result = Global`rumPairRepeatSplitSimpFixpoint[";
  print "          \"simpFixpointConv-\" <> ToString[Global`rumPairRepeatSplitDone],";
  print "          ruleThms, ctxThms, depth, t];";
  print "        ][[1]],";
  print "      seconds = AbsoluteTiming[";
  print "        result = simpFixpointConvOriginal[ruleThms, ctxThms, depth][t];";
  print "        ][[1]]";
  print "    ];";
  print "    Global`rumPairRepeatRecord[\"simpFixpointConv\", ruleThms, ctxThms, depth, t, seconds];";
  print "    result";
  print "  ];";
}
{
  if ($0 ~ /^descendOnce\[ruleThms_, ctxThms_, depth_\]\[t_\] :=/) {
    sub(/^descendOnce/, "descendOnceOriginal");
  } else if ($0 ~ /^simpStepConv\[baseConv_, ruleThms_, ctxThms_, depth_\]\[t_\] :=/) {
    sub(/^simpStepConv/, "simpStepConvOriginal");
  } else if ($0 ~ /^simpFixpointConv\[ruleThms_, ctxThms_, depth_\]\[t_\] :=/) {
    sub(/^simpFixpointConv/, "simpFixpointConvOriginal");
  }
  if (!emitted && $0 ~ /^End\[\];/) {
    emit_wrappers();
    emitted = 1;
  }
  print;
}
END {
  if (!emitted) {
    emit_wrappers();
  }
}
' "${simp_src}" > "${instrumented_simp}"

awk -v target="${target}" '
{
  print;
  if (target == "first-convrule" && $0 ~ /HOL`Auto`Simp`simpConv\[\{mkPairDefThm\}\], lhsAppXY\];/) {
    print "    Print[\"RUM_PAIR_REPEAT first_convrule_is_thm=\", HOL`Kernel`isThm[simplifiedEq]];";
    print "    Print[\"RUM_PAIR_REPEAT first_convrule_changed=\", HOL`Kernel`concl[simplifiedEq][[2]] =!= lhsAppXY];";
    print "    Global`rumPairRepeatPrintSummary[];";
    print "    Exit[0];";
  }
}
' "${pair_src}" > "${instrumented_pair}"

cat > "${driver_wls}" <<EOF
Get["ErrorUtil.wl"];
Get["Types.wl"];
Get["Terms.wl"];
Get["Kernel.wl"];
Get["Bootstrap.wl"];
Get["Equal.wl"];
Get["Bool.wl"];
Get["Drule.wl"];
Get["Tactics.wl"];
Get["auto/PropTaut.wl"];
Get["auto/Meson.wl"];
Get["${instrumented_simp}"];
Get["${instrumented_pair}"];
Print["RUM_PAIR_REPEAT full_load_mkPairInjThm_is_thm=", HOL\`Kernel\`isThm[HOL\`Stdlib\`Pair\`mkPairInjThm]];
Print["RUM_PAIR_REPEAT full_load_pairInjThm_is_thm=", HOL\`Kernel\`isThm[HOL\`Stdlib\`Pair\`pairInjThm]];
Global\`rumPairRepeatPrintSummary[];
Null
EOF

printf 'probe\tengine\ttarget\tstatus\texit_code\telapsed_seconds\tstdout\tstderr\trows\ttop_time\ttop_calls\n' > "${summary_file}"

echo "[rum-pair-repeat] mma-hol: ${mma_hol_root}"
echo "[rum-pair-repeat] engine: ${engine}; target: ${target}; track_step: ${track_step}; timeout: ${timeout_seconds}s"
if [[ -n "${split_kind}" ]]; then
  echo "[rum-pair-repeat] split: ${split_kind}; fingerprint: ${split_fingerprint}; limit: ${split_limit}; match-index: ${split_match_index}"
fi
echo "[rum-pair-repeat] logs: ${output_dir}"

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
elif [[ "${exit_code}" -eq 124 ]]; then
  status=timeout
else
  status=fail
fi

awk -F '\t' '$1 == "RUM_PAIR_REPEAT_ROW" { print }' "${stdout_file}" > "${rows_file}" || true
if [[ -s "${rows_file}" ]]; then
  awk -F '\t' '
    function seconds(raw, parts) {
      sub(/^.*N\[/, "", raw);
      sub(/\]$/, "", raw);
      if (raw ~ /\//) {
        split(raw, parts, "/");
        return parts[1] / parts[2];
      }
      return raw + 0;
    }
    $1 == "RUM_PAIR_REPEAT_ROW" { print seconds($9) "\t" $0 }
  ' "${rows_file}" | sort -t $'\t' -k1,1nr | head -50 > "${top_time_file}"
  awk -F '\t' '$1 == "RUM_PAIR_REPEAT_ROW" { print $8 "\t" $0 }' \
    "${rows_file}" | sort -t $'\t' -k1,1nr | head -50 > "${top_calls_file}"
else
  : > "${top_time_file}"
  : > "${top_calls_file}"
fi

printf 'pair-simp-repetition\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
  "${engine}" "${target}" "${status}" "${exit_code}" "${elapsed}" \
  "${stdout_file}" "${stderr_file}" "${rows_file}" "${top_time_file}" \
  "${top_calls_file}" >> "${summary_file}"

echo "${status} (${elapsed}s)"
echo "[rum-pair-repeat] summary: ${summary_file}"
echo "[rum-pair-repeat] rows: ${rows_file}"
echo "[rum-pair-repeat] top by inclusive time: ${top_time_file}"
echo "[rum-pair-repeat] top by calls: ${top_calls_file}"

exit "${exit_code}"
