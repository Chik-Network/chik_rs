use klvm_traits::{
    destructure_list, destructure_quote, klvm_list, klvm_quote, match_list, match_quote, FromKlvm,
    FromKlvmError, KlvmDecoder, KlvmEncoder, MatchByte, ToKlvm, ToKlvmError,
};

#[derive(Debug, Clone)]
pub struct CurriedProgram<P, A> {
    pub program: P,
    pub args: A,
}

impl<N, D: KlvmDecoder<Node = N>, P, A> FromKlvm<D> for CurriedProgram<P, A>
where
    P: FromKlvm<D>,
    A: FromKlvm<D>,
{
    fn from_klvm(decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        let destructure_list!(_, destructure_quote!(program), args) =
            <match_list!(MatchByte<2>, match_quote!(P), A)>::from_klvm(decoder, node)?;
        Ok(Self { program, args })
    }
}

impl<N, E: KlvmEncoder<Node = N>, P, A> ToKlvm<E> for CurriedProgram<P, A>
where
    P: ToKlvm<E>,
    A: ToKlvm<E>,
{
    fn to_klvm(&self, encoder: &mut E) -> Result<N, ToKlvmError> {
        klvm_list!(2, klvm_quote!(&self.program), &self.args).to_klvm(encoder)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use klvm_traits::klvm_curried_args;
    use klvmr::{serde::node_to_bytes, Allocator};

    use super::*;

    fn check<P, A>(program: &P, args: &A, expected: &str)
    where
        P: Debug + PartialEq + ToKlvm<Allocator> + FromKlvm<Allocator>,
        A: Debug + PartialEq + ToKlvm<Allocator> + FromKlvm<Allocator>,
    {
        let a = &mut Allocator::new();

        let curry = CurriedProgram {
            program: &program,
            args: &args,
        }
        .to_klvm(a)
        .unwrap();
        let actual = node_to_bytes(a, curry).unwrap();
        assert_eq!(hex::encode(actual), expected);

        let curried = CurriedProgram::<P, A>::from_klvm(a, curry).unwrap();
        assert_eq!(&curried.program, program);
        assert_eq!(&curried.args, args);
    }

    #[test]
    fn curry() {
        check(
            &"xyz".to_string(),
            &klvm_curried_args!("a".to_string(), "b".to_string(), "c".to_string()),
            "ff02ffff018378797affff04ffff0161ffff04ffff0162ffff04ffff0163ff0180808080",
        );
    }
}
