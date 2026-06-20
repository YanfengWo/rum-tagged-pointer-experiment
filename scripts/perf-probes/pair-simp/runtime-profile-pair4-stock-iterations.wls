Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-pair4-stock-iterations"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeIter = 1;
While[rumPairProbeIter <= 8,
  RumProfileReset[];
  rumPairProbeIterLabel =
    StringJoin["pair4-stock-iteration-", ToString[rumPairProbeIter]];
  rumPairProbeMeasure[
    rumPairProbeIterLabel,
    rumPairProbeStockIterThm =
      HOL`Auto`Simp`Private`simpFixpointConv[
        rumPairProbeBasicRules, {},
        HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4Same],
    Function[{value},
      Print["RUM_PAIR_PROBE ", rumPairProbeIterLabel, " is_thm=",
        HOL`Kernel`isThm[rumPairProbeStockIterThm]];
      Print["RUM_PAIR_PROBE ", rumPairProbeIterLabel, " changed=",
        HOL`Kernel`concl[rumPairProbeStockIterThm][[2]] =!=
          rumPairProbePair4Same];
      Null]];
  RumProfileSnapshot[rumPairProbeIterLabel];
  rumPairProbeIter++];

Null
