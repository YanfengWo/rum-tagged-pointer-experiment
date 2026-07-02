#!/usr/bin/env wolframscript
(* Export a Wolfram Language builtin-symbol inventory with rich per-symbol
   metadata (FunctionalityAreas grouping, usage, guide pages, versions) to JSON,
   for the "radical route" core-language scoping audit.

   Method + review policy: docs/wolfram-builtin-inventory.md. Outputs are
   generated data (WLBuiltin/, local review artifact — not authoritative source).
   First run may download Wolfram entity metadata (needs network / unsandboxed).

   Run:  WolframKernel -noprompt -script scripts/export-wl-builtins.wls
     or: wolframscript -file scripts/export-wl-builtins.wls
*)

syms = EntityList["WolframLanguageSymbol"];

props = {
  "Name",
  "FunctionalityAreas",
  "SubjectClassifications",
  "RelatedGuidePages",
  "EntityClasses",
  "PlaintextUsage",
  "URL",
  "VersionIntroduced",
  "VersionLastModified"
};

ClearAll[jsonClean];
jsonClean[x_] := Which[
  MissingQ[x], Null,
  MatchQ[x, _Hyperlink],
    With[{args = List @@ x},
      <|
        "label" -> ToString[args[[1]], InputForm],
        "target" -> ToString[args[[2]], InputForm]
      |>
    ],
  MatchQ[x, _Entity | _EntityClass], ToString[x, InputForm],
  AssociationQ[x], Association @ KeyValueMap[#1 -> jsonClean[#2] &, x],
  ListQ[x], jsonClean /@ x,
  True, x
];

rows = jsonClean /@ EntityValue[syms, props, "PropertyAssociation"];

CreateDirectory["WLBuiltin", CreateIntermediateDirectories -> True];

Export[
  FileNameJoin[{"WLBuiltin", "wolfram-language-symbols.json"}],
  rows,
  "RawJSON"
];

areas = DeleteDuplicates @ Sort @ Flatten @ DeleteMissing[
  Lookup[rows, "FunctionalityAreas", {}],
  Infinity
];

byArea = AssociationMap[
  Function[area,
    Lookup[
      Select[rows, MemberQ[Lookup[#, "FunctionalityAreas", {}], area] &],
      "Name"
    ]
  ],
  areas
];

Export[
  FileNameJoin[{"WLBuiltin", "wolfram-language-groups.json"}],
  byArea,
  "RawJSON"
];

Print["symbols: ", Length[rows], "  areas: ", Length[areas]];
