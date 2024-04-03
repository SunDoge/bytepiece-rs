use crate::error::Result;
use bytepiece::tokenizer::{make_owned_tokenizer, OwnedTokenizer, Pieces, Tokenize};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyType};
use pyo3::Python;

#[pyclass]
pub struct _Tokenizer {
    inner: OwnedTokenizer,
}

#[pymethods]
impl _Tokenizer {
    #[new]
    fn new(pieces: Pieces) -> Result<Self> {
        Ok(Self {
            inner: make_owned_tokenizer(pieces)?,
        })
    }

    #[classmethod]
    fn from_path(_cls: &Bound<PyType>, path: &str) -> Result<Self> {
        let tk = OwnedTokenizer::from_path(path)?;
        Ok(Self { inner: tk })
    }

    pub fn vocab_size(&self) -> usize {
        self.inner.vocab_size()
    }

    #[pyo3(signature = (text, alpha = -1.0))]
    pub fn tokenize<'py>(
        &self,
        py: Python<'py>,
        text: &Bound<PyBytes>,
        alpha: f64,
    ) -> Vec<Bound<'py, PyBytes>> {
        let bs = text.as_bytes();
        let tokens = py.allow_threads(|| self.inner.tokenize(&bs, alpha));
        tokens
            .into_iter()
            .map(|bs| PyBytes::new_bound(py, bs))
            .collect()
    }

    #[pyo3(signature = (text, add_bos = false, add_eos = false, alpha = -1.0))]
    pub fn encode(
        &self,
        py: Python<'_>,
        text: &Bound<PyBytes>,
        add_bos: bool,
        add_eos: bool,
        alpha: f64,
    ) -> Vec<usize> {
        let bs = text.as_bytes();
        py.allow_threads(|| self.inner.encode(bs, add_bos, add_eos, alpha))
    }

    pub fn decode<'py>(&self, py: Python<'py>, ids: Vec<usize>) -> Result<Bound<'py, PyBytes>> {
        let res = py.allow_threads(|| self.inner.decode(&ids))?;
        Ok(PyBytes::new_bound(py, &res))
    }

    pub fn id_to_piece<'py>(&self, py: Python<'py>, id: usize) -> Bound<'py, PyBytes> {
        PyBytes::new_bound(py, self.inner.id_to_piece(id))
    }

    pub fn piece_to_id(&self, piece: &Bound<PyBytes>) -> usize {
        self.inner.piece_to_id(piece.as_bytes())
    }
}
