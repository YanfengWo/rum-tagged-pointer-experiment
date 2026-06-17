Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-fold-callback-repeat"];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-fold-callback-repeat",
  Do[Fold[#1 &, 0, {1, 2}], {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["fold-callback-repeat"];

Null
