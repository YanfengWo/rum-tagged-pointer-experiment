# Differential Test Assets

This directory is the staging area for `rum`'s future differential and
conformance work.

If this directory is copied by itself into another checkout or environment,
start with:

- [HANDOFF.md](./HANDOFF.md)
- [seed/ham-import/MANIFEST.md](./seed/ham-import/MANIFEST.md)
- [seed/ham-import/ADAPTATION-NOTES.md](./seed/ham-import/ADAPTATION-NOTES.md)

This tree is intentionally split into two kinds of content:

- `seed/`: imported or historical material that is useful as input when
  building `rum`'s own conformance corpus
- future `cases/`: curated `rum`-owned fixtures and runners

Current seed source:

- [seed/ham-import](./seed/ham-import)

The imported `ham` files are kept as a seed pack, not as authoritative `rum`
coverage. They should be curated, normalized, and gradually migrated into
`rum`-owned fixture formats as `rum-difftest` is built out.
