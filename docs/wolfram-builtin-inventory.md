# Wolfram Builtin Inventory

This document records how to regenerate a Wolfram Language builtin-symbol
inventory if the project later wants a broad post-`CL-*` scan for useful
infrastructure functions.

The old local `WLBuiltin/` workspace was generated data, not an authoritative
source file. The authoritative source is the local Wolfram system's entity
framework, especially the `WolframLanguageSymbol` entity type.

Use this inventory as a reference input only. The current implementation
roadmap is still driven by the support matrix, real-project smoke lanes such as
`CL-Y`, and concrete migration blockers. Do not let the full builtin table turn
into an implicit commitment to clone the whole Wolfram system.

## Query

Useful discovery queries:

```wl
EntityList["WolframLanguageSymbol"]
EntityProperties["WolframLanguageSymbol"]
```

Relevant properties observed in the local Wolfram kernel include:

```wl
{
  "Name",
  "FunctionalityAreas",
  "SubjectClassifications",
  "RelatedGuidePages",
  "EntityClasses",
  "PlaintextUsage",
  "URL",
  "VersionIntroduced",
  "VersionLastModified"
}
```

The first run may download Wolfram entity metadata. In an agent sandbox this may
need unsandboxed execution approval.

## Generate JSON

Run with a local kernel, for example:

```bash
/Applications/Wolfram.app/Contents/MacOS/WolframKernel -noprompt -script scripts/export-wl-builtins.wls
```

The following script body is sufficient if the helper script does not already
exist:

```wl
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
  MissingQ[x], None,
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
```

Expected outputs:

- `WLBuiltin/wolfram-language-symbols.json`: one JSON object per builtin symbol
- `WLBuiltin/wolfram-language-groups.json`: `FunctionalityAreas` to symbol-name
  lists

## Review Policy

- Regenerate the inventory when planning a deliberate post-mainline builtin
  survey.
- Prefer real-project evidence from `CL-Y` and adapter examples over table-only
  coverage pressure.
- Keep full system features in `CL-X` unless a narrow, project-owned glue
  subset is justified by a concrete smoke lane.
- Commit generated JSON only if it is deliberately adopted as project data;
  otherwise keep it as a local review artifact.
