import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';

const require = createRequire(import.meta.url);
const web = await import(pathToFileURL(`${process.cwd()}/bytepiece-wasm/pkg-web/bytepiece.js`));
await web.default({ module_or_path: readFileSync('bytepiece-wasm/pkg-web/bytepiece_bg.wasm') });
const node = require('../bytepiece-wasm/pkg-nodejs/bytepiece.js');
const enc = new TextEncoder();
const model = {};
for (let b = 0; b < 256; b++) model[Buffer.from([b]).toString('base64')] = [b + 3, '', 1];
model[Buffer.from('ab').toString('base64')] = [259, '', 100];
const bytes = enc.encode(JSON.stringify(model));
for (const [name, { Tokenizer }] of [['web', web], ['nodejs', node]]) {
    const tk = new Tokenizer(bytes);
    assert.equal(tk.vocabSize, 260);
    assert.deepEqual([...tk.encode('ab')], [259]);
    assert.deepEqual([...tk.encode('ab', true, true)], [1, 259, 2]);
    for (const text of ['', 'ab\n\nab', '你好🚀', 'e\u0301', '\0\r\n']) {
        assert.equal(tk.decode(tk.encode(text)), text.normalize('NFC'));
        assert.deepEqual(Buffer.concat(tk.tokenize(text)), Buffer.from(text.normalize('NFC')));
    }
    const binary = Uint8Array.from({ length: 256 }, (_, i) => i);
    assert.deepEqual(tk.decodeBytes(tk.encodeBytes(binary)), binary);
    assert.deepEqual(Buffer.concat(tk.tokenizeBytes(binary)), Buffer.from(binary));
    assert.equal(tk.decode(tk.encodeBytes(Uint8Array.of(97, 255, 98))), 'ab');
    const variations = new Set();
    for (let i = 0; i < 500; i++) {
        const ids = tk.encode('ab', false, false, 0);
        assert.equal(tk.decode(ids), 'ab');
        variations.add([...ids].join(','));
    }
    assert.equal(variations.size, 2);
    assert.throws(() => tk.decode(Uint32Array.of(9999)), /unknown token ID/);
    assert.throws(() => new Tokenizer(enc.encode('{}')), /invalid model/);
    assert.throws(() => new Tokenizer(enc.encode('invalid JSON')));
    tk.free();
    if (process.argv.includes('--upstream')) {
        const real = new Tokenizer(readFileSync('models/bytepiece_80k.model'));
        const cases = JSON.parse(readFileSync('target/upstream-cases.json', 'utf8'));
        for (const { text, ids } of cases) {
            assert.deepEqual([...real.encode(text)], ids, text);
            assert.equal(real.decode(Uint32Array.from(ids)), text.normalize('NFC'));
            assert.deepEqual(Buffer.concat(real.tokenize(text)), Buffer.from(text.normalize('NFC')));
        }
        real.free();
        console.log(`${name}: matched ${cases.length} upstream cases`);
    }
    console.log(`${name}: Wasm runtime checks passed`);
}
