# CLAUDE.md: autumn-plugin-three

Three.js plugin for Autumn (`autumn-web` 0.8). Style: ASD-STE100 for docs
and comments (short sentences, active voice, simple present).

## Layout

| Path | Role |
|---|---|
| `src/assets.rs` | `THREE_ASSETS` bundle, `VENDORED` pins, import rewrites. |
| `src/plugin.rs` | `ThreePlugin` (installs the bundle). |
| `src/script.rs` | `three_script()`, `three_stylesheet()`. |
| `src/scene.rs` | Typed builder → `data-three-*` markup. |
| `assets/parse.js` | Pure attribute parsers. Node-tested. |
| `assets/init.js` | Runtime: build, loop, dispose. E2E-tested. |
| `assets/three.css` | Default size, hidden declarations, fallback. |
| `assets/*.min.js`, addons | Vendored Three.js 0.185.1. Do not edit by hand. |
| `scripts/vendor.sh` | Re-vendor Three.js. |
| `scripts/make_fixture_glb.py` | Writes `static/models/*.glb`. |
| `examples/three_demo.rs` | Demo app. |
| `examples/e2e_fixture.rs` | E2E routes, one per scenario. |
| `tests/js/` | `node --test` unit tests for `parse.js`. |
| `tests/e2e/` | Playwright + Chromium (SwiftShader) tests. |
| `docs/plan.md`, `docs/adr/` | Plan and decisions. |

## Rules

- Rust and JS stay in lockstep. A new attribute needs: builder method,
  `ATTR` entry in `parse.js`, parser test, E2E test, README row. The test
  `every_emitted_attribute_and_value_is_known_to_the_runtime` checks names.
- The builder never emits non-finite numbers (proptest).
- No inline script, inline style, import map, or `eval` (CSP).
- Vendored files change only through `scripts/vendor.sh`. Update
  `VENDORED` and `assets/manifest.json` in the same commit.
- `init.js` is embedded at compile time. Rebuild the fixture before E2E.

## Commands

```sh
cargo fmt && cargo clippy --all-targets -- -D warnings
cargo test
cargo llvm-cov --lib --fail-under-lines 85 --summary-only
npm ci && npm run test:unit
cargo build --example e2e_fixture && npm run test:e2e
E2E_SKIP_COVERAGE=1 node --test --test-name-pattern="<name>" tests/e2e/scene.test.mjs
```

## Gotchas

- `pkill -f e2e_fixture` also matches your own shell command. Use
  `pkill -f "examples/[e]2e_fixture"`.
- autumn-web 0.8.0 does not read `AUTUMN_SECURITY__HEADERS__CSP_NONCE__ENABLED`.
  The nonce E2E test writes an `autumn.toml` (`AUTUMN_MANIFEST_DIR`).
- htmx injects an inline `<style>`. Pages set
  `includeIndicatorStyles: false` for nonce mode.
