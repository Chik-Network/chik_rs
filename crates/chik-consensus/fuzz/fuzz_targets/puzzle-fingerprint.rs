#![no_main]
use libfuzzer_sys::fuzz_target;

use chik_consensus::puzzle_fingerprint::compute_puzzle_fingerprint;
use klvmr::Allocator;
use klvmr::serde::node_from_bytes;

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let Ok(conditions) = node_from_bytes(&mut a, data) else {
        return;
    };

    let _ = compute_puzzle_fingerprint(&a, conditions);
});
