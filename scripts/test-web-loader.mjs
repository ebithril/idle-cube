import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

// Exercise the actual bundled import with the matrix counts used by the game.
const bundle = readFileSync(new URL('../web/mq_js_bundle.js', import.meta.url), 'utf8');
const source = bundle.match(/glUniformMatrix4fv:(function\([^)]*\)\{[^}]*\})/);
assert.ok(source, 'Matrix upload import must exist in the bundle');

for (const count of [1, 3]) {
    const offset = 32;
    const buffer = new ArrayBuffer(offset + count * 16 * 4);
    const matrices = new Float32Array(buffer, offset);
    matrices.set(Array.from({ length: count * 16 }, (_, i) => i + 1));
    let uploaded;
    const location = {};
    const upload = runInNewContext(`(${source[1]})`, {
        GL: { uniforms: [location], validateGLObjectID() {} },
        assert,
        Float32Array,
        getArray: (pointer, type, length) => new type(buffer, pointer, length),
        gl: {
            uniformMatrix4fv(target, transpose, data) {
                assert.equal(target, location);
                assert.equal(transpose, false);
                uploaded = Array.from(data);
            },
        },
    });
    upload(0, count, 0, offset);
    assert.deepEqual(uploaded, Array.from(matrices), `Upload all ${count} matrices`);
}
console.log('Web loader matrix uploads pass (single matrix and three-matrix arrays).');
