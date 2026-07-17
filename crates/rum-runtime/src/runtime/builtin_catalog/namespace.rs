use super::message_templates::*;
use super::*;

pub(super) const NAMESPACE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new(
        "$Context",
        "Evaluates to the current default context name as a string.",
    ).protected(),
    BuiltinSpec::new(
        "$ContextPath",
        "Evaluates to the active context search path as a list of strings.",
    ),
    BuiltinSpec::new(
        "$Aborted",
        "Represents a computation aborted by Abort or an external interrupt.",
    ).protected(),
    BuiltinSpec::new(
        "$Canceled",
        "Represents a canceled operation for confirmation-flow compatibility.",
    ).protected(),
    BuiltinSpec::new("$Failed", "Represents a failed operation.").protected(),
    BuiltinSpec::new(
        "$MessageList",
        "Lists the message names emitted by the most recent top-level evaluation.",
    ).protected(),
    BuiltinSpec::new(
        "$Packages",
        "Lists package contexts loaded into the current session.",
    ),
    BuiltinSpec::new(
        "$InputFileName",
        "Evaluates to the path of the file currently being loaded, or the empty string at top level.",
    ).protected(),
    BuiltinSpec::new("$Path", "Lists search roots used by the package loader."),
    BuiltinSpec::new(
        "Nothing",
        "Represents a list element that is removed from evaluated Lists.",
    ).protected(),
    BuiltinSpec::ordinary(
        "FileNameJoin",
        "Joins path components into a platform path string.",
        builtin_file_name_join,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a list of string path components",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "DirectoryName",
        "Returns a parent directory path string for a file name.",
        builtin_directory_name,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string file name",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "ParentDirectory",
        "Returns the parent directory for an existing directory path.",
        builtin_parent_directory,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string directory name",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "FileNameTake",
        "Takes file name components from a path string.",
        builtin_file_name_take,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string file name",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Context",
        "Returns the context of the current session or of a symbol.",
        HoldAttribute::HoldFirst,
        builtin_context,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or string argument",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Contexts",
        "Lists known contexts in the current session.",
        builtin_contexts,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string context-name pattern",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Names",
        "Lists symbol names matching a current first-pass name pattern subset.",
        builtin_names,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string pattern, RegularExpression, StringExpression, Alternatives, or a list of them",
        },
        BuiltinMessageSpec {
            tag: "optval",
            template: "Reports an unsupported Names option value.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_attributes(
        "SymbolName",
        "Returns the short name of a symbol without its context.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_symbol_name,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "NameQ",
        "Tests whether a string is the exact name of an existing symbol.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_name_q,
    ),
    BuiltinSpec::new(
        "ResolveContextAliases",
        "Represents the current Names option symbol for context-alias resolution.",
    ).protected(),
    BuiltinSpec::new(
        "SpellingCorrection",
        "Represents the current Names option symbol for spelling-correction control.",
    ).protected(),
    BuiltinSpec::new("File", "Represents an explicit file specification.").protected(),
    BuiltinSpec::ordinary(
        "FindFile",
        "Finds the full path of a file or package using the current search path.",
        builtin_find_file,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string or File[\"...\"] load target",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "notfound",
            template: "Reports that the requested file could not be located.",
        },
    ])
    .protected()
    .with_capability(Capability::FileRead),
    BuiltinSpec::ordinary(
        "FileNames",
        "Lists sorted file paths whose names match a folk wildcard pattern.",
        builtin_file_names,
    )
    .with_messages(&[
        ARGTU,
    ])
    .protected()
    .with_capability(Capability::FileRead),
    BuiltinSpec::ordinary(
        "Get",
        "Reads a file or package and evaluates the expressions it contains.",
        builtin_get,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string or File[\"...\"] load target",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "io",
            template: "could not read `1`: `2`",
        },
        BuiltinMessageSpec {
            tag: "load",
            template: "Reports a loader failure while reading or evaluating a file.",
        },
        BuiltinMessageSpec {
            tag: "notfound",
            template: "Reports that the requested file could not be located.",
        },
        BuiltinMessageSpec {
            tag: "syntax",
            template: "Reports a syntax error while evaluating a loaded file.",
        },
    ])
    .protected()
    .with_capability(Capability::FileRead),
    BuiltinSpec::ordinary(
        "Needs",
        "Loads a package only if its context has not already been loaded.",
        builtin_needs,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid context name or explicit load target.",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected `1` argument(s), got `2`",
        },
        BuiltinMessageSpec {
            tag: "load",
            template: "context `1` could not be loaded",
        },
    ])
    .protected()
    .with_capability(Capability::FileRead),
    BuiltinSpec::ordinary(
        "Begin",
        "Switches into a context and prepends it to the context path.",
        builtin_begin,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a string context name",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "End",
        "Restores the previous context after a matching Begin.",
        builtin_end,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "context",
            template: "no matching Begin[] frame is active",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "BeginPackage",
        "Starts a package context with a package-oriented context path.",
        builtin_begin_package,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid package context or dependency list.",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected `1` argument(s), got `2`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "EndPackage",
        "Ends a package context and leaves it on the restored context path.",
        builtin_end_package,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "context",
            template: "no matching BeginPackage[] frame is active",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "DeclarePackage",
        "Declares stub names for a package context and prepends that context to the search path.",
        builtin_declare_package,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid package context or symbol-name list.",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Save",
        "Appends a textual first-pass serialization of selected symbols to a file.",
        HoldAttribute::HoldRest,
        builtin_save,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid path, object specification, context filter, or option list.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "io",
            template: "could not write `1`: `2`",
        },
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected()
    .with_capability(Capability::FileWrite),
    BuiltinSpec::ordinary_with_hold(
        "DumpSave",
        "Writes a textual first-pass dump of selected symbols to a file and returns the saved symbol list.",
        HoldAttribute::HoldRest,
        builtin_dump_save,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid path, object specification, context filter, or option list.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "io",
            template: "could not write `1`: `2`",
        },
        BuiltinMessageSpec {
            tag: "optval",
            template: "Unsupported value `1` for option CompressionLevel in DumpSave; only 1 is currently supported.",
        },
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected()
    .with_capability(Capability::FileWrite),
    BuiltinSpec::ordinary_with_hold(
        "Once",
        "Evaluates an expression at most once per session-local cache key in the current first-pass subset.",
        HoldAttribute::HoldAll,
        builtin_once,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid option specification.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "pers",
            template: "Unsupported value `1` for option `2` in Once; supported values are ExpirationDate -> None and PersistenceTime -> None or 0.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Message",
        "Emits a message described by a MessageName expression.",
        HoldAttribute::HoldFirst,
        builtin_message,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the first argument to be a MessageName expression",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected at least 1 argument",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "MessageName",
        "Represents a symbolic message name such as f::tag.",
        HoldAttribute::HoldAll,
        builtin_message_name,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string, plus a message tag string or symbol",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Messages",
        "Lists message names defined for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        builtin_messages,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "MessageDefinitions",
        "Lists stored message template definitions for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        builtin_message_definitions,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "MessageTags",
        "Lists message tags defined globally, for a symbol, or for one message query.",
        HoldAttribute::HoldAll,
        builtin_message_tags,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Information",
        "Returns a first-pass InformationData summary for a symbol or symbol-name string.",
        HoldAttribute::HoldFirst,
        Session::eval_information,
    )
    .with_messages(&[
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Definition",
        "Returns a held first-pass expression containing visible definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_definition,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ValueQ",
        "Tests whether an expression has a value by trial evaluation without committing side effects.",
        HoldAttribute::HoldFirst,
        builtin_value_q,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "DefaultValues",
        "Lists stored Default[...] definitions for a symbol in the current optional-pattern subset.",
        HoldAttribute::HoldFirst,
        builtin_default_values,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "OwnValues",
        "Lists stored immediate or delayed ownvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_own_values,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "setval",
            template: "Reports an invalid OwnValues assignment.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "UpValues",
        "Lists stored tagged upvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_up_values,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "setval",
            template: "Reports an invalid UpValues assignment.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "DownValues",
        "Lists stored downvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_down_values,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "setval",
            template: "Reports an invalid DownValues assignment.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SubValues",
        "Lists stored subvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_sub_values,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "setval",
            template: "Reports an invalid SubValues assignment.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Attributes",
        "Lists the currently active attributes for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_attributes,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SyntaxInformation",
        "Lists the current syntax-information metadata rules for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_syntax_information,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Options",
        "Lists the current default option rules for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_options,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "OptionValue",
        "Looks up an option value from explicit rules or a symbol's defaults.",
        HoldAttribute::HoldAll,
        builtin_option_value,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid option target, option name, or explicit option rules.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "nodef",
            template: "Unknown option `1` for `2`.",
        },
        BuiltinMessageSpec {
            tag: "optnf",
            template: "Option name `1` not found in defaults for `2`.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ArgumentsOptions",
        "Partitions a held call into positional arguments and trailing option rules.",
        HoldAttribute::HoldFirst,
        builtin_arguments_options,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid held call, argument-count specification, or result association.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "CheckArguments",
        "Checks whether a held call has an allowed positional argument count and valid trailing options.",
        HoldAttribute::HoldFirst,
        builtin_check_arguments,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid held call, argument-count specification, or result association.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SetAttributes",
        "Adds attributes to one or more symbols.",
        HoldAttribute::HoldFirst,
        builtin_set_attributes,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid target or attribute specification.",
        },
        ARGX_EXACT,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ClearAttributes",
        "Removes attributes from one or more symbols.",
        HoldAttribute::HoldFirst,
        builtin_clear_attributes,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid target or attribute specification.",
        },
        ARGX_EXACT,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SetOptions",
        "Updates known default options for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_set_options,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid target or option-rule specification.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "optnf",
            template: "`1` is not a known option for `2`.",
        },
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Protect",
        "Adds the Protected attribute to one or more symbols.",
        HoldAttribute::HoldAll,
        builtin_protect,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Unprotect",
        "Removes the Protected attribute from one or more symbols.",
        HoldAttribute::HoldAll,
        builtin_unprotect,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Unset",
        "Removes a specific stored definition or message template.",
        HoldAttribute::HoldFirst,
        builtin_unset,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol, definition lhs, message name, or tagged definition target",
        },
        ARGX_EXACT,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Clear",
        "Clears values and rules from one or more symbols.",
        HoldAttribute::HoldAll,
        builtin_clear,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ClearAll",
        "Clears values, rules, messages, and user-defined attributes from one or more symbols.",
        HoldAttribute::HoldAll,
        builtin_clear_all,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Remove",
        "Removes the current stored state for one or more symbols in the first-pass namespace subset.",
        HoldAttribute::HoldAll,
        builtin_remove,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
        BuiltinMessageSpec {
            tag: "rmptc",
            template: "Symbol `1` is Protected and cannot be removed.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Check",
        "Evaluates an expression and substitutes a fallback if matching messages are emitted.",
        HoldAttribute::HoldAll,
        builtin_check,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the optional message selector to be a symbol, a \"symbol::tag\" string, or a list of those",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 2 or 3 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Quiet",
        "Evaluates an expression while suppressing matching runtime messages it emits.",
        HoldAttribute::HoldAll,
        builtin_quiet,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the optional message selector to be a symbol, a \"symbol::tag\" string, or a list of those",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 1 or 2 argument(s), got `1`",
        },
    ])
    .protected(),
];
