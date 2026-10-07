// Unit tests for assets/parse.js. Run: node --test tests/js
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  ATTR,
  CAMERA_DEFAULTS,
  GEOMETRIES,
  LIGHTS,
  MESH_DEFAULTS,
  parseArgs,
  parseAspect,
  parseClip,
  parseColor,
  parseKeyword,
  parseModelUrl,
  parseNumber,
  parseVec3,
  readScene,
} from "../../assets/parse.js";

/** A fake element: attributes plus children. */
function el(attrs = {}, children = []) {
  return {
    getAttribute: (name) => (name in attrs ? attrs[name] : null),
    hasAttribute: (name) => name in attrs,
    children,
  };
}

const BASE = "https://example.test/page";

test("parseNumber accepts finite numbers and clamps", () => {
  assert.equal(parseNumber("1.5", 0), 1.5);
  assert.equal(parseNumber(" -2 ", 0), -2);
  assert.equal(parseNumber("5", 0, 0, 1), 1);
  assert.equal(parseNumber("-5", 0, 0, 1), 0);
});

test("parseNumber rejects bad input", () => {
  for (const bad of [null, undefined, "", "abc", "NaN", "Infinity", "-Infinity", "1e999", "1px", "0x10"]) {
    assert.equal(parseNumber(bad, 7), 7, String(bad));
  }
});

test("parseVec3 reads three numbers or one number", () => {
  assert.deepEqual(parseVec3("1,2,3", [0, 0, 0]), [1, 2, 3]);
  assert.deepEqual(parseVec3(" 1 , -2.5 , 3 ", [0, 0, 0]), [1, -2.5, 3]);
  assert.deepEqual(parseVec3("2", [0, 0, 0]), [2, 2, 2]);
});

test("parseVec3 rejects bad input as a whole", () => {
  for (const bad of [null, "", "1,2", "1,2,3,4", "1,x,3", "NaN,0,0", "1,,3"]) {
    assert.deepEqual(parseVec3(bad, [9, 9, 9]), [9, 9, 9], String(bad));
  }
});

test("parseVec3 returns a copy of the fallback", () => {
  const fallback = [1, 2, 3];
  const out = parseVec3(null, fallback);
  out[0] = 99;
  assert.deepEqual(fallback, [1, 2, 3]);
});

test("parseColor reads #rgb and #rrggbb", () => {
  assert.equal(parseColor("#ff8800", 0), 0xff8800);
  assert.equal(parseColor("#FF8800", 0), 0xff8800);
  assert.equal(parseColor("#f80", 0), 0xff8800);
  assert.equal(parseColor(" #000000 ", 1), 0);
});

test("parseColor rejects other syntax", () => {
  for (const bad of [null, "", "red", "#ff88", "#gg0000", "ff8800", "rgb(1,2,3)", "#ff880000"]) {
    assert.equal(parseColor(bad, 42), 42, String(bad));
  }
});

test("parseKeyword accepts only listed values", () => {
  assert.equal(parseKeyword("orbit", ["none", "orbit"], "none"), "orbit");
  assert.equal(parseKeyword(" ORBIT ", ["none", "orbit"], "none"), "orbit");
  assert.equal(parseKeyword("fly", ["none", "orbit"], "none"), "none");
  assert.equal(parseKeyword(null, ["none", "orbit"], "none"), "none");
});

test("parseAspect reads w/h", () => {
  assert.equal(parseAspect("16/9"), 16 / 9);
  assert.equal(parseAspect("2.35/1"), 2.35);
  for (const bad of [null, "", "16", "16/0", "0/9", "-1/2", "a/b", "1/2/3"]) {
    assert.equal(parseAspect(bad), null, String(bad));
  }
});

test("parseArgs fills defaults and rejects bad sizes", () => {
  assert.deepEqual(parseArgs("box", "1,2,3"), [1, 2, 3]);
  assert.deepEqual(parseArgs("box", "2"), [2, 1, 1]);
  assert.deepEqual(parseArgs("box", null), [1, 1, 1]);
  assert.deepEqual(parseArgs("box", "-1,x,NaN"), [1, 1, 1]);
  assert.deepEqual(parseArgs("sphere", "1,2,3"), [1], "extra args are ignored");
  assert.deepEqual(parseArgs("cylinder", "0,1,2"), [0, 1, 2], "zero is allowed");
  assert.equal(parseArgs("teapot", "1"), null);
});

