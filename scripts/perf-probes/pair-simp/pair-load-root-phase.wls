Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=pair-load-root-phase"];

rumPairProbeMeasure[
  "root-preload-simp-stack",
  rumPairProbeLoadSimpStack[],
  Function[{value}, Null]];

rumPairProbeMeasure[
  "root-get-stdlib-pair",
  Get[FileNameJoin[{"stdlib", "Pair.wl"}]],
  Function[{value}, Null]];

Print[
  "RUM_PAIR_PROBE pair_mkPairInjThm_is_thm=",
  HOL`Kernel`isThm[HOL`Stdlib`Pair`mkPairInjThm]];
Print[
  "RUM_PAIR_PROBE pair_pairInjThm_is_thm=",
  HOL`Kernel`isThm[HOL`Stdlib`Pair`pairInjThm]];

Null
