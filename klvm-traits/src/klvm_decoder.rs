use crate::FromKlvmError;

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
