use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn runec() -> &'static str {
    env!("CARGO_BIN_EXE_runec")
}

fn hello_world() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../example/hello_world.rnw")
}

#[test]
fn jit_runs_compilation_unit_and_accepts_program_arguments() {
    let output = Command::new(runec())
        .arg(hello_world())
        .arg("--jit")
        .arg("--")
        .arg("first")
        .arg("--second")
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"Hello, World!\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn jit_rejects_output_options() {
    for options in [["--jit", "--emit", "binary"].as_slice(), ["--jit", "-o", "app"].as_slice()] {
        let output = Command::new(runec()).arg(hello_world()).args(options).output().unwrap();

        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
    }
}

#[cfg(debug_assertions)]
#[test]
fn benchmark_reports_jit_stage_timings_to_stderr() {
    let output =
        Command::new(runec()).arg(hello_world()).arg("--jit").arg("--benchmark").output().unwrap();

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"Hello, World!\n");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for label in [
        "[benchmark]",
        "source loading",
        "parsing",
        "semantic analysis",
        "MIR generation",
        "codegen",
        "JIT finalization",
        "compilation",
        "execution",
        "total",
    ] {
        assert!(stderr.contains(label), "missing `{label}` in:\n{stderr}");
    }
}

#[cfg(debug_assertions)]
#[test]
fn benchmark_reports_aot_compilation_without_execution() {
    let temp = TempDir::new();
    let executable = temp.path.join(format!("bench{}", std::env::consts::EXE_SUFFIX));
    let output = Command::new(runec())
        .arg(hello_world())
        .arg("--benchmark")
        .arg("-o")
        .arg(executable)
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("linking"), "{stderr}");
    assert!(stderr.contains("compilation"), "{stderr}");
    assert!(!stderr.contains("execution"), "{stderr}");
    assert!(!stderr.contains("total"), "{stderr}");
}

#[test]
fn emits_a_standalone_binary() {
    let temp = TempDir::new();
    let executable = temp.path.join(format!("hello{}", std::env::consts::EXE_SUFFIX));
    let output = Command::new(runec())
        .arg(hello_world())
        .arg("--emit")
        .arg("binary")
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let output = Command::new(executable).output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Hello, World!\n");
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("runec-cli-test-{}-{id}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
