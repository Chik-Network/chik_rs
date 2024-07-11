#![allow(clippy::option_option)]

mod apply_constants;
mod from_klvm;
mod helpers;
mod parser;
mod to_klvm;

use apply_constants::impl_apply_constants;
use from_klvm::from_klvm;
use proc_macro::TokenStream;

use proc_macro2::Span;
use syn::{parse_macro_input, DeriveInput, Ident};
use to_klvm::to_klvm;

const CRATE_NAME: &str = "klvm_traits";

fn crate_name(name: Option<Ident>) -> Ident {
    name.unwrap_or_else(|| Ident::new(CRATE_NAME, Span::call_site()))
}

#[proc_macro_derive(ToKlvm, attributes(klvm))]
pub fn to_klvm_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    to_klvm(ast).into()
}

#[proc_macro_derive(FromKlvm, attributes(klvm))]
pub fn from_klvm_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    from_klvm(ast).into()
}

#[proc_macro_attribute]
pub fn apply_constants(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_apply_constants(ast).into()
}
