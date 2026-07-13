# Kernel C Usability Audit Findings

## Count reconciliation and coverage

The installed-reference subtraction reproduces the brief exactly: 612 installed
headings minus 453 stdlib catalog names gives 159 kernel names; subtracting the
55 names in unit 8 and the 47 names in unit 9 leaves this unit's exact 57-name
complement. Both inputs are duplicate-free, the earlier units are disjoint, and
the authoritative unit-10 set equals the computed complement.

```text
$ grep -c '^### ' docs/reference/builtins.md
612
$ python3 - <<'PY'
from pathlib import Path
import re

spec_re = re.compile(r'BuiltinSpec::[A-Za-z_]+\(\s*"([^"]+)"')
heading_re = re.compile(r'^### (.+)$', re.M)
stdlib = [
    name
    for path in sorted(Path('crates/rum-stdlib/src/catalog').glob('*.rs'))
    for name in spec_re.findall(path.read_text())
]
reference = heading_re.findall(Path('docs/reference/builtins.md').read_text())
unit8 = [
    'Set', 'SetDelayed', 'TagSet', 'TagSetDelayed', 'TagUnset', 'UpSet',
    'UpSetDelayed', 'AddTo', 'SubtractFrom', 'TimesBy', 'DivideBy',
    'Increment', 'Decrement', 'PreIncrement', 'PreDecrement', 'AppendTo',
    'PrependTo', 'ApplyTo', 'OwnValues', 'DownValues', 'UpValues', 'SubValues',
    'DefaultValues', 'ValueQ', 'Definition', 'Information', 'Clear', 'ClearAll',
    'Remove', 'Unset', 'Attributes', 'SetAttributes', 'ClearAttributes',
    'Protect', 'Unprotect', 'HoldAll', 'HoldFirst', 'HoldRest',
    'HoldAllComplete', 'SyntaxInformation', 'Options', 'SetOptions',
    'OptionValue', 'ArgumentsOptions', 'Message', 'Messages', 'MessageName',
    'MessageTags', 'MessageDefinitions', '$MessageList', 'Quiet', 'Check',
    'CheckArguments', 'Off', 'On',
]
unit9 = [
    'Block', 'Module', 'With', 'Function', 'Unique',
    'Hold', 'HoldComplete', 'HoldForm', 'HoldPattern', 'ReleaseHold', 'Evaluate',
    'Unevaluated', 'Once', 'Sequence', 'Nothing', 'Sow', 'Reap',
    'Repeated', 'RepeatedNull', 'Longest', 'Shortest', 'IgnoreCase', 'Overlaps',
    'Whitespace', 'NumberString', 'SpellingCorrection', 'Rule', 'RuleDelayed',
    'NormalizeRules', 'Part', 'Extract', 'ReplacePart', 'ReplaceAt', 'Position',
    'Count', 'MemberQ', 'FreeQ', 'Level', 'Thread', 'MissingQ', 'InputForm',
    'OutputForm', 'StandardForm', 'TraditionalForm', 'Short', 'Automatic', '$Failed',
]
unit10 = [
    '$Context', '$ContextPath', '$Packages', 'Begin', 'End', 'BeginPackage',
    'EndPackage', 'Context', 'Contexts', 'DeclarePackage', 'Needs',
    'ResolveContextAliases', 'Names', 'NameQ', 'SymbolName', '$Path',
    '$InputFileName', 'DirectoryName', 'ParentDirectory', 'FileNameJoin',
    'FileNameTake', 'FindFile', 'File', 'Get', 'Save', 'DumpSave',
    'AbsoluteTime', 'AbsoluteTiming', 'Timing', 'RepeatedTiming',
    'TimeConstrained', 'Pause', 'MemoryConstrained', 'MemoryInUse',
    'MaxMemoryUsed', 'ByteCount', 'EvaluationData', 'Print', 'Echo',
    'EchoFunction', 'EchoLabel', 'Monitor', 'Trace', 'TracePrint', 'Stack',
    'RumProfileReset', 'RumProfileSnapshot', 'Input', 'InputString', 'Dialog',
    'TestCreate', 'TestReport', '$Aborted', '$Canceled', 'ByteArray',
    'DiffObject', 'TemplateObject',
]
ref_set, std_set = set(reference), set(stdlib)
kernel = ref_set - std_set
complement = kernel - set(unit8) - set(unit9)
print(f'installed reference headings = {len(reference)}')
print(f'stdlib catalog names = {len(stdlib)}')
print(f'kernel = {len(reference)} - {len(stdlib)} = {len(kernel)}')
print(f'unit 10 derivation = {len(kernel)} - {len(unit8)} - {len(unit9)} = {len(complement)}')
print(f'unit 10 authoritative names = {len(unit10)}')
print(f'unit 10 complement exact = {set(unit10) == complement}')
print(f'unit 10 missing from complement = {sorted(set(unit10) - complement)}')
print(f'complement absent from unit 10 = {sorted(complement - set(unit10))}')
print(f'unit 8/unit 9 overlap = {sorted(set(unit8) & set(unit9))}')
print(f'unit 8/unit 10 overlap = {sorted(set(unit8) & set(unit10))}')
print(f'unit 9/unit 10 overlap = {sorted(set(unit9) & set(unit10))}')
print(f'stdlib duplicate names = {len(stdlib) - len(std_set)}')
print(f'reference duplicate names = {len(reference) - len(ref_set)}')
print(f'stdlib names absent from reference = {sorted(std_set - ref_set)}')
assert len(reference) == len(ref_set) == 612
assert len(stdlib) == len(std_set) == 453
assert len(kernel) == 159
assert len(unit8) == 55 and len(unit9) == 47 and len(unit10) == 57
assert set(unit10) == complement
print('COMPLEMENT-EQUALITY-OK 57/57')
PY
installed reference headings = 612
stdlib catalog names = 453
kernel = 612 - 453 = 159
unit 10 derivation = 159 - 55 - 47 = 57
unit 10 authoritative names = 57
unit 10 complement exact = True
unit 10 missing from complement = []
complement absent from unit 10 = []
unit 8/unit 9 overlap = []
unit 8/unit 10 overlap = []
unit 9/unit 10 overlap = []
stdlib duplicate names = 0
reference duplicate names = 0
stdlib names absent from reference = []
COMPLEMENT-EQUALITY-OK 57/57
```

The findings-table verifier in the final section prints:

```text
COVERAGE-OK 57/57
```

The attribute head-start also reproduces: all 57 reference blocks contain an
attribute line, and only the three intentionally writable session variables
`$ContextPath`, `$Packages`, and `$Path` omit `Protected`. Protection is not a
finding in this unit.

## Capability headline

`Input`, `InputString`, and `Dialog` come first because their declared `Pure`
label is a genuine kind decision: each can suspend evaluation and request input
from the embedding host, exactly like the earlier `Exit` lifecycle-control
precedent, although they read no ambient source directly. They are installed and
allowed in the standard sandbox today; the host converts the runtime suspension
to `HostEvalOutcome::InputRequired` and exposes an explicit resume operation.
No unilateral relabel is proposed.

There are also two concrete enforcement mismatches. `BeginPackage[..., deps]`
is `Pure` but calls `eval_needs` directly, and a `DeclarePackage` stub later calls
the same loader directly from symbol evaluation; both paths bypass the only
dispatch-time capability check. The standard sandbox remains protected by its
second defense (it omits the file provider), but a Pure-only policy with an
explicit provider is not protected by the declared policy.

The closed-stdin head-start was stale on this checkout: all three CLI probes
printed `error: input required` and exited **1**, matching the CLI's documented
failure contract. There is no CLI exit-code gap.

## Top gaps by script-user value

1. **Safety / capability enforcement:** `BeginPackage` dependency loading and
   `DeclarePackage` autoload can reach the file provider through internal
   `eval_needs` calls without a `FileRead` dispatch check.
2. **Safety:** `Pause[10^100]` panics in `Duration::from_secs_f64` and terminates
   the CLI with exit 101; the single confirming run triggered the stop-loss.
3. **Capability decision:** `Input`, `InputString`, and `Dialog` are `Pure` even
   though they request host interaction and suspend/resume the evaluation.
4. **Loader safety:** a missing `BeginPackage` dependency emits `Get`/`Needs`
   messages but still opens the package and installs the missing context on
   `$ContextPath`.
5. **Host-isolation design input:** a successful `Get` containing an unbalanced
   `Begin` leaves the new context active after the load and subsequent request.
6. **Persistence truthfulness:** `DumpSave` accepts `CompressionLevel` but never
   reads it; levels 0 and 9 produced byte-identical textual files. Its promised
   saved-symbol list also re-evaluates symbols to their own values.
7. **Option truthfulness:** `ResolveContextAliases` is registered and defaults
   to `True`, but `Names` never reads it.
