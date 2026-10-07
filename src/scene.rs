//! The typed scene builder: [`Scene`], [`Mesh`], [`Model`], [`Light`].
//!
//! A [`Scene`] renders one `<div data-three="scene">` element. Each object
//! renders one hidden child declaration (`data-three-mesh`,
//! `data-three-model`, `data-three-light`). `init.js` reads the markup and
//! builds the Three.js scene. You can also write the markup by hand; the
//! README has the attribute reference.
//!
//! Units: lengths in Three.js units, angles in degrees, spin and turntable
//! speeds in degrees per second.
//!
//! The builder never emits a non-finite number. A setter ignores `NaN` and
//! infinite input. Out-of-range input is clamped.

use autumn_web::{Markup, html};
use maud::Render;

/// Returns `value` when it is finite, else `fallback`.
const fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

/// Returns `Some(value)` when it is finite, else `None`.
const fn finite(value: f32) -> Option<f32> {
    if value.is_finite() { Some(value) } else { None }
}

/// Formats a finite number for an attribute. `-0` becomes `0`.
fn num(value: f32) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        format!("{value}")
    }
}

/// A 3D vector: position, rotation (degrees), scale, or spin (degrees per
/// second).
///
/// ```rust
/// use autumn_plugin_three::Vec3;
///
/// assert_eq!(Vec3::from([1.0, 2.0, 3.0]), Vec3::new(1.0, 2.0, 3.0));
/// assert_eq!(Vec3::splat(2.0), Vec3::new(2.0, 2.0, 2.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

impl Vec3 {
    /// The zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Makes a vector.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Makes a vector with all three components set to `value`.
    #[must_use]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value)
    }

    /// Replaces each non-finite component with the same component of
    /// `fallback`.
    const fn finite_or(self, fallback: Self) -> Self {
        Self::new(
            finite_or(self.x, fallback.x),
            finite_or(self.y, fallback.y),
            finite_or(self.z, fallback.z),
        )
    }

    /// The `x,y,z` attribute value.
    fn attr(self) -> String {
        format!("{},{},{}", num(self.x), num(self.y), num(self.z))
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self::new(x, y, z)
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self::new(x, y, z)
    }
}

/// An sRGB color.
///
/// ```rust
/// use autumn_plugin_three::Color;
///
/// assert_eq!(Color::hex(0xff8800).to_string(), "#ff8800");
/// assert_eq!(Color::rgb(0, 128, 255).to_string(), "#0080ff");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color(u32);

impl Color {
    /// White (`#ffffff`).
    pub const WHITE: Self = Self(0x00ff_ffff);
    /// Black (`#000000`).
    pub const BLACK: Self = Self(0);

    /// Makes a color from `0xRRGGBB`. Bits above 24 are ignored.
    #[must_use]
    pub const fn hex(value: u32) -> Self {
        Self(value & 0x00ff_ffff)
    }

    /// Makes a color from red, green, and blue channels.
    #[must_use]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    /// The color as `0xRRGGBB`.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:06x}", self.0)
    }
}

/// The aspect ratio (width / height) of a scene.
///
/// The named ratios have CSS rules in `three.css`, so the size is correct
/// before JavaScript runs. `init.js` applies a [`Aspect::Ratio`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Aspect {
    /// 16 / 9 (default).
    Wide,
    /// 4 / 3.
    Standard,
    /// 1 / 1.
    Square,
    /// 21 / 9.
    Ultrawide,
    /// 3 / 2.
    Photo,
    /// 3 / 4.
    Portrait,
    /// A custom `width / height`. Both must be finite and positive, else
    /// the scene uses [`Aspect::Wide`].
    Ratio(f32, f32),
}

impl Aspect {
    /// The `data-three-aspect` value.
    fn attr(self) -> String {
        match self {
            Self::Wide => "16/9".to_owned(),
            Self::Standard => "4/3".to_owned(),
            Self::Square => "1/1".to_owned(),
            Self::Ultrawide => "21/9".to_owned(),
            Self::Photo => "3/2".to_owned(),
            Self::Portrait => "3/4".to_owned(),
            Self::Ratio(w, h) if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 => {
                format!("{}/{}", num(w), num(h))
            }
            Self::Ratio(..) => Self::Wide.attr(),
        }
    }
}

/// Camera controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Controls {
    /// No user control (default).
    None,
    /// Orbit: drag to rotate, wheel or pinch to zoom, right-drag to pan.
    Orbit,
    /// Orbit without zoom, so the mouse wheel scrolls the page.
    OrbitNoZoom,
}

impl Controls {
    /// The `data-three-controls` value.
    const fn attr(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Orbit => "orbit",
            Self::OrbitNoZoom => "orbit-no-zoom",
        }
    }
}

/// Image-based scene light.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Three.js `RoomEnvironment`: soft studio light. Good for PBR models.
    Room,
}

impl Environment {
    /// The `data-three-environment` value.
    const fn attr(self) -> &'static str {
        match self {
            Self::Room => "room",
        }
    }
}

/// A perspective camera.
///
/// Defaults (applied by `init.js`): field of view `50`, position `0,1,4`,
/// target `0,0,0`.
///
/// ```rust
/// use autumn_plugin_three::Camera;
///
/// let camera = Camera::perspective().fov(40.0).position([0.0, 2.0, 6.0]);
/// # let _ = camera;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[must_use]
pub struct Camera {
    fov: Option<f32>,
    position: Option<Vec3>,
    target: Option<Vec3>,
}

impl Camera {
    /// Makes a perspective camera with the defaults.
    pub const fn perspective() -> Self {
        Self {
            fov: None,
            position: None,
            target: None,
        }
    }

