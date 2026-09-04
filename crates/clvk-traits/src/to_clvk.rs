use std::{rc::Rc, sync::Arc};

use clvkr::Atom;
use num_bigint::BigInt;

use crate::{ClvkEncoder, ToClvkError, encode_number};

pub trait ToClvk<E>
where
    E: ClvkEncoder,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<E::Node, ToClvkError>;
}

macro_rules! clvk_primitive {
    ($primitive:ty) => {
        impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for $primitive {
            fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
                let bytes = self.to_be_bytes();
                #[allow(unused_comparisons)]
                encoder.encode_atom(Atom::Borrowed(&encode_number(&bytes, *self < 0)))
            }
        }
    };
}

clvk_primitive!(u8);
clvk_primitive!(i8);
clvk_primitive!(u16);
clvk_primitive!(i16);
clvk_primitive!(u32);
clvk_primitive!(i32);
clvk_primitive!(u64);
clvk_primitive!(i64);
clvk_primitive!(u128);
clvk_primitive!(i128);
clvk_primitive!(usize);
clvk_primitive!(isize);

impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for BigInt {
    fn to_clvk(&self, encoder: &mut E) -> Result<<E as ClvkEncoder>::Node, ToClvkError> {
        encoder.encode_bigint(self.clone())
    }
}

impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for bool {
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        i32::from(*self).to_clvk(encoder)
    }
}

impl<N, E: ClvkEncoder<Node = N>, T> ToClvk<E> for &T
where
    T: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        T::to_clvk(*self, encoder)
    }
}

impl<N, E: ClvkEncoder<Node = N>, T> ToClvk<E> for Box<T>
where
    T: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        T::to_clvk(self, encoder)
    }
}

impl<N, E: ClvkEncoder<Node = N>, T> ToClvk<E> for Rc<T>
where
    T: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        T::to_clvk(self, encoder)
    }
}

impl<N, E: ClvkEncoder<Node = N>, T> ToClvk<E> for Arc<T>
where
    T: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        T::to_clvk(self, encoder)
    }
}

impl<N, E: ClvkEncoder<Node = N>, A, B> ToClvk<E> for (A, B)
where
    A: ToClvk<E>,
    B: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        let first = self.0.to_clvk(encoder)?;
        let rest = self.1.to_clvk(encoder)?;
        encoder.encode_pair(first, rest)
    }
}

impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for () {
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        encoder.encode_atom(Atom::Borrowed(&[]))
    }
}

impl<N, E: ClvkEncoder<Node = N>, T> ToClvk<E> for &[T]
where
    T: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        let mut result = encoder.encode_atom(Atom::Borrowed(&[]))?;
        for item in self.iter().rev() {
            let value = item.to_clvk(encoder)?;
            result = encoder.encode_pair(value, result)?;
        }
        Ok(result)
    }
}

impl<N, E: ClvkEncoder<Node = N>, T, const LEN: usize> ToClvk<E> for [T; LEN]
where
    T: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        self.as_slice().to_clvk(encoder)
    }
}

impl<N, E: ClvkEncoder<Node = N>, T> ToClvk<E> for Vec<T>
where
    T: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        self.as_slice().to_clvk(encoder)
    }
}

impl<N, E: ClvkEncoder<Node = N>, T> ToClvk<E> for Option<T>
where
    T: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        match self {
            Some(value) => value.to_clvk(encoder),
            None => encoder.encode_atom(Atom::Borrowed(&[])),
        }
    }
}

impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for &str {
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        encoder.encode_atom(Atom::Borrowed(self.as_bytes()))
    }
}

impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for String {
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        self.as_str().to_clvk(encoder)
    }
}

#[cfg(feature = "chik-bls")]
impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for chik_bls::PublicKey {
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        encoder.encode_atom(Atom::Borrowed(&self.to_bytes()))
    }
}

#[cfg(feature = "chik-bls")]
impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for chik_bls::Signature {
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        encoder.encode_atom(Atom::Borrowed(&self.to_bytes()))
    }
}

#[cfg(feature = "chik-secp")]
impl<E> ToClvk<E> for chik_secp::K1PublicKey
where
    E: ClvkEncoder,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<E::Node, ToClvkError> {
        encoder.encode_atom(Atom::Borrowed(&self.to_bytes()))
    }
}

