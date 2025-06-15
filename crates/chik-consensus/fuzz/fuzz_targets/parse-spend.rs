#![no_main]
use libfuzzer_sys::fuzz_target;

use chik_consensus::get_puzzle_and_solution::parse_coin_spend;
use chik_fuzz::{make_list, BitCursor};
use klvmr::allocator::Allocator;

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let input = make_list(&mut a, &mut BitCursor::new(data));

    let _ret = parse_coin_spend(&a, input);
});
