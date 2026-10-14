`mq_js_bundle.js` is vendored from macroquad 0.4.14's
`js/mq_js_bundle.js` (https://github.com/not-fl3/macroquad/tree/v0.4.14/js).
Macroquad is distributed under the MIT or Apache-2.0 license.
Update this file together with the pinned macroquad dependency.

Local compatibility fix: `glUniformMatrix4fv` reads `16 * count` floats rather
than only 16. The upstream 0.4.14 bundle ignores the matrix count, leaving the
second and third elements of the cube's rotation arrays at zero in WebGL.
When updating the bundle, retain this fix unless upstream has corrected it.
Run `node scripts/test-web-loader.mjs` to check matrix array uploads.