#[cfg(feature = "chik-secp")]
impl<E> ToClvk<E> for chik_secp::K1Signature
where
    E: ClvkEncoder,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<E::Node, ToClvkError> {
        encoder.encode_atom(Atom::Borrowed(&self.to_bytes()))
    }
}

#[cfg(feature = "chik-secp")]
impl<E> ToClvk<E> for chik_secp::R1PublicKey
where
    E: ClvkEncoder,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<E::Node, ToClvkError> {
        encoder.encode_atom(Atom::Borrowed(&self.to_bytes()))
    }
}

#[cfg(feature = "chik-secp")]
impl<E> ToClvk<E> for chik_secp::R1Signature
where
    E: ClvkEncoder,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<E::Node, ToClvkError> {
        encoder.encode_atom(Atom::Borrowed(&self.to_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use clvkr::{Allocator, serde::node_to_bytes};
    use hex::ToHex;

    use super::*;

    fn encode<T>(a: &mut Allocator, value: T) -> Result<String, ToClvkError>
    where
        T: ToClvk<Allocator>,
    {
        let actual = value.to_clvk(a)?;
        let actual_bytes = node_to_bytes(a, actual).unwrap();
        Ok(actual_bytes.encode_hex())
    }

    #[test]
    fn test_nodeptr() {
        let a = &mut Allocator::new();
        let ptr = a.one();
        assert_eq!(ptr.to_clvk(a).unwrap(), ptr);
    }

    #[test]
    fn test_primitives() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, 0u8), Ok("80".to_owned()));
        assert_eq!(encode(a, 0i8), Ok("80".to_owned()));
        assert_eq!(encode(a, 5u8), Ok("05".to_owned()));
        assert_eq!(encode(a, 5u32), Ok("05".to_owned()));
        assert_eq!(encode(a, 5i32), Ok("05".to_owned()));
        assert_eq!(encode(a, -27i32), Ok("81e5".to_owned()));
        assert_eq!(encode(a, -0), Ok("80".to_owned()));
        assert_eq!(encode(a, -128i8), Ok("8180".to_owned()));
    }

    #[test]
    fn test_bool() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, true), Ok("01".to_owned()));
        assert_eq!(encode(a, false), Ok("80".to_owned()));
    }

    #[test]
    fn test_reference() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, [1, 2, 3]), encode(a, [1, 2, 3]));
        assert_eq!(encode(a, Some(42)), encode(a, Some(42)));
        assert_eq!(encode(a, Some(&42)), encode(a, Some(42)));
        assert_eq!(encode(a, Some(&42)), encode(a, Some(42)));
    }

    #[test]
    fn test_smart_pointers() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, Box::new(42)), encode(a, 42));
        assert_eq!(encode(a, Rc::new(42)), encode(a, 42));
        assert_eq!(encode(a, Arc::new(42)), encode(a, 42));
    }

    #[test]
    fn test_pair() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, (5, 2)), Ok("ff0502".to_owned()));
        assert_eq!(
            encode(a, (-72, (90121, ()))),
            Ok("ff81b8ff8301600980".to_owned())
        );
        assert_eq!(
            encode(a, (((), ((), ((), (((), ((), ((), ()))), ())))), ())),
            Ok("ffff80ff80ff80ffff80ff80ff80808080".to_owned())
        );
    }

    #[test]
    fn test_nil() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, ()), Ok("80".to_owned()));
    }

    #[test]
    fn test_slice() {
        let a = &mut Allocator::new();
        assert_eq!(
            encode(a, [1, 2, 3, 4].as_slice()),
            Ok("ff01ff02ff03ff0480".to_owned())
        );
        assert_eq!(encode(a, [0; 0].as_slice()), Ok("80".to_owned()));
    }

    #[test]
    fn test_array() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, [1, 2, 3, 4]), Ok("ff01ff02ff03ff0480".to_owned()));
        assert_eq!(encode(a, [0; 0]), Ok("80".to_owned()));
    }

    #[test]
    fn test_vec() {
        let a = &mut Allocator::new();
        assert_eq!(
            encode(a, vec![1, 2, 3, 4]),
            Ok("ff01ff02ff03ff0480".to_owned())
        );
        assert_eq!(encode(a, vec![0; 0]), Ok("80".to_owned()));
    }

    #[test]
    fn test_option() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, Some("hello")), Ok("8568656c6c6f".to_owned()));
        assert_eq!(encode(a, None::<&str>), Ok("80".to_owned()));
        assert_eq!(encode(a, Some("")), Ok("80".to_owned()));
    }

    #[test]
    fn test_str() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, "hello"), Ok("8568656c6c6f".to_owned()));
        assert_eq!(encode(a, ""), Ok("80".to_owned()));
    }

    #[test]
    fn test_string() {
        let a = &mut Allocator::new();
        assert_eq!(
            encode(a, "hello".to_string()),
            Ok("8568656c6c6f".to_owned())
        );
        assert_eq!(encode(a, String::new()), Ok("80".to_owned()));
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
            encode(a, PublicKey::from_bytes(&bytes).unwrap()),
            Ok("b0b8f7dd239557ff8c49d338f89ac1a258a863fa52cd0a502e3aaae4b6738ba39ac8d982215aa3fa16bc5f8cb7e44b954d".to_string())
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
            encode(a, Signature::from_bytes(&bytes).unwrap()),
            Ok("c060a3994dc9c0ef41a903d3335f0afe42ba16c88e7881706798492da4a1653cd10c69c841eeb56f44ae005e2bad27fb7ebb16ce8bbfbd708ea91dd4ff24f030497b50e694a8270eccd07dbc206b8ffe0c34a9ea81291785299fae8206a1e1bbc1d1".to_string())
        );
    }

    #[cfg(feature = "chik-secp")]
    #[test]
    fn test_secp_public_key() {
        use chik_secp::{K1PublicKey, R1PublicKey};
        use hex_literal::hex;

        let a = &mut Allocator::new();

        let k1_pk = K1PublicKey::from_bytes(&hex!(
            "02827cdbbed87e45683d448be2ea15fb72ba3732247bda18474868cf5456123fb4"
        ))
        .unwrap();
        assert_eq!(
            encode(a, k1_pk),
            Ok("a102827cdbbed87e45683d448be2ea15fb72ba3732247bda18474868cf5456123fb4".to_string())
        );

        let r1_pk = R1PublicKey::from_bytes(&hex!(
            "037dc85102f5eb7867b9580fea8b242c774173e1a47db320c798242d3a7a7579e4"
        ))
        .unwrap();
        assert_eq!(
            encode(a, r1_pk),
            Ok("a1037dc85102f5eb7867b9580fea8b242c774173e1a47db320c798242d3a7a7579e4".to_string())
        );
    }

    #[cfg(feature = "chik-secp")]
    #[test]
    fn test_secp_signature() {
        use chik_secp::{K1Signature, R1Signature};
        use hex_literal::hex;

        let a = &mut Allocator::new();

        let k1_sig = K1Signature::from_bytes(&hex!(
            "6f07897d1d28b8698af5dec5ca06907b1304b227dc9f740b8c4065cf04d5e8653ae66aa17063e7120ee7f22fae54373b35230e259244b90400b65cf00d86c591"
        ))
        .unwrap();
        assert_eq!(
            encode(a, k1_sig),
            Ok("c0406f07897d1d28b8698af5dec5ca06907b1304b227dc9f740b8c4065cf04d5e8653ae66aa17063e7120ee7f22fae54373b35230e259244b90400b65cf00d86c591".to_string())
        );

        let r1_sig = R1Signature::from_bytes(&hex!(
            "550e83da8cf9b2d407ed093ae213869ebd7ceaea603920f87d535690e52b40537915d8fe3d5a96c87e700c56dc638c32f7a2954f2ba409367d1a132000cc2228"
        ))
        .unwrap();
        assert_eq!(
            encode(a, r1_sig),
            Ok("c040550e83da8cf9b2d407ed093ae213869ebd7ceaea603920f87d535690e52b40537915d8fe3d5a96c87e700c56dc638c32f7a2954f2ba409367d1a132000cc2228".to_string())
        );
    }
}
