use clap::Parser;
use std::fs;

use chik_consensus::fast_forward::fast_forward_singleton;
use chik_protocol::Bytes32;
use chik_protocol::{Coin, CoinSpend, Program};
use chik_traits::streamable::Streamable;
use klvm_traits::{FromKlvm, ToKlvm};
use klvm_utils::tree_hash;
use klvmr::allocator::Allocator;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to CoinSpend file
    #[arg(short, long)]
    spend: String,

    /// fast-forward the CoinSpend to spend a coin whose parent's parent is this
    /// coin ID.
    #[arg(short, long)]
    new_parents_parent: String,

    /// Save resulting CoinSpend to this file
    #[arg(short, long)]
    output_file: String,
}

fn main() {
    let args = Args::parse();

    let spend_bytes = fs::read(args.spend).expect("read file");
    let spend = CoinSpend::from_bytes(&spend_bytes).expect("parse CoinSpend");

    let new_parents_parent: Bytes32 = hex::decode(args.new_parents_parent)
        .expect("invalid hex")
        .try_into()
        .unwrap();

    let mut a = Allocator::new_limited(500_000_000);
    let puzzle = spend.puzzle_reveal.to_klvm(&mut a).expect("to_klvm");
    let solution = spend.solution.to_klvm(&mut a).expect("to_klvm");
    let puzzle_hash = Bytes32::from(tree_hash(&a, puzzle));

    let new_parent_coin = Coin {
        parent_coin_info: new_parents_parent,
        puzzle_hash,
        amount: spend.coin.amount,
    };

    let new_coin = Coin {
        parent_coin_info: new_parent_coin.coin_id(),
        puzzle_hash,
        amount: spend.coin.amount,
    };

    let new_solution = fast_forward_singleton(
        &mut a,
        puzzle,
        solution,
        &spend.coin,
        &new_coin,
        &new_parent_coin,
    )
    .expect("fast-forward");

    let new_spend = CoinSpend {
        coin: new_parent_coin,
        puzzle_reveal: spend.puzzle_reveal,
        solution: Program::from_klvm(&a, new_solution).expect("new solution"),
    };
    let mut bytes = Vec::<u8>::new();
    new_spend.stream(&mut bytes).expect("stream CoinSpend");
    fs::write(args.output_file, bytes).expect("write");
}
