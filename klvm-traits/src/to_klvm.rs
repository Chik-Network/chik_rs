use num_bigint::BigInt;

use crate::{KlvmEncoder, ToKlvmError};

pub trait ToKlvm<N> {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError>;
}

pub fn simplify_int_bytes(mut slice: &[u8]) -> &[u8] {
    while (!slice.is_empty()) && (slice[0] == 0) {
        if slice.len() > 1 && (slice[1] & 0x80 == 0x80) {
            break;
        }
        slice = &slice[1..];
    }
    slice
}

macro_rules! klvm_primitive {
    ($primitive:ty) => {
        impl<N> ToKlvm<N> for $primitive {
            fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
                let number = BigInt::from(*self);
                encoder.encode_atom(simplify_int_bytes(&number.to_signed_bytes_be()))
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

impl<N, T> ToKlvm<N> for &T
where
    T: ToKlvm<N>,
{
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        T::to_klvm(*self, encoder)
    }
}

impl<N, A, B> ToKlvm<N> for (A, B)
where
    A: ToKlvm<N>,
    B: ToKlvm<N>,
{
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        let first = self.0.to_klvm(encoder)?;
        let rest = self.1.to_klvm(encoder)?;
        encoder.encode_pair(first, rest)
    }
}

impl<N> ToKlvm<N> for () {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        encoder.encode_atom(&[])
    }
}

impl<N, T> ToKlvm<N> for &[T]
where
    T: ToKlvm<N>,
{
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        let mut result = encoder.encode_atom(&[])?;
        for item in self.iter().rev() {
            let value = item.to_klvm(encoder)?;
            result = encoder.encode_pair(value, result)?;
        }
        Ok(result)
    }
}

impl<N, T, const LEN: usize> ToKlvm<N> for [T; LEN]
where
    T: ToKlvm<N>,
{
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        self.as_slice().to_klvm(encoder)
    }
}

impl<N, T> ToKlvm<N> for Vec<T>
where
    T: ToKlvm<N>,
{
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        self.as_slice().to_klvm(encoder)
    }
}

impl<N, T> ToKlvm<N> for Option<T>
where
    T: ToKlvm<N>,
{
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        match self {
            Some(value) => value.to_klvm(encoder),
            None => encoder.encode_atom(&[]),
        }
    }
}

impl<N> ToKlvm<N> for &str {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        encoder.encode_atom(self.as_bytes())
    }
}

impl<N> ToKlvm<N> for String {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        self.as_str().to_klvm(encoder)
    }
}

#[cfg(feature = "chik-bls")]
impl<N> ToKlvm<N> for chik_bls::PublicKey {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        encoder.encode_atom(&self.to_bytes())
    }
}

#[cfg(feature = "chik-bls")]
impl<N> ToKlvm<N> for chik_bls::Signature {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        encoder.encode_atom(&self.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{node_to_str, TestAllocator, TestNode};

    use super::*;

    fn encode<T>(value: T) -> Result<String, ToKlvmError>
    where
        T: ToKlvm<TestNode>,
    {
        let mut a = TestAllocator::new();
        let node = value.to_klvm(&mut a).unwrap();
        Ok(node_to_str(&a, &node))
    }

    #[test]
    fn test_primitives() {
        assert_eq!(encode(0u8), Ok("NIL".to_owned()));
        assert_eq!(encode(0i8), Ok("NIL".to_owned()));
        assert_eq!(encode(5u8), Ok("05".to_owned()));
        assert_eq!(encode(5u32), Ok("05".to_owned()));
        assert_eq!(encode(5i32), Ok("05".to_owned()));
        assert_eq!(encode(-27i32), Ok("e5".to_owned()));
        assert_eq!(encode(-0), Ok("NIL".to_owned()));
        assert_eq!(encode(-128i8), Ok("80".to_owned()));
    }

    #[test]
    fn test_reference() {
        assert_eq!(encode([1, 2, 3]), encode([1, 2, 3]));
        assert_eq!(encode(Some(42)), encode(Some(42)));
        assert_eq!(encode(Some(&42)), encode(Some(42)));
        assert_eq!(encode(Some(&42)), encode(Some(42)));
    }

    #[test]
    fn test_pair() {
        assert_eq!(encode((5, 2)), Ok("( 05 02".to_owned()));
        assert_eq!(
            encode((-72, (90121, ()))),
            Ok("( b8 ( 016009 NIL".to_owned())
        );
        assert_eq!(
            encode((((), ((), ((), (((), ((), ((), ()))), ())))), ())),
            Ok("( ( NIL ( NIL ( NIL ( ( NIL ( NIL ( NIL NIL NIL NIL".to_owned())
        );
    }

    #[test]
    fn test_nil() {
        assert_eq!(encode(()), Ok("NIL".to_owned()));
    }

    #[test]
    fn test_slice() {
        assert_eq!(
            encode([1, 2, 3, 4].as_slice()),
            Ok("( 01 ( 02 ( 03 ( 04 NIL".to_owned())
        );
        assert_eq!(encode([0; 0].as_slice()), Ok("NIL".to_owned()));
    }

    #[test]
    fn test_array() {
        assert_eq!(
            encode([1, 2, 3, 4]),
            Ok("( 01 ( 02 ( 03 ( 04 NIL".to_owned())
        );
        assert_eq!(encode([0; 0]), Ok("NIL".to_owned()));
    }

    #[test]
    fn test_vec() {
        assert_eq!(
            encode(vec![1, 2, 3, 4]),
            Ok("( 01 ( 02 ( 03 ( 04 NIL".to_owned())
        );
        assert_eq!(encode(vec![0; 0]), Ok("NIL".to_owned()));
    }

    #[test]
    fn test_option() {
        assert_eq!(encode(Some("hello")), Ok("68656c6c6f".to_owned()));
        assert_eq!(encode(None::<&str>), Ok("NIL".to_owned()));
        assert_eq!(encode(Some("")), Ok("NIL".to_owned()));
    }

    #[test]
    fn test_str() {
        assert_eq!(encode("hello"), Ok("68656c6c6f".to_owned()));
        assert_eq!(encode(""), Ok("NIL".to_owned()));
    }

    #[test]
    fn test_string() {
        assert_eq!(encode("hello".to_string()), Ok("68656c6c6f".to_owned()));
        assert_eq!(encode("".to_string()), Ok("NIL".to_owned()));
    }

    #[cfg(feature = "chik-bls")]
    #[test]
    fn test_public_key() {
        use chik_bls::PublicKey;
        use hex_literal::hex;

        let valid_bytes = hex!("b8f7dd239557ff8c49d338f89ac1a258a863fa52cd0a502e3aaae4b6738ba39ac8d982215aa3fa16bc5f8cb7e44b954d");
        assert_eq!(
            encode(PublicKey::from_bytes(&valid_bytes).unwrap()),
            Ok(hex::encode(valid_bytes))
        );
    }

    #[cfg(feature = "chik-bls")]
    #[test]
    fn test_signature() {
        use chik_bls::Signature;
        use hex_literal::hex;

        let valid_bytes = hex!("a3994dc9c0ef41a903d3335f0afe42ba16c88e7881706798492da4a1653cd10c69c841eeb56f44ae005e2bad27fb7ebb16ce8bbfbd708ea91dd4ff24f030497b50e694a8270eccd07dbc206b8ffe0c34a9ea81291785299fae8206a1e1bbc1d1");
        assert_eq!(
            encode(Signature::from_bytes(&valid_bytes).unwrap()),
            Ok(hex::encode(valid_bytes))
        );
    }
}
