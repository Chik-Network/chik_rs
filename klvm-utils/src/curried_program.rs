use klvm_traits::{
    destructure_list, destructure_quote, klvm_list, klvm_quote, match_list, match_quote, FromKlvm,
    MatchByte, Result, ToKlvm,
};
use klvmr::{allocator::NodePtr, Allocator};

#[derive(Debug, Clone)]
pub struct CurriedProgram<T> {
    pub program: NodePtr,
    pub args: T,
}

impl<T> FromKlvm for CurriedProgram<T>
where
    T: FromKlvm,
{
    fn from_klvm(a: &Allocator, ptr: NodePtr) -> Result<Self> {
        let destructure_list!(_, destructure_quote!(program), args) =
            <match_list!(MatchByte<2>, match_quote!(NodePtr), T)>::from_klvm(a, ptr)?;

        Ok(Self { program, args })
    }
}

impl<T> ToKlvm for CurriedProgram<T>
where
    T: ToKlvm,
{
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        klvm_list!(2, klvm_quote!(self.program), self.args.to_klvm(a)?).to_klvm(a)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use klvm_traits::klvm_curried_args;
    use klvmr::serde::node_to_bytes;

    use super::*;

    fn check<T, A>(program: T, args: A, expected: &str)
    where
        T: Debug + ToKlvm + PartialEq + FromKlvm,
        A: Debug + Clone + PartialEq + ToKlvm + FromKlvm,
    {
        let a = &mut Allocator::new();

        let curry = CurriedProgram {
            program: program.to_klvm(a).unwrap(),
            args: args.clone(),
        }
        .to_klvm(a)
        .unwrap();
        let actual = node_to_bytes(a, curry).unwrap();
        assert_eq!(hex::encode(actual), expected);

        let curried = CurriedProgram::<A>::from_klvm(a, curry).unwrap();
        let round_program = T::from_klvm(a, curried.program).unwrap();
        assert_eq!(round_program, program);
        assert_eq!(curried.args, args);
    }

    #[test]
    fn curry() {
        check(
            "xyz".to_string(),
            klvm_curried_args!("a".to_string(), "b".to_string(), "c".to_string()),
            "ff02ffff018378797affff04ffff0161ffff04ffff0162ffff04ffff0163ff0180808080",
        );
    }
}
