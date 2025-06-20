use crate::conditions::{
    process_single_spend, validate_conditions, MempoolVisitor, ParseState, SpendBundleConditions,
};
use crate::consensus_constants::ConsensusConstants;
use crate::flags::{DONT_VALIDATE_SIGNATURE, MEMPOOL_MODE};
use crate::run_block_generator::subtract_cost;
use crate::solution_generator::calculate_generator_length;
use crate::spendbundle_validation::get_flags_for_height_and_constants;
use crate::validation_error::ErrorCode;
use crate::validation_error::ValidationErr;
use chik_bls::PublicKey;
use chik_protocol::{Bytes, SpendBundle};
use klvm_utils::tree_hash;
use klvmr::allocator::Allocator;
use klvmr::chik_dialect::ChikDialect;
use klvmr::reduction::Reduction;
use klvmr::run_program::run_program;
use klvmr::serde::node_from_bytes;

const QUOTE_BYTES: usize = 2;

pub fn get_conditions_from_spendbundle(
    a: &mut Allocator,
    spend_bundle: &SpendBundle,
    max_cost: u64,
    height: u32,
    constants: &ConsensusConstants,
) -> Result<SpendBundleConditions, ValidationErr> {
    Ok(run_spendbundle(
        a,
        spend_bundle,
        max_cost,
        height,
        DONT_VALIDATE_SIGNATURE,
        constants,
    )?
    .0)
}

// returns the conditions for the spendbundle, along with the (public key,
// message) pairs emitted by the spends (for validating the aggregate signature)
#[allow(clippy::type_complexity)]
pub fn run_spendbundle(
    a: &mut Allocator,
    spend_bundle: &SpendBundle,
    max_cost: u64,
    height: u32,
    flags: u32,
    constants: &ConsensusConstants,
) -> Result<(SpendBundleConditions, Vec<(PublicKey, Bytes)>), ValidationErr> {
    let flags = get_flags_for_height_and_constants(height, constants) | flags | MEMPOOL_MODE;

    // below is an adapted version of the code from run_block_generators::run_block_generator2()
    // it assumes no block references are passed in
    let mut cost_left = max_cost;
    let dialect = ChikDialect::new(flags);
    let mut ret = SpendBundleConditions::default();
    let mut state = ParseState::default();
    // We don't pay the size cost (nor execution cost) of being wrapped by a
    // quote (in solution_generator).
    let generator_length_without_quote =
        calculate_generator_length(&spend_bundle.coin_spends) - QUOTE_BYTES;

    let byte_cost = generator_length_without_quote as u64 * constants.cost_per_byte;
    subtract_cost(a, &mut cost_left, byte_cost)?;

    for coin_spend in &spend_bundle.coin_spends {
        // process the spend
        let puz = node_from_bytes(a, coin_spend.puzzle_reveal.as_slice())?;
        let sol = node_from_bytes(a, coin_spend.solution.as_slice())?;
        let parent = a.new_atom(coin_spend.coin.parent_coin_info.as_slice())?;
        let amount = a.new_number(coin_spend.coin.amount.into())?;
        let Reduction(klvm_cost, conditions) = run_program(a, &dialect, puz, sol, cost_left)?;

        ret.execution_cost += klvm_cost;
        subtract_cost(a, &mut cost_left, klvm_cost)?;

        let buf = tree_hash(a, puz);
        if coin_spend.coin.puzzle_hash != buf.into() {
            return Err(ValidationErr(puz, ErrorCode::WrongPuzzleHash));
        }
        let puzzle_hash = a.new_atom(&buf)?;
        process_single_spend::<MempoolVisitor>(
            a,
            &mut ret,
            &mut state,
            parent,
            puzzle_hash,
            amount,
            conditions,
            flags,
            &mut cost_left,
            constants,
        )?;
    }

    validate_conditions(a, &ret, &state, a.nil(), flags)?;

    assert!(max_cost >= cost_left);
    ret.cost = max_cost - cost_left;
    Ok((ret, state.pkm_pairs))
}

#[cfg(test)]
mod tests {
    use crate::consensus_constants::TEST_CONSTANTS;

