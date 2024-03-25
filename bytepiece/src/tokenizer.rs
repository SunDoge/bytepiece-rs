use super::common::SpatialToken;
use super::utils::normalize;
use crate::utils::logsumexp;
use crate::{common, Result};
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
    fn tokenize<'s, T: AsRef<[u8]>>(&self, text: &'s T, alpha: f64) -> Vec<&'s [u8]>;
    fn piece_to_id(&self, p: &[u8]) -> usize;
    fn id_to_piece(&self, i: usize) -> &[u8];
    fn vocab_size(&self) -> usize;

    fn pieces_to_ids(&self, pieces: &[&[u8]]) -> Vec<usize> {
        pieces.iter().map(|p| self.piece_to_id(p)).collect()
    }

    fn ids_to_pieces(&self, ids: &[usize]) -> Vec<&[u8]> {
        ids.iter().map(|i| self.id_to_piece(*i)).collect()
    }

    fn encode(
        &self,
        text: impl AsRef<[u8]>,
        add_bos: bool,
        add_eos: bool,
        alpha: f64,
    ) -> Vec<usize> {
        let mut pieces = if add_bos {
            let mut pieces = vec![SpatialToken::Bos as usize];
            for p in self.tokenize(&text, alpha) {
                pieces.push(self.piece_to_id(p));
            }
            pieces
        } else {
            self.tokenize(&text, alpha)
                .into_iter()
                .map(|p| self.piece_to_id(p))
                .collect()
        };
        if add_eos {
            pieces.push(SpatialToken::Eos as usize);
        }
        pieces
    }

    fn decode(&self, ids: &[usize]) -> Result<Vec<u8>> {
        let piece: Vec<u8> = ids
            .iter()
            .filter(|i| **i > 2)
            .flat_map(|i| self.id_to_piece(*i).iter().copied())
            .collect();
        Ok(piece)
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
    fn id_to_piece(&self, i: usize) -> &[u8] {
        self.id_to_piece[&i]
    }

    fn piece_to_id(&self, p: &[u8]) -> usize {
        self.piece_to_id[p]
    }

    fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    fn tokenize<'s, T: AsRef<[u8]>>(&self, text: &'s T, alpha: f64) -> Vec<&'s [u8]> {
        // let nfc_text: String = text.nfc().collect();
        normalize(text.as_ref(), 0)
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

        let mut id_to_piece: HashMap<usize, &[u8]> =
            piece_to_id.iter().map(|(k, v)| (*v, *k)).collect();
        id_to_piece.insert(SpatialToken::Pad as usize, common::PAD);
        id_to_piece.insert(SpatialToken::Bos as usize, common::BOS);
        id_to_piece.insert(SpatialToken::Eos as usize, common::EOS);

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

    fn _tokenize<'s>(&self, text: &'s [u8], alpha: f64) -> impl Iterator<Item = &'s [u8]> {
        // Every ending's max score.
        let mut scores = vec![-std::f64::INFINITY; text.len() + 1];
        scores[0] = 0.0;
        // Every ending's start.
        let mut routes: Vec<usize> = (0..text.len() + 1).collect();

        for mat in self.ac.find_overlapping_iter(text) {
            let start = mat.start();
            let end = mat.end();

            let index = mat.pattern().as_usize();
            let value = self.values[index];

            if alpha < 0.0 {
                let score = scores[start] + value;
                if score > scores[end] {
                    scores[end] = score;
                    routes[end] = start;
                }
            } else {
                let score = scores[start] + alpha * value;
                scores[end] = logsumexp(scores[end], score);
                if fastrand::f64() < (score - scores[end]).exp() {
                    routes[end] = start;
                }
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
        tokens.into_iter().rev()
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
    fn tokenize<'s, T: AsRef<[u8]>>(&self, text: &'s T, alpha: f64) -> Vec<&'s [u8]> {
        self.borrow_tokenizer().tokenize(text, alpha)
    }

    fn vocab_size(&self) -> usize {
        self.borrow_tokenizer().vocab_size()
    }

    fn id_to_piece(&self, i: usize) -> &[u8] {
        self.borrow_tokenizer().id_to_piece(i)
    }

    fn piece_to_id(&self, p: &[u8]) -> usize {
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
