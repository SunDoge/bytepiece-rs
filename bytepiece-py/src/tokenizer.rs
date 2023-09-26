use crate::Result;
use bytepiece::tokenizer::{make_owned_tokenizer, OwnedTokenizer, Pieces, Tokenize};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyType};
use pyo3::Python;

#[pyclass]
pub struct Tokenizer {
    inner: OwnedTokenizer,
}

#[pymethods]
impl Tokenizer {
    #[new]
    fn new(pieces: Pieces) -> Result<Self> {
        Ok(Self {
            inner: make_owned_tokenizer(pieces)?,
        })
    }

    #[classmethod]
    fn from_path(_cls: &PyType, path: &str) -> Result<Self> {
        let tk = OwnedTokenizer::from_path(path)?;
        Ok(Self { inner: tk })
    }

    pub fn vocab_size(&self) -> usize {
        self.inner.vocab_size()
    }

    #[pyo3(signature = (text, alpha = -1.0))]
    pub fn tokenize<'py>(&self, py: Python<'py>, text: &str, alpha: f64) -> Vec<&'py PyBytes> {
        self.inner
            .tokenize(text, alpha)
            .iter()
            .map(|bs| PyBytes::new(py, *bs))
            .collect()
    }

    #[pyo3(signature = (text, add_bos = false, add_eos = false, alpha = -1.0))]
    pub fn encode(
        &self,
        py: Python<'_>,
        text: &str,
        add_bos: bool,
        add_eos: bool,
        alpha: f64,
    ) -> Vec<usize> {
        py.allow_threads(|| self.inner.encode(text, add_bos, add_eos, alpha))
    }

    pub fn decode(&self, py: Python<'_>, ids: Vec<usize>) -> Result<String> {
        py.allow_threads(|| Ok(self.inner.decode(&ids)?))
    }
}
