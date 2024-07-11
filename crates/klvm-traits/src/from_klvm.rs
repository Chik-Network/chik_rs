use std::{rc::Rc, sync::Arc};

use num_bigint::{BigInt, Sign};

use crate::{FromKlvmError, KlvmDecoder};

pub trait FromKlvm<N>: Sized {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError>;
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
                let number = BigInt::from_signed_bytes_be(bytes.as_ref());
                let (sign, mut vec) = number.to_bytes_be();

                if vec.len() < std::mem::size_of::<$primitive>() {
                    let mut zeros = vec![0; LEN - vec.len()];
                    zeros.extend(vec);
                    vec = zeros;
                }

                let value = <$primitive>::from_be_bytes(vec.as_slice().try_into().or(Err(
                    FromKlvmError::WrongAtomLength {
                        expected: LEN,
                        found: bytes.as_ref().len(),
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

impl<N> FromKlvm<N> for bool {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        let atom = decoder.decode_atom(&node)?;
        match atom.as_ref() {
            [] => Ok(false),
            [1] => Ok(true),
            _ => Err(FromKlvmError::Custom(
                "expected boolean value of either `()` or `1`".to_string(),
            )),
        }
    }
}

impl<N, T> FromKlvm<N> for Box<T>
where
    T: FromKlvm<N>,
{
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        T::from_klvm(decoder, node).map(Box::new)
    }
}

impl<N, T> FromKlvm<N> for Rc<T>
where
    T: FromKlvm<N>,
{
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        T::from_klvm(decoder, node).map(Rc::new)
    }
}

impl<N, T> FromKlvm<N> for Arc<T>
where
    T: FromKlvm<N>,
{
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        T::from_klvm(decoder, node).map(Arc::new)
    }
}

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
        if bytes.as_ref().is_empty() {
            Ok(())
        } else {
            Err(FromKlvmError::WrongAtomLength {
                expected: 0,
                found: bytes.as_ref().len(),
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
                }

                items.push(T::from_klvm(decoder, first)?);
                node = rest;
            } else {
                let bytes = decoder.decode_atom(&node)?;
                if bytes.as_ref().is_empty() {
                    return items.try_into().or(Err(FromKlvmError::ExpectedPair));
                }

                return Err(FromKlvmError::WrongAtomLength {
                    expected: 0,
                    found: bytes.as_ref().len(),
                });
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
                if bytes.as_ref().is_empty() {
                    return Ok(items);
                }

                return Err(FromKlvmError::WrongAtomLength {
                    expected: 0,
                    found: bytes.as_ref().len(),
                });
            }
        }
    }
}

impl<N, T> FromKlvm<N> for Option<T>
where
    T: FromKlvm<N>,
{
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        if let Ok(atom) = decoder.decode_atom(&node) {
            if atom.as_ref().is_empty() {
                return Ok(None);
            }
        }
        Ok(Some(T::from_klvm(decoder, node)?))
    }
}

impl<N> FromKlvm<N> for String {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        let bytes = decoder.decode_atom(&node)?;
        Ok(Self::from_utf8(bytes.as_ref().to_vec())?)
    }
}

#[cfg(feature = "chik-bls")]
impl<N> FromKlvm<N> for chik_bls::PublicKey {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        let bytes = decoder.decode_atom(&node)?;
        let error = Err(FromKlvmError::WrongAtomLength {
            expected: 48,
            found: bytes.as_ref().len(),
        });
        let bytes: [u8; 48] = bytes.as_ref().try_into().or(error)?;
        Self::from_bytes(&bytes).map_err(|error| FromKlvmError::Custom(error.to_string()))
    }
}

#[cfg(feature = "chik-bls")]
impl<N> FromKlvm<N> for chik_bls::Signature {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        let bytes = decoder.decode_atom(&node)?;
        let error = Err(FromKlvmError::WrongAtomLength {
            expected: 96,
            found: bytes.as_ref().len(),
        });
        let bytes: [u8; 96] = bytes.as_ref().try_into().or(error)?;
        Self::from_bytes(&bytes).map_err(|error| FromKlvmError::Custom(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use klvmr::{serde::node_from_bytes, Allocator, NodePtr};

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
    fn test_bool() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "80"), Ok(false));
        assert_eq!(decode(a, "01"), Ok(true));
        assert_eq!(
            decode::<bool>(a, "05"),
            Err(FromKlvmError::Custom(
                "expected boolean value of either `()` or `1`".to_string(),
            ))
        );
    }

    #[test]
    fn test_smart_pointers() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "80"), Ok(Box::new(0u8)));
        assert_eq!(decode(a, "80"), Ok(Rc::new(0u8)));
        assert_eq!(decode(a, "80"), Ok(Arc::new(0u8)));
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
        assert_eq!(decode(a, "80"), Ok([0; 0]));
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
        assert_ne!(decode(a, "80"), Ok(Some(String::new())));
    }

    #[test]
    fn test_string() {
        let a = &mut Allocator::new();
        assert_eq!(decode(a, "8568656c6c6f"), Ok("hello".to_string()));
        assert_eq!(decode(a, "80"), Ok(String::new()));
    }

    #[cfg(feature = "chik-bls")]
    #[test]
    fn test_public_key() {
        use chik_bls::PublicKey;
        use hex_literal::hex;

        let a = &mut Allocator::new();

        let bytes = hex!(
            "
            b8f7dd239557ff8c49d338f89ac1a258a863fa52cd0a502e
            3aaae4b6738ba39ac8d982215aa3fa16bc5f8cb7e44b954d
            "
        );

        assert_eq!(
            decode(a, "b0b8f7dd239557ff8c49d338f89ac1a258a863fa52cd0a502e3aaae4b6738ba39ac8d982215aa3fa16bc5f8cb7e44b954d"),
            Ok(PublicKey::from_bytes(&bytes).unwrap())
        );
        assert_eq!(
            decode::<PublicKey>(a, "8568656c6c6f"),
            Err(FromKlvmError::WrongAtomLength {
                expected: 48,
                found: 5
            })
        );
    }

    #[cfg(feature = "chik-bls")]
    #[test]
    fn test_signature() {
        use chik_bls::Signature;
        use hex_literal::hex;

        let a = &mut Allocator::new();

        let bytes = hex!(
            "
            a3994dc9c0ef41a903d3335f0afe42ba16c88e7881706798492da4a1653cd10c
            69c841eeb56f44ae005e2bad27fb7ebb16ce8bbfbd708ea91dd4ff24f030497b
            50e694a8270eccd07dbc206b8ffe0c34a9ea81291785299fae8206a1e1bbc1d1
            "
        );
        assert_eq!(
            decode(a, "c060a3994dc9c0ef41a903d3335f0afe42ba16c88e7881706798492da4a1653cd10c69c841eeb56f44ae005e2bad27fb7ebb16ce8bbfbd708ea91dd4ff24f030497b50e694a8270eccd07dbc206b8ffe0c34a9ea81291785299fae8206a1e1bbc1d1"),
            Ok(Signature::from_bytes(&bytes).unwrap())
        );
        assert_eq!(
            decode::<Signature>(a, "8568656c6c6f"),
            Err(FromKlvmError::WrongAtomLength {
                expected: 96,
                found: 5
            })
        );
    }
}
