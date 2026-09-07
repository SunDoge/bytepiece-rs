use super::common::SpatialToken;
use super::utils::segments;
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
    pattern_info: Vec<(usize, usize)>,
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
        self.segment_with(text.as_ref(), alpha, |text, start, end, _| {
            &text[start..end]
        })
    }

    fn encode(
        &self,
        text: impl AsRef<[u8]>,
        add_bos: bool,
        add_eos: bool,
        alpha: f64,
    ) -> Vec<usize> {
        let mut ids = self.segment_with(text.as_ref(), alpha, |_, _, _, pattern| {
            self.pattern_info[pattern].0
        });
        if add_bos {
            ids.insert(0, SpatialToken::Bos as usize);
        }
        if add_eos {
            ids.push(SpatialToken::Eos as usize);
        }
        ids
    }

    fn decode(&self, ids: &[usize]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        for &id in ids.iter().filter(|&&id| id > 2) {
            let piece = self
                .id_to_piece
                .get(&id)
                .ok_or(crate::Error::UnknownId(id))?;
            bytes.extend_from_slice(piece);
        }
        Ok(bytes)
    }
}

impl<'a> Tokenizer<'a> {
    pub fn from_pieces(pieces: &'a Pieces) -> Result<Self> {
        let mut ids = std::collections::HashSet::with_capacity(pieces.len());
        for (piece, &(id, _, count)) in pieces {
            if piece.is_empty() || id < 3 || !ids.insert(id) || count == 0 {
                return Err(crate::Error::InvalidModel(
                    "pieces must be nonempty, have positive counts and unique IDs >= 3".into(),
                ));
            }
        }
        for byte in 0..=255u8 {
            if !pieces.contains_key(&[byte][..]) {
                return Err(crate::Error::InvalidModel(format!(
                    "missing single-byte piece {byte}"
                )));
            }
        }
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

        let total: f64 = pieces.values().map(|vs| vs.2 as f64).sum();
        let log_total = total.ln();

        let values: Vec<f64> = pieces
            .values()
            .map(|vs| (vs.2 as f64).ln() - log_total)
            .collect();

        let pattern_info = pieces
            .iter()
            .map(|(piece, vs)| (vs.0, piece.len()))
            .collect();

        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::Standard)
            .build(pieces.keys())?;

