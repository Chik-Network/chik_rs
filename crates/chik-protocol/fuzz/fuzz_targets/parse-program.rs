#![no_main]
use chik_protocol::Program;
use chik_traits::Streamable;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = Program::from_bytes(data);
});
