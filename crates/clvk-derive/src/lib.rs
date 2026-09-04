#![allow(clippy::option_option)]

mod apply_constants;
mod from_clvk;
mod helpers;
mod parser;
mod to_clvk;

use apply_constants::impl_apply_constants;
use from_clvk::from_clvk;
use proc_macro::TokenStream;

use proc_macro2::Span;
use syn::{DeriveInput, Ident, parse_macro_input};
use to_clvk::to_clvk;

const CRATE_NAME: &str = "clvk_traits";

fn crate_name(name: Option<Ident>) -> Ident {
    name.unwrap_or_else(|| Ident::new(CRATE_NAME, Span::call_site()))
}

#[proc_macro_derive(ToClvk, attributes(clvk))]
pub fn to_clvk_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    to_clvk(ast).into()
}

#[proc_macro_derive(FromClvk, attributes(clvk))]
pub fn from_clvk_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    from_clvk(ast).into()
}

#[proc_macro_attribute]
pub fn apply_constants(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_apply_constants(ast).into()
}
