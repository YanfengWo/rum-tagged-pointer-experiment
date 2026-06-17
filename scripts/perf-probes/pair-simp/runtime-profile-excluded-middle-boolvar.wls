Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-excluded-middle-boolvar"];
rumPairProbeLoadSimpStack[];
rumPairProbeP = rumPairProbeBoolVar["p"];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-excluded-middle-boolvar",
  HOL`Bool`EXCLUDEDMIDDLE[rumPairProbeP],
  rumPairProbePrintThmResult["profile-excluded-middle-boolvar", rumPairProbeP]];

RumProfileSnapshot["excluded-middle-boolvar"];

Null