8. **Compatibility:** `EchoFunction[f]` treats `f` as a label and never applies
   it; it duplicates `EchoLabel[f]` rather than implementing the inherited
   function form.
9. **Contract:** `ByteCount` is documented as canonical-text bytes but returns
   the structural memory proxy (`1` and `123456789` both returned 32).
10. **Discoverability:** `Contexts` has no filter form, and `TestCreate` accepts
    option-like rules while `Options[TestCreate]` is empty.
11. **Candidate addition, not an auto-gap:** `FileNames` is absent after the FS
    purity removal, but mma-hol's `run_all.wls` uses the familiar read-only glob
    idiom; a long-tail, provider-mediated `FileRead` head is worth a decision.

## Findings table

### Contexts / packages

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `$Context` | WL | OK | boundary | Reads the snapshotable session's current context; the nested `Begin`/`End` probe moved through Global, Audit, and Audit-private contexts and restored the original (E2; `eval_core/symbol.rs:5-15`). | Preserve the session-state value. | |
| `$ContextPath` | WL | OK | boundary | The writable session variable controls lookup and is captured/restored with loader and host snapshots; explicit assignment remains the documented writable-variable exclusion (E2; `resolution.rs:100-127`; `session/session_helpers.rs:38-89`). | No capability or protection change. | |
| `$Packages` | WL | OK | boundary | Lists loaded package contexts, is writable session state, and round-trips through snapshots; `Needs` is idempotent once its context appears (E3; `eval_core/symbol.rs:17-24`). | Preserve this lightweight package-discovery surface. | |
| `Begin` | WL | gap / decision | safety | Context frames are session state rather than request-local state. A successful loaded file with an unmatched `Begin` left the Leaked context current after `Get` returned (E3), while host `reset()` restores the configured context (`rum-host/src/tests.rs:933-975`). | Carry this as dun host-isolation input: decide whether top-level/request boundaries should diagnose unbalanced frames; do not change session-state purity here. | M / decision |
| `End` | WL | OK | safety | Correctly restores a matching `Begin`; an unmatched call emits `End::context` and stays inert instead of corrupting the stack (E2; `builtins_context_loader.rs:269-281`). | No change. | |
| `BeginPackage` | WL | gap | capability | Declared `Pure`, but its dependency loop calls `self.eval_needs(...)` directly after dispatch (`builtins_context_loader.rs:284-315`), bypassing the capability check that exists only in `eval_dispatch/mod.rs:111-138`. With a provider present this is an actual file-read seam. | Gate the internal dependency load by `FileRead` (or route through a capability-aware internal API); do not treat a label-only change as sufficient. Evaluator/loader change needs A/B and the mma-hol Strict gate at the director's gate. | M/L |
| `BeginPackage` | WL | gap | safety | Missing dependency loads emit `Get::notfound` and `Needs::load`, but the `$Failed` value is ignored; the probe still entered the FailMain context and put the missing dependency context on `$ContextPath` (E3). | Fail closed and restore the just-opened package frame, or explicitly contract partial entry; loader work needs the mma-hol Strict gate at the director's gate. | M |
| `EndPackage` | WL | OK | boundary | Restores the prior context and prepends the opened public package context to the prior path (`session/session_api.rs:484-509`); the package probe returned to the Global context (E3). | Preserve the public-context visibility transition. | |
| `Context` | WL | OK | flexibility | Zero-arg, held-symbol, and string forms resolve through the current session; the probe distinguished a qualified Audit symbol from unqualified lookup (E2; `builtins_context_loader.rs:52-71`). | No form change. | |
| `Contexts` | WL | gap | convenience | Only `Contexts[]` is accepted. A string filter for the Audit context emits `Contexts::argx` and remains inert even though the unfiltered result can be large (E2; `builtins_context_loader.rs:74-116`). | Add a string-pattern filter over the already-collected deterministic context set. | S/M |
| `DeclarePackage` | WL | gap | capability | The `Pure` call inserts stubs, but evaluating an unresolved stub later invokes `eval_needs` directly from the evaluator hot path (`package_tools.rs:44-77`; `eval_core/symbol.rs:50-57`). The successful autoload probe confirms the latent load (E3); dispatch policy is bypassed. | Make implicit stub loading perform an explicit `FileRead` authorization check. This touches symbol evaluation and `Needs`: A/B plus the mma-hol Strict gate at the director's gate. | L |
| `Needs` | WL | OK | safety | Correctly carries `FileRead`, honors `$Path`, skips an already loaded context, and reports missing loads as messages plus `$Failed` (`builtins_context_loader.rs:201-250`; E3). | Preserve idempotence and fail-closed top-level dispatch. | |
| `ResolveContextAliases` | rum | gap | safety | It is a protected rum-native `Names` option symbol with default `True`, but `eval_names` reads only `SpellingCorrection` and `IgnoreCase`; no code reads this option (`options/defaults.rs:76-80`; `namespace_tools/names.rs:66-95`, `:242-246`). `True` and `False` produced identical results (E2). | Until alias semantics exist, remove it from `Options[Names]` or reject a non-default request explicitly; do not advertise a silent control. | S |

### Symbol table

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `Names` | WL | OK | flexibility | Context-qualified wildcard patterns work and enumerate fully qualified matches; unqualified patterns are visibility-filtered through `$ContextPath` (E2; `namespace_tools/names.rs:168-199`, `:276-292`). | Preserve the deterministic sorted result. | |
| `NameQ` | WL | OK | safety | Performs an exact existence query without creating the symbol; the probe returned `False` for an unassigned qualified name (`namespace_tools/names.rs:114-125`, `:284-292`; E2). | No change. | |
| `SymbolName` | WL | OK | flexibility | Qualified, unassigned symbols return only the short component (`"unassigned"`) for both Audit and Global contexts (E2; `namespace_tools/names.rs:98-112`). | No change. | |

### Paths / files / loading

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `$Path` | WL | OK | boundary | Writable package-search roots are session state. The `/tmp` package probe resolved through the assigned root, while relative nested loads search the current load's parent first (`resolution.rs:232-288`; E3). | Preserve provider-mediated search ordering. | |
| `$InputFileName` | WL | OK | boundary | Reads only the session load stack: the package observed its resolved `/tmp` path while top level returned `""` (`eval_core/symbol.rs:26-32`; E3). | No capability change. | |
| `DirectoryName` | WL | OK | safety | Pure lexical `Path`/`PathBuf` math; nonexistent inputs are processed without probing the filesystem (E2; `builtins_context_loader/file_names.rs:27-54`). | Keep it provider-free. | |
| `ParentDirectory` | WL | OK | safety | Pure lexical parent computation; the nonexistent-directory probe returned a parent string without an existence read (E2; `builtins_context_loader/file_names.rs:56-76`). | Keep it provider-free; the reference should continue to avoid implying an existence check. | |
| `FileNameJoin` | WL | OK | flexibility | Joins a list of components using platform path construction and returned `"a/b/c.txt"` on this host (E2; `builtins_context_loader/file_names.rs:4-25`). | No filesystem capability. | |
| `FileNameTake` | WL | OK | flexibility | One-argument leaf and supported component specs operate lexically; the probe returned `"c.txt"`, `"b/c.txt"`, and `"a/b"` (E2; `builtins_context_loader/file_names.rs:78-106`). | No change. | |
| `FindFile` | WL | OK | capability | Carries `FileRead` and probes only through `FileIo`; missing targets emit `FindFile::notfound` plus `$Failed` (`builtins_context_loader.rs:118-152`; `resolution.rs:247-259`; E3). | Preserve provider-only access. | |
| `File` | WL | OK | boundary | Protected inert file-spec wrapper accepted by loader and persistence normalization; `Get[File[path]]` loaded the scratch file (E3; `builtin_catalog/namespace.rs:94-109`). | Keep it a data head, not an I/O operation. | |
| `Get` | WL | OK | safety | Missing and malformed files produce messages plus `$Failed`; malformed-load mutations were rolled back to the Global context with no leaked symbol (`session/session_eval.rs:283-303`; E3). | Preserve error rollback. Any loader fix needs the mma-hol Strict gate at the director's gate. | |
| `Get` | WL | gap / decision | safety | Loader rollback occurs only on an error. A syntactically valid scratch file ending after an unmatched `Begin` succeeds and leaves that context active across the load boundary (E3; `session/session_eval.rs:293-303`). | Decide whether a successful load with a deeper context stack is invalid and should restore/diagnose. Any change needs the mma-hol Strict gate at the director's gate. | M / decision |
| `Save` | WL | OK | safety | Its reference truthfully says append: two saves produced two textual definitions, and the provider receives `FileWriteMode::Append` (`package_tools.rs:542-557`; E4). | Preserve explicit append semantics. | |
| `DumpSave` | WL | OK | boundary | This is rum's textual snapshot subset, not a binary format. It replaces the target and emits a deterministic package-restore preamble where relevant (`package_tools.rs:351-447`, `:542-557`; E4). | Do not expand it into a binary/foreign dump format. | |
| `DumpSave` | WL | gap | safety | `Options[DumpSave]` advertises `CompressionLevel`, but the implementation reads only `SymbolAttributes`; repository search finds no consumer. Levels 0 and 9 produced byte-identical text (E4; `options/defaults.rs:68-71`; `package_tools.rs:633-640`). | Reject unsupported `CompressionLevel` values or remove the option until implemented; never silently accept it. | S |
| `DumpSave` | WL | gap | convenience | The handler builds an ordinary list of saved symbols; fixed-point evaluation immediately substitutes their own values. Saving `dumpB=2` returned `List[2]`, not a usable symbol list (E4; `package_tools.rs:235-242`). | Return a representation that preserves symbol identity (held symbols or names), and pin it in the rum snapshot contract. | S/M |

