use bytepiece::tokenizer::{make_owned_tokenizer, OwnedTokenizer, Pieces, Tokenize};
use pyo3::prelude::*;

#[pyclass]
pub struct _Tokenizer {
    inner: OwnedTokenizer,
}

#[pymethods]
impl _Tokenizer {
    #[new]
    fn new(pieces: Pieces) -> Self {
        Self {
            inner: make_owned_tokenizer(pieces),
        }
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

    pub fn tokenize<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str> {
        self.inner.tokenize(text, alpha)
    }
}
