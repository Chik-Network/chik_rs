#![no_main]
use libfuzzer_sys::fuzz_target;

use chik_consensus::sanitize_int::{SanitizedUint, sanitize_uint};
use chik_consensus::validation_error::{ErrorCode, ValidationErr};
use clvkr::allocator::Allocator;

fuzz_target!(|data: &[u8]| {
    let mut a = Allocator::new();
    let atom = a.new_atom(data).unwrap();
    match sanitize_uint(
        &a,
        atom,
        8,
        ValidationErr::Err(ErrorCode::InvalidCoinAmount),
    ) {
        Ok(SanitizedUint::Ok(_)) => {
            assert!(data.len() <= 9);
            if data.len() == 9 {
                assert_eq!(data[0], 0);
            }
        }
        Ok(SanitizedUint::NegativeOverflow) => {
            assert_ne!(data[0] & 0x80, 0);
        }
        Ok(SanitizedUint::PositiveOverflow) => {
            assert!(data.len() > 8);
        }
        Err(ValidationErr::Err(c)) => {
            assert_eq!(c, ErrorCode::InvalidCoinAmount);
        }
        _ => {
            panic!("invalid state");
        }
    }

    match sanitize_uint(
        &a,
        atom,
        4,
        ValidationErr::Err(ErrorCode::InvalidCoinAmount),
    ) {
        Ok(SanitizedUint::Ok(_)) => {
            assert!(data.len() <= 5);
            if data.len() == 5 {
                assert_eq!(data[0], 0);
            }
        }
        Ok(SanitizedUint::NegativeOverflow) => {
            assert_ne!(data[0] & 0x80, 0);
        }
        Ok(SanitizedUint::PositiveOverflow) => {
            assert!(data.len() > 4);
        }
        Err(ValidationErr::Err(c)) => {
            assert_eq!(c, ErrorCode::InvalidCoinAmount);
        }
        _ => {
            panic!("invalid state");
        }
    }
});
