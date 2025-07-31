mod loaded;
mod module_type;
mod stdlib_registry;

use crate::runeway::runtime::environment::{EnvRef, Environment};
use std::fs;
use std::path::{Path, PathBuf};

use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::core::parser::parse_code;
use crate::runeway::executor::interpreter::ASTInterpreter;
pub use module_type::{register as register_module, RNWModule};
pub use stdlib_registry::register_stdlib;

pub fn load_library(import_path: String, working_dir: &Path) -> RWResult<EnvRef> {
    if let Some(name) = import_path.strip_prefix("std::") {
        if !loaded::is_loaded(&import_path) {
            let module = match stdlib_registry::get_stdlib(name) {
                Some(loader) => loader(),
                None => {
                    return Err(
                        RuneWayError::new(RuneWayErrorKind::error_with_code("ImportError"))
                            .with_message(format!("Cannot load library `{}`", import_path)),
                    );
                }
            };
            loaded::register_loaded(&import_path, module.clone());
            return Ok(module);
        }
        Ok(loaded::get_loaded(&import_path)
            .unwrap_or_else(|| panic!("InternalError: Cannot load the library '{}'", import_path)))
    } else {
        let import_path = if !import_path.ends_with(".rnw") {
            import_path + ".rnw"
        } else {
            import_path
        };
        let mut path = PathBuf::from(&import_path);
        if !path.is_absolute() {
            path = working_dir.join(&path);
        }
        if !path.is_file() {
            return Err(
                RuneWayError::new(RuneWayErrorKind::error_with_code("FileSystemError"))
                    .with_message(format!(
                        "Path is not a file or it does not exists: {}",
                        path.display()
                    )),
            );
        }
        path = path.canonicalize().map_err(|e| {
            RuneWayError::new(RuneWayErrorKind::error_with_code("FileSystemError"))
                .with_message(format!("Cannot canonicalize path: {}", path.display()))
                .with_note(format!("Raw Error: {}", e))
        })?;
        if !loaded::is_loaded(&import_path) {
            // TODO: Fix circular using (interpreter <-> runtime)

            let (filename, code) = if let Some(file_name) = path.file_name() {
                (file_name.to_str().unwrap(), fs::read_to_string(&path)?)
            } else {
                panic!("Internal: No filename found.");
            };
            let parsed_code = parse_code(filename.to_owned(), code.clone())
                .map_err(|e| e.with_source(filename, &code))?;
            let env = Environment::new_global();
            ASTInterpreter::execute_many(
                env.clone(),
                parsed_code.ast,
                working_dir,
                filename.to_string(),
                &code,
            )
            .map_err(|e| e.with_source(filename, &code))?;
            loaded::register_loaded(&import_path, env.clone());
            return Ok(env);
        }
        Ok(loaded::get_loaded(&import_path)
            .unwrap_or_else(|| panic!("InternalError: Cannot load the library '{}'", import_path)))
    }
}
