use std::io::{self, Write};

use runec_abi::{RUNTIME_PRINT, RUNTIME_PRINTLN, RuntimeFunctionId, runtime_function};

pub type RuntimeFunctionAddress = *const u8;

#[derive(Debug, Copy, Clone)]
pub struct RuntimeSymbol {
    pub id: RuntimeFunctionId,
    pub name: &'static str,
    pub address: RuntimeFunctionAddress,
}

pub fn symbols() -> [RuntimeSymbol; 2] {
    [
        RuntimeSymbol {
            id: RUNTIME_PRINT,
            name: runtime_function(RUNTIME_PRINT).expect("print runtime declaration").symbol,
            address: __runeway_print as RuntimeFunctionAddress,
        },
        RuntimeSymbol {
            id: RUNTIME_PRINTLN,
            name: runtime_function(RUNTIME_PRINTLN).expect("println runtime declaration").symbol,
            address: __runeway_println as RuntimeFunctionAddress,
        },
    ]
}

pub fn resolve_symbol(name: &str) -> Option<RuntimeFunctionAddress> {
    symbols().into_iter().find(|symbol| symbol.name == name).map(|symbol| symbol.address)
}

#[unsafe(no_mangle)]
/// Writes `len` bytes starting at `ptr` to standard output.
///
/// # Safety
///
/// `ptr` must be null only when `len` is zero. Otherwise it must point to
/// `len` readable bytes that remain valid for the duration of the call.
pub unsafe extern "C" fn __runeway_print(ptr: *const u8, len: usize) {
    // SAFETY: Forwarded directly from the runtime ABI contract.
    let Some(bytes) = (unsafe { bytes_from_abi(ptr, len) }) else {
        return;
    };

    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(bytes);
    let _ = stdout.flush();
}

#[unsafe(no_mangle)]
/// Writes `len` bytes starting at `ptr` to standard output, followed by a
/// newline.
///
/// # Safety
///
/// `ptr` must be null only when `len` is zero. Otherwise it must point to
/// `len` readable bytes that remain valid for the duration of the call.
pub unsafe extern "C" fn __runeway_println(ptr: *const u8, len: usize) {
    // SAFETY: Forwarded directly from the runtime ABI contract.
    let Some(bytes) = (unsafe { bytes_from_abi(ptr, len) }) else {
        return;
    };

    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(bytes);
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}

unsafe fn bytes_from_abi<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if ptr.is_null() {
        return (len == 0).then_some(&[]);
    }

    // SAFETY: The runtime ABI requires `ptr` to reference `len` readable bytes
    // for the duration of the call.
    Some(unsafe { std::slice::from_raw_parts(ptr, len) })
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use runec_abi::RUNTIME_FUNCTIONS;

    use super::{bytes_from_abi, resolve_symbol, symbols};

    #[test]
    fn exports_every_declared_runtime_symbol() {
        let symbols = symbols();
        assert_eq!(symbols.len(), RUNTIME_FUNCTIONS.len());
        for declaration in RUNTIME_FUNCTIONS {
            assert!(resolve_symbol(declaration.symbol).is_some());
        }
    }

    #[test]
    fn rejects_unknown_runtime_symbols() {
        assert!(resolve_symbol("__runeway_unknown").is_none());
    }

    #[test]
    fn accepts_null_pointer_only_for_empty_slices() {
        // SAFETY: bytes_from_abi handles null pointers before attempting to
        // create a slice.
        let empty = unsafe { bytes_from_abi(ptr::null(), 0) };
        let invalid = unsafe { bytes_from_abi(ptr::null(), 1) };

        assert_eq!(empty, Some(&[][..]));
        assert!(invalid.is_none());
    }
}
