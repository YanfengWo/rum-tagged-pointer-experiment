Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-proptaut-boolvars-p-or-notp"];
rumPairProbeLoadSimpStack[];
rumPairProbeP = rumPairProbeBoolVar["p"];
rumPairProbeTerm =
  HOL`Auto`PropTaut`Private`ptOr[
    rumPairProbeP,
    HOL`Auto`PropTaut`Private`ptNot[rumPairProbeP]];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-proptaut-boolvars-p-or-notp",
  HOL`Auto`PropTaut`Private`propTautBoolVars[rumPairProbeTerm],
  rumPairProbePrintThmResult[
    "profile-proptaut-boolvars-p-or-notp",
    rumPairProbeTerm]];

RumProfileSnapshot["proptaut-boolvars-p-or-notp"];

Null
