// Browser E2E tests: real Chromium, real WebGL (SwiftShader).
// Run: cargo build --example e2e_fixture && npm run test:e2e
import { after, before, describe, test } from "node:test";
import assert from "node:assert/strict";

import { pixel, sleep, start, waitState } from "./harness.mjs";

let app;
before(async () => {
  app = await start();
});
after(async () => {
  await app?.close();
});

/** Fails on any CSP violation or page error. */
async function assertClean(page) {
  assert.deepEqual(await page.evaluate(() => window.__csp), [], "no CSP violations");
  assert.deepEqual(page.errors, [], "no page errors");
}

/** Reads a value from the `autumnThree` handle of `#id`. */
function read(page, id, fn) {
  return page.evaluate(
    ([id, source]) => new Function("h", `return (${source})(h)`)(document.getElementById(id).autumnThree),
    [id, fn.toString()],
  );
}

describe("rendering", () => {
  test("a scene renders the declared mesh over the background", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    const [r, g, b] = await pixel(page, "scene", 0.5, 0.5);
    assert.ok(r > 200 && g < 50 && b < 50, `center is red: ${[r, g, b]}`);
    const [r2, g2, b2] = await pixel(page, "scene", 0.02, 0.02);
    assert.ok(r2 < 50 && g2 < 50 && b2 > 200, `corner is the blue background: ${[r2, g2, b2]}`);
    assert.deepEqual(await page.evaluate(() => window.__events), [["three:ready", "scene", null]]);
    assert.equal(await page.locator("#scene > canvas").count(), 1);
    assert.equal(await page.locator("#scene > canvas").getAttribute("aria-hidden"), "true");
    assert.equal(await page.locator(".fallback").isVisible(), false, "fallback hides when ready");
    await assertClean(page);
  });

  test("the handle exposes Three.js for custom code", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    const info = await read(page, "scene", (h) => ({
      revision: h.THREE.REVISION,
      meshes: h.root.children.filter((o) => o.isMesh).length,
      fov: h.camera.fov,
    }));
    assert.deepEqual(info, { revision: "185", meshes: 1, fov: 50 });
    const shared = await page.evaluate(async () => {
      const THREE = await import("/static/_plugins/three/three.module.min.js");
      return THREE === document.getElementById("scene").autumnThree.THREE;
    });
    assert.equal(shared, true, "user modules share the plugin Three.js instance");
  });

  test("hand-written markup works and bad declarations only warn", async () => {
    const page = await app.open("/handwritten");
    await waitState(page, "scene", "ready");
    const [r, g, b] = await pixel(page, "scene");
    assert.ok(g > 200 && r < 50 && b < 50, `green box: ${[r, g, b]}`);
    const warnings = await page.evaluate(() => window.__warnings.join("\n"));
    for (const word of ["teapot", "laser", "javascript"]) assert.match(warnings, new RegExp(word));
    assert.equal(await read(page, "scene", (h) => h.camera.fov), 50, "bad fov uses the default");
    await assertClean(page);
  });

  test("named and custom aspect ratios size the scene", async () => {
    const page = await app.open("/aspect");
    await waitState(page, "custom", "ready");
    const square = await page.locator("#square").boundingBox();
    const custom = await page.locator("#custom").boundingBox();
    assert.ok(Math.abs(square.width - square.height) <= 1, JSON.stringify(square));
    assert.ok(Math.abs(custom.width / custom.height - 2) < 0.02, JSON.stringify(custom));
  });

  test("the canvas follows the element size", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    const before = await read(page, "scene", (h) => h.renderer.getContext().drawingBufferWidth);
    await page.setViewportSize({ width: 400, height: 600 });
    await page.waitForFunction(
      (w) => document.getElementById("scene").autumnThree.renderer.getContext().drawingBufferWidth !== w,
      before,
    );
    const { width, aspect } = await read(page, "scene", (h) => ({
      width: h.renderer.getContext().drawingBufferWidth,
      aspect: h.camera.aspect,
    }));
    assert.ok(width < before, `${width} < ${before}`);
    assert.ok(Math.abs(aspect - 16 / 9) < 0.02, `camera aspect ${aspect}`);
  });

  test("the pixel ratio is capped at 2", async () => {
    const page = await app.open("/basic", { deviceScaleFactor: 3 });
    await waitState(page, "scene", "ready");
    assert.equal(await read(page, "scene", (h) => h.renderer.getPixelRatio()), 2);
  });
});

