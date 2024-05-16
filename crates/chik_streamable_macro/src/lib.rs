extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::token::Pub;
use syn::Lit::Int;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, Fields, FieldsNamed, FieldsUnnamed, Index, Type,
    Visibility,
};

#[proc_macro_attribute]
pub fn streamable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let found_crate =
        crate_name("chik-protocol").expect("chik-protocol is present in `Cargo.toml`");

    let chik_protocol = match &found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(name, Span::call_site());
            quote!(#ident)
        }
    };

    let is_message = &attr.to_string() == "message";

    let mut input: DeriveInput = parse_macro_input!(item);
    let name = input.ident.clone();
    let name_ref = &name;

    let mut extra_impls = Vec::new();

    if let Data::Struct(data) = &mut input.data {
        let mut field_names = Vec::new();
        let mut field_types = Vec::new();

        for (i, field) in data.fields.iter_mut().enumerate() {
            field.vis = Visibility::Public(Pub::default());
            field_names.push(Ident::new(
                &field
                    .ident
                    .as_ref()
                    .map(|ident| ident.to_string())
                    .unwrap_or(format!("field_{i}")),
                Span::mixed_site(),
            ));
            field_types.push(field.ty.clone());
        }

        let init_names = field_names.clone();

        let initializer = match &data.fields {
            Fields::Named(..) => quote!( Self { #( #init_names ),* } ),
            Fields::Unnamed(..) => quote!( Self( #( #init_names ),* ) ),
            Fields::Unit => quote!(Self),
        };

        if field_names.is_empty() {
            extra_impls.push(quote! {
                impl Default for #name_ref {
                    fn default() -> Self {
                        Self::new()
                    }
                }
            });
        }

        extra_impls.push(quote! {
            impl #name_ref {
                #[allow(clippy::too_many_arguments)]
                pub fn new( #( #field_names: #field_types ),* ) -> #name_ref {
                    #initializer
                }
            }
        });

        if is_message {
            extra_impls.push(quote! {
                impl #chik_protocol::ChikProtocolMessage for #name_ref {
                    fn msg_type() -> #chik_protocol::ProtocolMessageTypes {
                        #chik_protocol::ProtocolMessageTypes::#name_ref
                    }
                }
            });
        }
    } else {
        panic!("only structs are supported");
    }

    let main_derives = quote! {
        #[derive(chik_streamable_macro::Streamable, Hash, Debug, Clone, Eq, PartialEq)]
    };

    // If you're calling the macro from `chik-protocol`, enable Python bindings and arbitrary conditionally.
    // Otherwise, you're calling it from an external crate which doesn't have this infrastructure setup.
    // In that case, the caller can add these macros manually if they want to.
    let attrs = if matches!(found_crate, FoundCrate::Itself) {
        quote! {
            #[cfg_attr(
                feature = "py-bindings", pyo3::pyclass(frozen), derive(
                    chik_py_streamable_macro::PyJsonDict,
                    chik_py_streamable_macro::PyStreamable,
                    chik_py_streamable_macro::PyGetters
                )
            )]
            #main_derives
            #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
        }
    } else {
        main_derives
    };

    quote! {
        #attrs
        #input
        #( #extra_impls )*
    }
    .into()
}

