use super::adapt_response::eval_err_to_pyresult;
use chik_consensus::allocator::make_allocator;
use chik_consensus::gen::flags::ALLOW_BACKREFS;
use chik_protocol::LazyNode;
use klvmr::chik_dialect::ChikDialect;
use klvmr::cost::Cost;
use klvmr::reduction::Response;
use klvmr::run_program::run_program;
use klvmr::serde::{node_from_bytes, node_from_bytes_backrefs, serialized_length_from_bytes};
use pyo3::buffer::PyBuffer;
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

    let r: Response = (|| -> PyResult<Response> {
        let deserialize = if (flags & ALLOW_BACKREFS) != 0 {
            node_from_bytes_backrefs
        } else {
            node_from_bytes
        };
        let program = deserialize(&mut allocator, program)?;
        let args = deserialize(&mut allocator, args)?;
        let dialect = ChikDialect::new(flags);

        Ok(py.allow_threads(|| run_program(&mut allocator, &dialect, program, args, max_cost)))
    })()?;
    match r {
        Ok(reduction) => {
            let val = LazyNode::new(Rc::new(allocator), reduction.1);
            Ok((reduction.0, val))
        }
        Err(eval_err) => eval_err_to_pyresult(eval_err, &allocator),
    }
}
