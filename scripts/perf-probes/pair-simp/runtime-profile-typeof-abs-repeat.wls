Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-typeof-abs-repeat"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeTerm = rumPairProbeMkPairLambdaTerm[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-typeof-abs-repeat",
  Do[HOL`Terms`typeOf[rumPairProbeTerm], {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["typeof-abs-repeat"];

Null
