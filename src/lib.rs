//! Three.js 3D scenes for Autumn, with Maud + htmx ergonomics.
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
//! - The crate vendors [Three.js](https://threejs.org) 0.185.1 (MIT) and
//!   five addons. There is no npm and no bundler. [`THREE_ASSETS`] serves
//!   them under `/static/_plugins/three/` with SRI hashes.
//! - The builder renders `data-three-*` attributes. `init.js` reads them and
//!   builds the scene. You can also write the attributes by hand.
//! - `init.js` scans on load, on `htmx:afterSwap`, and on each DOM insertion.
//!   It disposes a scene when its element leaves the document.
//! - The `three:ready` event gives your own JavaScript the scene, camera,
//!   renderer, and the shared Three.js module.
//!
//! # Limits
//!
//! - WebGL only. No WebGPU renderer.
//! - No shadows, post-processing, or physics in the declarative layer. Use
//!   the `three:ready` event for custom code.
//! - Each scene has its own WebGL context. Browsers allow about 16 at a time.
//! - Models: glTF 2.0 / GLB only. No Draco, KTX2, or Meshopt compression.
//!   Images inside a GLB work under the default CSP. External model and
//!   image URLs need a CSP that allows them.

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
