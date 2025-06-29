#![no_main]
use libfuzzer_sys::fuzz_target;

use chik_consensus::get_puzzle_and_solution::get_puzzle_and_solution_for_coin;
use chik_fuzz::{make_tree, BitCursor};
use chik_protocol::Coin;
use klvmr::allocator::Allocator;
use std::collections::HashSet;

const HASH: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let input = make_tree(&mut a, &mut BitCursor::new(data), false);

    let _ret = get_puzzle_and_solution_for_coin(
        &a,
        input,
        &HashSet::new(),
        &Coin::new(HASH.into(), HASH.into(), 1337),
    );
});
