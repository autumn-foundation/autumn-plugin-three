// autumn-plugin-three runtime.
//
// Finds [data-three="scene"] elements and builds a Three.js scene for each.
// Scans on load, on htmx:afterSwap, and on any DOM insertion. Disposes a
// scene when its element leaves the document. One element owns at most
// one renderer.
//
// All imports are relative, so the module graph needs no import map and
// works under `script-src 'self'`.

import * as THREE from "./three.module.min.js";
import { ATTR, readScene } from "./parse.js";

const SCENE = `[${ATTR.scene}="scene"]`;
const DEG = Math.PI / 180;
const MAX_PIXEL_RATIO = 2;
const MAX_DELTA_SECONDS = 0.1;
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");

/** Live scenes: element → state. */
const live = new Map();

/** Returns a function that calls `load` once and caches the promise. */
const once = (load) => {
  let promise;
  return () => (promise ??= load());
};

// Addons load only when a scene needs them.
const loadOrbit = once(() => import("./OrbitControls.js"));
const loadGltf = once(() => import("./GLTFLoader.js"));
const loadRoom = once(() => import("./RoomEnvironment.js"));

/** Geometry constructors. Arguments come from parse.js (validated sizes). */
const GEOMETRY = {
  "box": ([w, h, d]) => new THREE.BoxGeometry(w, h, d),
  "sphere": ([r]) => new THREE.SphereGeometry(r, 48, 32),
  "plane": ([w, h]) => new THREE.PlaneGeometry(w, h),
  "torus": ([r, t]) => new THREE.TorusGeometry(r, t, 32, 96),
  "torus-knot": ([r, t]) => new THREE.TorusKnotGeometry(r, t, 160, 24),
  "cylinder": ([a, b, h]) => new THREE.CylinderGeometry(a, b, h, 48),
  "cone": ([r, h]) => new THREE.ConeGeometry(r, h, 48),
  "capsule": ([r, l]) => new THREE.CapsuleGeometry(r, l, 8, 24),
  "icosahedron": ([r]) => new THREE.IcosahedronGeometry(r),
  "dodecahedron": ([r]) => new THREE.DodecahedronGeometry(r),
  "octahedron": ([r]) => new THREE.OctahedronGeometry(r),
  "tetrahedron": ([r]) => new THREE.TetrahedronGeometry(r),
  "ring": ([i, o]) => new THREE.RingGeometry(i, o, 48),
};

/** Flat geometries show both sides. */
const DOUBLE_SIDED = new Set(["plane", "ring"]);

/** Makes the material of a mesh declaration. */
function makeMaterial(o) {
  const base = {
    wireframe: o.wireframe,
    transparent: o.opacity < 1,
    opacity: o.opacity,
    side: DOUBLE_SIDED.has(o.kind) ? THREE.DoubleSide : THREE.FrontSide,
  };
  const colored = { ...base, color: o.color };
  const lit = { ...colored, emissive: o.emissive };
  const pbr = { ...lit, metalness: o.metalness, roughness: o.roughness };
  switch (o.material) {
    case "basic": return new THREE.MeshBasicMaterial(colored);
    case "normal": return new THREE.MeshNormalMaterial(base);
    case "lambert": return new THREE.MeshLambertMaterial(lit);
    case "phong": return new THREE.MeshPhongMaterial(lit);
    case "physical": return new THREE.MeshPhysicalMaterial(pbr);
    default: return new THREE.MeshStandardMaterial(pbr);
  }
}

/** Makes a light from a light declaration. */
function makeLight(o) {
  let light;
  switch (o.kind) {
    case "ambient": return new THREE.AmbientLight(o.color, o.intensity);
    case "hemisphere": light = new THREE.HemisphereLight(o.color, o.ground, o.intensity); break;
    case "point": light = new THREE.PointLight(o.color, o.intensity); break;
    case "spot": light = new THREE.SpotLight(o.color, o.intensity, 0, Math.PI / 6, 0.3); break;
    default: light = new THREE.DirectionalLight(o.color, o.intensity);
  }
  light.position.fromArray(o.position);
  return light;
}

