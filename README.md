# flexlint

**flexlint** is a flexible linter with rules defined by regular expression.

[![Build Status](https://travis-ci.org/dalance/flexlint.svg?branch=master)](https://travis-ci.org/dalance/flexlint)
[![Crates.io](https://img.shields.io/crates/v/flexlint.svg)](https://crates.io/crates/flexlint)
[![codecov](https://codecov.io/gh/dalance/flexlint/branch/master/graph/badge.svg)](https://codecov.io/gh/dalance/flexlint)

## Install
Download from [release page](https://github.com/dalance/flexlint/releases/latest), and extract to the directory in PATH.

Alternatively you can install by [cargo](https://crates.io).

```
cargo install flexlint
```

## Usage

### Option

```
flexlint 0.1.0
dalance <dalance@gmail.com>
A flexible linter with rules specified by regular expression

USAGE:
    flexlint [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -s, --simple     Show results by simple format
    -V, --version    Prints version information
    -v, --verbose    Show verbose message

OPTIONS:
    -r, --rule <rule>    Rule file [default: .flexlint.toml]
```

Rule file is searched to the upper directory until `/`.
So you can put rule file (`.flexlint.toml`) on the repository root like `.gitignore`.

### Rule definition

Rule definition is below:

```toml
[[rules]]
name      =  ""   # name of rule
pattern   =  ""   # check pattern by regexp
required  =  ""   # required pattern by regexp [Optional]
forbidden =  ""   # forbidden pattern by regexp [Optional]
ignore    =  ""   # ignore pattern by regexp [Optional]
hint      =  ""   # hint message
includes  =  [""] # include file globs
excludes  =  [""] # exclude file globs [Optional]
```

If `pattern` is matched, `required` or `forbidden` is tried to match at the `pattern` matched point.
So `required` pattern is not matched, or `forbidden` pattern is matched, then check is failed.
`required` and `forbidden` is optional, but if both of them is not defined, check is skipped.
If the `pattern` matched point is included in the `ignore` matched range, check is skipped.

The example for `if` with brace of C/C++ is below:

```toml
[[rules]]
name      = "'if' with brace"
pattern   = '(?m)(^|[\t ])if\s'
forbidden = '(?m)(^|[\t ])if\s[^;{]*$'
ignore    = '(/\*/?([^/]|[^*]/)*\*/)|(//.*\n)'
hint      = "multiline 'if' must have brace"
includes  = ["**/*.c", "**/*.cpp"]
excludes  = ["external/*.c"]
```

`pattern` is matched `if` keyword and `forbidden` check that `if` must have `;` or `{` until the end of line.
( This example don't support some brace-style, you can modify it )

`ignore` is defined to skip single line comment (`// ...`) and multi-line comment (`/* ... */`).

If files matched `includes` match `excludes` too, the files are skipped.

### Regular expression

The syntax of regular expression follows [Rust regex crate](https://docs.rs/regex/latest/regex/#syntax).

### The example of output

If flexlint is executed in `example` directory of the repository, the output is below:

```console
$ cd example
$ flexlint
Fail: 'if' with brace
   --> test.c:4:4
  |
4 |     if ( hoge )
  |    ^^^^ hint: multiline 'if' must have brace

Fail: verilog 'always' forbidden
   --> test.sv:7:4
  |
7 |     always @ ( posedge clk ) begin
  |    ^^^^^^^^ hint: 'always' must be replaced to 'always_comb'/'always_ff'

Fail: 'if' with 'begin'
   --> test.sv:13:8
   |
13 |         if ( rst )
   |        ^^^^ hint: 'if' statement must have 'begin'

Fail: 'else' with 'begin'
   --> test.sv:15:8
   |
15 |         else
   |        ^^^^^^ hint: 'else' statement must have 'begin'

```
