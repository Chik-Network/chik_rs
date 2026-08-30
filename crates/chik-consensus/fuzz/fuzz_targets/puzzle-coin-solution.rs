#![no_main]
use libfuzzer_sys::fuzz_target;

use chik_consensus::get_puzzle_and_solution::get_puzzle_and_solution_for_coin;
use chik_protocol::Coin;
use klvm_fuzzing::ArbitraryKlvmTree;

const HASH: [u8; 32] = [0_u8; 32];

fuzz_target!(|input: ArbitraryKlvmTree| {
    let _ret = get_puzzle_and_solution_for_coin(
        &input.allocator,
        input.tree,
        &Coin::new(HASH.into(), HASH.into(), 1337),
    );
});