### Timing / memory / constraints

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `AbsoluteTime` | WL | OK | capability | Reads `SystemTime::now`, returns an exact rational Unix timestamp, and correctly carries `Clock` (`debug_tools/timing.rs:10-21`, `:124-143`; E5). | No label change. | |
| `AbsoluteTiming` | WL | OK | flexibility | Returns the documented two-element `{elapsed,value}` exact-rational shape; the implementation uses `Instant` and therefore correctly carries `Clock` (E5; `debug_tools/timing.rs:23-75`). | Preserve the current documented wall-clock subset. | |
| `Timing` | WL | OK | flexibility | Returns the same documented `{elapsed,value}` first-pass wall-clock shape and is labeled `Clock` (E5; `debug_tools/timing.rs:23-75`). | No label change; any future CPU-time distinction is separate scope. | |
| `RepeatedTiming` | WL | OK | boundary | Runs the held expression three times and returns the best elapsed duration plus the last value; the probe returned `{rational,6}` (`debug_tools/timing.rs:3-8`, `:77-120`; E5). | Preserve the documented fixed-sample subset. | |
| `TimeConstrained` | WL | OK | safety | Two/three-argument forms use a cooperative `Instant` deadline; budget exhaustion returns `$Aborted` or evaluates the provided fallback (E5; `resources/constraints.rs:5-64`, `:205-224`). `Clock` is the correct kind. | No change. | |
| `Pause` | WL | gap | safety | Negative/non-numeric forms diagnose and remain inert, but `Pause[10^100]` passes the finite check and panics in `Duration::from_secs_f64`, exiting 101 (E5; `builtins_input.rs:107-127`, `:419-434`). | Use a checked duration conversion and return `Pause::numnm` for unrepresentable finite values. | S |
| `MemoryConstrained` | WL | OK | capability | Correctly remains `Pure`: it meters deterministic session/`Expr` structural proxies and never reads RSS or allocator globals; exhaustion returns `$Aborted`/fallback (E5; `resources/constraints.rs:66-141`, `:226-245`; `resources/memory.rs:49-90`, `:257-308`). | Keep cooperative proxy budgeting separate from host hard budgets. | |
| `MemoryInUse` | WL | OK | capability | Reads the session structural memory proxy, but exposes internal resource state and is conservatively `Diagnostic`; sandbox denies it while trusted/Diagnostic profiles allow it (`resources/memory.rs:19-30`; `builtin_catalog/debug.rs:106-117`). | No label change. | |
| `MaxMemoryUsed` | WL | OK | capability | Samples and returns the session proxy high-water mark, an internal diagnostic seam rather than process RSS (`resources/memory.rs:33-47`, `:251-253`; E5). `Diagnostic` is appropriate. | No change. | |
| `ByteCount` | WL | gap | contract | Reference/catalog promise a canonical-text byte count, but the handler returns `expr_memory_proxy_bytes`. Probe results `ByteCount[1] == ByteCount[123456789] == 32` disprove the documented text metric (E5; `resources/memory.rs:5-17`, `:257-308`). | Decide and align one contract; the shared structural proxy appears intentional, so a catalog/reference correction plus focused tests is the smallest likely fix. | S / decision |
| `EvaluationData` | WL | OK | capability | Always captures elapsed `Instant` timing along with result, messages, output, and trace, so `Clock` is honest (`debug_tools/evaluation_data.rs:14-48`, `:83-177`; E5). | No label change. | |

### Diagnostics / output

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `Print` | WL | OK | capability | Renders arguments into the session print stream and returns `Null`; kernel code calls `push_print`, which queues output or invokes a host-installed sink rather than writing stdout (`builtins_input.rs:97-105`; `session/session_helpers.rs:176-182`; E6). `Pure` matches the session-state doctrine. | Preserve session mediation. | |
| `Echo` | WL | OK | flexibility | One- and two-argument label forms print through the same session seam and return the original value; probes printed `42` and `tag: 7` (`debug_tools/basic.rs:30-49`; E6). | No capability change. | |
| `EchoFunction` | WL | gap | flexibility | `EchoFunction[f]` constructs `Echo[Slot[1],f]`, treating `f` as a label. The function probe printed `Function[Plus[2, 1]]: 2` and returned 2; `f` was never applied, making the form duplicate `EchoLabel` (E6; `debug_tools/basic.rs:52-64`). | Implement the inherited function-transform form; keep labeling on `EchoLabel` (and the existing two-argument `Echo` form). | S/M |
| `EchoLabel` | WL | OK | convenience | Its operator form labels the echoed argument and returns it; `EchoLabel["sum"][1+1]` printed `sum: 2` and returned 2 (E6; `debug_tools/basic.rs:66-82`). | No change. | |
| `Monitor` | WL | OK | capability | Display updates and callbacks are session-evaluated; changed displays go through `push_print`, with no ambient output read/write (`debug_tools/monitor.rs:42-80`, `:116-144`; E6). `Pure` is correct. | Keep the current folk display/callback subset. | |
| `Trace` | WL | OK | boundary | Captures structured session evaluation events and returns held trace data; the probe returned `List[Hold[Plus[1,1]],Hold[2]]` (`debug_tools/trace.rs:17-80`, `:150-212`; E6). | No debugger/stepper expansion. | |
| `TracePrint` | WL | OK | capability | Uses the same structured capture but emits rendered lines only through `push_print` (`debug_tools/trace.rs:82-148`, `:195-212`; E6). `Pure` is correct. | Preserve session mediation. | |
| `Stack` | WL | OK | boundary | Returns held evaluator frames and is empty at top level; the probe returned `{}` (`debug_tools/basic.rs:84-97`; E6). | Keep the folk introspection subset. | |

### Profiling

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `RumProfileReset` | rum | OK | capability | Mutates process-global diagnostic counters only when profiling was env-configured; otherwise it immediately returns and the builtin returns `Null` (`profiling/reset.rs:3-10`; `debug_tools/basic.rs:99-110`; E6). `Diagnostic` is correct. | Preserve the env-gated cold-load `AtomicU8` pattern and zero profiling-off allocation/lock cost. | |
| `RumProfileSnapshot` | rum | OK | capability | Returns `Null`; with profiling off it emits nothing, while configured modes print diagnostics directly to stderr (`profiling/summary.rs:19-31`; `evaluation_cache/profile/report.rs:6-13`; E6). `Diagnostic`, not `Pure`, is correct. | Preserve no-op-off behavior and the cold gate. | |

### Interactive input

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `Input` | WL | gap / decision | capability | Declared/allowed as `Pure`, but it creates a typed host request, suspends the evaluator, then parses and evaluates host-supplied source on resume (`builtins_input.rs:83-95`, `:173-233`, `:288-297`; `session/session_requests.rs:108-158`). This is host-interaction kind, analogous to the user-decided `Exit -> Process` precedent. | Director/user decision: retain `Pure`, introduce a capability kind, or otherwise model host interaction. Do not unilaterally relabel. | M / decision |
| `Input` | WL | OK | safety | Under closed stdin the CLI printed `input required` and exited 1, exactly matching its documented evaluation-failure contract; the brief's exit-0 head-start does not reproduce (E7; `rum-cli/src/main.rs:333-355`, `:375-389`). | No CLI exit-code fix. | |
| `InputString` | WL | gap / decision | capability | The same `Pure` suspension seam returns host text verbatim; rum-host exposes the request kind/prompt/password fields and an explicit resume operation (`builtins_input.rs:74-81`, `:173-233`; `rum-host/src/session.rs:267-287`, `:338-354`). | Resolve with the same host-interaction capability decision as `Input`; no unilateral relabel. | M / decision |
| `Dialog` | WL | gap / decision | capability | `Dialog[]` is not a UI subsession: it defaults to `Input[""]`, while an explicit body evaluates in optional `Block` locals and catches `Return`. The default therefore suspends for host input despite `Pure` (`builtins_input.rs:39-71`, `:268-285`; E7). | Record the intrinsic rum subset and decide the host-interaction label with `Input`; do not pursue WL dialog UI semantics. | M / decision |

