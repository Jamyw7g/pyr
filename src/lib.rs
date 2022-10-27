use pyo3::prelude::*;

mod client;
mod common;
mod error;
mod response;
mod types;

use client::Client;
use response::{Header, Response};

/// A Python module implemented in Rust.
#[pymodule]
fn pyr(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add_class::<Response>()?;
    m.add_class::<Header>()?;
    pyo3::prepare_freethreaded_python();
    Ok(())
}
