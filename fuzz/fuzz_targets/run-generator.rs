#![no_main]
use chik::gen::flags::ALLOW_BACKREFS;
use chik::gen::run_block_generator::{run_block_generator, run_block_generator2};
use klvmr::allocator::Allocator;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut a = Allocator::new();
    let _ = run_block_generator::<&[u8]>(&mut a, data, &[], 11000000000, ALLOW_BACKREFS);
    let _ = run_block_generator2::<&[u8]>(&mut a, data, &[], 11000000000, ALLOW_BACKREFS);
});
