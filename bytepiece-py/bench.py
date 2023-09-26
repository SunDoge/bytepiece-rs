import bytepiece
import timeit
import bytepiece_py
import rs_bytepiece
import functools
import time


TEXT = "BytePiece是一个Byte-based的Unigram分词器，纯Python实现，更加易读和易拓展。由于采用了新的训练算法，所以压缩率通常比现有Tokenizer更高，同时支持多进程加速训练。此外，它直接操作文本的UTF-8 Bytes，几乎不进行任何的预处理，所以更加纯粹和语言无关。"


t1 = bytepiece.Tokenizer("models/bytepiece_80k.model")
t2 = bytepiece_py.Tokenizer.from_path('models/bytepiece_80k.model')
t3 = rs_bytepiece.Tokenizer("models/bytepiece_80k.model")

assert t1.encode(TEXT) == t2.encode(TEXT)
print(t1.encode(TEXT))
print(t2.encode(TEXT))


print('bytepiece:')
print(timeit.timeit("t1.encode(TEXT)", globals=globals(), number=10000))
print('bytepiece-py (ours)')
print(timeit.timeit("t2.encode(TEXT)", globals=globals(), number=10000))
print('rs-bytepiece')
print(timeit.timeit("t3.encode(TEXT)", globals=globals(), number=10000))
