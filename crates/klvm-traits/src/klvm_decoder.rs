use klvmr::{allocator::SExp, Allocator, Atom, NodePtr};

use crate::{
    destructure_list, destructure_quote, match_list, match_quote, FromKlvm, FromKlvmError,
    MatchByte,
};

pub trait KlvmDecoder: Sized {
    type Node: Clone + FromKlvm<Self::Node>;

    fn decode_atom(&self, node: &Self::Node) -> Result<Atom<'_>, FromKlvmError>;
    fn decode_pair(&self, node: &Self::Node) -> Result<(Self::Node, Self::Node), FromKlvmError>;

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
}

pub trait FromNodePtr {
    fn from_node_ptr(a: &Allocator, node: NodePtr) -> Result<Self, FromKlvmError>
    where
        Self: Sized;
}

impl<T> FromNodePtr for T
where
    T: FromKlvm<NodePtr>,
{
    fn from_node_ptr(a: &Allocator, node: NodePtr) -> Result<Self, FromKlvmError>
    where
        Self: Sized,
    {
        T::from_klvm(a, node)
    }
}

impl FromKlvm<NodePtr> for NodePtr {
    fn from_klvm(
        _decoder: &impl KlvmDecoder<Node = NodePtr>,
        node: NodePtr,
    ) -> Result<Self, FromKlvmError> {
        Ok(node)
    }
}
