Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-module-frame-repeat"];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-module-frame-repeat",
  Do[Module[{x}, x = i; x], {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["module-frame-repeat"];

Null
