use chik_protocol::Bytes32;
use clvk_traits::{FromClvk, ToClvk};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToClvk, FromClvk)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[clvk(transparent)]
pub enum Proof {
    Lineage(LineageProof),
    Eve(EveProof),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToClvk, FromClvk)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[clvk(list)]
pub struct LineageProof {
    pub parent_parent_coin_info: Bytes32,
    pub parent_inner_puzzle_hash: Bytes32,
    pub parent_amount: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToClvk, FromClvk)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[clvk(list)]
pub struct EveProof {
    pub parent_parent_coin_info: Bytes32,
    pub parent_amount: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToClvk, FromClvk)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[clvk(list)]
pub struct CoinProof {
    pub parent_coin_info: Bytes32,
    pub inner_puzzle_hash: Bytes32,
    pub amount: u64,
}
