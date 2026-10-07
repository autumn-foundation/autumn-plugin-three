//! Tag helpers: [`three_script()`] and [`three_stylesheet()`].
//!
//! Three.js is a set of ES modules. Module imports resolve to plain URLs
//! (for example `/static/_plugins/three/three.module.min.js`). The page and
//! the addons must import the same URL, or two copies of Three.js load.
//!
//! [`three_script()`] emits:
//!
//! 1. `<link rel="modulepreload">` with SRI for the module graph. The
//!    browser checks the bytes before the imports use them.
//! 2. One `<script type="module">` for `init.js` at its hashed URL, with SRI.
//!
//! There is no import map and no inline script, so the default Autumn CSP
//! (`script-src 'self'`) allows all tags.

use autumn_web::{Markup, html};

use crate::assets::{CORE_JS, INIT_JS, MODULE_JS, PARSE_JS, THREE_ASSETS, THREE_CSS};

/// Files that `init.js` imports statically, in dependency order.
const MODULE_GRAPH: [&str; 3] = [CORE_JS, MODULE_JS, PARSE_JS];

/// Renders the tags that load Three.js and the plugin runtime.
///
/// Put it in the page `<head>`. Module scripts run after parsing, so the
/// scenes in `<body>` exist when `init.js` runs.
///
/// ```rust
/// use autumn_plugin_three::three_script;
/// use autumn_web::html;
///
/// let head = html! { head { (three_script()) } }.into_string();
/// assert!(head.contains(r#"type="module""#));
/// assert!(head.contains(r#"rel="modulepreload""#));
/// ```
#[must_use]
pub fn three_script() -> Markup {
    html! {
        @for path in MODULE_GRAPH {
            @if let Some(asset) = THREE_ASSETS.get(path) {
                link rel="modulepreload" href=(asset.plain_url())
                    integrity=(asset.integrity()) crossorigin="anonymous";
            }
        }
        @if let Some(init) = THREE_ASSETS.get(INIT_JS) {
            script type="module" src=(init.url()) integrity=(init.integrity())
                crossorigin="anonymous" {}
        }
    }
}

/// Renders the `<link>` tag for the plugin stylesheet.
///
/// The stylesheet gives scenes a default size (`aspect-ratio: 16 / 9`),
/// hides object declarations, and shows fallback content when needed. Put
/// it in the page `<head>`.
///
/// ```rust
/// use autumn_plugin_three::three_stylesheet;
///
/// let html = three_stylesheet().into_string();
/// assert!(html.contains(r#"href="/static/_plugins/three/three."#), "{html}");
/// ```
#[must_use]
pub fn three_stylesheet() -> Markup {
    THREE_ASSETS.stylesheet_tag(THREE_CSS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{CORE_JS, INIT_JS, MODULE_JS, PARSE_JS, THREE_ASSETS, THREE_CSS};

    fn asset(path: &str) -> &'static autumn_web::assets::PluginAsset {
        THREE_ASSETS.get(path).expect("file is bundled")
    }

    #[test]
    fn module_graph_is_preloaded_with_sri_at_plain_urls() {
        let html = three_script().into_string();
        for path in [CORE_JS, MODULE_JS, PARSE_JS] {
            let a = asset(path);
            let tag = format!(
                r#"<link rel="modulepreload" href="{}" integrity="{}" crossorigin="anonymous">"#,
                a.plain_url(),
                a.integrity()
            );
            assert!(html.contains(&tag), "{path}: {html}");
        }
    }

    #[test]
    fn entry_is_a_module_script_with_sri_and_fingerprinted_url() {
        let html = three_script().into_string();
        let init = asset(INIT_JS);
        let tag = format!(
            r#"<script type="module" src="{}" integrity="{}" crossorigin="anonymous"></script>"#,
            init.url(),
            init.integrity()
        );
        assert!(html.contains(&tag), "{html}");
        assert_eq!(html.matches("<script").count(), 1, "one entry: {html}");
        assert!(!html.contains("importmap"), "no inline import map: {html}");
    }

    #[test]
    fn preloads_come_before_the_entry() {
        let html = three_script().into_string();
        let entry = html.find("<script").expect("entry");
        let last_preload = html.rfind("modulepreload").expect("preload");
        assert!(last_preload < entry, "{html}");
    }

    #[test]
    fn stylesheet_link_carries_sri_and_fingerprinted_url() {
        let html = three_stylesheet().into_string();
        let css = asset(THREE_CSS);
        assert!(html.contains(r#"rel="stylesheet""#), "{html}");
        assert!(html.contains(&format!(r#"href="{}""#, css.url())), "{html}");
        assert!(
            html.contains(&format!(r#"integrity="{}""#, css.integrity())),
            "{html}"
        );
    }
}