    use super::*;
    use crate::allocator::make_allocator;
    use crate::conditions::{ELIGIBLE_FOR_DEDUP, ELIGIBLE_FOR_FF};
    use crate::run_block_generator::run_block_generator2;
    use crate::solution_generator::solution_generator;
    use chik_bls::Signature;
    use chik_protocol::CoinSpend;
    use chik_traits::Streamable;
    use klvmr::chik_dialect::LIMIT_HEAP;
    use rstest::rstest;
    use std::fs::read;

    const QUOTE_EXECUTION_COST: u64 = 20;
    const QUOTE_BYTES_COST: u64 = QUOTE_BYTES as u64 * TEST_CONSTANTS.cost_per_byte;

    #[rstest]
    #[case("3000253", 8, 2, 51_216_870)]
    #[case("1000101", 34, 15, 250_083_677)]
    fn test_get_conditions_from_spendbundle(
        #[case] filename: &str,
        #[case] spends: usize,
        #[case] additions: usize,
        #[values(0, 1, 1_000_000, 5_000_000)] height: u32,
        #[case] cost: u64,
    ) {
        let bundle = SpendBundle::from_bytes(
            &read(format!("../../test-bundles/{filename}.bundle")).expect("read file"),
        )
        .expect("parse bundle");

        let mut a = make_allocator(LIMIT_HEAP);
        let conditions =
            get_conditions_from_spendbundle(&mut a, &bundle, cost, height, &TEST_CONSTANTS)
                .expect("get_conditions_from_spendbundle");

        assert_eq!(conditions.spends.len(), spends);
        let create_coins = conditions
            .spends
            .iter()
            .fold(0, |sum, spend| sum + spend.create_coin.len());
        assert_eq!(create_coins, additions);
        assert_eq!(conditions.cost, cost);
        // Generate a block with the same spend bundle and compare its cost
        let program_spends = bundle.coin_spends.iter().map(|coin_spend| {
            (
                coin_spend.coin,
                &coin_spend.puzzle_reveal,
                &coin_spend.solution,
            )
        });
        let program = solution_generator(program_spends).expect("solution_generator failed");
        let blocks: &[&[u8]] = &[];
        let block_conds = run_block_generator2(
            &mut a,
            program.as_slice(),
            blocks,
            11_000_000_000,
            MEMPOOL_MODE | DONT_VALIDATE_SIGNATURE,
            &Signature::default(),
            None,
            &TEST_CONSTANTS,
        )
        .expect("run_block_generator2 failed");
        // The cost difference here is because get_conditions_from_spendbundle
        // does not include the overhead to make a block.
        assert_eq!(
            conditions.cost,
            block_conds.cost - QUOTE_EXECUTION_COST - QUOTE_BYTES_COST
        );

        assert_eq!(
            conditions.execution_cost,
            block_conds.execution_cost - QUOTE_EXECUTION_COST
        );
        assert_eq!(conditions.condition_cost, block_conds.condition_cost);
    }

    #[rstest]
    #[case("bb13")]
    #[case("e3c0")]
    fn test_get_conditions_from_spendbundle_fast_forward(
        #[case] filename: &str,
        #[values(0, 1, 1_000_000, 5_000_000)] height: u32,
    ) {
        let cost = 77_341_866;
        let spend = CoinSpend::from_bytes(
            &read(format!("../../ff-tests/{filename}.spend")).expect("read file"),
        )
        .expect("parse Spend");

        let bundle = SpendBundle::new(vec![spend], Signature::default());

        let mut a = make_allocator(LIMIT_HEAP);
        let conditions =
            get_conditions_from_spendbundle(&mut a, &bundle, cost, height, &TEST_CONSTANTS)
                .expect("get_conditions_from_spendbundle");

        assert_eq!(conditions.spends.len(), 1);
        let spend = &conditions.spends[0];
        assert_eq!(spend.flags, ELIGIBLE_FOR_FF | ELIGIBLE_FOR_DEDUP);
        assert_eq!(conditions.cost, cost);
    }

