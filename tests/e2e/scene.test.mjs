// Browser E2E tests: real Chromium, real WebGL (SwiftShader).
// Run: cargo build --example e2e_fixture && npm run test:e2e
import { after, before, describe, test } from "node:test";
import assert from "node:assert/strict";

import { lineCoverage, pixel, sleep, start, waitState } from "./harness.mjs";

/** Minimum line coverage of init.js over this suite. */
const MIN_INIT_COVERAGE = 95;

let app;
before(async () => {
  app = await start();
});
after(async () => {
  await app?.close();
  if (process.env.E2E_SKIP_COVERAGE) return; // Filtered dev runs.
  const init = lineCoverage("init");
  console.log(`init.js line coverage: ${init?.percent.toFixed(1)}% (uncovered lines: ${init?.uncovered.join(", ")})`);
  assert.ok(init && init.percent >= MIN_INIT_COVERAGE, `init.js coverage ${init?.percent} < ${MIN_INIT_COVERAGE}`);
});

/** Fails on any CSP violation or page error. */
async function assertClean(page) {
  assert.deepEqual(await page.evaluate(() => window.__csp), [], "no CSP violations");
  assert.deepEqual(page.errors, [], "no page errors");
}

/**
 * Waits until `fn(handle)` of `#id` is truthy. It polls from Node: the page
 * CSP blocks the `eval` that `page.waitForFunction` needs for closures.
 */
