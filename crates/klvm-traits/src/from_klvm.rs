use std::{rc::Rc, sync::Arc};

use num_bigint::BigInt;

use crate::{decode_number, FromKlvmError, KlvmDecoder};

pub trait FromKlvm<D>: Sized
where
    D: KlvmDecoder,
{
    fn from_klvm(decoder: &D, node: D::Node) -> Result<Self, FromKlvmError>;
}

macro_rules! klvm_primitive {
    ($primitive:ty, $signed:expr) => {
        impl<N, D: KlvmDecoder<Node = N>> FromKlvm<D> for $primitive {
            fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
                const LEN: usize = std::mem::size_of::<$primitive>();

                let atom = decoder.decode_atom(&node)?;
                let slice = atom.as_ref();

                let Some(bytes) = decode_number(slice, $signed) else {
                    return Err(FromKlvmError::WrongAtomLength {
                        expected: LEN,
                        found: slice.len(),
                    });
                };

                Ok(<$primitive>::from_be_bytes(bytes))
            }
        }
    };
}

klvm_primitive!(u8, false);
klvm_primitive!(i8, true);
klvm_primitive!(u16, false);
klvm_primitive!(i16, true);
klvm_primitive!(u32, false);
klvm_primitive!(i32, true);
klvm_primitive!(u64, false);
klvm_primitive!(i64, true);
klvm_primitive!(u128, false);
klvm_primitive!(i128, true);
klvm_primitive!(usize, false);
klvm_primitive!(isize, true);

impl<N, D: KlvmDecoder<Node = N>> FromKlvm<D> for BigInt {
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        decoder.decode_bigint(&node)
    }
}

impl<N, D: KlvmDecoder<Node = N>> FromKlvm<D> for bool {
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
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

impl<N, D: KlvmDecoder<Node = N>, T> FromKlvm<D> for Box<T>
where
    T: FromKlvm<D>,
{
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        T::from_klvm(decoder, node).map(Box::new)
    }
}

impl<N, D: KlvmDecoder<Node = N>, T> FromKlvm<D> for Rc<T>
where
    T: FromKlvm<D>,
{
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        T::from_klvm(decoder, node).map(Rc::new)
    }
}

impl<N, D: KlvmDecoder<Node = N>, T> FromKlvm<D> for Arc<T>
where
    T: FromKlvm<D>,
{
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        T::from_klvm(decoder, node).map(Arc::new)
    }
}

impl<N, D: KlvmDecoder<Node = N>, A, B> FromKlvm<D> for (A, B)
where
    A: FromKlvm<D>,
    B: FromKlvm<D>,
{
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        let (first, rest) = decoder.decode_pair(&node)?;
        let first = A::from_klvm(decoder, first)?;
        let rest = B::from_klvm(decoder, rest)?;
        Ok((first, rest))
    }
}

