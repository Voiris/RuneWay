use std::ffi::OsString;
use std::path::PathBuf;

pub const USAGE: &str = "Usage: runec <ROOT> [-o <PATH>] [--emit binary] [--jit] [-- <ARGS>...]";
pub const HELP: &str = concat!(
    "RuneWay compiler\n\n",
    "Usage: runec <ROOT> [-o <PATH>] [--emit binary] [--jit] [-- <ARGS>...]\n\n",
    "Arguments:\n",
    "  <ROOT>           Root source file of the compilation unit\n",
    "  [ARGS]...        Arguments passed to a JIT-executed program\n\n",
    "Options:\n",
    "  -o <PATH>          Write the executable to PATH\n",
    "      --emit <KIND>  Select the output kind [possible value: binary]\n",
    "      --jit          Compile and execute in memory without producing a file\n",
    "  -h, --help         Print help\n",
    "  -V, --version      Print version\n",
);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EmitKind {
    Binary,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cli {
    pub root: PathBuf,
    pub output: Option<PathBuf>,
    pub emit: Option<EmitKind>,
    pub jit: bool,
    pub program_args: Vec<OsString>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    Run(Cli),
    Help,
    Version,
}

impl Cli {
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<ParseOutcome, String> {
        let mut args = args.into_iter();
        let mut root = None;
        let mut output = None;
        let mut emit = None;
        let mut jit = false;
        let mut program_args = Vec::new();

        while let Some(arg) = args.next() {
            if arg == "--" {
                program_args.extend(args);
                break;
            }

            match arg.to_str() {
                Some("-h" | "--help") => return Ok(ParseOutcome::Help),
                Some("-V" | "--version") => return Ok(ParseOutcome::Version),
                Some("--jit") => {
                    if jit {
                        return Err("option `--jit` may only be used once".into());
                    }
                    jit = true;
                }
                Some("-o") => {
                    if output.is_some() {
                        return Err("option `-o` may only be used once".into());
                    }
                    output = Some(PathBuf::from(args.next().ok_or("option `-o` requires a path")?));
                }
                Some("--emit") => {
                    if emit.is_some() {
                        return Err("option `--emit` may only be used once".into());
                    }
                    emit =
                        Some(parse_emit(args.next().ok_or("option `--emit` requires a value")?)?);
                }
                Some(value) if value.starts_with("--emit=") => {
                    if emit.is_some() {
                        return Err("option `--emit` may only be used once".into());
                    }
                    emit = Some(parse_emit(OsString::from(&value[7..]))?);
                }
                Some(value) if value.starts_with('-') => {
                    return Err(format!("unknown option `{value}`"));
                }
                _ => {
                    if root.is_some() {
                        return Err(format!("unexpected argument `{}`", arg.to_string_lossy()));
                    }
                    root = Some(PathBuf::from(arg));
                }
            }
        }

        let root = root.ok_or("missing root source file")?;
        if jit && emit.is_some() {
            return Err("option `--jit` cannot be used with `--emit`".into());
        }
        if jit && output.is_some() {
            return Err("option `--jit` cannot be used with `-o`".into());
        }
        if !jit && !program_args.is_empty() {
            return Err("program arguments after `--` require `--jit`".into());
        }

        Ok(ParseOutcome::Run(Self { root, output, emit, jit, program_args }))
    }
}

fn parse_emit(value: OsString) -> Result<EmitKind, String> {
    match value.to_str() {
        Some("binary") => Ok(EmitKind::Binary),
        _ => Err(format!(
            "unsupported emission kind `{}`; expected `binary`",
            value.to_string_lossy()
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::{Cli, EmitKind, ParseOutcome};

    fn parse(args: &[&str]) -> Result<ParseOutcome, String> {
        Cli::parse(args.iter().map(OsString::from))
    }

    #[test]
    fn parses_binary_output() {
        assert_eq!(
            parse(&["main.rnw", "--emit", "binary", "-o", "app"]).unwrap(),
            ParseOutcome::Run(Cli {
                root: PathBuf::from("main.rnw"),
                output: Some(PathBuf::from("app")),
                emit: Some(EmitKind::Binary),
                jit: false,
                program_args: vec![],
            })
        );
    }

    #[test]
    fn parses_jit_program_arguments() {
        let ParseOutcome::Run(cli) =
            parse(&["main.rnw", "--jit", "--", "first", "--second"]).unwrap()
        else {
            panic!("expected runnable CLI");
        };
        assert!(cli.jit);
        assert_eq!(cli.program_args, ["first", "--second"]);
    }

    #[test]
    fn jit_conflicts_with_emit_and_output() {
        assert!(parse(&["main.rnw", "--jit", "--emit", "binary"]).is_err());
        assert!(parse(&["main.rnw", "--jit", "-o", "app"]).is_err());
    }
}
