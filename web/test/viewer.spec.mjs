import { randomUUID } from "node:crypto";
import { test, expect } from "@playwright/test";
import { fixture } from "./fixtures.mjs";

const unhandled = new WeakMap();
test.beforeEach(({ page }) => {
  const errors = [];
  unhandled.set(page, errors);
  page.on("pageerror", (error) => errors.push(error.message));
});
test.afterEach(({ page }) => {
  expect(unhandled.get(page)).toEqual([]);
});

async function mount(page, scenario = "basic", options = {}, width = 16, height = 16) {
  const id = randomUUID();
  await page.goto("/");
  await page.evaluate(async ({ scenario, id, options, width, height }) => {
    const { Map } = await import("/dist/index.js");
    const canvas = document.querySelector("canvas");
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    window.mapErrors = [];
    window.loads = 0;
    window.changes = 0;
    window.testMap = new Map(`/fixtures/${scenario}/${id}`);
    testMap.addEventListener("error", (event) => mapErrors.push(event.error.message));
    testMap.addEventListener("load", () => window.loads++);
    testMap.addEventListener("viewchange", () => window.changes++);
    await testMap.render(canvas, { initialPosition: { x: 8, y: 8 }, ...options });
  }, { scenario, id, options, width, height });
  return id;
}

async function ready(page) {
  await expect.poll(() => page.evaluate(() => window.loads)).toBeGreaterThan(0);
}

async function pixel(page, x = 8, y = 8) {
  return page.evaluate(({ x, y }) => {
    const context = document.querySelector("canvas").getContext("2d");
    return [...context.getImageData(Math.floor(x * devicePixelRatio), Math.floor(y * devicePixelRatio), 1, 1).data];
  }, { x, y });
}

async function state(request, id) {
  return (await request.get(`/stats/${id}`)).json();
}

test("loads only visible tiles, converts signed ARGB and draws negative coordinates and void", async ({ page, request }) => {
  const id = await mount(page, "basic", {}, 48, 16);
  await ready(page);
  expect(await pixel(page, 8)).toEqual([255, 0, 0, 255]);
  expect(await pixel(page, 24)).toEqual([0, 255, 0, 255]);
  expect(await pixel(page, 40)).toEqual([0, 0, 255, 255]);
  const stats = await state(request, id);
  expect(Object.keys(stats.requests)).toHaveLength(3);
  const remote = fixture("basic").metadata.tiles.find((tile) => tile.x === 10);
  expect(stats.requests[`${remote.hash}.mpt`]).toBeUndefined();
  await page.evaluate(() => testMap.setView({ x: 80, y: 8 }));
  await expect.poll(() => pixel(page)).toEqual([0, 0, 0, 0]);
  await page.evaluate(() => testMap.setView({ x: 1e300, y: 8 }));
  await expect.poll(() => pixel(page)).toEqual([0, 0, 0, 0]);
  expect(await page.evaluate(() => mapErrors)).toEqual([]);
});

test("uses the indexed tile size rather than hardcoding 2048", async ({ page }) => {
  await mount(page, "large", { initialPosition: { x: 16, y: 16 } }, 32, 32);
  await ready(page);
  expect(await pixel(page, 30, 30)).toEqual([0, 255, 0, 255]);
});

test("uses nearest-neighbor scaling without introducing blended pixel colors", async ({ page }) => {
  await mount(page, "pixels", { initialScale: 4 }, 64, 64);
  await ready(page);
  const row = await page.evaluate(() => [
    ...document.querySelector("canvas").getContext("2d").getImageData(0, 32, 64, 1).data,
  ]);
  for (let x = 0; x < 64; x++) {
    expect(row.slice(x * 4, x * 4 + 4)).toEqual(
      Math.floor(x / 4) & 1 ? [0, 0, 255, 255] : [255, 0, 0, 255],
    );
  }
});

test("deduplicates identical hashes shared by visible tile coordinates", async ({ page, request }) => {
  const id = await mount(page, "shared", { initialPosition: { x: 16, y: 8 } }, 32, 16);
  await ready(page);
  expect(await pixel(page, 4)).toEqual([0, 255, 0, 255]);
  expect(await pixel(page, 28)).toEqual([0, 255, 0, 255]);
  expect(Object.values((await state(request, id)).requests)).toEqual([1]);
});

