//! Vendored Three.js assets, embedded at compile time.
//!
//! The bundle holds:
//!
//! - Three.js core: `three.core.min.js` and `three.module.min.js`
//!   (upstream bytes, unchanged).
//! - Addons: `OrbitControls.js`, `GLTFLoader.js`, `BufferGeometryUtils.js`,
//!   `SkeletonUtils.js`, `RoomEnvironment.js`. Only their import specifiers
//!   change (see [`VendoredFile::rewrites`]).
//! - Plugin files: `parse.js`, `init.js`, `three.css`.
//!
//! [`ThreePlugin`](crate::ThreePlugin) installs [`THREE_ASSETS`] through
//! `AppBuilder::plugin_assets`. Autumn serves each file under
//! `/static/_plugins/three/` at a hashed URL (immutable) and at its plain
//! URL (`must-revalidate`). Autumn computes the SRI hashes.
//!
//! `manifest.json` and `THREE-LICENSE` are not in the bundle. They are not
//! served.

use autumn_web::assets::PluginAssets;

/// URL namespace of the bundle. Files are served under
/// `/static/_plugins/three/`.
pub const ASSETS_NAMESPACE: &str = "three";

/// Pinned Three.js version.
pub const THREE_VERSION: &str = "0.185.1";

/// Plain URL of the Three.js ES module.
///
/// Import Three.js from this URL in your own module scripts. Then your code
/// and the plugin share one Three.js instance:
///
/// ```js
/// import * as THREE from "/static/_plugins/three/three.module.min.js";
/// ```
pub const THREE_MODULE_URL: &str = "/static/_plugins/three/three.module.min.js";

/// Three.js core (classes and math).
pub(crate) const CORE_JS: &str = "three.core.min.js";
/// Three.js module (renderers). Imports [`CORE_JS`].
pub(crate) const MODULE_JS: &str = "three.module.min.js";
/// Orbit camera controls addon.
pub(crate) const ORBIT_CONTROLS_JS: &str = "OrbitControls.js";
/// glTF / GLB loader addon.
pub(crate) const GLTF_LOADER_JS: &str = "GLTFLoader.js";
/// Geometry helpers. [`GLTF_LOADER_JS`] imports it.
pub(crate) const BUFFER_GEOMETRY_UTILS_JS: &str = "BufferGeometryUtils.js";
/// Skeleton helpers. [`GLTF_LOADER_JS`] imports it.
pub(crate) const SKELETON_UTILS_JS: &str = "SkeletonUtils.js";
/// Room environment (image-based light) addon.
pub(crate) const ROOM_ENVIRONMENT_JS: &str = "RoomEnvironment.js";
/// Plugin attribute parsers (no Three.js dependency).
pub(crate) const PARSE_JS: &str = "parse.js";
/// Plugin runtime: scans `[data-three="scene"]` and builds scenes.
pub(crate) const INIT_JS: &str = "init.js";
/// Plugin default styles.
pub(crate) const THREE_CSS: &str = "three.css";

/// The plugin asset bundle.
///
/// [`ThreePlugin`](crate::ThreePlugin) installs it. Use it directly only to
/// make URLs or tags yourself:
///
/// ```rust
/// use autumn_plugin_three::THREE_ASSETS;
///
/// let url = THREE_ASSETS.url("init.js");
/// assert!(url.starts_with("/static/_plugins/three/init."), "{url}");
/// let sri = THREE_ASSETS.integrity("init.js").expect("init.js is bundled");
/// assert!(sri.starts_with("sha384-"));
/// ```
pub static THREE_ASSETS: PluginAssets = PluginAssets::from_files(
    ASSETS_NAMESPACE,
    &[
        (CORE_JS, include_bytes!("../assets/three.core.min.js")),
        (MODULE_JS, include_bytes!("../assets/three.module.min.js")),
        (
            ORBIT_CONTROLS_JS,
            include_bytes!("../assets/OrbitControls.js"),
        ),
        (GLTF_LOADER_JS, include_bytes!("../assets/GLTFLoader.js")),
        (
            BUFFER_GEOMETRY_UTILS_JS,
            include_bytes!("../assets/BufferGeometryUtils.js"),
        ),
        (
            SKELETON_UTILS_JS,
            include_bytes!("../assets/SkeletonUtils.js"),
        ),
        (
            ROOM_ENVIRONMENT_JS,
            include_bytes!("../assets/RoomEnvironment.js"),
        ),
        (PARSE_JS, include_bytes!("../assets/parse.js")),
        (INIT_JS, include_bytes!("../assets/init.js")),
        (THREE_CSS, include_bytes!("../assets/three.css")),
    ],
);

