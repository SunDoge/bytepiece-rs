from . import bytepiece_py as _ext
from typing import Union, Dict, Tuple
import unicodedata


def normalize(text: str) -> bytes:
    return unicodedata.normalize('NFC', text).encode()


class Tokenizer:

    def __init__(self, pieces: Union[str, Dict[str, Tuple[str, int, str]]]) -> None:
        if isinstance(pieces, str):
            self._tokenizer = _ext._Tokenizer.from_path(pieces)
        else:
            self._tokenizer = _ext._Tokenizer(pieces)

    def encode(self, text: Union[str, bytes], add_bos: bool = False, add_eos: bool = False, alpha: float = -1.0):
        if isinstance(text, str):
            text = normalize(text)
        return self._tokenizer.encode(text, add_bos=add_bos, add_eos=add_eos, alpha=alpha)

    def decode(self):
        return self._tokenizer.decode()

    def tokenize(self):
        if isinstance(text, str):
            text = normalize(text)
        return self._tokenizer.tokenize(text)

    def vocab_size(self):
        return self._tokenizer.vocab_size()
