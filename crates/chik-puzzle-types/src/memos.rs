use clvk_traits::{FromClvk, ToClvk};
use clvkr::NodePtr;

/// The purpose of this type is to be an optional field at the end of a create coin condition
/// or payment in the notarized payment list. It can either be nil (no memos specified) or an
/// extra field that is typically a list of memos (although can technically be any structure).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ToClvk, FromClvk)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[clvk(untagged, list)]
pub enum Memos<T = NodePtr> {
    /// An arbitrary CLVK structure that represents the memos
    Some(T),
    /// No memos specified
    #[default]
    None,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use clvkr::Allocator;
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_memos_roundtrip(
        #[values(Memos::None, Memos::Some(0), Memos::Some(100))] expected: Memos<u64>,
    ) -> Result<()> {
        let mut allocator = Allocator::new();

        let ptr = expected.to_clvk(&mut allocator)?;
        let memos = Memos::<u64>::from_clvk(&allocator, ptr)?;

        assert_eq!(memos, expected);

        Ok(())
    }
}