/** Applies position, rotation (degrees), and scale. */
function place(object, o) {
  object.position.fromArray(o.position);
  object.rotation.set(o.rotation[0] * DEG, o.rotation[1] * DEG, o.rotation[2] * DEG);
  object.scale.fromArray(o.scale);
}

/** Registers `object` to spin when its spin is not zero. */
function addSpin(state, object, spin) {
  if (spin.some((v) => v !== 0)) {
    object.userData.spin = spin;
    state.spinners.push(object);
  }
}

/** Sends a bubbling event from `el`. */
function emit(el, type, detail) {
  el.dispatchEvent(new CustomEvent(type, { bubbles: true, detail }));
}

/** Marks `el` as failed: fallback shows, `three:error` fires. */
function fail(el, error, src) {
  console.warn("autumn-plugin-three:", src ?? "", error);
  el.setAttribute(ATTR.state, "error");
  emit(el, "three:error", { error, src });
}

/** True when automatic motion is allowed for this scene. */
function motionAllowed(state) {
  return state.reducedAnimate || !reducedMotion.matches;
}

/** True when the scene has automatic motion. */
function hasMotion(state) {
  return state.turntable !== 0 || state.spinners.length > 0 || state.mixers.length > 0;
}

/** Renders one frame now. */
function renderNow(state) {
  if (!state.disposed && state.renderer) state.renderer.render(state.scene, state.camera);
}

/** Renders one frame on the next animation frame, unless the loop runs. */
function requestRender(state) {
  if (state.looping || state.frame || state.disposed) return;
  state.frame = requestAnimationFrame(() => {
    state.frame = 0;
    renderNow(state);
  });
}

/** One loop step: advance motion by the time delta, then render. */
function tick(state, time) {
  const dt = Math.min(Math.max((time - state.last) / 1000, 0), MAX_DELTA_SECONDS);
  state.last = time;
  state.root.rotation.y += state.turntable * DEG * dt;
  for (const object of state.spinners) {
    const [x, y, z] = object.userData.spin;
    object.rotation.x += x * DEG * dt;
    object.rotation.y += y * DEG * dt;
    object.rotation.z += z * DEG * dt;
  }
  for (const mixer of state.mixers) mixer.update(dt);
  renderNow(state);
}

/** Runs the loop only while the scene is ready, visible, and has allowed motion. */
function updateLoop(state) {
  const run = !state.disposed && state.ready && state.visible && hasMotion(state) && motionAllowed(state);
  if (run === state.looping) return;
  state.looping = run;
  state.last = performance.now();
  state.renderer.setAnimationLoop(run ? (time) => tick(state, time) : null);
  if (!run) requestRender(state);
}

/** Matches the drawing buffer and camera to the element size. */
function resize(state) {
  const width = Math.max(1, Math.round(state.el.clientWidth));
  const height = Math.max(1, Math.round(state.el.clientHeight));
  state.renderer.setSize(width, height, false);
  state.camera.aspect = width / height;
  state.camera.updateProjectionMatrix();
  renderNow(state);
}

/** Frees the GPU resources of `object` and its children. */
function disposeTree(object) {
  object.traverse((child) => {
    child.geometry?.dispose();
    for (const material of [].concat(child.material ?? [])) {
      for (const value of Object.values(material)) {
        if (value?.isTexture) value.dispose();
      }
      material.dispose();
    }
  });
}

/** Stops and frees a scene. The element shows its fallback again. */
function dispose(state) {
  if (state.disposed) return;
  state.disposed = true;
  live.delete(state.el);
  cancelAnimationFrame(state.frame);
  for (const observer of state.observers) observer.disconnect();
  state.controls?.dispose();
  for (const mixer of state.mixers) mixer.stopAllAction();
  if (state.renderer) {
    state.renderer.setAnimationLoop(null);
    disposeTree(state.scene);
    state.scene.environment?.dispose();
    state.renderer.dispose();
    state.renderer.forceContextLoss();
  }
  state.canvas.remove();
  delete state.el.autumnThree;
  state.el.setAttribute(ATTR.state, "disposed");
}

