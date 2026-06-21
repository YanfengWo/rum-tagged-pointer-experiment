Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-var-basic-baseconv-chain-segmented"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];

rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

RumProfileReset[];
rumPairProbeBasicOnlyRules = rumPairProbeRules["basic"];

rumPairProbeMeasure[
  "var-basic-ruleConvs-build",
  (
    rumPairProbePreparedRules =
      HOL`Auto`Simp`simpPrepareRule /@ rumPairProbeBasicOnlyRules;
    rumPairProbeProductiveRules =
      Select[
        rumPairProbePreparedRules,
        HOL`Auto`Simp`Private`productiveSimpRule];
    rumPairProbeProver =
      HOL`Auto`Simp`Private`makeProver[
        rumPairProbeBasicOnlyRules, {},
        HOL`Auto`Simp`Private`$simpDepthLimit];
    rumPairProbeRuleConvs =
      (HOL`Auto`Simp`Private`simpRuleConv[#, rumPairProbeProver] &) /@
        rumPairProbeProductiveRules
  ),
  Function[{value},
    Print["RUM_PAIR_PROBE var-basic-rule-count=",
      Length[rumPairProbeBasicOnlyRules]];
    Print["RUM_PAIR_PROBE var-basic-productive-count=",
      Length[rumPairProbeProductiveRules]];
    Null]];
RumProfileSnapshot["var-basic-ruleConvs-build"];

RumProfileReset[];
rumPairProbeMeasure[
  "var-basic-combineSimpConvs-build",
  rumPairProbeBaseConv =
    HOL`Auto`Simp`Private`combineSimpConvs[rumPairProbeRuleConvs],
  Function[{value}, Null]];
RumProfileSnapshot["var-basic-combineSimpConvs-build"];

RumProfileReset[];
rumPairProbeMeasure[
  "var-basic-baseConv-top-TRYCONV-REPEATC",
  rumPairProbeBaseTopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeBaseConv]][rumPairProbeXV],
  rumPairProbePrintThmResult[
    "var-basic-baseConv-top-TRYCONV-REPEATC",
    rumPairProbeXV]];
RumProfileSnapshot["var-basic-baseConv-top-TRYCONV-REPEATC"];

RumProfileReset[];
rumPairProbeMeasure[
  "var-basic-ruleConvs-loop-tryConv",
  Module[{i = 1, res = HOL`Drule`Private`$convFailed},
    rumPairProbeRuleLoopFailures = 0;
    While[i <= Length[rumPairProbeRuleConvs],
      res = HOL`Drule`Private`tryConv[
        rumPairProbeRuleConvs[[i]][rumPairProbeXV]];
      If[res =!= HOL`Drule`Private`$convFailed, Break[]];
      rumPairProbeRuleLoopFailures++;
      i++];
    res
  ],
  Function[{value},
    Print["RUM_PAIR_PROBE var-basic-rule-loop failures=",
      rumPairProbeRuleLoopFailures];
    Print["RUM_PAIR_PROBE var-basic-rule-loop failed=",
      value === HOL`Drule`Private`$convFailed];
    Null]];
RumProfileSnapshot["var-basic-ruleConvs-loop-tryConv"];

Null
