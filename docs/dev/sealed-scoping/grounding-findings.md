# Sealed-scoping substrate grounding findings

Date: 2026-07-18

Scope: diagnostic only. No source was changed. All commands below were run
from the repository root with the release `rum-cli`.

## Executive verdict

The distinctness primitive exists: for equal symbol text, `Symbol.identity`
participates in equality, and evaluated rum source has no constructor for an
arbitrary non-`None` identity. The design's more specific substrate claim is
**refuted**, however. A `Module` local is `Symbol::new(context,
"name$counter")`, so it has `identity: None`; its uniqueness is carried only
by its mangled name, and a caller that learns that name can spell the same
symbol. `allocate_fresh_symbol` therefore cannot be reused as a seal.

Two other grounding discrepancies matter to later batches:

1. Core `Symbol` serde preserves an identity, but the current public host
   snapshot is textual rum source. It does not use that serde and cannot
   preserve an identity-bearing definition as an identity-bearing definition.
2. There is an `Information` builtin, but there is no current prefix
   `?symbol` inspection syntax/handler. `?` is only the binary `PatternTest`
   token today.

## Q1 — Identity provenance and the distinctness invariant

### Representation and equality

- `Symbol.identity` is `Option<NonZeroU64>` at
  `crates/rum-core/src/expr.rs:35-39`.
- `Symbol::new` sets it to `None` at
  `crates/rum-core/src/expr.rs:108-116`.
- `Symbol::with_identity` rejects zero and sets `Some(identity)` at
  `crates/rum-core/src/expr.rs:118-126`.
- equality requires both the interned text id and identity to match at
  `crates/rum-core/src/expr.rs:183-186`. The text id is shared even when
  identity differs (`crates/rum-core/src/expr.rs:164-168`). Thus two symbols
  with the same context and name but different identities are non-equal.

### Complete in-tree `with_identity` producer list

Verbatim search:

```text
rg -n 'Symbol::with_identity' crates
crates/rum-core/src/expr.rs:249:            Some(identity) => Symbol::with_identity(parts.context, parts.name, identity),
crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:527:    Symbol::with_identity(
crates/rum-runtime/src/runtime/resolution.rs:137:            .map(|identity| Symbol::with_identity(context, name, identity))
```

These are all in-tree calls to the public constructor:

| Producer | Mechanism | Identity range actually accepted/produced |
|---|---|---|
| `crates/rum-core/src/expr.rs:245-251` | Deserialization of a serialized `Symbol` | The serialized field is `Option<u64>` (`crates/rum-core/src/expr.rs:236-243`). Valid `Some` values are `1..=u64::MAX`; `Some(0)` reaches `with_identity` and panics because zero is invalid. This producer has no reserved numeric region. |
| `crates/rum-runtime/src/runtime/resolution.rs:129-139` | Active symbol for a `Remove` generation | It copies the `u64` stored in `symbol_generations`. The sole normal entry writer starts a key at 1 and writes `max(current_generation, removed_symbol.identity_or_0) + 1` (`crates/rum-runtime/src/runtime/definitions/definition_clear.rs:308-322`). Therefore the normal per-name sequence is 1, 2, ... with no reserved bits or explicit upper bound before integer exhaustion. |
| `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:525-531` | Fresh rule-RHS alpha rename | `RULE_RHS_ALPHA_ID_MARKER | id`, where the marker is `1_u64 << 63` (`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:18-20`) and `id` starts at 1. Its intended collision-free range is `0x8000000000000001..=0xffffffffffffffff` while the counter remains below `2^63`; the code does not enforce that bound, so after that point values repeat in the high-bit region. |

The public Rust constructor itself accepts every nonzero `u64`; an external
trusted host can therefore mint an identity. There are no other in-tree
producers. In particular, neither `Module` nor `Unique` occurs in the list.

### Source construction paths

