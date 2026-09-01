use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use runec_codegen_cranelift::ENTRY_SYMBOL;

pub fn link_binary(object: &[u8], output: &Path) -> Result<(), String> {
    let temp = TempDir::new()?;
    let object_path = temp.path.join("unit.o");
    let wrapper_path = temp.path.join("entry.rs");

    fs::write(&object_path, object)
        .map_err(|error| format!("could not write temporary object file: {error}"))?;
    fs::write(&wrapper_path, wrapper_source())
        .map_err(|error| format!("could not write temporary runtime wrapper: {error}"))?;

    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let status = Command::new(&rustc)
        .arg(&wrapper_path)
        .arg("--edition=2024")
        .arg("-C")
        .arg(format!("link-arg={}", object_path.display()))
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| {
            format!("could not start `{}` to link the executable: {error}", rustc.to_string_lossy())
        })?;

    if status.success() { Ok(()) } else { Err(format!("linker driver exited with {status}")) }
}

fn wrapper_source() -> String {
    format!(
        r#"use std::io::{{self, Write}};

unsafe extern "C" {{
    #[link_name = "{ENTRY_SYMBOL}"]
    fn runeway_main();
}}

#[unsafe(no_mangle)]
unsafe extern "C" fn __runeway_print(ptr: *const u8, len: usize) {{
    let Some(bytes) = (unsafe {{ bytes_from_abi(ptr, len) }}) else {{
        return;
    }};
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(bytes);
    let _ = stdout.flush();
}}

#[unsafe(no_mangle)]
unsafe extern "C" fn __runeway_println(ptr: *const u8, len: usize) {{
    let Some(bytes) = (unsafe {{ bytes_from_abi(ptr, len) }}) else {{
        return;
    }};
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(bytes);
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}}

unsafe fn bytes_from_abi<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {{
    if ptr.is_null() {{
        return (len == 0).then_some(&[]);
    }}
    Some(unsafe {{ std::slice::from_raw_parts(ptr, len) }})
}}

fn main() {{
    unsafe {{ runeway_main() }};
}}
"#
    )
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("system clock is before the Unix epoch: {error}"))?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runec-{}-{nonce}", std::process::id()));
        fs::create_dir(&path)
            .map_err(|error| format!("could not create temporary linker directory: {error}"))?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
