# Wolfram Function Repository core-language survey (2026-07-07)

What WFR is: community-contributed single functions (individual
authors), reviewed and published by Wolfram with documentation AND
visible source code, used via `ResourceFunction`. It is third-party
code under repository terms — NOT open-source licensed.

**Absorption rule (stricter than the official-docs rule):** idea-level
only. We may read entry names, summaries, and usage signatures (public
functional facts); we NEVER open or read an entry's source-code
section. Anything absorbed is an independent folk implementation from
the described behavior. Names like `Duplicates` or `BinarySearch` are
generic English, not product identifiers.

Context: several once-famous WFR entries later graduated into WL
builtins (`Until`, `Enclose`, `WithCleanup` — rum already has all
three), so WFR is a useful DEMAND SIGNAL for what scripting users
reach for. The category listing pages are JS-rendered (not
scrapeable); this survey verified a knowledge-derived shortlist
against the individual entry pages instead — it is a high-signal
sample, not an exhaustive catalog sweep.

## Verified shortlist (entry → what it does → rum fit)

Structure/functional (fold into UNIT-2 fix batches):

- `SelectPositions[list, crit(, n)]` (+ operator form) — positions of
  elements passing a test; Select's position-flavored sibling. Tiny.
- `Duplicates[list]` — elements occurring ≥2 times, first-appearance
  order. Pairs with Tally/GatherBy. Tiny.
- `BinarySearch[sortedList, val]` — last position ≤ val, 0 if none;
  documented no-validation contract. Already parked once by the
  Python-stdlib sweep — second demand signal; fail-closed variant to
  be decided at design (rum may prefer a message on unsorted-looking
  input rather than silent nonsense; folk scope).

Associations (fold into UNIT-3 fix batches):

- `KeyGroupBy[assoc, f(, red)]` (+ operator form) — GroupBy over KEYS
  with optional reduction.
- `AssociationKeyDeflatten[flatAssoc(, f)]` — flat association with
  list keys → nested association (inverse of AssociationKeyFlatten;
  audit unit 3 should check whether a flatten twin is wanted too).
  HIGH value: pairs with the codec family — nested config ↔ flat
  dotted keys.
- `MergeNested[{assoc…}, f]` (+ operator form) — recursive Merge for
  nested associations; recursion stops at the shallowest depth;
  matches Merge on flat data. HIGH value: config layering
  (defaults + overrides), composes with ConformSchema and the
  six-format codec story.

Evaluation-control (defer to UNITS 8-10 / hold-design work):

- `ReplaceAllUnheld[expr, rules]` — apply rules only to parts that
  would evaluate (respects Hold* attributes and held subexpressions).
  Kernel-adjacent; design alongside the SubValuesHoldAll-equivalent
  (see wl15-core-survey.md §1).

Not verified / dropped: `FoldWhile` (entry page redirects to auth —
name likely wrong or entry absent; WL builtins NestWhile/TakeWhile
cover most of the intent).
