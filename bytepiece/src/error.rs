pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("base64 error: {0}")]
    Base64(#[from] base64_simd::Error),

    #[error("fail to build aho corasick: {0}")]
    AhoCorasickBuild(#[from] aho_corasick::BuildError),

    #[error("fail to parse bytes")]
    FromUtf8(#[from] std::string::FromUtf8Error),
}
