use clvkr::{Allocator, Atom, NodePtr, allocator::SExp};
use num_bigint::BigInt;

use crate::{
    FromClvk, FromClvkError, MatchByte, destructure_list, destructure_quote, match_list,
    match_quote,
};

pub trait ClvkDecoder: Sized {
    type Node: Clone + FromClvk<Self>;

    fn decode_atom(&self, node: &Self::Node) -> Result<Atom<'_>, FromClvkError>;
    fn decode_pair(&self, node: &Self::Node) -> Result<(Self::Node, Self::Node), FromClvkError>;

    fn decode_bigint(&self, node: &Self::Node) -> Result<BigInt, FromClvkError> {
        let atom = self.decode_atom(node)?;
        Ok(BigInt::from_signed_bytes_be(atom.as_ref()))
    }

    fn decode_curried_arg(
        &self,
        node: &Self::Node,
    ) -> Result<(Self::Node, Self::Node), FromClvkError> {
        let destructure_list!(_, destructure_quote!(first), rest) =
            <match_list!(MatchByte<4>, match_quote!(Self::Node), Self::Node)>::from_clvk(
                self,
                node.clone(),
            )?;
        Ok((first, rest))
    }

    /// This is a helper function that just calls `clone` on the node.
    /// It's required only because the compiler can't infer that `N` is `Clone`,
    /// since there's no `Clone` bound on the `FromClvk` trait.
    fn clone_node(&self, node: &Self::Node) -> Self::Node {
        node.clone()
    }
}

impl ClvkDecoder for Allocator {
    type Node = NodePtr;

    fn decode_atom(&self, node: &Self::Node) -> Result<Atom<'_>, FromClvkError> {
        if let SExp::Atom = self.sexp(*node) {
            Ok(self.atom(*node))
        } else {
            Err(FromClvkError::ExpectedAtom)
        }
    }

    fn decode_pair(&self, node: &Self::Node) -> Result<(Self::Node, Self::Node), FromClvkError> {
        if let SExp::Pair(first, rest) = self.sexp(*node) {
            Ok((first, rest))
        } else {
            Err(FromClvkError::ExpectedPair)
        }
    }

    fn decode_bigint(&self, node: &Self::Node) -> Result<BigInt, FromClvkError> {
        if let SExp::Atom = self.sexp(*node) {
            Ok(self.number(*node))
        } else {
            Err(FromClvkError::ExpectedAtom)
        }
    }
}

impl FromClvk<Allocator> for NodePtr {
    fn from_clvk(_decoder: &Allocator, node: NodePtr) -> Result<Self, FromClvkError> {
        Ok(node)
    }
}
