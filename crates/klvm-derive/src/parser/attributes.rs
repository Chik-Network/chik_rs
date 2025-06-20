use std::fmt;

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, Ident, Token,
};

/// The representation of fields when converted to and from KLVM.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Repr {
    /// Represents `(A . (B . (C . ())))`.
    ProperList,
    /// The same as [Repr::ProperList], but the terminator doesn't have to be `()`.
    List,
    /// Represents `(c (q . A) (c (q . B) (c (q . C) 1)))`.
    Curry,
    /// Represents the first field `A` on its own, with no other fields allowed.
    Transparent,
    /// Represents `A` on its own, if it's an atom.
    Atom,
}

impl Repr {
    pub fn expect(repr: Option<Repr>) -> Repr {
        repr.expect(
            "missing either `list`, `proper_list`, `curry`, `transparent`, or `atom` in `klvm` attribute options",
        )
    }
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::List => "list",
            Self::ProperList => "proper_list",
            Self::Curry => "curry",
            Self::Transparent => "transparent",
            Self::Atom => "atom",
        })
    }
}

/// All of the possible options of the `klvm` attribute and the enum `repr` attribute.
/// They must be validated after being parsed to prevent invalid option configurations.
pub struct KlvmOptions {
    /// The representation of the fields.
    pub repr: Option<Repr>,
    /// The value of the field, also removed the actual field from the struct.
    /// This is useful for constant fields which shouldn't be in the constructor.
    pub constant: Option<Expr>,
    /// Whether the enum should parse variants one after the other instead of using the discriminant.
    pub untagged: bool,
    /// The integer type used for the enum discriminant.
    pub enum_repr: Option<Ident>,
    /// The name of the `klvm_traits` crate to use, useful for renamed dependencies for example.
    pub crate_name: Option<Ident>,
    /// The default value of the field, if it's not present in the KLVM object.
    /// If the default is set to `None`, it will assume the type is `Option` and the default will be `None`.
    pub default: Option<Option<Expr>>,
    /// Whether the field is a rest field, which will consume the rest of the KLVM object.
    pub rest: bool,
}

/// All of the possible options of the `klvm` attribute.
enum KlvmOption {
    Repr(Repr),
    Constant(Expr),
    CrateName(Ident),
    Untagged,
    Default(Option<Expr>),
    Rest,
}

impl Parse for KlvmOption {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;

        match ident.to_string().as_str() {
            "list" => Ok(Self::Repr(Repr::List)),
            "proper_list" => Ok(Self::Repr(Repr::ProperList)),
            "curry" => Ok(Self::Repr(Repr::Curry)),
            "transparent" => Ok(Self::Repr(Repr::Transparent)),
            "atom" => Ok(Self::Repr(Repr::Atom)),
            "untagged" => Ok(Self::Untagged),
            "constant" => {
                input.parse::<Token![=]>()?;
                Ok(Self::Constant(input.parse()?))
            }
            "crate_name" => {
                input.parse::<Token![=]>()?;
                Ok(Self::CrateName(input.parse()?))
            }
            "default" => {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    Ok(Self::Default(Some(input.parse()?)))
                } else {
                    Ok(Self::Default(None))
                }
            }
            "rest" => Ok(Self::Rest),
            _ => Err(syn::Error::new(ident.span(), "unknown argument")),
        }
    }
}

/// Parses the `klvm` attribute options and `repr` option from the given attributes.
pub fn parse_klvm_options(attrs: &[Attribute]) -> KlvmOptions {
    let mut options = KlvmOptions {
        repr: None,
        constant: None,
        untagged: false,
        enum_repr: None,
        crate_name: None,
        default: None,
        rest: false,
    };

    for attr in attrs {
        let Some(ident) = attr.path().get_ident() else {
            continue;
        };

        if ident == "repr" {
            let repr = attr.parse_args::<Ident>().unwrap();
            let text = repr.to_string();
            let text = text.as_str();

            // Check if the repr is an integer type. If not, it's not an enum discriminant repr.
            // For example, `#[repr(C)]` should not be relevant to the KLVM conversions.
            // This is intended for things like `#[repr(u8)]` or `#[repr(i32)]`.
            let is_unsigned_int = matches!(text, "u8" | "u16" | "u32" | "u64" | "u128" | "usize");
            let is_signed_int = matches!(text, "i8" | "i16" | "i32" | "i64" | "i128" | "isize");

            if !is_unsigned_int && !is_signed_int {
                continue;
            }

            options.enum_repr = Some(repr);
        }

        if ident != "klvm" {
            continue;
        }

        let parsed_options = attr
            .parse_args_with(Punctuated::<KlvmOption, Token![,]>::parse_terminated)
            .unwrap_or_else(|error| panic!("failed to parse `klvm` attribute options: {error}"));

        for option in parsed_options {
            match option {
                KlvmOption::Untagged => {
                    assert!(!options.untagged, "duplicate `untagged` option");
                    options.untagged = true;
                }
                KlvmOption::Repr(repr) => {
                    assert!(options.repr.is_none(), "duplicate repr option `{repr}`");
                    options.repr = Some(repr);
                }
                KlvmOption::Constant(value) => {
                    assert!(options.constant.is_none(), "duplicate `constant` option");
                    options.constant = Some(value);
                }
                KlvmOption::CrateName(crate_name) => {
                    assert!(
                        options.crate_name.is_none(),
                        "duplicate `crate_name` option"
                    );
                    options.crate_name = Some(crate_name);
                }
                KlvmOption::Default(default) => {
                    assert!(options.default.is_none(), "duplicate `default` option");
                    options.default = Some(default);
                }
                KlvmOption::Rest => {
                    assert!(!options.rest, "duplicate `rest` option");
                    options.rest = true;
                }
            }
        }
    }

    options
}
