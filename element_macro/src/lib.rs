extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro]
pub fn bind_reactives(input: TokenStream) -> TokenStream {
    TokenStream::new()
}