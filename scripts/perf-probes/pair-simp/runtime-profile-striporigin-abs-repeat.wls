Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-striporigin-abs-repeat"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeTerm = rumPairProbeMkPairLambdaTerm[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-striporigin-abs-repeat",
  Do[HOL`Terms`stripOrigin[rumPairProbeTerm], {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["striporigin-abs-repeat"];

Null