impl<N, D: KlvmDecoder<Node = N>> FromKlvm<D> for () {
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
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

impl<N, D: KlvmDecoder<Node = N>, T, const LEN: usize> FromKlvm<D> for [T; LEN]
where
    T: FromKlvm<D>,
{
    fn from_klvm(decoder: &D, mut node: N) -> Result<Self, FromKlvmError> {
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

impl<N, D: KlvmDecoder<Node = N>, T> FromKlvm<D> for Vec<T>
where
    T: FromKlvm<D>,
{
    fn from_klvm(decoder: &D, mut node: N) -> Result<Self, FromKlvmError> {
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

impl<N, D: KlvmDecoder<Node = N>, T> FromKlvm<D> for Option<T>
where
    T: FromKlvm<D>,
{
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        if let Ok(atom) = decoder.decode_atom(&node) {
            if atom.as_ref().is_empty() {
                return Ok(None);
            }
        }
        Ok(Some(T::from_klvm(decoder, node)?))
    }
}

impl<N, D: KlvmDecoder<Node = N>> FromKlvm<D> for String {
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        let bytes = decoder.decode_atom(&node)?;
        Ok(Self::from_utf8(bytes.as_ref().to_vec())?)
    }
}

#[cfg(feature = "chik-bls")]
impl<N, D: KlvmDecoder<Node = N>> FromKlvm<D> for chik_bls::PublicKey {
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
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
impl<N, D: KlvmDecoder<Node = N>> FromKlvm<D> for chik_bls::Signature {
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        let bytes = decoder.decode_atom(&node)?;
        let error = Err(FromKlvmError::WrongAtomLength {
            expected: 96,
            found: bytes.as_ref().len(),
        });
        let bytes: [u8; 96] = bytes.as_ref().try_into().or(error)?;
        Self::from_bytes(&bytes).map_err(|error| FromKlvmError::Custom(error.to_string()))
    }
}

#[cfg(feature = "chik-secp")]
impl<D> FromKlvm<D> for chik_secp::K1PublicKey
where
    D: KlvmDecoder,
{
    fn from_klvm(decoder: &D, node: D::Node) -> Result<Self, FromKlvmError> {
        let atom = decoder.decode_atom(&node)?;
        let bytes: [u8; Self::SIZE] =
            atom.as_ref()
                .try_into()
                .map_err(|_| FromKlvmError::WrongAtomLength {
                    expected: Self::SIZE,
                    found: atom.len(),
                })?;
        Self::from_bytes(&bytes).map_err(|error| FromKlvmError::Custom(error.to_string()))
    }
}

#[cfg(feature = "chik-secp")]
impl<D> FromKlvm<D> for chik_secp::K1Signature
where
    D: KlvmDecoder,
{
    fn from_klvm(decoder: &D, node: D::Node) -> Result<Self, FromKlvmError> {
        let atom = decoder.decode_atom(&node)?;
        let bytes: [u8; Self::SIZE] =
            atom.as_ref()
                .try_into()
                .map_err(|_| FromKlvmError::WrongAtomLength {
                    expected: Self::SIZE,
                    found: atom.len(),
                })?;
        Self::from_bytes(&bytes).map_err(|error| FromKlvmError::Custom(error.to_string()))
    }
}

#[cfg(feature = "chik-secp")]
impl<D> FromKlvm<D> for chik_secp::R1PublicKey
where
    D: KlvmDecoder,
{
    fn from_klvm(decoder: &D, node: D::Node) -> Result<Self, FromKlvmError> {
        let atom = decoder.decode_atom(&node)?;
        let bytes: [u8; Self::SIZE] =
            atom.as_ref()
                .try_into()
                .map_err(|_| FromKlvmError::WrongAtomLength {
                    expected: Self::SIZE,
                    found: atom.len(),
                })?;
        Self::from_bytes(&bytes).map_err(|error| FromKlvmError::Custom(error.to_string()))
    }
}

#[cfg(feature = "chik-secp")]
impl<D> FromKlvm<D> for chik_secp::R1Signature
where
    D: KlvmDecoder,
{
    fn from_klvm(decoder: &D, node: D::Node) -> Result<Self, FromKlvmError> {
        let atom = decoder.decode_atom(&node)?;
        let bytes: [u8; Self::SIZE] =
            atom.as_ref()
                .try_into()
                .map_err(|_| FromKlvmError::WrongAtomLength {
                    expected: Self::SIZE,
                    found: atom.len(),
                })?;
        Self::from_bytes(&bytes).map_err(|error| FromKlvmError::Custom(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use klvmr::{serde::node_from_bytes, Allocator};

    use super::*;

    fn decode<T>(a: &mut Allocator, hex: &str) -> Result<T, FromKlvmError>
    where
        T: FromKlvm<Allocator>,
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

    #[cfg(feature = "chik-secp")]
    #[test]
    fn test_secp_public_key() {
        use chik_secp::{K1PublicKey, R1PublicKey};
        use hex_literal::hex;

        let a = &mut Allocator::new();

        let bytes = hex!("02827cdbbed87e45683d448be2ea15fb72ba3732247bda18474868cf5456123fb4");

        assert_eq!(
            decode(
                a,
                "a102827cdbbed87e45683d448be2ea15fb72ba3732247bda18474868cf5456123fb4"
            ),
            Ok(K1PublicKey::from_bytes(&bytes).unwrap())
        );
        assert_eq!(
            decode::<K1PublicKey>(a, "8568656c6c6f"),
            Err(FromKlvmError::WrongAtomLength {
                expected: 33,
                found: 5
            })
        );

        let bytes = hex!("037dc85102f5eb7867b9580fea8b242c774173e1a47db320c798242d3a7a7579e4");

        assert_eq!(
            decode(
                a,
                "a1037dc85102f5eb7867b9580fea8b242c774173e1a47db320c798242d3a7a7579e4"
            ),
            Ok(R1PublicKey::from_bytes(&bytes).unwrap())
        );
        assert_eq!(
            decode::<R1PublicKey>(a, "8568656c6c6f"),
            Err(FromKlvmError::WrongAtomLength {
                expected: 33,
                found: 5
            })
        );
    }

    #[cfg(feature = "chik-secp")]
    #[test]
    fn test_secp_signature() {
        use chik_secp::K1Signature;
        use hex_literal::hex;

        let a = &mut Allocator::new();

        let bytes = hex!(
            "
            6f07897d1d28b8698af5dec5ca06907b1304b227dc9f740b8c4065cf04d5e865
            3ae66aa17063e7120ee7f22fae54373b35230e259244b90400b65cf00d86c591
            "
        );

        assert_eq!(
            decode(a, "c0406f07897d1d28b8698af5dec5ca06907b1304b227dc9f740b8c4065cf04d5e8653ae66aa17063e7120ee7f22fae54373b35230e259244b90400b65cf00d86c591"),
            Ok(K1Signature::from_bytes(&bytes).unwrap())
        );
        assert_eq!(
            decode::<K1Signature>(a, "8568656c6c6f"),
            Err(FromKlvmError::WrongAtomLength {
                expected: 64,
                found: 5
            })
        );

        let bytes = hex!(
            "
            550e83da8cf9b2d407ed093ae213869ebd7ceaea603920f87d535690e52b4053
            7915d8fe3d5a96c87e700c56dc638c32f7a2954f2ba409367d1a132000cc2228
            "
        );

        assert_eq!(
            decode(a, "c040550e83da8cf9b2d407ed093ae213869ebd7ceaea603920f87d535690e52b40537915d8fe3d5a96c87e700c56dc638c32f7a2954f2ba409367d1a132000cc2228"),
            Ok(K1Signature::from_bytes(&bytes).unwrap())
        );
        assert_eq!(
            decode::<K1Signature>(a, "8568656c6c6f"),
            Err(FromKlvmError::WrongAtomLength {
                expected: 64,
                found: 5
            })
        );
    }
}
