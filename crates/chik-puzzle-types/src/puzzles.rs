pub mod cat;
pub mod did;
pub mod nft;
pub mod offer;
pub mod singleton;
pub mod standard;

#[cfg(test)]
#[macro_export]
macro_rules! assert_puzzle_hash {
    ($puzzle:ident => $puzzle_hash:ident) => {
        let mut a = clvkr::Allocator::new();
        let ptr = clvkr::serde::node_from_bytes(&mut a, &$puzzle).unwrap();
        let hash = clvk_utils::tree_hash(&mut a, ptr);
        assert_eq!($puzzle_hash, hash);
    };
}
