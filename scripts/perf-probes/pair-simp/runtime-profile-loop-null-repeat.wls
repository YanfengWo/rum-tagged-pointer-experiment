Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-loop-null-repeat"];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-loop-null-repeat",
  Do[Null, {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["loop-null-repeat"];

Null
