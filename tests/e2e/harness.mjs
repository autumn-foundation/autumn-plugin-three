// E2E harness: starts the e2e_fixture app and a headless Chromium.
//
// The fixture binary must exist: `cargo build --example e2e_fixture`.
// Chromium uses SwiftShader, so WebGL works without a GPU.
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createServer } from "node:net";
import { chromium } from "playwright";

const BINARY = new URL("../../target/debug/examples/e2e_fixture", import.meta.url).pathname;

/** Returns a free TCP port. */
function freePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.unref();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

/** Polls `url` until it answers. */
async function waitForHttp(url, child, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`fixture exited with ${child.exitCode}`);
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Not up yet.
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`fixture did not start: ${url}`);
}

/**
 * Merges V8 block coverage of plugin files. A byte counts as run when the
 * innermost range that holds it has a count above zero, in any page.
 */
const coverage = new Map(); // file name → { source, run: Uint8Array }

function addCoverage(entries) {
  for (const entry of entries) {
    const name = entry.url.match(/\/static\/_plugins\/three\/([a-z]+)(?:\.[0-9a-f]{8})?\.js$/i)?.[1];
    if (!name || !["init", "parse"].includes(name) || !entry.source) continue;
    const file = coverage.get(name) ?? { source: entry.source, run: new Uint8Array(entry.source.length) };
    coverage.set(name, file);
    const local = new Int8Array(entry.source.length).fill(-1);
    // Outer ranges first; inner ranges overwrite them.
    const ranges = entry.functions.flatMap((f) => f.ranges).sort((a, b) => a.startOffset - b.startOffset || b.endOffset - a.endOffset);
    for (const r of ranges) local.fill(r.count > 0 ? 1 : 0, r.startOffset, r.endOffset);
    local.forEach((v, i) => v === 1 && (file.run[i] = 1));
  }
}

/**
 * Strict line coverage of a plugin file: `{ percent, uncovered }`. A code
 * line counts as run only when every non-space byte on it ran. Comment and
 * closing-bracket lines do not count.
 */
export function lineCoverage(name) {
  const file = coverage.get(name);
  if (!file) return null;
  let offset = 0;
  let code = 0;
  let inComment = false;
  const uncovered = [];
  file.source.split("\n").forEach((line, index) => {
    const text = line.trim();
    const comment = inComment || text.startsWith("//") || text.startsWith("/*") || text.startsWith("*");
    if (text.startsWith("/*")) inComment = !text.includes("*/");
    else if (inComment && text.includes("*/")) inComment = false;
    if (text !== "" && !comment && !/^[})\];,]+$/.test(text)) {
      code += 1;
      for (let i = 0; i < line.length; i += 1) {
        if (line[i] !== " " && !file.run[offset + i]) {
          uncovered.push(index + 1);
          break;
        }
      }
    }
    offset += line.length + 1;
  });
  return { percent: (100 * (code - uncovered.length)) / code, uncovered };
}

/** Records events, CSP violations, and errors in every page. */
const RECORDER = () => {
  window.__events = [];
  window.__csp = [];
  window.__warnings = [];
  window.__details = [];
  for (const type of ["three:ready", "three:error"]) {
    document.addEventListener(type, (e) => {
      window.__events.push([type, e.target.id, e.detail?.src ?? null]);
      window.__details.push(type === "three:ready" ? e.detail === e.target.autumnThree : e.detail?.error instanceof Error);
    }, true);
  }
  document.addEventListener("securitypolicyviolation", (e) => window.__csp.push(`${e.violatedDirective} ${e.blockedURI}`));
  const warn = console.warn.bind(console);
  console.warn = (...args) => {
    window.__warnings.push(args.map(String).join(" "));
    warn(...args);
  };
};

/**
 * Starts the fixture and the browser. `env` adds environment variables.
 * The harness writes `toml` to `autumn.toml` in a temp dir
 * (AUTUMN_MANIFEST_DIR).
 * Returns `{ base, open, close }`.
 */
export async function start({ env = {}, toml = null } = {}) {
  if (toml !== null) {
    const dir = mkdtempSync(join(tmpdir(), "three-e2e-"));
    writeFileSync(join(dir, "autumn.toml"), toml);
    env = { ...env, AUTUMN_MANIFEST_DIR: dir };
  }
  if (!existsSync(BINARY)) {
    throw new Error(`missing ${BINARY}: run cargo build --example e2e_fixture`);
  }
  const port = await freePort();
  const child = spawn(BINARY, [], {
    env: { ...process.env, AUTUMN_SERVER__PORT: String(port), AUTUMN_SERVER__HOST: "127.0.0.1", ...env },
    stdio: ["ignore", "ignore", "pipe"],
  });
  let stderr = "";
  child.stderr.on("data", (d) => (stderr += d));
  const base = `http://127.0.0.1:${port}`;
  let browser;
  try {
    await waitForHttp(`${base}/basic`, child);
    browser = await chromium.launch({
      args: ["--enable-unsafe-swiftshader", "--use-angle=swiftshader", "--ignore-gpu-blocklist"],
    });
  } catch (error) {
    // Kill the fixture, or it keeps the test process alive.
    child.kill();
    throw new Error(`${error.message}\n${stderr}`);
  }
  const contexts = [];
  const pages = [];

  /**
   * Opens `path` in a fresh context. Options: Playwright context options,
   * plus `init` (a function to run before page scripts).
   */
  async function open(path, { init, ...options } = {}) {
    const context = await browser.newContext({ viewport: { width: 800, height: 600 }, ...options });
    contexts.push(context);
    const page = await context.newPage();
    if (options.javaScriptEnabled !== false) {
      await page.coverage.startJSCoverage({ resetOnNavigation: false });
      pages.push(page);
    }
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
    await page.addInitScript(RECORDER);
    if (init) await page.addInitScript(init);
    await page.goto(`${base}${path}`);
    page.errors = errors;
    return page;
  }

  async function close() {
    for (const page of pages) {
      if (!page.isClosed()) addCoverage(await page.coverage.stopJSCoverage().catch(() => []));
    }
    for (const context of contexts) await context.close().catch(() => {});
    await browser.close();
    child.kill();
  }

  return { base, open, close };
}

/** Waits until `#id` has `data-three-state` equal to `state`. */
export async function waitState(page, id, state, timeout = 15_000) {
  await page.waitForFunction(
    ([id, state]) => document.getElementById(id)?.getAttribute("data-three-state") === state,
    [id, state],
    { timeout },
  );
}

/** Renders `#id` now and reads the RGBA pixel at canvas fraction (fx, fy). */
export async function pixel(page, id, fx = 0.5, fy = 0.5) {
  return page.evaluate(
    ([id, fx, fy]) => {
      const handle = document.getElementById(id).autumnThree;
      handle.render();
      const gl = handle.renderer.getContext();
      const x = Math.floor(gl.drawingBufferWidth * fx);
      const y = Math.floor(gl.drawingBufferHeight * (1 - fy));
      const out = new Uint8Array(4);
      gl.readPixels(x, y, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);
      return [...out];
    },
    [id, fx, fy],
  );
}

/** Waits `ms` milliseconds. */
export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
