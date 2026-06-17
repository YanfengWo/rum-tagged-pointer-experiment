Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-proptaut-free-bool-vars-schema"];
rumPairProbeLoadSimpStack[];
rumPairProbeTerm = rumPairProbeImpCongSchema[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-proptaut-free-bool-vars-schema",
  Do[
    HOL`Auto`PropTaut`Private`propFreeBoolVars[rumPairProbeTerm],
    {i, 1, 1000}],
  Function[{value}, Null]];

RumProfileSnapshot["proptaut-free-bool-vars-schema"];

Null
