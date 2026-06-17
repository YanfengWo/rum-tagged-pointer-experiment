Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-tymatch-var-repeat"];
rumPairProbeLoadSimpStack[];
rumPairProbeVarPattern = HOL`Types`tyVar["A"];
rumPairProbeVarTarget = HOL`Kernel`boolTy;

RumProfileReset[];

rumPairProbeMeasure[
  "profile-tymatch-var-repeat",
  Do[
    HOL`Kernel`Private`tyMatch[
      rumPairProbeVarPattern,
      rumPairProbeVarTarget,
      <||>],
    {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["tymatch-var-repeat"];

Null