        Ok(Self {
            id_to_piece,
            piece_to_id,
            vocab_size,
            values,
            pattern_info,
            ac,
        })
    }

    fn segment_with<'s, T>(
        &self,
        text: &'s [u8],
        alpha: f64,
        mut emit: impl FnMut(&'s [u8], usize, usize, usize) -> T,
    ) -> Vec<T> {
        let mut scores = Vec::new();
        let mut routes = Vec::new();
        let mut tokens = Vec::new();
        // Buffers are local to a call, so concurrent callers need no locks.
        for text in segments(text) {
            scores.clear();
            scores.resize(text.len() + 1, f64::NEG_INFINITY);
            scores[0] = 0.0;
            routes.clear();
            routes.resize(text.len() + 1, 0);
            // Keep sampling and deterministic matching in separate loops: the common
            // Viterbi path does not need a per-match alpha check or random state.
            if alpha < 0.0 || !alpha.is_finite() {
                for mat in self.ac.find_overlapping_iter(text) {
                    let (start, end, pattern) = (mat.start(), mat.end(), mat.pattern().as_usize());
                    let score = scores[start] + self.values[pattern];
                    if score > scores[end] {
                        scores[end] = score;
                        routes[end] = pattern;
                    }
                }
            } else {
                for mat in self.ac.find_overlapping_iter(text) {
                    let (start, end, pattern) = (mat.start(), mat.end(), mat.pattern().as_usize());
                    let score = scores[start] + alpha * self.values[pattern];
                    let total = logsumexp(scores[end], score);
                    if scores[end] == f64::NEG_INFINITY || fastrand::f64() < (score - total).exp() {
                        routes[end] = pattern;
                    }
                    scores[end] = total;
                }
            }
            let offset = tokens.len();
            let mut end = text.len();
            while end > 0 {
                let pattern = routes[end];
                let start = end - self.pattern_info[pattern].1;
                tokens.push(emit(text, start, end, pattern));
                end = start;
            }
            tokens[offset..].reverse();
        }
        tokens
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

    fn encode(
        &self,
        text: impl AsRef<[u8]>,
        add_bos: bool,
        add_eos: bool,
        alpha: f64,
    ) -> Vec<usize> {
        self.borrow_tokenizer()
            .encode(text, add_bos, add_eos, alpha)
    }

    fn decode(&self, ids: &[usize]) -> Result<Vec<u8>> {
        self.borrow_tokenizer().decode(ids)
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
        Self::from_slice(&buf)
    }

    /// Load a JSON model from memory (including browser-fetched model bytes).
    pub fn from_slice(buf: &[u8]) -> Result<Self> {
        make_owned_tokenizer(parse_pieces_from_slice(buf)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pieces() -> Pieces {
        let mut pieces: Pieces = (0..=255u8)
            .map(|b| (vec![b], (b as usize + 3, String::new(), 1)))
            .collect();
        pieces.insert(b"ab".to_vec(), (259, String::new(), 100));
        pieces.insert(b"\n\n".to_vec(), (260, String::new(), 100));
        pieces
    }

    #[test]
    fn deterministic_roundtrip_and_direct_ids() {
        let tk = make_owned_tokenizer(pieces()).unwrap();
        for text in [b"".as_slice(), b"ab", b"ab\n\nab", b"\xff\x00\xfe"] {
            for bos in [false, true] {
                for eos in [false, true] {
                    let ids = tk.encode(text, bos, eos, -1.0);
                    assert_eq!(tk.decode(&ids).unwrap(), text);
                    let expected = tk.pieces_to_ids(&tk.tokenize(&text, -1.0));
                    assert_eq!(
                        &ids[usize::from(bos)..ids.len() - usize::from(eos)],
                        expected
                    );
                }
            }
        }
        assert_eq!(tk.encode(b"ab", false, false, -1.0), [259]);
        assert_eq!(tk.decode(&[1, 259, 2]).unwrap(), b"ab");
        assert!(matches!(
            tk.decode(&[99999]),
            Err(crate::Error::UnknownId(99999))
        ));
        let bytes: Vec<_> = (0..=255u8).collect();
        assert_eq!(
            tk.decode(&tk.encode(&bytes, false, false, -1.0)).unwrap(),
            bytes
        );
    }

    #[test]
    fn rejects_models_that_could_leave_routes_unreachable() {
        assert!(make_owned_tokenizer(Pieces::new()).is_err());
        for kind in 0..5 {
            let mut p = pieces();
            match kind {
                0 => {
                    p.remove(&vec![0]);
                }
                1 => {
                    p.insert(vec![], (999, String::new(), 1));
                }
                2 => {
                    p.get_mut(&b"ab"[..]).unwrap().0 = 1;
                }
                3 => {
                    p.get_mut(&b"ab"[..]).unwrap().0 = 3;
                }
                _ => {
                    p.get_mut(&b"ab"[..]).unwrap().2 = 0;
                }
            }
            assert!(make_owned_tokenizer(p).is_err());
        }
        assert!(OwnedTokenizer::from_slice(b"not json").is_err());
        assert!(OwnedTokenizer::from_slice(br#"{"!": [3,"",1]}"#).is_err());
    }

    #[test]
    fn sampling_matches_unigram_probability() {
        let tk = make_owned_tokenizer(pieces()).unwrap();
        // For "ab", weights are P(ab) and P(a)*P(b).
        let total = 456.0_f64;
        let alpha = 0.2;
        let whole = (100.0 / total).powf(alpha);
        let split = (1.0 / total).powf(2.0 * alpha);
        let expected = whole / (whole + split);
        fastrand::seed(42);
        let samples = 20_000;
        let count = (0..samples)
            .filter(|_| tk.encode(b"ab", false, false, alpha) == [259])
            .count();
        assert!((count as f64 / samples as f64 - expected).abs() < 0.02);
        for alpha in [0.0, 1.0, 100.0, f64::MAX, f64::NAN, f64::INFINITY] {
            let text = b"ab\n\n\xffab";
            assert_eq!(
                tk.decode(&tk.encode(text, false, false, alpha)).unwrap(),
                text
            );
        }
    }
}
