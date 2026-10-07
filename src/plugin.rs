//! [`ThreePlugin`]: installs the Three.js assets in an Autumn app.
//!
//! The plugin installs [`THREE_ASSETS`] through `AppBuilder::plugin_assets`.
//! It reads no configuration and adds no startup hooks.

use std::borrow::Cow;

use autumn_web::app::AppBuilder;
use autumn_web::plugin::Plugin;

use crate::assets::THREE_ASSETS;

/// The plugin name in Autumn diagnostics.
pub const PLUGIN_NAME: &str = "autumn-plugin-three";

/// Installs the Three.js assets in an Autumn app.
///
/// ```rust,no_run
/// use autumn_plugin_three::ThreePlugin;
///
/// # async fn run() {
/// autumn_web::app()
///     .plugin(ThreePlugin::new())
///     .run()
///     .await;
/// # }
/// ```
///
/// Then put [`three_script`](crate::three_script) and
/// [`three_stylesheet`](crate::three_stylesheet) in the page `<head>`.
#[derive(Debug, Default)]
#[must_use]
pub struct ThreePlugin;

impl ThreePlugin {
    /// Makes the plugin. It reads no configuration.
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for ThreePlugin {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(PLUGIN_NAME)
    }

    fn build(self, app: AppBuilder) -> AppBuilder {
        app.plugin_assets(&THREE_ASSETS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{CORE_JS, INIT_JS, MODULE_JS, THREE_ASSETS, THREE_CSS};
    use autumn_web::assets::{PLUGIN_ASSETS_ROUTE_MARKER, asset_url};
    use autumn_web::plugin_conformance::{ConformanceConfig, run_conformance};
    use autumn_web::route_listing::{RouteClassification, RouteSource};
    use autumn_web::test::{TestApp, TestClient};

    const JS: &str = "text/javascript; charset=utf-8";
    const CSS: &str = "text/css; charset=utf-8";
    const IMMUTABLE: &str = "public, max-age=31536000, immutable";
    const REVALIDATE: &str = "public, max-age=0, must-revalidate";

    fn client() -> TestClient {
        TestApp::new().plugin(ThreePlugin::new()).build()
    }

    fn url(path: &str) -> String {
        THREE_ASSETS.url(path)
    }

    #[tokio::test]
    async fn every_file_serves_at_its_fingerprinted_url() {
        let client = client();
        for asset in THREE_ASSETS.iter() {
            let response = client.get(asset.url()).send().await;
            let is_css = std::path::Path::new(asset.logical_path())
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("css"));
            let expected = if is_css { CSS } else { JS };
            response
                .assert_ok()
                .assert_header("content-type", expected)
                .assert_header("cache-control", IMMUTABLE);
            assert_eq!(
                response.body.as_slice(),
                asset.bytes(),
                "{}",
                asset.logical_path()
            );
        }
    }

    #[tokio::test]
    async fn module_files_serve_at_plain_urls_with_etags() {
        // Module imports resolve to plain URLs. They must work.
        let client = client();
        for path in [CORE_JS, MODULE_JS, INIT_JS, THREE_CSS] {
            let plain = format!("/static/_plugins/three/{path}");
            let response = client.get(&plain).send().await;
            response
                .assert_ok()
                .assert_header("cache-control", REVALIDATE);
            let etag = response.header("etag").expect("etag").to_owned();
            client
                .get(&plain)
                .header("if-none-match", &etag)
                .send()
                .await
                .assert_status(304);
        }
    }

    #[tokio::test]
    async fn unbundled_and_stale_paths_are_not_found() {
        let client = client();
        for path in [
            "/static/_plugins/three/manifest.json",
            "/static/_plugins/three/THREE-LICENSE",
            "/static/_plugins/three/init.00000000.js",
            "/static/_plugins/three/three.webgpu.js",
        ] {
            client.get(path).send().await.assert_status(404);
        }
    }

    #[tokio::test]
    async fn asset_url_resolves_the_installed_bundle() {
        let _client = client();
        for path in [CORE_JS, MODULE_JS, INIT_JS, THREE_CSS] {
            assert_eq!(asset_url(&format!("_plugins/three/{path}")), url(path));
        }
    }

    #[test]
    fn bundle_routes_are_public_plugin_routes() {
        let app = autumn_web::app().plugin(ThreePlugin::new());
        let infos = app.plugin_route_infos().expect("route infos");
        let routes: Vec<_> = infos
            .iter()
            .filter(|info| info.path.starts_with("/static/_plugins/three/"))
            .collect();
        assert_eq!(routes.len(), 20, "ten files, two URLs each: {infos:?}");
        for info in routes {
            assert_eq!(info.method, "GET");
            assert_eq!(info.classification, RouteClassification::Public);
            assert_eq!(info.middleware, [PLUGIN_ASSETS_ROUTE_MARKER]);
            assert_eq!(info.source, RouteSource::Plugin(PLUGIN_NAME.to_owned()));
        }
    }

    #[test]
    fn plugin_passes_conformance() {
        let app = autumn_web::app().plugin(ThreePlugin::new());
        let infos = app.plugin_route_infos().expect("route infos");
        let report = run_conformance(&ConformanceConfig::new(PLUGIN_NAME), &infos);
        assert!(report.passed(), "{}", report.to_text_report());
    }

    #[tokio::test]
    async fn installing_the_plugin_twice_is_harmless() {
        let client = TestApp::new()
            .plugin(ThreePlugin::new())
            .plugin(ThreePlugin::new())
            .build();
        client.get(&url(INIT_JS)).send().await.assert_ok();
    }
}
