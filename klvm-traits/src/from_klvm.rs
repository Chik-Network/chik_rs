use klvmr::{allocator::NodePtr, Allocator};
use num_bigint::{BigInt, Sign};

use crate::{KlvmDecoder, FromKlvmError};

pub trait FromKlvm<N>: Sized {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError>;
}

pub trait FromNodePtr {
    fn from_node_ptr(a: &Allocator, node: NodePtr) -> Result<Self, FromKlvmError>
    where
        Self: Sized;
}

impl<T> FromNodePtr for T
where
    T: FromKlvm<NodePtr>,
{
    fn from_node_ptr(a: &Allocator, node: NodePtr) -> Result<Self, FromKlvmError>
    where
        Self: Sized,
    {
        T::from_klvm(a, node)
    }
}

impl FromKlvm<NodePtr> for NodePtr {
    fn from_klvm(
        _decoder: &impl KlvmDecoder<Node = NodePtr>,
        node: NodePtr,
    ) -> Result<Self, FromKlvmError> {
        Ok(node)
    }
}

macro_rules! klvm_primitive {
    ($primitive:ty) => {
        impl<N> FromKlvm<N> for $primitive {
            fn from_klvm(
                decoder: &impl KlvmDecoder<Node = N>,
                node: N,
            ) -> Result<Self, FromKlvmError> {
                const LEN: usize = std::mem::size_of::<$primitive>();

                let bytes = decoder.decode_atom(&node)?;
                let number = BigInt::from_signed_bytes_be(bytes);
                let (sign, mut vec) = number.to_bytes_be();

                if vec.len() < std::mem::size_of::<$primitive>() {
                    let mut zeros = vec![0; LEN - vec.len()];
                    zeros.extend(vec);
                    vec = zeros;
                }

                let value = <$primitive>::from_be_bytes(vec.as_slice().try_into().or(Err(
                    FromKlvmError::WrongAtomLength {
                        expected: LEN,
                        found: bytes.len(),
                    },
                ))?);

                Ok(if sign == Sign::Minus {
                    value.wrapping_neg()
                } else {
                    value
                })
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

impl<N, A, B> FromKlvm<N> for (A, B)
where
    A: FromKlvm<N>,
    B: FromKlvm<N>,
{
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        let (first, rest) = decoder.decode_pair(&node)?;
        let first = A::from_klvm(decoder, first)?;
        let rest = B::from_klvm(decoder, rest)?;
        Ok((first, rest))
    }
}

impl<N> FromKlvm<N> for () {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        let bytes = decoder.decode_atom(&node)?;
        if bytes.is_empty() {
            Ok(())
        } else {
            Err(FromKlvmError::WrongAtomLength {
                expected: 0,
                found: bytes.len(),
            })
        }
    }
}

impl<N, T, const LEN: usize> FromKlvm<N> for [T; LEN]
where
    T: FromKlvm<N>,
{
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, mut node: N) -> Result<Self, FromKlvmError> {
        let mut items = Vec::with_capacity(LEN);
        loop {
            if let Ok((first, rest)) = decoder.decode_pair(&node) {
                if items.len() >= LEN {
                    return Err(FromKlvmError::ExpectedAtom);
                } else {
                    items.push(T::from_klvm(decoder, first)?);
                    node = rest;
                }
            } else {
                let bytes = decoder.decode_atom(&node)?;
                if bytes.is_empty() {
                    return items.try_into().or(Err(FromKlvmError::ExpectedPair));
                } else {
                    return Err(FromKlvmError::WrongAtomLength {
                        expected: 0,
                        found: bytes.len(),
                    });
                }
            }
        }
    }
}

impl<N, T> FromKlvm<N> for Vec<T>
where
    T: FromKlvm<N>,
{
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, mut node: N) -> Result<Self, FromKlvmError> {
        let mut items = Vec::new();
        loop {
            if let Ok((first, rest)) = decoder.decode_pair(&node) {
                items.push(T::from_klvm(decoder, first)?);
                node = rest;
            } else {
                let bytes = decoder.decode_atom(&node)?;
                if bytes.is_empty() {
                    return Ok(items);
                } else {
                    return Err(FromKlvmError::WrongAtomLength {
                        expected: 0,
                        found: bytes.len(),
                    });
                }
            }
        }
    }
}

impl<N, T> FromKlvm<N> for Option<T>
where
    T: FromKlvm<N>,
{
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        if let Ok(&[]) = decoder.decode_atom(&node) {
            Ok(None)
        } else {
            Ok(Some(T::from_klvm(decoder, node)?))
        }
    }
}

impl<N> FromKlvm<N> for String {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        let bytes = decoder.decode_atom(&node)?;
        Ok(Self::from_utf8(bytes.to_vec())?)
    }
}

#[cfg(test)]
mod tests {
    use klvmr::{allocator::NodePtr, serde::node_from_bytes, Allocator};

    use super::*;

    fn decode<T>(a: &mut Allocator, hex: &str) -> Result<T, FromKlvmError>
    where
        T: FromKlvm<NodePtr>,
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