describe("lights and environment", () => {
  test("room environment, declared lights, and default lights", async () => {
    const page = await app.open("/room");
    await waitState(page, "default-lights", "ready");
    assert.equal(await read(page, "scene", (h) => h.scene.environment !== null), true);
    assert.equal(await read(page, "scene", (h) => h.scene.children.filter((o) => o.isLight).length), 0);
    assert.deepEqual(
      await read(page, "lit", (h) => h.scene.children.filter((o) => o.isLight).map((o) => o.type).sort()),
      ["AmbientLight", "PointLight"],
    );
    assert.deepEqual(
      await read(page, "default-lights", (h) => h.scene.children.filter((o) => o.isLight).map((o) => o.type).sort()),
      ["DirectionalLight", "HemisphereLight"],
    );
    const [r, g, b] = await pixel(page, "default-lights");
    assert.ok(r + g + b > 150, `default lights light the sphere: ${[r, g, b]}`);
    await assertClean(page);
  });
});

describe("motion", () => {
  test("spin and turntable animate", async () => {
    const page = await app.open("/spin");
    await waitState(page, "scene", "ready");
    await sleep(500);
    const state = await read(page, "scene", (h) => ({
      looping: h.looping,
      root: h.root.rotation.y,
      mesh: h.root.children.find((o) => o.isMesh).rotation.y,
    }));
    assert.equal(state.looping, true);
    assert.ok(state.root !== 0 && state.mesh !== 0, JSON.stringify(state));
  });

  test("reduced motion stops automatic motion", async () => {
    const page = await app.open("/spin", { reducedMotion: "reduce" });
    await waitState(page, "scene", "ready");
    await sleep(500);
    const state = await read(page, "scene", (h) => ({
      looping: h.looping,
      root: h.root.rotation.y,
      mesh: h.root.children.find((o) => o.isMesh).rotation.y,
    }));
    assert.deepEqual(state, { looping: false, root: 0, mesh: 0 });
  });

  test("a scene can opt back into motion", async () => {
    const page = await app.open("/spin-animate", { reducedMotion: "reduce" });
    await waitState(page, "scene", "ready");
    await sleep(500);
    assert.notEqual(await read(page, "scene", (h) => h.root.children.find((o) => o.isMesh).rotation.y), 0);
  });

  test("off-screen and static scenes do not loop", async () => {
    const page = await app.open("/offscreen");
    await page.evaluate(() => (document.getElementById("spacer").style.height = "3000px"));
    await waitState(page, "below", "ready");
    await sleep(200);
    assert.equal(await read(page, "still", (h) => h.looping), false, "static scene");
    assert.equal(await read(page, "below", (h) => h.looping), false, "off-screen scene");
    const angle = await read(page, "below", (h) => h.root.children[0].rotation.y);
    await page.locator("#below").scrollIntoViewIfNeeded();
    await page.waitForFunction(() => document.getElementById("below").autumnThree.looping === true);
    await sleep(300);
    assert.notEqual(await read(page, "below", (h) => h.root.children[0].rotation.y), angle);
  });
});

describe("controls", () => {
  test("orbit controls move the camera on drag", async () => {
    const page = await app.open("/orbit");
    await waitState(page, "scene", "ready");
    const before = await read(page, "scene", (h) => h.camera.position.toArray());
    const box = await page.locator("#scene > canvas").boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2 + 150, box.y + box.height / 2, { steps: 5 });
    await page.mouse.up();
    const afterDrag = await read(page, "scene", (h) => h.camera.position.toArray());
    assert.notDeepEqual(afterDrag, before);
    assert.equal(await read(page, "scene", (h) => h.looping), false, "no loop for controls alone");
    await assertClean(page);
  });
});

describe("models", () => {
  test("a GLB loads, fits, centers, and plays its clip", async () => {
    const page = await app.open("/model");
    await waitState(page, "scene", "ready");
    const fitted = await read(page, "scene", (h) => {
      const box = new h.THREE.Box3().setFromObject(h.models[0]);
      const size = box.getSize(new h.THREE.Vector3());
      const center = box.getCenter(new h.THREE.Vector3());
      return { size: Math.max(size.x, size.y, size.z), center: center.toArray() };
    });
    assert.ok(Math.abs(fitted.size - 2) < 1e-3, `fitted size ${fitted.size}`);
    for (const c of fitted.center) assert.ok(Math.abs(c) < 1e-3, `centered ${fitted.center}`);
    await sleep(400);
    const mixers = await read(page, "scene", (h) => h.mixers.map((m) => m.time));
    assert.equal(mixers.length, 2);
    assert.ok(mixers.every((t) => t > 0), `clips play: ${mixers}`);
    assert.deepEqual(await page.evaluate(() => window.__events), [["three:ready", "scene", null]]);
    await assertClean(page);
  });

  test("a failed model shows the fallback and fires three:error", async () => {
    const page = await app.open("/model-missing");
    await waitState(page, "scene", "error");
    const events = await page.evaluate(() => window.__events);
    assert.equal(events.length, 1);
    assert.equal(events[0][0], "three:error");
    assert.match(events[0][2], /missing\.glb$/);
    assert.equal(await page.locator(".fallback").isVisible(), true);
    assert.equal(await page.locator("#scene > canvas").isVisible(), false);
    page.errors.length = 0; // The 404 is logged by the browser. That is expected.
  });
});

