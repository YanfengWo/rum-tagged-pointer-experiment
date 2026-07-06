# Text Utilities

`rum` includes pure scripting helpers for shaping text, shell-style words,
URLs, path strings, HTML snippets, and simple script arguments.

The helpers in this chapter are deterministic string and list transforms. They
do not run shell commands, inspect the file system, or perform network I/O.

## Wrapping, Indenting, And Tables

`StringWrap` greedily wraps words to a positive character width. Runs of input
whitespace are normalized to single spaces; words longer than the width stay
unbroken on their own line.

```text
$ cargo run -q -p rum-cli -- --eval 'StringWrap["the quick brown fox jumps over the lazy dog", 10]'
"the quick\nbrown fox\njumps over\nthe lazy\ndog"
```

`StringIndent` prefixes non-blank lines. Empty and whitespace-only lines are
left unchanged. `StringDedent` removes the common leading whitespace from
non-blank lines, and `StringExpandTabs` expands tabs to fixed column stops.

```text
$ cargo run -q -p rum-cli -- --eval 'StringIndent["alpha\n \n beta", "> "]'
"> alpha\n \n>  beta"
```

`AlignColumns` takes a list of string rows, left-aligns each column to the
maximum width seen in that column, separates columns with two spaces, and trims
trailing whitespace from each row.

```text
$ cargo run -q -p rum-cli -- --eval 'AlignColumns[{{"name","qty"},{"apple","3"},{"kiwi","10"}}]'
"name   qty\napple  3\nkiwi   10"
```

## HTML And Shell Strings

`HtmlEscape` escapes `&`, `<`, `>`, `"`, and `'`. `HtmlUnescape` handles the
five named entities plus decimal and hexadecimal numeric references; unknown
entities are preserved verbatim.

```text
$ cargo run -q -p rum-cli -- --eval '{HtmlEscape["a<b & c"], HtmlUnescape["a&lt;b &amp; &#233;"]}'
List["a&lt;b &amp; c", "a<b & é"]
```

`ShellSplit` implements a small POSIX-style word splitter: whitespace separates
words, single and double quotes group words, and backslashes escape the next
character outside single quotes. Unterminated quotes and dangling escapes fail
closed with `$Failed`. `ShellQuote` emits a single-quoted shell word and uses
the standard `'\''` sequence for embedded single quotes.

```text
$ cargo run -q -p rum-cli -- --eval 'ShellSplit["cc -o \"my prog\" main.c"]'
List["cc", "-o", "my prog", "main.c"]
```

## Path And URL Strings

The path helpers are pure string utilities. `FileExtension` returns the final
component extension without the dot, `FileBaseName` returns the final component
without that extension, and `FileNameSplit` splits slash-separated paths. An
absolute leading `/` is returned as its own component; dotfiles such as
`.bashrc` have no extension.

```text
$ cargo run -q -p rum-cli -- --eval '{FileExtension["a/b/c.tar.gz"], FileBaseName["a/b/c.tar.gz"], FileNameSplit["/a/b/c"]}'
List["gz", "c.tar", List["/", "a", "b", "c"]]
```

`UrlEncode` percent-encodes every UTF-8 byte outside the RFC 3986 unreserved
set. `UrlDecode` reverses that encoding and fails closed on malformed percent
escapes or non-UTF-8 output. `UrlParse` accepts only
`scheme://host[:port][/path][?query][#fragment]`; scheme-less host/path strings
are rejected.

```text
$ cargo run -q -p rum-cli -- --eval 'UrlDecode[UrlEncode["a b/世界"]]'
"a b/世界"
```

```text
$ cargo run -q -p rum-cli -- --eval 'UrlParse["https://example.com:8080/x/y?q=1#top"]'
Association[Rule["Scheme", "https"], Rule["Host", "example.com"], Rule["Port", 8080], Rule["Path", "/x/y"], Rule["Query", "q=1"], Rule["Fragment", "top"]]
```

## Script Arguments

`ParseArgs` turns a list of strings into an association with positional
arguments, long `--key=value` options, and bare `--flag` flags. `--` ends option
parsing, so later strings are positional. Short-option clustering is outside
the current contract.

```text
$ cargo run -q -p rum-cli -- --eval 'ParseArgs[{"--verbose", "--out=x.txt", "--", "--literal", "pos"}]'
Association[Rule["Positional", List["--literal", "pos"]], Rule["Options", Association[Rule["out", "x.txt"]]], Rule["Flags", List["verbose"]]]
```

These helpers compose as ordinary expressions:

```text
$ cargo run -q -p rum-cli -- --eval 'AlignColumns[{{"field", "value"}, {"host", Lookup[UrlParse[Lookup[Lookup[ParseArgs[{"--url=https://example.com:443/x?q=1"}], "Options"], "url"]], "Host"]}, {"path", Lookup[UrlParse[Lookup[Lookup[ParseArgs[{"--url=https://example.com:443/x?q=1"}], "Options"], "url"]], "Path"]}}]'
"field  value\nhost   example.com\npath   /x"
```

## StringForm

`StringForm` is an eager formatter in `rum`: it returns a `String`, not a lazy
display wrapper. `` `n` `` inserts the n-th argument after `ToString` rendering,
`` `` `` inserts the next sequential argument, and `\`` emits a literal
backtick in the template.

```text
$ cargo run -q -p rum-cli -- --eval 'StringForm["`1` + `2` = `3`", 1, 2, 3]'
"1 + 2 = 3"
```
