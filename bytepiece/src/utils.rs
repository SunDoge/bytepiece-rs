/// The byte equivalent of upstream's `.*\n+|.+`, without regex or allocation.
pub fn segments(mut text: &[u8]) -> impl Iterator<Item = &[u8]> {
    std::iter::from_fn(move || {
        if text.is_empty() {
            return None;
        }
        let mut end = memchr::memchr(b'\n', text).unwrap_or(text.len());
        while end < text.len() && text[end] == b'\n' {
            end += 1;
        }
        let (segment, rest) = text.split_at(end);
        text = rest;
        Some(segment)
    })
}

#[inline]
pub fn logsumexp(x: f64, y: f64) -> f64 {
    let (x, y) = if x < y { (y, x) } else { (x, y) };
    if y == f64::NEG_INFINITY {
        return x;
    }
    x + (y - x).exp().ln_1p()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitting_preserves_bytes_and_newline_runs() {
        let input = b"\n\na\r\n\nb\xff\nlast";
        let chunks: Vec<_> = segments(input).collect();
        assert_eq!(chunks, vec![&b"\n\n"[..], b"a\r\n\n", b"b\xff\n", b"last"]);
        assert_eq!(chunks.concat(), input);
        assert_eq!(segments(b"").count(), 0);
    }

    #[test]
    fn natural_log_sum() {
        assert!((logsumexp(2.0_f64.ln(), 3.0_f64.ln()) - 5.0_f64.ln()).abs() < 1e-14);
        assert_eq!(
            logsumexp(f64::NEG_INFINITY, f64::NEG_INFINITY),
            f64::NEG_INFINITY
        );
    }
}
