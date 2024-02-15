use chik_protocol::Bytes32;
use klvm_traits::{FromKlvm, ToKlvm};

#[derive(Debug, Clone, PartialEq, Eq, ToKlvm, FromKlvm)]
#[cfg_attr(fuzzing, derive(arbitrary::Arbitrary))]
#[klvm(untagged, tuple)]
pub enum Proof {
    Lineage(LineageProof),
    Eve(EveProof),
}

#[derive(Debug, Clone, PartialEq, Eq, ToKlvm, FromKlvm)]
#[cfg_attr(fuzzing, derive(arbitrary::Arbitrary))]
#[klvm(list)]
pub struct LineageProof {
    pub parent_coin_info: Bytes32,
    pub inner_puzzle_hash: Bytes32,
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, ToKlvm, FromKlvm)]
#[cfg_attr(fuzzing, derive(arbitrary::Arbitrary))]
#[klvm(list)]
pub struct EveProof {
    pub parent_coin_info: Bytes32,
    pub amount: u64,
}
