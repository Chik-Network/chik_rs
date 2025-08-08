#![no_main]
use libfuzzer_sys::{arbitrary, fuzz_target};

use chik_consensus::get_puzzle_and_solution::parse_coin_spend;
use chik_fuzzing::make_list;
use klvmr::allocator::Allocator;

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let mut unstructured = arbitrary::Unstructured::new(data);
    let input = make_list(&mut a, &mut unstructured);

    let _ret = parse_coin_spend(&a, input);
});
