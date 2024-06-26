use proc_macro2::TokenStream;
use quote::quote;

use crate::helpers::Repr;

pub struct Macros {
    /// Encodes a nested tuple containing each of the field values within.
    pub klvm_macro: TokenStream,

    /// Decodes a nested tuple containing each of the field types within.
    pub match_macro: TokenStream,

    /// Destructures the values into the field names.
    pub destructure_macro: TokenStream,
}

pub fn repr_macros(crate_name: &TokenStream, repr: Repr) -> Macros {
    let (klvm_macro, match_macro, destructure_macro) = match repr {
        Repr::List => (
            quote!( #crate_name::klvm_list ),
            quote!( #crate_name::match_list ),
            quote!( #crate_name::destructure_list ),
        ),
        Repr::Tuple => (
            quote!( #crate_name::klvm_tuple ),
            quote!( #crate_name::match_tuple ),
            quote!( #crate_name::destructure_tuple ),
        ),
        Repr::Curry => (
            quote!( #crate_name::klvm_curried_args ),
            quote!( #crate_name::match_curried_args ),
            quote!( #crate_name::destructure_curried_args ),
        ),
    };

    Macros {
        klvm_macro,
        match_macro,
        destructure_macro,
    }
}
