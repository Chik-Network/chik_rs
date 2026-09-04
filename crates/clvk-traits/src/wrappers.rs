use crate::{ClvkDecoder, ClvkEncoder, FromClvk, FromClvkError, ToClvk, ToClvkError};

/// A wrapper for an intermediate CLVK value. This is required to
/// implement `ToClvk` and `FromClvk` for `N`, since the compiler
/// cannot guarantee that the generic `N` type doesn't already
/// implement these traits.
pub struct Raw<N>(pub N);

impl<N, D: ClvkDecoder<Node = N>> FromClvk<D> for Raw<N> {
    fn from_clvk(_decoder: &D, node: N) -> Result<Self, FromClvkError> {
        Ok(Self(node))
    }
}

impl<N, E: ClvkEncoder<Node = N>> ToClvk<E> for Raw<N> {
    fn to_clvk(&self, encoder: &mut E) -> Result<N, ToClvkError> {
        Ok(encoder.clone_node(&self.0))
    }
}
