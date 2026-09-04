use crate::validation_error::ValidationErr;
use clvk_traits::{FromClvkError, ToClvkError};
use clvkr::error::EvalErr;
use thiserror::Error;

#[cfg(feature = "py-bindings")]
use pyo3::PyErr;

#[derive(Debug, PartialEq, Error)]
pub enum Error {
    #[error("To CLVK {0}")]
    ToClvk(#[from] ToClvkError),

    #[error("From CLVK {0}")]
    FromClvk(#[from] FromClvkError),

    #[error("Eval {0}")]
    Eval(#[from] EvalErr),

    #[error("Validation {0}")]
    Validation(#[from] ValidationErr),

    #[error("BLS {0}")]
    Bls(#[from] chik_bls::Error),

    #[error("not a singleton mod hash")]
    NotSingletonModHash,

    #[error("inner puzzle hash mismatch")]
    InnerPuzzleHashMismatch,

    #[error("puzzle hash mismatch")]
    PuzzleHashMismatch,

    #[error("coin amount mismatch")]
    CoinAmountMismatch,

    #[error("coin amount is even")]
    CoinAmountEven,

    #[error("parent coin mismatch")]
    ParentCoinMismatch,

    #[error("coin mismatch")]
    CoinMismatch,

    #[error("expected lineage proof, found eve proof")]
    ExpectedLineageProof,

    #[error("{0}")]
    Custom(String),
}

#[cfg(feature = "py-bindings")]
impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        pyo3::exceptions::PyValueError::new_err(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
