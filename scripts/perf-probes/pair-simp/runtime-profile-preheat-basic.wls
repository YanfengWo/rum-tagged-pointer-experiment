Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-preheat-basic"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

RumProfileSnapshot["preheat-basic"];

Null
