import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { fixture } from "./fixtures.mjs";

const states = new Map();
const html = '<!doctype html><html><head><style>body{margin:0}canvas{display:block;width:16px;height:16px}</style></head><body><canvas id="map" width="16" height="16"></canvas></body></html>';
const realMap = process.env.MINECRAFT_TEST_MAP && resolve(process.env.MINECRAFT_TEST_MAP);
const realWorld = process.env.MINECRAFT_TEST_WORLD && resolve(process.env.MINECRAFT_TEST_WORLD);

const server = createServer(async (request, response) => {
  const path = new URL(request.url, "http://localhost").pathname;
  try {
    if (path === "/") {
      response.writeHead(200, { "Content-Type": "text/html" }).end(html);
    } else if (/^\/dist\/[a-z-]+\.js$/.test(path)) {
      response.writeHead(200, { "Content-Type": "text/javascript" });
      response.end(await readFile(new URL(`..${path}`, import.meta.url)));
    } else if (path.startsWith("/stats/")) {
      response.writeHead(200, { "Content-Type": "application/json" }).end(JSON.stringify(states.get(path.slice(7)) ?? null));
    } else if (/^\/fixtures\/[a-z-]+\/[a-z0-9-]+\/(index\.map|[0-9a-f]{64}\.mpt)$/.test(path)) {
      const [, , scenario, id, file] = path.split("/");
      const data = fixture(scenario);
      if (file === "index.map") {
        response.writeHead(200, { "Content-Type": "application/json" }).end(JSON.stringify(data.metadata));
        return;
      }
      let state = states.get(id);
      if (!state) {
        state = { active: 0, peak: 0, aborted: 0, requests: {} };
        states.set(id, state);
      }
      state.active++;
      state.peak = Math.max(state.peak, state.active);
      state.requests[file] = (state.requests[file] ?? 0) + 1;
      let timer;
      response.on("close", () => {
        state.active--;
        if (!response.writableEnded) state.aborted++;
        clearTimeout(timer);
      });
      timer = setTimeout(() => {
        if (scenario === "http-error") {
          response.writeHead(503).end("Fixture unavailable");
        } else if (data.files.has(file)) {
          response.writeHead(200, { "Content-Type": "application/octet-stream", "Cache-Control": "no-store" }).end(data.files.get(file));
        } else {
          response.writeHead(404).end("Unknown fixture tile");
        }
      }, scenario === "slow" ? 300 : scenario === "many" ? 60 : 0);
    } else if (realMap && /^\/real\/(index\.map|[0-9a-f]{64}\.mpt)$/.test(path)) {
      response.writeHead(200, { "Content-Type": path.endsWith(".map") ? "application/json" : "application/octet-stream" });
      response.end(await readFile(join(realMap, path.slice(6))));
    } else if (realWorld && /^\/reference\.(colors|chunks)$/.test(path)) {
      response.writeHead(200, { "Content-Type": "application/octet-stream" });
      response.end(await readFile(join(realWorld, path.slice(1))));
    } else {
      response.writeHead(404).end("Not found");
    }
  } catch (error) {
    console.error(error);
    if (!response.headersSent) response.writeHead(500);
    response.end("Test server error");
  }
});
server.listen(4174, "127.0.0.1");
