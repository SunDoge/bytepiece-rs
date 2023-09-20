use std::{collections::HashMap, ops::AddAssign};

pub struct Trainer {
    order: usize,
    max_vocab_size: Vec<usize>,
    max_piece_length: usize,
    min_count: usize,
}

impl Default for Trainer {
    fn default() -> Self {
        Self {
            order: 6,
            max_vocab_size: vec![10000],
            max_piece_length: 36,
            min_count: 2,
        }
    }
}

impl Trainer {
    pub fn new(
        order: usize,
        max_vocab_size: &[usize],
        max_piece_length: usize,
        min_count: usize,
    ) -> Self {
        Self {
            order,
            max_vocab_size: max_vocab_size.to_vec(),
            max_piece_length,
            min_count,
        }
    }

    fn count_ngrams<'a>(&self, texts: &[&'a str]) -> Vec<HashMap<&'a str, usize>> {
        let mut ngrams = vec![HashMap::new(); self.order + 1];
        for text in texts {
            for i in 0..text.len() {
                for j in 0..(self.order + 1) {
                    let k = &text[i..(i + j)];
                    ngrams[j].entry(k).or_insert(0).add_assign(1);
                }
            }
        }
        ngrams
    }

    // fn prune_ngrams<'a>(
    //     &self,
    //     ngrams: &mut [HashMap<&'a str, usize>],
    // ) -> Vec<HashMap<&'a str, f64>> {
    //     for i in 0..=255u8 {
    //         let key = unsafe { std::str::from_utf8_unchecked(&[i]) };
    //         if !ngrams[0].contains_key(key) {
    //             ngrams.get_mut(1).unwrap().insert(key, 1);
    //             ngrams
    //                 .get_mut(0)
    //                 .unwrap()
    //                 .get_mut("")
    //                 .unwrap()
    //                 .add_assign(1);
    //         }
    //     }

    //     let mut new_ngrams = vec![HashMap::new(); self.order + 1];
    //     for i in (0..ngrams.len() - 1).rev() {
    //         let ngram1: HashMap<&str, f64> = ngrams[i]
    //             .iter()
    //             .filter(|(key, value)| {
    //                 key.len() == i && **value > (if i > 0 { self.min_count } else { 0 })
    //             })
    //             .map(|(key, value)| (*key, (*value as f64).log2()))
    //             .collect();

    //         if i < ngrams.len() - 1 {
    //             let ngram2: HashMap<&str, f64> = ngrams[i + 1]
    //                 .iter()
    //                 .map(|(key, value)| (*key, *value as f64 - ngram1[&key[..i]]))
    //                 .collect();
    //             new_ngrams[i + 1] = ngram2;
    //         }

    //         new_ngrams[i] = ngram1;
    //     }
    //     new_ngrams
    // }
}
