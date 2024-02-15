#![no_main]
use chik::gen::conditions::MempoolVisitor;
use chik::gen::flags::ALLOW_BACKREFS;
use chik::gen::run_puzzle::run_puzzle;
use chik_protocol::CoinSpend;
use chik_traits::streamable::Streamable;
use klvmr::Allocator;
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();

    let Ok(spend) = CoinSpend::parse::<false>(&mut Cursor::new(data)) else {
        return;
    };
    let _ = run_puzzle::<MempoolVisitor>(
        &mut a,
        spend.puzzle_reveal.as_slice(),
        spend.solution.as_slice(),
        (&spend.coin.parent_coin_info).into(),
        spend.coin.amount,
        11000000000,
        ALLOW_BACKREFS,
    );
});
