Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-pair4-stock-repeat8"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

RumProfileReset[];
rumPairProbeMeasure[
  "pair4-stock-repeat8",
  Do[
    rumPairProbeStockThm =
      HOL`Auto`Simp`Private`simpFixpointConv[
        rumPairProbeBasicRules, {},
        HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4Same],
    {8}],
  Function[{value},
    Print["RUM_PAIR_PROBE pair4-stock-repeat8 repeats=", 8];
    Print["RUM_PAIR_PROBE pair4-stock-repeat8 is_thm=",
      HOL`Kernel`isThm[rumPairProbeStockThm]];
    Print["RUM_PAIR_PROBE pair4-stock-repeat8 changed=",
      HOL`Kernel`concl[rumPairProbeStockThm][[2]] =!=
        rumPairProbePair4Same];
    Null]];
RumProfileSnapshot["pair4-stock-repeat8-target"];

Null