describe("progressive enhancement", () => {
  test("no WebGL: the fallback shows and three:error fires", async () => {
    const page = await app.open("/basic", {
      init: () => {
        const original = HTMLCanvasElement.prototype.getContext;
        HTMLCanvasElement.prototype.getContext = function (type, ...rest) {
          return /webgl/.test(type) ? null : original.call(this, type, ...rest);
        };
      },
    });
    await waitState(page, "scene", "error");
    assert.equal(await page.locator(".fallback").isVisible(), true);
    assert.equal(await page.locator("#scene > canvas").count(), 0);
    const events = await page.evaluate(() => window.__events);
    assert.deepEqual(events.map((e) => e[0]), ["three:error"]);
  });

  test("no JavaScript: the fallback shows and declarations stay hidden", async () => {
    const page = await app.open("/basic", { javaScriptEnabled: false });
    assert.equal(await page.locator(".fallback").isVisible(), true);
    assert.equal(await page.locator("[data-three-mesh]").isVisible(), false);
    const box = await page.locator("#scene").boundingBox();
    assert.ok(Math.abs(box.width / box.height - 16 / 9) < 0.02, `default 16/9 box: ${JSON.stringify(box)}`);
  });
});

describe("lifecycle", () => {
  test("an htmx swap builds the new scene and disposes the old one", async () => {
    const page = await app.open("/swap");
    await waitState(page, "first", "ready");
    await page.evaluate(() => {
      const el = document.getElementById("first");
      window.__old = { el, renderer: el.autumnThree.renderer };
    });
    await page.click("#next");
    await waitState(page, "second", "ready");
    const old = await page.evaluate(() => ({
      state: window.__old.el.getAttribute("data-three-state"),
      handle: window.__old.el.autumnThree === undefined,
      lost: window.__old.renderer.getContext().isContextLost(),
      canvases: document.querySelectorAll("canvas").length,
    }));
    assert.deepEqual(old, { state: "disposed", handle: true, lost: true, canvases: 1 });
    await assertClean(page);
  });

  test("removal without htmx disposes the scene; a move does not", async () => {
    const page = await app.open("/aspect");
    await waitState(page, "custom", "ready");
    await page.evaluate(() => {
      const moved = document.getElementById("square");
      window.__moved = moved.autumnThree.renderer;
      document.body.appendChild(moved);
      document.getElementById("custom").remove();
    });
    await page.waitForFunction(() => document.getElementById("square") && window.__moved);
    await sleep(200);
    const state = await page.evaluate(() => ({
      moved: document.getElementById("square").getAttribute("data-three-state"),
      same: document.getElementById("square").autumnThree?.renderer === window.__moved,
    }));
    assert.deepEqual(state, { moved: "ready", same: true });
  });

  test("a removed scene is disposed", async () => {
    const page = await app.open("/aspect");
    await waitState(page, "custom", "ready");
    await page.evaluate(() => {
      const el = document.getElementById("custom");
      window.__removed = { el, renderer: el.autumnThree.renderer };
      el.remove();
    });
    await page.waitForFunction(() => window.__removed.el.getAttribute("data-three-state") === "disposed");
    assert.equal(await page.evaluate(() => window.__removed.renderer.getContext().isContextLost()), true);
  });

  test("scans never initialize a scene twice", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    await page.evaluate(() => {
      document.body.dispatchEvent(new CustomEvent("htmx:afterSwap", { bubbles: true, detail: { target: document.body } }));
      document.body.appendChild(document.createElement("div"));
    });
    await sleep(200);
    assert.equal(await page.locator("#scene > canvas").count(), 1);
    assert.equal((await page.evaluate(() => window.__events)).length, 1);
  });

  test("restored markup with a stale canvas initializes again", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    await page.evaluate(() => {
      const html = document.getElementById("scene").outerHTML.replace('id="scene"', 'id="restored"');
      document.body.insertAdjacentHTML("beforeend", html);
    });
    await waitState(page, "restored", "ready");
    await page.waitForFunction(() => window.__events.length === 2);
    assert.equal(await page.locator("#restored > canvas").count(), 1);
    const [r] = await pixel(page, "restored");
    assert.ok(r > 200, "restored scene renders");
  });
});
