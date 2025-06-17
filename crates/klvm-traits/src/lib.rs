//! # KLVM Traits
//! This is a library for encoding and decoding Rust values using a KLVM allocator.
//! It provides implementations for every fixed-width signed and unsigned integer type,
//! as well as many other values in the standard library that would be common to encode.

#![cfg_attr(feature = "derive", doc = "\n\n")]
#![cfg_attr(feature = "derive", doc = include_str!("../docs/derive_macros.md"))]

#[cfg(feature = "derive")]
pub use klvm_derive::*;

mod error;
mod from_klvm;
mod int_encoding;
mod klvm_decoder;
mod klvm_encoder;
mod macros;
mod match_byte;
mod to_klvm;
mod wrappers;

pub use error::*;
pub use from_klvm::*;
pub use int_encoding::*;
pub use klvm_decoder::*;
pub use klvm_encoder::*;
pub use match_byte::*;
pub use to_klvm::*;
pub use wrappers::*;

pub use klvmr::Atom;

#[cfg(test)]
#[cfg(feature = "derive")]
mod derive_tests {
    extern crate self as klvm_traits;

    use super::*;

    use std::fmt::Debug;

    use klvmr::{serde::node_to_bytes, Allocator};

    fn check<T>(value: &T, expected: &str)
    where
        T: Debug + PartialEq + ToKlvm<Allocator> + FromKlvm<Allocator>,
    {
        let a = &mut Allocator::new();

        let ptr = value.to_klvm(a).unwrap();

        let actual = node_to_bytes(a, ptr).unwrap();
        assert_eq!(expected, hex::encode(actual));

        let round_trip = T::from_klvm(a, ptr).unwrap();
        assert_eq!(value, &round_trip);
    }

    fn coerce_into<A, B>(value: A) -> B
    where
        A: ToKlvm<Allocator>,
        B: FromKlvm<Allocator>,
    {
        let a = &mut Allocator::new();
        let ptr = value.to_klvm(a).unwrap();
        B::from_klvm(a, ptr).unwrap()
    }

    #[test]
    fn test_list_struct() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct {
            a: u64,
            b: i32,
        }

