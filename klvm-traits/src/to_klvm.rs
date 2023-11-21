use klvmr::{allocator::NodePtr, Allocator};

use crate::{Error, Result};

pub trait ToKlvm {
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr>;
}

impl ToKlvm for NodePtr {
    fn to_klvm(&self, _a: &mut Allocator) -> Result<NodePtr> {
        Ok(*self)
    }
}

macro_rules! klvm_primitive {
    ($primitive:ty) => {
        impl ToKlvm for $primitive {
            fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
                a.new_number((*self).into()).map_err(Error::Allocator)
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

impl<T> ToKlvm for &T
where
    T: ToKlvm,
{
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        T::to_klvm(*self, a)
    }
}

impl<A, B> ToKlvm for (A, B)
where
    A: ToKlvm,
    B: ToKlvm,
{
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        let first = self.0.to_klvm(a)?;
        let rest = self.1.to_klvm(a)?;
        Ok(a.new_pair(first, rest)?)
    }
}

impl ToKlvm for () {
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        Ok(a.null())
    }
}

impl<T> ToKlvm for &[T]
where
    T: ToKlvm,
{
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        let mut result = a.null();
        for item in self.iter().rev() {
            let value = item.to_klvm(a)?;
            result = a.new_pair(value, result)?;
        }
        Ok(result)
    }
}

impl<T, const N: usize> ToKlvm for [T; N]
where
    T: ToKlvm,
{
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        self.as_slice().to_klvm(a)
    }
}

impl<T> ToKlvm for Vec<T>
where
    T: ToKlvm,
{
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        self.as_slice().to_klvm(a)
    }
}

impl<T> ToKlvm for Option<T>
where
    T: ToKlvm,
{
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        match self {
            Some(value) => value.to_klvm(a),
            None => Ok(a.null()),
        }
    }
}

impl ToKlvm for &str {
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        Ok(a.new_atom(self.as_bytes())?)
    }
}

impl ToKlvm for String {
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        self.as_str().to_klvm(a)
    }
}

#[cfg(test)]
mod tests {
    use hex::ToHex;
    use klvmr::serde::node_to_bytes;

    use super::*;

    fn encode<T>(a: &mut Allocator, value: T) -> Result<String>
    where
        T: ToKlvm,
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
