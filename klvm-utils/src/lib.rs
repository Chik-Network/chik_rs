//! # KLVM Utils
//! This provides various commonly needed utilities for working with KLVM values.
//!
//! ## Currying Example
//!
//! ```rust
//! use klvm_utils::CurriedProgram;
//! use klvm_traits::{ToKlvm, klvm_curried_args};
//! use klvmr::{Allocator, serde::node_to_bytes};
//!
//! let a = &mut Allocator::new();
//!
//! let program = a.one();
//!
//! let ptr = CurriedProgram {
//!     program,
//!     args: klvm_curried_args!(42, 75),
//! }
//! .to_klvm(a)
//! .unwrap();
//!
//! let hex = hex::encode(node_to_bytes(a, ptr).unwrap());
//!
//! // (a (q . 1) (c (q . 42) (c (q . 75) 1)))
//! assert_eq!(hex, "ff02ffff0101ffff04ffff012affff04ffff014bff01808080");

mod curried_program;
mod tree_hash;

pub use curried_program::*;
pub use tree_hash::*;
/*
    let curry = CurriedProgram {
        program: program.to_klvm(a).unwrap(),
        args: args.clone(),
    }
    .to_klvm(a)
    .unwrap();
    let actual = node_to_bytes(a, curry).unwrap();
    assert_eq!(hex::encode(actual), expected);

    let curried = CurriedProgram::<A>::from_klvm(a, curry).unwrap();
    let round_program = T::from_klvm(a, curried.program).unwrap();
*/