#[proc_macro_derive(Streamable)]
pub fn chik_streamable_macro(input: TokenStream) -> TokenStream {
    let found_crate = crate_name("chik-traits").expect("chik-traits is present in `Cargo.toml`");

    let crate_name = match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    };

    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let mut fnames = Vec::<Ident>::new();
    let mut findices = Vec::<Index>::new();
    let mut ftypes = Vec::<Type>::new();
    match data {
        Data::Enum(e) => {
            let mut names = Vec::<Ident>::new();
            let mut values = Vec::<u8>::new();
            for v in e.variants.iter() {
                names.push(v.ident.clone());
                let expr = match &v.discriminant {
                    Some((_, expr)) => expr,
                    None => {
                        panic!("unsupported enum");
                    }
                };
                let l = match expr {
                    Expr::Lit(l) => l,
                    _ => {
                        panic!("unsupported enum (no literal)");
                    }
                };
                let i = match &l.lit {
                    Int(i) => i,
                    _ => {
                        panic!("unsupported enum (literal is not integer)");
                    }
                };
                match i.base10_parse::<u8>() {
                    Ok(v) => values.push(v),
                    Err(_) => {
                        panic!("unsupported enum (value not u8)");
                    }
                }
            }
            let ret = quote! {
                impl #crate_name::Streamable for #ident {
                    fn update_digest(&self, digest: &mut sha2::Sha256) {
                        <u8 as #crate_name::Streamable>::update_digest(&(*self as u8), digest);
                    }
                    fn stream(&self, out: &mut Vec<u8>) -> #crate_name::chik_error::Result<()> {
                        <u8 as #crate_name::Streamable>::stream(&(*self as u8), out)
                    }
                    fn parse<const TRUSTED: bool>(input: &mut std::io::Cursor<&[u8]>) -> #crate_name::chik_error::Result<Self> {
                        let v = <u8 as #crate_name::Streamable>::parse::<TRUSTED>(input)?;
                        match &v {
                            #(#values => Ok(Self::#names),)*
                            _ => Err(#crate_name::chik_error::Error::InvalidEnum),
                        }
                    }
                }
            };
            return ret.into();
        }
        Data::Union(_) => {
            panic!("Streamable does not support Unions");
        }
        Data::Struct(s) => match s.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                for (index, f) in unnamed.iter().enumerate() {
                    findices.push(Index::from(index));
                    ftypes.push(f.ty.clone());
                }
            }
            Fields::Unit => {}
            Fields::Named(FieldsNamed { named, .. }) => {
                for f in named.iter() {
                    fnames.push(f.ident.as_ref().unwrap().clone());
                    ftypes.push(f.ty.clone());
                }
            }
        },
    };

    if !fnames.is_empty() {
        let ret = quote! {
            impl #crate_name::Streamable for #ident {
                fn update_digest(&self, digest: &mut sha2::Sha256) {
                    #(self.#fnames.update_digest(digest);)*
                }
                fn stream(&self, out: &mut Vec<u8>) -> #crate_name::chik_error::Result<()> {
                    #(self.#fnames.stream(out)?;)*
                    Ok(())
                }
                fn parse<const TRUSTED: bool>(input: &mut std::io::Cursor<&[u8]>) -> #crate_name::chik_error::Result<Self> {
                    Ok(Self { #( #fnames: <#ftypes as #crate_name::Streamable>::parse::<TRUSTED>(input)?, )* })
                }
            }
        };
        ret.into()
    } else if !findices.is_empty() {
        let ret = quote! {
            impl #crate_name::Streamable for #ident {
                fn update_digest(&self, digest: &mut sha2::Sha256) {
                    #(self.#findices.update_digest(digest);)*
                }
                fn stream(&self, out: &mut Vec<u8>) -> #crate_name::chik_error::Result<()> {
                    #(self.#findices.stream(out)?;)*
                    Ok(())
                }
                fn parse<const TRUSTED: bool>(input: &mut std::io::Cursor<&[u8]>) -> #crate_name::chik_error::Result<Self> {
                    Ok(Self( #( <#ftypes as #crate_name::Streamable>::parse::<TRUSTED>(input)?, )* ))
                }
            }
        };
        ret.into()
    } else {
        // this is an empty type (Unit)
        let ret = quote! {
            impl #crate_name::Streamable for #ident {
                fn update_digest(&self, _digest: &mut sha2::Sha256) {}
                fn stream(&self, _out: &mut Vec<u8>) -> #crate_name::chik_error::Result<()> {
                    Ok(())
                }
                fn parse<const TRUSTED: bool>(_input: &mut std::io::Cursor<&[u8]>) -> #crate_name::chik_error::Result<Self> {
                    Ok(Self{})
                }
            }
        };
        ret.into()
    }
}