    // given a block generator and block-refs, convert run the generator to
    // produce the SpendBundle for the block without runningi, or validating,
    // the puzzles.
    #[cfg(not(debug_assertions))]
    fn convert_block_to_bundle(generator: &[u8], block_refs: &[Vec<u8>]) -> SpendBundle {
        use crate::run_block_generator::extract_n;
        use crate::run_block_generator::setup_generator_args;
        use crate::validation_error::ErrorCode;
        use chik_protocol::Coin;
        use klvmr::op_utils::first;
        use klvmr::serde::node_from_bytes_backrefs;
        use klvmr::serde::node_to_bytes;

        let mut a = make_allocator(MEMPOOL_MODE);

        let generator = node_from_bytes_backrefs(&mut a, generator).expect("node_from_bytes");
        let args = setup_generator_args(&mut a, block_refs).expect("setup_generator_args");
        let dialect = ChikDialect::new(MEMPOOL_MODE);
        let Reduction(_, mut all_spends) =
            run_program(&mut a, &dialect, generator, args, 11_000_000_000).expect("run_program");

        all_spends = first(&a, all_spends).expect("first");

        let mut spends = Vec::<CoinSpend>::new();

        // at this point all_spends is a list of:
        // (parent-coin-id puzzle-reveal amount solution . extra)
        // where extra may be nil, or additional extension data
        while let Some((spend, rest)) = a.next(all_spends) {
            all_spends = rest;
            // process the spend
            let [parent_id, puzzle, amount, solution, _spend_level_extra] =
                extract_n::<5>(&a, spend, ErrorCode::InvalidCondition).expect("extract_n");

            spends.push(CoinSpend::new(
                Coin::new(
                    a.atom(parent_id).as_ref().try_into().expect("parent_id"),
                    tree_hash(&a, puzzle).into(),
                    a.number(amount).try_into().expect("amount"),
                ),
                node_to_bytes(&a, puzzle).expect("node_to_bytes").into(),
                node_to_bytes(&a, solution).expect("node_to_bytes").into(),
            ));
        }
        SpendBundle::new(spends, Signature::default())
    }

