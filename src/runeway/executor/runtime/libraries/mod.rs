mod module_type;
mod loaded;
mod stdlib_registry;

use crate::runeway::executor::runtime::environment::EnvRef;

pub use module_type::RNWModule;
pub use stdlib_registry::register_stdlib;

pub fn load_library(path: String) -> EnvRef {
    if let Some(name) = path.strip_prefix("std::") {
        if !loaded::is_loaded(&path) {
            let module = match stdlib_registry::get_stdlib(name) {
                Some(loader) => loader(),
                None => panic!("Cannot load the library '{}'", path),
            };
            loaded::register_loaded(&path, module.clone());
        }
        loaded::get_loaded(&path).unwrap_or_else(|| panic!("Cannot load the library '{}'", path))
    } else {
        unimplemented!()
    }
}
