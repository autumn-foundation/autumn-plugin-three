# Plan: autumn-plugin-three 0.1.0

Target: Autumn `autumn-web` 0.8.0. Prior art: `autumn-plugin-motion` 0.2.0.
Style: ASD-STE100. Method: SPEC → RED → GREEN → REFACTOR.

## 1. Goal

Give Autumn apps 3D scenes with Three.js. Do not use npm, a bundler, or
inline script. Write the scene in Rust or in HTML attributes. Keep the
scenes correct across htmx swaps.

## 2. Brainstorming

Ideas (all ideas first, no filter):

1. Vendor Three.js in the crate and serve it with `PluginAssets`.
2. Typed Rust builder: `Scene`, `Mesh`, `Model`, `Light`, `Camera`.
3. Declarative `data-three-*` attributes, one interpreter (`init.js`).
4. Child declaration elements for objects, so htmx can render objects.
5. JSON scene description in one attribute.
6. glTF/GLB model viewer with auto-fit and animation clips.
7. Orbit controls, turntable, per-object spin.
8. `RoomEnvironment` image-based light for good PBR with no setup.
9. `three:ready` / `three:error` DOM events, so custom JS can use the scene.
10. Re-scan on `htmx:afterSwap`; dispose on `htmx:beforeCleanupElement`.
11. Render only when visible; pause when off screen.
12. Fallback content when WebGL or JS is not available.
13. Reduced-motion guard, as in the motion plugin.
14. Import map for `three` (bare specifier).
15. Rewrite addon imports to relative paths at vendor time.
16. WebGPU renderer.
17. Shadows, post-processing, physics.
18. Headless Chromium tests that check real WebGL pixels.

Selected: 1–4, 6–13, 15, 18. Rejected:

- 5: hard to write by hand; children are htmx-friendly.
- 14: an import map is an inline script. The default CSP blocks it.
- 16, 17: large scope. Record as limits.

## 3. Reverse brainstorming

Question: "How can we make this plugin fail?" Then invert each answer.

| Way to fail | Prevention |
|---|---|
| Inline `<script>` or import map blocked by CSP. | External module files only. Addon imports rewritten to relative URLs. |
| Two copies of Three.js load (hashed URL and plain URL). | All module imports use plain URLs. Only the entry uses a hashed URL. |
| Vendored bytes drift from upstream. | `sha384` pins. A test reverses the import rewrites and checks the upstream hash. |
| htmx swaps leak WebGL contexts (browser limit ≈16). | Dispose on `htmx:beforeCleanupElement` and when the element is disconnected. |
| A scene initializes two times. | A `Map` from element to state. |
| `NaN` or `Infinity` in attributes breaks the scene. | Rust: never emit non-finite numbers. JS: reject non-finite input. |
| Bad attribute value throws and stops all scenes. | Parse defensively. Catch per scene. Show fallback. |
| Model URL fails to load. | `three:error` event. With a fallback: show it and free the GPU. Without: keep the other objects. |
| GLB textures load from `blob:` URLs; `connect-src 'self'` blocks them. | GLTFLoader plugin decodes embedded images with `createImageBitmap(blob)`. |
| A page module registers `three:ready` after the first scan. | First scan waits for `DOMContentLoaded`. |
| Scene has zero height. | Default `aspect-ratio: 16 / 9` in `three.css`. |
| Animation loop runs off screen and uses battery power. | `IntersectionObserver` stops the loop. Render on demand when static. |
| High DPI screens render 9× pixels. | Pixel ratio capped at 2. |
| Reduced-motion users see motion. | No automatic motion unless `data-three-reduced="animate"`. |
| Tests only check strings; the scene does not render. | E2E test in headless Chromium reads canvas pixels. |
| User's own module imports a second Three.js. | Document the one module URL to import. |

## 4. Six thinking hats

- **White (facts).** Three.js 0.185.1 is the newest release with
  upstream-minified `three.core.min.js` and `three.module.min.js`
  (0.186.x ships unminified files only). The builds are ES modules.
  Addons import the bare specifier `three`. Autumn 0.8 serves
  `PluginAssets` with hashed URLs, SRI, ETag, and Range. Default CSP:
  `script-src 'self'`, `img-src 'self' data:`, `connect-src 'self'`.
  Compression is off by default. We do not use Verus. The crate has no
  `unsafe` code and no Rust state machine. Proptests cover the builder
  invariants. E2E tests cover the JS lifecycle.
- **Red (feelings).** Users want "a 3D thing on my page in five lines".
  A spinning model in a hero section must feel easy.
- **Black (risks).** ESM graph and SRI: dynamic imports have no
  `integrity`. Same-origin bytes from the binary lower this risk. A
  `modulepreload` with SRI covers the core graph. GLB textures use `blob:`
  URLs; the default `connect-src` blocks them (fixed in `init.js`).
- **Yellow (benefits).** One crate, no build step. Typed API catches
  errors at compile time. Scenes work in htmx partials. SRI on all entry
  tags.
- **Green (creative).** Child declaration elements. Auto-fit for models.
  `three:ready` gives full Three.js access. Turntable mode.
- **Blue (process).** Write spec (this file + ADRs). Then RED tests per
  slice, GREEN code, REFACTOR. Slices: assets → plugin → script tags →
  builder → JS parse → JS runtime → example → e2e → docs → review.

## 5. Spec (invariants)

1. The bundle holds exactly the served files. `manifest.json` is not served.
2. Each vendored upstream file matches its pinned `sha384`, after the
   rewrites are reversed.
3. Every module import in the bundle resolves to a file in the bundle.
4. `three_script()` preloads the module graph with SRI, then loads the
   entry as `type="module"` with SRI.
5. The builder never emits a non-finite number.
6. One scene element owns at most one renderer. Removal frees it.
7. Reduced motion: no automatic motion without opt-in.
8. A bad declaration never stops other scenes.

## 6. Slices

| # | Slice | RED test first |
|---|---|---|
| 1 | Asset bundle + manifest | `assets.rs` tests |
| 2 | `ThreePlugin` | serve, 404, routes, conformance |
| 3 | `three_script()` / `three_stylesheet()` | tag tests |
| 4 | Builder | unit + proptest |
| 5 | `parse.js` | `node --test` |
| 6 | `init.js` runtime | e2e (Chromium + WebGL) |
| 7 | Example + fixture GLB | e2e |
| 8 | CI, README, ADRs, CLAUDE.md | review |
