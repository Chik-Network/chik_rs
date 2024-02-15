use klvmr::{allocator::NodePtr, Allocator};
use num_bigint::BigInt;

use crate::{KlvmEncoder, ToKlvmError};

pub trait ToKlvm<N> {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError>;
}

pub trait ToNodePtr {
    fn to_node_ptr(&self, a: &mut Allocator) -> Result<NodePtr, ToKlvmError>;
}

impl<T> ToNodePtr for T
where
    T: ToKlvm<NodePtr>,
{
    fn to_node_ptr(&self, a: &mut Allocator) -> Result<NodePtr, ToKlvmError> {
        self.to_klvm(a)
    }
}

impl ToKlvm<NodePtr> for NodePtr {
    fn to_klvm(
        &self,
        _encoder: &mut impl KlvmEncoder<Node = NodePtr>,
    ) -> Result<NodePtr, ToKlvmError> {
        Ok(*self)
    }
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

#[cfg(test)]
mod tests {
    use hex::ToHex;
    use klvmr::{serde::node_to_bytes, Allocator};

    use super::*;

    fn encode<T>(a: &mut Allocator, value: T) -> Result<String, ToKlvmError>
    where
        T: ToKlvm<NodePtr>,
    {
        let actual = value.to_klvm(a).unwrap();
        let actual_bytes = node_to_bytes(a, actual).unwrap();
        Ok(actual_bytes.encode_hex())
    }

    #[test]
    fn test_nodeptr() {
        let a = &mut Allocator::new();
        let ptr = a.one();
        assert_eq!(ptr.to_klvm(a).unwrap(), ptr);
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
    fn test_reference() {
        let a = &mut Allocator::new();
        assert_eq!(encode(a, [1, 2, 3]), encode(a, [1, 2, 3]));
        assert_eq!(encode(a, Some(42)), encode(a, Some(42)));
        assert_eq!(encode(a, Some(&42)), encode(a, Some(42)));
        assert_eq!(encode(a, Some(&42)), encode(a, Some(42)));
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
        assert_eq!(encode(a, "".to_string()), Ok("80".to_owned()));
    }
}
