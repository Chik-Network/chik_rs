#![no_main]
use chik_protocol::FullBlock;
use chik_traits::Streamable;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = FullBlock::from_bytes(data);
});
