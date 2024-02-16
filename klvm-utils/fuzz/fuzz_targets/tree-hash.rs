#![no_main]
use libfuzzer_sys::fuzz_target;

use fuzzing_utils::{make_tree, BitCursor};
use klvm_utils::tree_hash;
use klvmr::allocator::Allocator;

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let input = make_tree(&mut a, &mut BitCursor::new(data), false);
    let _ret = tree_hash(&a, input);
});
