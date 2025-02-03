#![allow(clippy::items_after_statements)]

use klvm_traits::{KlvmEncoder, ToKlvm, ToKlvmError};
use klvmr::Atom;

use crate::{tree_hash_atom, tree_hash_pair, TreeHash};

pub trait ToTreeHash {
    fn tree_hash(&self) -> TreeHash;
}

impl<T> ToTreeHash for T
where
    T: ToKlvm<TreeHasher>,
{
    fn tree_hash(&self) -> TreeHash {
        self.to_klvm(&mut TreeHasher).unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TreeHasher;

impl KlvmEncoder for TreeHasher {
    type Node = TreeHash;

    fn encode_atom(&mut self, bytes: Atom<'_>) -> Result<Self::Node, ToKlvmError> {
        Ok(tree_hash_atom(bytes.as_ref()))
    }

    fn encode_pair(
        &mut self,
        first: Self::Node,
        rest: Self::Node,
    ) -> Result<Self::Node, ToKlvmError> {
        Ok(tree_hash_pair(first, rest))
    }
}

impl ToKlvm<TreeHasher> for TreeHash {
    fn to_klvm(&self, _encoder: &mut TreeHasher) -> Result<TreeHash, ToKlvmError> {
        Ok(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use klvm_traits::ToKlvm;

    use crate::{curry_tree_hash, CurriedProgram};

    #[test]
    fn test_tree_hash() {
        assert_eq!(
            hex::encode(().tree_hash()),
            "4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a"
        );
        assert_eq!(
            hex::encode([1, 2, 3].tree_hash()),
            "bcd55bcd0daebba8cb158547e8480dc968570faf958f1e31a9887d6ae3dba591"
        );
        assert_eq!(
            hex::encode("hello".tree_hash()),
            "cceeb7a985ecc3dabcb4c8f666cd637f16f008e3c963db6aa6f83a7b288c54ef"
        );
        assert_eq!(
            hex::encode(((1, 2), (3, 4)).tree_hash()),
            "2824018d148bc6aed0847e2c86aaa8a5407b916169f15b12cea31fa932fc4c8d"
        );

        // This is the default hidden puzzle for the standard transaction.
        // Its tree hash is known, and its KLVM is `(=)`.
        assert_eq!(
            hex::encode([9].tree_hash()),
            "711d6c4e32c92e53179b199484cf8c897542bc57f2b22582799f9d657eec4699"
        );
    }

    #[test]
    fn test_curry_tree_hash() {
        let hash_1 = [1, 2, 3].tree_hash();
        let hash_2 = [4, 5, 6].tree_hash();
        let hash_3 = [7, 8, 9].tree_hash();

        let manual = curry_tree_hash(hash_1, &[hash_2, hash_3]);

        #[derive(ToKlvm)]
        #[klvm(curry)]
        struct Args<T> {
            a: T,
            b: T,
        }

        let hash = CurriedProgram {
            program: hash_1,
            args: Args {
                a: hash_2,
                b: hash_3,
            },
        }
        .tree_hash();

        assert_eq!(hash, manual);
    }
}
