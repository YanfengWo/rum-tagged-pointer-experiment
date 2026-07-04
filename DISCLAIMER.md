# Disclaimer, Trademarks, and Project History

## Independent project

`rum` is an independent, personal open-source project. It is **not affiliated
with, sponsored by, endorsed by, or connected to Wolfram Research, Inc.** or
any of its products or services in any way.

## What `rum` is

`rum` is an embeddable, sandbox-first symbolic term-rewriting language and
runtime written in Rust. Its surface syntax belongs to the Wolfram Language
family of notations; its evaluator implements `rum`'s **own documented
semantics** (see the language guide and builtin reference under `docs/`).

- It contains **no Wolfram source code**, and does not include, embed, link
  against, or redistribute any Wolfram software, libraries, or data.
- It makes **no compatibility claims**: it is not a drop-in replacement for,
  nor a clone of, any Wolfram product, and this repository publishes no
  compatibility matrices or conformance reports.
- It deliberately diverges where documented: exact rational arithmetic only, a
  "good-enough" policy for a number of functions, its own host/embedding API.
  It implements **no Wolfram communication protocols** (such as WSTP); host
  integrations use native Rust interfaces.

## Trademarks

Wolfram®, Mathematica®, Wolfram Language™, and related names and marks are
trademarks or registered trademarks of Wolfram Research, Inc. They are used
here **nominatively** only — to name the notation family that influenced
`rum`'s syntax and to state non-affiliation — and imply no endorsement. A few
third-party ecosystem identifiers are retained so that existing editors
highlight `rum` code sensibly (a `codemirror_mode` of `mathematica`, the `mma`
Pygments lexer name, the `.wl` file extension); they name those ecosystems'
modes, not an affiliation.

## Development method and project history

Through its early development (until July 2026), `rum`'s behavior was built
from the publicly available Wolfram Language documentation and was checked in
part by **black-box differential testing**. No reverse
engineering, decompilation, or disassembly of any Wolfram software was ever
performed, and no Wolfram-generated outputs, documentation text, or data were
ever committed to this repository or redistributed.

In July 2026, out of respect for Wolfram Research's rights and license terms,
the project **removed the differential-testing harness and all comparison
pipelines, retired its compatibility matrix, and adopted a standing policy**:
`rum`'s tests verify `rum`'s own documented behavior only, and the project
neither performs nor accepts contributions that involve compatibility testing
against Wolfram products. The complete development history, including these
removals, is preserved unrewritten in this repository's git history.

## Direction

`rum`'s roadmap centers on embedded scripting and sandboxed evaluation for
Rust hosts (and, later, other languages via native bindings). It does not aim
to track, reimplement, or compete with the Wolfram product line.

## Using Wolfram products

This project neither requires nor encourages the use of any Wolfram product.
If you use one, that use is governed by your own license agreement with
Wolfram Research. Nothing in this repository grants any rights in Wolfram
software.

If any rights holder has a concern about anything in this repository, please
contact the maintainer; it will be addressed promptly and in good faith.
