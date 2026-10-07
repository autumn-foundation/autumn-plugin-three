//! This crate adds Three.js 3D scenes to Autumn apps. It works with Maud
//! and htmx.
//!
//! Add [`ThreePlugin`] to the app. Put [`three_stylesheet()`] and
//! [`three_script()`] in the page `<head>`. Then write a [`Scene`]:
//!
//! ```rust,no_run
//! use autumn_plugin_three::{
//!     Color, Controls, Environment, Mesh, Scene, ThreePlugin, three_script, three_stylesheet,
//! };
//! use autumn_web::prelude::*;
//!
//! #[get("/")]
//! async fn index() -> Markup {
//!     html! {
//!         html {
//!             head { (three_stylesheet()) (three_script()) }
//!             body {
//!                 (Scene::new()
//!                     .label("An orange torus knot")
//!                     .environment(Environment::Room)
//!                     .controls(Controls::Orbit)
//!                     .turntable(20.0)
//!                     .add(Mesh::torus_knot(0.8, 0.25).color(Color::hex(0xff7a18)).metalness(0.6)))
//!             }
//!         }
//!     }
//! }
//!
//! # async fn run() {
//! autumn_web::app()
//!     .plugin(ThreePlugin::new())
//!     .routes(routes![index])
//!     .run()
//!     .await;
//! # }
//! ```
//!
//! # How it works
//!
//! - The crate contains [Three.js](https://threejs.org) 0.185.1 (MIT) and
//!   five addons. It does not use npm or a bundler. [`THREE_ASSETS`] serves
//!   them under `/static/_plugins/three/`. SRI covers `init.js` and the
//!   core modules. The addons load from the same origin without SRI.
//! - The builder renders `data-three-*` attributes. `init.js` reads them and
//!   builds the scene. You can also write the attributes by hand.
//! - `init.js` scans on load, on `htmx:afterSwap`, and on each DOM insertion.
//!   It disposes a scene when its element leaves the document.
//! - The `three:ready` event gives your own JavaScript the scene, camera,
//!   renderer, and the shared Three.js module.
//!
//! # Limits
//!
//! - The plugin uses WebGL only. It does not support the WebGPU renderer.
//! - The declarative layer has no shadows, post-processing, or physics. Use
//!   the `three:ready` event for custom code.
//! - Each scene has its own WebGL context. Browsers keep about 16 contexts.
//!   When the browser drops a context, the scene shows its fallback.
//! - The plugin loads only glTF 2.0 and GLB files. It does not decode
//!   Draco, KTX2, or Meshopt compression. Images inside a GLB work with the
//!   default CSP. External model and image URLs need a CSP that allows them.
//! - Do not let user content keep `data-three-*` attributes. User markup
//!   could then start scenes and load model URLs.

mod assets;
mod plugin;
mod scene;
mod script;

pub use assets::{
    ASSETS_NAMESPACE, THREE_ASSETS, THREE_MODULE_URL, THREE_VERSION, VENDORED, VendoredFile,
};
pub use plugin::{PLUGIN_NAME, ThreePlugin};
pub use scene::{
    Aspect, Camera, Clip, Color, Controls, Environment, Geometry, Light, LightKind, Material, Mesh,
    Model, Scene, SceneObject, Vec3,
};
pub use script::{three_script, three_stylesheet};
