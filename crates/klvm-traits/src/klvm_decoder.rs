use klvmr::{allocator::SExp, Allocator, Atom, NodePtr};
use num_bigint::BigInt;

use crate::{
    destructure_list, destructure_quote, match_list, match_quote, FromKlvm, FromKlvmError,
    MatchByte,
};

pub trait KlvmDecoder: Sized {
    type Node: Clone + FromKlvm<Self>;

    fn decode_atom(&self, node: &Self::Node) -> Result<Atom<'_>, FromKlvmError>;
    fn decode_pair(&self, node: &Self::Node) -> Result<(Self::Node, Self::Node), FromKlvmError>;

    fn decode_bigint(&self, node: &Self::Node) -> Result<BigInt, FromKlvmError> {
        let atom = self.decode_atom(node)?;
        Ok(BigInt::from_signed_bytes_be(atom.as_ref()))
    }

    fn decode_curried_arg(
        &self,
        node: &Self::Node,
    ) -> Result<(Self::Node, Self::Node), FromKlvmError> {
        let destructure_list!(_, destructure_quote!(first), rest) =
            <match_list!(MatchByte<4>, match_quote!(Self::Node), Self::Node)>::from_klvm(
                self,
                node.clone(),
            )?;
        Ok((first, rest))
    }

    /// This is a helper function that just calls `clone` on the node.
    /// It's required only because the compiler can't infer that `N` is `Clone`,
    /// since there's no `Clone` bound on the `FromKlvm` trait.
    fn clone_node(&self, node: &Self::Node) -> Self::Node {
        node.clone()
    }
}

impl KlvmDecoder for Allocator {
    type Node = NodePtr;

    fn decode_atom(&self, node: &Self::Node) -> Result<Atom<'_>, FromKlvmError> {
        if let SExp::Atom = self.sexp(*node) {
            Ok(self.atom(*node))
        } else {
            Err(FromKlvmError::ExpectedAtom)
        }
    }

    fn decode_pair(&self, node: &Self::Node) -> Result<(Self::Node, Self::Node), FromKlvmError> {
        if let SExp::Pair(first, rest) = self.sexp(*node) {
            Ok((first, rest))
        } else {
            Err(FromKlvmError::ExpectedPair)
        }
    }

    fn decode_bigint(&self, node: &Self::Node) -> Result<BigInt, FromKlvmError> {
        if let SExp::Atom = self.sexp(*node) {
            Ok(self.number(*node))
        } else {
            Err(FromKlvmError::ExpectedAtom)
        }
    }
}

impl FromKlvm<Allocator> for NodePtr {
    fn from_klvm(_decoder: &Allocator, node: NodePtr) -> Result<Self, FromKlvmError> {
        Ok(node)
    }
}
