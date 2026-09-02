# `runec` command-line interface

`runec` is the RuneWay compiler. It accepts one root source file and compiles one compilation unit.
Project management, toolchain management, and commands such as `run`, `check`, `test`, and `new`
belong to a future higher-level tool.

## Usage

```console
runec <ROOT> [-o <PATH>] [--emit binary]
runec <ROOT> --jit [-- <ARGS>...]
```

Use `runec --help` to print the supported options and `runec --version` to print the compiler
version.

## Compilation root

Every compilation receives exactly one root source file, normally `main.rnw` for an executable or
`lib.rnw` for a library:

```console
runec main.rnw
runec lib.rnw
```

The root and every module reachable through its imports form one compilation unit. Imported
modules are not compiled as independent units. Parsing, semantic analysis, MIR lowering, and code
generation operate on the complete unit.

Module loading is not implemented yet; the current compiler handles the supplied root file. The
one-root, one-compilation-unit rule defines how imports will integrate with the CLI.

## Binary output

Without an execution option, `runec` emits an executable. Its default name is derived from the root
file name in the current directory. `-o` overrides the path:

```console
runec main.rnw
runec main.rnw -o main
```

The only currently supported `--emit` value is `binary`. In this mode the compilation unit must
define a compatible `main()` entry point; library output will be added with a future emission
kind:

```console
runec main.rnw --emit binary
runec main.rnw --emit binary -o main
```

`--emit` is intentionally singular. One invocation emits at most one file containing the complete
compilation unit; imported modules will never produce separate output files.

When the generated output is not needed, callers may direct it to the operating system's null
device using `-o /dev/null` on Unix-like systems or `-o NUL` on Windows. This still performs the
full requested compilation.

The current binary linker is bootstrapped through `rustc`, selected from the `RUSTC` environment
variable or found on `PATH`. This is an implementation limitation of the initial CLI and can be
replaced by a dedicated native linker driver later without changing the public interface.

## JIT execution

`--jit` compiles the complete unit in memory and immediately invokes its `main` function without
creating an output file:

```console
runec main.rnw --jit
runec main.rnw --jit -- first second
```

Everything after `--` is treated as a program argument, including values beginning with `-`. The
current entry ABI is `main() -> unit`, so the CLI preserves these arguments but RuneWay code cannot
observe them until an argument API is added to the runtime.

JIT execution and file emission are mutually exclusive. These combinations are CLI errors:

```console
runec main.rnw --jit -o main
runec main.rnw --jit --emit binary
```

## Responsibilities

`runec` is deliberately limited to compiler responsibilities:

- load the root file and its imported modules;
- compile them as one unit;
- report diagnostics;
- emit one executable or run the unit through the JIT.

A future higher-level tool may implement project workflows by driving `runec` internally.
