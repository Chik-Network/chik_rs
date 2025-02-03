use klvmr::{Allocator, Atom, NodePtr};
use num_bigint::BigInt;

use crate::{klvm_list, klvm_quote, ToKlvm, ToKlvmError};

pub trait KlvmEncoder: Sized {
    type Node: Clone + ToKlvm<Self>;

    fn encode_atom(&mut self, atom: Atom<'_>) -> Result<Self::Node, ToKlvmError>;
    fn encode_pair(
        &mut self,
        first: Self::Node,
        rest: Self::Node,
    ) -> Result<Self::Node, ToKlvmError>;

    fn encode_bigint(&mut self, number: BigInt) -> Result<Self::Node, ToKlvmError> {
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
    ) -> Result<Self::Node, ToKlvmError> {
        const OP_C: u8 = 4;
        klvm_list!(OP_C, klvm_quote!(first), rest).to_klvm(self)
    }

    /// This is a helper function that just calls `clone` on the node.
    /// It's required only because the compiler can't infer that `N` is `Clone`,
    /// since there's no `Clone` bound on the `ToKlvm` trait.
    fn clone_node(&self, node: &Self::Node) -> Self::Node {
        node.clone()
    }
}

impl KlvmEncoder for Allocator {
    type Node = NodePtr;

    fn encode_atom(&mut self, atom: Atom<'_>) -> Result<Self::Node, ToKlvmError> {
        match atom {
            Atom::Borrowed(bytes) => self.new_atom(bytes),
            Atom::U32(bytes, _len) => self.new_small_number(u32::from_be_bytes(bytes)),
        }
        .or(Err(ToKlvmError::OutOfMemory))
    }

    fn encode_pair(
        &mut self,
        first: Self::Node,
        rest: Self::Node,
    ) -> Result<Self::Node, ToKlvmError> {
        self.new_pair(first, rest).or(Err(ToKlvmError::OutOfMemory))
    }

    fn encode_bigint(&mut self, number: BigInt) -> Result<Self::Node, ToKlvmError> {
        self.new_number(number).or(Err(ToKlvmError::OutOfMemory))
    }
}

impl ToKlvm<Allocator> for NodePtr {
    fn to_klvm(&self, _encoder: &mut Allocator) -> Result<NodePtr, ToKlvmError> {
        Ok(*self)
    }
}
