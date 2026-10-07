// autumn-plugin-three: attribute parsers.
//
// Pure functions. No Three.js, no DOM globals. init.js uses them, and
// tests/js runs them in Node. Bad input never throws: each parser returns
// its fallback.

/** Every attribute the plugin reads or writes. */
export const ATTR = Object.freeze({
  scene: "data-three",
  aspect: "data-three-aspect",
  background: "data-three-background",
  camera: "data-three-camera",
  target: "data-three-target",
  fov: "data-three-fov",
  controls: "data-three-controls",
  environment: "data-three-environment",
  turntable: "data-three-turntable",
  reduced: "data-three-reduced",
  fallback: "data-three-fallback",
  mesh: "data-three-mesh",
  args: "data-three-args",
  material: "data-three-material",
  color: "data-three-color",
  emissive: "data-three-emissive",
  metalness: "data-three-metalness",
  roughness: "data-three-roughness",
  opacity: "data-three-opacity",
  wireframe: "data-three-wireframe",
  position: "data-three-position",
  rotation: "data-three-rotation",
  scale: "data-three-scale",
  spin: "data-three-spin",
  model: "data-three-model",
  fit: "data-three-fit",
  clip: "data-three-clip",
  light: "data-three-light",
  ground: "data-three-ground",
  intensity: "data-three-intensity",
  state: "data-three-state",
  canvas: "data-three-canvas",
});

/** Default sizes per geometry, in `data-three-args` order. */
export const GEOMETRIES = Object.freeze({
  "box": [1, 1, 1],
  "sphere": [0.5],
  "plane": [1, 1],
  "torus": [0.5, 0.2],
  "torus-knot": [0.5, 0.15],
  "cylinder": [0.5, 0.5, 1],
  "cone": [0.5, 1],
  "capsule": [0.3, 0.6],
  "icosahedron": [0.5],
  "dodecahedron": [0.5],
  "octahedron": [0.5],
  "tetrahedron": [0.5],
  "ring": [0.25, 0.5],
});

/** Allowed `data-three-material` values. The first is the default. */
export const MATERIALS = Object.freeze(["standard", "physical", "basic", "lambert", "phong", "normal"]);

/** Allowed `data-three-controls` values. The first is the default. */
export const CONTROLS = Object.freeze(["none", "orbit", "orbit-no-zoom"]);

/** Allowed `data-three-environment` values. */
export const ENVIRONMENTS = Object.freeze(["room"]);

/** `data-three-reduced` value that keeps motion for reduced-motion users. */
export const REDUCED_ANIMATE = "animate";

/** Defaults per light kind. */
export const LIGHTS = Object.freeze({
  "ambient": Object.freeze({ intensity: 1, ground: 0x444444, position: [0, 0, 0] }),
  "directional": Object.freeze({ intensity: 2, ground: 0x444444, position: [3, 5, 4] }),
  "point": Object.freeze({ intensity: 20, ground: 0x444444, position: [2, 3, 2] }),
  "spot": Object.freeze({ intensity: 40, ground: 0x444444, position: [0, 4, 2] }),
  "hemisphere": Object.freeze({ intensity: 2, ground: 0x444444, position: [0, 1, 0] }),
});

/** Camera defaults. */
export const CAMERA_DEFAULTS = Object.freeze({ fov: 50, position: [0, 1, 4], target: [0, 0, 0] });

/** Mesh material defaults. */
export const MESH_DEFAULTS = Object.freeze({
  material: "standard",
  color: 0xffffff,
  emissive: 0x000000,
  metalness: 0,
  roughness: 0.5,
  opacity: 1,
  wireframe: false,
});

const NUMBER = /^[+-]?(\d+\.?\d*|\.\d+)(e[+-]?\d+)?$/i;
const HEX6 = /^#([0-9a-f]{6})$/i;
const HEX3 = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/i;

/** Parses a finite decimal number, clamped to `[min, max]`. */
export function parseNumber(value, fallback, min = -Infinity, max = Infinity) {
  if (typeof value !== "string") return fallback;
  const text = value.trim();
  if (!NUMBER.test(text)) return fallback;
  const n = Number(text);
  if (!Number.isFinite(n)) return fallback;
  return Math.min(max, Math.max(min, n));
}

/** Parses `x,y,z` or one number for all three. Bad input returns a copy of `fallback`. */
export function parseVec3(value, fallback) {
  if (typeof value === "string") {
    const parts = value.split(",");
    if (parts.length === 1 || parts.length === 3) {
      const nums = parts.map((p) => parseNumber(p, NaN));
      if (nums.every(Number.isFinite)) {
        return nums.length === 1 ? [nums[0], nums[0], nums[0]] : nums;
      }
    }
  }
  return [...fallback];
}

/** Parses `#rrggbb` or `#rgb` to `0xrrggbb`. */
export function parseColor(value, fallback) {
  if (typeof value !== "string") return fallback;
  const text = value.trim();
  const six = HEX6.exec(text);
  if (six) return Number.parseInt(six[1], 16);
  const three = HEX3.exec(text);
  if (three) return Number.parseInt(three[1] + three[1] + three[2] + three[2] + three[3] + three[3], 16);
  return fallback;
}

/** Returns the lower-case `value` when `allowed` has it, else `fallback`. */
export function parseKeyword(value, allowed, fallback) {
  if (typeof value !== "string") return fallback;
  const key = value.trim().toLowerCase();
  return allowed.includes(key) ? key : fallback;
}

/** Parses `w/h` to a positive ratio, else `null`. */
export function parseAspect(value) {
  if (typeof value !== "string") return null;
  const parts = value.split("/");
  if (parts.length !== 2) return null;
  const w = parseNumber(parts[0], NaN);
  const h = parseNumber(parts[1], NaN);
  return w > 0 && h > 0 ? w / h : null;
}

