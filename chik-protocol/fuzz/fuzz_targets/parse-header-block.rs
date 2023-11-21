#![no_main]
use chik_protocol::HeaderBlock;
use chik_traits::Streamable;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = HeaderBlock::from_bytes(data);
});