    /// Sets the vertical field of view in degrees. Clamped to `1..=179`.
    pub const fn fov(mut self, degrees: f32) -> Self {
        if let Some(value) = finite(degrees) {
            self.fov = Some(value.clamp(1.0, 179.0));
        }
        self
    }

    /// Sets the camera position.
    pub fn position(mut self, position: impl Into<Vec3>) -> Self {
        self.position = Some(position.into().finite_or(Vec3::new(0.0, 1.0, 4.0)));
        self
    }

    /// Sets the point the camera looks at. Orbit controls rotate around it.
    pub fn target(mut self, target: impl Into<Vec3>) -> Self {
        self.target = Some(target.into().finite_or(Vec3::ZERO));
        self
    }
}

/// A primitive geometry. Sizes are in Three.js units.
///
/// A size that is not finite, or is negative, renders as the default size
/// for that geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Geometry {
    /// A box (`box`). Default `1,1,1`.
    Box {
        /// Size on X.
        width: f32,
        /// Size on Y.
        height: f32,
        /// Size on Z.
        depth: f32,
    },
    /// A sphere (`sphere`). Default radius `0.5`.
    Sphere {
        /// Radius.
        radius: f32,
    },
    /// A flat plane in the XY plane (`plane`). Default `1,1`.
    Plane {
        /// Size on X.
        width: f32,
        /// Size on Y.
        height: f32,
    },
    /// A torus (`torus`). Default `0.5,0.2`.
    Torus {
        /// Radius from the center to the tube center.
        radius: f32,
        /// Tube radius.
        tube: f32,
    },
    /// A torus knot (`torus-knot`). Default `0.5,0.15`.
    TorusKnot {
        /// Radius of the knot.
        radius: f32,
        /// Tube radius.
        tube: f32,
    },
    /// A cylinder (`cylinder`). Default `0.5,0.5,1`.
    Cylinder {
        /// Top radius.
        radius_top: f32,
        /// Bottom radius.
        radius_bottom: f32,
        /// Height.
        height: f32,
    },
    /// A cone (`cone`). Default `0.5,1`.
    Cone {
        /// Base radius.
        radius: f32,
        /// Height.
        height: f32,
    },
    /// A capsule (`capsule`). Default `0.3,0.6`.
    Capsule {
        /// Radius.
        radius: f32,
        /// Length of the middle section.
        length: f32,
    },
    /// An icosahedron (`icosahedron`). Default radius `0.5`.
    Icosahedron {
        /// Radius.
        radius: f32,
    },
    /// A dodecahedron (`dodecahedron`). Default radius `0.5`.
    Dodecahedron {
        /// Radius.
        radius: f32,
    },
    /// An octahedron (`octahedron`). Default radius `0.5`.
    Octahedron {
        /// Radius.
        radius: f32,
    },
    /// A tetrahedron (`tetrahedron`). Default radius `0.5`.
    Tetrahedron {
        /// Radius.
        radius: f32,
    },
    /// A flat ring in the XY plane (`ring`). Default `0.25,0.5`.
    Ring {
        /// Inner radius.
        inner_radius: f32,
        /// Outer radius.
        outer_radius: f32,
    },
}

impl Geometry {
    /// The `data-three-mesh` value.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Box { .. } => "box",
            Self::Sphere { .. } => "sphere",
            Self::Plane { .. } => "plane",
            Self::Torus { .. } => "torus",
            Self::TorusKnot { .. } => "torus-knot",
            Self::Cylinder { .. } => "cylinder",
            Self::Cone { .. } => "cone",
            Self::Capsule { .. } => "capsule",
            Self::Icosahedron { .. } => "icosahedron",
            Self::Dodecahedron { .. } => "dodecahedron",
            Self::Octahedron { .. } => "octahedron",
            Self::Tetrahedron { .. } => "tetrahedron",
            Self::Ring { .. } => "ring",
        }
    }

    /// The sizes, in `data-three-args` order, with their defaults.
    fn sizes(&self) -> Vec<(f32, f32)> {
        match *self {
            Self::Box {
                width,
                height,
                depth,
            } => vec![(width, 1.0), (height, 1.0), (depth, 1.0)],
            Self::Sphere { radius }
            | Self::Icosahedron { radius }
            | Self::Dodecahedron { radius }
            | Self::Octahedron { radius }
            | Self::Tetrahedron { radius } => vec![(radius, 0.5)],
            Self::Plane { width, height } => vec![(width, 1.0), (height, 1.0)],
            Self::Torus { radius, tube } => vec![(radius, 0.5), (tube, 0.2)],
            Self::TorusKnot { radius, tube } => vec![(radius, 0.5), (tube, 0.15)],
            Self::Cylinder {
                radius_top,
                radius_bottom,
                height,
            } => vec![(radius_top, 0.5), (radius_bottom, 0.5), (height, 1.0)],
            Self::Cone { radius, height } => vec![(radius, 0.5), (height, 1.0)],
            Self::Capsule { radius, length } => vec![(radius, 0.3), (length, 0.6)],
            Self::Ring {
                inner_radius,
                outer_radius,
            } => vec![(inner_radius, 0.25), (outer_radius, 0.5)],
        }
    }

    /// The `data-three-args` value. Bad sizes become defaults.
    fn args(&self) -> String {
        self.sizes()
            .into_iter()
            .map(|(value, default)| {
                num(if value.is_finite() && value >= 0.0 {
                    value
                } else {
                    default
                })
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// A mesh material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Material {
    /// PBR metal/roughness (`standard`, default).
    #[default]
    Standard,
    /// PBR with more options (`physical`).
    Physical,
    /// No lighting (`basic`).
    Basic,
    /// Matte, cheap lighting (`lambert`).
    Lambert,
    /// Shiny, cheap lighting (`phong`).
    Phong,
    /// Colors from surface normals (`normal`). Needs no light.
    Normal,
}

impl Material {
    /// The `data-three-material` value.
    const fn attr(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Physical => "physical",
            Self::Basic => "basic",
            Self::Lambert => "lambert",
            Self::Phong => "phong",
            Self::Normal => "normal",
        }
    }
}

/// Position, rotation, scale, and spin of an object.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Transform {
    position: Option<Vec3>,
    rotation: Option<Vec3>,
    scale: Option<Vec3>,
    spin: Option<Vec3>,
}

