#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;
use chik_protocol::FullBlock;
use chik_protocol::Streamable;

fuzz_target!(|data: &[u8]| {
    let _ret = <FullBlock as Streamable>::parse(&mut Cursor::<&[u8]>::new(data));
});
