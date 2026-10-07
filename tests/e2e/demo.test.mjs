// E2E smoke test of examples/three_demo.rs: every scene builds, the htmx
// gallery swaps, the custom script gets three:ready, and the page is clean.
import { after, before, test } from "node:test";
import assert from "node:assert/strict";

import { start } from "./harness.mjs";

let app;
before(async () => {
  app = await start({ example: "three_demo", ready: "/" });
});
after(async () => {
  await app?.close();
});

test("the demo page builds every scene with no errors", async () => {
  const page = await app.open("/", { viewport: { width: 1000, height: 900 } });
  const states = () =>
    page.evaluate(() => [...document.querySelectorAll('[data-three="scene"]')].map((e) => e.getAttribute("data-three-state")));
  await page.waitForFunction(
    () => [...document.querySelectorAll('[data-three="scene"]')].every((e) => e.getAttribute("data-three-state") === "ready"),
    null,
    { timeout: 20_000 },
  );
  assert.equal((await states()).length, 5);

  // htmx gallery: a new scene replaces the old one.
  const old = await page.evaluate(() => {
    window.__oldGallery = document.querySelector("#gallery [data-three]");
    return document.querySelectorAll("canvas").length;
  });
  await page.click("button");
  await page.waitForFunction(() => {
    const scene = document.querySelector("#gallery [data-three]");
    return scene !== window.__oldGallery && scene?.getAttribute("data-three-state") === "ready";
  });
  assert.equal(await page.evaluate(() => window.__oldGallery.getAttribute("data-three-state")), "disposed");
  assert.equal(await page.evaluate(() => document.querySelectorAll("canvas").length), old);

  // static/js/demo.js: a click on the cube changes its color.
  const color = () => page.evaluate(() => document.getElementById("custom").autumnThree.root.children[0].material.color.getHex());
  const before = await color();
  await page.locator("#custom canvas").click();
  assert.notEqual(await color(), before);

  assert.deepEqual(await page.evaluate(() => window.__csp), []);
  assert.deepEqual(page.errors, []);
});
