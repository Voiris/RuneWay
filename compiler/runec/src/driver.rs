use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use runec_codegen_cranelift::{AotBackend, JitBackend, JitTimings};
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
    let mut benchmark = cli.benchmark.then(BenchmarkTimings::new);
    let mut source_map = SourceMap::new();
    let source = SourceFileLoader.load(cli.root.clone()).map_err(|error| {
        DriverError::Message(format!("could not load `{}`: {error}", cli.root.display()))
    })?;
    let source_id = source_map.add_file(source);
    if let Some(benchmark) = &mut benchmark {
        benchmark.source_loading = benchmark.finish_stage();
    }
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
    if let Some(benchmark) = &mut benchmark {
        benchmark.parsing = benchmark.finish_stage();
    }

    let lowered = HirLowerer::new().lower(&stmts);
    stop_on_diagnostics(lowered.diags, &source_map)?;
    let mut hir = lowered.map;

    let semantic = SemanticChecker::new().check(&mut hir);
    stop_on_diagnostics(semantic.diags, &source_map)?;
    if let Some(benchmark) = &mut benchmark {
        benchmark.semantic_analysis = benchmark.finish_stage();
    }

    let lowered = MirLowerer::new(&semantic.info).lower(&hir);
    stop_on_diagnostics(lowered.diags, &source_map)?;
    if let Some(benchmark) = &mut benchmark {
        benchmark.mir_generation = benchmark.finish_stage();
    }

    if cli.jit {
        run_jit(&lowered.module, diagnostic_span, &source_map, &cli.program_args, benchmark)
    } else {
        let started = benchmark.as_ref().map(|_| Instant::now());
        let object = AotBackend::emit_object(&lowered.module, "runeway_unit", diagnostic_span)
            .map_err(|diagnostic| {
                report(*diagnostic, &source_map);
                DriverError::Reported
            })?;
        let codegen = started.map_or(Duration::ZERO, |started| started.elapsed());

        let output = cli.output.clone().unwrap_or_else(|| default_output(&cli.root));
        let started = benchmark.as_ref().map(|_| Instant::now());
        link::link_binary(&object, &output).map_err(DriverError::Message)?;
        let linking = started.map_or(Duration::ZERO, |started| started.elapsed());

        if let Some(benchmark) = benchmark {
            benchmark.print_aot(codegen, linking);
        }
        Ok(())
    }
}

fn run_jit(
    module: &runec_mir::MirModule<'_>,
    diagnostic_span: Span,
    source_map: &SourceMap,
    program_args: &[OsString],
    benchmark: Option<BenchmarkTimings>,
) -> Result<(), DriverError> {
    // The CLI owns these arguments already. The current entry ABI is `main() -> unit`;
    // a future runtime argument API can expose them without changing the CLI contract.
    let _ = program_args;
    let started = benchmark.as_ref().map(|_| Instant::now());
    let symbols = runec_runtime::symbols().map(|symbol| (symbol.name, symbol.address));
    let mut backend = JitBackend::new(symbols, diagnostic_span).map_err(|diagnostic| {
        report(*diagnostic, source_map);
        DriverError::Reported
    })?;
    let backend_setup = started.map_or(Duration::ZERO, |started| started.elapsed());

    if let Some(benchmark) = benchmark {
        let mut jit = backend.run_benchmarked(module).map_err(|diagnostic| {
            report(*diagnostic, source_map);
            DriverError::Reported
        })?;
        jit.codegen += backend_setup;
        benchmark.print_jit(jit);
        Ok(())
    } else {
        backend.run(module).map_err(|diagnostic| {
            report(*diagnostic, source_map);
            DriverError::Reported
        })
    }
}

struct BenchmarkTimings {
    stage_started: Instant,
    source_loading: Duration,
    parsing: Duration,
    semantic_analysis: Duration,
    mir_generation: Duration,
}

impl BenchmarkTimings {
    fn new() -> Self {
        Self {
            stage_started: Instant::now(),
            source_loading: Duration::ZERO,
            parsing: Duration::ZERO,
            semantic_analysis: Duration::ZERO,
            mir_generation: Duration::ZERO,
        }
    }

    fn finish_stage(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now.duration_since(self.stage_started);
        self.stage_started = now;
        elapsed
    }

    fn print_jit(self, jit: JitTimings) {
        let compilation = self.source_loading
            + self.parsing
            + self.semantic_analysis
            + self.mir_generation
            + jit.codegen
            + jit.finalization;
        let total = compilation + jit.execution;

        eprintln!("[benchmark]");
        print_timing("source loading", self.source_loading);
        print_timing("parsing", self.parsing);
        print_timing("semantic analysis", self.semantic_analysis);
        print_timing("MIR generation", self.mir_generation);
        print_timing("codegen", jit.codegen);
        print_timing("JIT finalization", jit.finalization);
        eprintln!("--------------------------------");
        print_timing("compilation", compilation);
        print_timing("execution", jit.execution);
        print_timing("total", total);
    }

    fn print_aot(self, codegen: Duration, linking: Duration) {
        let compilation = self.source_loading
            + self.parsing
            + self.semantic_analysis
            + self.mir_generation
            + codegen
            + linking;

        eprintln!("[benchmark]");
        print_timing("source loading", self.source_loading);
        print_timing("parsing", self.parsing);
        print_timing("semantic analysis", self.semantic_analysis);
        print_timing("MIR generation", self.mir_generation);
        print_timing("codegen", codegen);
        print_timing("linking", linking);
        eprintln!("--------------------------------");
        print_timing("compilation", compilation);
    }
}

fn print_timing(label: &str, duration: Duration) {
    eprintln!("{label:<20} {:>8.3} ms", duration.as_secs_f64() * 1_000.0);
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
