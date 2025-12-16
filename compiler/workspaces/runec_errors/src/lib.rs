mod diagnostics;
mod labels;
mod message;
mod emit;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use diagnostics::Diagnostic;

static DIAGNOSTICS: Lazy<Arc<Mutex<Vec<Diagnostic>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));


