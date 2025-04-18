#![no_main]

use chik_protocol::Coin;
use chik_protocol::{Bytes32, SpendBundle};
use chik_traits::Streamable;
use klvm_traits::FromKlvm;
use klvmr::op_utils::{first, rest};
use klvmr::{Allocator, NodePtr};
use libfuzzer_sys::fuzz_target;
use std::collections::HashSet;

fuzz_target!(|data: &[u8]| {
    let Ok(bundle) = SpendBundle::from_bytes(data) else {
        return;
    };
    let Ok(additions) = bundle.additions() else {
        return;
    };

    let additions = additions.iter().copied().collect::<HashSet<_>>();

    let mut expected = HashSet::new();

    let mut a = Allocator::new();
    let mut total_cost = 0;
    for cs in &bundle.coin_spends {
        let (cost, mut conds) = cs
            .puzzle_reveal
            .run(&mut a, 0, 11_000_000_000, &cs.solution)
            .expect("run");
        total_cost += cost;

        let parent_coin_info = cs.coin.coin_id();

        while let Some((c, tail)) = a.next(conds) {
            conds = tail;
            let op = first(&a, c).expect("first");
            let c = rest(&a, c).expect("rest");
            let buf = a.atom(op);
            if buf.as_ref().len() != 1 {
                continue;
            }
            if buf.as_ref()[0] == 51 {
                let (puzzle_hash, (amount, _)) =
                    <(Bytes32, (u64, NodePtr))>::from_klvm(&a, c).expect("parse spend");
                expected.insert(Coin {
                    parent_coin_info,
                    puzzle_hash,
                    amount,
                });
                total_cost += 1_800_000;
            }
        }
    }

    assert!(total_cost <= 11_000_000_000);

    assert_eq!(additions, expected);
});
