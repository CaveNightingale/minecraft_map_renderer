export interface Tile {
  readonly x: number;
  readonly z: number;
  readonly hash: string;
}

export interface MapIndex {
  readonly format: 1;
  readonly tileSize: number;
  readonly palette: readonly number[];
  readonly tiles: readonly Tile[];
}

export interface TileData {
  readonly colors: Uint8Array;
  readonly coverage: Uint8Array;
}

function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function parseIndex(value: unknown): MapIndex {
  if (!object(value) || value.format !== 1) {
    throw new Error("Unsupported index.map format; expected format 1");
  }
  const size = value.tileSize;
  if (
    typeof size !== "number" ||
    !Number.isSafeInteger(size) ||
    size <= 0 ||
    size % 16 !== 0 ||
    !Number.isSafeInteger(size * size)
  ) {
    throw new Error("Invalid tileSize: expected a positive multiple of 16 with a safe integer pixel count");
  }
  if (
    !Array.isArray(value.palette) ||
    value.palette.length !== 256 ||
    !Array.from(value.palette).every(
      (color: unknown) =>
        typeof color === "number" &&
        Number.isInteger(color) &&
        color >= -2147483648 &&
        color <= 2147483647,
    )
  ) {
    throw new Error("Invalid palette: expected 256 signed 32-bit ARGB values");
  }
  const palette: number[] = value.palette;
  if (palette.slice(0, 4).some((color) => (color >>> 24) !== 0)) {
    throw new Error("Invalid palette: color IDs 0 through 3 must be transparent");
  }
  if (!Array.isArray(value.tiles)) {
    throw new Error("Invalid index.map tiles list");
  }
  const coordinates = new Set<string>();
  const tiles: Tile[] = value.tiles.map((tile: unknown) => {
    if (
      !object(tile) ||
      typeof tile.x !== "number" ||
      typeof tile.z !== "number" ||
      !Number.isSafeInteger(tile.x) ||
      !Number.isSafeInteger(tile.z) ||
      ![tile.x, tile.z].every(
        (coordinate) =>
          Number.isSafeInteger(coordinate * size) &&
          Number.isSafeInteger((coordinate + 1) * size),
      ) ||
      typeof tile.hash !== "string" ||
      !/^[0-9a-f]{64}$/.test(tile.hash)
    ) {
      throw new Error("Invalid tile coordinates or SHA-256 hash in index.map");
    }
    const key = `${tile.x},${tile.z}`;
    if (coordinates.has(key)) {
      throw new Error(`Duplicate tile coordinates ${key}`);
    }
    coordinates.add(key);
    return { x: tile.x, z: tile.z, hash: tile.hash };
  });
  return { format: 1, tileSize: size, palette: [...palette], tiles };
}

async function inflate(payload: Uint8Array): Promise<Uint8Array> {
  if (typeof DecompressionStream === "undefined") {
    throw new Error("Tile decompression requires the DecompressionStream (gzip) API");
  }
  const stream = new Blob([payload as Uint8Array<ArrayBuffer>])
    .stream()
    .pipeThrough(new DecompressionStream("gzip"));
  try {
    return new Uint8Array(await new Response(stream).arrayBuffer());
  } catch (error) {
    throw new Error("Invalid tile gzip payload", { cause: error });
  }
}

export async function decodeTile(
  buffer: ArrayBuffer,
  tileSize: number,
  hash: string,
): Promise<TileData> {
  const pixels = tileSize * tileSize;
  const chunksPerRow = tileSize / 16;
  const chunks = chunksPerRow * chunksPerRow;
  const coverageLength = Math.ceil(chunks / 8);
  const view = new DataView(buffer);
  const magic = [0x4d, 0x50, 0x54, 2, 0, 0, 0, 0];
  if (!Number.isSafeInteger(pixels) || tileSize <= 0 || tileSize % 16 !== 0 || buffer.byteLength <= 16) {
    throw new Error("Invalid tile file length or dimensions");
  }
  if (
    magic.some((byte, offset) => view.getUint8(offset) !== byte) ||
    view.getBigUint64(8, true) !== BigInt(tileSize)
  ) {
    throw new Error("Invalid tile magic, version or size");
  }
  if (!globalThis.crypto?.subtle) {
    throw new Error("SHA-256 verification requires HTTPS or localhost (Web Crypto)");
  }
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", buffer));
  const actualHash = Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join("");
  if (actualHash !== hash) {
    throw new Error(`Tile SHA-256 mismatch: expected ${hash}, got ${actualHash}`);
  }
  const payload = await inflate(new Uint8Array(buffer, 16));
  if (payload.length !== pixels + coverageLength) {
    throw new Error("Invalid tile file length or dimensions");
  }
  const colors = payload.subarray(0, pixels);
  const coverage = payload.subarray(pixels);
  const remainder = chunks % 8;
  if (remainder !== 0 && (coverage[coverageLength - 1]! >>> remainder) !== 0) {
    throw new Error("Nonzero unused chunk coverage bits");
  }
  for (let z = 0; z < tileSize; z++) {
    for (let x = 0; x < tileSize; x++) {
      const chunk = Math.floor(z / 16) * chunksPerRow + Math.floor(x / 16);
      if (!(coverage[chunk >>> 3]! & (1 << (chunk & 7))) && colors[z * tileSize + x] !== 0) {
        throw new Error(`Nonzero color in uncovered chunk at tile pixel ${x},${z}`);
      }
    }
  }
  return { colors, coverage };
}
