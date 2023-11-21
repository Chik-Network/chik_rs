use arbitrary::{Arbitrary, Unstructured};
use klvm_traits::{FromKlvm, Result, ToKlvm};
use klvmr::{allocator::NodePtr, Allocator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proof {
    Lineage(LineageProof),
    Eve(EveProof),
}

impl FromKlvm for Proof {
    fn from_klvm(a: &Allocator, node: NodePtr) -> Result<Self> {
        LineageProof::from_klvm(a, node)
            .map(Self::Lineage)
            .or_else(|_| EveProof::from_klvm(a, node).map(Self::Eve))
    }
}

impl ToKlvm for Proof {
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        match self {
            Self::Lineage(lineage_proof) => lineage_proof.to_klvm(a),
            Self::Eve(eve_proof) => eve_proof.to_klvm(a),
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
