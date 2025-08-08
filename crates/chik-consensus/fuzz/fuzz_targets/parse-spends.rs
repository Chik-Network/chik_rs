#![no_main]
use libfuzzer_sys::fuzz_target;

use chik_bls::Signature;
use chik_consensus::conditions::{parse_spends, MempoolVisitor};
use chik_fuzz::{make_list, BitCursor};
use klvmr::{Allocator, NodePtr};

use chik_consensus::consensus_constants::TEST_CONSTANTS;
use chik_consensus::flags::{NO_UNKNOWN_CONDS, STRICT_ARGS_COUNT};

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let input = make_list(&mut a, &mut BitCursor::new(data));
    // spends is a list of spends
    let input = a.new_pair(input, NodePtr::NIL).unwrap();
    for flags in &[0, STRICT_ARGS_COUNT, NO_UNKNOWN_CONDS] {
        let _ret = parse_spends::<MempoolVisitor>(
            &a,
            input,
            33_000_000_000,
            0, // klvm_cost
            *flags,
            &Signature::default(),
            None,
            &TEST_CONSTANTS,
        );
    }
});
