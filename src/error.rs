use crate::gen::validation_error::ValidationErr;
use klvmr::reduction::EvalErr;
use thiserror::Error;

#[cfg(feature = "py-bindings")]
use pyo3::PyErr;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("KLVM {0}")]
    Klvm(#[from] klvm_traits::Error),

    #[error("Eval {0}")]
    Eval(#[from] EvalErr),

    #[error("Validation {0}")]
    Validation(#[from] ValidationErr),

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

    #[error("{0}")]
    Custom(String),
}

#[cfg(feature = "py-bindings")]
impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        pyo3::exceptions::PyValueError::new_err(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;