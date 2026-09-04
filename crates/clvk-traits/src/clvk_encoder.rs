use clvkr::{Allocator, Atom, NodePtr};
use num_bigint::BigInt;

use crate::{ToClvk, ToClvkError, clvk_list, clvk_quote};

pub trait ClvkEncoder: Sized {
    type Node: Clone + ToClvk<Self>;

    fn encode_atom(&mut self, atom: Atom<'_>) -> Result<Self::Node, ToClvkError>;
    fn encode_pair(
        &mut self,
        first: Self::Node,
        rest: Self::Node,
    ) -> Result<Self::Node, ToClvkError>;

    fn encode_bigint(&mut self, number: BigInt) -> Result<Self::Node, ToClvkError> {
        let bytes = number.to_signed_bytes_be();
        let mut slice = bytes.as_slice();

        // Remove leading zeros.
        while !slice.is_empty() && slice[0] == 0 {
            if slice.len() > 1 && (slice[1] & 0x80 == 0x80) {
                break;
            }
            slice = &slice[1..];
        }

        self.encode_atom(Atom::Borrowed(slice))
    }

    fn encode_curried_arg(
        &mut self,
        first: Self::Node,
        rest: Self::Node,
    ) -> Result<Self::Node, ToClvkError> {
        const OP_C: u8 = 4;
        clvk_list!(OP_C, clvk_quote!(first), rest).to_clvk(self)
    }

    /// This is a helper function that just calls `clone` on the node.
    /// It's required only because the compiler can't infer that `N` is `Clone`,
    /// since there's no `Clone` bound on the `ToClvk` trait.
    fn clone_node(&self, node: &Self::Node) -> Self::Node {
        node.clone()
    }
}

impl ClvkEncoder for Allocator {
    type Node = NodePtr;

    fn encode_atom(&mut self, atom: Atom<'_>) -> Result<Self::Node, ToClvkError> {
        match atom {
            Atom::Borrowed(bytes) => self.new_atom(bytes),
            Atom::U32(bytes, _len) => self.new_small_number(u32::from_be_bytes(bytes)),
        }
        .or(Err(ToClvkError::OutOfMemory))
    }

    fn encode_pair(
        &mut self,
        first: Self::Node,
        rest: Self::Node,
    ) -> Result<Self::Node, ToClvkError> {
        self.new_pair(first, rest).or(Err(ToClvkError::OutOfMemory))
    }

    fn encode_bigint(&mut self, number: BigInt) -> Result<Self::Node, ToClvkError> {
        self.new_number(number).or(Err(ToClvkError::OutOfMemory))
    }
}

impl ToClvk<Allocator> for NodePtr {
    fn to_clvk(&self, _encoder: &mut Allocator) -> Result<NodePtr, ToClvkError> {
        Ok(*self)
    }
}
