import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

test("packed tarball installs offline, imports without DOM and supplies usable declarations", async () => {
  const directory = await mkdtemp(join(tmpdir(), "minecraft-map-package-"));
  const root = fileURLToPath(new URL("..", import.meta.url));
  try {
    const [packed] = JSON.parse(execFileSync("npm", [
      "pack", "--ignore-scripts", "--json", "--pack-destination", directory,
    ], { cwd: root, encoding: "utf8" }));
    assert.deepEqual(packed.files.map((file) => file.path).sort(), [
      "README.md", "dist/index.d.ts", "dist/index.js",
      "dist/protocol.d.ts", "dist/protocol.js", "package.json",
    ]);
    const consumer = join(directory, "consumer");
    await mkdir(consumer);
    await writeFile(join(consumer, "package.json"), JSON.stringify({ private: true, type: "module" }));
    execFileSync("npm", [
      "install", "--ignore-scripts", "--offline", "--no-audit", "--no-fund",
      join(directory, packed.filename),
    ], { cwd: consumer, stdio: "pipe" });
    execFileSync(process.execPath, ["--input-type=module", "--eval", `
      import assert from "node:assert/strict";
      import { Map, MapErrorEvent } from "minecraft-map-renderer";
      assert.equal(typeof Map, "function");
      assert.equal(typeof MapErrorEvent, "function");
      const viewer = new Map("https://example.invalid/world");
      assert.deepEqual(viewer.view, { x: 0, y: 0, scale: 1 });
      viewer.destroy();
    `], { cwd: consumer, stdio: "pipe" });
    await writeFile(join(consumer, "consumer.ts"), `
      import { Map, MapErrorEvent, type RenderOptions, type View } from "minecraft-map-renderer";
      const viewer = new Map("/map");
      const options: RenderOptions = {
        initialPosition: { x: -16, y: 32 }, initialScale: 2,
        scaleRange: { min: 1, max: 4 }, maxConcurrentRequests: 2, maxCachedTiles: 4,
      };
      const view: View = viewer.view;
      viewer.setView(view, 2);
      viewer.addEventListener("error", (event) => {
        if (event instanceof MapErrorEvent) console.error(event.error.message);
      });
      void viewer.render(document.createElement("canvas"), options);
      // @ts-expect-error Scales must be numeric.
      viewer.setView({ x: 0, y: 0 }, "2");
      // @ts-expect-error Position requires a Y/world-Z coordinate.
      viewer.setView({ x: 0 });
    `);
    execFileSync(process.execPath, [
      resolve(root, "node_modules/typescript/bin/tsc"),
      "--noEmit", "--strict", "--target", "ES2022", "--module", "NodeNext", "consumer.ts",
    ], { cwd: consumer, stdio: "pipe" });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
