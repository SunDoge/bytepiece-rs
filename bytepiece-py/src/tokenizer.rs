use crate::Result;
use bytepiece::tokenizer::{
    make_owned_tokenizer, OwnedTokenizer, Pieces, Tokenize,
};
use pyo3::prelude::*;
use pyo3::types::PyType;

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

    pub fn id_to_piece(&self, id: usize) -> &str {
        self.inner.id_to_piece(id)
    }

    pub fn piece_to_id(&self, p: &str) -> usize {
        self.inner.piece_to_id(p)
    }

    #[pyo3(signature = (text, alpha = -1.0))]
    pub fn tokenize<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str> {
        self.inner.tokenize(text, alpha)
    }

    pub fn pieces_to_ids(&self, pieces: Vec<&str>) -> Vec<usize> {
        self.inner.pieces_to_ids(&pieces)
    }

    pub fn ids_to_pieces(&self, ids: Vec<usize>) -> Vec<&str> {
        self.inner.ids_to_pieces(&ids)
    }

    #[pyo3(signature = (text, add_bos = false, add_eos = false, alpha = -1.0))]
    pub fn encode(&self, text: &str, add_bos: bool, add_eos: bool, alpha: f64) -> Vec<usize> {
        self.inner.encode(text, add_bos, add_eos, alpha)
    }

    pub fn decode(&self, ids: Vec<usize>) -> String {
        self.inner.decode(&ids)
    }
}
