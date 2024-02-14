//! # KLVM Traits
//! This is a library for encoding and decoding Rust values using a KLVM allocator.
//! It provides implementations for every fixed-width signed and unsigned integer type,
//! as well as many other values in the standard library that would be common to encode.
//!
//! As well as the built-in implementations, this library exposes two derive macros
//! for implementing the `ToKlvm` and `FromKlvm` traits on structs. They be marked
//! with one of the following encodings:
//!
//! * `#[klvm(tuple)]` for unterminated lists such as `(A . (B . C))`.
//! * `#[klvm(list)]` for proper lists such as `(A B C)`, or in other words `(A . (B . (C . ())))`.
//! * `#[klvm(curry)]` for curried arguments such as `(c (q . A) (c (q . B) (c (q . C) 1)))`.

#![cfg_attr(
    feature = "derive",
    doc = r#"
## Derive Example

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(tuple)]
struct Point {
    x: i32,
    y: i32,
}

let a = &mut Allocator::new();

let point = Point { x: 5, y: 2 };
let ptr = point.to_klvm(a).unwrap();

assert_eq!(Point::from_klvm(a, ptr).unwrap(), point);
```
"#
)]

#[cfg(feature = "derive")]
pub use klvm_derive::*;

mod error;
mod from_klvm;
mod macros;
mod match_byte;
mod to_klvm;

pub use error::*;
pub use from_klvm::*;
pub use match_byte::*;
pub use to_klvm::*;

#[cfg(test)]
#[cfg(feature = "derive")]
mod tests {
    extern crate self as klvm_traits;

    use std::fmt;

    use klvmr::{serde::node_to_bytes, Allocator};

    use super::*;

    fn check<T>(value: T, expected: &str)
    where
        T: fmt::Debug + PartialEq + ToKlvm + FromKlvm,
    {
        let a = &mut Allocator::new();

        let ptr = value.to_klvm(a).unwrap();
        let round_trip = T::from_klvm(a, ptr).unwrap();
        assert_eq!(value, round_trip);

        let bytes = node_to_bytes(a, ptr).unwrap();
        let actual = hex::encode(bytes);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_tuple() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq, Eq)]
        #[klvm(tuple)]
        struct TupleStruct {
            a: u64,
            b: i32,
        }

        check(TupleStruct { a: 52, b: -32 }, "ff3481e0");
    }

    #[test]
    fn test_list() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq, Eq)]
        #[klvm(list)]
        struct ListStruct {
            a: u64,
            b: i32,
        }

        check(ListStruct { a: 52, b: -32 }, "ff34ff81e080");
    }

    #[test]
    fn test_args() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq, Eq)]
        #[klvm(curry)]
        struct CurryStruct {
            a: u64,
            b: i32,
        }

        check(
            CurryStruct { a: 52, b: -32 },
            "ff04ffff0134ffff04ffff0181e0ff018080",
        );
    }

    #[test]
    fn test_unnamed() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq, Eq)]
        #[klvm(tuple)]
        struct UnnamedStruct(String, String);

        check(UnnamedStruct("A".to_string(), "B".to_string()), "ff4142");
    }

    #[test]
    fn test_newtype() {
        #[derive(Debug, ToKlvm, FromKlvm, PartialEq, Eq)]
        #[klvm(tuple)]
        struct NewTypeStruct(String);

        check(NewTypeStruct("XYZ".to_string()), "8358595a");
    }
}