/** Parses geometry sizes. Missing or bad sizes get defaults. Unknown kind: `null`. */
export function parseArgs(kind, value) {
  const defaults = Object.hasOwn(GEOMETRIES, kind) ? GEOMETRIES[kind] : null;
  if (!defaults) return null;
  const parts = typeof value === "string" ? value.split(",") : [];
  return defaults.map((d, i) => {
    const n = parseNumber(parts[i], NaN);
    return n >= 0 ? n : d;
  });
}

/** Resolves a model URL against `base`. Only `http:` and `https:` pass. */
export function parseModelUrl(value, base) {
  if (typeof value !== "string" || value.trim() === "") return null;
  try {
    const url = new URL(value.trim(), base);
    return url.protocol === "http:" || url.protocol === "https:" ? url.href : null;
  } catch {
    return null;
  }
}

/** Parses a clip selector: `*` or a clip name. */
export function parseClip(value) {
  if (typeof value !== "string") return null;
  const name = value.trim();
  return name === "" ? null : name;
}

/** Reads position, rotation, scale, and spin. */
function readTransform(child) {
  return {
    position: parseVec3(child.getAttribute(ATTR.position), [0, 0, 0]),
    rotation: parseVec3(child.getAttribute(ATTR.rotation), [0, 0, 0]),
    scale: parseVec3(child.getAttribute(ATTR.scale), [1, 1, 1]),
    spin: parseVec3(child.getAttribute(ATTR.spin), [0, 0, 0]),
  };
}

/** Reads a mesh declaration, or `null` for an unknown kind. */
function readMesh(child) {
  const kind = child.getAttribute(ATTR.mesh).trim().toLowerCase();
  const args = parseArgs(kind, child.getAttribute(ATTR.args));
  if (!args) return null;
  const d = MESH_DEFAULTS;
  return {
    type: "mesh",
    kind,
    args,
    material: parseKeyword(child.getAttribute(ATTR.material), MATERIALS, d.material),
    color: parseColor(child.getAttribute(ATTR.color), d.color),
    emissive: parseColor(child.getAttribute(ATTR.emissive), d.emissive),
    metalness: parseNumber(child.getAttribute(ATTR.metalness), d.metalness, 0, 1),
    roughness: parseNumber(child.getAttribute(ATTR.roughness), d.roughness, 0, 1),
    opacity: parseNumber(child.getAttribute(ATTR.opacity), d.opacity, 0, 1),
    wireframe: child.getAttribute(ATTR.wireframe) === "true",
    ...readTransform(child),
  };
}

/** Reads a model declaration, or `null` for a bad URL. */
function readModel(child, base) {
  const src = parseModelUrl(child.getAttribute(ATTR.model), base);
  if (!src) return null;
  const fit = parseNumber(child.getAttribute(ATTR.fit), NaN);
  return {
    type: "model",
    src,
    fit: fit > 0 ? fit : null,
    clip: parseClip(child.getAttribute(ATTR.clip)),
    ...readTransform(child),
  };
}

/** Reads a light declaration, or `null` for an unknown kind. */
function readLight(child) {
  const kind = child.getAttribute(ATTR.light).trim().toLowerCase();
  const d = Object.hasOwn(LIGHTS, kind) ? LIGHTS[kind] : null;
  if (!d) return null;
  return {
    type: "light",
    kind,
    color: parseColor(child.getAttribute(ATTR.color), 0xffffff),
    ground: parseColor(child.getAttribute(ATTR.ground), d.ground),
    intensity: parseNumber(child.getAttribute(ATTR.intensity), d.intensity, 0),
    position: parseVec3(child.getAttribute(ATTR.position), d.position),
  };
}

/**
 * Reads a scene element into a plain config object.
 *
 * `el` needs `getAttribute` and `children`. Each child needs
 * `getAttribute` and `hasAttribute`. Bad declarations are skipped and
 * named in `warnings`.
 */
export function readScene(el, base) {
  const warnings = [];
  const objects = [];
  for (const child of el.children ?? []) {
    let object;
    let attr;
    if (child.hasAttribute(ATTR.mesh)) {
      attr = ATTR.mesh;
      object = readMesh(child);
    } else if (child.hasAttribute(ATTR.model)) {
      attr = ATTR.model;
      object = readModel(child, base);
    } else if (child.hasAttribute(ATTR.light)) {
      attr = ATTR.light;
      object = readLight(child);
    } else {
      continue;
    }
    if (object) {
      objects.push(object);
    } else {
      warnings.push(`ignored ${attr}="${child.getAttribute(attr)}"`);
    }
  }
  const env = el.getAttribute(ATTR.environment);
  return {
    aspect: parseAspect(el.getAttribute(ATTR.aspect)),
    background: parseColor(el.getAttribute(ATTR.background), null),
    camera: {
      fov: parseNumber(el.getAttribute(ATTR.fov), CAMERA_DEFAULTS.fov, 1, 179),
      position: parseVec3(el.getAttribute(ATTR.camera), CAMERA_DEFAULTS.position),
      target: parseVec3(el.getAttribute(ATTR.target), CAMERA_DEFAULTS.target),
    },
    controls: parseKeyword(el.getAttribute(ATTR.controls), CONTROLS, CONTROLS[0]),
    environment: parseKeyword(env, ENVIRONMENTS, null),
    turntable: parseNumber(el.getAttribute(ATTR.turntable), 0),
    reducedAnimate: el.getAttribute(ATTR.reduced) === REDUCED_ANIMATE,
    objects,
    warnings,
  };
}
