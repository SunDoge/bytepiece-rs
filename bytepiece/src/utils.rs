use once_cell::sync::Lazy;
use regex::Regex;

pub fn normalize(text: &str, max_len: usize) -> Vec<&str> {
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
