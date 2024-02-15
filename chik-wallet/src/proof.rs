use arbitrary::{Arbitrary, Unstructured};
use klvm_traits::{FromKlvm, FromKlvmError, KlvmDecoder, KlvmEncoder, ToKlvm, ToKlvmError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proof {
    Lineage(LineageProof),
    Eve(EveProof),
}

impl<N> FromKlvm<N> for Proof {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        LineageProof::from_klvm(decoder, decoder.clone_node(&node))
            .map(Self::Lineage)
            .or_else(|_| EveProof::from_klvm(decoder, node).map(Self::Eve))
    }
}

impl<N> ToKlvm<N> for Proof {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        match self {
            Self::Lineage(lineage_proof) => lineage_proof.to_klvm(encoder),
            Self::Eve(eve_proof) => eve_proof.to_klvm(encoder),
        }
    }
}

impl<'a> Arbitrary<'a> for Proof {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let is_eve = u.ratio(3, 10)?;
        if is_eve {
            Ok(Self::Eve(EveProof {
                parent_coin_info: u.arbitrary()?,
                amount: u.arbitrary()?,
            }))
        } else {
            Ok(Self::Lineage(LineageProof {
                parent_coin_info: u.arbitrary()?,
                inner_puzzle_hash: u.arbitrary()?,
                amount: u.arbitrary()?,
            }))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list)]
pub struct LineageProof {
    pub parent_coin_info: [u8; 32],
    pub inner_puzzle_hash: [u8; 32],
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list)]
pub struct EveProof {
    pub parent_coin_info: [u8; 32],
    pub amount: u64,
}