A directly written source name is lowered through
`Session::resolve_symbol_name` (`crates/rum-runtime/src/runtime/resolution.rs:66-69`).
`Symbol[string]` calls the same resolver
(`crates/rum-stdlib/src/structure_ops/surface.rs:254-286`). `ToExpression`
parses, lowers through the session, and evaluates the result
(`crates/rum-stdlib/src/strings/text/builtins.rs:407-439`). For a non-System
name, `active_symbol_for_name` returns `with_identity` only when
`symbol_generations` already has the `(context, name)` key; otherwise it
returns `Symbol::new` (`crates/rum-runtime/src/runtime/resolution.rs:129-139`).
A new session initializes that map empty
(`crates/rum-runtime/src/runtime/session/session_state.rs:377-382`).

Verbatim probe of a fresh name, before any `Remove` for that name:

```text
target/release/rum-cli <<'EOF'
viaSymbol = Symbol["Grounding`fresh"]
viaToExpression = ToExpression["Grounding`fresh"]
same = {SameQ[Grounding`fresh, viaSymbol], SameQ[Grounding`fresh, viaToExpression]}
Grounding`fresh = 271
{same, Grounding`fresh, viaSymbol, viaToExpression}
EOF
List[List[True, True], 271, 271, 271]
```

The CLI does not print the private `identity` field, so the field-level proof
comes from the branches above. The observable proof is that the directly
lowered symbol and both runtime constructors are the same symbol and reach the
same binding. With the generation map empty, the only branch that can produce
that result is `Symbol::new`, hence all three have `identity: None`. There is no
rum syntax or builtin that accepts an identity number.

**Q1 verdict:** confirmed. Text and identity jointly define distinctness;
source can create only the active ordinary/Remove-generation symbol selected
by the resolver, not an arbitrary identity.

## Q2 — Is a `Module` gensym distinguished by name or identity?

`eval_module` constructs its rename map by calling `allocate_fresh_symbol`
(`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:32-40`). That calls
`allocate_generated_symbol` with a `$` separator
(`crates/rum-runtime/src/runtime/scoping/unique.rs:81-83`), and the candidate
constructor is exactly:

```text
Symbol::new(context.to_string(), format!("{name_prefix}{separator}{id}"))
```

at `crates/rum-runtime/src/runtime/scoping/unique.rs:36-43`. The counter is
incremented until the textual candidate name is unused
(`crates/rum-runtime/src/runtime/scoping/unique.rs:149-167`).

Verbatim returned-form probe:

```text
target/release/rum-cli --eval 'Module[{x}, Hold[x]]' </dev/null
Hold[x$1]
```

Verbatim reachability probe. The escaped local is given an own value, then the
exact printed `x$1` name is used directly, through `Symbol`, and through
`ToExpression`:

```text
target/release/rum-cli --eval 'escaped = Module[{x}, x = 73; Hold[x]]; {escaped, SameQ[escaped, Hold[x$1]], x$1, Symbol["Global`x$1"], ToExpression["Global`x$1"]}' </dev/null
List[Hold[x$1], True, 73, 73, 73]
```

The held local is exactly equal to directly spelled `x$1`, and all three
spelling paths reach its value. Its uniqueness is therefore carried by the
mangled name, not by `identity`.

**Q2 verdict:** the design §1 claims that a `Module` gensym has
`Some(identity)` and that `allocate_fresh_symbol` can be reused. Both claims
are **refuted**. As-is, `allocate_fresh_symbol` would create a source-reachable
private whenever its generated name is known, so it is unsuitable as a seal.

## Q3 — `symbol_generations` is the existing identity-distinctness path

`mark_symbol_removed` is the only normal writer that creates or advances a
`symbol_generations` entry. It skips System symbols, forms the textual key,
and writes one more than both the current and removed identities
(`crates/rum-runtime/src/runtime/definitions/definition_clear.rs:308-322`).
`active_symbol_for_name` then reconstructs the active identity-bearing symbol
from that entry (`crates/rum-runtime/src/runtime/resolution.rs:129-139`).

The script probe below deliberately uses separate top-level lines. Script
forms are evaluated one at a time (`crates/rum-runtime/src/runtime/session/session_eval.rs:253-288`),
so each `f` after a `Remove` is lowered after the generation map has advanced.
The first `Remove` creates generation 1. `held` captures generation 1; the
second `Remove` advances the active name to generation 2.

```text
target/release/rum-cli <<'EOF'
f = 11
Remove[f]
f = 22
held = Hold[f]
Remove[f]
f = 33
{held, Hold[f], SameQ[held, Hold[f]], f, Symbol["Global`f"], ToExpression["Global`f"]}
EOF
List[Hold[f], Hold[f], False, 33, 33, 33]
```

The two held forms print the same text but `SameQ` is `False`. The freshly
lowered direct name, `Symbol`, and `ToExpression` all select generation 2 and
reach 33. A correction to the wording in the brief is worth making explicit:
after `Remove`, the fresh symbol is not `Symbol::new`; because the generation
entry exists, it is `Symbol::with_identity(..., new_generation)`.

For a future sealed symbol, suppose the runtime creates
`Symbol::with_identity(context, name, sealed_id)`, keys definitions and other
state by that symbol, and does **not** insert `(context, name)` into
`symbol_generations`. The exact branches in `active_symbol_for_name` are:

1. System context returns `Symbol::system`
   (`crates/rum-runtime/src/runtime/resolution.rs:129-132`).
2. Otherwise, a generation-map hit returns `with_identity`
   (`crates/rum-runtime/src/runtime/resolution.rs:134-137`).
3. A miss returns `Symbol::new(context, name)`
   (`crates/rum-runtime/src/runtime/resolution.rs:138`).

The sealed symbol's presence in `definitions` does not affect step 2.
`symbol_exists` checks maps using the already constructed full `Symbol` key
(`crates/rum-runtime/src/runtime/resolution.rs:141-157`), whose equality
includes identity. Consequently a
spelled sealed name produces `identity: None`, is unequal to the sealed key,
and cannot reach the sealed definition.

**Q3 verdict:** confirmed. Keeping the sealed identity out of
`symbol_generations` is the crux; the existing miss branch already produces
the non-forging ordinary symbol.

## Q4 — Reuse surface and required allocator

### Body rewrite is reusable

`SymbolExprMap` is a hash map from full `Symbol` keys to replacement `Expr`
values (`crates/rum-runtime/src/runtime/model/cache.rs:11-19`). The reusable
function is:

```text
pub fn substitute_lexical_bindings(
    expr: &Expr,
    bindings: &SymbolExprMap,
    shadowed: &BTreeSet<Symbol>,
) -> Expr
```

at `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:365-384`.
Its atom branch replaces a symbol by map lookup unless it is shadowed
(`crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:45-55`,
`crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:386-427`).
`Module` uses it for both the
body and initializers with an empty initial shadow set
(`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:77-85`,
`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:107-116`).
This is exactly the operation needed to rewrite a lowered source-unit body
from ordinary placeholder symbols to its Public, Package, and sealed Private
symbols. It is reusable as-is and already handles nested lexical shadowing.

### Fresh-name allocator is not reusable

`allocate_fresh_symbol` and `allocate_generated_symbol` only mint unused
textual names with `Symbol::new`
(`crates/rum-runtime/src/runtime/scoping/unique.rs:36-43`,
`crates/rum-runtime/src/runtime/scoping/unique.rs:81-83`,
`crates/rum-runtime/src/runtime/scoping/unique.rs:149-167`). They never set
identity. Their session counter is
`next_generated_symbol_id` (`crates/rum-runtime/src/runtime/model/session.rs:57-64`),
initialized to zero (`crates/rum-runtime/src/runtime/session/session_state.rs:397-404`).
There is no session identity counter.

A real seal therefore needs a new identity-minting allocator and a
snapshot-persisted session field, for example `next_sealed_symbol_identity`,
rather than reusing the textual suffix counter. Its output must:

- never be zero (`Symbol::with_identity` requires `NonZeroU64`);
- be disjoint, for the same context/name text, from the per-name Remove
  sequence, which currently starts at 1 and reserves no bits; and
- be disjoint from the rule-alpha high-bit region marked by bit 63,
  `0x8000000000000000`
  (`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:18-20`,
  `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:525-531`).

Identity equality is only relevant when context/name text is also equal, but
an attacker can deliberately operate on the same readable private name. A
suggested bit-62 sealed region would be practically separated, but it is not
formally disjoint while `Remove` has no reserved-region rule. The collision
region cannot be selected from current code alone: the director must choose a
formal partition and make all producers honor it (or choose another
collision-proof scheme). Deserialization also accepts every nonzero value and
remains a trusted-host surface.

**Q4 verdict:** reuse `substitute_lexical_bindings`; do not reuse either
fresh-name allocator. Add a distinct, persisted identity allocator after the
numeric-region decision.

## Q5 — Introspection and name-enumeration surfaces

`Capability::Diagnostic` is the existing reveal capability and occupies bit
5 (`crates/rum-runtime/src/runtime/capabilities.rs:1-27`). A policy query is
available through `CapabilityPolicy::allows`
(`crates/rum-runtime/src/runtime/capabilities.rs:75-83`).

### Design-named language surfaces

| Surface and exact code site | Current exposure | Batch-2 filter attachment point |
|---|---|---|
| `Names`: `eval_names` at `crates/rum-runtime/src/runtime/namespace_tools/names.rs:67-114`; source set at `enumerated_names_symbols`, `crates/rum-runtime/src/runtime/namespace_tools/names.rs:267-292` | The source set contains builtin keys, canonical and extra System names, then user keys from `definitions`, `attributes`, `options`, `syntax_information`, `declared_stubs`, and `message_definitions` (`crates/rum-runtime/src/runtime/namespace_tools/names.rs:270-289`). | Put the identity-aware sealed-symbol filter in `names_matching_patterns` before candidate rendering, at the loop beginning `crates/rum-runtime/src/runtime/namespace_tools/names.rs:195`. This centralizes both ordinary `Names` and `resolve_names_matching_expr`. Permit sealed entries only for owner scope or `Diagnostic`. |
| Explicit-context `Names` | `names_matching_patterns` renders a full name immediately when `explicit_context` is true (`crates/rum-runtime/src/runtime/namespace_tools/names.rs:186-217`). Only the non-explicit branch applies `visible_contexts` (`crates/rum-runtime/src/runtime/namespace_tools/names.rs:196-205`). Thus ``Names["Pkg`Private`*"]`` bypasses context visibility today. | The sealed filter must run before the `if explicit_context` branch at `crates/rum-runtime/src/runtime/namespace_tools/names.rs:195-196`; adding it only to `visible_contexts` is insufficient. |
| `NameQ` | `eval_name_q` calls `exact_symbol_name_exists` (`crates/rum-runtime/src/runtime/namespace_tools/names.rs:132-150`). A qualified query constructs the active symbol and tests `symbol_exists`; an unqualified query uses normal resolution (`crates/rum-runtime/src/runtime/namespace_tools/names.rs:314-322`). | At `exact_symbol_name_exists`. A sealed-only textual key must return false without owner/`Diagnostic`; a diagnostic query needs a sealed-registry lookup because normal resolution intentionally cannot construct the sealed identity. If a distinct ordinary symbol with the same text has its own state, `NameQ` should truthfully report that ordinary symbol. |
| `Contexts` | `eval_contexts` starts with System, Global, current context, context path, and loaded packages, then extracts contexts from `definitions`, `attributes`, `options`, `syntax_information`, `declared_stubs`, `removed_symbols`, `message_definitions`, and `symbol_generations` (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:102-168`). | Filter every symbol-derived insertion at `crates/rum-runtime/src/runtime/builtins_context_loader.rs:138-154` before inserting its context. Private contexts must also never be placed in `current_context`, `context_path`, or `loaded_packages` for an outside scope; otherwise the earlier insertions leak them. Diagnostic/owner may include them. |
| `Definition` | `eval_definition` normalizes a held symbol or symbol-name string, then calls `definition_expr_for_symbol` (`crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:4-14`, `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:152-202`). String normalization itself uses `resolve_symbol_name` (`crates/rum-runtime/src/runtime/messages/message_catalog.rs:164-175`, `crates/rum-runtime/src/runtime/messages/message_catalog.rs:207-213`). The renderer exposes attributes, messages, options, all definition-catalog entries, defaults, and syntax information. | In `eval_definition`, before line 7 calls the renderer. Non-diagnostic targets need the authorization check after ordinary normalization. Diagnostic/owner string targets need a separate cold sealed-registry lookup by text, because ordinary normalization intentionally returns the identity-`None` decoy. Keep a defensive check at `definition_expr_for_symbol` for a trusted host-supplied sealed `Expr`. |
| `Information` | `eval_information` normalizes its held target and calls `information_data_expr_for_symbol` (`crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:16-30`). The latter exposes usage, messages, all value tables, options, attributes, and full name (`crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:245-278`). | In `eval_information`, before line 29 calls the renderer, with the same ordinary-target gate and cold diagnostic/owner sealed-target lookup as `Definition`. A shared inspection-authorization/target helper should serve both. |
| Prefix `?symbol` | No such display path currently exists. The lexer tokenizes `?` (`crates/rum-syntax/src/lexer/core.rs:127-136`), but the parser maps it only to binary `PatternTest` (`crates/rum-syntax/src/parser/helpers.rs:162-177`; spelling at `crates/rum-syntax/src/ast.rs:322-325`). | There is no current handler to filter. If prefix `?` is added later, lower it to the already-authorized `Information` path rather than creating a second renderer/gate. |

