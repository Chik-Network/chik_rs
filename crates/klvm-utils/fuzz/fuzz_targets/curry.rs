#![no_main]
use klvm_traits::{FromKlvm, ToKlvm};
use libfuzzer_sys::{arbitrary, fuzz_target};

use chik_fuzzing::make_tree;
use klvm_utils::CurriedProgram;
use klvmr::allocator::{Allocator, NodePtr};

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let mut unstructured = arbitrary::Unstructured::new(data);
    let (input, _) = make_tree(&mut a, &mut unstructured);
    if let Ok(curry) = CurriedProgram::<NodePtr, NodePtr>::from_klvm(&a, input) {
        curry.to_klvm(&mut a).unwrap();
    }
});
