extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Message)]
pub fn message_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let as_str = name.to_string();

    let expanded = quote! {
        impl Message for #name {
            fn message_type(&self) -> String {
                #as_str.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}