The source-unforgeability mechanism already hides a private from
``Definition["fully`qualified`name"]`` and `Information[...]`: string
normalization returns the ordinary `identity: None` symbol, not the sealed
key. The explicit policy gate is still needed for owner/diagnostic views,
host-supplied identity-bearing expressions, and defense in depth.
If two source units seal the same readable context/name with different
identities, a diagnostic string target is ambiguous; the sealed registry must
either return all matches or require a unit-qualified selector. Normal name
resolution must not be changed to solve that cold introspection problem.

### Additional enumeration/exposure sites found by the code sweep

These are not all named in design §4, but they enumerate the same maps and
would otherwise weaken existence hiding:

- Symbol completion enumerates `builtins`, `definitions`, `attributes`,
  `syntax_information`, `declared_stubs`, `message_definitions`, and known
  contexts at `crates/rum-runtime/src/runtime/session/session_api.rs:374-423`.
  Filter sealed symbols in `insert_completion_candidate` before rendering
  (`crates/rum-runtime/src/runtime/session/session_api.rs:438-453`) and filter
  private context candidates too.
- `Messages[]` and `MessageDefinitions[]` can enumerate identity-bearing user
  message keys globally (`crates/rum-runtime/src/runtime/messages/builtins_messages.rs:50-102`;
  collection at `crates/rum-runtime/src/runtime/messages/message_catalog.rs:245-255`,
  `crates/rum-runtime/src/runtime/messages/message_catalog.rs:296-308`, and
  `crates/rum-runtime/src/runtime/messages/message_catalog.rs:352-390`).
  `MessageTags[]` can reveal their tags
  (`crates/rum-runtime/src/runtime/messages/builtins_messages.rs:104-126`;
  `crates/rum-runtime/src/runtime/messages/message_catalog.rs:331-349`). Apply
  the sealed filter in
  `merged_message_catalog_sources` before insertion, with diagnostic/owner
  override.
