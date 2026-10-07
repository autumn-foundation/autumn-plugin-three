// E2E: the plugin works under Autumn's CSP nonce mode.
// In nonce mode `style-src` and `script-src` allow 'self' + nonce only.
import { after, before, test } from "node:test";
import assert from "node:assert/strict";

import { pixel, start, waitState } from "./harness.mjs";

let app;
before(async () => {
  // autumn-web 0.8.0 documents AUTUMN_SECURITY__HEADERS__CSP_NONCE__ENABLED
  // but does not read it, so use autumn.toml.
  app = await start({ toml: "[security.headers.csp_nonce]\nenabled = true\n" });
});
after(async () => {
  await app?.close();
});

test("nonce mode: the CSP is strict and scenes still render", async () => {
  const response = await fetch(`${app.base}/basic`);
  const csp = response.headers.get("content-security-policy");
  assert.match(csp, /script-src 'self' 'nonce-/);
  assert.doesNotMatch(csp, /unsafe-inline/);

  const page = await app.open("/basic");
  await waitState(page, "scene", "ready");
  const [r] = await pixel(page, "scene");
  assert.ok(r > 200, "red cube renders");
  assert.deepEqual(await page.evaluate(() => window.__csp), []);
  assert.deepEqual(page.errors, []);
});

test("nonce mode: models and custom aspect ratios work", async () => {
  const page = await app.open("/model");
  await waitState(page, "scene", "ready");
  const aspect = await app.open("/aspect");
  await waitState(aspect, "custom", "ready");
  const box = await aspect.locator("#custom").boundingBox();
  assert.ok(Math.abs(box.width / box.height - 2) < 0.02, JSON.stringify(box));
  assert.deepEqual(await page.evaluate(() => window.__csp), []);
  assert.deepEqual(await aspect.evaluate(() => window.__csp), []);
});
