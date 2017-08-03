# word_replace - Word replacement tool for text file, with toml-based dictionary

### Brief introduction
`word_replace` finds words tagged by `@@` in input text files, and
replace them into words defined in the toml-based dictionary.
It was designed for ease of technical word translation in the source files
for a markdown book (a.k.a. mdbook).

### Korean-specific feature for postposition
For Korean, `word_replace` reads the `[ko-postpos]` table in the toml file,
and automatically adjust postpositions after tagged words depending on
whether the final jamo exists or not.

### Usage
```
word_replace [-w] [-d <dictfile>] [-l <language>] [-r <root>] [<INPUT>] [<OUTPUT>]
    -w, --warning        Shows all warnings.
    -d <dictfile>        Sets a dictionary file. default: dict.toml
    -l <language>        Sets a language. default: ko
    -r <root>            Sets a root directory. default: .
    <INPUT>     Set a source directory. default: src_pre
    <OUTPUT>    Set a destination directory. default: src
```
