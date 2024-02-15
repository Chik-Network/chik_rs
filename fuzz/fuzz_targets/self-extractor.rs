#![no_main]

use chik::compression::compressor::wrap_atom_with_decompression_program;

use klvm_utils::tree_hash;
use klvmr::allocator::Allocator;
use klvmr::chik_dialect::ChikDialect;
use klvmr::run_program::run_program;
use klvmr::serde::node_to_bytes_backrefs;
use libfuzzer_sys::fuzz_target;

fn do_fuzz(data: &[u8], short_atoms: bool) {
    let mut allocator = Allocator::new();
    let mut cursor = fuzzing_utils::BitCursor::new(data);

    let program = fuzzing_utils::make_tree(&mut allocator, &mut cursor, short_atoms);

    let original_hash = tree_hash(&allocator, program);

    let serialized_program_bytes = node_to_bytes_backrefs(&allocator, program).unwrap();
    let serialized_program_atom = allocator.new_atom(&serialized_program_bytes).unwrap();

    let self_extracting_program =
        wrap_atom_with_decompression_program(&mut allocator, serialized_program_atom).unwrap();

    let dialect = &ChikDialect::new(0);

    let args = allocator.nil();

    let max_cost = u64::MAX;

    let decompressed_program = run_program(
        &mut allocator,
        dialect,
        self_extracting_program,
        args,
        max_cost,
    )
    .unwrap()
    .1;

    let new_hash = tree_hash(&allocator, decompressed_program);

    if original_hash != new_hash {
        panic!("hashes do not match");
    }
}

fuzz_target!(|data: &[u8]| {
    do_fuzz(data, true);
    do_fuzz(data, false);
});