### Tests

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `TestCreate` | rum | OK | boundary | The rum-native shape is an `Association` holding input and expected output, with optional test metadata; the probe confirms the association contract (`debug_tools/test_report/create.rs:4-60`; E8). | Preserve the rum-native data shape. | |
| `TestCreate` | rum | gap | convenience | It accepts `TestID`, `ExpectedMessages`, and `SameTest` rule names (including string names), but registers no defaults: `Options[TestCreate]` returned `{}` (`test_report/parsing.rs:69-97`; E8). | Expose discoverable symbolic defaults/metadata or a dedicated introspection contract; defer any general string-keyed-option cleanup to the host line. | S/M |
| `TestReport` | WL / rum-shaped | OK | convenience | The association report exposes counts, success, per-test held input/expected output, actual output, messages/text, output log, timing, and failure outcome; property projection works (E8; `test_report/report.rs:180-285`). | Preserve the useful failure detail and rum-native association shape. | |
| `TestReport` | WL / rum-shaped | gap / decision | capability | Test execution is session-pure, but each case unconditionally records an `Instant` duration and the report always includes `"Timing"`; that field alone forces the whole head to `Clock` (`test_report/evaluation.rs:169-180`, `:232-309`; `test_report/report.rs:193-257`). | Ordered decision input: consider optional timing or a separate timed reader, without redesigning the test runner in this audit. | M / decision |

### Markers / data heads

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `$Aborted` | WL | OK | safety | Protected inert marker paired with `TimeConstrained`/`MemoryConstrained` exhaustion and abort capture; it remains a symbol when evaluated (E5, E8; `builtin_catalog/namespace.rs:12-19`). | Preserve the shared marker. | |
| `$Canceled` | WL | OK | boundary | Protected inert confirmation-flow compatibility marker; it remains a symbol and performs no ambient action (E8; `builtin_catalog/namespace.rs:16-19`). | No change. | |
| `ByteArray` | WL | OK | boundary | Protected inert data head; resource accounting recognizes encoded payloads structurally without granting I/O (`builtin_catalog/strings.rs:113-116`; `resources/memory.rs:277-293`). | Keep payload validation in consumers. | |
| `DiffObject` | rum | OK | boundary | Protected rum-native result wrapper with no handler or surrounding-system touch (`builtin_catalog/strings.rs:109-112`; E8). | No change. | |
| `TemplateObject` | rum | OK | boundary | Protected rum-native template wrapper with no handler or ambient authority (`builtin_catalog/strings.rs:105-108`; E8). | No change. | |

### Candidate addition (outside the authoritative 57)

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| `FileNames` | WL | candidate, not auto-gap | convenience | The head is absent (`NameQ["FileNames"] -> False`, and the call stays inert; E8). It was deliberately removed with general FS builtins, but mma-hol `run_all.wls` uses the folk glob discovery idiom and silently skipped most tests when it was absent (`PROGRESS.md:634-650`; `docs/dev/usability-pass.md:325-334`). | Decision candidate: a long-tail, provider-mediated read-only glob head labeled `FileRead`; no broader filesystem family or package-manager scope. If accepted, it needs the mma-hol Strict gate at the director's gate. | M / decision |

## Capability-label review (whole-unit)

All 57 core specs are installed in both profiles. The standard sandbox inherits
`install_core_builtins = true`, then allows only `{Pure}` and removes file and
entropy providers; the trusted CLI installs core builtins into an unrestricted
runtime session and installs `StdFileIo` (`rum-host/src/config.rs:145-175`;
`rum-cli/src/main.rs:46-50`). Thus “denied” below means installed but rejected at
dispatch, not absent. Dispatch is the sole generic capability enforcement seam
(`eval_dispatch/mod.rs:111-138`).

