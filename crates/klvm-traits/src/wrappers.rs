use crate::{FromKlvm, FromKlvmError, KlvmDecoder, KlvmEncoder, ToKlvm, ToKlvmError};

/// A wrapper for an intermediate KLVM value. This is required to
/// implement `ToKlvm` and `FromKlvm` for `N`, since the compiler
/// cannot guarantee that the generic `N` type doesn't already
/// implement these traits.
pub struct Raw<N>(pub N);

impl<N> FromKlvm<N> for Raw<N> {
    fn from_klvm(_decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        Ok(Self(node))
    }
}

impl<N> ToKlvm<N> for Raw<N> {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        Ok(encoder.clone_node(&self.0))
    }
}
