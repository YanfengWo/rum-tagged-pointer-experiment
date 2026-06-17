Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-vsubst-abs-repeat"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeTerm = rumPairProbeMkPairLambdaTerm[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-vsubst-abs-repeat",
  Do[HOL`Terms`vsubst[<||>, rumPairProbeTerm], {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["vsubst-abs-repeat"];

Null