/// Adds the transform setters to an object builder.
macro_rules! transform_setters {
    () => {
        /// Sets the position.
        pub fn position(mut self, position: impl Into<Vec3>) -> Self {
            self.transform.position = Some(position.into().finite_or(Vec3::ZERO));
            self
        }

        /// Sets the rotation in degrees (X, Y, Z order).
        pub fn rotation(mut self, degrees: impl Into<Vec3>) -> Self {
            self.transform.rotation = Some(degrees.into().finite_or(Vec3::ZERO));
            self
        }

        /// Sets the scale. Use [`Vec3::splat`] for a uniform scale.
        pub fn scale(mut self, scale: impl Into<Vec3>) -> Self {
            self.transform.scale = Some(scale.into().finite_or(Vec3::splat(1.0)));
            self
        }

        /// Spins the object, in degrees per second on each axis.
        ///
        /// Spin stops when the user prefers reduced motion, unless the scene
        /// calls [`Scene::animate_reduced_motion`].
        pub fn spin(mut self, degrees_per_second: impl Into<Vec3>) -> Self {
            self.transform.spin = Some(degrees_per_second.into().finite_or(Vec3::ZERO));
            self
        }
    };
}

/// A primitive mesh: a [`Geometry`] with a [`Material`].
///
/// ```rust
/// use autumn_plugin_three::{Color, Mesh};
///
/// let knot = Mesh::torus_knot(0.8, 0.25)
///     .color(Color::hex(0xff7a18))
///     .metalness(0.3)
///     .roughness(0.4)
///     .spin([0.0, 45.0, 0.0]);
/// # let _ = knot;
/// ```
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct Mesh {
    geometry: Geometry,
    material: Option<Material>,
    color: Option<Color>,
    emissive: Option<Color>,
    metalness: Option<f32>,
    roughness: Option<f32>,
    opacity: Option<f32>,
    wireframe: bool,
    transform: Transform,
}

impl Mesh {
    /// Makes a mesh with the default material.
    pub const fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            material: None,
            color: None,
            emissive: None,
            metalness: None,
            roughness: None,
            opacity: None,
            wireframe: false,
            transform: Transform {
                position: None,
                rotation: None,
                scale: None,
                spin: None,
            },
        }
    }

    /// A cube with edge length `size`.
    pub const fn cube(size: f32) -> Self {
        Self::cuboid(size, size, size)
    }

    /// A box.
    pub const fn cuboid(width: f32, height: f32, depth: f32) -> Self {
        Self::new(Geometry::Box {
            width,
            height,
            depth,
        })
    }

    /// A sphere.
    pub const fn sphere(radius: f32) -> Self {
        Self::new(Geometry::Sphere { radius })
    }

    /// A flat plane.
    pub const fn plane(width: f32, height: f32) -> Self {
        Self::new(Geometry::Plane { width, height })
    }

    /// A torus.
    pub const fn torus(radius: f32, tube: f32) -> Self {
        Self::new(Geometry::Torus { radius, tube })
    }

    /// A torus knot.
    pub const fn torus_knot(radius: f32, tube: f32) -> Self {
        Self::new(Geometry::TorusKnot { radius, tube })
    }

    /// A cylinder.
    pub const fn cylinder(radius_top: f32, radius_bottom: f32, height: f32) -> Self {
        Self::new(Geometry::Cylinder {
            radius_top,
            radius_bottom,
            height,
        })
    }

    /// A cone.
    pub const fn cone(radius: f32, height: f32) -> Self {
        Self::new(Geometry::Cone { radius, height })
    }

    /// A capsule.
    pub const fn capsule(radius: f32, length: f32) -> Self {
        Self::new(Geometry::Capsule { radius, length })
    }

    /// An icosahedron.
    pub const fn icosahedron(radius: f32) -> Self {
        Self::new(Geometry::Icosahedron { radius })
    }

    /// A dodecahedron.
    pub const fn dodecahedron(radius: f32) -> Self {
        Self::new(Geometry::Dodecahedron { radius })
    }

    /// An octahedron.
    pub const fn octahedron(radius: f32) -> Self {
        Self::new(Geometry::Octahedron { radius })
    }

    /// A tetrahedron.
    pub const fn tetrahedron(radius: f32) -> Self {
        Self::new(Geometry::Tetrahedron { radius })
    }

    /// A flat ring.
    pub const fn ring(inner_radius: f32, outer_radius: f32) -> Self {
        Self::new(Geometry::Ring {
            inner_radius,
            outer_radius,
        })
    }

    /// Sets the material. Default: [`Material::Standard`].
    pub const fn material(mut self, material: Material) -> Self {
        self.material = Some(material);
        self
    }

    /// Sets the base color. Default: white.
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the emissive (glow) color. Lit materials only.
    pub const fn emissive(mut self, color: Color) -> Self {
        self.emissive = Some(color);
        self
    }

    /// Sets the metalness, `0..=1`. PBR materials only.
    pub const fn metalness(mut self, value: f32) -> Self {
        if let Some(v) = finite(value) {
            self.metalness = Some(v.clamp(0.0, 1.0));
        }
        self
    }

    /// Sets the roughness, `0..=1`. PBR materials only.
    pub const fn roughness(mut self, value: f32) -> Self {
        if let Some(v) = finite(value) {
            self.roughness = Some(v.clamp(0.0, 1.0));
        }
        self
    }

    /// Sets the opacity, `0..=1`. A value below `1` makes the material
    /// transparent.
    pub const fn opacity(mut self, value: f32) -> Self {
        if let Some(v) = finite(value) {
            self.opacity = Some(v.clamp(0.0, 1.0));
        }
        self
    }

    /// Renders the edges only.
    pub const fn wireframe(mut self) -> Self {
        self.wireframe = true;
        self
    }

    transform_setters!();
}

