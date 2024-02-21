use klvmr::{allocator::SExp, Allocator, NodePtr};

use crate::{FromKlvm, FromKlvmError};

pub trait KlvmDecoder {
    type Node: Clone;

    fn decode_atom(&self, node: &Self::Node) -> Result<&[u8], FromKlvmError>;
    fn decode_pair(&self, node: &Self::Node) -> Result<(Self::Node, Self::Node), FromKlvmError>;

    /// This is a helper function that just calls `clone` on the node.
    /// It's required only because the compiler can't infer that `N` is `Clone`,
    /// since there's no `Clone` bound on the `FromKlvm` trait.
    fn clone_node(&self, node: &Self::Node) -> Self::Node {
        node.clone()
    }
}

impl KlvmDecoder for Allocator {
    type Node = NodePtr;

    fn decode_atom(&self, node: &Self::Node) -> Result<&[u8], FromKlvmError> {
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
