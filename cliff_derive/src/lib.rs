extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput};

/// Usage:
/// ```
/// #[derive(Message)]
/// #[namespace("NameSpace")] // Optional Argument
/// struct MessageStruct{};
/// ```
#[proc_macro_derive(Message, attributes(namespace))]
pub fn message_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ns = input.attrs.iter().find(|&attr| {
        if let Some(ident) = attr.path.get_ident() {
            ident == "namespace"
        } else {
            false
        }
    });

    let namespace = get_namespace(ns);

    let name = input.ident;
    let str_ident = name.to_string();

    let as_str = match namespace {
        Some(ns) => quote! { format!("{}:{}", #ns, #str_ident) },
        None => quote! { #str_ident.to_string() },
    };

    let expanded = quote! {
        impl Message for #name {
            fn message_type(&self) -> String {
                #as_str
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_namespace(attr: Option<&Attribute>) -> Option<proc_macro2::TokenStream> {
    if let Some(namespace_attr) = attr {
        let tokens = namespace_attr.tokens.clone();
        return match tokens
            .into_iter()
            .filter_map(|tree| match tree {
                proc_macro2::TokenTree::Group(group) => Some(group.stream()),
                _ => None,
            })
            .flatten()
            .filter_map(|tree| match tree {
                proc_macro2::TokenTree::Literal(lit) => Some(lit),
                _ => None,
            })
            .nth(0)
        {
            Some(lit) => Some(proc_macro2::TokenStream::from(
                proc_macro2::TokenTree::from(lit),
            )),
            None => None,
        };
    }

    None
}
