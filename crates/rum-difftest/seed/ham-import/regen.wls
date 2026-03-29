(* regen.wls
   Regenerate Ham golden files from Mathematica.
   Output format matches Ham's renderOutput:
     - Rational numbers as  5/2  (single line)
     - Strings without quotes, even inside lists
     - Lists recursively formatted
     - Everything else in InputForm

   Run from the ham-difftest directory:
       wolframscript -file regen.wls
*)

files = {
  "golden/arithmetic.txt",
  "golden/lists.txt",
  "golden/strings.txt",
  "golden/patterns.txt",
  "golden/scoping.txt",
  "golden/control.txt",
  "golden/definitions.txt",
  "golden/evaluation_control.txt",
  "golden/ordering.txt"
};

(* Recursively format a Mathematica result to match Ham's output. *)
hamFormat[expr_] :=
  Which[
    StringQ[expr],
      expr,   (* string: no surrounding quotes *)
    ListQ[expr],
      "{" <> StringRiffle[hamFormat /@ expr, ", "] <> "}",
    Head[expr] === Rational,
      ToString[Numerator[expr]] <> "/" <> ToString[Denominator[expr]],
    True,
      ToString[expr, InputForm]
  ];

processLine[line_String] :=
  Which[
    StringTrim[line] === "" || StringStartsQ[StringTrim[line], "#"],
      line,
    StringContainsQ[line, " !!! "],
      line,   (* known-diff lines: preserve unchanged *)
    StringContainsQ[line, " ||| "],
      Module[{input, expr, result},
        input  = First[StringSplit[line, " ||| ", 2]];
        expr   = Quiet[Check[ToExpression[input], $Failed]];
        result = If[expr === $Failed, "ERROR", hamFormat[expr]];
        input <> " ||| " <> result
      ],
    True, line
  ];

processFile[file_String] := Module[{lines, outLines, stream},
  Print["Processing ", file, " ..."];
  lines    = ReadList[file, String];
  outLines = processLine /@ lines;
  stream   = OpenWrite[file];
  WriteString[stream, StringRiffle[outLines, "\n"] <> "\n"];
  Close[stream];
  Print["  ", Length[Select[outLines, StringContainsQ[#, " ||| "] &]], " test lines written."]
];

processFile /@ files;
Print["All files regenerated."];
