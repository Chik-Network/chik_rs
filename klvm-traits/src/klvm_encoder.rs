use klvmr::{Allocator, NodePtr};

use crate::{ToKlvm, ToKlvmError};

pub trait KlvmEncoder {
    type Node: Clone;

    fn encode_atom(&mut self, bytes: &[u8]) -> Result<Self::Node, ToKlvmError>;
    fn encode_pair(
        &mut self,
        first: Self::Node,
        rest: Self::Node,
    ) -> Result<Self::Node, ToKlvmError>;

    /// This is a helper function that just calls `clone` on the node.
    /// It's required only because the compiler can't infer that `N` is `Clone`,
    /// since there's no `Clone` bound on the `ToKlvm` trait.
    fn clone_node(&self, node: &Self::Node) -> Self::Node {
        node.clone()
    }
}

impl KlvmEncoder for Allocator {
    type Node = NodePtr;

    fn encode_atom(&mut self, bytes: &[u8]) -> Result<Self::Node, ToKlvmError> {
        self.new_atom(bytes).or(Err(ToKlvmError::OutOfMemory))
    }

    fn encode_pair(
        &mut self,
        first: Self::Node,
        rest: Self::Node,
    ) -> Result<Self::Node, ToKlvmError> {
        self.new_pair(first, rest).or(Err(ToKlvmError::OutOfMemory))
    }
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