test("every geometry has defaults", () => {
  const kinds = Object.keys(GEOMETRIES).sort();
  assert.deepEqual(kinds, [
    "box", "capsule", "cone", "cylinder", "dodecahedron", "icosahedron", "octahedron",
    "plane", "ring", "sphere", "tetrahedron", "torus", "torus-knot",
  ]);
  for (const kind of kinds) {
    assert.deepEqual(parseArgs(kind, null), GEOMETRIES[kind], kind);
  }
});

test("parseModelUrl allows http(s) and relative URLs only", () => {
  assert.equal(parseModelUrl("/m.glb", BASE), "https://example.test/m.glb");
  assert.equal(parseModelUrl("m.glb", BASE), "https://example.test/m.glb");
  assert.equal(parseModelUrl("http://cdn.test/a.gltf", BASE), "http://cdn.test/a.gltf");
  for (const bad of [null, "", "   ", "javascript:alert(1)", "data:model/gltf+json,{}", "file:///etc/passwd", "blob:x"]) {
    assert.equal(parseModelUrl(bad, BASE), null, String(bad));
  }
});

test("parseClip reads * or a name", () => {
  assert.equal(parseClip("*"), "*");
  assert.equal(parseClip(" Walk "), "Walk");
  assert.equal(parseClip(""), null);
  assert.equal(parseClip(null), null);
});

test("readScene applies defaults to an empty scene", () => {
  const config = readScene(el({ [ATTR.scene]: "scene" }), BASE);
  assert.deepEqual(config, {
    aspect: null,
    background: null,
    camera: { ...CAMERA_DEFAULTS, position: [0, 1, 4], target: [0, 0, 0] },
    controls: "none",
    environment: null,
    turntable: 0,
    reducedAnimate: false,
    objects: [],
    warnings: [],
  });
});

test("readScene reads the scene attributes", () => {
  const config = readScene(
    el({
      [ATTR.scene]: "scene",
      [ATTR.aspect]: "1/1",
      [ATTR.background]: "#101018",
      [ATTR.camera]: "0,2,6",
      [ATTR.target]: "0,0.5,0",
      [ATTR.fov]: "40",
      [ATTR.controls]: "orbit-no-zoom",
      [ATTR.environment]: "room",
      [ATTR.turntable]: "-12.5",
      [ATTR.reduced]: "animate",
    }),
    BASE,
  );
  assert.equal(config.aspect, 1);
  assert.equal(config.background, 0x101018);
  assert.deepEqual(config.camera, { fov: 40, position: [0, 2, 6], target: [0, 0.5, 0] });
  assert.equal(config.controls, "orbit-no-zoom");
  assert.equal(config.environment, "room");
  assert.equal(config.turntable, -12.5);
  assert.equal(config.reducedAnimate, true);
});

test("readScene clamps the field of view", () => {
  assert.equal(readScene(el({ [ATTR.fov]: "500" }), BASE).camera.fov, 179);
  assert.equal(readScene(el({ [ATTR.fov]: "0" }), BASE).camera.fov, 1);
});

test("readScene reads a mesh with defaults", () => {
  const config = readScene(el({}, [el({ [ATTR.mesh]: "sphere" })]), BASE);
  assert.deepEqual(config.objects, [
    {
      type: "mesh",
      kind: "sphere",
      args: [0.5],
      ...MESH_DEFAULTS,
      position: [0, 0, 0],
      rotation: [0, 0, 0],
      scale: [1, 1, 1],
      spin: [0, 0, 0],
    },
  ]);
});

