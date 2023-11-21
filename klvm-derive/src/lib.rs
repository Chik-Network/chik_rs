extern crate proc_macro;

mod from_klvm;
mod helpers;
mod to_klvm;

use from_klvm::from_klvm;
use syn::{parse_macro_input, DeriveInput};
use to_klvm::to_klvm;

#[proc_macro_derive(ToKlvm, attributes(klvm))]
pub fn to_klvm_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    to_klvm(ast).into()
}

#[proc_macro_derive(FromKlvm, attributes(klvm))]
pub fn from_klvm_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    from_klvm(ast).into()
}
