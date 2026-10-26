import { test } from "node:test";
import assert from "node:assert/strict";
import { decodeTile, parseIndex } from "../dist/protocol.js";
import { encodeTile, hash, index, palette, tile } from "./fixtures.mjs";

const arrayBuffer = (buffer) => Uint8Array.from(buffer).buffer;

test("parses minimal index, signed ARGB, negative coordinates and shared content", () => {
  const digest = hash(tile());
  const metadata = index(16, [{ x: -1, z: -2, hash: digest }, { x: 0, z: 0, hash: digest }]);
  assert.deepEqual(parseIndex(metadata), metadata);
  assert.deepEqual(parseIndex(index()).tiles, []);
  metadata.palette[4] = 0;
  assert.equal(palette[4], -65536);
});

test("rejects unknown versions, unsafe sizes/coordinates, malformed hashes and duplicates", () => {
  const entry = { x: 0, z: 0, hash: hash(tile()) };
  const invalid = [
    null, [], { ...index(), format: 2 },
    ...[0, -16, 15, 17, Infinity, 94906272].map((tileSize) => ({ ...index(), tileSize })),
    index(16, [{ ...entry, x: 0.5 }]),
    index(16, [{ ...entry, x: Number.MAX_SAFE_INTEGER }]),
    index(16, [{ ...entry, hash: "../unsafe" }]),
    index(16, [entry, entry]),
    { ...index(), tiles: {} },
  ];
  for (const value of invalid) assert.throws(() => parseIndex(value), Error);
});

test("rejects invalid palette length, signed range, holes and opaque void colors", () => {
  for (const bad of [
    [], Array(256), [...palette.slice(0, 255), 0x80000000],
    [...palette.slice(0, 255), 1.5], [palette[4], ...palette.slice(1)],
  ]) {
    assert.throws(() => parseIndex({ ...index(), palette: bad }), /palette/);
  }
});

test("decodes variable-sized tiles and coverage, including all unsigned color IDs", async () => {
  for (const size of [16, 32, 64]) {
    const colors = Buffer.alloc(size * size);
    for (let i = 0; i < size * size; i++) colors[i] = i & 255;
    const chunks = (size / 16) ** 2;
    const coverage = Buffer.alloc(Math.ceil(chunks / 8));
    for (let i = 0; i < chunks; i++) coverage[i >> 3] |= 1 << (i & 7);
    const data = encodeTile(size, colors, coverage);
    const decoded = await decodeTile(arrayBuffer(data), size, hash(data));
    assert.deepEqual(decoded.colors, new Uint8Array(colors));
    assert.equal(decoded.coverage.length, Math.ceil((size / 16) ** 2 / 8));
  }
  const empty = tile(16, 0, false);
  assert.ok((await decodeTile(arrayBuffer(empty), 16, hash(empty))).colors.every((color) => color === 0));
});

test("rejects truncation, trailing bytes, wrong magic, size, version and content hash", async () => {
  const good = tile();
  for (const length of [0, 8]) {
    await assert.rejects(decodeTile(new ArrayBuffer(length), 16, hash(good)), /length/);
  }
  for (const length of [good.length - 1, good.length + 1]) {
    const awkward = Buffer.alloc(length);
    good.copy(awkward, 0, 0, Math.min(length, good.length));
    await assert.rejects(decodeTile(arrayBuffer(awkward), 16, hash(awkward)), /length|gzip/);
  }
  for (const offset of [0, 3, 4, 8]) {
    const data = Buffer.from(good);
    data[offset] ^= 1;
    await assert.rejects(decodeTile(arrayBuffer(data), 16, hash(data)), /magic, version or size/);
  }
  const damaged = Buffer.from(good);
  damaged[16] ^= 1;
  await assert.rejects(decodeTile(arrayBuffer(damaged), 16, hash(good)), /SHA-256 mismatch/);
});

test("rejects uncovered terrain and reserved coverage bits even with a valid hash", async () => {
  const terrain = tile(16, 5, false);
  await assert.rejects(decodeTile(arrayBuffer(terrain), 16, hash(terrain)), /uncovered chunk/);
  const colors = Buffer.alloc(16 * 16, 5);
  const coverage = Buffer.from([0x81]);
  const reserved = encodeTile(16, colors, coverage);
  await assert.rejects(decodeTile(arrayBuffer(reserved), 16, hash(reserved)), /unused chunk coverage/);
});