test("limits concurrency without evicting tiles still visible", async ({ page, request }) => {
  const id = await mount(page, "many", {
    initialPosition: { x: 64, y: 8 }, maxConcurrentRequests: 2, maxCachedTiles: 1,
  }, 128, 16);
  await ready(page);
  const stats = await state(request, id);
  expect(stats.peak).toBe(2);
  expect(Object.values(stats.requests)).toEqual(Array(8).fill(1));
  expect(await pixel(page, 4)).toEqual([255, 0, 0, 255]);
  expect((await pixel(page, 124))[3]).toBe(255);
});

test("retains recent offscreen tiles and evicts least-recently-used tiles", async ({ page, request }) => {
  const id = await mount(page, "basic", { maxCachedTiles: 1 });
  await ready(page);
  await page.evaluate(() => testMap.setView({ x: 24, y: 8 }));
  await expect.poll(() => pixel(page)).toEqual([0, 0, 255, 255]);
  await page.evaluate(() => testMap.setView({ x: 8, y: 8 }));
  await expect.poll(() => pixel(page)).toEqual([0, 255, 0, 255]);
  const green = fixture("basic").metadata.tiles.find((tile) => tile.x === 0);
  expect((await state(request, id)).requests[`${green.hash}.mpt`]).toBe(2);
  // A larger cache can satisfy a round trip without a second request.
  await page.evaluate(() => testMap.destroy());
  const retained = await mount(page, "basic", { maxCachedTiles: 2 });
  await ready(page);
  await page.evaluate(() => testMap.setView({ x: 24, y: 8 }));
  await expect.poll(() => pixel(page)).toEqual([0, 0, 255, 255]);
  await page.evaluate(() => testMap.setView({ x: 8, y: 8 }));
  await expect.poll(() => pixel(page)).toEqual([0, 255, 0, 255]);
  expect((await state(request, retained)).requests[`${green.hash}.mpt`]).toBe(1);
});

test("cancels obsolete requests, drops queued work and can re-enter the same area", async ({ page, request }) => {
  const id = await mount(page, "slow", {
    initialPosition: { x: 64, y: 8 }, maxConcurrentRequests: 2,
  }, 128, 16);
  await expect.poll(async () => (await state(request, id))?.active ?? 0).toBe(2);
  await page.evaluate(() => testMap.setView({ x: 1000, y: 8 }));
  await expect.poll(async () => (await state(request, id)).aborted).toBe(2);
  expect(Object.keys((await state(request, id)).requests)).toHaveLength(2);
  await page.evaluate(() => testMap.setView({ x: 64, y: 8 }));
  await expect.poll(() => pixel(page, 4)).toEqual([255, 0, 0, 255]);
  await expect.poll(async () => (await state(request, id)).active).toBe(0);
  expect(await page.evaluate(() => mapErrors)).toEqual([]);
});

test("zooms around the cursor, clamps scale, pans with pointer capture and reports view changes", async ({ page }) => {
  await mount(page, "basic", {}, 32, 16);
  await ready(page);
  await page.mouse.move(22, 8);
  await page.mouse.wheel(0, -300);
  await expect.poll(() => page.evaluate(() => testMap.view.scale)).toBeGreaterThan(1);
  const zoomed = await page.evaluate(() => testMap.view);
  expect(zoomed.x + 6 / zoomed.scale).toBeCloseTo(14, 10);
  expect(zoomed.y).toBe(8);
  await page.mouse.move(16, 8);
  await page.mouse.down();
  await page.mouse.move(56, 8);
  await page.mouse.up();
  const panned = await page.evaluate(() => testMap.view);
  expect(panned.x).toBeCloseTo(zoomed.x - 40 / zoomed.scale, 10);
  await page.mouse.move(22, 8);
  await page.mouse.wheel(0, -10000);
  await expect.poll(() => page.evaluate(() => testMap.view.scale)).toBe(4);
  await page.mouse.wheel(0, 10000);
  await expect.poll(() => page.evaluate(() => testMap.view.scale)).toBe(1);
  await page.mouse.down();
  await page.mouse.wheel(0, -100);
  await expect.poll(() => page.evaluate(() => testMap.view.scale)).toBeGreaterThan(1);
  const duringDrag = await page.evaluate(() => testMap.view);
  await page.mouse.move(24, 8);
  await page.mouse.up();
  expect((await page.evaluate(() => testMap.view)).x).toBeCloseTo(duringDrag.x - 2 / duringDrag.scale, 10);
  expect(await page.evaluate(() => changes)).toBeGreaterThan(2);
});

