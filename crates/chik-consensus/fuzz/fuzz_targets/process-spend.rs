#![no_main]
use chik_consensus::conditions::{
    MempoolVisitor, ParseState, SpendBundleConditions, process_single_spend,
};
use chik_consensus::consensus_constants::TEST_CONSTANTS;
use chik_consensus::flags::ConsensusFlags;
use klvm_fuzzing::ArbitraryKlvmTree;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|args: (ArbitraryKlvmTree, [u8; 32], [u8; 32], u64)| {
    let (conds, parent_id, puzzle_hash, amount) = args;
    let mut a = conds.allocator;
    let mut ret = SpendBundleConditions::default();
    let mut state = ParseState::default();

    let parent_id = a.new_atom(&parent_id).expect("new_atom");
    let puzzle_hash = a.new_atom(&puzzle_hash).expect("new_atom");
    let amount = a.new_number(amount.into()).expect("new_atom");

    for flags in &[
        ConsensusFlags::empty(),
        ConsensusFlags::STRICT_ARGS_COUNT,
        ConsensusFlags::NO_UNKNOWN_CONDS,
    ] {
        let mut cost_left = 110_000_000;
        let _ = process_single_spend::<MempoolVisitor>(
            &a,
            &mut ret,
            &mut state,
            parent_id,
            puzzle_hash,
            amount,
            conds.tree,
            *flags,
            &mut cost_left,
            0, // klvm_cost
            &TEST_CONSTANTS,
        );
    }
});
