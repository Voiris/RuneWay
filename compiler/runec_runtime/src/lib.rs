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
            name: runtime_function(RUNTIME_PRINT)
                .expect("print runtime declaration")
                .symbol,
            address: __runeway_print as RuntimeFunctionAddress,
        },
        RuntimeSymbol {
            id: RUNTIME_PRINTLN,
            name: runtime_function(RUNTIME_PRINTLN)
                .expect("println runtime declaration")
                .symbol,
            address: __runeway_println as RuntimeFunctionAddress,
        },
    ]
}

pub fn resolve_symbol(name: &str) -> Option<RuntimeFunctionAddress> {
    symbols()
        .into_iter()
        .find(|symbol| symbol.name == name)
        .map(|symbol| symbol.address)
}

#[unsafe(no_mangle)]
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
    use runec_abi::RUNTIME_FUNCTIONS;

    use super::{resolve_symbol, symbols};

    #[test]
    fn exports_every_declared_runtime_symbol() {
        let symbols = symbols();
        assert_eq!(symbols.len(), RUNTIME_FUNCTIONS.len());
        for declaration in RUNTIME_FUNCTIONS {
            assert!(resolve_symbol(declaration.symbol).is_some());
        }
    }
}