test("movable=false disables drag and a fixed scale range disables wheel zoom", async ({ page }) => {
  await mount(page, "basic", { movable: false, scaleRange: { min: 1, max: 1 } }, 32, 16);
  await ready(page);
  await page.mouse.move(8, 8);
  await page.mouse.down();
  await page.mouse.move(24, 8);
  await page.mouse.up();
  await page.mouse.wheel(0, -300);
  expect(await page.evaluate(() => testMap.view)).toEqual({ x: 8, y: 8, scale: 1 });
  expect(await page.evaluate(() => changes)).toBe(0);
});

for (const [scenario, message] of [["http-error", "HTTP 503"], ["corrupt", "SHA-256 mismatch"]]) {
  test(`notifies and visibly marks ${scenario} rather than silently showing void`, async ({ page, request }) => {
    const id = await mount(page, scenario);
    await expect.poll(() => page.evaluate(() => mapErrors.length)).toBe(1);
    expect((await page.evaluate(() => mapErrors))[0]).toContain(message);
    await expect.poll(async () => (await pixel(page, 4, 8))[3]).toBe(255);
    expect(Object.values((await state(request, id)).requests)).toEqual([1]);
    await page.evaluate(() => testMap.setView({ x: 200, y: 8 }));
    await expect.poll(() => pixel(page)).toEqual([0, 0, 0, 0]);
    await page.evaluate(() => testMap.setView({ x: 8, y: 8 }));
    await expect.poll(() => page.evaluate(() => mapErrors.length)).toBe(2);
  });
}

test("rejects initialization errors, invalid options, repeated render and invalid setView", async ({ page }) => {
  await expect(mount(page, "bad-index")).rejects.toThrow(/Unsupported index.map format/);
  await expect(mount(page, "basic", { scaleRange: { min: 4, max: 1 } })).rejects.toThrow(/Invalid position/);
  await expect(mount(page, "basic", { initialScale: 8 })).rejects.toThrow(/Invalid position/);
  await mount(page);
  await ready(page);
  await expect(page.evaluate(() => testMap.render(document.querySelector("canvas")))).rejects.toThrow(/only once/);
  await expect(page.evaluate(() => testMap.setView({ x: Infinity, y: 0 }))).rejects.toThrow(/Invalid view/);
  await expect(page.evaluate(() => testMap.setView({ x: 0, y: 0 }, 0))).rejects.toThrow(/Invalid view/);
});

test("empty indexes stay transparent without any tile requests", async ({ page }) => {
  const tiles = [];
  page.on("request", (request) => { if (request.url().endsWith(".mpt")) tiles.push(request.url()); });
  await mount(page, "empty");
  await ready(page);
  expect(await pixel(page)).toEqual([0, 0, 0, 0]);
  expect(tiles).toEqual([]);
});

test("destroy aborts requests, removes handlers, restores canvas and is idempotent", async ({ page, request }) => {
  const id = await mount(page, "slow", { maxConcurrentRequests: 1 });
  await expect.poll(async () => (await state(request, id))?.active ?? 0).toBe(1);
  await page.evaluate(() => { testMap.destroy(); testMap.destroy(); });
  await expect.poll(async () => (await state(request, id)).aborted).toBe(1);
  await page.mouse.move(8, 8);
  await page.mouse.wheel(0, -100);
  await page.mouse.down();
  await page.mouse.move(12, 8);
  await page.mouse.up();
  expect(await page.evaluate(() => changes)).toBe(0);
  expect(await page.evaluate(() => mapErrors)).toEqual([]);
  expect(await page.evaluate(() => document.querySelector("canvas").style.touchAction)).toBe("");
  await expect(page.evaluate(() => testMap.setView({ x: 0, y: 0 }))).rejects.toThrow(/destroyed/);
});