    #[cfg(not(debug_assertions))]
    #[rstest]
    #[case("new-agg-sigs")]
    #[case("infinity-g1")]
    #[case("block-1ee588dc")]
    #[case("block-6fe59b24")]
    #[case("block-b45268ac")]
    #[case("block-c2a8df0d")]
    #[case("block-e5002df2")]
    #[case("block-4671894")]
    #[case("block-225758")]
    #[case("assert-puzzle-announce-fail")]
    #[case("block-834752")]
    #[case("block-834752-compressed")]
    #[case("block-834760")]
    #[case("block-834761")]
    #[case("block-834765")]
    #[case("block-834766")]
    #[case("block-834768")]
    #[case("create-coin-different-amounts")]
    #[case("create-coin-hint-duplicate-outputs")]
    #[case("create-coin-hint")]
    #[case("create-coin-hint2")]
    #[case("deep-recursion-plus")]
    #[case("double-spend")]
    #[case("duplicate-coin-announce")]
    #[case("duplicate-create-coin")]
    #[case("duplicate-height-absolute-div")]
    #[case("duplicate-height-absolute-substr-tail")]
    #[case("duplicate-height-absolute-substr")]
    #[case("duplicate-height-absolute")]
    #[case("duplicate-height-relative")]
    #[case("duplicate-outputs")]
    #[case("duplicate-reserve-fee")]
    #[case("duplicate-seconds-absolute")]
    #[case("duplicate-seconds-relative")]
    #[case("height-absolute-ladder")]
    //#[case("infinite-recursion1")]
    //#[case("infinite-recursion2")]
    //#[case("infinite-recursion3")]
    //#[case("infinite-recursion4")]
    #[case("invalid-conditions")]
    #[case("just-puzzle-announce")]
    #[case("many-create-coin")]
    #[case("many-large-ints-negative")]
    #[case("many-large-ints")]
    #[case("max-height")]
    #[case("multiple-reserve-fee")]
    #[case("negative-reserve-fee")]
    //#[case("recursion-pairs")]
    #[case("unknown-condition")]
    #[case("duplicate-messages")]
    fn run_generator(#[case] name: &str) {
        use crate::run_block_generator::run_block_generator;
        use crate::test_generators::{print_conditions, print_diff};
        use std::fs::read_to_string;

        let filename = format!("../../generator-tests/{name}.txt");
        println!("file: {filename}");
        let test_file = read_to_string(filename).expect("test file not found");
        let (generator, expected) = test_file.split_once('\n').expect("invalid test file");
        let generator_buffer = hex::decode(generator).expect("invalid hex encoded generator");

        let expected = match expected.split_once("STRICT:\n") {
            Some((_, m)) => m,
            None => expected,
        };

        let mut block_refs = Vec::<Vec<u8>>::new();

        let filename = format!("../../generator-tests/{name}.env");
        if let Ok(env_hex) = read_to_string(&filename) {
            println!("block-ref file: {filename}");
            block_refs.push(hex::decode(env_hex).expect("hex decode env-file"));
        }

        let bundle = convert_block_to_bundle(&generator_buffer, &block_refs);

        // run the whole block through run_block_generator() to ensure the
        // output conditions match and update the cost. The cost
        // of just the spend bundle will be lower
        let (block_cost, block_output) = {
            let mut a = make_allocator(MEMPOOL_MODE);
            let block_conds = run_block_generator(
                &mut a,
                &generator_buffer,
                &block_refs,
                11_000_000_000,
                MEMPOOL_MODE | DONT_VALIDATE_SIGNATURE,
                &Signature::default(),
                None,
                &TEST_CONSTANTS,
            );
            match block_conds {
                Ok(ref conditions) => (conditions.cost, print_conditions(&a, &conditions)),
                Err(code) => {
                    println!("error: {code:?}");
                    (0, format!("FAILED: {}\n", u32::from(code.1)))
                }
            }
        };

        let mut a = make_allocator(LIMIT_HEAP);
        let conds = get_conditions_from_spendbundle(
            &mut a,
            &bundle,
            11_000_000_000,
            5_000_000,
            &TEST_CONSTANTS,
        );

        let output = match conds {
            Ok(mut conditions) => {
                // the cost of running the spend bundle should never be higher
                // than the whole block but it's likely less.
                // but only if the byte cost is not taken into account. The
                // block will likely be smaller because the compression makes it
                // smaller.
                let block_byte_cost = generator_buffer.len() as u64 * TEST_CONSTANTS.cost_per_byte;
                let program_spends = bundle.coin_spends.iter().map(|coin_spend| {
                    (
                        coin_spend.coin,
                        &coin_spend.puzzle_reveal,
                        &coin_spend.solution,
                    )
                });
                let generator_length_without_quote = solution_generator(program_spends)
                    .expect("solution_generator failed")
                    .len()
                    - QUOTE_BYTES;
                let bundle_byte_cost =
                    generator_length_without_quote as u64 * TEST_CONSTANTS.cost_per_byte;
                println!(" block_cost: {block_cost} bytes: {block_byte_cost}");
                println!("bundle_cost: {} bytes: {bundle_byte_cost}", conditions.cost);
                println!("execution_cost: {}", conditions.execution_cost);
                println!("condition_cost: {}", conditions.condition_cost);
                assert!(conditions.cost - bundle_byte_cost <= block_cost - block_byte_cost);
                assert!(conditions.cost > 0);
                assert!(conditions.execution_cost > 0);
                assert_eq!(
                    conditions.cost,
                    conditions.condition_cost + conditions.execution_cost + bundle_byte_cost
                );
                // update the cost we print here, just to be compatible with
                // the test cases we have. We've already ensured the cost is
                // lower
                conditions.cost = block_cost;
                print_conditions(&a, &conditions)
            }
            Err(code) => {
                println!("error: {code:?}");
                format!("FAILED: {}\n", u32::from(code.1))
            }
        };

        if output != block_output {
            print_diff(&output, &block_output);
            panic!("run_block_generator produced a different result than get_conditions_from_spendbundle()");
        }

        if output != expected {
            print_diff(&output, expected);
            panic!("mismatching condition output");
        }
    }
}