- `SymbolName` exposes the name of an already-held symbol
  (`crates/rum-runtime/src/runtime/namespace_tools/names.rs:116-130`), and
  `Context` exposes the context of an already-held symbol
  (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:80-99`). They do not
  enumerate and cannot by themselves forge a sealed symbol, but a caller that
  receives a leaked sealed expression can inspect it. The director should
  decide whether the real boundary promises non-leakage of such expressions
  or also requires these accessors to redact.
- The runtime/host definition summary and context APIs enumerate definition
  keys directly (`crates/rum-runtime/src/runtime/session/session_api.rs:191-208`,
  `crates/rum-runtime/src/runtime/session/session_api.rs:247-262`; exposed by
  `crates/rum-host/src/session.rs:447-480`). Completion
  and host inspection are trusted embedding surfaces, but if capability policy
  is meant to govern them, filter in the runtime APIs rather than each host
  adapter.
- Context-targeted `Save`/`DumpSave` enumerates definitions, attributes,
  options, syntax information, and messages in `symbols_in_context`
  (`crates/rum-runtime/src/runtime/package_tools.rs:271-359`), then returns
  fully qualified names
  (`crates/rum-runtime/src/runtime/package_tools.rs:258-265`) and renders bodies
  (`crates/rum-runtime/src/runtime/package_tools.rs:374-405`,
  `crates/rum-runtime/src/runtime/package_tools.rs:472-529`). A non-diagnostic
  language caller must not be able
  to dump sealed definitions. Attach the filter in `symbols_in_context` and in
  `render_saved_symbols` as defense for explicit identity-bearing targets.

**Q5 verdict:** the named filter points are localized, but `Names` is not the
only enumerator. Completion, global message catalogs, host summaries/contexts,
and persistence enumeration must be included in the later visibility audit or
explicitly classified as trusted-only surfaces.

## Q6 — Snapshot round-trip and default-path cost

### Serde and actual snapshot surfaces

At the core value layer, identity round-trip is confirmed:

- serialization emits `context`, `name`, and, when present, the numeric
  `identity` (`crates/rum-core/src/expr.rs:221-233`);
- deserialization dispatches `Some(identity)` back through
  `Symbol::with_identity` and `None` through `Symbol::new`
  (`crates/rum-core/src/expr.rs:236-251`); and
- `Expr` itself derives `Serialize`/`Deserialize`, so nested symbols use that
  implementation (`crates/rum-core/src/expr.rs:652-656`).

That proves a serialized `Symbol`/`Expr` round-trips its identity. It does not
prove current session snapshot round-trip. `rum-cli --help` exposes evaluation,
parse, format, and file modes but no session snapshot/restore command. More
importantly, the public host snapshot is explicitly a textual source image:
`snapshot_to_bytes` calls `snapshot_definitions_source`, and restore resets and
evaluates that source (`crates/rum-host/src/session.rs:179-219`). The runtime
renderer enumerates definition keys and prints rum source
(`crates/rum-runtime/src/runtime/session/session_api.rs:210-225`;
`crates/rum-runtime/src/runtime/package_tools.rs:374-405`,
`crates/rum-runtime/src/runtime/package_tools.rs:472-529`). `Symbol` display
prints only the
qualified textual name, never identity (`crates/rum-core/src/expr.rs:255-258`).

Therefore there is no honest CLI identity-bearing snapshot probe to show, and
there is no identity-bearing `Module` gensym round-trip test: current `Module`
gensyms have `identity: None`. Existing snapshot tests exercise textual names
and definitions, not the `with_identity` serde branch. A sealed-private session
round-trip will require an identity-preserving session image or an explicit
trusted restore representation plus a test. Encoding an identity in ordinary
rum source would contradict the no-source-constructor boundary and should not
be introduced accidentally.

### Current per-name lookup cost

`lower_syntax_expr` invokes `resolve_symbol_name` for every raw symbol token
(`crates/rum-runtime/src/runtime/resolution.rs:66-69`). Today:

1. A qualified raw name is split and its context normalized, then sent
   directly to `active_symbol_for_name`
   (`crates/rum-runtime/src/runtime/resolution.rs:101-105`).
2. An unqualified raw name allocates/uses a `BTreeSet` to deduplicate the
   context path plus current context, constructs an active symbol in each,
   and calls `symbol_exists` until one exists
   (`crates/rum-runtime/src/runtime/resolution.rs:107-124`). If
   creation is allowed and none exists, it constructs the current-context
   active symbol once more (`:126`).
3. `active_symbol_for_name` has a System fast path. Every non-System attempt
   constructs a generation key, probes `symbol_generations`, then creates
   either `with_identity` or `new`
   (`crates/rum-runtime/src/runtime/resolution.rs:129-139`).
4. `symbol_generation_key` allocates two `String`s on every such attempt
   (`crates/rum-runtime/src/runtime/resolution.rs:488-490`). This is
   pre-existing cost.

The hard constraint is that when no sealed unit is registered, ordinary name
resolution must remain byte-for-byte the existing path: no new allocation,
map probe, or unconditional branch. For Private sealing, that is achievable in
the strongest way: add **no resolver-side lookup at all**. Rewritten internal
references already contain the sealed `Symbol`; outside spelling should keep
running the existing miss branch and produce `Symbol::new`. The sealed registry
is needed at registration, introspection, and snapshot boundaries, not on the
ordinary resolver hot path.

If a later requirement insists on a resolver-side policy check (for example,
hard enforcement of Package scope), the semantic seam would be after explicit
context normalization and before `active_symbol_for_name`, or immediately
after the System fast path and before the generation-map lookup at
`crates/rum-runtime/src/runtime/resolution.rs:132-134`. An empty-map/`None`
guard there gives "sealed state empty -> no sealed-map lookup or allocation,"
but it is still a new branch and therefore does not satisfy the brief's literal
"byte-identically" constraint. The no-check Private design avoids that
contradiction entirely.

**Q6 verdict:** core Symbol serde preserves identity, but the current public
text snapshot does not. Private sealing can impose zero default resolver cost
by leaving `resolve_symbol_name` and `active_symbol_for_name` unchanged.

## Batch-1 mechanism map

### 1. Registration seam

Use a held wrapper over a host-provided body, provisionally:

```text
DefinePackage["Pkg`", Public[{f, g}], PackageScope[{h}], body...]
```

The repository's current Module-class registration seam is the scoping core
catalog (`crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs:4-32`),
which is included by `core_builtin_specs`
(`crates/rum-runtime/src/runtime/builtin_catalog.rs:32-64`) and installed into
sessions by `rum_stdlib::install_core`
(`crates/rum-stdlib/src/lib.rs:39-42`). For the present architecture rule that
stdlib owns builtin specs/implementations while runtime owns kernel machinery,
the concrete split should be: a held `DefinePackage` spec/adapter registered by
`install_core`, calling a kernel-resident source-unit registration API in
`rum-runtime`. `Public` and `PackageScope` should be accepted only as held
declaration forms within that wrapper (and cataloged as protected inert heads
if they can appear in parsed expressions).

### 2. Two-pass scan

Lower the wrapper body once to ordinary placeholder symbols, without
evaluating it.

1. Validate and normalize the owning public context.
2. Scan the held declarations and collect Public names and Package names;
   reject duplicates, overlap, malformed names, and declarations outside the
   owning package.
3. Walk definition-bearing top-level forms and collect every symbol the unit
   binds. Existing assignment classification can identify ordinary/message/
   syntax definition owners (`crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:197-216`),
   but the batch brief must define the complete binding grammar as well:
   attributes, options, messages, tagged/up-set definitions, stubs, and
   compound/nested top-level forms.
4. Classify collected bindings: declared Public; declared Package; otherwise
   Private. Reference-only symbols are not made private.

### 3. Symbol allocation and rewrite

- Public targets become ordinary `identity: None` symbols in `` Pkg` ``.
- Package targets become ordinary `identity: None` symbols in
  `` Pkg`PackageScope` ``.
- Private targets become readable
  ``Symbol::with_identity("Pkg`Private`", name, sealed_id)`` values from the new
  allocator described in Q4.
- Build one `SymbolExprMap` from each lowered placeholder symbol to its target
  symbol expression, then rewrite the complete held body with
  `substitute_lexical_bindings`.
- Register sealed metadata keyed by the full identity-bearing symbol and its
  owning unit/package for later owner/Diagnostic decisions, plus a cold
  text-to-sealed-symbol index for diagnostic introspection. Do not put the
  private `(context, name)` in `symbol_generations`, and do not consult the
  cold index from ordinary resolution.
- Evaluate/install the rewritten body so existing definition, attribute,
  option, syntax, and message paths key their maps by the rewritten symbols.
  Transactional rollback is required if scan, rewrite, or installation fails.

### 4. Resolution outcome

Keep ordinary resolution unchanged. Code already holding a rewritten private
symbol reaches its sealed definitions. Outside direct spelling, `Symbol`, and
`ToExpression` all call `active_symbol_for_name`; because the private name is
absent from `symbol_generations`, they receive the same-text
`Symbol::new(...), identity: None`. Full-symbol equality fails against the
sealed key, so no sealed binding is reached. An attacker may define a decoy
ordinary symbol with the same text, but it remains a distinct binding.

### 5. Required source-reachability tests for batch 1

Use a package with a public function that demonstrably reaches its rewritten
private helper/value, then assert all outside paths fail to reach that binding:

1. Direct fully qualified source spelling of ``Pkg`Private`x`` produces an
   ordinary identity-`None` symbol and does not return the sealed value. This
   direct case is necessary because Q2 proved it breaks the current Module
   model.
2. ``ToExpression["Pkg`Private`x"]`` produces `identity: None`, is unequal to
   the sealed symbol in a Rust-level assertion, and reaches no sealed binding.
3. ``Symbol["Pkg`Private`x"]`` has the same result.
4. ``ToExpression["Pkg`Private`" <> "x"]`` has the same result.
5. Repeating the paths after assigning a value to the ordinary decoy proves
   they reach only the decoy and the public function still reaches the sealed
   private.
6. Snapshot/restore is not a batch-1 test under the design plan, but the new
   identity counter and sealed registry must be designed as persistable from
   the start.

The design §5 visibility tests remain mandatory for batch 2: explicit-context
`Names` and `Contexts` do not enumerate the sealed name/context, and
`Definition`, `Information`, and any future `?` alias do not print the private
body without owner/`Capability::Diagnostic`.

## Stop-loss and open decisions for the director

No requested Q1-Q3 probe hit stop-loss.

1. **Identity collision partition:** current Remove generations reserve no
   bits, while rule-alpha owns the bit-63 region. Select a formal sealed region
   and update producer invariants accordingly; a merely "large" starting
   value is not a proof of disjointness.
2. **Snapshot representation:** core serde is sufficient for a Symbol but the
   public session snapshot is text and loses identity. Decide the trusted,
   identity-preserving session image before claiming sealed snapshot support.
3. **Package-tier semantics:** a Package symbol described as an ordinary
   interned symbol remains reachable by a fully qualified name because the
   explicit-context resolver branch returns it without a visibility check
   (`crates/rum-runtime/src/runtime/resolution.rs:101-105`). This matches the
   design's statement that Package is a convention, but not the table's phrase
   "only sibling units." Decide which wording/behavior is authoritative. Hard
   enforcement would require resolver policy and its hot-path cost gate.
4. **Prefix `?`:** no such syntax exists today. Either remove it from the
   sealed-scoping acceptance surface or separately add it as an alias routed
   through the `Information` authorization point.
5. **Binding scan grammar:** define exactly which top-level mutating forms
   count as "the unit binds" before implementing pass 1; ordinary definition
   classification alone does not cover attributes/options/stubs.
6. **Owner visibility for string construction:** direct source references
   inside a unit are rewritten and work, but a string-built private name still
   resolves to `identity: None` unless owner scope gains a resolver exception.
   Keeping it unforgeable everywhere preserves the zero-cost resolver design;
   allowing owner-side `ToExpression` requires an explicit scoped policy and
   cost decision.
7. **Duplicate private display names:** two units may seal the same private
   context/name with different identities. Decide how Diagnostic `Names`,
   `Definition`, and `Information` select or display those otherwise
   indistinguishable textual names.

## Verification record

The release binary was rebuilt before probing:

```text
scripts/run-bounded.py --timeout 600 -- cargo build --release -p rum-cli
    Finished `release` profile [optimized] target(s) in 0.06s
```

No microbenchmark or source test gate was run because this brief changes no
source; the verbatim probes and code citations are the required evidence.
