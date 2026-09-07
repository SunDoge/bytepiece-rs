use bytepiece::{prelude::*, Error};
use unicode_normalization::UnicodeNormalization;
use wasm_bindgen::prelude::*;

fn js_error(error: Error) -> JsError {
    JsError::new(&error.to_string())
}

/// A tokenizer backed by a BytePiece JSON model loaded from a Uint8Array.
#[wasm_bindgen]
pub struct Tokenizer {
    inner: OwnedTokenizer,
}

#[wasm_bindgen]
impl Tokenizer {
    #[wasm_bindgen(constructor)]
    pub fn new(model: &[u8]) -> Result<Tokenizer, JsError> {
        Ok(Self {
            inner: OwnedTokenizer::from_slice(model).map_err(js_error)?,
        })
    }

    #[wasm_bindgen(getter, js_name = vocabSize)]
    pub fn vocab_size(&self) -> usize {
        self.inner.vocab_size()
    }

    /// Strings receive NFC normalization, matching the original Python tokenizer.
    pub fn encode(
        &self,
        text: &str,
        add_bos: Option<bool>,
        add_eos: Option<bool>,
        alpha: Option<f64>,
    ) -> Vec<usize> {
        self.encode_bytes(
            text.nfc().collect::<String>().as_bytes(),
            add_bos,
            add_eos,
            alpha,
        )
    }

    /// Bytes are processed losslessly, without Unicode normalization.
    #[wasm_bindgen(js_name = encodeBytes)]
    pub fn encode_bytes(
        &self,
        text: &[u8],
        add_bos: Option<bool>,
        add_eos: Option<bool>,
        alpha: Option<f64>,
    ) -> Vec<usize> {
        self.inner.encode(
            text,
            add_bos.unwrap_or(false),
            add_eos.unwrap_or(false),
            alpha.unwrap_or(-1.0),
        )
    }

    pub fn tokenize(&self, text: &str, alpha: Option<f64>) -> js_sys::Array {
        self.tokenize_bytes(text.nfc().collect::<String>().as_bytes(), alpha)
    }

    #[wasm_bindgen(js_name = tokenizeBytes)]
    pub fn tokenize_bytes(&self, text: &[u8], alpha: Option<f64>) -> js_sys::Array {
        self.inner
            .tokenize(&text, alpha.unwrap_or(-1.0))
            .into_iter()
            .map(js_sys::Uint8Array::from)
            .collect()
    }

    pub fn decode(&self, ids: &[usize]) -> Result<String, JsError> {
        let bytes = self.decode_bytes(ids)?;
        // Upstream decode(errors="ignore") drops invalid UTF-8 bytes.
        let mut rest = bytes.as_slice();
        let mut output = String::new();
        loop {
            match std::str::from_utf8(rest) {
                Ok(valid) => {
                    output.push_str(valid);
                    break;
                }
                Err(error) => {
                    let (valid, tail) = rest.split_at(error.valid_up_to());
                    output.push_str(std::str::from_utf8(valid).unwrap());
                    match error.error_len() {
                        Some(len) => rest = &tail[len..],
                        None => break,
                    }
                }
            }
        }
        Ok(output)
    }

    #[wasm_bindgen(js_name = decodeBytes)]
    pub fn decode_bytes(&self, ids: &[usize]) -> Result<Vec<u8>, JsError> {
        self.inner.decode(ids).map_err(js_error)
    }
}
