use super::utils::normalize;
use crate::Result;
use aho_corasick::{AhoCorasick, MatchKind};
use ouroboros::self_referencing;
use std::collections::HashMap;
use std::path::Path;

pub type Pieces = HashMap<Vec<u8>, (usize, String, usize)>;

pub fn parse_pieces_from_slice(buf: &[u8]) -> Result<Pieces> {
    let dict: HashMap<&str, (usize, String, usize)> = serde_json::from_slice(buf)?;
    let base64 = base64_simd::STANDARD;
    dict.into_iter()
        .map(|(key, value)| {
            let new_key = base64.decode_to_vec(key)?;
            Ok((new_key, value))
        })
        .collect::<Result<HashMap<_, _>>>()
}

pub trait Tokenize {
    fn tokenize<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str>;
    fn piece_to_id(&self, p: &str) -> usize;
    fn id_to_piece(&self, i: usize) -> &str;
    fn vocab_size(&self) -> usize;

    fn pieces_to_ids(&self, pieces: &[&str]) -> Vec<usize> {
        pieces.into_iter().map(|p| self.piece_to_id(*p)).collect()
    }

    fn ids_to_pieces(&self, ids: &[usize]) -> Vec<&str> {
        ids.into_iter().map(|i| self.id_to_piece(*i)).collect()
    }

    fn encode(&self, text: &str, add_bos: bool, add_eos: bool, alpha: f64) -> Vec<usize> {
        let mut pieces = if add_bos {
            let mut pieces = vec![1];
            for p in self.tokenize(text, alpha) {
                pieces.push(self.piece_to_id(p));
            }
            pieces
        } else {
            self.tokenize(text, alpha)
                .into_iter()
                .map(|p| self.piece_to_id(p))
                .collect()
        };
        if add_eos {
            pieces.push(2);
        }
        pieces
    }

    fn decode(&self, ids: &[usize]) -> String {
        let pieces: Vec<&str> = ids
            .into_iter()
            .filter(|i| **i > 2)
            .map(|i| self.id_to_piece(*i))
            .collect();
        pieces.join("")
    }
}

pub struct Tokenizer<'a> {
    piece_to_id: HashMap<&'a [u8], usize>,
    id_to_piece: HashMap<usize, &'a [u8]>,
    vocab_size: usize,
    values: Vec<f64>,
    ac: AhoCorasick,
}

impl<'a> Tokenize for Tokenizer<'a> {
    fn id_to_piece(&self, i: usize) -> &str {
        std::str::from_utf8(self.id_to_piece[&i]).unwrap()
    }

    fn piece_to_id(&self, p: &str) -> usize {
        self.piece_to_id[p.as_bytes()]
    }

    fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    fn tokenize<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str> {
        normalize(text, 0)
            .into_iter()
            .flat_map(|s| self._tokenize(s, alpha))
            .collect()
    }
}

impl<'a> Tokenizer<'a> {
    pub fn from_pieces(pieces: &'a Pieces) -> Result<Self> {
        let piece_to_id: HashMap<&[u8], usize> = pieces
            .iter()
            .map(|(key, value)| (key.as_slice(), value.0))
            .collect();

        let id_to_piece: HashMap<usize, &[u8]> =
            piece_to_id.iter().map(|(k, v)| (*v, *k)).collect();

        let vocab_size = pieces.len() + 3;

        let total: usize = pieces.values().map(|vs| vs.2).sum();
        let log_total = (total as f64).log2();

        let values: Vec<f64> = pieces
            .values()
            .map(|vs| (vs.2 as f64).log2() - log_total)
            .collect();

        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::Standard)
            .build(pieces.keys())?;

        Ok(Self {
            id_to_piece,
            piece_to_id,
            vocab_size,
            values,
            ac,
        })
    }

    fn _tokenize<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str> {
        let mut scores = vec![-std::f64::INFINITY; text.len() + 1];
        scores[0] = 0.0;

        let mut routes: Vec<usize> = (0..text.len() + 1).collect();

        for mat in self.ac.find_overlapping_iter(text.as_bytes()) {
            let start = mat.start();
            let end = mat.end();

            let index = mat.pattern().as_usize();
            let value = self.values[index];

            let score = scores[start] + value;

            if alpha <= 0.0 && score > scores[end] {
                scores[end] = score;
                routes[end] = start;
            } else if alpha > 0.0 && rand::random::<f64>() < sigmoid((score - scores[end]) * alpha)
            {
                scores[end] = score;
                routes[end] = start;
            }
        }

        let mut text_slice = text;
        let mut end = routes.len() - 1;
        let mut tokens = Vec::new();

        while !text_slice.is_empty() {
            let start = routes[end];
            // dbg!(text_slice, start, end);
            tokens.push(&text_slice[start..end]);
            text_slice = &text[..start];
            end = start;
        }
        tokens.reverse();
        tokens
    }
}

#[inline]
fn sigmoid(x: f64) -> f64 {
    if x >= 0. {
        1. / (1. + (-x).exp())
    } else {
        1. - 1. / (1. + x.exp())
    }
}

#[self_referencing]
pub struct OwnedTokenizer {
    pieces: Pieces,
    #[borrows(pieces)]
    #[covariant]
    tokenizer: Tokenizer<'this>,
}

impl Tokenize for OwnedTokenizer {
    fn tokenize<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str> {
        self.borrow_tokenizer().tokenize(text, alpha)
    }

    fn vocab_size(&self) -> usize {
        self.borrow_tokenizer().vocab_size()
    }

    fn id_to_piece(&self, i: usize) -> &str {
        self.borrow_tokenizer().id_to_piece(i)
    }

    fn piece_to_id(&self, p: &str) -> usize {
        self.borrow_tokenizer().piece_to_id(p)
    }
}

pub fn make_owned_tokenizer(pieces: Pieces) -> Result<OwnedTokenizer> {
    OwnedTokenizerTryBuilder {
        pieces,
        tokenizer_builder: |pieces: &Pieces| Tokenizer::from_pieces(pieces),
    }
    .try_build()
}

impl OwnedTokenizer {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let buf = std::fs::read(path)?;
        let pieces = parse_pieces_from_slice(&buf)?;
        make_owned_tokenizer(pieces)
    }
}

#[cfg(test)]
mod tests {}
