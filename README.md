# autumn-plugin-three

This plugin adds [Three.js](https://threejs.org) 3D scenes to
[Autumn](https://autumn-web.app) apps. It works with Maud and htmx. It does
not use npm, a bundler, or inline script. You write a scene in Rust or as
HTML attributes. Scenes work in htmx partials.

- The crate contains Three.js **0.185.1** (MIT license). A `sha384` hash
  locks each vendored file.
- The crate needs `autumn-web` **0.8** and Rust **1.88** or later.

## Quickstart

Add the plugin:

```rust
use autumn_plugin_three::ThreePlugin;

autumn_web::app()
    .plugin(ThreePlugin::new())
    .run()
    .await;
```

Put the tags in the layout `<head>`. Put them before your own module
scripts:

```rust
use autumn_plugin_three::{three_script, three_stylesheet};

html! {
    head {
        (three_stylesheet())
        (three_script())
    }
}
```

Write a scene:

```rust
use autumn_plugin_three::{Color, Controls, Environment, Mesh, Scene};

html! {
    (Scene::new()
        .label("An orange torus knot")
        .environment(Environment::Room)
        .controls(Controls::OrbitNoZoom)
        .turntable(20.0)
        .add(Mesh::torus_knot(0.8, 0.25).color(Color::hex(0xff7a18)).metalness(0.6)))
}
```

## Model viewer

```rust
use autumn_plugin_three::{Controls, Environment, Model, Scene};

(Scene::new()
    .environment(Environment::Room)
    .controls(Controls::Orbit)
    .add(Model::gltf("/static/models/chair.glb").fit(2.0).play_all())
    .fallback(html! { img src="/static/chair.jpg" alt="A chair"; }))
```

`fit(size)` scales the model so that its largest side is `size`. It also
puts the model center at the model position. `play_all()` or
`play("Walk")` plays animation clips in a loop.

## htmx

The runtime scans the page after each htmx swap. When htmx or a script
removes a scene element, the runtime disposes the scene. This releases the
GPU memory and the WebGL context.

```rust
#[get("/shape")]
async fn shape() -> Markup {
    html! { (Scene::new().add(Mesh::dodecahedron(0.9).spin([0.0, 60.0, 0.0]))) }
}
```

```html
<button hx-get="/shape" hx-target="#gallery">Next shape</button>
```

When a swap changes the declarations inside a scene element (for example,
`hx-target` is the scene), the runtime builds the scene again.

## Your own JavaScript

Listen for `three:ready`. The event detail is the scene handle. Add the
listener in a module or `defer` script, so that it runs before
`DOMContentLoaded`. A script that runs later can read `element.autumnThree`.

```js
document.addEventListener("three:ready", (event) => {
  const { THREE, scene, camera, renderer, root, requestRender } = event.detail;
  // Add objects, raycast, or change materials. Then:
  requestRender();
});
```

Import Three.js only from the plugin URL. Then your code and the plugin
use one Three.js instance:

```js
import * as THREE from "/static/_plugins/three/three.module.min.js";
```

| Handle field | Meaning |
|---|---|
| `THREE` | The Three.js module. |
| `scene`, `camera`, `renderer` | The scene objects. |
| `root` | The group that holds meshes and models. The turntable turns it. |
| `controls` | `OrbitControls`, or `null`. |
| `models`, `mixers` | Model groups and animation mixers. |
| `render()` | Renders one frame now. |
| `requestRender()` | Renders one frame on the next animation frame. |
| `update()` | Checks for motion again. Call it after you add a mixer. |
| `looping` | `true` while the animation loop runs. |

## Attribute reference

The builder writes these attributes. You can also write them by hand.
Angles use degrees. Speeds use degrees per second. Colors use `#rrggbb` or
`#rgb`.

If a value is not valid, the runtime uses the default. The runtime clamps
numbers to their range. If a mesh kind, light kind, or model URL is not
valid, the runtime ignores the object and logs a warning.

### Scene (`data-three="scene"`)

| Attribute | Values | Default |
|---|---|---|
| `data-three-aspect` | `16/9`, `4/3`, `1/1`, `21/9`, `3/2`, `3/4`, or `w/h` (clamped to `0.1`–`10`) | `16/9` |
| `data-three-background` | color | transparent |
| `data-three-camera` | camera position `x,y,z` | `0,1,4` |
| `data-three-target` | look-at point `x,y,z` | `0,0,0` |
| `data-three-fov` | degrees, `1`–`179` | `50` |
| `data-three-controls` | `none`, `orbit`, `orbit-no-zoom` | `none` |
| `data-three-environment` | `room` | none |
| `data-three-turntable` | degrees per second around Y | `0` |
| `data-three-reduced` | `animate`: keep motion for reduced-motion users | — |

Put fallback content in a child `<div data-three-fallback>`. The browser
shows it when JavaScript or WebGL is not available, and when a model does
not load.

### Mesh (`data-three-mesh`)

| Attribute | Values | Default |
|---|---|---|
| `data-three-mesh` | `box`, `sphere`, `plane`, `torus`, `torus-knot`, `cylinder`, `cone`, `capsule`, `icosahedron`, `dodecahedron`, `octahedron`, `tetrahedron`, `ring` | — |
| `data-three-args` | sizes, comma-separated (see below) | per kind |
| `data-three-material` | `standard`, `physical`, `basic`, `lambert`, `phong`, `normal` | `standard` |
| `data-three-color` | color | `#ffffff` |
| `data-three-emissive` | color | `#000000` |
| `data-three-metalness` | `0`–`1` | `0` |
| `data-three-roughness` | `0`–`1` | `0.5` |
| `data-three-opacity` | `0`–`1` | `1` |
| `data-three-wireframe` | `true` | `false` |

Sizes: `box` w,h,d (`1,1,1`) · `sphere` r (`0.5`) · `plane` w,h (`1,1`) ·
`torus` r,tube (`0.5,0.2`) · `torus-knot` r,tube (`0.5,0.15`) · `cylinder`
top,bottom,h (`0.5,0.5,1`) · `cone` r,h (`0.5,1`) · `capsule` r,length
(`0.3,0.6`) · polyhedra r (`0.5`) · `ring` inner,outer (`0.25,0.5`).

### Model (`data-three-model`)

| Attribute | Values | Default |
|---|---|---|
| `data-three-model` | `.glb` or `.gltf` URL (`http(s)` or relative) | — |
| `data-three-fit` | largest side after scaling | no fit |
| `data-three-clip` | `*` (all clips) or a clip name | none |

### Mesh and model transform

| Attribute | Values | Default |
|---|---|---|
| `data-three-position` | `x,y,z` | `0,0,0` |
| `data-three-rotation` | degrees `x,y,z` | `0,0,0` |
| `data-three-scale` | `x,y,z` or one number | `1,1,1` |
| `data-three-spin` | degrees per second `x,y,z` | `0,0,0` |

### Light (`data-three-light`)

| Attribute | Values | Default |
|---|---|---|
| `data-three-light` | `ambient`, `directional`, `point`, `spot`, `hemisphere` | — |
| `data-three-color` | color (the sky color for `hemisphere`) | `#ffffff` |
| `data-three-ground` | color, `hemisphere` only | `#444444` |
| `data-three-intensity` | `0` or more | per kind |
| `data-three-position` | `x,y,z`. For `hemisphere`, the sky direction. | per kind |

If a scene has no light and no environment, the runtime adds a hemisphere
light and a directional light.

### Events and states

| Name | Meaning |
|---|---|
| `three:ready` | The scene is ready. `detail` is the handle. The event bubbles. |
| `three:error` | WebGL, the context, or a model failed. `detail` is `{ error, src }`. The event bubbles. |
| `data-three-state` | `loading`, `ready`, `error`, or `disposed`. The runtime sets it. |

```mermaid
stateDiagram-v2
    [*] --> loading: scan (load, htmx swap, DOM insert)
    loading --> ready: built
    loading --> error: no WebGL, or a model failed
    ready --> error: WebGL context lost
    ready --> disposed: element removed
    ready --> loading: declarations changed
    error --> loading: element inserted again, or declarations changed
    disposed --> loading: element inserted again
```

A scene in the `error` state stays in that state. Other swaps do not
build it again.

## Behavior

- **Reduced motion.** When the user prefers reduced motion, spin,
  turntable, and clips stop. Orbit controls continue to work. To keep
  motion in a scene, use `animate_reduced_motion()`.
- **Performance.** The animation loop runs only while the scene is visible
  and has motion. A static scene renders only when it changes. The runtime
  limits the pixel ratio to 2.
- **Accessibility.** `label("…")` sets `role="img"` and `aria-label`. The
  canvas has `aria-hidden="true"`.

## Security and CSP

- The plugin works with the default Autumn CSP and in nonce mode
  (`[security.headers.csp_nonce] enabled = true`). It uses no inline
  script, no inline style, no import map, and no `eval`.
- Images inside a GLB work with the default CSP. Safari 16 and older, and
  Firefox 97 and older, load these images from `blob:` URLs. For textured
  models in these browsers, add `blob:` to `img-src`.
- Do not let user content keep `data-three-*` attributes. A sanitizer that
  keeps `data-*` attributes lets user markup start scenes and load model
  URLs.
- SRI covers `init.js` and the core modules (`three_script()` preloads
  them). The addons load from the same origin without SRI. See
  [ADR 0001](docs/adr/0001-esm-module-graph.md).

htmx adds an inline `<style>` for its indicators. Nonce mode blocks this
style. To stop the style, add this tag to the layout:

```rust
meta name="htmx-config" content=r#"{"includeIndicatorStyles":false}"#;
```

## How it works

- `assets/` holds the vendored files and the plugin files. `THREE_ASSETS`
  serves them under `/static/_plugins/three/`. The browser keeps hashed
  URLs in its cache. It checks plain URLs again at each page load.
- Three.js is a set of ES modules. Module imports use plain URLs, so the
  page, the addons, and your code use one Three.js instance. See
  [ADR 0001](docs/adr/0001-esm-module-graph.md).
- The addons import the bare specifier `three`. `scripts/vendor.sh` makes
  the imports relative. A test reverses the rewrites and checks the
  upstream hash. See [ADR 0002](docs/adr/0002-addon-import-rewrites.md).
- `parse.js` reads the attributes. `init.js` builds and disposes the
  scenes. See [ADR 0003](docs/adr/0003-declarative-scene-runtime.md).

## Demo

```sh
cargo run --example three_demo
# Open http://127.0.0.1:3000
```

## Tests

```sh
cargo test                                # Rust unit, property, and doc tests
npm ci && npm run test:unit               # parse.js (node --test)
npx playwright install chromium           # one time
cargo build --example e2e_fixture
npm run test:e2e                          # Chromium + WebGL (SwiftShader)
```

The E2E tests read real canvas pixels. They also run in CSP nonce mode.
They measure strict line coverage of `init.js` (minimum 95 %).

## Limits

- The plugin uses WebGL only. It does not support the WebGPU renderer.
- The declarative layer has no shadows, post-processing, or physics. Use
  `three:ready` for custom code.
- Each scene has its own WebGL context. Browsers keep about 16 contexts.
  When the browser drops a context, the scene shows its fallback.
- The plugin loads only glTF 2.0 and GLB files. It does not decode Draco,
  KTX2, or Meshopt compression.
- External model and image URLs need a CSP that allows them.
- The runtime does not watch attribute changes on a live scene. Change the
  declarations, or swap the scene.
- To upgrade Three.js, release a new plugin version. Use
  `scripts/vendor.sh`.

## License

Apache-2.0 for the plugin. Three.js is MIT: see `assets/THREE-LICENSE`.
