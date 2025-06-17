use chik_bls::PublicKey;
use chik_protocol::Bytes32;
use chik_puzzles::DID_INNERPUZ_HASH;
use klvm_traits::{FromKlvm, ToKlvm};
use klvm_utils::{CurriedProgram, ToTreeHash, TreeHash};

use crate::{singleton::SingletonStruct, CoinProof};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToKlvm, FromKlvm)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[klvm(curry)]
pub struct DidArgs<I, M> {
    pub inner_puzzle: I,
    pub recovery_list_hash: Option<Bytes32>,
    pub num_verifications_required: u64,
    pub singleton_struct: SingletonStruct,
    pub metadata: M,
}

impl<I, M> DidArgs<I, M> {
    pub fn new(
        inner_puzzle: I,
        recovery_list_hash: Option<Bytes32>,
        num_verifications_required: u64,
        singleton_struct: SingletonStruct,
        metadata: M,
    ) -> Self {
        Self {
            inner_puzzle,
            recovery_list_hash,
            num_verifications_required,
            singleton_struct,
            metadata,
        }
    }
}

impl DidArgs<TreeHash, TreeHash> {
    pub fn curry_tree_hash(
        inner_puzzle: TreeHash,
        recovery_list_hash: Option<Bytes32>,
        num_verifications_required: u64,
        singleton_struct: SingletonStruct,
        metadata: TreeHash,
    ) -> TreeHash {
        CurriedProgram {
            program: TreeHash::new(DID_INNERPUZ_HASH),
            args: DidArgs {
                inner_puzzle,
                recovery_list_hash,
                num_verifications_required,
                singleton_struct,
                metadata,
            },
        }
        .tree_hash()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ToKlvm, FromKlvm)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[klvm(list)]
#[repr(u8)]
pub enum DidSolution<I> {
    Recover(#[klvm(rest)] Box<DidRecoverySolution>) = 0,
    Spend(I) = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, ToKlvm, FromKlvm)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[klvm(list)]
pub struct DidRecoverySolution {
    pub amount: u64,
    pub new_inner_puzzle_hash: Bytes32,
    pub recovery_coins: Vec<CoinProof>,
    pub public_key: PublicKey,
    pub recovery_list_reveal: Vec<Bytes32>,
}

#[cfg(test)]
mod tests {
    use chik_puzzles::DID_INNERPUZ;
    use klvm_traits::{klvm_list, match_list};
    use klvmr::{
        run_program,
        serde::{node_from_bytes, node_to_bytes},
        Allocator, ChikDialect,
    };

    use super::*;

    #[test]
    fn did_solution() {
        let a = &mut Allocator::new();

        let ptr = klvm_list!(1, klvm_list!(42, "test")).to_klvm(a).unwrap();
        let did_solution = DidSolution::<match_list!(i32, String)>::from_klvm(a, ptr).unwrap();
        assert_eq!(
            did_solution,
            DidSolution::Spend(klvm_list!(42, "test".to_string()))
        );

        let puzzle = node_from_bytes(a, &DID_INNERPUZ).unwrap();
        let curried = CurriedProgram {
            program: puzzle,
            args: DidArgs::new(1, None, 1, SingletonStruct::new(Bytes32::default()), ()),
        }
        .to_klvm(a)
        .unwrap();

        let output = run_program(a, &ChikDialect::new(0), curried, ptr, u64::MAX)
            .expect("could not run did puzzle and solution");
        assert_eq!(
            hex::encode(node_to_bytes(a, output.1).unwrap()),
            "ff2aff847465737480"
        );
    }

    #[test]
    fn did_solution_roundtrip() {
        let a = &mut Allocator::new();
        let did_solution = DidSolution::Spend(a.nil());
        let ptr = did_solution.to_klvm(a).unwrap();
        let roundtrip = DidSolution::from_klvm(a, ptr).unwrap();
        assert_eq!(did_solution, roundtrip);
    }
}
