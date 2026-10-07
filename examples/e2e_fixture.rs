//! Test fixture app for the browser E2E tests in `tests/e2e/`.
//!
//! Each route is one scenario. Run it with:
//!
//! ```sh
//! AUTUMN_SERVER__PORT=3111 cargo run --example e2e_fixture
//! ```
//!
//! Not a demo. See `examples/three_demo.rs` for the demo.

use autumn_plugin_three::{
    Aspect, Camera, Color, Controls, Environment, Light, Material, Mesh, Model, Scene,
    THREE_ASSETS, ThreePlugin, three_script, three_stylesheet,
};
use autumn_web::assets::asset_url;
use autumn_web::{Markup, html};

/// The crate `static/` dir: the fixture model.
static STATIC: autumn_web::include_dir::Dir = autumn_web::embed_static!();

#[autumn_web::main]
async fn main() {
    autumn_web::app()
        .plugin(ThreePlugin::new())
        .embedded_static(&STATIC)
        .routes(autumn_web::routes![
            basic,
            spin,
            spin_animate,
            model,
            model_missing,
            swap,
            swap_next,
            offscreen,
            orbit,
            aspect,
            room,
            handwritten,
            kinds,
            model_no_fallback,
            model_clip_missing,
            late_script,
            textured,
            textured_external,
            params,
            broken_texture,
        ])
        .run()
        .await;
}

/// The page shell.
fn page(content: &Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "three e2e" }
                // htmx adds an inline <style> for indicators. CSP nonce mode
                // blocks it, so turn it off.
                meta name="htmx-config" content=r#"{"includeIndicatorStyles":false}"#;
                link rel="stylesheet" href=(asset_url("css/fixture.css"));
                (three_stylesheet())
                (three_script())
                script src=(asset_url("js/htmx.min.js")) defer {}
                script type="module" src=(asset_url("js/fixture-listener.js")) {}
            }
            body { (content) }
        }
    }
}

/// An unlit red cube that fills the view center. Pixel tests read it.
const fn red_cube() -> Mesh {
    Mesh::cube(2.0)
        .material(Material::Basic)
        .color(Color::hex(0x00ff_0000))
}

/// The fallback text.
fn fallback() -> Markup {
    html! { p class="fallback" { "No 3D here." } }
}

#[autumn_web::get("/basic")]
async fn basic() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .label("A red cube")
            .background(Color::hex(0x0000_00ff))
            .camera(Camera::perspective().position([0.0, 0.0, 3.0]))
            .add(red_cube())
            .fallback(fallback()))
    })
}

#[autumn_web::get("/spin")]
async fn spin() -> Markup {
    page(&html! {
        (Scene::new().id("scene").turntable(90.0).add(red_cube().spin([0.0, 180.0, 0.0])))
    })
}

#[autumn_web::get("/spin-animate")]
async fn spin_animate() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .animate_reduced_motion()
            .add(red_cube().spin([0.0, 180.0, 0.0])))
    })
}

#[autumn_web::get("/model")]
async fn model() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .add(Model::gltf("/static/models/gem.glb").fit(2.0))
            .add(Model::gltf("/static/models/gem.glb").fit(1.0).play("Spin").position([3.0, 0.0, 0.0]))
            .add(Model::gltf("/static/models/gem.glb").fit(1.0).play_all().position([-3.0, 0.0, 0.0]))
            .fallback(fallback()))
    })
}

#[autumn_web::get("/model-missing")]
async fn model_missing() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .add(red_cube())
            .add(Model::gltf("/static/models/missing.glb"))
            .fallback(fallback()))
    })
}

#[autumn_web::get("/swap")]
async fn swap() -> Markup {
    page(&html! {
        button id="next" hx-get="/swap/next" hx-target="#slot" hx-swap="innerHTML" { "Next" }
        div id="slot" {
            (Scene::new().id("first").add(red_cube().spin([0.0, 90.0, 0.0])))
        }
    })
}

#[autumn_web::get("/swap/next")]
async fn swap_next() -> Markup {
    html! {
        (Scene::new()
            .id("second")
            .add(Mesh::sphere(1.0).material(Material::Normal).spin([0.0, 90.0, 0.0])))
    }
}

#[autumn_web::get("/offscreen")]
async fn offscreen() -> Markup {
    page(&html! {
        (Scene::new().id("still").add(red_cube()))
        div id="spacer" {}
        (Scene::new().id("below").add(red_cube().spin([0.0, 90.0, 0.0])))
    })
}

#[autumn_web::get("/orbit")]
async fn orbit() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .controls(Controls::Orbit)
            .camera(Camera::perspective().position([0.0, 0.0, 5.0]))
            .add(red_cube()))
    })
}

#[autumn_web::get("/aspect")]
async fn aspect() -> Markup {
    page(&html! {
        (Scene::new().id("square").aspect(Aspect::Square).add(red_cube()))
        (Scene::new().id("custom").aspect(Aspect::Ratio(2.0, 1.0)).add(red_cube()))
    })
}

