#![no_main]
use chik_consensus::conditions::{
    process_single_spend, MempoolVisitor, ParseState, SpendBundleConditions,
};
use chik_consensus::consensus_constants::TEST_CONSTANTS;
use chik_consensus::flags::{NO_UNKNOWN_CONDS, STRICT_ARGS_COUNT};
use chik_fuzz::{make_tree, BitCursor};
use klvmr::allocator::Allocator;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let mut ret = SpendBundleConditions::default();
    let mut state = ParseState::default();

    let parent_id = a.new_atom(&[0_u8; 32]).unwrap();
    let puzzle_hash = a.new_atom(&[0_u8; 32]).unwrap();
    let amount = a.new_atom(&[100_u8]).unwrap();

    let conds = make_tree(&mut a, &mut BitCursor::new(data), false);

    for flags in &[0, STRICT_ARGS_COUNT, NO_UNKNOWN_CONDS] {
        let mut cost_left = 11_000_000;
        let _ = process_single_spend::<MempoolVisitor>(
            &a,
            &mut ret,
            &mut state,
            parent_id,
            puzzle_hash,
            amount,
            conds,
            *flags,
            &mut cost_left,
            &TEST_CONSTANTS,
        );
    }
});