test("readScene reads every mesh attribute", () => {
  const [mesh] = readScene(
    el({}, [
      el({
        [ATTR.mesh]: "torus-knot",
        [ATTR.args]: "1,0.3",
        [ATTR.material]: "physical",
        [ATTR.color]: "#ff0000",
        [ATTR.emissive]: "#00ff00",
        [ATTR.metalness]: "2",
        [ATTR.roughness]: "0.25",
        [ATTR.opacity]: "0.5",
        [ATTR.wireframe]: "true",
        [ATTR.position]: "1,2,3",
        [ATTR.rotation]: "0,90,0",
        [ATTR.scale]: "2",
        [ATTR.spin]: "0,30,0",
      }),
    ]),
    BASE,
  ).objects;
  assert.deepEqual(mesh, {
    type: "mesh",
    kind: "torus-knot",
    args: [1, 0.3],
    material: "physical",
    color: 0xff0000,
    emissive: 0x00ff00,
    metalness: 1,
    roughness: 0.25,
    opacity: 0.5,
    wireframe: true,
    position: [1, 2, 3],
    rotation: [0, 90, 0],
    scale: [2, 2, 2],
    spin: [0, 30, 0],
  });
});

test("readScene skips unknown meshes with a warning", () => {
  const config = readScene(el({}, [el({ [ATTR.mesh]: "teapot" }), el({ [ATTR.mesh]: "box" })]), BASE);
  assert.equal(config.objects.length, 1);
  assert.equal(config.objects[0].kind, "box");
  assert.match(config.warnings[0], /teapot/);
});

test("readScene falls back to the standard material", () => {
  const [mesh] = readScene(el({}, [el({ [ATTR.mesh]: "box", [ATTR.material]: "chrome" })]), BASE).objects;
  assert.equal(mesh.material, "standard");
});

test("readScene reads models", () => {
  const config = readScene(
    el({}, [
      el({ [ATTR.model]: "/m.glb", [ATTR.fit]: "2", [ATTR.clip]: "Walk", [ATTR.position]: "0,-1,0" }),
      el({ [ATTR.model]: "/n.glb", [ATTR.fit]: "-1" }),
      el({ [ATTR.model]: "javascript:alert(1)" }),
    ]),
    BASE,
  );
  assert.deepEqual(config.objects, [
    {
      type: "model",
      src: "https://example.test/m.glb",
      fit: 2,
      clip: "Walk",
      position: [0, -1, 0],
      rotation: [0, 0, 0],
      scale: [1, 1, 1],
      spin: [0, 0, 0],
    },
    {
      type: "model",
      src: "https://example.test/n.glb",
      fit: null,
      clip: null,
      position: [0, 0, 0],
      rotation: [0, 0, 0],
      scale: [1, 1, 1],
      spin: [0, 0, 0],
    },
  ]);
  assert.match(config.warnings[0], /javascript/);
});

test("readScene reads lights with per-kind defaults", () => {
  const config = readScene(
    el({}, [
      el({ [ATTR.light]: "hemisphere", [ATTR.ground]: "#332211" }),
      el({ [ATTR.light]: "point", [ATTR.intensity]: "-3", [ATTR.position]: "1,2,3", [ATTR.color]: "#ff0000" }),
      el({ [ATTR.light]: "laser" }),
    ]),
    BASE,
  );
  assert.deepEqual(config.objects, [
    { type: "light", kind: "hemisphere", color: 0xffffff, ground: 0x332211, intensity: LIGHTS.hemisphere.intensity, position: LIGHTS.hemisphere.position },
    { type: "light", kind: "point", color: 0xff0000, ground: LIGHTS.point.ground, intensity: 0, position: [1, 2, 3] },
  ]);
  assert.match(config.warnings[0], /laser/);
});

test("readScene ignores children that are not declarations", () => {
  const config = readScene(el({}, [el({ [ATTR.fallback]: "" }), el({ class: "x" })]), BASE);
  assert.deepEqual(config.objects, []);
  assert.deepEqual(config.warnings, []);
});

test("every light kind has defaults", () => {
  assert.deepEqual(Object.keys(LIGHTS).sort(), ["ambient", "directional", "hemisphere", "point", "spot"]);
  for (const kind of Object.keys(LIGHTS)) {
    const light = LIGHTS[kind];
    assert.equal(typeof light.intensity, "number", kind);
    assert.equal(light.position.length, 3, kind);
  }
});

test("attribute names are unique and prefixed", () => {
  const names = Object.values(ATTR);
  assert.equal(new Set(names).size, names.length);
  for (const name of names) {
    assert.match(name, /^data-three(-[a-z]+)*$/);
  }
});
