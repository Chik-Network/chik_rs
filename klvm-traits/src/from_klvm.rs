use std::array::TryFromSliceError;

use klvmr::{
    allocator::{NodePtr, SExp},
    op_utils::nullp,
    Allocator,
};
use num_bigint::Sign;

use crate::{Error, Result};

pub trait FromKlvm: Sized {
    fn from_klvm(a: &Allocator, ptr: NodePtr) -> Result<Self>;
}

impl FromKlvm for NodePtr {
    fn from_klvm(_a: &Allocator, ptr: NodePtr) -> Result<Self> {
        Ok(ptr)
    }
}

macro_rules! klvm_primitive {
    ($primitive:ty) => {
        impl FromKlvm for $primitive {
            fn from_klvm(a: &Allocator, ptr: NodePtr) -> Result<Self> {
                if let SExp::Atom = a.sexp(ptr) {
                    let (sign, mut vec) = a.number(ptr).to_bytes_be();
                    if vec.len() < std::mem::size_of::<$primitive>() {
                        let mut zeros = vec![0; std::mem::size_of::<$primitive>() - vec.len()];
                        zeros.extend(vec);
                        vec = zeros;
                    }
                    let value =
                        <$primitive>::from_be_bytes(vec.as_slice().try_into().map_err(
                            |error: TryFromSliceError| Error::Custom(error.to_string()),
                        )?);
                    Ok(if sign == Sign::Minus {
                        value.wrapping_neg()
                    } else {
                        value
                    })
                } else {
                    Err(Error::ExpectedAtom(ptr))
                }
            }
        }
    };
}

klvm_primitive!(u8);
klvm_primitive!(i8);
klvm_primitive!(u16);
klvm_primitive!(i16);
klvm_primitive!(u32);
klvm_primitive!(i32);
klvm_primitive!(u64);
klvm_primitive!(i64);
klvm_primitive!(u128);
klvm_primitive!(i128);
klvm_primitive!(usize);
klvm_primitive!(isize);

impl<A, B> FromKlvm for (A, B)
where
    A: FromKlvm,
    B: FromKlvm,
{
    fn from_klvm(a: &Allocator, ptr: NodePtr) -> Result<Self> {
        match a.sexp(ptr) {
            SExp::Pair(first, rest) => Ok((A::from_klvm(a, first)?, B::from_klvm(a, rest)?)),
            SExp::Atom => Err(Error::ExpectedCons(ptr)),
        }
    }
}

impl FromKlvm for () {
    fn from_klvm(a: &Allocator, ptr: NodePtr) -> Result<Self> {
        if nullp(a, ptr) {
            Ok(())
        } else {
            Err(Error::ExpectedNil(ptr))
        }
    }
}

impl<T, const N: usize> FromKlvm for [T; N]
where
    T: FromKlvm,
{
    fn from_klvm(a: &Allocator, mut ptr: NodePtr) -> Result<Self> {
        let mut items = Vec::with_capacity(N);
        loop {
            match a.sexp(ptr) {
                SExp::Atom => {
                    if nullp(a, ptr) {
                        return match items.try_into() {
                            Ok(value) => Ok(value),
                            Err(_) => Err(Error::ExpectedCons(ptr)),
                        };
                    } else {
                        return Err(Error::ExpectedNil(ptr));
                    }
                }
                SExp::Pair(first, rest) => {
                    if items.len() >= N {
                        return Err(Error::ExpectedAtom(ptr));
                    } else {
                        items.push(T::from_klvm(a, first)?);
                        ptr = rest;
                    }
                }
            }
        }
    }
}

impl<T> FromKlvm for Vec<T>
where
    T: FromKlvm,
{
    fn from_klvm(a: &Allocator, mut ptr: NodePtr) -> Result<Self> {
        let mut items = Vec::new();
        loop {
            match a.sexp(ptr) {
                SExp::Atom => {
                    if nullp(a, ptr) {
                        return Ok(items);
                    } else {
                        return Err(Error::ExpectedNil(ptr));
                    }
                }
                SExp::Pair(first, rest) => {
                    items.push(T::from_klvm(a, first)?);
                    ptr = rest;
                }
            }
        }
    }
}

impl<T: FromKlvm> FromKlvm for Option<T> {
    fn from_klvm(a: &Allocator, ptr: NodePtr) -> Result<Self> {
        if nullp(a, ptr) {
            Ok(None)
        } else {
            Ok(Some(T::from_klvm(a, ptr)?))
        }
    }
}

impl FromKlvm for String {
    fn from_klvm(a: &Allocator, ptr: NodePtr) -> Result<Self> {
        if let SExp::Atom = a.sexp(ptr) {
            Self::from_utf8(a.atom(ptr).to_vec()).map_err(|error| Error::Custom(error.to_string()))
        } else {
            Err(Error::ExpectedAtom(ptr))
        }
    }
}

#[cfg(test)]
mod tests {
    use klvmr::serde::node_from_bytes;

    use super::*;

    fn decode<T>(a: &mut Allocator, hex: &str) -> Result<T>
    where
        T: FromKlvm,
    {
        let bytes = hex::decode(hex).unwrap();
        let actual = node_from_bytes(a, &bytes).unwrap();
        T::from_klvm(a, actual)
    }

    #[test]
    fn test_nodeptr() {
        let a = &mut Allocator::new();
        let ptr = a.one();
        assert_eq!(NodePtr::from_klvm(a, ptr).unwrap(), ptr);
    }

    #[test]
    fn test_primitives() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "80"), Ok(0u8));
        assert_eq!(decode(a, "80"), Ok(0i8));
        assert_eq!(decode(a, "05"), Ok(5u8));
        assert_eq!(decode(a, "05"), Ok(5u32));
        assert_eq!(decode(a, "05"), Ok(5i32));
        assert_eq!(decode(a, "81e5"), Ok(-27i32));
        assert_eq!(decode(a, "80"), Ok(-0));
        assert_eq!(decode(a, "8180"), Ok(-128i8));
    }

    #[test]
    fn test_pair() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "ff0502"), Ok((5, 2)));
        assert_eq!(decode(a, "ff81b8ff8301600980"), Ok((-72, (90121, ()))));
        assert_eq!(
            decode(a, "ffff80ff80ff80ffff80ff80ff80808080"),
            Ok((((), ((), ((), (((), ((), ((), ()))), ())))), ()))
        );
    }

    #[test]
    fn test_nil() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "80"), Ok(()));
    }

    #[test]
    fn test_array() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "ff01ff02ff03ff0480"), Ok([1, 2, 3, 4]));
        assert_eq!(decode(a, "80"), Ok([] as [i32; 0]));
    }

    #[test]
    fn test_vec() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "ff01ff02ff03ff0480"), Ok(vec![1, 2, 3, 4]));
        assert_eq!(decode(a, "80"), Ok(Vec::<i32>::new()));
    }

    #[test]
    fn test_option() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "8568656c6c6f"), Ok(Some("hello".to_string())));
        assert_eq!(decode(a, "80"), Ok(None::<String>));

        // Empty strings get decoded as None instead, since both values are represented by nil bytes.
        // This could be considered either intended behavior or not, depending on the way it's used.
        assert_ne!(decode(a, "80"), Ok(Some("".to_string())));
    }

    #[test]
    fn test_string() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "8568656c6c6f"), Ok("hello".to_string()));
        assert_eq!(decode(a, "80"), Ok("".to_string()));
    }
}
