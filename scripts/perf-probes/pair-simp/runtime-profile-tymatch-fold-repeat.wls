Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-tymatch-fold-repeat"];
rumPairProbeLoadSimpStack[];
rumPairProbePattern = rumPairProbeTyMatchPattern[];
rumPairProbeTarget = rumPairProbeTyMatchTarget[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-tymatch-fold-repeat",
  Do[
    HOL`Kernel`Private`tyMatch[
      rumPairProbePattern,
      rumPairProbeTarget,
      <||>],
    {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["tymatch-fold-repeat"];

Null
