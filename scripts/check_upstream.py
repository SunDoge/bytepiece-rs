"""Compare the built Python binding with bojone/bytepiece 0.6.3.

Requires bytepiece (with AHOCORASICK_BYTES=1), bytepiece-py, and the 80k model.
Run from the repository root. Also writes the corpus consumed by the Wasm test.
"""
import json
import random
import unicodedata
from pathlib import Path

from bytepiece import Tokenizer as Reference
from bytepiece_py import Tokenizer

model = "models/bytepiece_80k.model"
reference = Reference(model)
actual = Tokenizer(model)
texts = [
    "", "今天天气不错", "Hello, world!", "BytePiece是一个Byte-based的Unigram分词器。",
    "\n", "\n\n", "a\n\n\nb\r\n", "e\u0301 café", "👩‍💻🚀🏳️‍🌈", "\x00\x01\x7f",
    "1234567890 １２３４５", "日本語 한국어 العربية русский", " " * 200,
    "a" * 4096, "\n" * 500, "hello\n世界\n\n" * 100,
]
rng = random.Random(42)
alphabet = "abcXYZ0123 \t\r\n中文你好é\u0301🙂🚀\x00"
texts.extend("".join(rng.choices(alphabet, k=rng.randrange(1, 300))) for _ in range(200))
cases = []
for text in texts:
    for bos, eos in [(False, False), (True, False), (False, True), (True, True)]:
        expected = reference.encode(text, add_bos=bos, add_eos=eos)
        observed = actual.encode(text, add_bos=bos, add_eos=eos)
        assert observed == expected, (repr(text), expected, observed)
        assert actual.decode(observed) == unicodedata.normalize("NFC", text)
    assert actual.tokenize(text) == reference.tokenize(text), repr(text)
    cases.append({"text": text, "ids": reference.encode(text)})
# Binary input is a Rust extension to the upstream string API.
binary = bytes(range(256))
assert b"".join(actual.tokenize(binary)) == binary
assert actual._tokenizer.decode(actual.encode(binary)) == binary
Path("target/upstream-cases.json").write_text(json.dumps(cases, ensure_ascii=False))
print(f"Matched upstream: {len(texts)} texts, {len(texts) * 4} encode cases, tokenize and decode")