| Builtin | Declared label | Actual touch / seam | Verdict | Installed/profile behavior today |
|---|---|---|---|---|
| `$Context` | Pure | Reads current session context (`eval_core/symbol.rs:5-8`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `$ContextPath` | Pure | Reads/writes snapshotable session lookup path (`eval_core/symbol.rs:8-15`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `$Packages` | Pure | Reads/writes session loaded-package list (`eval_core/symbol.rs:17-24`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Begin` | Pure | Mutates session context frames only (`session/session_api.rs:456-468`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `End` | Pure | Restores a session context frame (`session/session_api.rs:470-482`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `BeginPackage` | Pure | Mutates session context, then may call file-loading `eval_needs` directly (`builtins_context_loader.rs:284-315`). | **mismatched: internal FileRead bypass** | Sandbox: installed + allowed; standard profile lacks provider. CLI: installed + allowed with provider. |
| `EndPackage` | Pure | Restores session package frame/path (`session/session_api.rs:499-509`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Context` | Pure | Resolves session symbol/context metadata (`builtins_context_loader.rs:52-71`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Contexts` | Pure | Enumerates session-owned symbol/context state (`builtins_context_loader.rs:74-116`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `DeclarePackage` | Pure | Inserts session stubs; later stub evaluation calls `eval_needs` directly (`package_tools.rs:44-77`; `eval_core/symbol.rs:50-57`). | **mismatched: latent FileRead bypass** | Sandbox: installed + allowed; standard profile lacks provider. CLI: installed + allowed with provider. |
| `Needs` | FileRead | Probes/reads only via `FileIo` (`builtins_context_loader.rs:201-250`). | matches | Sandbox: installed + denied, no provider. CLI: installed + allowed with provider. |
| `ResolveContextAliases` | Pure | Inert session option marker; currently no consumer. | matches capability (option-truth gap) | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Names` | Pure | Enumerates session symbol tables only (`namespace_tools/names.rs:168-199`, `:249-292`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `NameQ` | Pure | Exact session symbol-existence lookup (`namespace_tools/names.rs:114-125`, `:284-292`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `SymbolName` | Pure | Pure symbol metadata projection (`namespace_tools/names.rs:98-112`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `$Path` | Pure | Reads session package-search paths; no probe (`eval_core/symbol.rs:34-40`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `$InputFileName` | Pure | Reads session load stack (`eval_core/symbol.rs:26-32`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `DirectoryName` | Pure | Lexical `Path` parent calculation only (`builtins_context_loader/file_names.rs:27-54`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `ParentDirectory` | Pure | Lexical `Path` parent calculation only (`builtins_context_loader/file_names.rs:56-76`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `FileNameJoin` | Pure | Lexical `PathBuf` construction only (`builtins_context_loader/file_names.rs:4-25`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `FileNameTake` | Pure | Lexical path component selection (`builtins_context_loader/file_names.rs:78-106`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `FindFile` | FileRead | Uses provider `probe` over session search roots (`resolution.rs:247-288`). | matches | Sandbox: installed + denied, no provider. CLI: installed + allowed with provider. |
| `File` | Pure | Inert file-spec data wrapper. | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Get` | FileRead | Provider read plus evaluation of source (`session/session_eval.rs:283-303`). | matches | Sandbox: installed + denied, no provider. CLI: installed + allowed with provider. |
| `Save` | FileWrite | Provider write in append mode (`package_tools.rs:542-557`). | matches | Sandbox: installed + denied, no provider. CLI: installed + allowed with provider. |
| `DumpSave` | FileWrite | Provider write in replace mode (`package_tools.rs:542-557`). | matches | Sandbox: installed + denied, no provider. CLI: installed + allowed with provider. |
| `AbsoluteTime` | Clock | Reads `SystemTime::now` (`debug_tools/timing.rs:10-21`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `AbsoluteTiming` | Clock | Reads `Instant` elapsed time (`debug_tools/timing.rs:50-75`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `Timing` | Clock | Reads `Instant` elapsed time (`debug_tools/timing.rs:50-75`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `RepeatedTiming` | Clock | Reads three `Instant` durations (`debug_tools/timing.rs:77-120`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `TimeConstrained` | Clock | Reads `Instant` against a cooperative deadline (`resources/constraints.rs:33-64`, `:144-151`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `Pause` | Clock | Sleeps the current thread for a duration (`builtins_input.rs:107-127`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `MemoryConstrained` | Pure | Uses deterministic session/Expr structural proxies, not process memory (`resources/constraints.rs:94-141`; `resources/memory.rs:49-90`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `MemoryInUse` | Diagnostic | Exposes current session memory-proxy internals (`resources/memory.rs:19-30`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `MaxMemoryUsed` | Diagnostic | Exposes session proxy high-water mark (`resources/memory.rs:33-47`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `ByteCount` | Pure | Computes a deterministic structural proxy from its argument (`resources/memory.rs:5-17`). | matches capability (contract-truth gap) | Sandbox: installed + allowed. CLI: installed + allowed. |
| `EvaluationData` | Clock | Captures `Instant` timing plus session diagnostics (`debug_tools/evaluation_data.rs:14-48`, `:83-177`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `Print` | Pure | Emits into session print response/sink (`builtins_input.rs:97-105`; `session/session_helpers.rs:176-182`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Echo` | Pure | Emits into session print response/sink (`debug_tools/basic.rs:30-49`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `EchoFunction` | Pure | Builds a pure function whose eventual `Echo` uses session output (`debug_tools/basic.rs:52-64`). | matches capability (form gap) | Sandbox: installed + allowed. CLI: installed + allowed. |
| `EchoLabel` | Pure | Builds a pure function using session output (`debug_tools/basic.rs:66-82`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Monitor` | Pure | Session evaluation scopes and session output only (`debug_tools/monitor.rs:42-80`, `:116-144`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Trace` | Pure | Captures session evaluator events (`debug_tools/trace.rs:150-212`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `TracePrint` | Pure | Captures events and routes lines through `push_print` (`debug_tools/trace.rs:195-212`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `Stack` | Pure | Reads current session evaluator frames (`debug_tools/basic.rs:84-97`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `RumProfileReset` | Diagnostic | Reads env-gated global profile configuration and resets global counters (`profiling/config.rs:3-23`, `:125-220`; `profiling/reset.rs:3-10`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `RumProfileSnapshot` | Diagnostic | Reads env-gated counters and may write diagnostics to stderr (`profiling/summary.rs:19-31`; `evaluation_cache/profile/report.rs:6-13`). | matches | Sandbox: installed + denied. CLI: installed + allowed. |
| `Input` | Pure | Creates host-input request, suspends, consumes host response (`builtins_input.rs:83-95`, `:224-233`). | **decision-candidate: host interaction kind** | Sandbox: installed + allowed/suspends. CLI: installed + allowed/suspends. |
| `InputString` | Pure | Creates host-input request, suspends, consumes host response (`builtins_input.rs:74-81`, `:224-233`). | **decision-candidate: host interaction kind** | Sandbox: installed + allowed/suspends. CLI: installed + allowed/suspends. |
| `Dialog` | Pure | May evaluate a body only; default body is `Input[""]` and suspends (`builtins_input.rs:39-71`, `:268-285`). | **decision-candidate: host interaction kind** | Sandbox: installed + allowed/suspends by default. CLI: installed + allowed/suspends by default. |
| `TestCreate` | Pure | Builds session-independent association data (`test_report/create.rs:4-60`). | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `TestReport` | Clock | Runs tests in session and unconditionally records `Instant` timing (`test_report/evaluation.rs:169-180`, `:232-309`; `test_report/report.rs:180-257`). | **decision-candidate: pure runner, timed report** | Sandbox: installed + denied. CLI: installed + allowed. |
| `$Aborted` | Pure | Inert data marker. | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `$Canceled` | Pure | Inert data marker. | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `ByteArray` | Pure | Inert data head; consumers inspect payload. | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `DiffObject` | Pure | Inert rum-native data head. | matches | Sandbox: installed + allowed. CLI: installed + allowed. |
| `TemplateObject` | Pure | Inert rum-native data head. | matches | Sandbox: installed + allowed. CLI: installed + allowed. |

The label count is therefore still the reference count—40 Pure, 8 Clock, 4
Diagnostic, 3 FileRead, and 2 FileWrite—but the behavioral review finds two
internal FileRead enforcement mismatches and four ordered kind decisions (the
three input heads plus `TestReport`). Output heads are certified Pure because
the runtime seam is session-mediated; profiling is deliberately different and
may write stderr only under a Diagnostic gate.

## Package-surface design input (parked SPF scoping half)

### Symbol creation and lookup

- Qualified names select their explicit normalized context. Unqualified lookup
  walks `$ContextPath` first and then `$Context`, deduplicating contexts; only
  if no existing symbol is found does creation choose the current context
  (`resolution.rs:100-127`). Declared stubs count as existing symbols
  (`resolution.rs:141-157`).
- `Begin[ctx]` expands a leading-backtick relative context against the current
  context, pushes both current context and path, makes `ctx` current, and
  prepends it to the path. `End[]` restores the exact saved pair and refuses a
  mismatched package frame (`context_and_load_helpers.rs:13-31`;
  `session/session_api.rs:456-482`).
- `$ContextPath`, `$Packages`, and `$Path` are intentionally writable
  snapshotable session variables. `$Context` and `$InputFileName` are derived
  reads (`eval_core/symbol.rs:4-41`; `definitions/definitions.rs:198-255`).

### Package entry, visibility, and loading

- `BeginPackage[ctx]` pushes a package frame, makes `ctx` current, sets the
  initial path to the target plus System context, and marks `ctx` loaded. Its optional
  dependency list is loaded with `Needs`; the final path is the target,
  dependencies, then the System context. `EndPackage[]` restores the prior current
  context and prepends the public package context to the prior path
  (`builtins_context_loader.rs:284-330`; `session/session_api.rs:484-509`).
- `Needs[ctx]` is idempotent by `$Packages`; otherwise it delegates to `Get`,
  marks the context loaded, and prepends it to `$ContextPath`. Context-shaped
  load targets expand to `.rum`, `.wl`, and `.m` candidates (flat,
  `Kernel/init`, then `init`) under the current load's parent followed by
  `$Path` roots (`builtins_context_loader.rs:201-250`;
  `context_and_load_helpers.rs:65-98`; `resolution.rs:232-288`).
- `Get` reads only through `FileIo`, pushes the resolved path so nested loads
  and `$InputFileName` work, evaluates forms sequentially, and rolls all
  loader/session definition state back on a syntax/evaluation error. Successful
  state—including an unbalanced context frame—persists (`session/session_eval.rs:226-303`;
  `session/session_helpers.rs:38-89`).
- `DeclarePackage[ctx,names]` creates declared stubs and prepends `ctx` to the
  path. First evaluation of an unresolved stub implicitly runs `Needs[ctx]`;
  this is the current lazy-autoload mechanism, not package metadata or a
  manager (`package_tools.rs:44-77`; `eval_core/symbol.rs:50-57`).

### Private convention and alias state

- The implemented Private-context convention is ordinary relative scoping:
  a leading-backtick `Private` spec under a package expands beneath that package. Visibility
  comes from context-path membership and qualified references; there is no
  separate public/private access-control layer (`context_and_load_helpers.rs:13-18`;
  `resolution.rs:100-127`).
- Persistence has one additional convention: DumpSave's generated package
  restore preamble excludes loaded contexts whose names end in
  `` `Private` `` (`package_tools.rs:418-447`, `:731-733`). This suffix check is
  not a general visibility rule.
- `ResolveContextAliases` is rum-native option vocabulary, not a current alias
  map. It is registered/defaulted but unused by `Names`; no context-alias
  representation or resolver exists in this surface (`options/defaults.rs:76-80`;
  `namespace_tools/names.rs:66-95`).

### Host lifecycle seam

Context/package state is included in host snapshots and restored on replay;
`reset()` rebuilds the session from its original configured context/path
(`rum-host/tests/lifecycle_contract.rs:166-204`;
`rum-host/src/tests.rs:933-975`). Ordinary request boundaries do not reset this
state, so an unbalanced successful `Begin` persists; the parked SPF/host design
must decide whether that is desired programmability or a boundary that needs a
diagnostic.

## Triage sketch

1. **Capability enforcement and kind decisions (highest safety value).** Add a
   capability-aware internal loader seam for `BeginPackage` dependencies and
   `DeclarePackage` stub autoload; separately take the `Input`/`InputString`/
   `Dialog` and `TestReport` label decisions using the `Exit` precedent. **A/B
   gate required** because stub autoload sits in symbol evaluation. **Needs the
   mma-hol Strict gate at the director's gate** because the fix touches
   `Needs`/loader behavior.
2. **Pause bounds hardening.** Replace the panicking float-to-duration
   conversion with checked validation and pin huge finite/negative cases. No
   evaluator A/B or mma-hol gate is indicated; run focused runtime/CLI tests.
3. **Package/load integrity.** Decide and then pin missing-dependency rollback
   and successful-unbalanced-context behavior; do not redesign SPF. **Needs the
   mma-hol Strict gate at the director's gate** because this changes
   `BeginPackage`/`Get`/`Needs` behavior. No hot-path A/B unless the chosen fix
   changes generic evaluation.
4. **Truthfulness and small usability forms.** Reject/remove the unused
   `ResolveContextAliases` and `CompressionLevel` controls, preserve symbols in
   the `DumpSave` result, align `ByteCount`'s contract, expose `TestCreate`
   metadata, add `Contexts` filtering, and correct `EchoFunction[f]`. These are
   separable S/M fixes; no A/B gate unless implementation moves into evaluator
   dispatch, and no mma-hol gate unless persistence round-trip loading changes.
5. **`FileNames` addition decision.** If the prior FS-purity removal is narrowed,
   add only a long-tail provider-mediated folk glob with `FileRead`, motivated
   by mma-hol `run_all.wls`; do not restore the stripped general FS family.
   **Needs the mma-hol Strict gate at the director's gate.** No evaluator A/B is
   indicated.

## Evidence appendix

All evaluator probes used the already-built local `target/debug/rum-cli`, fed
closed stdin, and were wrapped by `scripts/run-bounded.py`. Scratch loader and
persistence files lived only under `/tmp/rum-brief-223-20260713`. There were 40
evaluator invocations, below the approximate cap of 45. No external engine or
network source was used.

### E1 — inventory and registration spread

The count/equality transcript is in the opening section. The independent
runtime-catalog mapping found each authoritative name exactly once:

```text
$ python3 - <<'PY'
from pathlib import Path
import re
names = [
    '$Context', '$ContextPath', '$Packages', 'Begin', 'End', 'BeginPackage',
    'EndPackage', 'Context', 'Contexts', 'DeclarePackage', 'Needs',
    'ResolveContextAliases', 'Names', 'NameQ', 'SymbolName', '$Path',
    '$InputFileName', 'DirectoryName', 'ParentDirectory', 'FileNameJoin',
    'FileNameTake', 'FindFile', 'File', 'Get', 'Save', 'DumpSave',
    'AbsoluteTime', 'AbsoluteTiming', 'Timing', 'RepeatedTiming',
    'TimeConstrained', 'Pause', 'MemoryConstrained', 'MemoryInUse',
    'MaxMemoryUsed', 'ByteCount', 'EvaluationData', 'Print', 'Echo',
    'EchoFunction', 'EchoLabel', 'Monitor', 'Trace', 'TracePrint', 'Stack',
    'RumProfileReset', 'RumProfileSnapshot', 'Input', 'InputString', 'Dialog',
    'TestCreate', 'TestReport', '$Aborted', '$Canceled', 'ByteArray',
    'DiffObject', 'TemplateObject',
]
spec_re = re.compile(r'BuiltinSpec::[A-Za-z_]+\(\s*"([^"]+)"')
hits = {name: [] for name in names}
for path in sorted(Path('crates/rum-runtime/src/runtime/builtin_catalog').glob('*.rs')):
    for name in spec_re.findall(path.read_text()):
        if name in hits:
            hits[name].append(path.name)
by_file = {}
for name in names:
    assert len(hits[name]) == 1, (name, hits[name])
    by_file.setdefault(hits[name][0], []).append(name)
for file, members in sorted(by_file.items()):
    print(f'{file}: {len(members)}: {", ".join(members)}')
print(f'REGISTRATION-SPREAD-OK {sum(map(len, by_file.values()))}/57')
PY
debug.rs: 21: AbsoluteTime, AbsoluteTiming, Timing, RepeatedTiming, TimeConstrained, MemoryConstrained, MemoryInUse, MaxMemoryUsed, ByteCount, EvaluationData, Echo, EchoFunction, EchoLabel, Monitor, Trace, TracePrint, Stack, RumProfileReset, RumProfileSnapshot, TestCreate, TestReport
namespace.rs: 28: $Context, $ContextPath, $Packages, Begin, End, BeginPackage, EndPackage, Context, Contexts, DeclarePackage, Needs, ResolveContextAliases, Names, NameQ, SymbolName, $Path, $InputFileName, DirectoryName, ParentDirectory, FileNameJoin, FileNameTake, FindFile, File, Get, Save, DumpSave, $Aborted, $Canceled
scoping.rs: 5: Pause, Print, Input, InputString, Dialog
strings.rs: 3: ByteArray, DiffObject, TemplateObject
REGISTRATION-SPREAD-OK 57/57
```

Registration/labels are visible at
`crates/rum-runtime/src/runtime/builtin_catalog/namespace.rs:3-146`,
`crates/rum-runtime/src/runtime/builtin_catalog/debug.rs:3-160`,
`crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs:97-123`, and
`crates/rum-runtime/src/runtime/builtin_catalog/strings.rs:105-116`.

The reference-block check printed:

```text
attribute entries present = 57/57
without Protected = ['$ContextPath', '$Packages', '$Path']
ATTRIBUTE-HEADSTART-OK 57/57
```

### E2 — contexts, symbol names, and pure path math

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval '{ $Context, $ContextPath, Begin["Audit`"], $Context, $ContextPath, Begin["`Private`"], $Context, End[], $Context, End[], $Context, End[] }'
! End::context: no matching Begin[] frame is active
! End::context: no matching Begin[] frame is active
List["Global`", List["System`", "Global`"], "Audit`", "Audit`", List["Audit`", "System`", "Global`"], "Audit`Private`", "Audit`Private`", "Audit`Private`", "Audit`", "Audit`", "Global`", End[]]
```

The repeated message is fixed-point re-evaluation of the inert unmatched call,
not two shell invocations.

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'Audit`foo = 1; Global`foo = 2; $ContextPath = {"Audit`", "Global`", "System`"}; List[Context[], Context[Audit`foo], Context["foo"], Contexts[], Names["Audit`*"], Names["foo"], NameQ["Audit`foo"], NameQ["Audit`missing"], SymbolName[Audit`unassigned], SymbolName[Global`unassigned]]'
List["Global`", "Audit`", "Audit`", List["Audit`", "Global`", "System`"], List["Audit`foo"], List["foo"], True, False, "unassigned", "unassigned"]

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'Audit`foo = 1; List[Options[Names], Names["Audit`*", ResolveContextAliases -> True], Names["Audit`*", ResolveContextAliases -> False]]'
List[List[Rule[IgnoreCase, False], Rule[ResolveContextAliases, True], Rule[SpellingCorrection, False]], List["Audit`foo"], List["Audit`foo"]]

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[$Path, $InputFileName, FileNameJoin[{"a", "b", "c.txt"}], DirectoryName["a/b/c.txt"], ParentDirectory["a/b/"], ParentDirectory["/definitely/missing/rum-audit/child/"], FileNameTake["a/b/c.txt"], FileNameTake["a/b/c.txt", -2], FileNameTake["a/b/c.txt", 2], File["x.rum"]]'
List[List["."], "", "a/b/c.txt", "a/b/", "a/", "/definitely/missing/rum-audit/", "c.txt", "b/c.txt", "a/b", File["x.rum"]]
```

The source-visible counterpart is:

- lookup/create: `crates/rum-runtime/src/runtime/resolution.rs:100-157`;
- context stack: `crates/rum-runtime/src/runtime/session/session_api.rs:456-509`;
- `Contexts`: `crates/rum-runtime/src/runtime/builtins_context_loader.rs:74-116`;
- `Names` option reads: `crates/rum-runtime/src/runtime/namespace_tools/names.rs:66-95`,
  `:168-199`, `:242-246`;
- lexical path helpers: `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs:3-107`.

### E3 — loader/package probes under `/tmp`

Fixture contents:

```text
# /tmp/rum-brief-223-20260713/load-root/AuditPkg.rum
BeginPackage["AuditPkg`"]
public::usage = "public value"
public = 41
inputPath = $InputFileName
Begin["`Private`"]
helper[x_] := x + 1
End[]
publicFn[x_] := AuditPkg`Private`helper[x]
EndPackage[]

# /tmp/rum-brief-223-20260713/load-root/DepPkg.rum
BeginPackage["DepPkg`"]
dep = 5
EndPackage[]

# /tmp/rum-brief-223-20260713/load-root/malformed.rum
Begin["Broken`"]
brokenValue = 99
1 ! 2

# /tmp/rum-brief-223-20260713/load-root/unbalanced.rum
Begin["Leaked`"]
leakedValue = 23
```

`$Path`, `Needs` idempotence, private visibility, and `$InputFileName`:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval '$Path = {"/tmp/rum-brief-223-20260713/load-root"}; List[FindFile["AuditPkg`"], Needs["AuditPkg`"], Needs["AuditPkg`"], $Packages, $ContextPath, AuditPkg`public, AuditPkg`Private`helper[4], AuditPkg`inputPath, $InputFileName]'
List["/private/tmp/rum-brief-223-20260713/load-root/AuditPkg.rum", Null, Null, List["AuditPkg`"], List["AuditPkg`", "System`", "Global`"], 41, 5, "/private/tmp/rum-brief-223-20260713/load-root/AuditPkg.rum", ""]
exit=0
```

Missing/malformed failures and rollback:

The first bounded pass emitted `FindFile::notfound`, `Get::notfound`, and
`Get::syntax`; all three calls returned `$Failed`, the current context remained
Global, and `NameQ` was `False` for the Broken-context symbol. The fail-closed branches
are at `crates/rum-runtime/src/runtime/builtins_context_loader.rs:118-198`; full
loader-state rollback is at
`crates/rum-runtime/src/runtime/session/session_eval.rs:283-303` and
`session/session_helpers.rs:38-89`.

Dependency loading and lazy `DeclarePackage` autoload:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval '$Path = {"/tmp/rum-brief-223-20260713/load-root"}; List[BeginPackage["Main`", {"DepPkg`"}], $Context, $ContextPath, $Packages, DepPkg`dep, EndPackage[], $Context, $ContextPath]'
List["Main`", "Main`", List["Main`", "DepPkg`", "System`"], List["Main`", "DepPkg`"], 5, Null, "Global`", List["Main`", "System`", "Global`"]]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval '$Path = {"/tmp/rum-brief-223-20260713/load-root"}; List[DeclarePackage["AuditPkg`", {"public"}], $ContextPath, $Packages, AuditPkg`public, $Packages]'
List["AuditPkg`", List["AuditPkg`", "System`", "Global`"], List[], 41, List["AuditPkg`"]]
exit=0
```

A successful unbalanced load persists its context:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[Get["/tmp/rum-brief-223-20260713/load-root/unbalanced.rum"], $Context, $ContextPath, leakedValue]'
List[23, "Leaked`", List["Leaked`", "System`", "Global`"], 23]
```

A missing `BeginPackage` dependency reports failure but package entry continues:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[BeginPackage["FailMain`", {"DefinitelyMissing`"}], $Context, $ContextPath, $Packages, EndPackage[], $Context]'
List["FailMain`", "FailMain`", List["FailMain`", "DefinitelyMissing`", "System`"], List["FailMain`"], Null, "Global`"]
! Get::notfound: could not locate `DefinitelyMissing`` in ./DefinitelyMissing.rum, ./DefinitelyMissing/Kernel/init.rum, ./DefinitelyMissing/init.rum, ./DefinitelyMissing.wl, ./DefinitelyMissing/Kernel/init.wl, ./DefinitelyMissing/init.wl, ./DefinitelyMissing.m, ./DefinitelyMissing/Kernel/init.m, ./DefinitelyMissing/init.m
! Needs::load: context `DefinitelyMissing`` could not be loaded
exit=0
```

The critical enforcement facts are code-visible: the generic check is only at
`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:111-138`, while the two
direct loader calls are at
`crates/rum-runtime/src/runtime/builtins_context_loader.rs:306-313` and
`crates/rum-runtime/src/runtime/eval_core/symbol.rs:50-57`. Standard sandbox
double-defense and trusted provider setup are at
`crates/rum-host/src/config.rs:145-175`; the `FileIo` trait and ambient host
implementation are separated at `crates/rum-runtime/src/runtime/file_io.rs:64-100`
and `crates/rum-host/src/file_io.rs:8-60`.

### E4 — persistence probes under `/tmp`

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'saveA = 1; r1 = Save["/tmp/rum-brief-223-20260713/save-evidence.rum", saveA]; saveA = 2; r2 = Save["/tmp/rum-brief-223-20260713/save-evidence.rum", saveA]; List[r1, r2, saveA, Get["/tmp/rum-brief-223-20260713/save-evidence.rum"]]'
List[Null, Null, 2, 2]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'dumpB = 1; DumpSave["/tmp/rum-brief-223-20260713/dump-evidence.rum", dumpB]; dumpB = 2; r = DumpSave["/tmp/rum-brief-223-20260713/dump-evidence.rum", dumpB]; Clear[dumpB]; loaded = Get["/tmp/rum-brief-223-20260713/dump-evidence.rum"]; List[r, loaded, dumpB]'
List[List[2], 2, 2]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'compressionValue = 7; List[DumpSave["/tmp/rum-brief-223-20260713/compress-evidence0.rum", compressionValue, {CompressionLevel -> 0}], DumpSave["/tmp/rum-brief-223-20260713/compress-evidence9.rum", compressionValue, {CompressionLevel -> 9}]]'
List[List[7], List[7]]
exit=0
```

The captured files were:

```text
--- /tmp/rum-brief-223-20260713/save-evidence.rum
Set[saveA, 1]
Set[saveA, 2]
--- /tmp/rum-brief-223-20260713/dump-evidence.rum
Set[dumpB, 2]
--- /tmp/rum-brief-223-20260713/compress-evidence0.rum
Set[compressionValue, 7]
--- /tmp/rum-brief-223-20260713/compress-evidence9.rum
Set[compressionValue, 7]
compression-files-identical=yes
```

Persistence option defaults are at
`crates/rum-runtime/src/runtime/options/defaults.rs:64-71`; writing modes and
the ordinary-list return are at
`crates/rum-runtime/src/runtime/package_tools.rs:220-246`, `:542-557`. A source
search found `CompressionLevel` only in defaults/syntax discovery, while
`SymbolAttributes` is consumed at `package_tools.rs:633-640`.

### E5 — timing, constraints, memory, and `Pause`

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[AbsoluteTime[], Timing[1 + 1], AbsoluteTiming[2 + 2], RepeatedTiming[3 + 3], EvaluationData[4 + 4]]'
List[1783956602528247/1000000, List[1411/25000000, 2], List[10711/1000000000, 4], List[1989/200000000, 6], Association[Rule["Input", Hold[Plus[4, 4]]], Rule["Result", 8], Rule["Success", True], Rule["FailureType", None], Rule["Messages", List[]], Rule["MessagesExpressions", List[]], Rule["MessagesText", List[]], Rule["OutputLog", List[]], Rule["Trace", List[Hold[Plus[4, 4]], Hold[8]]], Rule["Timing", 45403/500000000], Rule["AbsoluteTiming", 45403/500000000], Rule["InputString", "Plus[4, 4]"]]]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[TimeConstrained[Pause[1/100], 1, $Aborted], TimeConstrained[Pause[1], 1/100, $Aborted], MemoryConstrained[Range[10], 100000, $Aborted], MemoryConstrained[Range[1000], 1, $Aborted]]'
List[Null, $Aborted, List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], $Aborted]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[ByteCount[1], ByteCount[123456789], ByteCount["a"], ByteCount["abcdefghij"], MemoryInUse[], MaxMemoryUsed[]]'
List[32, 32, 33, 42, 53397, 53399]
exit=0
```

Invalid small bounds fail inert with diagnostics:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[Pause[-1], Pause[1/0], TimeConstrained[1 + 1, -1, $Aborted], MemoryConstrained[1 + 1, -1, $Aborted]]'
List[Pause[-1], Pause[Power[0, -1]], TimeConstrained[Plus[1, 1], -1, $Aborted], MemoryConstrained[Plus[1, 1], -1, $Aborted]]
! Pause::numnm: Non-negative machine-sized number expected at position 1 in Pause[-1].
! Pause::numnm: Non-negative machine-sized number expected at position 1 in Pause[Power[0, -1]].
! Pause::numnm: Non-negative machine-sized number expected at position 1 in Pause[Power[0, -1]].
! TimeConstrained::argform: expected the timeout argument to be a non-negative exact number in the current first-pass subset
! MemoryConstrained::argform: expected the memory limit argument to be a non-negative exact integer
! Pause::numnm: Non-negative machine-sized number expected at position 1 in Pause[-1].
! Pause::numnm: Non-negative machine-sized number expected at position 1 in Pause[Power[0, -1]].
! TimeConstrained::argform: expected the timeout argument to be a non-negative exact number in the current first-pass subset
! MemoryConstrained::argform: expected the memory limit argument to be a non-negative exact integer
exit=0
```

The huge finite `Pause` was run once and not retried:

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'Pause[10^100]'

thread 'main' panicked at /Users/fft/.rustup/toolchains/1.85.0-x86_64-apple-darwin/lib/rustlib/src/rust/library/core/src/time.rs:944:23:
cannot convert float seconds to Duration: value is either too big or NaN
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
exit=101
```

Timing code is at
`crates/rum-runtime/src/runtime/debug_tools/timing.rs:3-143`; constraint and
fallback code is at
`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:4-269`;
the deterministic proxy and the `ByteCount` implementation are at
`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:4-90`,
`:257-308`. The panicking unchecked conversion is
`crates/rum-runtime/src/runtime/builtins_input.rs:419-434`.

### E6 — diagnostics, output, traces, and profiling-off shape

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[Print["p", 1], Echo[42], Echo[7, "tag"], EchoFunction[][1 + 1], EchoFunction[Function[Plus[Slot[1], 1]]][2], EchoLabel["sum"][1 + 1]]'
p1
42
tag: 7
2
Function[Plus[2, 1]]: 2
sum: 2
List[Null, 42, 7, 2, 2, 2]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[Monitor[(x = 1; x + 1), x], Trace[1 + 1], TracePrint[2 + 2], Stack[]]'
x
1
Plus[2, 2]
4
List[2, List[Hold[Plus[1, 1]], Hold[2]], 4, List[]]
exit=0

$ printf '' | env -u RUM_PROFILE_LEXICAL_SUBSTITUTION -u RUM_PROFILE_EVALUATION_CACHE scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[RumProfileReset[], RumProfileSnapshot["audit"]]'
List[Null, Null]
exit=0
```

Kernel output is routed by
`crates/rum-runtime/src/runtime/session/session_helpers.rs:176-182`; the public
host response carries prints separately (`crates/rum-host/tests/adapter_contract.rs:7-38`).
Profiling uses a cold `AtomicU8` env-load pattern
(`crates/rum-runtime/src/runtime/profiling/config.rs:3-23`, `:125-220`) and its
off-path reset/snapshot returns before counter locks or stderr
(`profiling/reset.rs:3-6`; `profiling/summary.rs:19-31`;
`evaluation_cache/profile/report.rs:6-9`).

### E7 — interactive input, each under its own closed-stdin command

```text
$ printf '' | scripts/run-bounded.py --timeout 15 -- target/debug/rum-cli --eval 'Input[]'
! input required: ?
error: input required: ?
exit=1

$ printf '' | scripts/run-bounded.py --timeout 15 -- target/debug/rum-cli --eval 'InputString["p> "]'
! input required: p> 
error: input required: p> 
exit=1

$ printf '' | scripts/run-bounded.py --timeout 15 -- target/debug/rum-cli --eval 'Dialog[]'
! input required
error: input required
exit=1

$ printf '' | scripts/run-bounded.py --timeout 15 -- target/debug/rum-cli --eval 'Dialog[Return[2]]'
2
exit=0
```

The runtime creates the request and suspends at
`crates/rum-runtime/src/runtime/builtins_input.rs:39-95`, `:173-233`; the
top-level request preserves continuation state at
`crates/rum-runtime/src/runtime/session/session_requests.rs:108-224`. rum-host
maps it to typed `HostEvalOutcome::InputRequired` and resumes explicitly
(`crates/rum-host/src/model/outcome.rs:9-18`, `:302-320`;
`crates/rum-host/src/session.rs:267-287`, `:338-354`), with public contract tests
at `crates/rum-host/tests/adapter_contract.rs:41-71` and
`crates/rum-host/src/tests.rs:645-663`. The earlier user-decided lifecycle
precedent is recorded in `docs/dev/codex/brief-183-exit-capability.md:1-80`.

### E8 — tests, markers/data heads, and absent `FileNames`

```text
$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[Head[TestCreate[1 + 1, 2]], TestCreate[1 + 1, 2, {TestID -> "sum"}], Options[TestCreate], Options[TestReport]]'
List[Association, Association[Rule["Input", Hold[Plus[1, 1]]], Rule["ExpectedOutput", Hold[2]], Rule["TestID", "sum"]], List[], List[Rule[SameTest, SameQ], Rule[TimeConstraint, Infinity], Rule[MemoryConstraint, Infinity]]]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], "TestsRunCount"], Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], "TestsFailedCount"], TestReport[{TestCreate[1 + 1, 3]}, {"Success", "TestResults"}]]'
List[2, 1, Association[Rule["Success", False], Rule["TestResults", List[Association[Rule["Input", Hold[Plus[1, 1]]], Rule["Success", False], Rule["ActualOutput", 2], Rule["ExpectedOutput", Hold[3]], Rule["ActualMessages", List[]], Rule["ActualMessagesExpressions", List[]], Rule["ActualMessagesText", List[]], Rule["ExpectedMessages", List[]], Rule["OutputLog", List[]], Rule["Timing", 10447/500000000], Rule["MessagesMatch", True], Rule["Outcome", "Failure"]]]]]]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[$Aborted, $Canceled, Head[$Aborted], Head[$Canceled], ByteArray[{1, 2, 255}], DiffObject["a", "b"], TemplateObject["x"]]'
List[$Aborted, $Canceled, Symbol, Symbol, ByteArray[List[1, 2, 255]], DiffObject["a", "b"], TemplateObject["x"]]
exit=0

$ printf '' | scripts/run-bounded.py --timeout 120 -- target/debug/rum-cli --eval 'List[NameQ["FileNames"], FileNames["*.rum"]]'
List[False, FileNames["*.rum"]]
exit=0
```

`TestCreate`'s association builder and option parser are at
`crates/rum-runtime/src/runtime/debug_tools/test_report/create.rs:4-60` and
`test_report/parsing.rs:69-97`; report failure detail/timing is assembled at
`test_report/report.rs:180-285`. The absence and acceptance-suite consumer are
recorded at `PROGRESS.md:634-650` and `docs/dev/usability-pass.md:325-334`.

## Stop-loss and open questions

- Stop-loss triggered once: `Pause[10^100]` panicked, so no second huge-duration
  variation was attempted. No input probe hung, and no probe misbehaved twice.
- Open decision: what capability kind, if any, should represent resumable host
  interaction for `Input`, `InputString`, and `Dialog`? This must follow the
  same user/director decision process as `Exit`.
- Open decision: should `TestReport` keep mandatory timing/`Clock`, make timing
  optional, or split timing from the pure runner?
- Open decision: should successful loads with unbalanced context frames remain
  legal persistent session mutation, or fail/restore at the loader/request
  boundary?
- Open decision: should the prior FS-purity ruling admit the narrow
  provider-mediated `FileNames` candidate required by the folk discovery idiom?

## Verification outputs

The mandatory commands and coverage script were run after the findings file was
complete. Outputs follow verbatim.

```text
$ grep -c '^### ' docs/reference/builtins.md
612
```

The full 453/159/55/47/57 derivation and complement equality assertion appear
verbatim in the opening section. Findings coverage was checked independently of
the capability table; repeated names are intentional extra gap rows, and the
only outside-unit row is the required `FileNames` candidate:

```text
$ python3 - <<'PY'
from collections import Counter
from pathlib import Path
import re

expected = [
    '$Context', '$ContextPath', '$Packages', 'Begin', 'End', 'BeginPackage',
    'EndPackage', 'Context', 'Contexts', 'DeclarePackage', 'Needs',
    'ResolveContextAliases', 'Names', 'NameQ', 'SymbolName', '$Path',
    '$InputFileName', 'DirectoryName', 'ParentDirectory', 'FileNameJoin',
    'FileNameTake', 'FindFile', 'File', 'Get', 'Save', 'DumpSave',
    'AbsoluteTime', 'AbsoluteTiming', 'Timing', 'RepeatedTiming',
    'TimeConstrained', 'Pause', 'MemoryConstrained', 'MemoryInUse',
    'MaxMemoryUsed', 'ByteCount', 'EvaluationData', 'Print', 'Echo',
    'EchoFunction', 'EchoLabel', 'Monitor', 'Trace', 'TracePrint', 'Stack',
    'RumProfileReset', 'RumProfileSnapshot', 'Input', 'InputString', 'Dialog',
    'TestCreate', 'TestReport', '$Aborted', '$Canceled', 'ByteArray',
    'DiffObject', 'TemplateObject',
]
text = Path('docs/dev/usability-pass/kernel-c-findings.md').read_text()
section = text.split('## Findings table\n', 1)[1].split(
    '## Capability-label review (whole-unit)\n', 1
)[0]
rows = re.findall(r'^\| `([^`]+)` \|', section, re.M)
counts = Counter(rows)
seen_order = []
for name in rows:
    if name in expected and name not in seen_order:
        seen_order.append(name)
missing = sorted(set(expected) - set(rows))
extras = sorted(set(rows) - set(expected))
duplicate_coverage = sorted(
    name for name, count in counts.items() if name in expected and count > 1
)
assert len(expected) == len(set(expected)) == 57
assert not missing, missing
assert extras == ['FileNames'], extras
assert seen_order == expected
print(f'findings rows = {len(rows)}')
print(f'authoritative names covered = {len(set(rows) & set(expected))}')
print(f'additional gap rows = {sum(counts[name] - 1 for name in expected)}')
print(f'duplicate authoritative names with extra gap rows = {duplicate_coverage}')
print(f'outside-unit candidate rows = {extras}')
print('COVERAGE-OK 57/57')
PY
findings rows = 65
authoritative names covered = 57
additional gap rows = 7
duplicate authoritative names with extra gap rows = ['BeginPackage', 'DumpSave', 'Get', 'Input', 'TestCreate', 'TestReport']
outside-unit candidate rows = ['FileNames']
COVERAGE-OK 57/57
```

The dedicated capability matrix was also checked for exact count and order:

```text
CAPABILITY-REVIEW-OK 57/57
```

```text
$ git status --short
?? docs/dev/usability-pass/kernel-c-findings.md

$ scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

Both commands exited 0. No code or behavior was changed, so no test suite was
required by this audit brief.
