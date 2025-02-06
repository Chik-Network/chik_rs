use chik_consensus::allocator::make_allocator;
use chik_protocol::LazyNode;
use klvmr::chik_dialect::ChikDialect;
use klvmr::cost::Cost;
use klvmr::reduction::Response;
use klvmr::run_program::run_program;
use klvmr::serde::{node_from_bytes_backrefs, node_to_bytes, serialized_length_from_bytes};
use pyo3::buffer::PyBuffer;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::rc::Rc;

#[allow(clippy::borrow_deref_ref)]
#[pyfunction]
pub fn serialized_length(program: PyBuffer<u8>) -> PyResult<u64> {
    assert!(program.is_c_contiguous(), "program must be contiguous");
    let program =
        unsafe { std::slice::from_raw_parts(program.buf_ptr() as *const u8, program.len_bytes()) };
    Ok(serialized_length_from_bytes(program)?)
}

#[allow(clippy::borrow_deref_ref)]
#[pyfunction]
pub fn run_chik_program(
    py: Python<'_>,
    program: &[u8],
    args: &[u8],
    max_cost: Cost,
    flags: u32,
) -> PyResult<(Cost, LazyNode)> {
    let mut allocator = make_allocator(flags);

    let reduction = (|| -> PyResult<Response> {
        let program = node_from_bytes_backrefs(&mut allocator, program)?;
        let args = node_from_bytes_backrefs(&mut allocator, args)?;
        let dialect = ChikDialect::new(flags);

        Ok(py.allow_threads(|| run_program(&mut allocator, &dialect, program, args, max_cost)))
    })()?
    .map_err(|e| {
        let blob = node_to_bytes(&allocator, e.0).ok().map(hex::encode);
        PyValueError::new_err((e.1, blob))
    })?;
    let val = LazyNode::new(Rc::new(allocator), reduction.1);
    Ok((reduction.0, val))
}