/** Adds a loaded glTF to the scene: fit, transform, spin, clips. */
function addModel(state, gltf, o) {
  if (state.disposed) {
    disposeTree(gltf.scene);
    return;
  }
  const fitted = new THREE.Group();
  fitted.add(gltf.scene);
  if (o.fit) {
    const box = new THREE.Box3().setFromObject(fitted);
    const size = box.getSize(new THREE.Vector3());
    const largest = Math.max(size.x, size.y, size.z);
    if (largest > 0) {
      const scale = o.fit / largest;
      fitted.scale.setScalar(scale);
      fitted.position.copy(box.getCenter(new THREE.Vector3())).multiplyScalar(-scale);
    }
  }
  const wrapper = new THREE.Group();
  wrapper.add(fitted);
  place(wrapper, o);
  addSpin(state, wrapper, o.spin);
  if (o.clip && gltf.animations.length) {
    const clips = o.clip === "*"
      ? gltf.animations
      : [THREE.AnimationClip.findByName(gltf.animations, o.clip)].filter(Boolean);
    if (clips.length) {
      const mixer = new THREE.AnimationMixer(gltf.scene);
      for (const clip of clips) mixer.clipAction(clip).play();
      state.mixers.push(mixer);
    } else {
      console.warn(`autumn-plugin-three: no clip "${o.clip}" in ${o.src}`);
    }
  }
  state.models.push(wrapper);
  state.root.add(wrapper);
}

/** Adds image-based light from the room environment. */
async function addRoom(state) {
  const { RoomEnvironment } = await loadRoom();
  if (state.disposed) return;
  const pmrem = new THREE.PMREMGenerator(state.renderer);
  const room = new RoomEnvironment();
  state.scene.environment = pmrem.fromScene(room, 0.04).texture;
  room.dispose();
  pmrem.dispose();
}

/** Adds orbit controls around the camera target. */
async function addControls(state, mode, target) {
  const { OrbitControls } = await loadOrbit();
  if (state.disposed) return;
  const controls = new OrbitControls(state.camera, state.canvas);
  controls.target.copy(target);
  controls.enableZoom = mode === "orbit";
  controls.update();
  controls.addEventListener("change", () => requestRender(state));
  state.controls = controls;
}

/** Loads all models. Returns the first failure, or `null`. */
async function addModels(state, models) {
  if (models.length === 0) return null;
  const { GLTFLoader } = await loadGltf();
  if (state.disposed) return null;
  const loader = new GLTFLoader();
  const results = await Promise.allSettled(
    models.map((o) => loader.loadAsync(o.src).then((gltf) => addModel(state, gltf, o))),
  );
  const index = results.findIndex((r) => r.status === "rejected");
  return index < 0 ? null : { error: results[index].reason, src: models[index].src };
}

