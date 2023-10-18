use once_cell::sync::Lazy;
use regex::bytes::Regex;

pub fn normalize(text: &[u8], max_len: usize) -> Vec<&[u8]> {
    if max_len > 0 {
        let pattern = format!(
            r".{{,{max_len}}}\n{{1,100}}|.{{1,{max_len}}}",
            max_len = max_len
        );
        let regex = Regex::new(&pattern).unwrap();
        regex.find_iter(text).map(|mat| mat.as_bytes()).collect()
    } else {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r".*\n+|.+").unwrap());
        RE.find_iter(text).map(|mat| mat.as_bytes()).collect()
    }
}

#[inline]
pub fn logsumexp(x: f64, y: f64) -> f64 {
    let (x, y) = if x < y { (y, x) } else { (x, y) };
    return x + (1.0 + (y - x).exp()).log2();
}
