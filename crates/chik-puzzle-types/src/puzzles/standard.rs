use chik_bls::PublicKey;
use chik_puzzles::P2_DELEGATED_PUZZLE_OR_HIDDEN_PUZZLE_HASH;
use hex_literal::hex;
use klvm_traits::{klvm_quote, FromKlvm, ToKlvm};
use klvm_utils::{CurriedProgram, ToTreeHash, TreeHash};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToKlvm, FromKlvm)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[klvm(curry)]
pub struct StandardArgs {
    pub synthetic_key: PublicKey,
}

impl StandardArgs {
    pub fn new(synthetic_key: PublicKey) -> Self {
        Self { synthetic_key }
    }

    pub fn curry_tree_hash(synthetic_key: PublicKey) -> TreeHash {
        CurriedProgram {
            program: TreeHash::new(P2_DELEGATED_PUZZLE_OR_HIDDEN_PUZZLE_HASH),
            args: StandardArgs::new(synthetic_key),
        }
        .tree_hash()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToKlvm, FromKlvm)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[klvm(list)]
pub struct StandardSolution<P, S> {
    pub original_public_key: Option<PublicKey>,
    pub delegated_puzzle: P,
    pub solution: S,
}

impl<T> StandardSolution<(u8, T), ()> {
    /// Outputs the provided condition list directly, without using the hidden puzzle.
    pub fn from_conditions(conditions: T) -> Self {
        Self {
            original_public_key: None,
            delegated_puzzle: klvm_quote!(conditions),
            solution: (),
        }
    }
}

/// This is the puzzle reveal of the [default hidden puzzle](https://chiklisp.com/standard-transactions#default-hidden-puzzle).
pub const DEFAULT_HIDDEN_PUZZLE: [u8; 3] = hex!("ff0980");

/// This is the puzzle hash of the [default hidden puzzle](https://chiklisp.com/standard-transactions#default-hidden-puzzle).
pub const DEFAULT_HIDDEN_PUZZLE_HASH: [u8; 32] = hex!(
    "
    711d6c4e32c92e53179b199484cf8c897542bc57f2b22582799f9d657eec4699
    "
);

#[cfg(test)]
mod tests {
    use chik_puzzles::P2_DELEGATED_PUZZLE_OR_HIDDEN_PUZZLE;
    use klvm_traits::ToKlvm;
    use klvm_utils::tree_hash;
    use klvmr::{serde::node_from_bytes, Allocator};

    use super::*;

    #[test]
    fn curry_tree_hash() {
        let synthetic_key = PublicKey::default();

        let mut a = Allocator::new();
        let mod_ptr = node_from_bytes(&mut a, &P2_DELEGATED_PUZZLE_OR_HIDDEN_PUZZLE).unwrap();

        let curried_ptr = CurriedProgram {
            program: mod_ptr,
            args: StandardArgs::new(synthetic_key),
        }
        .to_klvm(&mut a)
        .unwrap();

        let allocated_tree_hash = hex::encode(tree_hash(&a, curried_ptr));

        let tree_hash = hex::encode(StandardArgs::curry_tree_hash(synthetic_key));

        assert_eq!(allocated_tree_hash, tree_hash);
    }
}