impl Render for Mesh {
    fn render(&self) -> Markup {
        let t = &self.transform;
        html! {
            div hidden data-three-mesh=(self.geometry.kind())
                data-three-args=(self.geometry.args())
                data-three-material=[self.material.map(Material::attr)]
                data-three-color=[self.color]
                data-three-emissive=[self.emissive]
                data-three-metalness=[self.metalness.map(num)]
                data-three-roughness=[self.roughness.map(num)]
                data-three-opacity=[self.opacity.map(num)]
                data-three-wireframe=[self.wireframe.then_some("true")]
                data-three-position=[t.position.map(Vec3::attr)]
                data-three-rotation=[t.rotation.map(Vec3::attr)]
                data-three-scale=[t.scale.map(Vec3::attr)]
                data-three-spin=[t.spin.map(Vec3::attr)] {}
        }
    }
}

/// Which animation clips of a [`Model`] play.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clip {
    /// Play all clips (`*`).
    All,
    /// Play the clip with this name.
    Named(String),
}

/// A glTF / GLB model, loaded with the vendored `GLTFLoader`.
///
/// ```rust
/// use autumn_plugin_three::Model;
///
/// let duck = Model::gltf("/static/models/duck.glb").fit(2.0).play_all();
/// # let _ = duck;
/// ```
///
/// The URL must be `http(s)` or relative, and the app CSP must allow it
/// (`connect-src`). Embedded textures load as `blob:` images, so the
/// default `img-src 'self' data:` must also allow `blob:`.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct Model {
    src: String,
    fit: Option<f32>,
    clip: Option<Clip>,
    transform: Transform,
}

impl Model {
    /// Makes a model from a `.gltf` or `.glb` URL.
    pub fn gltf(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            fit: None,
            clip: None,
            transform: Transform::default(),
        }
    }

    /// Scales the model so its largest side is `size`, and centers it on its
    /// position. A size that is not finite and positive is ignored.
    pub const fn fit(mut self, size: f32) -> Self {
        if size.is_finite() && size > 0.0 {
            self.fit = Some(size);
        }
        self
    }

    /// Plays all animation clips in a loop.
    pub fn play_all(mut self) -> Self {
        self.clip = Some(Clip::All);
        self
    }

    /// Plays the clip with this name in a loop.
    pub fn play(mut self, name: impl Into<String>) -> Self {
        self.clip = Some(Clip::Named(name.into()));
        self
    }

    transform_setters!();
}

impl Render for Model {
    fn render(&self) -> Markup {
        let t = &self.transform;
        let clip = self.clip.as_ref().map(|clip| match clip {
            Clip::All => "*",
            Clip::Named(name) => name.as_str(),
        });
        html! {
            div hidden data-three-model=(self.src)
                data-three-fit=[self.fit.map(num)]
                data-three-clip=[clip]
                data-three-position=[t.position.map(Vec3::attr)]
                data-three-rotation=[t.rotation.map(Vec3::attr)]
                data-three-scale=[t.scale.map(Vec3::attr)]
                data-three-spin=[t.spin.map(Vec3::attr)] {}
        }
    }
}

/// The kind of a [`Light`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightKind {
    /// Even light from all sides (`ambient`).
    Ambient,
    /// Parallel rays, like the sun (`directional`).
    Directional,
    /// Light from one point (`point`).
    Point,
    /// A cone of light that points at the origin (`spot`).
    Spot,
    /// Sky color from above, ground color from below (`hemisphere`).
    Hemisphere,
}

impl LightKind {
    /// The `data-three-light` value.
    const fn attr(self) -> &'static str {
        match self {
            Self::Ambient => "ambient",
            Self::Directional => "directional",
            Self::Point => "point",
            Self::Spot => "spot",
            Self::Hemisphere => "hemisphere",
        }
    }
}

/// A light.
///
/// When a scene has no light and no [`Environment`], `init.js` adds a
/// default hemisphere light and a directional light.
///
/// ```rust
/// use autumn_plugin_three::{Color, Light};
///
/// let sun = Light::directional().intensity(2.5).position([3.0, 5.0, 2.0]);
/// let sky = Light::hemisphere().ground(Color::hex(0x332211));
/// # let _ = (sun, sky);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub struct Light {
    kind: LightKind,
    color: Option<Color>,
    ground: Option<Color>,
    intensity: Option<f32>,
    position: Option<Vec3>,
}

impl Light {
    /// Makes a light of `kind` with the defaults for that kind.
    pub const fn new(kind: LightKind) -> Self {
        Self {
            kind,
            color: None,
            ground: None,
            intensity: None,
            position: None,
        }
    }

    /// An ambient light.
    pub const fn ambient() -> Self {
        Self::new(LightKind::Ambient)
    }

    /// A directional light.
    pub const fn directional() -> Self {
        Self::new(LightKind::Directional)
    }

    /// A point light.
    pub const fn point() -> Self {
        Self::new(LightKind::Point)
    }

    /// A spot light.
    pub const fn spot() -> Self {
        Self::new(LightKind::Spot)
    }

    /// A hemisphere light.
    pub const fn hemisphere() -> Self {
        Self::new(LightKind::Hemisphere)
    }

