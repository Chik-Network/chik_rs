#![no_main]
use klvm_traits::{FromKlvm, ToKlvm};
use libfuzzer_sys::fuzz_target;

use klvm_utils::CurriedProgram;
use klvmr::allocator::{Allocator, NodePtr};
use fuzzing_utils::{make_tree, BitCursor};

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let input = make_tree(&mut a, &mut BitCursor::new(data), true);
    if let Ok(curry) = CurriedProgram::<NodePtr, NodePtr>::from_klvm(&a, input) {
        curry.to_klvm(&mut a).unwrap();
    }
});
