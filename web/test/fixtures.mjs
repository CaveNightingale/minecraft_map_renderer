import { createHash } from "node:crypto";
import { gzipSync } from "node:zlib";

export const palette = Array(256).fill(0);
palette[4] = 0xffff0000 | 0;
palette[5] = 0xff00ff00 | 0;
palette[6] = 0xff0000ff | 0;
palette[7] = 0xffffff00 | 0;
for (let i = 8; i < 256; i++) palette[i] = (0xff000000 | (i * 7919 & 0xffffff));

export function encodeTile(size, colors, coverage) {
  const header = Buffer.alloc(16);
  header.set([0x4d, 0x50, 0x54, 2, 0, 0, 0, 0]);
  header.writeBigUInt64LE(BigInt(size), 8);
  return Buffer.concat([header, gzipSync(Buffer.concat([colors, coverage]))]);
}

export function tile(size = 16, color = 5, covered = true) {
  const colors = Buffer.alloc(size * size, color);
  const coverage = Buffer.alloc(Math.ceil((size / 16) ** 2 / 8));
  if (covered) {
    for (let i = 0; i < (size / 16) ** 2; i++) coverage[i >> 3] |= 1 << (i & 7);
  }
  return encodeTile(size, colors, coverage);
}

export function hash(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

export function index(size = 16, tiles = []) {
  return { format: 1, tileSize: size, palette: [...palette], tiles };
}

export function fixture(name) {
  const size = name === "large" ? 32 : 16;
  const files = new Map();
  const tiles = [];
  const add = (x, color) => {
    let data;
    if (name === "pixels") {
      const colors = Buffer.alloc(size * size);
      for (let i = 0; i < size * size; i++) colors[i] = i & 1 ? 6 : 4;
      const coverage = Buffer.alloc(Math.ceil((size / 16) ** 2 / 8));
      for (let i = 0; i < (size / 16) ** 2; i++) coverage[i >> 3] |= 1 << (i & 7);
      data = encodeTile(size, colors, coverage);
    } else {
      data = tile(size, color);
    }
    const digest = hash(data);
    files.set(`${digest}.mpt`, data);
    tiles.push({ x, z: 0, hash: digest });
  };
  if (name === "many" || name === "slow") {
    for (let x = 0; x < 8; x++) add(x, x + 4);
  } else if (name === "shared") {
    add(0, 5);
    add(1, 5);
  } else if (name === "basic") {
    add(-1, 4);
    add(0, 5);
    add(1, 6);
    add(10, 7);
  } else if (name !== "empty") {
    add(0, 5);
  }
  const metadata = index(size, tiles);
  if (name === "bad-index") metadata.format = 2;
  if (name === "corrupt") {
    for (const data of files.values()) data[16] ^= 1;
  }
  return { metadata, files };
}