/** Builds the scene for `el`. */
async function build(el) {
  const config = readScene(el, document.baseURI);
  for (const warning of config.warnings) console.warn(`autumn-plugin-three: ${warning}`);
  const state = {
    el,
    canvas: document.createElement("canvas"),
    renderer: null,
    scene: new THREE.Scene(),
    root: new THREE.Group(),
    camera: new THREE.PerspectiveCamera(config.camera.fov, 1, 0.01, 1000),
    controls: null,
    models: [],
    mixers: [],
    spinners: [],
    observers: [],
    turntable: config.turntable,
    reducedAnimate: config.reducedAnimate,
    ready: false,
    visible: false,
    looping: false,
    disposed: false,
    frame: 0,
    last: 0,
  };
  live.set(el, state);
  el.setAttribute(ATTR.state, "loading");
  if (config.aspect) el.style.aspectRatio = String(config.aspect);
  // Markup restored from an htmx history snapshot can hold an old canvas.
  for (const stale of el.querySelectorAll(`:scope > canvas[${ATTR.canvas}]`)) stale.remove();

  try {
    state.renderer = new THREE.WebGLRenderer({
      canvas: state.canvas,
      antialias: true,
      alpha: config.background === null,
    });
  } catch (error) {
    fail(el, error, null);
    return;
  }
  const { renderer, scene, root, camera, canvas } = state;
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, MAX_PIXEL_RATIO));
  renderer.setClearColor(config.background ?? 0x000000, config.background === null ? 0 : 1);
  canvas.setAttribute(ATTR.canvas, "");
  canvas.setAttribute("aria-hidden", "true");
  el.append(canvas);

  scene.add(root);
  camera.position.fromArray(config.camera.position);
  const target = new THREE.Vector3().fromArray(config.camera.target);
  camera.lookAt(target);

  for (const o of config.objects) {
    if (o.type === "mesh") {
      const mesh = new THREE.Mesh(GEOMETRY[o.kind](o.args), makeMaterial(o));
      place(mesh, o);
      addSpin(state, mesh, o.spin);
      root.add(mesh);
    } else if (o.type === "light") {
      scene.add(makeLight(o));
    }
  }
  if (!config.environment && !config.objects.some((o) => o.type === "light")) {
    scene.add(new THREE.HemisphereLight(0xffffff, 0x444444, 1.5));
    const sun = new THREE.DirectionalLight(0xffffff, 2);
    sun.position.set(3, 5, 4);
    scene.add(sun);
  }

  const [, , failure] = await Promise.all([
    config.environment === "room" ? addRoom(state) : null,
    config.controls !== "none" ? addControls(state, config.controls, target) : null,
    addModels(state, config.objects.filter((o) => o.type === "model")),
  ]);
  if (state.disposed) return;

  if (failure && el.querySelector(`:scope > [${ATTR.fallback}]`)) {
    // A fallback exists: show it and free the GPU.
    dispose(state);
    fail(el, failure.error, failure.src);
    return;
  }

  const resizeObserver = new ResizeObserver(() => resize(state));
  const viewObserver = new IntersectionObserver((entries) => {
    state.visible = entries[entries.length - 1].isIntersecting;
    updateLoop(state);
  });
  resizeObserver.observe(el);
  viewObserver.observe(el);
  state.observers.push(resizeObserver, viewObserver);
  resize(state);

  el.autumnThree = {
    THREE,
    scene,
    root,
    camera,
    renderer,
    controls: state.controls,
    models: state.models,
    mixers: state.mixers,
    render: () => renderNow(state),
    requestRender: () => requestRender(state),
    get looping() {
      return state.looping;
    },
  };
  state.ready = true;
  updateLoop(state);
  if (failure) {
    // No fallback: keep the other objects on screen.
    fail(el, failure.error, failure.src);
  } else {
    el.setAttribute(ATTR.state, "ready");
    emit(el, "three:ready", el.autumnThree);
  }
}

/** Scene elements in `node`, including `node` itself. */
function scenesIn(node) {
  if (node.nodeType !== Node.ELEMENT_NODE && node.nodeType !== Node.DOCUMENT_NODE) return [];
  const found = [...node.querySelectorAll(SCENE)];
  if (node.matches?.(SCENE)) found.unshift(node);
  return found;
}

/** Builds every new connected scene in `node`. */
function scan(node) {
  for (const el of scenesIn(node)) {
    if (live.has(el) || !el.isConnected) continue;
    build(el).catch((error) => {
      const state = live.get(el);
      if (state) dispose(state);
      fail(el, error, null);
    });
  }
}

/** Disposes every scene in `node` that is no longer in the document. */
function sweep(node) {
  for (const el of scenesIn(node)) {
    const state = live.get(el);
    if (state && !el.isConnected) dispose(state);
  }
}

new MutationObserver((records) => {
  for (const record of records) {
    for (const node of record.removedNodes) sweep(node);
    for (const node of record.addedNodes) scan(node);
  }
}).observe(document.documentElement, { childList: true, subtree: true });

document.addEventListener("htmx:afterSwap", (event) => {
  const target = event.detail?.target;
  scan(target?.isConnected ? target : document);
});

document.addEventListener("htmx:beforeCleanupElement", (event) => {
  const state = live.get(event.target);
  if (state) dispose(state);
});

reducedMotion.addEventListener("change", () => {
  for (const state of live.values()) if (state.ready) updateLoop(state);
});

scan(document);
