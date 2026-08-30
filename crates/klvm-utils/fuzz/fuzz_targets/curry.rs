#![no_main]
use klvm_traits::{FromKlvm, ToKlvm};
use libfuzzer_sys::fuzz_target;

use klvm_fuzzing::ArbitraryKlvmTree;
use klvm_utils::CurriedProgram;
use klvmr::allocator::NodePtr;

fuzz_target!(|input: ArbitraryKlvmTree| {
    let mut a = input.allocator;
    if let Ok(curry) = CurriedProgram::<NodePtr, NodePtr>::from_klvm(&a, input.tree) {
        curry.to_klvm(&mut a).unwrap();
    }
});
