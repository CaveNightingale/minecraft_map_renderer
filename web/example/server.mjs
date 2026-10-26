import { createServer } from "node:http";
import { readFile, access } from "node:fs/promises";
import { resolve, join } from "node:path";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    map: { type: "string", default: "../target/reference-map" },
    port: { type: "string", default: "4173" },
  },
});
const port = Number(values.port);
if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error("Invalid --port");
const mapDirectory = resolve(values.map);
await access(join(mapDirectory, "index.map"));

const server = createServer(async (request, response) => {
  if (request.method !== "GET" && request.method !== "HEAD") {
    response.writeHead(405, { Allow: "GET, HEAD" }).end();
    return;
  }
  const path = new URL(request.url, "http://localhost").pathname;
  let file;
  let type;
  let cache = "no-cache";
  if (path === "/") {
    file = new URL("./index.html", import.meta.url);
    type = "text/html; charset=utf-8";
  } else if (/^\/dist\/[a-z-]+\.js$/.test(path)) {
    file = new URL(`..${path}`, import.meta.url);
    type = "text/javascript; charset=utf-8";
  } else if (/^\/map\/(index\.map|[0-9a-f]{64}\.mpt)$/.test(path)) {
    file = join(mapDirectory, path.slice("/map/".length));
    type = path.endsWith(".mpt") ? "application/octet-stream" : "application/json";
    if (path.endsWith(".mpt")) cache = "public, max-age=31536000, immutable";
  } else {
    response.writeHead(404).end("Not found");
    return;
  }
  try {
    const data = await readFile(file);
    response.writeHead(200, { "Content-Type": type, "Cache-Control": cache });
    response.end(request.method === "HEAD" ? undefined : data);
  } catch (error) {
    if (error.code === "ENOENT") {
      response.writeHead(404).end("Not found");
    } else {
      console.error(error);
      response.writeHead(500).end("Cannot read file");
    }
  }
});
server.listen(port, "127.0.0.1", () => {
  console.log(`Map: ${mapDirectory}\nOpen http://127.0.0.1:${port}`);
});
