#!/usr/bin/env python3

from chik_rs import (
    run_block_generator,
    SpendBundleConditions,
    run_block_generator2,
    ConsensusConstants,
    DONT_VALIDATE_SIGNATURE,
    G2Element,
)
from chik_rs.sized_bytes import bytes32
from chik_rs.sized_ints import uint8, uint16, uint32, uint64, uint128
from time import time
import sys
from time import perf_counter
from typing import Optional

DEFAULT_CONSTANTS = ConsensusConstants(
    SLOT_BLOCKS_TARGET=uint32(32),
    MIN_BLOCKS_PER_CHALLENGE_BLOCK=uint8(16),
    MAX_SUB_SLOT_BLOCKS=uint32(128),
    NUM_SPS_SUB_SLOT=uint32(64),
    SUB_SLOT_ITERS_STARTING=uint64(2**27),
    DIFFICULTY_CONSTANT_FACTOR=uint128(2**57),
    DIFFICULTY_STARTING=uint64(7),
    DIFFICULTY_CHANGE_MAX_FACTOR=uint32(3),
    SUB_EPOCH_BLOCKS=uint32(384),
    EPOCH_BLOCKS=uint32(4608),
    SIGNIFICANT_BITS=uint8(8),
    DISCRIMINANT_SIZE_BITS=uint16(1024),
    NUMBER_ZERO_BITS_PLOT_FILTER=uint8(9),
    MIN_PLOT_SIZE=uint8(32),
    MAX_PLOT_SIZE=uint8(50),
    SUB_SLOT_TIME_TARGET=uint16(600),
    NUM_SP_INTERVALS_EXTRA=uint8(3),
    MAX_FUTURE_TIME2=uint32(2 * 60),
    NUMBER_OF_TIMESTAMPS=uint8(11),
    GENESIS_CHALLENGE=bytes32.fromhex(
        "4b8f8f2f852c0a89a0f97a1bc91f1806e1f2efd4924fc3a82bec9a7b31b61f31"
    ),
    AGG_SIG_ME_ADDITIONAL_DATA=bytes32.fromhex(
        "6952ce05c863008c10b211baab87ee58e11c52fda1b9a13d0190d48d6b18354b"
    ),
    AGG_SIG_PARENT_ADDITIONAL_DATA=bytes32.fromhex(
        "b40b4987855d2e1f58dfebd7c051a7b26dbc3271b4d2c373dfe269405d3acb28"
    ),
    AGG_SIG_PUZZLE_ADDITIONAL_DATA=bytes32.fromhex(
        "e5292be0bccf780890ac29dba649ddf0eb51d1e6abcf176420d52edbed93a63a"
    ),
    AGG_SIG_AMOUNT_ADDITIONAL_DATA=bytes32.fromhex(
        "0c7bfa4d7810561c3a63f0b9a101318572f8387c868379506fe447fff21ec5ce"
    ),
    AGG_SIG_PUZZLE_AMOUNT_ADDITIONAL_DATA=bytes32.fromhex(
        "723223694461c96764541c6a435768ad956f5e9313bac800dab46dc24b300ac7"
    ),
    AGG_SIG_PARENT_AMOUNT_ADDITIONAL_DATA=bytes32.fromhex(
        "e5d5ae41ef3027e34fe5ca74e9a5a409d95157d60eb806e0954189a5d791ee03"
    ),
    AGG_SIG_PARENT_PUZZLE_ADDITIONAL_DATA=bytes32.fromhex(
        "cf0f5f05a4cfb22ad7af51aa0de936af79375d41a62a89ef417290cc44a6a781"
    ),
    GENESIS_PRE_FARM_POOL_PUZZLE_HASH=bytes32.fromhex(
        "09b2395c02bf08906a78e3bd10f4849e182c2b05086419ebb90ac94bcd9b0094"
    ),
    GENESIS_PRE_FARM_FARMER_PUZZLE_HASH=bytes32.fromhex(
        "68e9833c8ea4fe2f222bf36ea6ff2236ccc209eda50b56ed84091d75d3f3c4d5"
    ),
    MAX_VDF_WITNESS_SIZE=uint8(64),
    MEMPOOL_BLOCK_BUFFER=uint8(10),
    MAX_COIN_AMOUNT=uint64((1 << 64) - 1),
    MAX_BLOCK_COST_KLVM=uint64(11000000000),
    COST_PER_BYTE=uint64(12000),
    WEIGHT_PROOF_THRESHOLD=uint8(2),
    BLOCKS_CACHE_SIZE=uint32(4608 + (128 * 4)),
    WEIGHT_PROOF_RECENT_BLOCKS=uint32(1000),
    MAX_BLOCK_COUNT_PER_REQUESTS=uint32(32),
    MAX_GENERATOR_SIZE=uint32(1000000),
    MAX_GENERATOR_REF_LIST_SIZE=uint32(512),
    POOL_SUB_SLOT_ITERS=uint64(37600000000),
    SOFT_FORK6_HEIGHT=uint32(0),
    HARD_FORK_HEIGHT=uint32(5496000),
    PLOT_FILTER_128_HEIGHT=uint32(10542000),
    PLOT_FILTER_64_HEIGHT=uint32(15592000),
    PLOT_FILTER_32_HEIGHT=uint32(20643000),
)


