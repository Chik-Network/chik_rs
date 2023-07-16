#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;
use chik_protocol::TransactionsInfo;
use chik_protocol::Streamable;

fuzz_target!(|data: &[u8]| {
    let _ret = <TransactionsInfo as Streamable>::parse(&mut Cursor::<&[u8]>::new(data));
});
