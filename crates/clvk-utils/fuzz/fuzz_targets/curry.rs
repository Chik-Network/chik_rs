#![no_main]
use clvk_traits::{FromClvk, ToClvk};
use libfuzzer_sys::fuzz_target;

use clvk_fuzzing::ArbitraryClvkTree;
use clvk_utils::CurriedProgram;
use clvkr::allocator::NodePtr;

fuzz_target!(|input: ArbitraryClvkTree| {
    let mut a = input.allocator;
    if let Ok(curry) = CurriedProgram::<NodePtr, NodePtr>::from_clvk(&a, input.tree) {
        curry.to_clvk(&mut a).unwrap();
    }
});