def run_gen(
    fn: str, flags: int = 0, args: Optional[str] = None, version: int = 1
) -> tuple[Optional[int], Optional[SpendBundleConditions], float]:

    # constants from the main chik blockchain:
    # https://github.com/Chik-Network/chik-blockchain/blob/main/chik/consensus/default_constants.py
    max_cost = 11000000000
    cost_per_byte = 12000

    generator = bytes.fromhex(open(fn, "r").read().split("\n")[0])

    # add the block program arguments
    block_refs = []
    if args and args != "":
        try:
            with open(args, "r") as f:
                block_refs = [bytes.fromhex(f.read())]
        except OSError as e:
            pass

    block_runner = run_block_generator if version == 1 else run_block_generator2

    start_time = perf_counter()
    try:
        ret = block_runner(
            generator,
            block_refs,
            max_cost,
            flags | DONT_VALIDATE_SIGNATURE,
            G2Element(),
            None,
            DEFAULT_CONSTANTS,
        )
        run_time = perf_counter() - start_time
        return ret + (run_time,)
    except Exception as e:
        # GENERATOR_RUNTIME_ERROR
        run_time = perf_counter() - start_time
        return (117, None, run_time)


def print_spend_bundle_conditions(result) -> str:
    ret = ""
    if result.reserve_fee > 0:
        ret += f"RESERVE_FEE: {result.reserve_fee}\n"
    if result.height_absolute > 0:
        ret += f"ASSERT_HEIGHT_ABSOLUTE {result.height_absolute}\n"
    if result.seconds_absolute > 0:
        ret += f"ASSERT_SECONDS_ABSOLUTE {result.seconds_absolute}\n"
    if result.before_seconds_absolute is not None:
        ret += f"ASSERT_BEFORE_SECONDS_ABSOLUTE {result.before_seconds_absolute}\n"
    if result.before_height_absolute is not None:
        ret += f"ASSERT_BEFORE_HEIGHT_ABSOLUTE {result.before_height_absolute}\n"
    for a in sorted(result.agg_sig_unsafe):
        ret += f"AGG_SIG_UNSAFE pk: {a[0]} msg: {a[1].hex()}\n"
    ret += "SPENDS:\n"
    for s in sorted(result.spends, key=lambda x: x.coin_id):
        ret += f"- coin id: {s.coin_id.hex()} ph: {s.puzzle_hash.hex()}\n"

        if s.height_relative is not None:
            ret += f"  ASSERT_HEIGHT_RELATIVE {s.height_relative}\n"
        if s.seconds_relative is not None:
            ret += f"  ASSERT_SECONDS_RELATIVE {s.seconds_relative}\n"
        if s.before_height_relative is not None:
            ret += f"  ASSERT_BEFORE_HEIGHT_RELATIVE {s.before_height_relative}\n"
        if s.before_seconds_relative is not None:
            ret += f"  ASSERT_BEFORE_SECONDS_RELATIVE {s.before_seconds_relative}\n"
        for a in sorted(s.create_coin):
            if a[2] is not None and len(a[2]) > 0:
                ret += f"  CREATE_COIN: ph: {a[0].hex()} amount: {a[1]} hint: {a[2].hex()}\n"
            else:
                ret += f"  CREATE_COIN: ph: {a[0].hex()} amount: {a[1]}\n"
        for a in sorted(s.agg_sig_me):
            ret += f"  AGG_SIG_ME pk: {a[0]} msg: {a[1].hex()}\n"
        for a in sorted(s.agg_sig_parent):
            ret += f"  AGG_SIG_PARENT pk: {a[0]} msg: {a[1].hex()}\n"
        for a in sorted(s.agg_sig_puzzle):
            ret += f"  AGG_SIG_PUZZLE pk: {a[0]} msg: {a[1].hex()}\n"
        for a in sorted(s.agg_sig_amount):
            ret += f"  AGG_SIG_AMOUNT pk: {a[0]} msg: {a[1].hex()}\n"
        for a in sorted(s.agg_sig_puzzle_amount):
            ret += f"  AGG_SIG_PUZZLE_AMOUNT pk: {a[0]} msg: {a[1].hex()}\n"
        for a in sorted(s.agg_sig_parent_amount):
            ret += f"  AGG_SIG_PARENT_AMOUNT pk: {a[0]} msg: {a[1].hex()}\n"
        for a in sorted(s.agg_sig_parent_puzzle):
            ret += f"  AGG_SIG_PARENT_PUZZLE pk: {a[0]} msg: {a[1].hex()}\n"
    ret += f"cost: {result.cost}\n"
    ret += f"removal_amount: {result.removal_amount}\n"
    ret += f"addition_amount: {result.addition_amount}\n"
    return ret


if __name__ == "__main__":
    try:
        error_code, result, run_time = run_gen(
            sys.argv[1],
            0 if len(sys.argv) < 3 else int(sys.argv[2]),
            None if len(sys.argv) < 4 else sys.argv[3],
        )
        if error_code is not None:
            print(f"Validation Error: {error_code}")
            print(f"run-time: {run_time:.2f}s")
            sys.exit(1)
        start_time = time()
        print("Spend bundle:")
        print(print_spend_bundle_conditions(result))
        print_time = time() - start_time
        print(f"run-time: {run_time:.2f}s")
        print(f"print-time: {print_time:.2f}s")
    except Exception as e:
        run_time = time() - start_time
        print("FAIL:", e)
        print(f"run-time: {run_time:.2f}s")
