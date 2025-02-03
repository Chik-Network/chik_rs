use crate::{FromKlvm, FromKlvmError, KlvmDecoder, KlvmEncoder, ToKlvm, ToKlvmError};

/// A wrapper for an intermediate KLVM value. This is required to
/// implement `ToKlvm` and `FromKlvm` for `N`, since the compiler
/// cannot guarantee that the generic `N` type doesn't already
/// implement these traits.
pub struct Raw<N>(pub N);

impl<N, D: KlvmDecoder<Node = N>> FromKlvm<D> for Raw<N> {
    fn from_klvm(_decoder: &D, node: N) -> Result<Self, FromKlvmError> {
        Ok(Self(node))
    }
}

impl<N, E: KlvmEncoder<Node = N>> ToKlvm<E> for Raw<N> {
    fn to_klvm(&self, encoder: &mut E) -> Result<N, ToKlvmError> {
        Ok(encoder.clone_node(&self.0))
    }
}
