mod error;
mod tokenizer;

use pyo3::prelude::*;
use tokenizer::_Tokenizer;

pub use error::{Error, Result};

/// A Python module implemented in Rust.
#[pymodule]
fn bytepiece_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<_Tokenizer>()?;
    Ok(())
}
