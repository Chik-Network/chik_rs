#![no_main]
use klvm_traits::{FromKlvm, ToKlvm};
use libfuzzer_sys::fuzz_target;

use chik_fuzz::{make_tree, BitCursor};
use klvm_utils::CurriedProgram;
use klvmr::allocator::{Allocator, NodePtr};

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let input = make_tree(&mut a, &mut BitCursor::new(data), true);
    if let Ok(curry) = CurriedProgram::<NodePtr, NodePtr>::from_klvm(&a, input) {
        curry.to_klvm(&mut a).unwrap();
    }
});
