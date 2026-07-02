# Disclaimer, Trademarks, and Implementation Notes

> DRAFT for the author's review. This is not legal advice. Before release, verify the
> statements below against the actual Wolfram product license(s) you use and, if in doubt,
> consult a qualified professional.

## Unofficial, independent project

`rum` is an **independent, unofficial, personal open-source project**. It is **not
affiliated with, sponsored by, endorsed by, certified by, or connected to Wolfram Research,
Inc.** or any of its products or services in any way.

## Trademarks

Wolfram®, Mathematica®, Wolfram Language™, *Wolfram*, *WolframKernel*, *wolframscript*, and
related names, logos, and marks are trademarks or registered trademarks of **Wolfram
Research, Inc.** Any use of these names in this project is **nominative** — solely to
describe compatibility with, or reference to, the Wolfram Language — and does **not** imply
ownership, affiliation, or endorsement. All trademarks are the property of their respective
owners.

## What `rum` is (and is not)

`rum` is a **clean, from-scratch implementation, in Rust, of a documented SUBSET of the
Wolfram Language's semantics**. It:

- contains **no Wolfram source code**, and does **not** include, embed, link against, or
  redistribute any Wolfram software, libraries, binaries, or data;
- implements only a subset of the language (see `docs/core-language-support-matrix.md`), with
  its own independent evaluator, and deliberately diverges where documented (e.g. exact
  rationals only; a "folk"/good-enough policy for some functions);
- is **not** a drop-in replacement for, nor a clone of, any Wolfram product.

## Implementation method

`rum`'s behavior is derived from:

1. the **publicly available Wolfram Language documentation**, and
2. **black-box behavioral observation** — evaluating expressions and comparing the resulting
   outputs (differential testing).

`rum` was **not** produced by reverse-engineering, decompiling, or disassembling any Wolfram
software, and contains no material extracted from Wolfram's software internals.

## No redistributed Wolfram content

This repository does **not** ship any Wolfram-generated content — no function-documentation
text, no symbol metadata, and no kernel outputs. The optional differential-testing harness
(`rum-difftest`) is designed to use a **contributor's own, separately licensed** Wolfram
installation at test time and to keep oracle outputs **transient** (never committed). Any
function *names* referenced (e.g. in the support matrix or source) are used as API/identifier
names.

## Your responsibility when using a Wolfram product with `rum`

Using a Wolfram product — for example, as a differential-testing oracle for `rum` — is
governed by **that product's own license agreement (EULA)**. If you run such tests, **you are
responsible for ensuring your own use complies with your Wolfram license.** Nothing here
grants any rights in Wolfram software.

---

*If you are the author preparing a release:* review this file against your Wolfram EULA (in
particular any clauses on benchmarking, developing compatible/competing software, and
permitted use), then remove this note and adjust the wording as appropriate.