        // Includes the nil terminator.
        check(&Struct { a: 52, b: -32 }, "ff34ff81e080");
    }

    #[test]
    fn test_list_struct_with_rest() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct {
            a: u64,
            #[klvm(rest)]
            b: i32,
        }

        // Does not include the nil terminator.
        check(&Struct { a: 52, b: -32 }, "ff3481e0");
    }

    #[test]
    fn test_solution_struct() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct {
            a: u64,
            b: i32,
        }

        // Includes the nil terminator.
        check(&Struct { a: 52, b: -32 }, "ff34ff81e080");

        // Allows additional parameters.
        let mut allocator = Allocator::new();
        let ptr = klvm_list!(100, 200, 300, 400)
            .to_klvm(&mut allocator)
            .unwrap();
        let value = Struct::from_klvm(&allocator, ptr).unwrap();
        assert_eq!(value, Struct { a: 100, b: 200 });

        // Doesn't allow differing types for the actual solution parameters.
        let mut allocator = Allocator::new();
        let ptr = klvm_list!([1, 2, 3], 200, 300)
            .to_klvm(&mut allocator)
            .unwrap();
        Struct::from_klvm(&allocator, ptr).unwrap_err();
    }

    #[test]
    fn test_solution_struct_with_rest() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct {
            a: u64,
            #[klvm(rest)]
            b: i32,
        }

        // Does not include the nil terminator.
        check(&Struct { a: 52, b: -32 }, "ff3481e0");

        // Does not allow additional parameters, since it consumes the rest.
        let mut allocator = Allocator::new();
        let ptr = klvm_list!(100, 200, 300, 400)
            .to_klvm(&mut allocator)
            .unwrap();
        Struct::from_klvm(&allocator, ptr).unwrap_err();
    }

    #[test]
    fn test_curry_struct() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(curry)]
        struct Struct {
            a: u64,
            b: i32,
        }

        check(
            &Struct { a: 52, b: -32 },
            "ff04ffff0134ffff04ffff0181e0ff018080",
        );
    }

    #[test]
    fn test_curry_struct_with_rest() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(curry)]
        struct Struct {
            a: u64,
            #[klvm(rest)]
            b: i32,
        }

        check(&Struct { a: 52, b: -32 }, "ff04ffff0134ff81e080");
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct(String, String);

        check(&Struct("A".to_string(), "B".to_string()), "ff41ff4280");
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct(#[klvm(rest)] String);

        check(&Struct("XYZ".to_string()), "8358595a");
    }

    #[test]
    fn test_optional() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct {
            a: u64,
            #[klvm(default)]
            b: Option<i32>,
        }

        check(
            &Struct {
                a: 52,
                b: Some(-32),
            },
            "ff34ff81e080",
        );
        check(&Struct { a: 52, b: None }, "ff3480");
    }

    #[test]
    fn test_default() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct {
            a: u64,
            #[klvm(default = 42)]
            b: i32,
        }

        check(&Struct { a: 52, b: 32 }, "ff34ff2080");
        check(&Struct { a: 52, b: 42 }, "ff3480");
    }

    #[test]
    fn test_default_owned() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        struct Struct {
            a: u64,
            #[klvm(default = "Hello".to_string())]
            b: String,
        }

        check(
            &Struct {
                a: 52,
                b: "World".to_string(),
            },
            "ff34ff85576f726c6480",
        );
        check(
            &Struct {
                a: 52,
                b: "Hello".to_string(),
            },
            "ff3480",
        );
    }

    #[test]
    fn test_constants() {
        #[derive(ToKlvm, FromKlvm)]
        #[apply_constants]
        #[derive(Debug, PartialEq)]
        #[klvm(list)]
        struct RunTailCondition<P, S> {
            #[klvm(constant = 51)]
            opcode: u8,
            #[klvm(constant = ())]
            blank_puzzle_hash: (),
            #[klvm(constant = -113)]
            magic_amount: i8,
            puzzle: P,
            solution: S,
        }

        check(
            &RunTailCondition {
                puzzle: "puzzle".to_string(),
                solution: "solution".to_string(),
            },
            "ff33ff80ff818fff8670757a7a6c65ff88736f6c7574696f6e80",
        );
    }

    #[test]
    fn test_enum() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        enum Enum {
            A(i32),
            B { x: i32 },
            C,
        }

        check(&Enum::A(32), "ff80ff2080");
        check(&Enum::B { x: -72 }, "ff01ff81b880");
        check(&Enum::C, "ff0280");
    }

    #[test]
    fn test_explicit_discriminant() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list)]
        #[repr(u8)]
        enum Enum {
            A(i32) = 42,
            B { x: i32 } = 34,
            C = 11,
        }

        check(&Enum::A(32), "ff2aff2080");
        check(&Enum::B { x: -72 }, "ff22ff81b880");
        check(&Enum::C, "ff0b80");
    }

    #[test]
    fn test_untagged_enum() {
        // This has to be a proper list, since it's too ambiguous if extraneous parameters are parsed
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(proper_list, untagged)]
        enum Enum {
            A(i32),
            B {
                x: i32,
                y: i32,
            },
            #[klvm(curry)]
            C {
                curried_value: String,
            },
        }

        check(&Enum::A(32), "ff2080");
        check(&Enum::B { x: -72, y: 94 }, "ff81b8ff5e80");
        check(
            &Enum::C {
                curried_value: "Hello".to_string(),
            },
            "ff04ffff018548656c6c6fff0180",
        );
    }

    #[test]
    fn test_untagged_enum_parsing_order() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list, untagged)]
        enum Enum {
            // This variant is parsed first, so `B` will never be deserialized.
            A(i32),
            // When `B` is serialized, it will round trip as `A` instead.
            B(i32),
            // `C` will be deserialized as a fallback when the bytes don't deserialize to a valid `i32`.
            C(String),
        }

        // This round trips to the same value, since `A` is parsed first.
        assert_eq!(coerce_into::<Enum, Enum>(Enum::A(32)), Enum::A(32));

        // This round trips to `A` instead of `B`, since `A` is parsed first.
        assert_eq!(coerce_into::<Enum, Enum>(Enum::B(32)), Enum::A(32));

        // This round trips to `A` instead of `C`, since the bytes used to represent
        // this string are also a valid `i32` value.
        assert_eq!(
            coerce_into::<Enum, Enum>(Enum::C("Hi".into())),
            Enum::A(18537)
        );

        // This round trips to `C` instead of `A`, since the bytes used to represent
        // this string exceed the size of `i32`.
        assert_eq!(
            coerce_into::<Enum, Enum>(Enum::C("Hello, world!".into())),
            Enum::C("Hello, world!".into())
        );
    }

    #[test]
    fn test_custom_crate_name() {
        use klvm_traits as klvm_traits2;
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq)]
        #[klvm(list, crate_name = klvm_traits2)]
        struct Struct {
            a: u64,
            b: i32,
        }

        check(&Struct { a: 52, b: -32 }, "ff34ff81e080");
    }
}