#[autumn_web::get("/room")]
async fn room() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .environment(Environment::Room)
            .add(Mesh::torus_knot(0.8, 0.25).metalness(1.0).roughness(0.2)))
        (Scene::new()
            .id("lit")
            .add(Mesh::sphere(1.0))
            .add(Light::ambient().intensity(0.5))
            .add(Light::point().position([2.0, 2.0, 2.0])))
        (Scene::new().id("default-lights").add(Mesh::sphere(1.0)))
    })
}

#[autumn_web::get("/handwritten")]
async fn handwritten() -> Markup {
    page(&html! {
        div id="scene" data-three="scene" data-three-camera="0,0,3" data-three-fov="bad" {
            div hidden data-three-mesh="box" data-three-args="2,2,2"
                data-three-material="basic" data-three-color="#00ff00" {}
            div hidden data-three-mesh="teapot" {}
            div hidden data-three-light="laser" {}
            div hidden data-three-model="javascript:alert(1)" {}
        }
    })
}

#[autumn_web::get("/kinds")]
async fn kinds() -> Markup {
    let materials = [
        Material::Standard,
        Material::Physical,
        Material::Basic,
        Material::Lambert,
        Material::Phong,
        Material::Normal,
    ];
    let meshes = [
        Mesh::cube(0.5),
        Mesh::sphere(0.3),
        Mesh::plane(0.5, 0.5),
        Mesh::torus(0.3, 0.1),
        Mesh::torus_knot(0.3, 0.1),
        Mesh::cylinder(0.2, 0.3, 0.5),
        Mesh::cone(0.3, 0.5),
        Mesh::capsule(0.2, 0.3),
        Mesh::icosahedron(0.3),
        Mesh::dodecahedron(0.3),
        Mesh::octahedron(0.3),
        Mesh::tetrahedron(0.3),
        Mesh::ring(0.1, 0.3).opacity(0.5).wireframe(),
    ];
    let mut scene = Scene::new().id("scene");
    for (index, mesh) in meshes.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let x = index as f32 - 6.0;
        scene = scene.add(
            mesh.material(materials[index % materials.len()])
                .position([x, 0.0, 0.0]),
        );
    }
    page(&html! {
        (scene
            .add(Light::ambient())
            .add(Light::directional())
            .add(Light::point())
            .add(Light::spot())
            .add(Light::hemisphere()))
    })
}

#[autumn_web::get("/model-no-fallback")]
async fn model_no_fallback() -> Markup {
    page(&html! {
        (Scene::new().id("scene").add(red_cube()).add(Model::gltf("/static/models/missing.glb")))
    })
}

#[autumn_web::get("/model-clip-missing")]
async fn model_clip_missing() -> Markup {
    page(&html! {
        (Scene::new().id("scene").add(Model::gltf("/static/models/gem.glb").play("Nope")))
    })
}

/// A page without `three_script()`. The test adds the entry script later.
#[autumn_web::get("/late-script")]
async fn late_script() -> Markup {
    let init = THREE_ASSETS.url("init.js");
    html! {
        (maud::DOCTYPE)
        html {
            head { (three_stylesheet()) }
            body data-init=(init) { (Scene::new().id("scene").add(red_cube())) }
        }
    }
}

#[autumn_web::get("/textured")]
async fn textured() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .camera(Camera::perspective().position([0.0, 0.0, 1.2]))
            .add(Model::gltf("/static/models/tile.glb")))
    })
}

#[autumn_web::get("/textured-external")]
async fn textured_external() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .camera(Camera::perspective().position([0.0, 0.0, 1.2]))
            .add(Model::gltf("/static/models/tile-ext.gltf")))
    })
}

/// Every scene, mesh, and light parameter, for value checks.
#[autumn_web::get("/params")]
async fn params() -> Markup {
    page(&html! {
        (Scene::new()
            .id("scene")
            .camera(
                Camera::perspective()
                    .fov(30.0)
                    .position([0.0, 0.0, 5.0])
                    .target([1.0, 0.0, 0.0]),
            )
            .controls(Controls::OrbitNoZoom)
            .turntable(-90.0)
            .add(Mesh::cube(0.5).rotation([90.0, 0.0, 0.0]).position([-1.0, 0.0, 0.0]))
            .add(
                Mesh::cube(0.5)
                    .emissive(Color::hex(0x0000_ff00))
                    .spin([30.0, 0.0, 60.0])
                    .position([1.0, 0.0, 0.0]),
            )
            .add(
                Light::directional()
                    .color(Color::hex(0x00ff_0000))
                    .intensity(3.0)
                    .position([1.0, 2.0, 3.0]),
            )
            .add(
                Light::hemisphere()
                    .color(Color::hex(0x0000_00ff))
                    .ground(Color::hex(0x0000_ff00))
                    .intensity(0.5),
            ))
    })
}

#[autumn_web::get("/broken-texture")]
async fn broken_texture() -> Markup {
    page(&html! {
        (Scene::new().id("scene").add(Model::gltf("/static/models/broken.glb")))
    })
}
