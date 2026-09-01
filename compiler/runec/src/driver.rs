use std::ffi::OsString;
use std::path::PathBuf;

use runec_codegen_cranelift::{AotBackend, JitBackend};
use runec_errors::diagnostics::Diagnostic;
use runec_hir::lowering::HirLowerer;
use runec_mir::MirLowerer;
use runec_parse::{Lexer, ParseResult, Parser};
use runec_semantic::SemanticChecker;
use runec_source::byte_pos::BytePos;
use runec_source::source_loader::SourceFileLoader;
use runec_source::source_map::SourceMap;
use runec_source::span::Span;

use crate::cli::Cli;
use crate::link;

pub enum DriverError {
    Message(String),
    Reported,
}

pub fn run(cli: &Cli) -> Result<(), DriverError> {
    let mut source_map = SourceMap::new();
    let source = SourceFileLoader.load(cli.root.clone()).map_err(|error| {
        DriverError::Message(format!("could not load `{}`: {error}", cli.root.display()))
    })?;
    let source_id = source_map.add_file(source);
    let source_len = source_map.get_file(&source_id).expect("source was just added").src().len();
    let diagnostic_span = Span::new(
        BytePos::from_usize(0),
        BytePos::from_usize(source_len.min(BytePos::MAX)),
        source_id,
    );

    let tokens = Lexer::new(source_id, &source_map).lex_full().map_err(|diagnostic| {
        report(*diagnostic, &source_map);
        DriverError::Reported
    })?;

    let ParseResult { stmts, diags } = Parser::new(tokens, source_id, &source_map).parse_full();
    stop_on_diagnostics(diags, &source_map)?;

    let lowered = HirLowerer::new().lower(&stmts);
    stop_on_diagnostics(lowered.diags, &source_map)?;
    let mut hir = lowered.map;

    let semantic = SemanticChecker::new().check(&mut hir);
    stop_on_diagnostics(semantic.diags, &source_map)?;

    let lowered = MirLowerer::new(&semantic.info).lower(&hir);
    stop_on_diagnostics(lowered.diags, &source_map)?;

    if cli.jit {
        run_jit(&lowered.module, diagnostic_span, &source_map, &cli.program_args)
    } else {
        let object = AotBackend::emit_object(&lowered.module, "runeway_unit", diagnostic_span)
            .map_err(|diagnostic| {
                report(*diagnostic, &source_map);
                DriverError::Reported
            })?;
        let output = cli.output.clone().unwrap_or_else(|| default_output(&cli.root));
        link::link_binary(&object, &output).map_err(DriverError::Message)
    }
}

fn run_jit(
    module: &runec_mir::MirModule<'_>,
    diagnostic_span: Span,
    source_map: &SourceMap,
    program_args: &[OsString],
) -> Result<(), DriverError> {
    // The CLI owns these arguments already. The current entry ABI is `main() -> unit`;
    // a future runtime argument API can expose them without changing the CLI contract.
    let _ = program_args;
    let symbols = runec_runtime::symbols().map(|symbol| (symbol.name, symbol.address));
    let mut backend = JitBackend::new(symbols, diagnostic_span).map_err(|diagnostic| {
        report(*diagnostic, source_map);
        DriverError::Reported
    })?;
    backend.run(module).map_err(|diagnostic| {
        report(*diagnostic, source_map);
        DriverError::Reported
    })
}

fn stop_on_diagnostics(
    diagnostics: Vec<Diagnostic<'_>>,
    source_map: &SourceMap,
) -> Result<(), DriverError> {
    if diagnostics.is_empty() {
        return Ok(());
    }
    for diagnostic in diagnostics {
        report(diagnostic, source_map);
    }
    Err(DriverError::Reported)
}

fn report(diagnostic: Diagnostic<'_>, source_map: &SourceMap) {
    let mut rendered = String::new();
    diagnostic.emit(source_map, &mut rendered);
    eprintln!("{rendered}");
}

fn default_output(root: &std::path::Path) -> PathBuf {
    let mut name = root.file_stem().unwrap_or_default().to_os_string();
    name.push(std::env::consts::EXE_SUFFIX);
    PathBuf::from(name)
}