    /// Sets the color (the sky color for a hemisphere light).
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the ground color. Hemisphere lights only.
    pub const fn ground(mut self, color: Color) -> Self {
        self.ground = Some(color);
        self
    }

    /// Sets the intensity. Negative values become `0`.
    pub const fn intensity(mut self, value: f32) -> Self {
        if let Some(v) = finite(value) {
            self.intensity = Some(v.max(0.0));
        }
        self
    }

    /// Sets the position. Ambient and hemisphere lights ignore it.
    pub fn position(mut self, position: impl Into<Vec3>) -> Self {
        self.position = Some(position.into().finite_or(Vec3::ZERO));
        self
    }
}

impl Render for Light {
    fn render(&self) -> Markup {
        html! {
            div hidden data-three-light=(self.kind.attr())
                data-three-color=[self.color]
                data-three-ground=[self.ground]
                data-three-intensity=[self.intensity.map(num)]
                data-three-position=[self.position.map(Vec3::attr)] {}
        }
    }
}

/// One object in a [`Scene`].
#[derive(Debug, Clone, PartialEq)]
pub enum SceneObject {
    /// A primitive mesh.
    Mesh(Mesh),
    /// A glTF / GLB model.
    Model(Model),
    /// A light.
    Light(Light),
}

impl From<Mesh> for SceneObject {
    fn from(mesh: Mesh) -> Self {
        Self::Mesh(mesh)
    }
}

impl From<Model> for SceneObject {
    fn from(model: Model) -> Self {
        Self::Model(model)
    }
}

impl From<Light> for SceneObject {
    fn from(light: Light) -> Self {
        Self::Light(light)
    }
}

impl Render for SceneObject {
    fn render(&self) -> Markup {
        match self {
            Self::Mesh(mesh) => mesh.render(),
            Self::Model(model) => model.render(),
            Self::Light(light) => light.render(),
        }
    }
}

/// A 3D scene. Renders one `<div data-three="scene">` element.
///
/// ```rust
/// use autumn_plugin_three::{Camera, Color, Controls, Light, Mesh, Scene};
/// use autumn_web::html;
///
/// let scene = Scene::new()
///     .label("A spinning orange torus knot")
///     .camera(Camera::perspective().position([0.0, 0.0, 4.0]))
///     .controls(Controls::OrbitNoZoom)
///     .add(Mesh::torus_knot(0.8, 0.25).color(Color::hex(0xff7a18)).spin([0.0, 45.0, 0.0]))
///     .add(Light::directional().position([3.0, 5.0, 2.0]))
///     .fallback(html! { p { "3D preview needs WebGL." } });
///
/// let html = html! { (scene) }.into_string();
/// assert!(html.starts_with(r#"<div role="img""#), "{html}");
/// assert!(html.contains(r#"data-three="scene""#));
/// assert!(html.contains(r#"data-three-mesh="torus-knot""#));
/// ```
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct Scene {
    id: Option<String>,
    class: Option<String>,
    label: Option<String>,
    aspect: Option<Aspect>,
    background: Option<Color>,
    camera: Option<Camera>,
    controls: Option<Controls>,
    environment: Option<Environment>,
    turntable: Option<f32>,
    animate_reduced_motion: bool,
    objects: Vec<SceneObject>,
    fallback: Option<Markup>,
}

impl Scene {
    /// Makes an empty scene with the defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the element `id`.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the element `class`, for your own size or layout CSS.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Sets an accessible label. The element gets `role="img"` and
    /// `aria-label`.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the aspect ratio. Default: [`Aspect::Wide`] (16 / 9).
    pub const fn aspect(mut self, aspect: Aspect) -> Self {
        self.aspect = Some(aspect);
        self
    }

    /// Sets a solid background color. Default: transparent.
    pub const fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Sets the camera.
    pub const fn camera(mut self, camera: Camera) -> Self {
        self.camera = Some(camera);
        self
    }

    /// Sets the camera controls. Default: [`Controls::None`].
    pub const fn controls(mut self, controls: Controls) -> Self {
        self.controls = Some(controls);
        self
    }

    /// Sets the image-based light.
    pub const fn environment(mut self, environment: Environment) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Turns the whole scene around the Y axis, in degrees per second.
    /// Negative values turn clockwise (seen from above).
    pub const fn turntable(mut self, degrees_per_second: f32) -> Self {
        if let Some(v) = finite(degrees_per_second) {
            self.turntable = Some(v);
        }
        self
    }

    /// Keeps automatic motion (spin, turntable, clips) when the user prefers
    /// reduced motion. Default: automatic motion stops.
    pub const fn animate_reduced_motion(mut self) -> Self {
        self.animate_reduced_motion = true;
        self
    }

    /// Adds a mesh, model, or light.
    #[expect(
        clippy::should_implement_trait,
        reason = "builder method, not arithmetic"
    )]
    pub fn add(mut self, object: impl Into<SceneObject>) -> Self {
        self.objects.push(object.into());
        self
    }

    /// Sets fallback content. It shows without JavaScript, without WebGL,
    /// and when a model fails to load.
    pub fn fallback(mut self, markup: Markup) -> Self {
        self.fallback = Some(markup);
        self
    }
}

