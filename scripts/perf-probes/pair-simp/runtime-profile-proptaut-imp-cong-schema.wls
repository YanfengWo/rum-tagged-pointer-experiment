Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-proptaut-imp-cong-schema"];
rumPairProbeLoadSimpStack[];
rumPairProbeSchema = rumPairProbeImpCongSchema[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-proptaut-imp-cong-schema",
  rumPairProbeSchemaThm = HOL`Auto`PropTaut`propTaut[rumPairProbeSchema],
  Function[{value},
    Print["RUM_PAIR_PROBE profile-proptaut-imp-cong-schema is_thm=",
      HOL`Kernel`isThm[value]]]];

RumProfileSnapshot["proptaut-imp-cong-schema"];

Null
