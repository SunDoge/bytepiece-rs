mod error;
mod tokenizer;

use pyo3::prelude::*;
use tokenizer::_Tokenizer;

/// A Python module implemented in Rust.
#[pymodule]
fn _lowlevel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<_Tokenizer>()?;
    Ok(())
}
