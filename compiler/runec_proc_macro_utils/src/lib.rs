use proc_macro::TokenStream;
use quote::quote;
use syn::{Path, parse_macro_input};

#[proc_macro]
pub fn last_path_segment(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as Path);

    let Some(segment) = path.segments.last() else {
        return syn::Error::new_spanned(path, "expected non-empty path")
            .to_compile_error()
            .into();
    };
    let symbol = segment.ident.to_string();

    quote!(
        #symbol
    )
    .into()
}
