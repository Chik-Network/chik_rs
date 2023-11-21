#![no_main]
use chik_protocol::Foliage;
use chik_traits::Streamable;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = Foliage::from_bytes(data);
});
