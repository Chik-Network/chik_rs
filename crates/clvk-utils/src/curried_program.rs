use clvk_traits::{
    ClvkDecoder, ClvkEncoder, FromClvk, FromClvkError, MatchByte, ToClvk, ToClvkError, clvk_list,
    clvk_quote, destructure_list, destructure_quote, match_list, match_quote,
};

#[derive(Debug, Clone)]
pub struct CurriedProgram<P, A> {
    pub program: P,
    pub args: A,
}

impl<N, D: ClvkDecoder<Node = N>, P, A> FromClvk<D> for CurriedProgram<P, A>
where
    P: FromClvk<D>,
    A: FromClvk<D>,
{
    fn from_clvk(decoder: &D, node: N) -> Result<Self, FromClvkError> {
        let destructure_list!(_, destructure_quote!(program), args) =
            <match_list!(MatchByte<2>, match_quote!(P), A)>::from_clvk(decoder, node)?;
        Ok(Self { program, args })
    }
}

impl<N, E: ClvkEncoder<Node = N>, P, A> ToClvk<E> for CurriedProgram<P, A>
where
    P: ToClvk<E>,
    A: ToClvk<E>,
{
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        clvk_list!(2, clvk_quote!(&self.program), &self.args).to_clvk(encoder)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use clvk_traits::clvk_curried_args;
    use clvkr::{Allocator, serde::node_to_bytes};

    use super::*;

    fn check<P, A>(program: &P, args: &A, expected: &str)
    where
        P: Debug + PartialEq + ToClvk<Allocator> + FromClvk<Allocator>,
        A: Debug + PartialEq + ToClvk<Allocator> + FromClvk<Allocator>,
    {
        let a = &mut Allocator::new();

        let curry = CurriedProgram {
            program: &program,
            args: &args,
        }
        .to_clvk(a)
        .unwrap();
        let actual = node_to_bytes(a, curry).unwrap();
        assert_eq!(hex::encode(actual), expected);

        let curried = CurriedProgram::<P, A>::from_clvk(a, curry).unwrap();
        assert_eq!(&curried.program, program);
        assert_eq!(&curried.args, args);
    }

    #[test]
    fn curry() {
        check(
            &"xyz".to_string(),
            &clvk_curried_args!("a".to_string(), "b".to_string(), "c".to_string()),
            "ff02ffff018378797affff04ffff0161ffff04ffff0162ffff04ffff0163ff0180808080",
        );
    }
}
