//! Three.js 3D scenes for Autumn, with Maud + htmx ergonomics.

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
