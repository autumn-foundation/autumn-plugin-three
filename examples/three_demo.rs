//! Three.js demo: a small Autumn app that shows `autumn-plugin-three`.
//!
//! ```sh
//! cargo run --example three_demo
//! ```
//!
//! Open <http://127.0.0.1:3000>. The page shows:
//!
//! - a hero torus knot with studio light, a turntable, and orbit controls,
//! - a GLB model viewer (auto-fit, animation clip),
//! - an htmx gallery: each click swaps in a new scene from the server, and
//!   the old scene is disposed,
//! - a scene written as raw `data-three-*` attributes,
//! - custom JavaScript that uses the `three:ready` event.
//!
//! No inline script and no inline style: all JS and CSS are external files,
//! so the default Autumn CSP and nonce mode both work.

use std::sync::atomic::{AtomicUsize, Ordering};

use autumn_plugin_three::{
    Aspect, Camera, Color, Controls, Environment, Light, Material, Mesh, Model, Scene, ThreePlugin,
    three_script, three_stylesheet,
};
use autumn_web::assets::asset_url;
use autumn_web::{Markup, html};

/// The crate `static/` dir: demo CSS, demo JS, and the model.
static STATIC: autumn_web::include_dir::Dir = autumn_web::embed_static!();

/// Gallery position for the `/shape` partial.
static SHAPE: AtomicUsize = AtomicUsize::new(0);

#[autumn_web::main]
async fn main() {
    autumn_web::app()
        .plugin(ThreePlugin::new())
        .embedded_static(&STATIC)
        .routes(autumn_web::routes![index, shape])
        .run()
        .await;
}

/// The page shell.
fn layout(content: &Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                // htmx adds an inline <style> for indicators. CSP nonce mode
                // blocks it, so turn it off.
                meta name="htmx-config" content=r#"{"includeIndicatorStyles":false}"#;
                title { "Three.js demo" }
                link rel="stylesheet" href=(asset_url("css/demo.css"));
                (three_stylesheet())
                (three_script())
                script src=(asset_url("js/htmx.min.js")) defer {}
                script type="module" src=(asset_url("js/demo.js")) {}
            }
            body { main { (content) } }
        }
    }
}

#[autumn_web::get("/")]
async fn index() -> Markup {
    layout(&html! {
        section class="hero" {
            div {
                p class="kicker" { "autumn-plugin-three" }
                h1 { "3D in server-rendered HTML." }
                p { "Each scene on this page is Maud markup. Drag the knot." }
            }
            (Scene::new()
                .class("hero-scene")
                .label("An orange metal torus knot that turns slowly")
                .aspect(Aspect::Square)
                .environment(Environment::Room)
                .controls(Controls::OrbitNoZoom)
                .camera(Camera::perspective().position([0.0, 0.0, 4.0]))
                .turntable(20.0)
                .add(Mesh::torus_knot(0.8, 0.26).color(Color::hex(0x00ff_7a18)).metalness(0.6).roughness(0.25))
                .fallback(html! { p class="fallback" { "This 3D scene needs WebGL." } }))
        }

        section class="card" {
            h2 { "Model viewer" }
            p { "A GLB file, scaled to fit, playing its " code { "Spin" } " clip." }
            (Scene::new()
                .label("An orange box that spins")
                .environment(Environment::Room)
                .controls(Controls::Orbit)
                .background(Color::hex(0x0016_1a22))
                .camera(Camera::perspective().position([2.2, 1.6, 3.2]))
                .add(Model::gltf(asset_url("models/gem.glb")).fit(1.6).play("Spin"))
                .fallback(html! { p class="fallback" { "The model did not load." } }))
        }

        section class="card" {
            h2 { "htmx gallery" }
            p { "Each click gets a new scene from the server. The old scene frees its GPU memory." }
            button hx-get="/shape" hx-target="#gallery" hx-swap="innerHTML" { "Next shape" }
            div id="gallery" { (gallery_scene(0)) }
        }

        section class="card" {
            h2 { "Raw attributes" }
            p { "No builder: plain " code { "data-three-*" } " attributes." }
            div data-three="scene" data-three-aspect="21/9" data-three-camera="0,1.5,5" {
                div hidden data-three-mesh="plane" data-three-args="12,12" data-three-rotation="-90,0,0"
                    data-three-position="0,-1,0" data-three-color="#2a3140" {}
                div hidden data-three-mesh="icosahedron" data-three-args="0.8"
                    data-three-color="#5ad1ff" data-three-spin="20,40,0" data-three-material="phong" {}
                div hidden data-three-light="hemisphere" data-three-intensity="1.5" {}
                div hidden data-three-light="spot" data-three-position="0,4,2" {}
            }
        }

        section class="card" {
            h2 { "Your own JavaScript" }
            p { "Click the cube. " code { "static/js/demo.js" } " listens for " code { "three:ready" } "." }
            (Scene::new()
                .id("custom")
                .aspect(Aspect::Ultrawide)
                .add(Mesh::cube(1.2).color(Color::hex(0x008b_5cf6)).spin([15.0, 30.0, 0.0]))
                .add(Light::ambient().intensity(0.6))
                .add(Light::directional().position([2.0, 3.0, 4.0])))
        }
    })
}

/// One gallery scene. `index` selects the shape and color.
fn gallery_scene(index: usize) -> Markup {
    let shapes = [
        Mesh::torus(0.7, 0.25),
        Mesh::dodecahedron(0.9),
        Mesh::capsule(0.45, 0.8),
        Mesh::cone(0.7, 1.4),
        Mesh::cylinder(0.6, 0.6, 1.2),
        Mesh::sphere(0.9).material(Material::Normal),
        Mesh::ring(0.4, 0.9),
    ];
    let colors = [
        0x00f4_7266,
        0x00fb_bf24,
        0x0034_d399,
        0x0060_a5fa,
        0x00c0_84fc,
    ];
    let mesh = shapes[index % shapes.len()]
        .clone()
        .color(Color::hex(colors[index % colors.len()]))
        .spin([20.0, 60.0, 0.0]);
    html! {
        (Scene::new()
            .label("A spinning shape")
            .aspect(Aspect::Wide)
            .camera(Camera::perspective().position([0.0, 0.0, 3.5]))
            .add(mesh))
    }
}

#[autumn_web::get("/shape")]
async fn shape() -> Markup {
    gallery_scene(SHAPE.fetch_add(1, Ordering::Relaxed) + 1)
}
