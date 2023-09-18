use std::collections::HashMap;

use super::utils::normalize;
use aho_corasick::{AhoCorasick, MatchKind};

pub type Pieces = HashMap<Vec<u8>, (usize, String, usize)>;

pub fn parse_pieces_from_slice(buf: &[u8]) -> Pieces {
    let res: HashMap<&str, (usize, String, usize)> = serde_json::from_slice(buf).unwrap();
    let base64 = base64_simd::STANDARD;
    res.into_iter()
        .map(|(key, value)| {
            let new_key = base64.decode_to_vec(key).unwrap();
            (new_key, value)
        })
        .collect()
}

pub struct Tokenizer<'a> {
    piece_to_id: HashMap<&'a [u8], usize>,
    id_to_piece: HashMap<usize, &'a [u8]>,
    vocab_size: usize,
    values: Vec<f64>,
    ac: AhoCorasick,
}

impl<'a> Tokenizer<'a> {
    pub fn from_pieces(pieces: &'a Pieces) -> Self {
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
            .build(pieces.keys())
            .unwrap();

        Self {
            id_to_piece,
            piece_to_id,
            vocab_size,
            values,
            ac,
        }
    }

    pub fn tokenize<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str> {
        normalize(text, 0)
            .into_iter()
            .flat_map(|s| self.process(s, alpha))
            .collect()
    }

    pub fn encode(&self, text: &str, add_bos: bool, add_eos: bool, alpha: f64) -> Vec<usize> {
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

    pub fn decode(&self, ids: &[usize]) -> String {
        let pieces: Vec<&str> = ids
            .into_iter()
            .filter(|i| **i > 2)
            .map(|i| self.id_to_piece(*i))
            .collect();
        pieces.join("")
    }

    pub fn piece_to_id(&self, p: &str) -> usize {
        self.piece_to_id[p.as_bytes()]
    }

    pub fn id_to_piece(&self, i: usize) -> &str {
        std::str::from_utf8(self.id_to_piece[&i]).unwrap()
    }

    pub fn pieces_to_ids(&self, pieces: &[&str]) -> Vec<usize> {
        pieces.into_iter().map(|p| self.piece_to_id(*p)).collect()
    }

    pub fn ids_to_pieces(&self, ids: &[usize]) -> Vec<&str> {
        ids.into_iter().map(|i| self.id_to_piece(*i)).collect()
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    fn process<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str> {
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

fn sigmoid(x: f64) -> f64 {
    if x >= 0. {
        1. / (1. + (-x).exp())
    } else {
        1. - 1. / (1. + x.exp())
    }
}

#[cfg(test)]
mod tests {}