test.describe("high-DPI layout", () => {
  test.use({ deviceScaleFactor: 2 });

  test("resizes backing store without smoothing pixels or resetting the camera", async ({ page }) => {
    await mount(page);
    await ready(page);
    expect(await page.locator("canvas").getAttribute("width")).toBe("32");
    expect(await pixel(page)).toEqual([0, 255, 0, 255]);
    await page.evaluate(() => { document.querySelector("canvas").style.width = "48px"; });
    await expect.poll(() => pixel(page, 8)).toEqual([255, 0, 0, 255]);
    expect(await pixel(page, 24)).toEqual([0, 255, 0, 255]);
    expect(await pixel(page, 40)).toEqual([0, 0, 255, 255]);
    expect(await page.locator("canvas").getAttribute("width")).toBe("96");
    expect(await page.evaluate(() => testMap.view)).toEqual({ x: 8, y: 8, scale: 1 });
  });

  test("keeps intrinsic canvas dimensions stable when no CSS size is provided", async ({ page }) => {
    await page.goto("/");
    await page.evaluate(async () => {
      document.querySelector("style").remove();
      const { Map } = await import("/dist/index.js");
      window.testMap = new Map("/fixtures/empty/intrinsic");
      await testMap.render(document.querySelector("canvas"));
      await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    });
    expect(await page.locator("canvas").getAttribute("width")).toBe("32");
    expect(await page.locator("canvas").evaluate((canvas) => canvas.clientWidth)).toBe(16);
    await page.evaluate(() => testMap.destroy());
    expect(await page.locator("canvas").getAttribute("width")).toBe("16");
    expect(await page.locator("canvas").evaluate((canvas) => canvas.style.width)).toBe("");
  });
});

test("renders Rust output against all native reference RGBA pixels and coverage entries", async ({ page }) => {
  test.skip(!process.env.MINECRAFT_TEST_MAP || !process.env.MINECRAFT_TEST_WORLD, "Set native map/world paths to enable");
  await page.goto("/");
  const result = await page.evaluate(async () => {
    const { Map } = await import("/dist/index.js");
    const { decodeTile, parseIndex } = await import("/dist/protocol.js");
    const canvas = document.querySelector("canvas");
    canvas.style.width = "2048px";
    canvas.style.height = "2048px";
    const viewer = new Map("/real");
    const begin = performance.now();
    const loaded = new Promise((resolve, reject) => {
      viewer.addEventListener("load", resolve, { once: true });
      viewer.addEventListener("error", (event) => reject(event.error), { once: true });
    });
    await viewer.render(canvas, { initialPosition: { x: 1024, y: 1024 } });
    await loaded;
    const firstVisibleMs = performance.now() - begin;
    const metadata = parseIndex(await (await fetch("/real/index.map")).json());
    const reference = new Uint8Array(await (await fetch("/reference.colors")).arrayBuffer());
    const chunks = new Uint8Array(await (await fetch("/reference.chunks")).arrayBuffer());
    const pixels = canvas.getContext("2d").getImageData(0, 0, 2048, 2048).data;
    let colorDifferences = 0;
    for (let i = 0; i < reference.length; i++) {
      const color = metadata.palette[reference[i]];
      if (
        pixels[i * 4] !== ((color >>> 16) & 255) ||
        pixels[i * 4 + 1] !== ((color >>> 8) & 255) ||
        pixels[i * 4 + 2] !== (color & 255) ||
        pixels[i * 4 + 3] !== (color >>> 24)
      ) colorDifferences++;
    }
    const coverage = new Uint8Array(128 * 128);
    for (const tile of metadata.tiles) {
      const size = metadata.tileSize;
      if (tile.x * size >= 2048 || tile.z * size >= 2048 || (tile.x + 1) * size <= 0 || (tile.z + 1) * size <= 0) continue;
      const buffer = await (await fetch(`/real/${tile.hash}.mpt`)).arrayBuffer();
      const decoded = await decodeTile(buffer, size, tile.hash);
      const side = size / 16;
      for (let z = 0; z < side; z++) {
        for (let x = 0; x < side; x++) {
          const worldX = tile.x * side + x, worldZ = tile.z * side + z;
          if (worldX < 0 || worldZ < 0 || worldX >= 128 || worldZ >= 128) continue;
          const bit = z * side + x;
          coverage[worldZ * 128 + worldX] = (decoded.coverage[bit >> 3] >> (bit & 7)) & 1;
        }
      }
    }
    let coverageDifferences = 0;
    for (let i = 0; i < chunks.length; i++) {
      if (coverage[i] !== Number(chunks[i] === 2)) coverageDifferences++;
    }
    viewer.destroy();
    return { colors: reference.length, chunks: chunks.length, colorDifferences, coverageDifferences, firstVisibleMs };
  });
  expect(result.colors).toBe(4194304);
  expect(result.chunks).toBe(16384);
  expect(result.colorDifferences).toBe(0);
  expect(result.coverageDifferences).toBe(0);
  console.log(`Native browser comparison: ${JSON.stringify(result)}`);
});
