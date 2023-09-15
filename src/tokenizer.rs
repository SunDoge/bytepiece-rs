use std::collections::HashMap;

use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;
use rand::{rngs::ThreadRng, Rng};
use regex::{Matches, Regex};
use serde::{Deserialize, Serialize};

pub type Pieces = HashMap<String, (usize, String, usize)>;

// #[derive(Debug, Deserialize, Serialize)]
// pub struct Pieces(pub HashMap<String, (usize, String, usize)>);

// impl Pieces {
//     pub fn from_slice(buf: &[u8]) -> Self {
//         let pieces: Dict = serde_json::from_slice(buf).unwrap()
//     }
// }

pub fn parse_pieces_from_slice(buf: &[u8]) -> Pieces {
    let res: HashMap<&str, (usize, String, usize)> = serde_json::from_slice(buf).unwrap();
    let base64 = base64_simd::STANDARD;
    res.into_iter()
        .map(|(key, value)| {
            let new_key = base64.decode_to_vec(key).unwrap();
            let new_key = unsafe { String::from_utf8_unchecked(new_key) };
            (new_key, value)
        })
        .collect()
}

pub struct Tokenizer<'a> {
    piece_to_id: HashMap<&'a str, usize>,
    id_to_piece: HashMap<usize, &'a str>,
    vocab_size: usize,
    values: Vec<f64>,
    ac: AhoCorasick,
}

impl<'a> Tokenizer<'a> {
    pub fn from_dict(pieces: &'a Pieces) -> Self {
        // let base64 = base64_simd::STANDARD;
        // let pieces: HashMap<String, &[usize]> = pieces
        //     .iter()
        //     .map(|(key, value)| {
        //         let buf = base64.decode_to_vec(*key).unwrap();
        //         (String::from_utf8(buf).unwrap(), &value[..])
        //     })
        //     .collect();

        let piece_to_id: HashMap<&str, usize> = pieces
            .iter()
            .map(|(key, value)| (key.as_str(), value.0))
            .collect();

        let id_to_piece: HashMap<usize, &str> = piece_to_id.iter().map(|(k, v)| (*v, *k)).collect();

        let vocab_size = pieces.len() + 3;

        let total: usize = pieces.values().map(|vs| vs.2).sum();
        let log_total = (total as f64).log2();

        let values: Vec<f64> = pieces
            .values()
            .map(|vs| (vs.2 as f64).log2() - log_total)
            .collect();

        let ac = AhoCorasick::new(pieces.keys()).unwrap();

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
        self.piece_to_id[p]
    }

    pub fn id_to_piece(&self, i: usize) -> &str {
        self.id_to_piece[&i]
    }

    pub fn pieces_to_ids(&self, pieces: &[&str]) -> Vec<usize> {
        pieces.into_iter().map(|p| self.piece_to_id(*p)).collect()
    }

    pub fn ids_to_pieces(&self, ids: &[usize]) -> Vec<&str> {
        ids.into_iter().map(|i| self.id_to_piece(*i)).collect()
    }

    fn process<'s>(&self, text: &'s str, alpha: f64) -> Vec<&'s str> {
        let mut scores = vec![-std::f64::INFINITY; text.len() + 1];
        scores[0] = 0.0;

        let mut routes: Vec<usize> = (0..text.len() + 1).collect();

        for mat in self.ac.find_iter(text) {
            dbg!(&mat);
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
        dbg!(&routes);
        while !text_slice.is_empty() {
            let start = routes[end];
            // dbg!(text_slice, start, end);
            tokens.push(&text_slice[start..end]);
            text_slice = &text[..start];
            end = start;
        }
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

fn normalize<'a>(text: &'a str, max_len: usize) -> Vec<&'a str> {
    if max_len > 0 {
        let pattern = format!(
            r".{{,{max_len}}}\n{{1,100}}|.{{1,{max_len}}}",
            max_len = max_len
        );
        let regex = Regex::new(&pattern).unwrap();
        regex.find_iter(text).map(|mat| mat.as_str()).collect()
    } else {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r".*\n+|.+").unwrap());
        RE.find_iter(text).map(|mat| mat.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {}
