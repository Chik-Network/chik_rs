use crate::error::map_pyerr;
use chik_consensus::allocator::make_allocator;
use chik_consensus::flags::ConsensusFlags;
use chik_protocol::LazyNode;
use klvmr::chik_dialect::ChikDialect;
use klvmr::cost::Cost;
use klvmr::reduction::Response;
use klvmr::run_program::run_program;
use klvmr::serde::{
    node_from_bytes_backrefs, serialized_length_from_bytes, serialized_length_from_bytes_trusted,
};
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
use std::rc::Rc;

#[allow(clippy::borrow_deref_ref)]
#[pyfunction]
pub fn serialized_length(program: PyBuffer<u8>) -> PyResult<u64> {
    assert!(program.is_c_contiguous(), "program must be contiguous");
    let program =
        unsafe { std::slice::from_raw_parts(program.buf_ptr() as *const u8, program.len_bytes()) };
    serialized_length_from_bytes(program).map_err(map_pyerr)
}

#[allow(clippy::borrow_deref_ref)]
#[pyfunction]
pub fn serialized_length_trusted(program: PyBuffer<u8>) -> PyResult<u64> {
    assert!(program.is_c_contiguous(), "program must be contiguous");
    let program =
        unsafe { std::slice::from_raw_parts(program.buf_ptr() as *const u8, program.len_bytes()) };
    serialized_length_from_bytes_trusted(program).map_err(map_pyerr)
}

#[allow(clippy::borrow_deref_ref)]
#[pyfunction]
pub fn run_chik_program(
    py: Python<'_>,
    program: &[u8],
    args: &[u8],
    max_cost: Cost,
    flags: ConsensusFlags,
) -> PyResult<(Cost, LazyNode)> {
    let mut allocator = make_allocator(flags);
    let flags = flags.to_klvm_flags();

    let reduction = (|| -> PyResult<Response> {
        let program = node_from_bytes_backrefs(&mut allocator, program).map_err(map_pyerr)?;
        let args = node_from_bytes_backrefs(&mut allocator, args).map_err(map_pyerr)?;
        let dialect = ChikDialect::new(flags);

        Ok(py.detach(|| run_program(&mut allocator, &dialect, program, args, max_cost)))
    })()?
    .map_err(map_pyerr)?;
    let val = LazyNode::new(Rc::new(allocator), reduction.1);
    Ok((reduction.0, val))
}
