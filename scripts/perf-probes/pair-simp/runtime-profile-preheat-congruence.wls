Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-preheat-congruence"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-preheat-congruence",
  rumPairProbePreheatCongruence[],
  Function[{value}, Null]];

RumProfileSnapshot["preheat-congruence"];

Null