impl Render for Scene {
    fn render(&self) -> Markup {
        let camera = self.camera.unwrap_or_default();
        html! {
            div role=[self.label.as_ref().map(|_| "img")]
                aria-label=[self.label.as_deref()]
                id=[self.id.as_deref()]
                class=[self.class.as_deref()]
                data-three="scene"
                data-three-aspect=[self.aspect.map(Aspect::attr)]
                data-three-background=[self.background]
                data-three-camera=[camera.position.map(Vec3::attr)]
                data-three-target=[camera.target.map(Vec3::attr)]
                data-three-fov=[camera.fov.map(num)]
                data-three-controls=[self.controls.map(Controls::attr)]
                data-three-environment=[self.environment.map(Environment::attr)]
                data-three-turntable=[self.turntable.map(num)]
                data-three-reduced=[self.animate_reduced_motion.then_some("animate")] {
                @for object in &self.objects {
                    (object)
                }
                @if let Some(fallback) = &self.fallback {
                    div data-three-fallback { (fallback) }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn render(value: &impl Render) -> String {
        value.render().into_string()
    }

    /// A scene that uses every attribute the builder can emit.
    fn full_scene() -> Scene {
        Scene::new()
            .id("hero")
            .class("hero-3d")
            .label("A demo")
            .aspect(Aspect::Square)
            .background(Color::hex(0x0010_1018))
            .camera(
                Camera::perspective()
                    .fov(40.0)
                    .position([0.0, 2.0, 6.0])
                    .target([0.0, 0.5, 0.0]),
            )
            .controls(Controls::Orbit)
            .environment(Environment::Room)
            .turntable(-12.5)
            .animate_reduced_motion()
            .add(
                Mesh::sphere(1.0)
                    .material(Material::Physical)
                    .color(Color::rgb(255, 0, 0))
                    .emissive(Color::hex(0x0000_ff00))
                    .metalness(0.5)
                    .roughness(0.25)
                    .opacity(0.75)
                    .wireframe()
                    .position([1.0, 2.0, 3.0])
                    .rotation([0.0, 90.0, 0.0])
                    .scale(Vec3::splat(2.0))
                    .spin([0.0, 30.0, 0.0]),
            )
            .add(
                Model::gltf("/m.glb")
                    .fit(2.0)
                    .play("Walk")
                    .position([0.0, -1.0, 0.0]),
            )
            .add(Model::gltf("/n.glb").play_all())
            .add(
                Light::hemisphere()
                    .color(Color::WHITE)
                    .ground(Color::BLACK)
                    .intensity(2.0)
                    .position([0.0, 5.0, 0.0]),
            )
            .fallback(html! { p { "No WebGL" } })
    }

    #[test]
    fn empty_scene_is_one_bare_element() {
        assert_eq!(render(&Scene::new()), r#"<div data-three="scene"></div>"#);
    }

    #[test]
    fn full_scene_renders_every_scene_attribute() {
        let html = render(&full_scene());
        for expected in [
            r#"role="img""#,
            r#"aria-label="A demo""#,
            r#"id="hero""#,
            r#"class="hero-3d""#,
            r#"data-three-aspect="1/1""#,
            r##"data-three-background="#101018""##,
            r#"data-three-camera="0,2,6""#,
            r#"data-three-target="0,0.5,0""#,
            r#"data-three-fov="40""#,
            r#"data-three-controls="orbit""#,
            r#"data-three-environment="room""#,
            r#"data-three-turntable="-12.5""#,
            r#"data-three-reduced="animate""#,
            "<div data-three-fallback><p>No WebGL</p></div>",
        ] {
            assert!(html.contains(expected), "{expected} in {html}");
        }
    }

    #[test]
    fn mesh_renders_every_attribute() {
        let scene = full_scene();
        let SceneObject::Mesh(mesh) = &scene.objects[0] else {
            panic!("first object is the mesh");
        };
        assert_eq!(
            render(mesh),
            concat!(
                r#"<div hidden data-three-mesh="sphere" data-three-args="1" "#,
                r##"data-three-material="physical" data-three-color="#ff0000" "##,
                r##"data-three-emissive="#00ff00" data-three-metalness="0.5" "##,
                r#"data-three-roughness="0.25" data-three-opacity="0.75" "#,
                r#"data-three-wireframe="true" data-three-position="1,2,3" "#,
                r#"data-three-rotation="0,90,0" data-three-scale="2,2,2" "#,
                r#"data-three-spin="0,30,0"></div>"#
            )
        );
    }

    #[test]
    fn model_and_light_render_their_attributes() {
        let html = render(&full_scene());
        assert!(html.contains(concat!(
            r#"<div hidden data-three-model="/m.glb" data-three-fit="2" "#,
            r#"data-three-clip="Walk" data-three-position="0,-1,0"></div>"#
        )));
        assert!(
            html.contains(r#"<div hidden data-three-model="/n.glb" data-three-clip="*"></div>"#)
        );
        assert!(html.contains(concat!(
            r##"<div hidden data-three-light="hemisphere" data-three-color="#ffffff" "##,
            r##"data-three-ground="#000000" data-three-intensity="2" "##,
            r#"data-three-position="0,5,0"></div>"#
        )));
    }

    #[test]
    fn objects_render_in_order_before_the_fallback() {
        let html = render(&full_scene());
        let mesh = html.find("data-three-mesh").expect("mesh");
        let model = html.find("data-three-model").expect("model");
        let light = html.find("data-three-light").expect("light");
        let fallback = html.find("data-three-fallback").expect("fallback");
        assert!(mesh < model && model < light && light < fallback, "{html}");
    }

    #[test]
    fn every_geometry_renders_its_kind_and_sizes() {
        let cases = [
            (Mesh::cube(2.0), "box", "2,2,2"),
            (Mesh::cuboid(1.0, 2.0, 3.0), "box", "1,2,3"),
            (Mesh::sphere(0.5), "sphere", "0.5"),
            (Mesh::plane(4.0, 3.0), "plane", "4,3"),
            (Mesh::torus(1.0, 0.3), "torus", "1,0.3"),
            (Mesh::torus_knot(1.0, 0.3), "torus-knot", "1,0.3"),
            (Mesh::cylinder(0.0, 1.0, 2.0), "cylinder", "0,1,2"),
            (Mesh::cone(1.0, 2.0), "cone", "1,2"),
            (Mesh::capsule(0.5, 1.0), "capsule", "0.5,1"),
            (Mesh::icosahedron(1.0), "icosahedron", "1"),
            (Mesh::dodecahedron(1.0), "dodecahedron", "1"),
            (Mesh::octahedron(1.0), "octahedron", "1"),
            (Mesh::tetrahedron(1.0), "tetrahedron", "1"),
            (Mesh::ring(0.5, 1.0), "ring", "0.5,1"),
        ];
        for (mesh, kind, args) in cases {
            let html = render(&mesh);
            assert!(
                html.contains(&format!(r#"data-three-mesh="{kind}""#)),
                "{html}"
            );
            assert!(
                html.contains(&format!(r#"data-three-args="{args}""#)),
                "{html}"
            );
        }
    }

    #[test]
    fn bad_sizes_render_as_defaults() {
        let html = render(&Mesh::cuboid(f32::NAN, -1.0, f32::INFINITY));
        assert!(html.contains(r#"data-three-args="1,1,1""#), "{html}");
        let html = render(&Mesh::torus(-0.0, f32::NEG_INFINITY));
        assert!(html.contains(r#"data-three-args="0,0.2""#), "{html}");
    }

    #[test]
    fn setters_ignore_non_finite_input() {
        let html = render(
            &Scene::new()
                .camera(Camera::perspective().fov(f32::NAN))
                .turntable(f32::INFINITY)
                .add(
                    Mesh::cube(1.0)
                        .metalness(f32::NAN)
                        .roughness(f32::INFINITY)
                        .opacity(f32::NEG_INFINITY),
                )
                .add(Model::gltf("/m.glb").fit(f32::NAN))
                .add(Light::point().intensity(f32::NAN)),
        );
        for attr in [
            "data-three-fov",
            "data-three-turntable",
            "data-three-metalness",
            "data-three-roughness",
            "data-three-opacity",
            "data-three-fit",
            "data-three-intensity",
        ] {
            assert!(!html.contains(attr), "{attr} in {html}");
        }
    }

    #[test]
    fn vectors_replace_non_finite_components() {
        let html = render(
            &Mesh::cube(1.0)
                .position([f32::NAN, 1.0, 2.0])
                .scale([f32::INFINITY, 2.0, 2.0])
                .spin([0.0, f32::NAN, 0.0]),
        );
        assert!(html.contains(r#"data-three-position="0,1,2""#), "{html}");
        assert!(html.contains(r#"data-three-scale="1,2,2""#), "{html}");
        assert!(html.contains(r#"data-three-spin="0,0,0""#), "{html}");
        let html =
            render(&Scene::new().camera(Camera::perspective().position([0.0, f32::NAN, 9.0])));
        assert!(html.contains(r#"data-three-camera="0,1,9""#), "{html}");
    }

    #[test]
    fn ranges_are_clamped() {
        let html = render(&Mesh::cube(1.0).metalness(2.0).roughness(-1.0).opacity(1.5));
        assert!(html.contains(r#"data-three-metalness="1""#), "{html}");
        assert!(html.contains(r#"data-three-roughness="0""#), "{html}");
        assert!(html.contains(r#"data-three-opacity="1""#), "{html}");
        let html = render(&Scene::new().camera(Camera::perspective().fov(500.0)));
        assert!(html.contains(r#"data-three-fov="179""#), "{html}");
        let html = render(&Light::spot().intensity(-3.0));
        assert!(html.contains(r#"data-three-intensity="0""#), "{html}");
        let html = render(&Model::gltf("/m.glb").fit(-1.0));
        assert!(!html.contains("data-three-fit"), "{html}");
    }

    #[test]
    fn aspects_render_ratios() {
        for (aspect, value) in [
            (Aspect::Wide, "16/9"),
            (Aspect::Standard, "4/3"),
            (Aspect::Square, "1/1"),
            (Aspect::Ultrawide, "21/9"),
            (Aspect::Photo, "3/2"),
            (Aspect::Portrait, "3/4"),
            (Aspect::Ratio(2.35, 1.0), "2.35/1"),
            (Aspect::Ratio(0.0, 1.0), "16/9"),
            (Aspect::Ratio(f32::NAN, 1.0), "16/9"),
        ] {
            let html = render(&Scene::new().aspect(aspect));
            assert!(
                html.contains(&format!(r#"data-three-aspect="{value}""#)),
                "{html}"
            );
        }
    }

    #[test]
    fn enums_render_their_names() {
        for (controls, value) in [
            (Controls::None, "none"),
            (Controls::Orbit, "orbit"),
            (Controls::OrbitNoZoom, "orbit-no-zoom"),
        ] {
            assert!(
                render(&Scene::new().controls(controls))
                    .contains(&format!(r#"data-three-controls="{value}""#))
            );
        }
        for (material, value) in [
            (Material::Standard, "standard"),
            (Material::Physical, "physical"),
            (Material::Basic, "basic"),
            (Material::Lambert, "lambert"),
            (Material::Phong, "phong"),
            (Material::Normal, "normal"),
        ] {
            assert!(
                render(&Mesh::cube(1.0).material(material))
                    .contains(&format!(r#"data-three-material="{value}""#))
            );
        }
        for (light, value) in [
            (Light::ambient(), "ambient"),
            (Light::directional(), "directional"),
            (Light::point(), "point"),
            (Light::spot(), "spot"),
            (Light::hemisphere(), "hemisphere"),
        ] {
            assert_eq!(
                render(&light),
                format!(r#"<div hidden data-three-light="{value}"></div>"#)
            );
        }
    }

    #[test]
    fn colors_format_as_hex() {
        assert_eq!(Color::hex(0xff12_3456).to_string(), "#123456");
        assert_eq!(Color::hex(0xff12_3456).value(), 0x0012_3456);
        assert_eq!(Color::rgb(1, 2, 3).to_string(), "#010203");
        assert_eq!(Color::WHITE.to_string(), "#ffffff");
        assert_eq!(Color::BLACK.to_string(), "#000000");
    }

    #[test]
    fn vec3_conversions_agree() {
        assert_eq!(Vec3::from((1.0, 2.0, 3.0)), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(Vec3::from([1.0, 2.0, 3.0]), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(Vec3::ZERO, Vec3::splat(0.0));
    }

    #[test]
    fn user_text_is_escaped() {
        let html = render(
            &Scene::new()
                .label(r#""><script>alert(1)</script>"#)
                .add(Model::gltf(r#"/a.glb"><script>"#).play(r#"x"y"#)),
        );
        assert!(!html.contains("<script>"), "{html}");
        assert!(html.contains("&quot;&gt;&lt;script&gt;"), "{html}");
    }

    #[test]
    fn scene_renders_inside_maud() {
        let html = html! { main { (Scene::new()) } }.into_string();
        assert_eq!(html, r#"<main><div data-three="scene"></div></main>"#);
    }

    #[test]
    fn every_emitted_attribute_and_value_is_known_to_the_runtime() {
        // Rust and JS stay in lockstep: parse.js must name every attribute
        // and every keyword value the builder emits.
        let parse = include_str!("../assets/parse.js");
        let html = render(&full_scene());
        let mut names: Vec<&str> = html
            .split(|c: char| c.is_whitespace() || c == '<' || c == '>')
            .filter_map(|token| token.split('=').next())
            .filter(|name| name.starts_with("data-three"))
            .collect();
        names.sort_unstable();
        names.dedup();
        assert!(names.len() >= 30, "{names:?}");
        for name in names {
            assert!(
                parse.contains(&format!("\"{name}\"")),
                "parse.js names {name}"
            );
        }
        let geometries = [
            "box",
            "sphere",
            "plane",
            "torus",
            "torus-knot",
            "cylinder",
            "cone",
            "capsule",
            "icosahedron",
            "dodecahedron",
            "octahedron",
            "tetrahedron",
            "ring",
        ];
        let keywords = [
            "orbit",
            "orbit-no-zoom",
            "none",
            "room",
            "animate",
            "standard",
            "physical",
            "basic",
            "lambert",
            "phong",
            "normal",
            "ambient",
            "directional",
            "point",
            "spot",
            "hemisphere",
        ];
        for value in geometries.iter().chain(&keywords) {
            assert!(
                parse.contains(&format!("\"{value}\"")),
                "parse.js knows {value}"
            );
        }
    }

    /// Any `f32`, with non-finite values made likely.
    fn any_f32() -> impl Strategy<Value = f32> {
        prop_oneof![
            Just(f32::NAN),
            Just(f32::INFINITY),
            Just(f32::NEG_INFINITY),
            Just(-0.0_f32),
            any::<f32>(),
        ]
    }

    /// Each number inside the `data-three-*` attribute values.
    fn numbers(html: &str) -> Vec<String> {
        let mut out = Vec::new();
        for part in html.split("data-three-").skip(1) {
            let Some(value) = part.split('"').nth(1) else {
                continue;
            };
            for token in value.split([',', '/']) {
                if token.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
                    out.push(token.to_owned());
                }
            }
        }
        out
    }

    proptest! {
        #[test]
        fn builder_never_emits_non_finite_numbers(
            a in any_f32(), b in any_f32(), c in any_f32(), d in any_f32()
        ) {
            let html = render(
                &Scene::new()
                    .aspect(Aspect::Ratio(a, b))
                    .camera(Camera::perspective().fov(a).position([a, b, c]).target([d, a, b]))
                    .turntable(c)
                    .add(
                        Mesh::cylinder(a, b, c)
                            .metalness(d)
                            .roughness(a)
                            .opacity(b)
                            .position([a, b, c])
                            .rotation([b, c, d])
                            .scale([c, d, a])
                            .spin([d, a, b]),
                    )
                    .add(Model::gltf("/m.glb").fit(a).position([d, d, d]))
                    .add(Light::spot().intensity(c).position([a, c, d])),
            );
            prop_assert!(!html.contains("NaN"), "{}", html);
            prop_assert!(!html.contains("inf"), "{}", html);
            for number in numbers(&html) {
                let parsed: f32 = number.parse().map_err(|e| TestCaseError::fail(format!("{number}: {e}")))?;
                prop_assert!(parsed.is_finite(), "{}", number);
                prop_assert!(number != "-0", "{}", html);
            }
        }

        #[test]
        fn vec3_attr_round_trips_finite_values(x in -1.0e6_f32..1.0e6, y in -1.0e6_f32..1.0e6, z in -1.0e6_f32..1.0e6) {
            let attr = Vec3::new(x, y, z).attr();
            let parts: Vec<f32> = attr.split(',').map(|p| p.parse().expect("number")).collect();
            prop_assert_eq!(parts, vec![x + 0.0, y + 0.0, z + 0.0]);
        }

        #[test]
        fn clamped_values_stay_in_range(v in any_f32()) {
            let mesh = Mesh::cube(1.0).metalness(v).roughness(v).opacity(v);
            for value in [mesh.metalness, mesh.roughness, mesh.opacity].into_iter().flatten() {
                prop_assert!((0.0..=1.0).contains(&value));
            }
            if let Some(fov) = Camera::perspective().fov(v).fov {
                prop_assert!((1.0..=179.0).contains(&fov));
            }
        }
    }
}
