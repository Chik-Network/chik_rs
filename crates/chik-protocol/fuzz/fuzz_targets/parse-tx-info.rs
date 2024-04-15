#![no_main]
use chik_protocol::TransactionsInfo;
use chik_traits::Streamable;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = TransactionsInfo::from_bytes(data);
});