/// Upstream `import ... from 'three'` line end.
const BARE_THREE: &str = "} from 'three';";
/// Served replacement for [`BARE_THREE`].
const RELATIVE_THREE: &str = "} from './three.module.min.js';";
/// The rewrite every addon gets.
const THREE_REWRITE: (&str, &str) = (BARE_THREE, RELATIVE_THREE);

/// Provenance of one vendored upstream file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendoredFile {
    /// Logical path in [`THREE_ASSETS`].
    pub path: &'static str,
    /// Upstream URL (jsDelivr, pinned version).
    pub source: &'static str,
    /// `sha384` SRI of the upstream bytes, before the import rewrites.
    pub upstream_integrity: &'static str,
    /// Import rewrites, as `(upstream text, served text)`.
    ///
    /// Addons import the bare specifier `three` and `../utils/*.js`. These
    /// need an import map, which is an inline script. The default Autumn CSP
    /// blocks inline scripts, so `scripts/vendor.sh` makes the imports
    /// relative. Keep this table and `scripts/vendor.sh` in sync.
    pub rewrites: &'static [(&'static str, &'static str)],
}

/// The vendored upstream files and their pins.
///
/// A test checks each pin against the bundled bytes, after it reverses
/// the [`VendoredFile::rewrites`].
pub const VENDORED: &[VendoredFile] = &[
    VendoredFile {
        path: CORE_JS,
        source: "https://cdn.jsdelivr.net/npm/three@0.185.1/build/three.core.min.js",
        upstream_integrity: "sha384-rx+KIp/9ptjArhnFAcpVoOc/ynktDsRtRJKIbC7YVKylEvFu8sgmzk9RmQ+CIV48",
        rewrites: &[],
    },
    VendoredFile {
        path: MODULE_JS,
        source: "https://cdn.jsdelivr.net/npm/three@0.185.1/build/three.module.min.js",
        upstream_integrity: "sha384-QHQk1LzjJlJYNdthXjKCmffpDRZL3EqJ7LfqBzyKyvGgjAYM2ZVuYtFGg42NcAJ/",
        rewrites: &[],
    },
    VendoredFile {
        path: ORBIT_CONTROLS_JS,
        source: "https://cdn.jsdelivr.net/npm/three@0.185.1/examples/jsm/controls/OrbitControls.js",
        upstream_integrity: "sha384-4rziNxOBZKQ69i+w+f89KJ55TCYquwchVbByQwmaOeIOXdOU2PLDn3kOfXHwIJC9",
        rewrites: &[THREE_REWRITE],
    },
    VendoredFile {
        path: GLTF_LOADER_JS,
        source: "https://cdn.jsdelivr.net/npm/three@0.185.1/examples/jsm/loaders/GLTFLoader.js",
        upstream_integrity: "sha384-3CnKaFWE2emo2DOUQi/yFm4SMemUgSZ9IAJe/V2pyJTw9KXWYSmR0MiX/7RoPyiJ",
        rewrites: &[
            THREE_REWRITE,
            (
                "from '../utils/BufferGeometryUtils.js';",
                "from './BufferGeometryUtils.js';",
            ),
            (
                "from '../utils/SkeletonUtils.js';",
                "from './SkeletonUtils.js';",
            ),
        ],
    },
    VendoredFile {
        path: BUFFER_GEOMETRY_UTILS_JS,
        source: "https://cdn.jsdelivr.net/npm/three@0.185.1/examples/jsm/utils/BufferGeometryUtils.js",
        upstream_integrity: "sha384-05mkYituMJObxUkTK7xbW9SA45DEaxOge7FQUGrhW3dsFz0fckzykM6RMAPN26rD",
        rewrites: &[THREE_REWRITE],
    },
    VendoredFile {
        path: SKELETON_UTILS_JS,
        source: "https://cdn.jsdelivr.net/npm/three@0.185.1/examples/jsm/utils/SkeletonUtils.js",
        upstream_integrity: "sha384-Pozn8j5+YFr3ak8Pm90ayqDrGYn/DV7vVs/YIIqzJhzeJT0LQksoS1fZQ5lfsYlw",
        rewrites: &[THREE_REWRITE],
    },
    VendoredFile {
        path: ROOM_ENVIRONMENT_JS,
        source: "https://cdn.jsdelivr.net/npm/three@0.185.1/examples/jsm/environments/RoomEnvironment.js",
        upstream_integrity: "sha384-/H49oz0ZtMgJNgMZ+OhhuMuKBOsaiC3kY0/PZSvgJJXAOJJwmSBJjzlV5lJul3MB",
        rewrites: &[THREE_REWRITE],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use sha2::{Digest as _, Sha384};

    /// Computes the `sha384` SRI of bytes.
    fn sri(bytes: &[u8]) -> String {
        let digest = Sha384::digest(bytes);
        format!(
            "sha384-{}",
            base64::engine::general_purpose::STANDARD.encode(digest)
        )
    }

    /// True when `path` has the extension `ext` (any case).
    fn has_ext(path: &str, ext: &str) -> bool {
        std::path::Path::new(path)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    }

    /// The bundled bytes of `path` as text.
    fn text(path: &str) -> &'static str {
        let asset = THREE_ASSETS.get(path).expect("file is bundled");
        std::str::from_utf8(asset.bytes()).expect("bundled text is UTF-8")
    }

    /// Module specifiers in `source`: static `import`/`export ... from "x"`
    /// and dynamic `import("x")`. A static `from` counts only after `}` or
    /// on a line that starts with `import`/`export`, so text in strings
    /// (`from "srgb-linear"`) does not count. Comment lines are skipped.
    fn specifiers(source: &str) -> Vec<String> {
        let mut out = Vec::new();
        for line in source.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('*') || trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            let statement = trimmed.starts_with("import") || trimmed.starts_with("export");
            for marker in ["from", "import("] {
                let mut start = 0;
                while let Some(at) = line[start..].find(marker) {
                    let before = line[..start + at].trim_end();
                    start += at + marker.len();
                    if marker == "from" && !(statement || before.ends_with('}')) {
                        continue;
                    }
                    let after = line[start..].trim_start();
                    let Some(quote) = after.chars().next().filter(|c| *c == '"' || *c == '\'')
                    else {
                        continue;
                    };
                    if let Some(end) = after[1..].find(quote) {
                        out.push(after[1..=end].to_owned());
                    }
                }
            }
        }
        out
    }

    #[test]
    fn specifier_scan_finds_imports_and_skips_strings() {
        let source = "import { a } from './a.js';\n} from \"./b.js\";\nx}from\"./c.js\"\n\
                      const m = await import(\"./d.js\");\nwarn(`from \"srgb\"`);\n * from 'three'";
        assert_eq!(specifiers(source), ["./a.js", "./b.js", "./c.js", "./d.js"]);
    }

    #[test]
    fn bundle_holds_exactly_the_served_files() {
        let mut files: Vec<&str> = THREE_ASSETS
            .iter()
            .map(autumn_web::assets::PluginAsset::logical_path)
            .collect();
        files.sort_unstable();
        let mut expected = vec![
            BUFFER_GEOMETRY_UTILS_JS,
            GLTF_LOADER_JS,
            ORBIT_CONTROLS_JS,
            ROOM_ENVIRONMENT_JS,
            SKELETON_UTILS_JS,
            INIT_JS,
            PARSE_JS,
            THREE_CSS,
            CORE_JS,
            MODULE_JS,
        ];
        expected.sort_unstable();
        // `manifest.json` and `THREE-LICENSE` are not served.
        assert_eq!(files, expected);
        assert_eq!(THREE_ASSETS.namespace(), ASSETS_NAMESPACE);
        assert_eq!(THREE_ASSETS.mount_path(), "/static/_plugins/three");
    }

    #[test]
    fn bundle_integrity_matches_embedded_bytes() {
        for asset in THREE_ASSETS.iter() {
            assert_eq!(
                asset.integrity(),
                sri(asset.bytes()),
                "{}",
                asset.logical_path()
            );
        }
    }

    #[test]
    fn vendored_files_match_upstream_after_reversing_rewrites() {
        assert_eq!(VENDORED.len(), 7, "two core files and five addons");
        for file in VENDORED {
            let mut source = text(file.path).to_owned();
            for (upstream, served) in file.rewrites {
                assert_eq!(
                    source.matches(served).count(),
                    1,
                    "{}: rewrite target {served:?} is unique",
                    file.path
                );
                source = source.replace(served, upstream);
            }
            assert_eq!(
                sri(source.as_bytes()),
                file.upstream_integrity,
                "{} matches upstream",
                file.path
            );
            assert!(
                file.source.starts_with(&format!(
                    "https://cdn.jsdelivr.net/npm/three@{THREE_VERSION}/"
                )),
                "{}",
                file.source
            );
        }
    }

    #[test]
    fn core_files_are_unchanged_and_addons_are_rewritten() {
        for file in VENDORED {
            let is_core = file.path == CORE_JS || file.path == MODULE_JS;
            assert_eq!(file.rewrites.is_empty(), is_core, "{}", file.path);
            for (upstream, served) in file.rewrites {
                assert_ne!(upstream, served);
            }
        }
    }

    #[test]
    fn every_module_import_resolves_inside_the_bundle() {
        let mut checked = 0;
        for asset in THREE_ASSETS.iter() {
            let path = asset.logical_path();
            if !has_ext(path, "js") {
                continue;
            }
            for spec in specifiers(text(path)) {
                let target = spec
                    .strip_prefix("./")
                    .unwrap_or_else(|| panic!("{path}: bare or parent import {spec:?}"));
                assert!(
                    THREE_ASSETS.get(target).is_some(),
                    "{path}: import {spec:?} is not bundled"
                );
                checked += 1;
            }
        }
        assert!(checked >= 13, "the scan finds the imports ({checked})");
    }

    #[test]
    fn urls_are_fingerprinted_under_the_plugin_mount() {
        for asset in THREE_ASSETS.iter() {
            let path = asset.logical_path();
            assert_eq!(asset.plain_url(), format!("/static/_plugins/three/{path}"));
            let (stem, ext) = path.rsplit_once('.').expect("extension");
            let hash = asset
                .url()
                .strip_prefix(&format!("/static/_plugins/three/{stem}."))
                .and_then(|rest| rest.strip_suffix(&format!(".{ext}")))
                .expect("fingerprinted form");
            assert_eq!(hash.len(), 8);
            assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn module_url_is_the_plain_url_of_the_three_module() {
        assert_eq!(
            THREE_MODULE_URL,
            THREE_ASSETS.get(MODULE_JS).expect("bundled").plain_url()
        );
    }

    #[test]
    fn content_types_match_the_files() {
        for asset in THREE_ASSETS.iter() {
            let expected = if has_ext(asset.logical_path(), "css") {
                "text/css; charset=utf-8"
            } else {
                "text/javascript; charset=utf-8"
            };
            assert_eq!(asset.content_type(), expected, "{}", asset.logical_path());
        }
    }

    #[test]
    fn manifest_agrees_with_constants() {
        let manifest = include_str!("../assets/manifest.json");
        assert!(manifest.contains(&format!("\"version\": \"{THREE_VERSION}\"")));
        for file in VENDORED {
            assert!(manifest.contains(file.source), "{}", file.source);
            assert!(
                manifest.contains(file.upstream_integrity),
                "{}",
                file.upstream_integrity
            );
        }
    }

    #[test]
    fn vendor_script_agrees_with_constants() {
        let script = include_str!("../scripts/vendor.sh");
        assert!(script.contains(&format!("VERSION=\"{THREE_VERSION}\"")));
        for file in VENDORED {
            let hash = file.upstream_integrity.trim_start_matches("sha384-");
            assert!(script.contains(hash), "{} pin in vendor.sh", file.path);
        }
    }
}
