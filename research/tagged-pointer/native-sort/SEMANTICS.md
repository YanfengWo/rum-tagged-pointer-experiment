# Native symbolic Sort: conditional safety and semantic argument

This package changes only temporary operands of a wide, entirely symbolic
default Sort. The language's Expr representation and custom comparator paths
remain original. No recursive graph conversion occurs.

The canonical comparator in pinned stdlib ordering_sets.rs uses
`Symbol::cmp_qualified_name`, not Symbol::Ord or numeric id ordering. Global
and System context prefixes are hidden; other context bytes precede name bytes.
Generated identities do not participate in this comparison. Stability preserves
their original order when displayed names tie, including Global/System aliases.

Symbol24 retains complete Symbol. Safe16 retains authentic permanent SymbolText
and full Option<NonZeroU64> identity. Tagged16 retains the same text pointer
with a low-bit plain/generated tag; its second word is cached plain id or the
full generated identity. All fields are private and constructors accept only
safe core Symbol values. Pointer width and text alignment are checked. Address
tagging/clearing uses map_addr and keeps the original provenance. Numeric words
are never dereferenced. Text dereference is justified by the permanent immutable
interner contract. Neither representation owns an allocation or a reference
count. Copy and destruction therefore do not acquire/release an Arc count.

Tagged16 Send/Sync relies on permanent immutable text and checked SymbolText
Send+Sync. No mutation or raw construction is exposed. The earlier general
Tagged16 owner proof is not automatically a proof of this distinct static owner.

The safe core opaque-text comparator reuses the existing concatenated-byte
helper. Text pointer equality implies equal displayed names, so its shortcut
agrees with the original id shortcut. Otherwise it compares the exact hidden
context/name byte sequence. The tagged ordinary-id shortcut is valid because
ids are unique for authentic text. The full pairwise test compares all three
representations across 72 symbols (four contexts, six names, three identity
states), including Unicode, empty names, prefix-like names and u64::MAX.

Refinement maps each temporary slot back to the original Symbol via authentic
text and full identity. Construction preserves that value; sorting changes only
the permutation under the original qualified-name comparator; stable ties and
complete output reconstruction preserve the same ordered Expr result. Admission
requires length >=64 and every element to be an Atom::Symbol. Unsupported and
small operands take the unchanged structural path. No callback evaluation or
Session/GC safepoint occurs inside the pure temporary sorting phase.

Native checks cover ordered results, admission boundaries, late mixed fallback
and cross-thread transfer. Strict-provenance Miri runs both borrowing models.
Complete language acceptance compares default Sort with the original SortBy
string-key path, checks generated/repeated/contextual operands, mixed fallback,
small operands and source preservation. Full workspace checks and per-label
language timing gate cover the actual integration.

This is a manual conditional argument and bounded evidence, not a machine proof
of all Rust contexts or targets. Its static-text and comparator contracts are
pinned to this source. It does not provide a complete general expression owner,
transparent platform fallback or a universal LLVM performance guarantee.