async function until(page, id, fn, timeout = 10_000) {
  const deadline = Date.now() + timeout;
  for (;;) {
    const ok = await read(page, id, (h) => h).then(
      (h) => h !== undefined && read(page, id, fn),
      () => false,
    );
    if (ok) return;
    if (Date.now() > deadline) throw new Error(`timeout: ${fn}`);
    await sleep(50);
  }
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
    assert.deepEqual(await page.evaluate(() => window.__details), [true], "detail is the handle");
    assert.equal(await read(page, "scene", (h) => h.renderer.getClearAlpha()), 1, "solid background");
    assert.equal(await page.locator("#scene > canvas").count(), 1);
    assert.equal(await page.locator("#scene > canvas").getAttribute("aria-hidden"), "true");
    assert.equal(await page.locator(".fallback").isVisible(), false, "fallback hides when ready");
    await assertClean(page);
  });

  test("a page module after three_script() gets three:ready", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    assert.deepEqual(await page.evaluate(() => window.__lateReady), ["scene"]);
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

describe("kinds", () => {
  test("every geometry, material, and light kind builds", async () => {
    const page = await app.open("/kinds");
    await waitState(page, "scene", "ready");
    const info = await read(page, "scene", (h) => ({
      geometries: h.root.children.map((m) => m.geometry.type),
      materials: [...new Set(h.root.children.map((m) => m.material.type))].sort(),
      lights: h.scene.children.filter((o) => o.isLight).map((o) => o.type).sort(),
      ring: (({ transparent, opacity, wireframe, side }) => ({ transparent, opacity, wireframe, side }))(h.root.children[12].material),
    }));
    assert.deepEqual(info.geometries, [
      "BoxGeometry", "SphereGeometry", "PlaneGeometry", "TorusGeometry", "TorusKnotGeometry",
      "CylinderGeometry", "ConeGeometry", "CapsuleGeometry", "IcosahedronGeometry",
      "DodecahedronGeometry", "OctahedronGeometry", "TetrahedronGeometry", "RingGeometry",
    ]);
    assert.deepEqual(info.materials, [
      "MeshBasicMaterial", "MeshLambertMaterial", "MeshNormalMaterial", "MeshPhongMaterial",
      "MeshPhysicalMaterial", "MeshStandardMaterial",
    ]);
    assert.deepEqual(info.lights, ["AmbientLight", "DirectionalLight", "HemisphereLight", "PointLight", "SpotLight"]);
    assert.deepEqual(info.ring, { transparent: true, opacity: 0.5, wireframe: true, side: 2 });
    await assertClean(page);
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
    await until(page, "scene", (h) => h.root.rotation.y > 0.2);
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
    await until(page, "scene", (h) => h.root.children.find((o) => o.isMesh).rotation.y > 0.2);
  });

  test("a reduced-motion change at runtime stops the loop", async () => {
    const page = await app.open("/spin");
    await waitState(page, "scene", "ready");
    await page.waitForFunction(() => document.getElementById("scene").autumnThree.looping === true);
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.waitForFunction(() => document.getElementById("scene").autumnThree.looping === false);
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await page.waitForFunction(() => document.getElementById("scene").autumnThree.looping === true);
  });

  test("off-screen and static scenes do not loop", async () => {
    const page = await app.open("/offscreen");
    await waitState(page, "below", "ready");
    await sleep(300);
    assert.equal(await read(page, "still", (h) => h.looping), false, "static scene");
    assert.equal(await read(page, "below", (h) => h.looping), false, "off-screen scene never starts");
    assert.equal(await read(page, "below", (h) => h.root.children[0].rotation.y), 0);
    await page.locator("#below").scrollIntoViewIfNeeded();
    await until(page, "below", (h) => h.looping && h.root.children[0].rotation.y > 0.1);
  });
});

describe("controls", () => {
  test("orbit controls move the camera on drag", async () => {
    const page = await app.open("/orbit");
    await waitState(page, "scene", "ready");
    const before = await read(page, "scene", (h) => h.camera.position.toArray());
    const frames = await read(page, "scene", (h) => h.renderer.info.render.frame);
    const box = await page.locator("#scene > canvas").boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2 + 150, box.y + box.height / 2, { steps: 5 });
    await page.mouse.up();
    const afterDrag = await read(page, "scene", (h) => h.camera.position.toArray());
    assert.notDeepEqual(afterDrag, before);
    await until(page, "scene", new Function(`return (h) => h.renderer.info.render.frame > ${frames}`)());
    assert.equal(await read(page, "scene", (h) => h.looping), false, "no loop for controls alone");
    const distance = await read(page, "scene", (h) => h.camera.position.length());
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.wheel(0, 400);
    await until(page, "scene", new Function(`return (h) => Math.abs(h.camera.position.length() - ${distance}) > 0.01`)());
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

  test("a GLB with an embedded texture works under the default CSP", async () => {
    const page = await app.open("/textured");
    await waitState(page, "scene", "ready");
    await page.waitForFunction(() => {
      const map = document.getElementById("scene").autumnThree.models[0]?.getObjectByName("Tile")?.material.map;
      return map?.image;
    });
    const red = await pixel(page, "scene", 0.35, 0.3);
    const white = await pixel(page, "scene", 0.65, 0.7);
    assert.ok(red[0] > 200 && red[1] < 50 && red[2] < 50, `top left texel is red: ${red}`);
    assert.ok(white.slice(0, 3).every((c) => c > 200), `bottom right texel is white: ${white}`);
    const shared = await read(page, "scene", (h) => {
      const a = h.models[0].getObjectByName("Tile").material.map;
      const b = h.models[0].getObjectByName("Tile2").material.map;
      return { distinct: a !== b, sameImage: a.image === b.image, filters: [a.magFilter, b.magFilter] };
    });
    assert.deepEqual(shared, { distinct: true, sameImage: true, filters: [1003, 1006] }, "one image, two samplers");
    assert.equal(await read(page, "scene", (h) => h.models[0].getObjectByName("Tile").material.map.userData.source), "fixture", "image extras");
    await assertClean(page);
  });

  test("a failed model shows the fallback and fires three:error", async () => {
    const page = await app.open("/model-missing");
    await waitState(page, "scene", "error");
    const events = await page.evaluate(() => window.__events);
    assert.equal(events.length, 1);
    assert.equal(events[0][0], "three:error");
    assert.match(events[0][2], /missing\.glb$/);
    assert.deepEqual(await page.evaluate(() => window.__details), [true], "detail.error is an Error");
    assert.equal(await page.locator(".fallback").isVisible(), true);
    assert.equal(await page.locator("#scene > canvas").isVisible(), false);
    page.errors.length = 0; // The browser logs the 404. This is correct.
  });
});

describe("model edge cases", () => {
  test("a failed model without a fallback keeps the other objects", async () => {
    const page = await app.open("/model-no-fallback");
    await waitState(page, "scene", "error");
    assert.equal(await page.locator("#scene > canvas").isVisible(), true);
    const [r] = await pixel(page, "scene");
    assert.ok(r > 200, "the red cube still renders");
    assert.deepEqual((await page.evaluate(() => window.__events)).map((e) => e[0]), ["three:error"]);
  });

  test("a missing clip name warns and plays nothing", async () => {
    const page = await app.open("/model-clip-missing");
    await waitState(page, "scene", "ready");
    assert.equal(await read(page, "scene", (h) => h.mixers.length), 0);
    assert.match(await page.evaluate(() => window.__warnings.join("\n")), /no clip "Nope"/);
  });

  test("a model that loads after removal is freed", async () => {
    const page = await app.open("/basic");
    let release;
    let requested;
    const gate = new Promise((r) => (release = r));
    const inFlight = new Promise((r) => (requested = r));
    await page.route("**/gem.glb", async (route) => {
      requested();
      await gate;
      await route.continue();
    });
    await page.evaluate(() => {
      document.body.insertAdjacentHTML(
        "beforeend",
        '<div id="slow" data-three="scene"><div hidden data-three-model="/static/models/gem.glb"></div></div>',
      );
    });
    await inFlight;
    await page.evaluate(() => {
      window.__slow = document.getElementById("slow");
      window.__slow.remove();
    });
    await page.waitForFunction(() => window.__slow.getAttribute("data-three-state") === "disposed");
    release();
    await sleep(500);
    assert.equal(await page.evaluate(() => window.__slow.getAttribute("data-three-state")), "disposed");
    await assertClean(page);
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

  test("an unexpected build error shows the fallback", async () => {
    const page = await app.open("/basic", {
      init: () => {
        window.ResizeObserver = class {
          constructor() {
            throw new Error("boom");
          }
        };
      },
    });
    await waitState(page, "scene", "error");
    assert.equal(await page.locator(".fallback").isVisible(), true);
    assert.equal(await page.locator("#scene > canvas").count(), 0);
    assert.deepEqual((await page.evaluate(() => window.__events)).map((e) => e[0]), ["three:error"]);
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
    await sleep(200);
    const state = await page.evaluate(() => ({
      moved: document.getElementById("square").getAttribute("data-three-state"),
      same: document.getElementById("square").autumnThree?.renderer === window.__moved,
      canvases: document.querySelectorAll("canvas").length,
    }));
    assert.deepEqual(state, { moved: "ready", same: true, canvases: 1 }, "#custom is gone, #square stays");
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

  test("init.js added after page load still scans the page", async () => {
    const page = await app.open("/late-script");
    await page.waitForLoadState("load");
    await page.evaluate(() => {
      const script = document.createElement("script");
      script.type = "module";
      script.src = document.body.dataset.init;
      document.head.append(script);
    });
    await waitState(page, "scene", "ready");
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

describe("parameters", () => {
  test("camera, target, transform, material, and light values reach Three.js", async () => {
    const page = await app.open("/params");
    await waitState(page, "scene", "ready");
    const v = await read(page, "scene", (h) => {
      const [still, spinning] = h.root.children.filter((o) => o.isMesh);
      const dir = new h.THREE.Vector3();
      h.camera.getWorldDirection(dir);
      const sun = h.scene.children.find((o) => o.isDirectionalLight);
      const sky = h.scene.children.find((o) => o.isHemisphereLight);
      return {
        fov: h.camera.fov,
        target: h.controls.target.toArray(),
        lookAt: dir.toArray().map((n) => Math.round(n * 1000) / 1000 + 0),
        zoom: h.controls.enableZoom,
        alpha: h.renderer.getClearAlpha(),
        rotationX: still.rotation.x,
        emissive: spinning.material.emissive.getHex(),
        sun: [sun.color.getHex(), sun.intensity, sun.position.toArray()],
        sky: [sky.color.getHex(), sky.groundColor.getHex(), sky.intensity],
      };
    });
    const expected = new Float64Array([1, 0, -5]);
    const len = Math.hypot(...expected);
    assert.equal(v.fov, 30);
    assert.deepEqual(v.target, [1, 0, 0]);
    assert.deepEqual(v.lookAt, [...expected].map((n) => Math.round((n / len) * 1000) / 1000 + 0));
    assert.equal(v.zoom, false, "orbit-no-zoom");
    assert.equal(v.alpha, 0, "transparent background");
    assert.ok(Math.abs(v.rotationX - Math.PI / 2) < 1e-6, "degrees become radians");
    assert.equal(v.emissive, 0x00ff00);
    assert.deepEqual(v.sun, [0xff0000, 3, [1, 2, 3]]);
    assert.deepEqual(v.sky, [0x0000ff, 0x00ff00, 0.5]);
  });

  test("spin axes and turntable direction follow the values", async () => {
    const page = await app.open("/params");
    await waitState(page, "scene", "ready");
    await until(page, "scene", (h) => h.root.rotation.y < -0.3);
    const r = await read(page, "scene", (h) => {
      const spinning = h.root.children.filter((o) => o.isMesh)[1];
      return { x: spinning.rotation.x, y: spinning.rotation.y, z: spinning.rotation.z, turn: h.root.rotation.y };
    });
    assert.ok(r.x > 0 && r.z > 0 && r.y === 0, JSON.stringify(r));
    assert.ok(Math.abs(r.z / r.x - 2) < 0.01, `z spins twice as fast as x: ${JSON.stringify(r)}`);
    assert.ok(r.turn < 0, "negative turntable turns clockwise");
  });

  test("orbit-no-zoom ignores the mouse wheel", async () => {
    const page = await app.open("/params");
    await waitState(page, "scene", "ready");
    const before = await read(page, "scene", (h) => h.camera.position.distanceTo(h.controls.target));
    const box = await page.locator("#scene > canvas").boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.wheel(0, 400);
    await sleep(200);
    const after = await read(page, "scene", (h) => h.camera.position.distanceTo(h.controls.target));
    assert.ok(Math.abs(after - before) < 1e-6, `${before} -> ${after}`);
  });

  test("requestRender() and update() work from custom code", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    const frames = await read(page, "scene", (h) => h.renderer.info.render.frame);
    await page.evaluate(() => document.getElementById("scene").autumnThree.requestRender());
    await until(page, "scene", new Function(`return (h) => h.renderer.info.render.frame > ${frames}`)());
    assert.equal(await read(page, "scene", (h) => h.looping), false);
    await page.evaluate(() => {
      const h = document.getElementById("scene").autumnThree;
      const { THREE } = h;
      const track = new THREE.NumberKeyframeTrack(".rotation[y]", [0, 1], [0, Math.PI]);
      const mixer = new THREE.AnimationMixer(h.root.children[0]);
      mixer.clipAction(new THREE.AnimationClip("turn", 1, [track])).play();
      h.mixers.push(mixer);
      h.update();
    });
    await until(page, "scene", (h) => h.looping && h.mixers[0].time > 0);
  });
});

describe("robustness", () => {
  test("a failed scene stays failed when other content swaps in", async () => {
    const page = await app.open("/model-missing");
    await waitState(page, "scene", "error");
    await page.evaluate(() => {
      const detached = document.createElement("div");
      document.body.dispatchEvent(new CustomEvent("htmx:afterSwap", { bubbles: true, detail: { target: detached } }));
      document.body.append(document.createElement("p"));
    });
    await sleep(500);
    assert.equal(await page.evaluate(() => document.getElementById("scene").getAttribute("data-three-state")), "error");
    assert.equal((await page.evaluate(() => window.__events)).length, 1, "no second build");
    page.errors.length = 0;
  });

  test("new children rebuild the scene", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    await page.evaluate(() => {
      document.getElementById("scene").innerHTML =
        '<div hidden data-three-mesh="box" data-three-args="2,2,2" data-three-material="basic" data-three-color="#00ff00"></div>';
    });
    await page.waitForFunction(() => window.__events.length === 2);
    await waitState(page, "scene", "ready");
    assert.equal(await page.locator("#scene > canvas").count(), 1);
    const [r, g] = await pixel(page, "scene");
    assert.ok(g > 200 && r < 50, `the new green box renders: ${[r, g]}`);
    await assertClean(page);
  });

  test("a lost WebGL context shows the fallback", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    await page.evaluate(() => {
      document.getElementById("scene").autumnThree.renderer.getContext().getExtension("WEBGL_lose_context").loseContext();
    });
    await waitState(page, "scene", "error");
    assert.equal(await page.locator(".fallback").isVisible(), true);
    assert.equal(await page.locator("#scene > canvas").count(), 0);
    assert.deepEqual((await page.evaluate(() => window.__events)).map((e) => e[0]), ["three:ready", "three:error"]);
  });

  test("a failed addon import is retried by the next scene", async () => {
    const page = await app.open("/basic");
    await waitState(page, "scene", "ready");
    await page.route("**/OrbitControls.js", (route) => route.abort());
    const html = (id) =>
      `<div id="${id}" data-three="scene" data-three-controls="orbit"><div hidden data-three-mesh="box"></div></div>`;
    await page.evaluate((h) => document.body.insertAdjacentHTML("beforeend", h), html("first"));
    await waitState(page, "first", "error");
    await page.unroute("**/OrbitControls.js");
    await page.evaluate((h) => document.body.insertAdjacentHTML("beforeend", h), html("second"));
    await waitState(page, "second", "ready");
    page.errors.length = 0; // The aborted request logs an error. This is correct.
  });

  for (const [name, addon, decl] of [
    ["controls", "OrbitControls.js", 'data-three-controls="orbit"'],
    ["environment", "RoomEnvironment.js", 'data-three-environment="room"'],
    ["models", "GLTFLoader.js", ""],
  ]) {
    test(`removal while ${name} load frees the scene`, async () => {
      const page = await app.open("/basic");
      await waitState(page, "scene", "ready");
      let release;
      let requested;
      const gate = new Promise((r) => (release = r));
      const inFlight = new Promise((r) => (requested = r));
      await page.route(`**/${addon}`, async (route) => {
        requested();
        await gate;
        await route.continue();
      });
      const model = name === "models" ? '<div hidden data-three-model="/static/models/gem.glb"></div>' : "";
      await page.evaluate(
        (h) => document.body.insertAdjacentHTML("beforeend", h),
        `<div id="late" data-three="scene" ${decl}><div hidden data-three-mesh="box"></div>${model}</div>`,
      );
      await inFlight;
      await page.evaluate(() => {
        window.__late = document.getElementById("late");
        window.__late.remove();
      });
      release();
      await page.waitForFunction(() => window.__late.getAttribute("data-three-state") === "disposed");
      await sleep(300);
      assert.equal(await page.evaluate(() => window.__late.autumnThree), undefined);
      await assertClean(page);
    });
  }

  test("removing scenes with controls, environment, and models frees them", async () => {
    for (const [path, id] of [["/orbit", "scene"], ["/room", "scene"], ["/model", "scene"], ["/textured", "scene"]]) {
      const page = await app.open(path);
      await waitState(page, id, "ready");
      await page.evaluate((id) => {
        const el = document.getElementById(id);
        window.__gone = { el, renderer: el.autumnThree.renderer };
        el.remove();
      }, id);
      await page.waitForFunction(() => window.__gone.el.getAttribute("data-three-state") === "disposed");
      assert.equal(await page.evaluate(() => window.__gone.renderer.getContext().isContextLost()), true, path);
      await assertClean(page);
    }
  });

  test("an embedded image that does not decode logs an error", async () => {
    const page = await app.open("/broken-texture");
    await waitState(page, "scene", "ready");
    await page.waitForFunction(() => document.getElementById("scene").autumnThree.models.length === 1);
    assert.equal(await read(page, "scene", (h) => h.models[0].getObjectByName("Tile").material.map), null);
    assert.ok(page.errors.some((e) => /Couldn't load texture/.test(e)), page.errors.join("\n"));
  });

  test("a glTF with an external image uses the stock image path", async () => {
    const page = await app.open("/textured-external");
    await waitState(page, "scene", "ready");
    await page.waitForFunction(() => document.getElementById("scene").autumnThree.models[0]?.getObjectByName("Tile")?.material.map?.image);
    const red = await pixel(page, "scene", 0.35, 0.3);
    assert.ok(red[0] > 200 && red[1] < 50 && red[2] < 50, `top left texel is red: ${red}`);
    await assertClean(page);
  });
});
