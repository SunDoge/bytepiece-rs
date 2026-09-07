# BytePiece WebAssembly

从原版 BytePiece JSON 模型构建的浏览器 / Node.js 分词器，包含 JavaScript 和 TypeScript 声明。

## 构建

在仓库根目录执行（需要 Rust stable）：

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
scripts/build_wasm.sh
node scripts/test_wasm.mjs
```

产物位于 `bytepiece-wasm/pkg-web`（浏览器 ES module）和 `bytepiece-wasm/pkg-nodejs`（Node.js CommonJS）。CLI 必须与 Cargo 中 wasm-bindgen 的版本一致。生成文件不提交到 Git；CI 会上传构建产物。

## 浏览器

将 `pkg-web` 和模型部署到 HTTP 服务下：

```js
import init, { Tokenizer } from './pkg-web/bytepiece.js';

await init();
const response = await fetch('./bytepiece_80k.model');
if (!response.ok) throw new Error(`Model download failed: ${response.status}`);
const tokenizer = new Tokenizer(new Uint8Array(await response.arrayBuffer()));
const ids = tokenizer.encode('今天天气不错'); // Uint32Array
console.log(ids, tokenizer.decode(ids));
console.log(tokenizer.tokenize('今天天气不错')); // Uint8Array[]
tokenizer.free(); // 用完后释放模型及自动机内存
```

## Node.js

```js
const { readFileSync } = require('node:fs');
const { Tokenizer } = require('./pkg-nodejs/bytepiece.js');
const tokenizer = new Tokenizer(readFileSync('./bytepiece_80k.model'));
const ids = tokenizer.encode('你好', true, true); // 加入 BOS / EOS
console.log(tokenizer.decode(ids));
tokenizer.free();
```

`encode(text, addBos = false, addEos = false, alpha = -1)` 和 `tokenize(text, alpha = -1)` 对字符串执行 NFC 归一化，与原版一致。`alpha >= 0` 使用随机分词，负数使用 Viterbi 最优路径；随机序列不与原版 C 随机数生成器逐次一致。

`encodeBytes` / `tokenizeBytes` 接受 `Uint8Array`，不做 Unicode 归一化；`decodeBytes` 返回原始字节，可无损处理二进制数据。`decode` 与原版一样忽略非法 UTF-8 字节。`vocabSize` 是只读属性。

模型必须包含所有 256 个单字节 token，token 非空、计数为正且 ID 唯一且不小于 3。加载无效模型、解码不存在的 ID 会抛出 JavaScript Error。

## 与原版对照

安装原版 `bytepiece==0.6.3`（其 pyahocorasick 需用 `AHOCORASICK_BYTES=1` 从源码构建）及本仓库 Python wheel，下载 80k 模型后：

```sh
python scripts/check_upstream.py
node scripts/test_wasm.mjs --upstream
```

这会对 Python 绑定和两种 Wasm 模块运行同一组中英日韩阿拉伯语、emoji、NFC、空输入、连续换行与随机文本用例。Web 模块在 Node.js 的 WebAssembly 运行时中加载测试。
